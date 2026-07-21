//! T-2048 Track B — hub-side LOUD-refuse governors.
//!
//! Two independent primitives:
//!
//! * [`ConnGovernor`] — global cap on simultaneous connections. Refuses
//!   acquisition past `max`, returning a [`RetryHint`] the accept loop
//!   converts into a `HUB_AT_CAPACITY` (-32019) JSON-RPC error before
//!   closing the socket. Atomic, lock-free.
//!
//! * [`RateGovernor`] — per-sender token bucket. Each sender (peer_addr,
//!   peer_pid, or `params.from`) gets its own bucket sized at
//!   `rate_per_sec` tokens, refilled linearly. Refuses acquisition when
//!   the bucket is empty, returning a [`RetryHint`] the dispatcher
//!   converts into a `RATE_LIMITED` (-32008) JSON-RPC error.
//!
//! Time is injected (`now_ms: i64`) so refill behavior is deterministic
//! under test. Production callers pass
//! `SystemTime::now() ↦ Duration::since(UNIX_EPOCH).as_millis() as i64`.
//!
//! Both primitives expose minimal, observable counters
//! (`capacity_hits_total`, `rate_hits_total`) for the Track C governor
//! status RPC.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Default max simultaneous connections when `TERMLINK_MAX_CONNECTIONS` is
/// unset. Chosen at T-2048 filing: comfortably exceeds the largest known
/// production fleet (~30 agents × 8 concurrent listeners) by ~10×.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 256;

/// Default per-sender rate limit when `TERMLINK_RATE_LIMIT_PER_SEC` is
/// unset. Chosen at T-2048 filing: AEF-style burst patterns
/// (channel.subscribe + channel.unread + topic_stats in sub-second windows)
/// fit comfortably under this ceiling while a runaway loop (a stuck
/// poller hammering 10k/sec) is contained.
pub const DEFAULT_RATE_LIMIT_PER_SEC: u32 = 1000;

/// T-2137: Default eviction sweep cadence for the per-sender rate-bucket
/// HashMap. 60s balances "responsive memory release" vs "cheap" — the
/// eviction walk is O(buckets) under a single mutex.
pub const DEFAULT_RATE_EVICT_INTERVAL_SEC: u64 = 60;

/// T-2137: Default idle threshold above which a rate-bucket is dropped.
/// 5 minutes = 5× the longest realistic rate-limit window so a sender
/// that's just resting between bursts never loses its bucket (and
/// therefore never resets to a fresh full bucket mid-burst). A sender
/// idle for 5 min was either fully back to full capacity 4 min ago or
/// has disconnected — either way the bucket is no longer load-bearing.
pub const DEFAULT_RATE_EVICT_IDLE_THRESHOLD_MS: i64 = 300_000;

static CONN_GOVERNOR: OnceLock<ConnGovernor> = OnceLock::new();
static RATE_GOVERNOR: OnceLock<RateGovernor> = OnceLock::new();

/// Read env-var overrides and install the process-global governors.
///
/// Idempotent — calling more than once preserves the first install (matches
/// `OnceLock` semantics). The accept loop calls this at startup; tests that
/// need different limits can call directly before any acquisition.
pub fn init() {
    let max_conn = parse_env_u32("TERMLINK_MAX_CONNECTIONS", DEFAULT_MAX_CONNECTIONS);
    let rate = parse_env_u32("TERMLINK_RATE_LIMIT_PER_SEC", DEFAULT_RATE_LIMIT_PER_SEC);
    let _ = CONN_GOVERNOR.set(ConnGovernor::new(max_conn));
    let _ = RATE_GOVERNOR.set(RateGovernor::new(rate));
    tracing::info!(
        max_connections = max_conn,
        rate_limit_per_sec = rate,
        "Hub governors active (T-2048 — TERMLINK_MAX_CONNECTIONS / TERMLINK_RATE_LIMIT_PER_SEC to override)"
    );
}

/// Access the global `ConnGovernor`. Lazily falls back to defaults if
/// [`init`] was never called (test paths often skip the explicit init).
pub fn conn_governor() -> &'static ConnGovernor {
    CONN_GOVERNOR.get_or_init(|| ConnGovernor::new(DEFAULT_MAX_CONNECTIONS))
}

