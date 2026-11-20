//! T-3708 (P-24 slice a; 06a AL-29, arch 6.7; design `docs/reports/T-3708-design.md` 3.1-3.3).
//!
//! Honest status: every field carries the time its producer last succeeded and a bound; past the
//! bound it reads `unknown (last at T)`, never its last value (FCW-120). CLEAR is computed from the
//! fields and from real hand-overs, never from a heartbeat (FCW-121). Freshness is measured on the
//! monotonic clock; the wall time is shown only. The registry lives in memory, so after a restart
//! every measured field is unknown until its producer has run again (06a 2.13.f).
//!
//! Additive only (design 3.0): this module adds `fields` and `clear` to `status`; it changes no
//! existing key. `TERMLINK_SIDECAR_P24=off` hides both (rollback).

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::deadline::TICK_MS;

/// Producer names the registry stamps.
pub const PULL: &str = "pull";
pub const CONSUMERS: &str = "consumers";
pub const HEARTBEAT: &str = "heartbeat";
pub const LIVENESS: &str = "liveness";
pub const SKEW: &str = "skew";
pub const NTP: &str = "ntp";

/// Bounds (06a 2.13.c): per-tick fields 2 ticks, heartbeat 60 s, daily-derived 1 h.
pub const BOUND_TICKS_MS: u64 = 2 * TICK_MS;
pub const BOUND_HEARTBEAT_MS: u64 = 60_000;
pub const BOUND_DAILY_MS: u64 = 3_600_000;
/// V-8: a pending message is overdue for surfacing 2 ticks after READY or a hook call that missed it.
pub const SURFACE_AFTER_MS: u64 = 2 * TICK_MS;
/// A component heartbeat older than this is a bad field.
pub const COMPONENT_BAD_MS: u64 = 3 * TICK_MS;
/// CLEAR looks back this far for pending mail.
pub const CLEAR_WINDOW_MS: u64 = 3_600_000;

/// The value `quota` reads until receiver quotas exist (T-3708 D-2, operator ruling A).
pub const QUOTA_NOT_BUILT: &str = "absent: receiver quotas not built (P-17)";

/// The switch for the new keys (design 3.0 rollback).
pub const SWITCH_ENV: &str = "TERMLINK_SIDECAR_P24";

/// Whether the P-24 keys are shown (default on; `off` hides them).
pub fn enabled() -> bool {
    std::env::var(SWITCH_ENV).map(|v| v.trim() != "off").unwrap_or(true)
}

/// Time source: monotonic for freshness and deadlines, wall for display.
pub trait Clock: Send + Sync {
    fn mono_ms(&self) -> u64;
    fn wall_ms(&self) -> u64;
    /// Wall time at start plus monotonic time since: a wall-shaped time that a wall jump does not move
    /// (design 3.5.3). Deadlines compare against this.
    fn steady_ms(&self) -> u64;
}

/// The process clock.
pub struct SystemClock {
    start: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        SystemClock { start: Instant::now() }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn mono_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
    fn wall_ms(&self) -> u64 {
        crate::store::wall_now_ms()
    }
    fn steady_ms(&self) -> u64 {
        // the same timeline as every stamp and deadline of this process
        crate::store::now_ms()
    }
}

/// When a producer last succeeded: monotonic and wall time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub mono_ms: u64,
    pub wall_ms: u64,
}

/// The in-memory registry of producer successes and first missed hook calls.
pub struct Registry {
    clock: Arc<dyn Clock>,
    seen: Mutex<BTreeMap<String, Seen>>,
    /// msg_id -> steady time of the first hook call that ran while it was pending (V-8).
    hook_missed: Mutex<BTreeMap<String, u64>>,
}

