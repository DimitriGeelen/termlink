//! T-2027/T-2089 substrate primitive #9 slice 1 — hub-side current-value
//! index for broadcast-with-replay.
//!
//! Closes the late-joiner state-on-subscribe gap from
//! [`docs/architecture/parallel-execution-substrate.md`] §6 #9: a fresh
//! subscriber to `agent-presence` should be able to read "latest heartbeat
//! per agent_id" without walking 10K envelopes to produce 10 rows. The
//! cv_index makes that path O(K) where K = distinct cv_keys for the topic.
//!
//! ## Model
//!
//! Posters opt-in by adding `metadata.cv_key=<string>` to the post. The hub
//! maintains an in-memory `HashMap<topic, HashMap<cv_key, offset>>` updated
//! on every successful `channel.post` that carries a `cv_key`. Future
//! `channel.subscribe --include-current-value=true` (slice 2) reads from
//! this index to prepend cv-tagged envelopes to the live stream.
//!
//! Semantics (per T-2089 inception IW-2): **last-write-wins on `cv_key`** —
//! a second post with the same `cv_key` overwrites the prior offset. This
//! mirrors how `agent-presence` LIVE/STALE/OFFLINE is computed today
//! (latest heartbeat per agent_id wins).
//!
//! ## Storage
//!
//! In-memory only. Restart-tolerance comes from later slices via either
//! eager startup scan of the topic log OR lazy first-query scan — slice 1
//! ships only the steady-state record path; the read side (slice 2)
//! decides which rebuild mode best fits its query semantics.
//!
//! ## Per-topic cap (T-2089 A2)
//!
//! Distinct cv_keys per topic are capped at 1000 to bound memory under a
//! pathological poster that mints unbounded cv_keys. On overflow:
//!
//! * The post itself is NOT failed — `channel.post` stays atomic. The
//!   envelope is durably appended and visible to all live subscribers.
//! * The cv_index annotation is dropped — the post simply doesn't appear
//!   in the per-key current-value response.
//! * The `overflow_total` counter increments so `hub.governor_status`
//!   (T-2048 sibling) can surface the condition to the operator.
//!
//! Operator override: `TERMLINK_CV_INDEX_CAP_PER_TOPIC=<n>`. Default 1000.
//!
//! ## Lineage / related
//!
//! * T-2018 — parallel-execution-substrate ADR (§6 #9)
//! * T-2027 — substrate primitive #9 build umbrella
//! * T-2089 — inception (GO Design A — tagged-post current-value)
//! * T-2049 — dedupe module (template for this module's structure)
//! * T-2048 — governor module (observability sibling — `entries_active` /
//!   `overflow_total` accessors will surface here in slice 2 wiring)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Default per-topic cap on distinct `cv_key` entries. Matches the T-2089
/// A2 assumption (1000 distinct rooms per topic is well above realistic
/// fleet + chat-arc-thread + dm-pair load and bounds memory at well under
/// 1 MB per topic).
pub const DEFAULT_CV_INDEX_CAP_PER_TOPIC: usize = 1000;

static CV_INDEX: OnceLock<CvIndex> = OnceLock::new();

/// Read env-var override and install the process-global cv_index.
///
/// Idempotent — calling more than once preserves the first install
/// (`OnceLock` semantics). Called from hub startup alongside
/// [`crate::dedupe::init`] / [`crate::governor::init`]; tests that need
/// different limits construct [`CvIndex`] directly.
pub fn init() {
    let cap_per_topic =
        parse_env_usize("TERMLINK_CV_INDEX_CAP_PER_TOPIC", DEFAULT_CV_INDEX_CAP_PER_TOPIC);
    let _ = CV_INDEX.set(CvIndex::new(cap_per_topic));
    tracing::info!(
        cv_index_cap_per_topic = cap_per_topic,
        "Hub cv_index active (T-2027/T-2089 slice 1 — TERMLINK_CV_INDEX_CAP_PER_TOPIC to override)"
    );
}

/// Access the global [`CvIndex`]. Lazy-falls-back to defaults if [`init`]
/// was never called (test paths often skip explicit init, matching the
/// [`crate::dedupe::post_dedupe`] convention).
pub fn cv_index() -> &'static CvIndex {
    CV_INDEX.get_or_init(|| CvIndex::new(DEFAULT_CV_INDEX_CAP_PER_TOPIC))
}

