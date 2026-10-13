//! Component heartbeat (K-3, R-8, FP-1 failure 4).
//!
//! A sidecar is several cooperating loops in one process: the send pump, the
//! receive loop, the tick thread and one accept loop per API scope. Process
//! liveness says nothing about them; a loop that panicked or hangs leaves a live
//! pid behind a deaf agent. Each loop records when it last came round; once per
//! tick the daemon writes them to `heartbeat.json` in the agent's store and exits
//! non-zero, naming the silent loop, when one is older than [`STALE_AFTER_MS`].
//! The supervisor (systemd unit or cron backstop) then restarts the sidecar, and
//! the backstop reads the same file to catch a process that hangs as a whole.

use serde_json::{Value, json};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub const HEARTBEAT_FILE: &str = "heartbeat.json";

/// A loop silent for three ticks is dead. The longest legitimate pass is bounded
/// by the hub timeout (10 s per call), well inside this.
pub const STALE_AFTER_MS: u64 = 3 * crate::deadline::TICK_MS;

/// Who restarts this sidecar, as the starter declared it in the environment.
pub const SUPERVISOR_ENV: &str = "TERMLINK_SIDECAR_SUPERVISOR";
/// The project's `SIDECAR_KEEP_RUNNING` as it was when the sidecar started.
pub const KEEP_RUNNING_ENV: &str = "TERMLINK_SIDECAR_KEEP_RUNNING";

/// Test seams, honoured only in a test estate (never in production mode):
/// `TERMLINK_SIDECAR_TEST_HEARTBEAT_MS` (how often), `TERMLINK_SIDECAR_TEST_STALE_MS`
/// (how old is dead) and `TERMLINK_SIDECAR_TEST_FREEZE` (a component reported as
/// silent, to drive the exit path). Returns `(every_ms, stale_after_ms, frozen)`.
pub fn test_overrides(mode: &crate::estate::Mode) -> (u64, u64, Option<String>) {
    let defaults = (crate::deadline::TICK_MS, STALE_AFTER_MS, None);
    if !matches!(mode, crate::estate::Mode::Estate(_)) {
        return defaults;
    }
    let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<u64>().ok()).filter(|n| *n > 0);
    (
        num("TERMLINK_SIDECAR_TEST_HEARTBEAT_MS").unwrap_or(defaults.0),
        num("TERMLINK_SIDECAR_TEST_STALE_MS").unwrap_or(defaults.1),
        std::env::var("TERMLINK_SIDECAR_TEST_FREEZE").ok().filter(|s| !s.is_empty()),
    )
}

/// Names of the components whose last beat is older than `after` at `now`.
pub fn stale(beats: &[(String, u64)], now: u64, after: u64) -> Vec<String> {
    beats.iter().filter(|(_, t)| now.saturating_sub(*t) > after).map(|(n, _)| n.clone()).collect()
}

/// What the starter declared: `(supervisor, keep_running)`. Unset reads `none`
/// and `unknown`, never a guess.
pub fn declared() -> (String, String) {
    let sup = std::env::var(SUPERVISOR_ENV).ok().filter(|s| matches!(s.as_str(), "systemd" | "cron" | "none"));
    let keep = std::env::var(KEEP_RUNNING_ENV).ok().filter(|s| matches!(s.as_str(), "on" | "off"));
    (sup.unwrap_or_else(|| "none".into()), keep.unwrap_or_else(|| "unknown".into()))
}

/// Write the heartbeat file atomically (0600) and return the stale components.
pub fn write(dir: &Path, now: u64, beats: &[(String, u64)], after: u64) -> std::io::Result<Vec<String>> {
    let (supervisor, keep_running) = declared();
    let dead = stale(beats, now, after);
    let comps: serde_json::Map<String, Value> = beats.iter().map(|(n, t)| (n.clone(), json!(t))).collect();
    let body = json!({
        "pid": std::process::id(),
        "ts_ms": now,
        "supervisor": supervisor,
        "keep_running": keep_running,
        "stale_after_ms": after,
        "components": comps,
        "stale": dead,
    });
    let tmp = dir.join(format!("{HEARTBEAT_FILE}.tmp"));
    let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
    f.write_all(body.to_string().as_bytes())?;
    f.sync_all()?;
    std::fs::rename(&tmp, dir.join(HEARTBEAT_FILE))?;
    Ok(dead)
}

/// The status view of the last heartbeat: ages rather than timestamps.
pub fn view(dir: &Path, now: u64) -> Value {
    let Some(hb) = std::fs::read_to_string(dir.join(HEARTBEAT_FILE)).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok())
    else {
        return json!({"heartbeat": "none yet"});
    };
    let ts = hb["ts_ms"].as_u64().unwrap_or(0);
    let ages: serde_json::Map<String, Value> = hb["components"]
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), json!(now.saturating_sub(v.as_u64().unwrap_or(0))))).collect())
        .unwrap_or_default();
    json!({
        "supervisor": hb["supervisor"],
        "keep_running": hb["keep_running"],
        "heartbeat_age_ms": now.saturating_sub(ts),
        "component_age_ms": ages,
        "stale": hb["stale"],
    })
}