impl Registry {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Registry { clock, seen: Mutex::new(BTreeMap::new()), hook_missed: Mutex::new(BTreeMap::new()) }
    }

    pub fn clock(&self) -> &dyn Clock {
        &*self.clock
    }

    /// A producer succeeded now.
    pub fn touch(&self, name: &str) {
        let s = Seen { mono_ms: self.clock.mono_ms(), wall_ms: self.clock.wall_ms() };
        lock(&self.seen).insert(name.to_string(), s);
    }

    pub fn last(&self, name: &str) -> Option<Seen> {
        lock(&self.seen).get(name).copied()
    }

    /// A hook call ran while these messages were pending: keep the first such time per message.
    pub fn hook_call(&self, pending: &[String]) {
        let now = self.clock.steady_ms();
        let mut m = lock(&self.hook_missed);
        m.retain(|k, _| pending.contains(k));
        for id in pending {
            m.entry(id.clone()).or_insert(now);
        }
    }

    pub fn hook_missed_at(&self, msg_id: &str) -> Option<u64> {
        lock(&self.hook_missed).get(msg_id).copied()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match m.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

/// The process registry (system clock).
pub fn global() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| Registry::new(Arc::new(SystemClock::new())))
}

/// Stamp a producer success in the process registry.
pub fn touch(name: &str) {
    global().touch(name);
}

/// Steady time for deadline checks (design 3.5.3).
pub fn steady_ms() -> u64 {
    global().clock().steady_ms()
}

/// Render one measured field: fresh with its times, or `unknown (last at T)` past the bound (FCW-120).
pub fn render(value: &Value, seen: Option<Seen>, bound_ms: u64, now_mono: u64) -> Value {
    match seen {
        Some(s) if now_mono.saturating_sub(s.mono_ms) <= bound_ms => {
            json!({"state": "fresh", "value": value, "at_ms": s.wall_ms, "bound_ms": bound_ms})
        }
        Some(s) => json!({"state": "unknown", "shown": format!("unknown (last at {})", s.wall_ms), "last_at_ms": s.wall_ms, "bound_ms": bound_ms}),
        None => json!({"state": "unknown", "shown": "unknown (not measured since start)", "last_at_ms": null, "bound_ms": bound_ms}),
    }
}

/// A field computed at this call from local state: fresh by construction.
pub fn computed(value: &Value, now_wall: u64) -> Value {
    json!({"state": "fresh", "value": value, "at_ms": now_wall, "bound_ms": 0})
}

/// One chain row as `status` lists it (the subset V-8 and CLEAR read).
#[derive(Debug, Clone)]
pub struct Row {
    pub msg_id: String,
    pub stage: String,
    pub ts_ms: u64,
}

/// Rows of `status.rows` as [`Row`]s.
pub fn rows_of(status_rows: &Value) -> Vec<Row> {
    status_rows
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            Some(Row {
                msg_id: r.get("msg_id")?.as_str()?.to_string(),
                stage: r.get("stage")?.as_str()?.to_string(),
                ts_ms: r.get("peer_ts_ms").and_then(Value::as_u64).or_else(|| r.get("ts_ms").and_then(Value::as_u64))?,
            })
        })
        .collect()
}

/// Messages this agent received and stored but has not handed over: (msg_id, stored at).
/// A message this side SENT is never pending here.
pub fn pending(rows: &[Row]) -> Vec<(String, u64)> {
    let mut ids: Vec<&str> = rows.iter().filter(|r| r.stage == "STORED").map(|r| r.msg_id.as_str()).collect();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .filter(|id| !rows.iter().any(|r| r.msg_id == *id && (r.stage == "HANDED_OVER" || r.stage == "SENT")))
        .filter_map(|id| rows.iter().filter(|r| r.msg_id == id && r.stage == "STORED").map(|r| r.ts_ms).min().map(|t| (id.to_string(), t)))
        .collect()
}

/// Times of hand-overs this side made: HANDED_OVER lines of messages it received (a HANDED_OVER of a
/// message it SENT is the recipient's, reported back, and is not this agent's surfacing).
fn own_handovers(rows: &[Row]) -> impl Iterator<Item = u64> + '_ {
    rows.iter()
        .filter(|r| r.stage == "HANDED_OVER" && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
        .map(|r| r.ts_ms)
}

