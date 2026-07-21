//! T-2427: env-gated hub-side periodic retention sweep.
//!
//! Retention policies (`termlink-bus` `Retention`) are enforced only when
//! `Bus::sweep` runs — the bus deliberately runs no background thread
//! (T-1155: "enforcement is explicit, never implicit"). In the field that
//! pushed enforcement out to per-host operator crons, which are empirically
//! the least reliable component in the estate: T-1991 recurred on .121
//! because the sweep cron was never installed there, and the topic-growth
//! canary (T-2252) exists solely to detect "the cron never fired".
//!
//! This module is the middle path: an OPT-IN periodic sweep loop inside the
//! hub process, gated on `TERMLINK_SWEEP_INTERVAL_SECS` — set it once in the
//! systemd unit (the same place `TERMLINK_RUNTIME_DIR` already lives) and
//! every bounded topic on the hub is enforced on that cadence. Unset (the
//! default) preserves the exact pre-T-2427 behavior: no background sweep,
//! `channel.sweep` remains the only trigger. T-1155's explicitness survives
//! as a single opt-in declaration instead of N per-host crons.
//!
//! Telemetry: `retention_sweep_interval_secs` / `retention_sweep_runs_total`
//! / `retention_sweep_pruned_total` on `hub.governor_status`, so a wrapper
//! (fleet governor-status, /governor) can confirm the loop is firing —
//! the loop must never become the next silent-failure class it exists to
//! close.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use termlink_bus::{Bus, Retention};
use tokio::sync::watch;

/// Env var carrying the sweep cadence in seconds. Absent / empty / `0` /
/// unparseable ⇒ disabled (exact pre-T-2427 behavior).
pub const ENV_VAR: &str = "TERMLINK_SWEEP_INTERVAL_SECS";
/// Floor: sweeping more often than every 30s is pure overhead — retention
/// windows are minutes-to-days scale.
pub const MIN_INTERVAL_SECS: u64 = 30;
/// Ceiling: one day. Longer than that and the operator wanted "off".
pub const MAX_INTERVAL_SECS: u64 = 86_400;

/// Completed sweep passes since hub start (0 while disabled).
static SWEEP_RUNS_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Total records pruned by the periodic loop since hub start.
static SWEEP_PRUNED_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Active interval in seconds; 0 = loop not running (disabled).
static SWEEP_INTERVAL_ACTIVE_SECS: AtomicU64 = AtomicU64::new(0);

pub fn runs_total() -> u64 {
    SWEEP_RUNS_TOTAL.load(Ordering::Relaxed)
}

pub fn pruned_total() -> u64 {
    SWEEP_PRUNED_TOTAL.load(Ordering::Relaxed)
}

pub fn interval_active_secs() -> u64 {
    SWEEP_INTERVAL_ACTIVE_SECS.load(Ordering::Relaxed)
}

/// Parse `TERMLINK_SWEEP_INTERVAL_SECS` into an active interval.
///
/// `None` ⇒ disabled. Unparseable values disable LOUDLY (warn naming the
/// var and the received value) rather than silently picking a default —
/// a mistyped opt-in must not half-arm the loop.
pub fn interval_from_env() -> Option<Duration> {
    let raw = std::env::var(ENV_VAR).ok()?;
    parse_interval(&raw)
}

/// Pure parse step for [`interval_from_env`] — unit-testable without env
/// mutation races.
pub(crate) fn parse_interval(raw: &str) -> Option<Duration> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<u64>() {
        Ok(0) => None,
        Ok(n) => Some(Duration::from_secs(n.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS))),
        Err(_) => {
            tracing::warn!(
                env_var = ENV_VAR,
                value = %trimmed,
                "unparseable sweep interval — periodic retention sweeper DISABLED (set a positive integer number of seconds, or unset/0 to disable intentionally)"
            );
            None
        }
    }
}

/// One full sweep pass: enforce retention on every bounded topic.
///
/// Returns `(topics_pruned_from, records_pruned)`. `Forever` topics are
/// skipped (sweeping them is a no-op by contract, but skipping avoids N
/// pointless calls on debris-heavy hubs). Per-topic failures warn and
/// continue — one broken topic must never abort enforcement for the rest.
pub fn sweep_all(bus: &Bus, now_unix_ms: i64) -> (u64, u64) {
    let topics = match bus.list_topics() {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(error = %e, "periodic sweep: list_topics failed — pass skipped");
            return (0, 0);
        }
    };
    let mut topics_pruned = 0u64;
    let mut records_pruned = 0u64;
    for topic in topics {
        match bus.topic_retention(&topic) {
            Ok(Some(Retention::Forever)) | Ok(None) => continue,
            Ok(Some(_)) => match bus.sweep(&topic, now_unix_ms) {
                Ok(0) => {}
                Ok(n) => {
                    topics_pruned += 1;
                    records_pruned += n;
                }
                Err(e) => {
                    tracing::warn!(topic = %topic, error = %e, "periodic sweep failed for topic — continuing");
                }
            },
            Err(e) => {
                tracing::warn!(topic = %topic, error = %e, "periodic sweep: retention lookup failed — topic skipped");
            }
        }
    }
    (topics_pruned, records_pruned)
}