/// Access the global `RateGovernor`. Same lazy fallback as [`conn_governor`].
pub fn rate_governor() -> &'static RateGovernor {
    RATE_GOVERNOR.get_or_init(|| RateGovernor::new(DEFAULT_RATE_LIMIT_PER_SEC))
}

/// T-2432 (T-2430 GO, PL-218): single source of truth for the per-sender
/// rate-bucket key. Precedence: explicit `params.from` (declared operator /
/// agent identity) → `params.sender_id` (the signature-verified identity
/// fingerprint that every `channel.post` already carries, T-1427) →
/// `peer_addr` (network identity) → `peer_pid` (Unix-local process identity)
/// → `"anonymous"`. Preferring a stable identity over `peer_pid` is the PL-218
/// fix: one-shot CLI invocations previously each minted a fresh pid bucket,
/// so limits never accumulated per caller and buckets bloated (PL-209
/// mechanism — 380K live buckets observed fleet-wide). The tail of the
/// precedence is unchanged for clients that send neither field.
///
/// Extracted from the two duplicated derivations in `server.rs`
/// (`process_request_message` + `maybe_handle_ws_subscribe`) so the intercept
/// paths can never drift apart (T-2372 coherence requirement).
pub fn derive_sender_key(
    from: Option<&str>,
    sender_id: Option<&str>,
    peer_addr: Option<&str>,
    peer_pid: Option<u32>,
) -> String {
    from.map(str::to_string)
        .or_else(|| sender_id.map(str::to_string))
        .or_else(|| peer_addr.map(str::to_string))
        .or_else(|| peer_pid.map(|p| p.to_string()))
        .unwrap_or_else(|| "anonymous".to_string())
}

fn parse_env_u32(name: &str, default: u32) -> u32 {
    match std::env::var(name) {
        Ok(v) => v.parse::<u32>().unwrap_or_else(|_| {
            tracing::warn!(
                env = name,
                value = %v,
                default = default,
                "Hub governor: env var unparseable as u32, using default"
            );
            default
        }),
        Err(_) => default,
    }
}

/// Wall-clock `now_ms` source for production callers. Returns ms since
/// UNIX epoch; on the rare clock skew/error case returns 0 (the governor
/// treats negative deltas as zero refill).
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Hint to the LOUD-refuse path: how long the caller should back off
/// before retrying. The value is best-effort — `ConnGovernor` cannot
/// truly predict when a slot frees, so it always returns a fixed
/// fallback; `RateGovernor` returns the exact ms until next token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryHint {
    pub retry_after_ms: u64,
}

/// Global simultaneous-connection governor.
#[derive(Debug)]
pub struct ConnGovernor {
    current: AtomicU32,
    max: u32,
    capacity_hits_total: AtomicU64,
}

impl ConnGovernor {
    pub fn new(max: u32) -> Self {
        Self {
            current: AtomicU32::new(0),
            max,
            capacity_hits_total: AtomicU64::new(0),
        }
    }

    /// Try to reserve a slot. Returns `Ok(())` on admit, `Err(RetryHint)`
    /// when at capacity. The caller is responsible for [`release`] on
    /// successful acquisition (RAII not used to keep tokio::spawn shape
    /// flat).
    pub fn try_acquire(&self) -> Result<(), RetryHint> {
        let mut prev = self.current.load(Ordering::Acquire);
        loop {
            if prev >= self.max {
                self.capacity_hits_total.fetch_add(1, Ordering::Relaxed);
                return Err(RetryHint {
                    retry_after_ms: 1000,
                });
            }
            match self.current.compare_exchange_weak(
                prev,
                prev + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(actual) => prev = actual,
            }
        }
    }

    /// Release a previously acquired slot. No-op if `current` is
    /// already 0 (defensive — should not happen under correct usage).
    pub fn release(&self) {
        // saturating-sub via compare_exchange to keep current ≥ 0
        let mut prev = self.current.load(Ordering::Acquire);
        loop {
            if prev == 0 {
                return;
            }
            match self.current.compare_exchange_weak(
                prev,
                prev - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(actual) => prev = actual,
            }
        }
    }

