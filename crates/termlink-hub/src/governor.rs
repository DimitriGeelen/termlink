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
}

impl RateGovernor {
    pub fn new(rate_per_sec: u32) -> Self {
        Self {
            rate_per_sec,
            buckets: Mutex::new(HashMap::new()),
            rate_hits_total: AtomicU64::new(0),
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
    /// periodically by the hub to keep memory bounded; the slow tail
    /// of senders is the dominant cost.
    #[allow(dead_code)] // wired in slice 2
    pub fn evict_idle(&self, now_ms: i64, idle_threshold_ms: i64) {
        let mut buckets = self.buckets.lock().expect("rate buckets mutex poisoned");
        buckets.retain(|_, b| now_ms - b.last_refill_ms < idle_threshold_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        g.evict_idle(t1, 30_000);
        assert_eq!(
            g.buckets_active(),
            1,
            "ephemeral last_refill at t0, now-30s threshold should evict"
        );
    }
}