/// Record the latest offset for `(topic, cv_key)`. Last-write-wins:
/// if `(topic, cv_key)` was already present, the offset is updated to
/// the new value (mirrors agent-presence latest-heartbeat semantics).
///
/// Called from `handle_channel_post_with` after `bus.post` returns
/// `Ok(offset)` AND the envelope's metadata carries a `cv_key` field.
///
/// Returns `true` when the entry was recorded, `false` when the per-topic
/// cap was hit AND this is a NEW key (existing-key updates always succeed
/// since they don't grow the map). Callers can ignore the return —
/// overflow is silent at the post-handler layer; observability comes via
/// [`CvIndex::overflow_total`].
pub fn record(topic: &str, cv_key: &str, offset: u64) -> bool {
    cv_index().record(topic, cv_key, offset)
}

/// Read the current-value snapshot for `topic` as a list of
/// `(cv_key, offset)` pairs. Order is unspecified (HashMap iteration).
///
/// Used by `handle_channel_subscribe` (slice 2) when
/// `include_current_value=true`.
pub fn current_values(topic: &str) -> Vec<(String, u64)> {
    cv_index().current_values(topic)
}

/// Total entries across all topics (sum of inner-map sizes). Surfaced
/// via `hub.governor_status` in slice 2 for memory observability.
pub fn entries_active() -> u64 {
    cv_index().entries_active()
}

/// Monotonic counter of cap-overflow refusals across all topics. Surfaced
/// via `hub.governor_status` in slice 2 to alert operators that some
/// topic has saturated its 1000-key cap (likely poster misuse).
pub fn overflow_total() -> u64 {
    cv_index().overflow_total()
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    match std::env::var(name) {
        Ok(v) => v.parse::<usize>().unwrap_or_else(|_| {
            tracing::warn!(env = name, value = %v, default, "Hub cv_index: env var unparseable as usize, using default");
            default
        }),
        Err(_) => default,
    }
}

/// Process-global current-value index.
///
/// Two-level map: outer key is the topic, inner key is the cv_key, value
/// is the latest offset. New topics are created on first `record` call;
/// no explicit topic registration is required (matches how the rest of
/// the hub treats topics — they exist if anyone has posted to them).
pub struct CvIndex {
    cap_per_topic: usize,
    map: Mutex<HashMap<String, HashMap<String, u64>>>,
    overflow_total: AtomicU64,
}

impl CvIndex {
    /// Build a new index with the given per-topic cap. The cap is clamped
    /// to a minimum of 1 to avoid the degenerate "zero capacity, no
    /// inserts ever succeed" case.
    pub fn new(cap_per_topic: usize) -> Self {
        Self {
            cap_per_topic: cap_per_topic.max(1),
            map: Mutex::new(HashMap::new()),
            overflow_total: AtomicU64::new(0),
        }
    }