    pub fn current(&self) -> u32 {
        self.current.load(Ordering::Acquire)
    }

    pub fn max(&self) -> u32 {
        self.max
    }

    pub fn capacity_hits_total(&self) -> u64 {
        self.capacity_hits_total.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, Copy)]
struct RateBucket {
    tokens: f64,
    last_refill_ms: i64,
}

/// Per-sender token-bucket rate limiter.
#[derive(Debug)]
pub struct RateGovernor {
    rate_per_sec: u32,
    buckets: Mutex<HashMap<String, RateBucket>>,
    rate_hits_total: AtomicU64,
    /// T-2139: total buckets dropped by `evict_idle` across this hub's
    /// lifetime. Counts BUCKETS dropped (not eviction-loop iterations) so
    /// the value reflects work done. Operators read this through
    /// `hub.governor_status` to confirm the T-2137 eviction loop is
    /// actually firing.
    evictions_total: AtomicU64,
}

impl RateGovernor {
    pub fn new(rate_per_sec: u32) -> Self {
        Self {
            rate_per_sec,
            buckets: Mutex::new(HashMap::new()),
            rate_hits_total: AtomicU64::new(0),
            evictions_total: AtomicU64::new(0),
        }
    }

    /// Try to consume one token for `sender` at clock time `now_ms`.
    /// Returns `Ok(())` on admit (token consumed), `Err(RetryHint)`
    /// when bucket is empty (next refill in `retry_after_ms`).
    ///
    /// Bucket capacity = `rate_per_sec`. Refill rate = `rate_per_sec`
    /// tokens per second (1 token per `1000/rate_per_sec` ms).
    pub fn try_acquire(&self, sender: &str, now_ms: i64) -> Result<(), RetryHint> {
        if self.rate_per_sec == 0 {
            // Disabled — admit everything.
            return Ok(());
        }
        let capacity = self.rate_per_sec as f64;
        let refill_per_ms = capacity / 1000.0;

        let mut buckets = self.buckets.lock().expect("rate buckets mutex poisoned");
        let bucket = buckets.entry(sender.to_string()).or_insert(RateBucket {
            tokens: capacity,
            last_refill_ms: now_ms,
        });

        // Refill based on elapsed time. Clamp at capacity.
        let elapsed = (now_ms - bucket.last_refill_ms).max(0) as f64;
        bucket.tokens = (bucket.tokens + elapsed * refill_per_ms).min(capacity);
        bucket.last_refill_ms = now_ms;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            // ms until next whole token. tokens < 1 so deficit > 0.
            let deficit = 1.0 - bucket.tokens;
            let ms_until_refill = (deficit / refill_per_ms).ceil() as u64;
            self.rate_hits_total.fetch_add(1, Ordering::Relaxed);
            Err(RetryHint {
                retry_after_ms: ms_until_refill.max(1),
            })
        }
    }

    pub fn rate_per_sec(&self) -> u32 {
        self.rate_per_sec
    }

    pub fn rate_hits_total(&self) -> u64 {
        self.rate_hits_total.load(Ordering::Relaxed)
    }

    pub fn buckets_active(&self) -> usize {
        self.buckets.lock().expect("rate buckets mutex poisoned").len()
    }

    /// Evict buckets idle longer than `idle_threshold_ms`. Called
    /// periodically by the hub (T-2137: wired via
    /// [`spawn_rate_evict_loop`] at startup) to keep memory bounded; the
    /// slow tail of senders is the dominant cost. Returns the number of
    /// buckets dropped so the caller can log / surface the count.
    /// T-2139: also increments `evictions_total` so the count is
    /// observable through `hub.governor_status`.
    pub fn evict_idle(&self, now_ms: i64, idle_threshold_ms: i64) -> usize {
        let mut buckets = self.buckets.lock().expect("rate buckets mutex poisoned");
        let before = buckets.len();
        buckets.retain(|_, b| now_ms - b.last_refill_ms < idle_threshold_ms);
        let dropped = before - buckets.len();
        if dropped > 0 {
            self.evictions_total
                .fetch_add(dropped as u64, Ordering::Relaxed);
        }
        dropped
    }

    /// T-2139: monotonic count of rate-bucket evictions since hub start.
    /// Mirrors [`rate_hits_total`] — surfaced through `hub.governor_status`
    /// so operators can confirm the T-2137 eviction loop is firing.
    pub fn evictions_total(&self) -> u64 {
        self.evictions_total.load(Ordering::Relaxed)
    }
}

