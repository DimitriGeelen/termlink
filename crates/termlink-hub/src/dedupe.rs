//! T-2049 Gap A — hub-side LRU dedupe for client_msg_id idempotency.
//!
//! Closes the double-apply scenario described in T-2023 §4.A:
//!
//! 1. Spoke sends `channel.post topic X payload P` with `client_msg_id=K`.
//! 2. Hub commits the envelope at offset N, but the TCP response is lost
//!    BEFORE the spoke sees the ack (RST / network blip / hub bounce).
//! 3. Spoke retries from its offline queue → hub receives the SAME K
//!    from the same sender_id.
//! 4. Without dedupe: hub writes again at offset N+1 → subscribers see P
//!    twice with different offsets.
//! 5. With dedupe: hub recognises the (sender_id, K) pair, no-ops the
//!    append, and returns the CACHED `{offset: N, ts}` envelope to the
//!    client. Looks like success on the retrying side; substrate stays
//!    exactly-once.
//!
//! Design:
//!
//! * Key = `(sender_id: String, client_msg_id: String)` — sender_id is the
//!   identity fingerprint already verified by `handle_channel_post_with`
//!   (T-1427 invariant), so an attacker can't poison another sender's
//!   dedupe namespace.
//! * Value = `DedupeEntry { state: Pending | Committed{offset, ts}, seen_at_ms }`
//!   — `Pending` is the T-2435 pre-reservation that closes the concurrent
//!   double-post TOCTOU (both racers seeing a miss and both appending).
//! * Eviction = TTL-based (default 5 min) + capacity-bounded LRU (default
//!   10K entries). TTL keeps the cache small in steady state; LRU is the
//!   floor under pathological burst-of-distinct-ids load.
//! * Time = injected (`now_ms: i64`) for deterministic tests, matching the
//!   pattern established in [`crate::governor`] (T-2048).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Default time-to-live for a dedupe entry, milliseconds. Five minutes
/// chosen as the upper bound on realistic spoke reconnect windows:
/// a hub bounce or network blip that takes longer than this is no longer
/// a "lost ack on the same call", it's a fresh post and double-apply
/// risk is gone.
pub const DEFAULT_DEDUPE_TTL_MS: i64 = 300_000;

/// Default maximum number of distinct (sender_id, client_msg_id) entries
/// held in memory. 10K is comfortably above realistic burst-of-distinct
/// load (30 senders × 300 unique posts in a 5-min window) and bounds
/// memory at ~1 MB.
pub const DEFAULT_DEDUPE_CAPACITY: usize = 10_000;

static POST_DEDUPE: OnceLock<PostDedupe> = OnceLock::new();

/// Read env-var overrides and install the process-global dedupe cache.
///
/// Idempotent — calling more than once preserves the first install (matches
/// `OnceLock` semantics). Called from `run_with_tcp` and `run_blocking` at
/// hub startup; tests that need different limits call directly.
pub fn init() {
    let ttl_ms = parse_env_i64("TERMLINK_DEDUPE_TTL_MS", DEFAULT_DEDUPE_TTL_MS);
    let capacity = parse_env_usize("TERMLINK_DEDUPE_CAPACITY", DEFAULT_DEDUPE_CAPACITY);
    let _ = POST_DEDUPE.set(PostDedupe::new(ttl_ms, capacity));
    tracing::info!(
        dedupe_ttl_ms = ttl_ms,
        dedupe_capacity = capacity,
        "Hub post-dedupe active (T-2049 — TERMLINK_DEDUPE_TTL_MS / TERMLINK_DEDUPE_CAPACITY to override)"
    );
}

/// Access the global `PostDedupe`. Lazy fallback to defaults if [`init`]
/// was never called (test paths often skip explicit init).
pub fn post_dedupe() -> &'static PostDedupe {
    POST_DEDUPE.get_or_init(|| PostDedupe::new(DEFAULT_DEDUPE_TTL_MS, DEFAULT_DEDUPE_CAPACITY))
}

fn parse_env_i64(name: &str, default: i64) -> i64 {
    match std::env::var(name) {
        Ok(v) => v.parse::<i64>().unwrap_or_else(|_| {
            tracing::warn!(env = name, value = %v, default, "Hub dedupe: env var unparseable as i64, using default");
            default
        }),
        Err(_) => default,
    }
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    match std::env::var(name) {
        Ok(v) => v.parse::<usize>().unwrap_or_else(|_| {
            tracing::warn!(env = name, value = %v, default, "Hub dedupe: env var unparseable as usize, using default");
            default
        }),
        Err(_) => default,
    }
}

