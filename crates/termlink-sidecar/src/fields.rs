//! T-3708 (P-24 slice a; 06a AL-29, arch 6.7; design `docs/reports/T-3708-design.md` 3.1-3.3).
//!
//! Honest status: every field carries the time its producer last succeeded and a bound; past the
//! bound it reads `unknown (last at T)`, never its last value (FCW-120). CLEAR is computed from the
//! fields and from real hand-overs, never from a heartbeat (FCW-121). Freshness is measured on the
//! monotonic clock; the wall time is shown only. The registry lives in memory, so after a restart
//! every measured field is unknown until its producer has run again (06a 2.13.f).
//!
//! Time (r1 BLOCKER): every stored or sent stamp stays wall time (`store::now_ms`). A deadline is an
//! AGE, and [`Registry::age_ms`] corrects the wall age of a stamp by every wall jump this process saw
//! after the stamp (a jump = wall and monotonic time disagreeing by more than 2 s between two looks,
//! taken at least every heartbeat tick), so a wall jump moves no deadline (E-59). Without a jump the
//! age is plain `now - stamp`, as before.
//!
//! Additive only (design 3.0): this module adds `fields` and `clear` to `status`; it changes no
//! existing key. `TERMLINK_SIDECAR_P24=off|0|false|no` hides them (rollback).

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::deadline::TICK_MS;
use crate::store::Store;

/// Producer names the registry stamps.
pub const PULL: &str = "pull";
pub const REVOCATION: &str = "revocation";
pub const CONSUMERS: &str = "consumers";
pub const HEARTBEAT: &str = "heartbeat";
pub const LIVENESS: &str = "liveness";
pub const LEASE: &str = "lease";
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
/// CLEAR looks back this far for mail that was pending.
pub const CLEAR_WINDOW_MS: u64 = 3_600_000;
/// Wall and monotonic time disagreeing by more than this between two looks is a wall jump.
pub const JUMP_MS: i64 = 2_000;
/// At most this many jumps are kept (oldest first out).
pub const JUMPS_CAP: usize = 64;
/// At most this many stamps are remembered (lowest first out).
pub const AGES_CAP: usize = 20_000;

/// The value `quota` reads until receiver quotas exist (T-3708 D-2, carried provisionally: see the task).
pub const QUOTA_NOT_BUILT: &str = "absent: receiver quotas not built (P-17)";

/// The switch for the new keys (design 3.0 rollback).
pub const SWITCH_ENV: &str = "TERMLINK_SIDECAR_P24";

/// Whether the P-24 keys are shown (default on; `off`, `0`, `false`, `no` hide them, any case).
pub fn enabled() -> bool {
    match std::env::var(SWITCH_ENV) {
        Ok(v) => !matches!(v.trim().to_ascii_lowercase().as_str(), "off" | "0" | "false" | "no"),
        Err(_) => true,
    }
}

/// Time source: monotonic for freshness and ages, wall for stamps and display.
pub trait Clock: Send + Sync {
    fn mono_ms(&self) -> u64;
    fn wall_ms(&self) -> u64;
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

/// CLOCK_BOOTTIME in ms: monotonic, and it keeps counting through a suspend or VM pause (r2 MAJOR 3), which
/// `Instant` (CLOCK_MONOTONIC on Linux) does not. Falls back to `Instant` if the call fails.
fn boottime_ms() -> Option<u64> {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes into the timespec we own; CLOCK_BOOTTIME is a valid clock id on Linux.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
    (rc == 0).then(|| ts.tv_sec as u64 * 1000 + ts.tv_nsec as u64 / 1_000_000)
}

impl Clock for SystemClock {
    fn mono_ms(&self) -> u64 {
        boottime_ms().unwrap_or_else(|| self.start.elapsed().as_millis() as u64)
    }
    fn wall_ms(&self) -> u64 {
        crate::store::wall_now_ms()
    }
}

/// When a producer last succeeded: monotonic and wall time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub mono_ms: u64,
    pub wall_ms: u64,
}