/// T-2137: Spawn the periodic rate-bucket eviction loop. Idempotent
/// across multiple calls (each call spawns one loop; in practice this
/// is only called once per hub instance, from [`init`]). Reads cadence
/// + threshold from env vars `TERMLINK_RATE_EVICT_INTERVAL_SEC` (clamped
/// 5..=3600) and `TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS` (clamped
/// 1000..=3_600_000), falling back to
/// `DEFAULT_RATE_EVICT_INTERVAL_SEC` / `DEFAULT_RATE_EVICT_IDLE_THRESHOLD_MS`.
///
/// Closes T-2018 §6 #10 invariant ("retention/compaction designed in
/// from the start"). Before this, the per-sender rate-bucket HashMap
/// grew unbounded — `rate_buckets_active=258_236` was observed in
/// production against a ~5-agent fleet.
///
/// Must be called from within a tokio runtime (typically the hub's
/// startup path).
pub fn spawn_rate_evict_loop() {
    let interval_sec = parse_env_u64_clamped(
        "TERMLINK_RATE_EVICT_INTERVAL_SEC",
        DEFAULT_RATE_EVICT_INTERVAL_SEC,
        5,
        3600,
    );
    let idle_threshold_ms = parse_env_i64_clamped(
        "TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS",
        DEFAULT_RATE_EVICT_IDLE_THRESHOLD_MS,
        1000,
        3_600_000,
    );
    tracing::info!(
        interval_sec,
        idle_threshold_ms,
        "Hub rate-bucket eviction loop spawned (T-2137 — \
         TERMLINK_RATE_EVICT_INTERVAL_SEC / TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS to override)"
    );
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_sec));
        // Skip the immediate first tick — no buckets to evict at startup.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let dropped = rate_governor().evict_idle(now_ms(), idle_threshold_ms);
            if dropped > 0 {
                tracing::debug!(
                    dropped,
                    buckets_remaining = rate_governor().buckets_active(),
                    "Rate-bucket eviction swept stale entries"
                );
            }
        }
    });
}

pub(crate) fn parse_env_u64_clamped(name: &str, default: u64, min: u64, max: u64) -> u64 {
    let raw = std::env::var(name).ok();
    let parsed = match raw.as_deref() {
        Some(v) => v.parse::<u64>().unwrap_or_else(|_| {
            tracing::warn!(env = name, value = v, default, "Hub governor: env var unparseable as u64, using default");
            default
        }),
        None => default,
    };
    parsed.clamp(min, max)
}