/// The periodic loop. Spawn from server startup ONLY when
/// [`interval_from_env`] returned `Some` — the disabled path must not even
/// start the task. Mirrors `supervisor::run`'s shutdown/sleep select.
pub async fn run(interval: Duration, mut shutdown_rx: watch::Receiver<bool>) {
    SWEEP_INTERVAL_ACTIVE_SECS.store(interval.as_secs(), Ordering::Relaxed);
    tracing::info!(
        interval_secs = interval.as_secs(),
        "Periodic retention sweeper ENABLED (T-2427, {ENV_VAR}) — bounded topics enforced on this cadence"
    );
    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("Periodic retention sweeper shutting down");
                    break;
                }
            }
            _ = tokio::time::sleep(interval) => {
                let Some(bus) = crate::channel::bus() else {
                    // Bus not installed (shouldn't happen post-start) — skip quietly.
                    continue;
                };
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                let (topics, records) = sweep_all(bus, now);
                SWEEP_RUNS_TOTAL.fetch_add(1, Ordering::Relaxed);
                SWEEP_PRUNED_TOTAL.fetch_add(records, Ordering::Relaxed);
                if records > 0 {
                    tracing::info!(
                        topics_pruned = topics,
                        records_pruned = records,
                        "Periodic retention sweep complete (T-2427)"
                    );
                } else {
                    tracing::debug!("Periodic retention sweep complete — nothing to prune");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;
    use termlink_bus::Envelope;

    fn tmp_bus() -> (TempDir, Bus) {
        let dir = TempDir::new().unwrap();
        let bus = Bus::open(dir.path()).unwrap();
        (dir, bus)
    }

    fn env(topic: &str, payload: &[u8]) -> Envelope {
        Envelope {
            topic: topic.to_string(),
            sender_id: "test".to_string(),
            msg_type: "note".to_string(),
            payload: payload.to_vec(),
            artifact_ref: None,
            ts_unix_ms: 0,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn parse_interval_disabled_states() {
        assert_eq!(parse_interval(""), None, "empty → disabled");
        assert_eq!(parse_interval("   "), None, "whitespace → disabled");
        assert_eq!(parse_interval("0"), None, "explicit 0 → disabled");
        assert_eq!(parse_interval("garbage"), None, "unparseable → disabled");
        assert_eq!(parse_interval("-30"), None, "negative → unparseable → disabled");
        assert_eq!(parse_interval("30.5"), None, "float → unparseable → disabled");
    }

    #[test]
    fn parse_interval_clamps_to_bounds() {
        assert_eq!(
            parse_interval("5"),
            Some(Duration::from_secs(MIN_INTERVAL_SECS)),
            "below floor clamps up"
        );
        assert_eq!(parse_interval("300"), Some(Duration::from_secs(300)));
        assert_eq!(
            parse_interval("999999"),
            Some(Duration::from_secs(MAX_INTERVAL_SECS)),
            "above ceiling clamps down"
        );
        assert_eq!(parse_interval(" 3600 "), Some(Duration::from_secs(3600)), "trimmed");
    }

    #[tokio::test]
    async fn sweep_all_prunes_bounded_skips_forever() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("bounded", Retention::Messages(2)).unwrap();
        bus.create_topic("immortal", Retention::Forever).unwrap();
        for i in 0..5 {
            bus.post("bounded", &env("bounded", format!("b{i}").as_bytes()))
                .await
                .unwrap();
            bus.post("immortal", &env("immortal", format!("f{i}").as_bytes()))
                .await
                .unwrap();
        }
        let (topics, records) = sweep_all(&bus, 0);
        assert_eq!(topics, 1, "only the bounded topic is pruned from");
        assert_eq!(records, 3, "5 posted, Messages(2) keeps 2");
        // Forever topic untouched.
        let immortal_count = bus.subscribe("immortal", 0).unwrap().count();
        assert_eq!(immortal_count, 5);
    }

    #[tokio::test]
    async fn sweep_all_nothing_bounded_is_zero() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("a", Retention::Forever).unwrap();
        assert_eq!(sweep_all(&bus, 0), (0, 0));
    }
}