/// The in-memory registry of producer successes and of the ages of stamps this process has seen.
pub struct Registry {
    clock: Arc<dyn Clock>,
    seen: Mutex<BTreeMap<String, Seen>>,
    /// The last look at both clocks (mono, wall), and the wall jumps seen: (wall just before, delta).
    last_look: Mutex<Option<(u64, u64)>>,
    jumps: Mutex<Vec<(u64, i64)>>,
    /// stamp -> how many jumps had been seen when this process WROTE it ([`Registry::register`], r2 MAJOR 1).
    /// A stamp this process did not write (from before its start) predates all its jumps: epoch 0. Wall
    /// values alone cannot say which side of a backward jump a stamp is on.
    epochs: Mutex<BTreeMap<u64, usize>>,
}

impl Registry {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Registry { clock, seen: Mutex::new(BTreeMap::new()), last_look: Mutex::new(None), jumps: Mutex::new(vec![]), epochs: Mutex::new(BTreeMap::new()) }
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

    /// Look at both clocks; a disagreement past [`JUMP_MS`] since the last look is a wall jump. Both clocks
    /// are read under the lock, so two threads cannot record a false jump between them (r2 MINOR 7).
    pub fn look(&self) {
        let mut last = lock(&self.last_look);
        let (mono, wall) = (self.clock.mono_ms(), self.clock.wall_ms());
        if let Some((m0, w0)) = *last {
            let delta = (wall as i64 - w0 as i64) - (mono as i64 - m0 as i64);
            if delta.abs() > JUMP_MS {
                let mut j = lock(&self.jumps);
                if j.len() >= JUMPS_CAP {
                    // dropping the oldest jump shifts every index down by one
                    j.remove(0);
                    for e in lock(&self.epochs).values_mut() {
                        *e = e.saturating_sub(1);
                    }
                }
                j.push((w0 + mono.saturating_sub(m0), delta));
            }
        }
        *last = Some((mono, wall));
    }

    /// This process wrote `stamp` now: remember how many jumps came before it.
    pub fn register(&self, stamp_wall: u64) {
        self.look();
        let n = lock(&self.jumps).len();
        let mut e = lock(&self.epochs);
        if !e.contains_key(&stamp_wall) && e.len() >= AGES_CAP {
            if let Some(k) = e.keys().next().copied() {
                e.remove(&k);
            }
        }
        e.entry(stamp_wall).or_insert(n);
    }

    /// The age of a wall stamp at `now_wall`, less every wall jump that came after the stamp was written
    /// (E-59). A jump while the process was down is not seen here; the skew line shows it.
    pub fn age_ms(&self, stamp_wall: u64, now_wall: u64) -> u64 {
        self.look();
        let jumps = lock(&self.jumps);
        let epoch = lock(&self.epochs).get(&stamp_wall).copied().unwrap_or(0);
        let correction: i64 = jumps.iter().skip(epoch).map(|(_, d)| *d).sum();
        (now_wall as i64 - stamp_wall as i64 - correction).max(0) as u64
    }