fn parse_env_i64_clamped(name: &str, default: i64, min: i64, max: i64) -> i64 {
    let raw = std::env::var(name).ok();
    let parsed = match raw.as_deref() {
        Some(v) => v.parse::<i64>().unwrap_or_else(|_| {
            tracing::warn!(env = name, value = v, default, "Hub governor: env var unparseable as i64, using default");
            default
        }),
        None => default,
    };
    parsed.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── token conservation under contention (T-2440) ────────────────

    #[test]
    fn rate_governor_token_conservation_under_contention() {
        // T-2440 (round-7 R3): N threads race ONE sender's bucket at a
        // fixed now_ms — exactly `capacity` admits may succeed. Pins the
        // check-then-act inside the mutex; a refactor moving the check
        // outside the lock would oversell and fail this.
        use std::sync::{Arc, Barrier};

        let capacity = 5u32;
        let n = 20usize;
        let g = Arc::new(RateGovernor::new(capacity));
        let barrier = Arc::new(Barrier::new(n));
        let handles: Vec<_> = (0..n)
            .map(|_| {
                let g = Arc::clone(&g);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    g.try_acquire("one-sender", 1_000)
                })
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let admitted = results.iter().filter(|r| r.is_ok()).count();
        let refused = results.iter().filter(|r| r.is_err()).count();
        assert_eq!(admitted, capacity as usize, "no oversell, no lost token");
        assert_eq!(refused, n - capacity as usize);
        assert_eq!(g.buckets_active(), 1, "one sender == one bucket");
        assert_eq!(g.rate_hits_total(), (n - capacity as usize) as u64);
    }

    #[test]
    fn conn_governor_conserves_capacity_under_contention() {
        // T-2440: the lock-free CAS loop admits exactly `max` under an
        // N-thread race; release() then conserves the cap for a second
        // wave.
        use std::sync::{Arc, Barrier};

        let cap = 4u32;
        let n = 16usize;
        let g = Arc::new(ConnGovernor::new(cap));
        let barrier = Arc::new(Barrier::new(n));
        let handles: Vec<_> = (0..n)
            .map(|_| {
                let g = Arc::clone(&g);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    g.try_acquire().is_ok()
                })
            })
            .collect();
        let admitted = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(|ok| *ok)
            .count() as u32;
        assert_eq!(admitted, cap, "exactly max slots admitted");
        assert_eq!(g.current(), cap);

        // Release one → exactly one more admit possible.
        g.release();
        assert!(g.try_acquire().is_ok());
        assert!(g.try_acquire().is_err(), "cap conserved after release/reacquire");
        assert_eq!(g.current(), cap);
    }

    // ── derive_sender_key (T-2432) ──────────────────────────────────

    #[test]
    fn sender_key_prefers_explicit_from() {
        let k = derive_sender_key(Some("fp-abc"), Some("sid-1"), Some("10.0.0.1:5"), Some(42));
        assert_eq!(k, "fp-abc");
    }

    #[test]
    fn sender_key_falls_back_to_verified_sender_id() {
        let k = derive_sender_key(None, Some("sid-1"), Some("10.0.0.1:5"), Some(42));
        assert_eq!(k, "sid-1");
    }

    #[test]
    fn sender_key_falls_back_to_peer_addr_then_pid() {
        assert_eq!(
            derive_sender_key(None, None, Some("10.0.0.1:5"), Some(42)),
            "10.0.0.1:5"
        );
        assert_eq!(derive_sender_key(None, None, None, Some(42)), "42");
    }

    #[test]
    fn sender_key_anonymous_when_nothing_known() {
        assert_eq!(derive_sender_key(None, None, None, None), "anonymous");
    }

    #[test]
    fn same_identity_across_two_connections_shares_one_bucket() {
        // PL-218 regression shape: two "processes" (distinct pids) declaring
        // the same identity must land in ONE bucket so limits accumulate.
        let g = RateGovernor::new(2);
        let k1 = derive_sender_key(Some("fp-same"), None, None, Some(100));
        let k2 = derive_sender_key(Some("fp-same"), None, None, Some(101));
        assert_eq!(k1, k2);
        assert!(g.try_acquire(&k1, 0).is_ok());
        assert!(g.try_acquire(&k2, 0).is_ok());
        assert!(
            g.try_acquire(&k2, 0).is_err(),
            "third call under shared identity bucket must rate-limit"
        );
        assert_eq!(g.buckets_active(), 1, "one bucket, not one per pid");
    }

    // ── ConnGovernor ────────────────────────────────────────────────

    #[test]
    fn conn_governor_admits_up_to_max() {
        let g = ConnGovernor::new(3);
        assert!(g.try_acquire().is_ok());
        assert!(g.try_acquire().is_ok());
        assert!(g.try_acquire().is_ok());
        assert_eq!(g.current(), 3);
    }

    #[test]
    fn conn_governor_rejects_past_max_with_retry_hint() {
        let g = ConnGovernor::new(2);
        assert!(g.try_acquire().is_ok());
        assert!(g.try_acquire().is_ok());
        let err = g.try_acquire().expect_err("third must reject");
        assert!(err.retry_after_ms > 0);
        assert_eq!(g.capacity_hits_total(), 1);
    }

    #[test]
    fn conn_governor_release_frees_a_slot() {
        let g = ConnGovernor::new(2);
        g.try_acquire().unwrap();
        g.try_acquire().unwrap();
        assert!(g.try_acquire().is_err());
        g.release();
        assert_eq!(g.current(), 1);
        assert!(g.try_acquire().is_ok());
    }

    #[test]
    fn conn_governor_release_below_zero_is_noop() {
        let g = ConnGovernor::new(2);
        g.release();
        g.release();
        assert_eq!(g.current(), 0);
    }

    // ── RateGovernor ────────────────────────────────────────────────

    #[test]
    fn rate_governor_admits_burst_up_to_capacity_then_rejects() {
        let g = RateGovernor::new(5);
        let now = 1_000_000;
        // Burst of 5 should all admit.
        for i in 0..5 {
            assert!(
                g.try_acquire("alice", now).is_ok(),
                "burst slot {i} should admit"
            );
        }
        let err = g.try_acquire("alice", now).expect_err("6th must reject");
        assert!(err.retry_after_ms >= 1);
        assert_eq!(g.rate_hits_total(), 1);
    }

    #[test]
    fn rate_governor_refills_after_elapsed_time() {
        let g = RateGovernor::new(10); // 10 tokens/sec → 1 token per 100ms
        let t0 = 1_000_000;
        // Burn the full bucket.
        for _ in 0..10 {
            g.try_acquire("bob", t0).unwrap();
        }
        assert!(g.try_acquire("bob", t0).is_err());
        // 500ms later → 5 tokens refilled. We should be able to admit 5.
        let t1 = t0 + 500;
        for i in 0..5 {
            assert!(
                g.try_acquire("bob", t1).is_ok(),
                "refill slot {i} should admit"
            );
        }
        assert!(g.try_acquire("bob", t1).is_err());
    }

    #[test]
    fn rate_governor_isolates_senders() {
        let g = RateGovernor::new(2);
        let now = 1_000_000;
        g.try_acquire("alice", now).unwrap();
        g.try_acquire("alice", now).unwrap();
        assert!(g.try_acquire("alice", now).is_err()); // alice exhausted

        // bob has his own bucket and should still admit.
        g.try_acquire("bob", now).unwrap();
        g.try_acquire("bob", now).unwrap();
        assert!(g.try_acquire("bob", now).is_err());

        assert_eq!(g.buckets_active(), 2);
    }

    #[test]
    fn rate_governor_zero_rate_disables_limit() {
        let g = RateGovernor::new(0);
        let now = 1_000_000;
        for _ in 0..1000 {
            assert!(g.try_acquire("flood", now).is_ok());
        }
        assert_eq!(g.rate_hits_total(), 0);
    }

    #[test]
    fn rate_governor_retry_hint_matches_refill_period() {
        // 1000 tokens/sec → 1 ms per token. After bucket empty, hint
        // should be 1ms (the next token's arrival).
        let g = RateGovernor::new(1000);
        let now = 1_000_000;
        for _ in 0..1000 {
            g.try_acquire("carol", now).unwrap();
        }
        let hint = g.try_acquire("carol", now).expect_err("must reject");
        assert_eq!(
            hint.retry_after_ms, 1,
            "1000/sec → 1ms per token refill"
        );
    }

    #[test]
    fn rate_governor_refill_clamps_at_capacity() {
        let g = RateGovernor::new(5);
        let t0 = 1_000_000;
        // First call creates the bucket at capacity=5 and consumes 1 → 4 remain.
        g.try_acquire("dave", t0).unwrap();
        // 10 seconds later (way past full refill) — should still cap at 5.
        let t1 = t0 + 10_000;
        for i in 0..5 {
            assert!(
                g.try_acquire("dave", t1).is_ok(),
                "post-overflow slot {i} should admit"
            );
        }
        // 6th must reject — bucket capped at capacity, not unbounded.
        assert!(g.try_acquire("dave", t1).is_err());
    }

    #[test]
    fn rate_governor_evict_idle_drops_stale_buckets() {
        let g = RateGovernor::new(10);
        let t0 = 1_000_000;
        g.try_acquire("ephemeral", t0).unwrap();
        g.try_acquire("active", t0).unwrap();
        assert_eq!(g.buckets_active(), 2);

        // 60 seconds later, only "active" hits again.
        let t1 = t0 + 60_000;
        g.try_acquire("active", t1).unwrap();

        let dropped = g.evict_idle(t1, 30_000);
        assert_eq!(
            g.buckets_active(),
            1,
            "ephemeral last_refill at t0, now-30s threshold should evict"
        );
        assert_eq!(dropped, 1, "evict_idle should report 1 bucket dropped");
    }

    /// T-2139: `evictions_total` reflects the per-bucket eviction count
    /// across multiple sweeps — the monotonic counter exposed through
    /// `hub.governor_status` so operators see the T-2137 loop firing.
    #[test]
    fn rate_governor_evictions_total_accumulates() {
        let g = RateGovernor::new(10);
        assert_eq!(g.evictions_total(), 0, "fresh governor starts at zero");

        let t0 = 1_000_000;
        g.try_acquire("a", t0).unwrap();
        g.try_acquire("b", t0).unwrap();
        g.try_acquire("c", t0).unwrap();
        assert_eq!(g.buckets_active(), 3);

        // Evict the 3 buckets — all idle past threshold.
        let dropped = g.evict_idle(t0 + 60_000, 30_000);
        assert_eq!(dropped, 3);
        assert_eq!(g.evictions_total(), 3, "first sweep records 3 evictions");

        // Second sweep with no buckets — counter unchanged.
        let dropped = g.evict_idle(t0 + 90_000, 30_000);
        assert_eq!(dropped, 0);
        assert_eq!(g.evictions_total(), 3, "no-op sweep leaves counter alone");

        // Add 2 more, sweep, counter accumulates.
        g.try_acquire("d", t0 + 90_000).unwrap();
        g.try_acquire("e", t0 + 90_000).unwrap();
        let dropped = g.evict_idle(t0 + 200_000, 30_000);
        assert_eq!(dropped, 2);
        assert_eq!(
            g.evictions_total(),
            5,
            "monotonic counter must be additive across sweeps"
        );
    }

    /// T-2137: env-knob clamps (T-2018 §6 #10 retention/compaction).
    /// `spawn_rate_evict_loop` itself is one-line `tokio::spawn(loop)`
    /// over the already-tested `evict_idle` primitive — there's nothing
    /// the env-knob clamps don't already cover.
    #[test]
    fn rate_evict_env_knobs_clamp_to_bounds() {
        // Interval: clamp to [5, 3600].
        unsafe { std::env::set_var("TERMLINK_RATE_EVICT_INTERVAL_SEC", "1"); }
        assert_eq!(
            parse_env_u64_clamped("TERMLINK_RATE_EVICT_INTERVAL_SEC", 60, 5, 3600),
            5,
            "below-min should clamp UP to 5"
        );
        unsafe { std::env::set_var("TERMLINK_RATE_EVICT_INTERVAL_SEC", "999999"); }
        assert_eq!(
            parse_env_u64_clamped("TERMLINK_RATE_EVICT_INTERVAL_SEC", 60, 5, 3600),
            3600,
            "above-max should clamp DOWN to 3600"
        );
        unsafe { std::env::set_var("TERMLINK_RATE_EVICT_INTERVAL_SEC", "120"); }
        assert_eq!(
            parse_env_u64_clamped("TERMLINK_RATE_EVICT_INTERVAL_SEC", 60, 5, 3600),
            120,
            "in-range should pass through unchanged"
        );
        unsafe { std::env::set_var("TERMLINK_RATE_EVICT_INTERVAL_SEC", "garbage"); }
        assert_eq!(
            parse_env_u64_clamped("TERMLINK_RATE_EVICT_INTERVAL_SEC", 60, 5, 3600),
            60,
            "unparseable should fall back to default"
        );

        // Threshold: clamp to [1000, 3_600_000].
        unsafe { std::env::set_var("TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS", "100"); }
        assert_eq!(
            parse_env_i64_clamped(
                "TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS",
                300_000,
                1000,
                3_600_000
            ),
            1000,
            "below-min should clamp UP to 1000ms"
        );

        unsafe {
            std::env::remove_var("TERMLINK_RATE_EVICT_INTERVAL_SEC");
            std::env::remove_var("TERMLINK_RATE_EVICT_IDLE_THRESHOLD_MS");
        }
    }
}