/// Lifecycle of a dedupe entry (T-2435). A key is `Pending` from the
/// moment the first call reserves it until `record_offset` promotes it
/// after `bus.post` succeeds; a failed post releases the reservation via
/// [`PostDedupe::abort_reservation`] (or TTL ages it out if the caller
/// crashes mid-post).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryState {
    /// Reserved — the original post is still in flight, no offset yet.
    Pending,
    /// Committed — the original post succeeded at this offset/ts.
    Committed { offset: u64, ts_unix_ms: i64 },
}

/// One cached post — what the hub returns on a duplicate hit so the
/// retrying client sees the same response shape as the first call.
#[derive(Debug, Clone, Copy)]
struct DedupeEntry {
    state: EntryState,
    seen_at_ms: i64,
}

/// Outcome of `try_record_or_lookup`. `Newly` means the caller must
/// proceed with `bus.post` and then call [`PostDedupe::record_offset`]
/// on success or [`PostDedupe::abort_reservation`] on failure;
/// `Duplicate` means the caller must return the cached envelope without
/// appending; `InFlight` means a concurrent call holds the reservation
/// and the caller must refuse loudly with a retryable error (T-2435 —
/// never fall through to `bus.post`, that is the double-apply TOCTOU).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupeOutcome {
    /// First time seeing this (sender_id, client_msg_id) pair. The entry
    /// IS pre-reserved (`Pending`, T-2435) — the caller MUST follow up
    /// with `record_offset` (success) or `abort_reservation` (failure).
    Newly,
    /// Already-seen pair. The cached `(offset, ts_unix_ms)` of the
    /// original successful post is returned verbatim.
    Duplicate { offset: u64, ts_unix_ms: i64 },
    /// The original post with this pair is still in flight on another
    /// connection/task — no offset exists yet. Caller returns a
    /// structured retryable refusal (`POST_IN_FLIGHT`); the retrying
    /// spoke will hit `Duplicate` once the first post commits.
    InFlight,
}

/// Process-global recently-seen-posts cache.
pub struct PostDedupe {
    ttl_ms: i64,
    capacity: usize,
    map: Mutex<HashMap<(String, String), DedupeEntry>>,
    hits_total: AtomicU64,
}

impl PostDedupe {
    /// Build a new dedupe cache. `ttl_ms` and `capacity` must both be > 0;
    /// the function clamps zero to 1 to avoid div-by-zero / empty-cache
    /// edge cases that would render the dedupe useless.
    pub fn new(ttl_ms: i64, capacity: usize) -> Self {
        let ttl_ms = ttl_ms.max(1);
        let capacity = capacity.max(1);
        Self {
            ttl_ms,
            capacity,
            map: Mutex::new(HashMap::new()),
            hits_total: AtomicU64::new(0),
        }
    }

    /// Look up (sender_id, client_msg_id). On hit, return
    /// [`DedupeOutcome::Duplicate`] with the cached offset/ts AND
    /// increment `hits_total`. On miss, pre-reserve the slot with a
    /// placeholder entry, evicting expired and oldest entries as
    /// necessary, and return [`DedupeOutcome::Newly`] so the caller
    /// proceeds with the actual post.
    pub fn try_record_or_lookup(
        &self,
        sender_id: &str,
        client_msg_id: &str,
        now_ms: i64,
    ) -> DedupeOutcome {
        let key = (sender_id.to_string(), client_msg_id.to_string());
        let mut map = self.map.lock().expect("dedupe mutex poisoned");

        // Lazy TTL eviction — only when we're about to mutate.
        evict_expired_in(&mut map, now_ms, self.ttl_ms);

        if let Some(entry) = map.get(&key) {
            // Hit. Don't update seen_at_ms — the cache is keyed on the
            // first sighting so TTL anchors to the original post, not
            // the retry. (A spoke that retries every 30s for 4 minutes
            // would otherwise hold the entry forever.) Both hit kinds
            // count toward hits_total: each absorbs a would-be
            // double-append.
            self.hits_total.fetch_add(1, Ordering::Relaxed);
            return match entry.state {
                EntryState::Committed { offset, ts_unix_ms } => {
                    DedupeOutcome::Duplicate { offset, ts_unix_ms }
                }
                EntryState::Pending => DedupeOutcome::InFlight,
            };
        }

        // Miss. Pre-reserve under the SAME lock acquisition (T-2435) so
        // concurrent retries of the same pair collide on the entry
        // instead of both reaching `bus.post` — the double-apply TOCTOU
        // this cache exists to prevent. The caller MUST follow up with
        // `record_offset` (success) or `abort_reservation` (post
        // failed); a crashed caller's orphan reservation ages out by
        // TTL.
        evict_for_capacity_in(&mut map, self.capacity, &key);
        map.insert(
            key,
            DedupeEntry {
                state: EntryState::Pending,
                seen_at_ms: now_ms,
            },
        );
        DedupeOutcome::Newly
    }

