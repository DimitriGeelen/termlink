//! T-3708 unit tests for `fields` on a simulated clock (E-19, E-55 freshness; E-56 CLEAR; E-57 V-8).

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// A clock the test moves by hand; `wall_jump` moves the wall time only.
struct FakeClock {
    mono: AtomicU64,
    wall_offset: AtomicU64,
}

impl FakeClock {
    fn new() -> Arc<Self> {
        Arc::new(FakeClock { mono: AtomicU64::new(0), wall_offset: AtomicU64::new(1_000_000_000) })
    }
    fn advance(&self, ms: u64) {
        self.mono.fetch_add(ms, Ordering::SeqCst);
    }
    fn wall_jump(&self, ms: i64) {
        let w = self.wall_offset.load(Ordering::SeqCst) as i64 + ms;
        self.wall_offset.store(w as u64, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn mono_ms(&self) -> u64 {
        self.mono.load(Ordering::SeqCst)
    }
    fn wall_ms(&self) -> u64 {
        self.wall_offset.load(Ordering::SeqCst) + self.mono_ms()
    }
    fn steady_ms(&self) -> u64 {
        1_000_000_000 + self.mono_ms()
    }
}

fn row(id: &str, stage: &str, ts: u64) -> Row {
    Row { msg_id: id.into(), stage: stage.into(), ts_ms: ts }
}

fn fresh(v: Value) -> Value {
    json!({"state": "fresh", "value": v, "at_ms": 1, "bound_ms": 0})
}

fn healthy() -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("hub".into(), fresh(json!({"id": "h", "declared": "h", "match": true})));
    m.insert("revocation".into(), fresh(json!({"revocation": "verified seq 3 at 100", "refused_revoked": 0})));
    m.insert("consumers".into(), fresh(json!({"consumers": "one (proven)"})));
    m.insert("supervision".into(), fresh(json!({"stale": [], "component_age_ms": {"send": 100, "receive": 200}})));
    m.insert("quota".into(), fresh(json!(QUOTA_NOT_BUILT)));
    m.insert("last_surface".into(), fresh(json!({"bad": false})));
    m
}

// E-19 / E-55 (FCW-120): a field past its bound reads unknown, never its last value.
#[test]
fn e19_frozen_field_turns_unknown_after_its_bound() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    r.touch(PULL);
    let v = json!({"match": true});
    let at = render(&v, r.last(PULL), BOUND_TICKS_MS, c.mono_ms());
    assert_eq!(at["state"], "fresh");
    assert_eq!(at["value"], v);
    assert!(at["at_ms"].is_u64() && at["bound_ms"] == BOUND_TICKS_MS);
    c.advance(BOUND_TICKS_MS);
    assert_eq!(render(&v, r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh", "at the bound it is still fresh");
    c.advance(1);
    let late = render(&v, r.last(PULL), BOUND_TICKS_MS, c.mono_ms());
    assert_eq!(late["state"], "unknown");
    assert!(late.get("value").is_none(), "the last value is never served past the bound: {late}");
    assert!(late["shown"].as_str().unwrap().starts_with("unknown (last at "));
}

#[test]
fn e55_never_measured_reads_unknown_not_measured_since_start() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    let v = render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms());
    assert_eq!(v["state"], "unknown");
    assert_eq!(v["shown"], "unknown (not measured since start)");
    assert!(v["last_at_ms"].is_null());
}