    #[cfg(test)]
    pub(crate) fn jumps_len(&self) -> usize {
        lock(&self.jumps).len()
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

/// Age of a wall stamp in the process registry (see [`Registry::age_ms`]).
pub fn age_ms(stamp_wall: u64, now_wall: u64) -> u64 {
    global().age_ms(stamp_wall, now_wall)
}

/// Register a stamp this process writes (chain events, readiness, hook misses).
pub fn register(stamp_wall: u64) {
    global().register(stamp_wall);
}

/// The wall time now, registered as a stamp this process writes.
pub fn stamp_now() -> u64 {
    let t = crate::store::now_ms();
    register(t);
    t
}

/// Look at the clocks (called every heartbeat tick, so a jump is seen even when no age is asked for).
pub fn look() {
    global().look();
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

/// One chain row as `status` lists it (the subset V-8 and CLEAR read). Times are this side's own
/// stamps (`ts_ms`), never a peer's (r1 MAJOR 6).
#[derive(Debug, Clone)]
pub struct Row {
    pub msg_id: String,
    pub stage: String,
    pub ts_ms: u64,
}

/// Rows of `status.rows` as [`Row`]s. A row without its own time is aged as the oldest possible (0),
/// which can only make it look more overdue, never less (r1 MINOR 13).
pub fn rows_of(status_rows: &Value) -> Vec<Row> {
    status_rows
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            Some(Row {
                msg_id: r.get("msg_id")?.as_str()?.to_string(),
                stage: r.get("stage")?.as_str()?.to_string(),
                ts_ms: r.get("ts_ms").and_then(Value::as_u64).unwrap_or(0),
            })
        })
        .collect()
}

/// Messages this agent received and stored but has not handed over: (msg_id, stored at). A message
/// this side SENT is never pending here. `deliverable`, when given, narrows to the messages the
/// deliver path may still offer (a held, refused or otherwise closed message is not pending).
pub fn pending(rows: &[Row], deliverable: Option<&[String]>) -> Vec<(String, u64)> {
    let mut ids: Vec<&str> = rows.iter().filter(|r| r.stage == "STORED").map(|r| r.msg_id.as_str()).collect();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .filter(|id| !rows.iter().any(|r| r.msg_id == *id && (r.stage == "HANDED_OVER" || r.stage == "SENT")))
        .filter(|id| deliverable.is_none_or(|d| d.iter().any(|x| x == id)))
        .filter_map(|id| rows.iter().filter(|r| r.msg_id == id && r.stage == "STORED").map(|r| r.ts_ms).min().map(|t| (id.to_string(), t)))
        .collect()
}

/// Times of hand-overs this side made: HANDED_OVER lines of messages it received (a HANDED_OVER of a
/// message it SENT is the recipient's, reported back, and is not this agent's surfacing). A
/// HANDED_OVER line is written only with hand-over evidence (an unconfirmed attempt is a different
/// stage), so the line is the evidence.
fn own_handovers(rows: &[Row]) -> impl Iterator<Item = u64> + '_ {
    rows.iter()
        .filter(|r| r.stage == "HANDED_OVER" && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
        .map(|r| r.ts_ms)
}

/// V-8 hook misses, persisted (r1 MAJOR 7): the first hook call that ran while a message was pending.
const HOOK_MISSED_STATE: &str = "v8-hook-missed";

/// A hook call ran now while `pending` was pending: keep the first time per message, drop the rest.
pub fn record_hook_call(store: &Store, pending: &[String], now_wall: u64) {
    let old = hook_missed(store);
    let mut m: BTreeMap<String, u64> = old.iter().filter(|(k, _)| pending.contains(k)).map(|(k, v)| (k.clone(), *v)).collect();
    for id in pending {
        m.entry(id.clone()).or_insert_with(|| {
            register(now_wall);
            now_wall
        });
    }
    if m != old {
        let text: String = m.iter().map(|(k, t)| format!("{k} {t}\n")).collect();
        if let Err(e) = store.write_state(HOOK_MISSED_STATE, &text) {
            eprintln!("termlink-sidecar: V-8 hook-miss record not written: {e}");
        }
    }
}

/// msg_id -> wall time of the first hook call that missed it.
pub fn hook_missed(store: &Store) -> BTreeMap<String, u64> {
    store
        .read_state(HOOK_MISSED_STATE)
        .ok()
        .flatten()
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (k, t) = l.split_once(' ')?;
            Some((k.to_string(), t.trim().parse().ok()?))
        })
        .collect()
}

/// V-8 (06a 2.13.c, arch 6.7.3): the last real surfacing and whether it is bad. Bad iff a pending
/// message was not handed over 2 ticks after the agent became ready (counted from the later of storing
/// and readiness, as R-44.e) or 2 ticks after a hook call that missed it. `age(stamp)` gives the
/// jump-immune age of a wall stamp. An idle agent with nothing pending is never bad; its age is shown.
pub fn last_surface(
    rows: &[Row],
    pending: &[(String, u64)],
    ready_since: Option<u64>,
    hook_missed: &BTreeMap<String, u64>,
    age: &dyn Fn(u64) -> u64,
) -> Value {
    let last = own_handovers(rows).max();
    let mut overdue = vec![];
    let mut unanchored = vec![];
    for (id, stored) in pending {
        // the later of storing and readiness is the younger of the two ages
        let by_ready = ready_since.map(|r| age(*stored).min(age(r))).filter(|a| *a > SURFACE_AFTER_MS);
        let by_hook = hook_missed.get(id).map(|t| age(*t)).filter(|a| *a > SURFACE_AFTER_MS);
        if let Some(a) = by_ready.or(by_hook) {
            overdue.push(json!({"msg_id": id, "overdue_by_ms": a - SURFACE_AFTER_MS, "by": if by_ready.is_some() { "ready" } else { "hook call" }}));
        } else if ready_since.is_none() && !hook_missed.contains_key(id) && age(*stored) > SURFACE_AFTER_MS {
            // r2 MAJOR 4: pending mail with no READY and no hook call to measure from: surfacing cannot be judged
            unanchored.push(json!(id));
        }
    }
    json!({
        "last_surface_ms": last,
        "age_ms": last.map(age),
        "bad": !overdue.is_empty(),
        "overdue": overdue,
        "unknown": !unanchored.is_empty(),
        "unanchored": unanchored,
    })
}