    /// Release a `Pending` reservation after the guarded `bus.post`
    /// failed (T-2435). Removes the entry ONLY if it is still Pending —
    /// a Committed entry is never dropped here (that would re-open the
    /// sequential-retry double-apply window).
    pub fn abort_reservation(&self, sender_id: &str, client_msg_id: &str) {
        let key = (sender_id.to_string(), client_msg_id.to_string());
        let mut map = self.map.lock().expect("dedupe mutex poisoned");
        if let Some(entry) = map.get(&key)
            && entry.state == EntryState::Pending
        {
            map.remove(&key);
        }
    }

    /// Record the cached `{offset, ts_unix_ms}` for a successful post.
    /// Called by `handle_channel_post_with` after `bus.post` returns
    /// `Ok(offset)`. Evicts expired and (if at capacity) the oldest
    /// entry by `seen_at_ms`.
    pub fn record_offset(
        &self,
        sender_id: &str,
        client_msg_id: &str,
        now_ms: i64,
        offset: u64,
        ts_unix_ms: i64,
    ) {
        let key = (sender_id.to_string(), client_msg_id.to_string());
        let mut map = self.map.lock().expect("dedupe mutex poisoned");

        evict_expired_in(&mut map, now_ms, self.ttl_ms);
        evict_for_capacity_in(&mut map, self.capacity, &key);

        // Promote the Pending reservation (normal path) — or insert
        // directly for callers that record without a prior reservation
        // (kept for back-compat; the T-2435 handler always reserves).
        map.insert(
            key,
            DedupeEntry {
                state: EntryState::Committed { offset, ts_unix_ms },
                seen_at_ms: now_ms,
            },
        );
    }

    /// Evict expired entries (`now_ms - seen_at_ms > ttl_ms`). Called
    /// implicitly on every mutation; exposed for the future eviction
    /// housekeeping loop.
    pub fn evict_expired(&self, now_ms: i64) {
        let mut map = self.map.lock().expect("dedupe mutex poisoned");
        evict_expired_in(&mut map, now_ms, self.ttl_ms);
    }

    /// Current number of entries in the cache. Used by
    /// `hub.governor_status` for observability.
    pub fn entries_active(&self) -> u64 {
        self.map.lock().expect("dedupe mutex poisoned").len() as u64
    }

    /// Monotonic counter of duplicate-hit events. Used by
    /// `hub.governor_status` for observability.
    pub fn hits_total(&self) -> u64 {
        self.hits_total.load(Ordering::Relaxed)
    }

    /// Configured TTL in milliseconds. Used by `hub.governor_status`.
    pub fn ttl_ms(&self) -> i64 {
        self.ttl_ms
    }
}

fn evict_expired_in(
    map: &mut HashMap<(String, String), DedupeEntry>,
    now_ms: i64,
    ttl_ms: i64,
) {
    map.retain(|_, entry| now_ms.saturating_sub(entry.seen_at_ms) <= ttl_ms);
}