// E-59: a wall jump does not change freshness (measured on the monotonic clock) or steady time.
#[test]
fn e59_wall_jump_moves_neither_freshness_nor_steady_time() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    r.touch(PULL);
    let steady = c.steady_ms();
    c.wall_jump(3_600_000);
    assert_eq!(render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh");
    c.wall_jump(-7_200_000);
    assert_eq!(render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh");
    assert_eq!(c.steady_ms(), steady);
}

// E-56 (FCW-121): CLEAR needs real surfacing; a fresh heartbeat alone is never enough.
#[test]
fn e56_refused_confirmations_with_fresh_heartbeat_read_not_clear() {
    let now = 10_000_000;
    // mail stored 10 min ago, never handed over (every confirmation refused); heartbeat is fresh
    let rows = vec![row("m1", "RECEIVED", now - 600_000), row("m1", "STORED", now - 600_000)];
    let ls = last_surface(&rows, Some(now - 600_000), &|_| None, now);
    assert_eq!(ls["bad"], true);
    let mut f = healthy();
    f.insert("last_surface".into(), fresh(ls));
    let c = clear(&f, &rows, now);
    assert_eq!(c["state"], "NOT CLEAR");
    assert!(c["bad"].as_array().unwrap().iter().any(|b| b.as_str().unwrap().starts_with("last_surface")), "{c}");
}

#[test]
fn e56_healthy_receiver_reads_clear_after_the_hand_over() {
    let now = 10_000_000;
    let rows = vec![row("m1", "STORED", now - 600_000), row("m1", "HANDED_OVER", now - 590_000)];
    let mut f = healthy();
    f.insert("last_surface".into(), fresh(last_surface(&rows, Some(now - 600_000), &|_| None, now)));
    assert_eq!(clear(&f, &rows, now)["state"], "CLEAR");
}

#[test]
fn e56_a_sent_message_handed_over_by_the_recipient_is_not_our_surfacing() {
    let now = 10_000_000;
    let rows = vec![
        row("in", "STORED", now - 600_000),
        row("out", "SENT", now - 500_000),
        row("out", "HANDED_OVER", now - 400_000),
    ];
    let f = healthy();
    assert_eq!(clear(&f, &rows, now)["state"], "NOT CLEAR");
}

#[test]
fn clear_counts_unknown_and_each_bad_field() {
    let now = 10_000_000;
    let mut f = healthy();
    f.insert("consumers".into(), json!({"state": "unknown", "shown": "unknown (last at 1)"}));
    let c = clear(&f, &[], now);
    assert_eq!(c["state"], "NOT CLEAR");
    assert_eq!(c["unknown"], json!(["consumers"]));
    for (k, v, needle) in [
        ("hub", json!({"match": false}), "hub"),
        ("revocation", json!({"revocation": "stale since 5 (no pull)"}), "revocation"),
        ("consumers", json!({"consumers": "inbox consumed by 2 identities: a, b"}), "consumers"),
        ("supervision", json!({"stale": ["send"], "component_age_ms": {}}), "supervision"),
        ("supervision", json!({"stale": [], "component_age_ms": {"tick": 3 * TICK_MS + 1}}), "older than 3 ticks"),
    ] {
        let mut f = healthy();
        f.insert(k.into(), fresh(v));
        let c = clear(&f, &[], now);
        assert_eq!(c["state"], "NOT CLEAR", "{k}");
        assert!(c["bad"].to_string().contains(needle), "{k}: {c}");
    }
    let mut f = healthy();
    f.insert("revocation".into(), fresh(json!({"revocation": "unverified since start (no pull yet)"})));
    assert_eq!(clear(&f, &[], now)["unknown"], json!(["revocation"]));
}

// T-3703 r1 finding 5 (carried): the six `revocation` status forms bind, and CLEAR reads each one.
#[test]
fn revocation_vocabulary_binds_and_clear_reads_each_form() {
    for (line, expect) in [
        ("verified seq 7 at 1795150000000", "CLEAR"),
        ("off (PH-1 trust)", "CLEAR"),
        ("unverified since 1795150000000 (hub unreachable: refused)", "unknown"),
        ("unverified since start (no pull yet)", "unknown"),
        ("hub predates revocation list", "unknown"),
        ("stale since 1795150000000 (hub unreachable: refused)", "bad"),
        ("ROLLBACK refused, kept seq 7", "bad"),
    ] {
        let mut f = healthy();
        f.insert("revocation".into(), fresh(json!({"revocation": line, "refused_revoked": 0})));
        let c = clear(&f, &[], 10_000_000);
        let got = if c["state"] == "CLEAR" {
            "CLEAR"
        } else if c["unknown"] == json!(["revocation"]) {
            "unknown"
        } else if c["bad"].to_string().contains("revocation") {
            "bad"
        } else {
            "other"
        };
        assert_eq!(got, expect, "{line}: {c}");
    }
}

// D-2 (operator ruling A): the not-built quota value does not hold CLEAR back.
#[test]
fn d2_quota_not_built_does_not_hold_clear() {
    let f = healthy();
    assert_eq!(f["quota"]["value"], QUOTA_NOT_BUILT);
    assert_eq!(clear(&f, &[], 10_000_000)["state"], "CLEAR");
}

// E-57 (V-8): idle 48 h with nothing pending is never bad, its age is shown.
#[test]
fn e57_idle_48h_with_nothing_pending_is_not_bad() {
    let now = 200 * 3_600_000;
    let rows = vec![row("m0", "STORED", now - 48 * 3_600_000), row("m0", "HANDED_OVER", now - 48 * 3_600_000)];
    let v = last_surface(&rows, Some(now - 48 * 3_600_000), &|_| None, now);
    assert_eq!(v["bad"], false);
    assert_eq!(v["age_ms"], 48 * 3_600_000);
}

#[test]
fn e57_bad_exactly_after_ready_plus_two_ticks() {
    let ready = 1_000_000;
    let rows = vec![row("m1", "STORED", ready - 5_000)];
    assert_eq!(last_surface(&rows, Some(ready), &|_| None, ready + SURFACE_AFTER_MS)["bad"], false);
    let v = last_surface(&rows, Some(ready), &|_| None, ready + SURFACE_AFTER_MS + 1);
    assert_eq!(v["bad"], true);
    assert_eq!(v["overdue"][0]["by"], "ready");
}

#[test]
fn e57_stored_after_ready_counts_from_storing() {
    let ready = 1_000_000;
    let stored = ready + 100_000;
    let rows = vec![row("m1", "STORED", stored)];
    assert_eq!(last_surface(&rows, Some(ready), &|_| None, stored + SURFACE_AFTER_MS)["bad"], false);
    assert_eq!(last_surface(&rows, Some(ready), &|_| None, stored + SURFACE_AFTER_MS + 1)["bad"], true);
}

#[test]
fn e57_bad_two_ticks_after_a_hook_call_that_missed_it() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    let rows = vec![row("m1", "STORED", 0)];
    r.hook_call(&["m1".to_string()]);
    let t = c.steady_ms();
    c.advance(10_000);
    r.hook_call(&["m1".to_string()]); // a later call keeps the first time
    assert_eq!(r.hook_missed_at("m1"), Some(t));
    let missed = |id: &str| r.hook_missed_at(id);
    assert_eq!(last_surface(&rows, None, &missed, t + SURFACE_AFTER_MS)["bad"], false);
    let v = last_surface(&rows, None, &missed, t + SURFACE_AFTER_MS + 1);
    assert_eq!(v["bad"], true);
    assert_eq!(v["overdue"][0]["by"], "hook call");
    // handed over: no longer pending, the record is dropped at the next call
    r.hook_call(&[]);
    assert_eq!(r.hook_missed_at("m1"), None);
}