    /// Record `(topic, cv_key) -> offset` with last-write-wins. Returns
    /// `true` on accept, `false` on cap-overflow refusal for new keys.
    /// Updates to existing keys always succeed and return `true`.
    pub fn record(&self, topic: &str, cv_key: &str, offset: u64) -> bool {
        let mut map = self.map.lock().expect("cv_index mutex poisoned");
        let inner = map.entry(topic.to_string()).or_default();

        if inner.contains_key(cv_key) {
            // Existing key — last-write-wins update. Doesn't grow the map.
            inner.insert(cv_key.to_string(), offset);
            return true;
        }

        if inner.len() >= self.cap_per_topic {
            // New key but topic is at cap — refuse to grow the map.
            // The post itself stays atomic; only the index annotation
            // is dropped. Observable via `overflow_total`.
            self.overflow_total.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        inner.insert(cv_key.to_string(), offset);
        true
    }

    /// Snapshot of `(cv_key, offset)` pairs for `topic`. Empty Vec if
    /// the topic has no cv-tagged posts (or doesn't exist in the index).
    pub fn current_values(&self, topic: &str) -> Vec<(String, u64)> {
        let map = self.map.lock().expect("cv_index mutex poisoned");
        match map.get(topic) {
            Some(inner) => inner
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Total entries across all topics. O(N_topics).
    pub fn entries_active(&self) -> u64 {
        let map = self.map.lock().expect("cv_index mutex poisoned");
        map.values().map(|inner| inner.len() as u64).sum()
    }

    /// Number of distinct topics with at least one cv entry.
    pub fn topics_active(&self) -> u64 {
        self.map.lock().expect("cv_index mutex poisoned").len() as u64
    }

    /// Monotonic count of cap-overflow refusals.
    pub fn overflow_total(&self) -> u64 {
        self.overflow_total.load(Ordering::Relaxed)
    }

    /// Configured per-topic cap. Exposed for `hub.governor_status`.
    pub fn cap_per_topic(&self) -> usize {
        self.cap_per_topic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_index_returns_empty_snapshot() {
        let idx = CvIndex::new(1000);
        assert_eq!(idx.entries_active(), 0);
        assert_eq!(idx.topics_active(), 0);
        assert!(idx.current_values("anything").is_empty());
    }

    #[test]
    fn single_key_record_then_read() {
        let idx = CvIndex::new(1000);
        assert!(idx.record("agent-presence", "alice", 42));

        let snap = idx.current_values("agent-presence");
        assert_eq!(snap, vec![("alice".to_string(), 42)]);
        assert_eq!(idx.entries_active(), 1);
        assert_eq!(idx.topics_active(), 1);
    }

    #[test]
    fn key_update_is_last_write_wins() {
        let idx = CvIndex::new(1000);
        assert!(idx.record("agent-presence", "alice", 1));
        assert!(idx.record("agent-presence", "alice", 5));
        assert!(idx.record("agent-presence", "alice", 99));

        let snap = idx.current_values("agent-presence");
        assert_eq!(snap, vec![("alice".to_string(), 99)]);
        // Still only ONE entry — updates don't grow the map.
        assert_eq!(idx.entries_active(), 1);
    }

    #[test]
    fn multi_topic_isolation() {
        let idx = CvIndex::new(1000);
        assert!(idx.record("agent-presence", "alice", 10));
        assert!(idx.record("agent-presence", "bob", 20));
        assert!(idx.record("dm:a:b", "alice", 50));
        assert!(idx.record("dm:a:b", "carol", 60));

        let presence = idx.current_values("agent-presence");
        assert_eq!(presence.len(), 2);
        let mut presence_sorted = presence;
        presence_sorted.sort();
        assert_eq!(
            presence_sorted,
            vec![("alice".to_string(), 10), ("bob".to_string(), 20)]
        );

        let dm = idx.current_values("dm:a:b");
        assert_eq!(dm.len(), 2);
        let mut dm_sorted = dm;
        dm_sorted.sort();
        assert_eq!(
            dm_sorted,
            vec![("alice".to_string(), 50), ("carol".to_string(), 60)]
        );

        assert_eq!(idx.entries_active(), 4);
        assert_eq!(idx.topics_active(), 2);
    }

    #[test]
    fn cap_overflow_refuses_new_key_but_not_update() {
        let idx = CvIndex::new(2); // tiny cap for testability

        // Fill to cap.
        assert!(idx.record("topic-x", "k1", 1));
        assert!(idx.record("topic-x", "k2", 2));
        assert_eq!(idx.entries_active(), 2);

        // New key past cap — refused.
        assert!(!idx.record("topic-x", "k3", 3));
        assert_eq!(idx.entries_active(), 2);
        assert_eq!(idx.overflow_total(), 1);

        // Existing-key update is still allowed (doesn't grow the map).
        assert!(idx.record("topic-x", "k1", 99));
        let snap: HashMap<_, _> = idx.current_values("topic-x").into_iter().collect();
        assert_eq!(snap.get("k1"), Some(&99));
        assert_eq!(snap.get("k2"), Some(&2));
        assert!(snap.get("k3").is_none());

        // Overflow count unchanged by the update.
        assert_eq!(idx.overflow_total(), 1);

        // A DIFFERENT topic gets its own cap-budget.
        assert!(idx.record("topic-y", "y1", 10));
        assert!(idx.record("topic-y", "y2", 20));
        assert!(!idx.record("topic-y", "y3", 30));
        assert_eq!(idx.overflow_total(), 2);
    }

    #[test]
    fn zero_cap_clamps_to_minimum_one() {
        let idx = CvIndex::new(0);
        // Cap clamped to 1 — one insert succeeds, second is refused.
        assert!(idx.record("t", "a", 1));
        assert!(!idx.record("t", "b", 2));
        // Update to existing key still works.
        assert!(idx.record("t", "a", 7));
        let snap = idx.current_values("t");
        assert_eq!(snap, vec![("a".to_string(), 7)]);
    }

    #[test]
    fn current_values_unknown_topic_is_empty() {
        let idx = CvIndex::new(1000);
        idx.record("known", "k", 1);
        assert!(idx.current_values("unknown").is_empty());
    }

    #[test]
    fn overflow_counter_monotonic_across_topics() {
        let idx = CvIndex::new(1);
        // topic-a: 1 accepted + 2 refused
        idx.record("topic-a", "a1", 1);
        idx.record("topic-a", "a2", 2);
        idx.record("topic-a", "a3", 3);
        // topic-b: 1 accepted + 1 refused
        idx.record("topic-b", "b1", 1);
        idx.record("topic-b", "b2", 2);
        assert_eq!(idx.overflow_total(), 3);
    }
}