/// LRU eviction when at capacity and the incoming key is not already
/// present — find the oldest by `seen_at_ms`. O(n) but only fires when
/// the cache is full, which TTL keeps rare. Shared by the reservation
/// path (`try_record_or_lookup`, T-2435) and `record_offset`.
fn evict_for_capacity_in(
    map: &mut HashMap<(String, String), DedupeEntry>,
    capacity: usize,
    incoming: &(String, String),
) {
    if map.len() >= capacity && !map.contains_key(incoming) {
        if let Some(oldest_key) = map
            .iter()
            .min_by_key(|(_, v)| v.seen_at_ms)
            .map(|(k, _)| k.clone())
        {
            map.remove(&oldest_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_hit_returns_cached_offset() {
        let d = PostDedupe::new(60_000, 100);
        let sender = "fp_alice";
        let msg_id = "msg-123";

        // First call: miss.
        let outcome = d.try_record_or_lookup(sender, msg_id, 1000);
        assert_eq!(outcome, DedupeOutcome::Newly);

        // Caller succeeds → records.
        d.record_offset(sender, msg_id, 1000, 42, 999);

        // Second call: hit, returns cached.
        let outcome = d.try_record_or_lookup(sender, msg_id, 2000);
        assert_eq!(
            outcome,
            DedupeOutcome::Duplicate {
                offset: 42,
                ts_unix_ms: 999
            }
        );
    }

    #[test]
    fn distinct_sender_no_collision() {
        let d = PostDedupe::new(60_000, 100);
        let msg_id = "msg-X";

        let _ = d.try_record_or_lookup("alice", msg_id, 100);
        d.record_offset("alice", msg_id, 100, 1, 100);

        // Bob with the same msg_id is independent — should miss.
        let outcome = d.try_record_or_lookup("bob", msg_id, 200);
        assert_eq!(outcome, DedupeOutcome::Newly);
    }

    #[test]
    fn distinct_msg_id_no_collision() {
        let d = PostDedupe::new(60_000, 100);
        let sender = "alice";

        let _ = d.try_record_or_lookup(sender, "msg-A", 100);
        d.record_offset(sender, "msg-A", 100, 1, 100);

        // Same sender, different msg_id — should miss.
        let outcome = d.try_record_or_lookup(sender, "msg-B", 200);
        assert_eq!(outcome, DedupeOutcome::Newly);
    }

    #[test]
    fn ttl_eviction_lets_old_entries_through() {
        let d = PostDedupe::new(5_000, 100); // 5s TTL
        let sender = "alice";
        let msg_id = "msg-stale";

        let _ = d.try_record_or_lookup(sender, msg_id, 0);
        d.record_offset(sender, msg_id, 0, 7, 0);
        assert_eq!(d.entries_active(), 1);

        // Within TTL — still a hit.
        let outcome = d.try_record_or_lookup(sender, msg_id, 3_000);
        assert!(matches!(outcome, DedupeOutcome::Duplicate { .. }));

        // Past TTL — evicted, miss. The miss pre-reserves a fresh
        // Pending entry (T-2435), so the map holds exactly that one.
        let outcome = d.try_record_or_lookup(sender, msg_id, 10_000);
        assert_eq!(outcome, DedupeOutcome::Newly);
        assert_eq!(d.entries_active(), 1);
    }

    #[test]
    fn lru_eviction_at_capacity() {
        let d = PostDedupe::new(600_000, 3); // capacity = 3
        // Three distinct entries.
        d.record_offset("s1", "m1", 100, 1, 100);
        d.record_offset("s2", "m2", 200, 2, 200);
        d.record_offset("s3", "m3", 300, 3, 300);
        assert_eq!(d.entries_active(), 3);

        // Fourth entry forces eviction of the oldest (s1/m1).
        d.record_offset("s4", "m4", 400, 4, 400);
        assert_eq!(d.entries_active(), 3);

        // s1/m1 is gone — re-lookup misses.
        let outcome = d.try_record_or_lookup("s1", "m1", 500);
        assert_eq!(outcome, DedupeOutcome::Newly);

        // s4/m4 is present.
        let outcome = d.try_record_or_lookup("s4", "m4", 500);
        assert_eq!(
            outcome,
            DedupeOutcome::Duplicate {
                offset: 4,
                ts_unix_ms: 400
            }
        );
    }

    #[test]
    fn hit_counter_increments_per_duplicate() {
        let d = PostDedupe::new(60_000, 100);
        d.record_offset("alice", "m", 100, 5, 100);
        assert_eq!(d.hits_total(), 0);

        let _ = d.try_record_or_lookup("alice", "m", 200);
        assert_eq!(d.hits_total(), 1);

        let _ = d.try_record_or_lookup("alice", "m", 300);
        assert_eq!(d.hits_total(), 2);

        // Miss does NOT increment.
        let _ = d.try_record_or_lookup("alice", "different", 400);
        assert_eq!(d.hits_total(), 2);
    }

    #[test]
    fn ttl_anchors_to_first_sighting_not_retry() {
        // A spoke that retries every minute for 4 minutes should be
        // deduped on each retry, but the entry expires 5 min after
        // the FIRST post (not 5 min after the last retry).
        let d = PostDedupe::new(5_000, 100); // 5s TTL for test speed
        let sender = "alice";
        let msg_id = "m-retry";

        // First post at t=0.
        let _ = d.try_record_or_lookup(sender, msg_id, 0);
        d.record_offset(sender, msg_id, 0, 99, 0);

        // Retries at t=2s and t=4s — both dedupe-hit.
        let r1 = d.try_record_or_lookup(sender, msg_id, 2_000);
        let r2 = d.try_record_or_lookup(sender, msg_id, 4_000);
        assert!(matches!(r1, DedupeOutcome::Duplicate { .. }));
        assert!(matches!(r2, DedupeOutcome::Duplicate { .. }));

        // Retry at t=6s — TTL elapsed (5s from t=0); should MISS,
        // proving seen_at_ms wasn't refreshed by the retries.
        let r3 = d.try_record_or_lookup(sender, msg_id, 6_000);
        assert_eq!(r3, DedupeOutcome::Newly);
    }

    #[test]
    fn evict_expired_explicit_call() {
        let d = PostDedupe::new(5_000, 100);
        d.record_offset("s1", "m1", 0, 1, 0);
        d.record_offset("s2", "m2", 1_000, 2, 1_000);
        d.record_offset("s3", "m3", 4_000, 3, 4_000);
        assert_eq!(d.entries_active(), 3);

        // Explicit evict at t=6s — only s3 survives (4s < 5s ttl from t=6s? no:
        // 6000 - 4000 = 2000 < 5000, survives; 6000 - 1000 = 5000 == 5000, survives;
        // 6000 - 0 = 6000 > 5000, evicted).
        d.evict_expired(6_000);
        assert_eq!(d.entries_active(), 2);

        d.evict_expired(10_000);
        assert_eq!(d.entries_active(), 0);
    }

    #[test]
    fn in_flight_reservation_blocks_second_caller() {
        // T-2435: the sequential shape of the TOCTOU — caller A reserves,
        // caller B arrives before A commits. B must get InFlight, never
        // Newly (double-post) and never a fabricated Duplicate.
        let d = PostDedupe::new(60_000, 100);
        assert_eq!(d.try_record_or_lookup("a", "m", 100), DedupeOutcome::Newly);
        assert_eq!(
            d.try_record_or_lookup("a", "m", 150),
            DedupeOutcome::InFlight
        );
        // InFlight counts as an absorbed retry.
        assert_eq!(d.hits_total(), 1);

        // A commits — B's retry now sees the committed envelope.
        d.record_offset("a", "m", 200, 7, 180);
        assert_eq!(
            d.try_record_or_lookup("a", "m", 250),
            DedupeOutcome::Duplicate {
                offset: 7,
                ts_unix_ms: 180
            }
        );
    }

    #[test]
    fn abort_reservation_releases_pending_only() {
        let d = PostDedupe::new(60_000, 100);

        // Failed post: reserve → abort → a retry may post again.
        assert_eq!(d.try_record_or_lookup("a", "m", 100), DedupeOutcome::Newly);
        d.abort_reservation("a", "m");
        assert_eq!(d.try_record_or_lookup("a", "m", 200), DedupeOutcome::Newly);

        // Committed entries are never dropped by abort.
        d.record_offset("a", "m", 300, 9, 300);
        d.abort_reservation("a", "m");
        assert!(matches!(
            d.try_record_or_lookup("a", "m", 400),
            DedupeOutcome::Duplicate { offset: 9, .. }
        ));
    }

    #[test]
    fn concurrent_racers_exactly_one_newly() {
        // T-2435 regression armor: N threads race the same key through a
        // barrier — exactly 1 may get Newly, the rest InFlight. Before
        // the pre-reservation fix every racer got Newly and the hub
        // double-appended.
        use std::sync::Arc;
        use std::sync::Barrier;

        let d = Arc::new(PostDedupe::new(60_000, 100));
        let n = 10;
        let barrier = Arc::new(Barrier::new(n));
        let handles: Vec<_> = (0..n)
            .map(|_| {
                let d = Arc::clone(&d);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    d.try_record_or_lookup("racer", "same-msg", 1000)
                })
            })
            .collect();
        let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let newly = outcomes
            .iter()
            .filter(|o| **o == DedupeOutcome::Newly)
            .count();
        let in_flight = outcomes
            .iter()
            .filter(|o| **o == DedupeOutcome::InFlight)
            .count();
        assert_eq!(newly, 1, "exactly one racer may proceed to bus.post");
        assert_eq!(in_flight, n - 1, "all others must be refused loudly");

        // Winner commits; every subsequent retry gets the cached offset.
        d.record_offset("racer", "same-msg", 1100, 42, 1050);
        assert_eq!(
            d.try_record_or_lookup("racer", "same-msg", 1200),
            DedupeOutcome::Duplicate {
                offset: 42,
                ts_unix_ms: 1050
            }
        );
    }

    #[test]
    fn zero_ttl_clamps_to_minimum() {
        let d = PostDedupe::new(0, 0);
        // Should not panic and should accept entries (capacity clamped to 1).
        d.record_offset("alice", "m", 100, 1, 100);
        assert_eq!(d.entries_active(), 1);
        // TTL clamped to 1 — second insert at 100 + 1 forces eviction.
        d.record_offset("bob", "m", 102, 2, 102);
        assert_eq!(d.entries_active(), 1);
    }
}