#[test]
fn e57_no_ready_no_hook_call_is_not_bad_however_old() {
    let rows = vec![row("m1", "STORED", 0)];
    assert_eq!(last_surface(&rows, None, &|_| None, 100 * 3_600_000)["bad"], false);
}

#[test]
fn heartbeat_or_ready_never_counts_as_surfacing() {
    // only HANDED_OVER rows count; READY and heartbeats are not rows at all, and other stages do not count
    let rows = vec![row("m1", "STORED", 0), row("m1", "CLAIMED", 10), row("m1", "HANDOVER_UNCONFIRMED", 20)];
    let v = last_surface(&rows, Some(0), &|_| None, 1_000_000);
    assert!(v["last_surface_ms"].is_null());
    assert_eq!(v["bad"], true);
}

// E-59 control (design mutant 5): the process time every stamp and deadline uses does not follow a
// wall-clock jump of ±1 h; the wall clock (display only) does.
#[test]
fn e59_process_time_ignores_a_wall_jump() {
    let before = crate::store::now_ms();
    let wall_before = crate::store::wall_now_ms();
    crate::store::test_shift_wall_ms(3_600_000);
    let after = crate::store::now_ms();
    let wall_after = crate::store::wall_now_ms();
    crate::store::test_shift_wall_ms(-7_200_000);
    let back = crate::store::now_ms();
    crate::store::test_shift_wall_ms(3_600_000);
    assert!(wall_after >= wall_before + 3_600_000, "the seam moves the wall clock");
    assert!(after >= before && after - before < 60_000, "+1 h jump moved process time by {} ms", after - before);
    assert!(back >= after && back - after < 60_000, "-1 h jump moved process time");
}

#[test]
fn switch_off_hides_the_keys() {
    // The env switch is read per call; default on.
    // SAFETY (test): single-threaded use of this variable in this test binary.
    unsafe { std::env::set_var(SWITCH_ENV, "off") };
    assert!(!enabled());
    unsafe { std::env::remove_var(SWITCH_ENV) };
    assert!(enabled());
}