/// The revocation status forms (T-3703 design 5.5), as CLEAR reads them: ok, unknown or bad. Any other
/// line is bad: a form this code does not know is never read as fine (r1 MAJOR 5).
pub fn revocation_reading(line: &str) -> &'static str {
    if line.starts_with("verified seq ") || line == "off (PH-1 trust)" {
        "ok"
    } else if line.starts_with("unverified since ") || line == "hub predates revocation list" {
        "unknown"
    } else {
        "bad"
    }
}

/// CLEAR (FCW-121): no field bad, no field unknown, and if any mail was pending in the last hour (still
/// pending now at any age, or handed over within the hour) a hand-over this side made after the
/// earliest such STORED. Returns `{state, bad, unknown, why}`.
pub fn clear(fields: &serde_json::Map<String, Value>, rows: &[Row], pending_now: &[(String, u64)], now_wall: u64) -> Value {
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
                let s = v["revocation"].as_str().unwrap_or("(no line)");
                match revocation_reading(s) {
                    "ok" => {}
                    "unknown" => unknown.push("revocation".into()),
                    _ => bad.push(format!("revocation: {s}")),
                }
            }
            "consumers" => {
                let s = v["consumers"].as_str().unwrap_or("(no line)");
                if s.starts_with("unknown") {
                    unknown.push("consumers".into());
                } else if s != "one (proven)" {
                    bad.push(format!("consumers: {s}"));
                }
            }
            "supervision" => {
                if v.get("heartbeat").is_some() || !v["heartbeat_age_ms"].is_u64() {
                    unknown.push("supervision".into());
                } else {
                    if v["stale"].as_array().is_some_and(|a| !a.is_empty()) || !v["stale"].is_array() {
                        bad.push(format!("supervision: dead or unreadable component list: {}", v["stale"]));
                    }
                    if v["heartbeat_age_ms"].as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS {
                        bad.push("supervision: heartbeat older than 3 ticks".into());
                    }
                    let old: Vec<String> = v["component_age_ms"]
                        .as_object()
                        .map(|m| m.iter().filter(|(_, a)| a.as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS).map(|(k, _)| k.clone()).collect())
                        .unwrap_or_else(|| vec!["(no component ages)".into()]);
                    if !old.is_empty() {
                        bad.push(format!("supervision: component heartbeat older than 3 ticks: {}", old.join(", ")));
                    }
                }
            }
            "last_surface" if v["bad"] == Value::Bool(true) => bad.push("last_surface: pending mail not handed over".into()),
            "last_surface" if v["unknown"] == Value::Bool(true) => {
                unknown.push("last_surface (pending mail with no READY or hook call to measure from)".into())
            }
            "revocation_invariants" if v["broken"].as_array().is_some_and(|a| !a.is_empty()) => {
                bad.push(format!("revocation_invariants: {}", v["broken"]))
            }
            _ => {}
        }
    }
    let since = now_wall.saturating_sub(CLEAR_WINDOW_MS);
    let handed_in_window: Vec<u64> = rows
        .iter()
        .filter(|r| r.stage == "HANDED_OVER" && r.ts_ms >= since && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
        .filter_map(|h| rows.iter().filter(|r| r.msg_id == h.msg_id && r.stage == "STORED").map(|r| r.ts_ms).min())
        .collect();
    let earliest = pending_now.iter().map(|(_, t)| *t).chain(handed_in_window).min();
    let surfaced = earliest.is_none_or(|e| own_handovers(rows).any(|t| t >= e));
    if !surfaced {
        bad.push("no hand-over after mail that was pending in the last hour".into());
    }
    let state = if bad.is_empty() && unknown.is_empty() { "CLEAR" } else { "NOT CLEAR" };
    let why = match (bad.first(), unknown.first()) {
        (Some(b), _) => b.clone(),
        (None, Some(u)) => format!("{u}: unknown"),
        (None, None) => "every field fresh and good; mail surfaced".into(),
    };
    json!({"state": state, "bad": bad, "unknown": unknown, "why": why})
}

#[cfg(test)]
#[path = "fields_tests.rs"]
mod tests;