/// V-8 (06a 2.13.c, arch 6.7.3): the last real surfacing and whether it is bad.
/// Bad iff a pending message was not handed over 2 ticks after the agent became ready (counted from
/// the later of storing and readiness, as R-44.e) or 2 ticks after a hook call that missed it.
/// An idle agent with nothing pending is never bad; its age is shown. Only a HANDED_OVER line counts
/// as surfacing; a heartbeat or READY never does.
pub fn last_surface(rows: &[Row], ready_since: Option<u64>, hook_missed: &dyn Fn(&str) -> Option<u64>, now: u64) -> Value {
    let last = own_handovers(rows).max();
    let mut overdue = vec![];
    for (id, stored) in pending(rows) {
        let by_ready = ready_since.map(|t| t.max(stored)).filter(|t| now.saturating_sub(*t) > SURFACE_AFTER_MS);
        let by_hook = hook_missed(&id).filter(|t| now.saturating_sub(*t) > SURFACE_AFTER_MS);
        if let Some(t) = by_ready.or(by_hook) {
            overdue.push(json!({"msg_id": id, "since_ms": t, "by": if by_ready.is_some() { "ready" } else { "hook call" }}));
        }
    }
    json!({
        "last_surface_ms": last,
        "age_ms": last.map(|t| now.saturating_sub(t)),
        "bad": !overdue.is_empty(),
        "overdue": overdue,
    })
}

/// CLEAR (FCW-121): no field bad, no field unknown, and if anything was pending in the last hour, a
/// HANDED_OVER after the earliest such STORED. Returns `{state, bad, unknown, why}`.
pub fn clear(fields: &serde_json::Map<String, Value>, rows: &[Row], now: u64) -> Value {
    let mut bad: Vec<String> = vec![];
    let mut unknown: Vec<String> = vec![];
    for (name, f) in fields {
        if f["state"] != "fresh" {
            unknown.push(name.clone());
            continue;
        }
        let v = &f["value"];
        match name.as_str() {
            "hub" if v["match"] != Value::Bool(true) => bad.push("hub: does not match the declared hub".into()),
            "revocation" => {
                let s = v["revocation"].as_str().unwrap_or("");
                if s.starts_with("stale") || s.starts_with("ROLLBACK") {
                    bad.push(format!("revocation: {s}"));
                } else if s.starts_with("unverified") || s == "hub predates revocation list" {
                    // no verified list: revoked keys are not known here
                    unknown.push("revocation".into());
                }
            }
            "consumers" => {
                let s = v["consumers"].as_str().unwrap_or("");
                if s.starts_with("unknown") {
                    unknown.push("consumers".into());
                } else if s != "one (proven)" {
                    bad.push(format!("consumers: {s}"));
                }
            }
            "supervision" => {
                if v.get("heartbeat").is_some() {
                    unknown.push("supervision".into());
                } else {
                    if v["stale"].as_array().is_some_and(|a| !a.is_empty()) {
                        bad.push("supervision: a component is dead".into());
                    }
                    let old: Vec<&String> = v["component_age_ms"]
                        .as_object()
                        .map(|m| m.iter().filter(|(_, a)| a.as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS).map(|(k, _)| k).collect())
                        .unwrap_or_default();
                    if !old.is_empty() {
                        bad.push(format!("supervision: heartbeat older than 3 ticks: {}", old.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                    }
                }
            }
            "last_surface" if v["bad"] == Value::Bool(true) => bad.push("last_surface: pending mail not handed over".into()),
            "revocation_invariants" if v["broken"].as_array().is_some_and(|a| !a.is_empty()) => {
                bad.push(format!("revocation_invariants: {}", v["broken"]))
            }
            _ => {}
        }
    }
    // Pending in the last hour needs a real surfacing after the earliest such STORED.
    let since = now.saturating_sub(CLEAR_WINDOW_MS);
    let earliest = rows
        .iter()
        .filter(|r| r.stage == "STORED" && r.ts_ms >= since && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
        .map(|r| r.ts_ms)
        .min();
    let surfaced = earliest.is_none_or(|e| own_handovers(rows).any(|t| t >= e));
    if !surfaced {
        bad.push("no hand-over after mail pending in the last hour".into());
    }
    let state = if bad.is_empty() && unknown.is_empty() { "CLEAR" } else { "NOT CLEAR" };
    json!({"state": state, "bad": bad, "unknown": unknown})
}

#[cfg(test)]
#[path = "fields_tests.rs"]
mod tests;
