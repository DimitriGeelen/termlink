//! T-3708 unit tests for `fields` on a simulated clock (E-19, E-55 freshness; E-56 CLEAR; E-57 V-8; E-59 ages).
//! Nothing here changes process-wide state (r1 MINOR 11): the wall-shift and switch tests have their own binaries.

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
}

fn row(id: &str, stage: &str, ts: u64) -> Row {
    Row { msg_id: id.into(), stage: stage.into(), ts_ms: ts }
}

fn fresh(v: Value) -> Value {
    json!({"state": "fresh", "value": v, "at_ms": 1, "bound_ms": 0})
}

fn supervision_ok() -> Value {
    json!({"supervisor": "systemd", "keep_running": true, "heartbeat_age_ms": 100, "component_age_ms": {"send": 100, "receive": 200}, "stale": []})
}

fn healthy() -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("hub".into(), fresh(json!({"id": "h", "declared": "h", "match": true})));
    m.insert("revocation".into(), fresh(json!({"revocation": "verified seq 3 at 100", "refused_revoked": 0})));
    m.insert("consumers".into(), fresh(json!({"consumers": "one (proven)"})));
    m.insert("supervision".into(), fresh(supervision_ok()));
    m.insert("quota".into(), fresh(json!(QUOTA_NOT_BUILT)));
    m.insert("last_surface".into(), fresh(json!({"bad": false})));
    m
}

/// Ages on the plain wall clock, for tests that do not jump.
fn wall_age(now: u64) -> impl Fn(u64) -> u64 {
    move |t: u64| now.saturating_sub(t)
}

// r1 MAJOR 6: rows carry this side's own time, never the peer's.
#[test]
fn rows_use_the_local_stamp_not_the_peer_clock() {
    let rows = rows_of(&json!([{"msg_id": "m1", "stage": "STORED", "ts_ms": 1_000, "peer_ts_ms": 9_000_000}]));
    assert_eq!(rows[0].ts_ms, 1_000);
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

// E-59: freshness and ages do not follow a wall jump; the age keeps counting on the monotonic clock.
#[test]
fn e59_wall_jump_moves_neither_freshness_nor_ages() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    r.touch(PULL);
    let stamp = c.wall_ms() - 10_000; // a message stored 10 s ago
    assert_eq!(r.age_ms(stamp, c.wall_ms()), 10_000);
    c.wall_jump(3_600_000);
    assert_eq!(render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh");
    assert_eq!(r.age_ms(stamp, c.wall_ms()), 10_000, "+1 h wall jump must not age the stamp");
    c.wall_jump(-7_200_000);
    assert_eq!(r.age_ms(stamp, c.wall_ms()), 10_000, "-1 h wall jump must not make it younger");
    c.advance(5_000);
    assert_eq!(r.age_ms(stamp, c.wall_ms()), 15_000, "it ages on the monotonic clock");
    // a stamp written after the jumps is aged plainly
    let late = c.wall_ms() - 1_000;
    assert_eq!(r.age_ms(late, c.wall_ms()), 1_000);
}

#[test]
fn e59_v8_deadline_neither_early_nor_late_across_a_jump() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    let ready = c.wall_ms();
    let rows = vec![row("m1", "STORED", ready)];
    let pending = pending(&rows, None);
    let age = |t: u64| r.age_ms(t, c.wall_ms());
    let empty = BTreeMap::new();
    assert_eq!(last_surface(&rows, &pending, Some(ready), &empty, &age)["bad"], false);
    c.wall_jump(3_600_000); // +1 h: a wall-based deadline would fire at once
    assert_eq!(last_surface(&rows, &pending, Some(ready), &empty, &age)["bad"], false, "fired early on a forward jump");
    c.wall_jump(-7_200_000);
    c.advance(SURFACE_AFTER_MS + 1); // -1 h then 2 ticks: a wall-based deadline would wait another hour
    assert_eq!(last_surface(&rows, &pending, Some(ready), &empty, &age)["bad"], true, "late after a backward jump");
}

#[test]
fn jump_record_is_bounded_and_slow_drift_is_not_a_jump() {
    let c = FakeClock::new();
    let r = Registry::new(c.clone());
    r.look();
    for _ in 0..(JUMPS_CAP + 10) {
        c.wall_jump(10_000);
        r.look();
    }
    assert_eq!(r.jumps_len(), JUMPS_CAP);
    let c2 = FakeClock::new();
    let r2 = Registry::new(c2.clone());
    r2.look();
    c2.advance(30_000);
    c2.wall_jump(15); // NTP slew over one tick
    r2.look();
    assert_eq!(r2.jumps_len(), 0);
}

// E-56 (FCW-121): CLEAR needs real surfacing; a fresh heartbeat alone is never enough.
#[test]
fn e56_refused_confirmations_with_fresh_heartbeat_read_not_clear() {
    let now = 10_000_000;
    // mail stored 10 min ago, never handed over (every confirmation refused); heartbeat is fresh
    let rows = vec![row("m1", "RECEIVED", now - 600_000), row("m1", "STORED", now - 600_000)];
    let p = pending(&rows, None);
    let ls = last_surface(&rows, &p, Some(now - 600_000), &BTreeMap::new(), &wall_age(now));
    assert_eq!(ls["bad"], true);
    let mut f = healthy();
    f.insert("last_surface".into(), fresh(ls));
    let c = clear(&f, &rows, &p, now);
    assert_eq!(c["state"], "NOT CLEAR");
    assert!(c["bad"].as_array().unwrap().iter().any(|b| b.as_str().unwrap().starts_with("last_surface")), "{c}");
    assert!(c["why"].as_str().is_some_and(|w| !w.is_empty()));
}

#[test]
fn e56_healthy_receiver_reads_clear_after_the_hand_over() {
    let now = 10_000_000;
    let rows = vec![row("m1", "STORED", now - 600_000), row("m1", "HANDED_OVER", now - 590_000)];
    let p = pending(&rows, None);
    let mut f = healthy();
    f.insert("last_surface".into(), fresh(last_surface(&rows, &p, Some(now - 600_000), &BTreeMap::new(), &wall_age(now))));
    assert_eq!(clear(&f, &rows, &p, now)["state"], "CLEAR");
}

// r1 MAJOR 2: mail pending for longer than the hour still holds CLEAR (no READY, no hook call: a hung agent).
#[test]
fn mail_pending_for_more_than_an_hour_still_reads_not_clear() {
    let now = 100_000_000;
    let rows = vec![row("m1", "STORED", now - 2 * CLEAR_WINDOW_MS)];
    let p = pending(&rows, None);
    assert_eq!(clear(&healthy(), &rows, &p, now)["state"], "NOT CLEAR");
}

#[test]
fn e56_a_sent_message_handed_over_by_the_recipient_is_not_our_surfacing() {
    let now = 10_000_000;
    let rows = vec![row("in", "STORED", now - 600_000), row("out", "SENT", now - 500_000), row("out", "HANDED_OVER", now - 400_000)];
    let p = pending(&rows, None);
    assert_eq!(clear(&healthy(), &rows, &p, now)["state"], "NOT CLEAR");
}

#[test]
fn a_message_the_deliver_path_cannot_offer_is_not_pending() {
    let rows = vec![row("held", "STORED", 0), row("open", "STORED", 0)];
    let deliverable = vec!["open".to_string()];
    let p = pending(&rows, Some(&deliverable));
    assert_eq!(p.iter().map(|(i, _)| i.as_str()).collect::<Vec<_>>(), ["open"]);
}

#[test]
fn clear_counts_unknown_and_each_bad_field() {
    let now = 10_000_000;
    let mut f = healthy();
    f.insert("consumers".into(), json!({"state": "unknown", "shown": "unknown (last at 1)"}));
    let c = clear(&f, &[], &[], now);
    assert_eq!(c["state"], "NOT CLEAR");
    assert_eq!(c["unknown"], json!(["consumers"]));
    let dead = json!({"heartbeat_age_ms": 100, "component_age_ms": {}, "stale": ["send"]});
    let old = json!({"heartbeat_age_ms": 100, "component_age_ms": {"tick": 3 * TICK_MS + 1}, "stale": []});
    let old_beat = json!({"heartbeat_age_ms": 3 * TICK_MS + 1, "component_age_ms": {}, "stale": []});
    for (k, v, needle) in [
        ("hub", json!({"match": false}), "hub"),
        ("revocation", json!({"revocation": "stale since 5 (no pull)"}), "revocation"),
        ("revocation", json!({"revocation": "error: bad signature"}), "revocation"),
        ("revocation", json!({"revocation": 7}), "revocation"),
        ("consumers", json!({"consumers": "inbox consumed by 2 identities: a, b"}), "consumers"),
        ("supervision", dead, "dead"),
        ("supervision", old, "older than 3 ticks"),
        ("supervision", old_beat, "heartbeat older"),
    ] {
        let mut f = healthy();
        f.insert(k.into(), fresh(v));
        let c = clear(&f, &[], &[], now);
        assert_eq!(c["state"], "NOT CLEAR", "{k}");
        assert!(c["bad"].to_string().contains(needle), "{k}: {c}");
    }
    let mut f = healthy();
    f.insert("supervision".into(), fresh(json!({"heartbeat": "none yet"})));
    assert_eq!(clear(&f, &[], &[], now)["unknown"], json!(["supervision"]));
}

// r1 MAJOR 5: the supervision arm reads what beat::view really returns.
#[test]
fn clear_reads_a_real_heartbeat_view() {
    let dir = tempfile_dir();
    crate::beat::write(&dir, 1_000_000, &[("send".into(), 1_000_000), ("tick".into(), 1_000_000 - 3 * TICK_MS - 1)], 3 * TICK_MS).unwrap();
    let mut f = healthy();
    f.insert("supervision".into(), fresh(crate::beat::view(&dir, 1_000_000)));
    let c = clear(&f, &[], &[], 1_000_000);
    assert_eq!(c["state"], "NOT CLEAR", "{c}");
    crate::beat::write(&dir, 1_000_000, &[("send".into(), 1_000_000), ("tick".into(), 1_000_000)], 3 * TICK_MS).unwrap();
    let mut f = healthy();
    f.insert("supervision".into(), fresh(crate::beat::view(&dir, 1_000_000)));
    assert_eq!(clear(&f, &[], &[], 1_000_000)["state"], "CLEAR");
    let _ = std::fs::remove_dir_all(&dir);
}

fn tempfile_dir() -> std::path::PathBuf {
    // a marked test estate, so `Store::open` accepts it; leaked on purpose (the dir outlives the test)
    let e = Box::leak(Box::new(crate::testkit::Estate::create_without_hub().unwrap()));
    let d = e.store_root().join(format!("t3708-{}", crate::store::now_ms()));
    std::fs::create_dir_all(&d).unwrap();
    d
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
        ("something new", "bad"),
    ] {
        let mut f = healthy();
        f.insert("revocation".into(), fresh(json!({"revocation": line, "refused_revoked": 0})));
        let c = clear(&f, &[], &[], 10_000_000);
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

// E-127's reader (carried from T-3703 r1 finding 17): both invariants fire, and only when broken.
#[test]
fn e127_reader_fires_on_each_broken_invariant_only() {
    use crate::revocation::{PN13, e127_broken};
    let pn13 = PN13.as_millis() as u64;
    let key = "ed25519:abcdef0123456789aaaa".to_string();
    let held: BTreeMap<String, u64> = [(key.clone(), 5_000)].into_iter().collect();
    let now = 10 * pn13;
    let line = format!("unverified since {} (hub unreachable)", now - pn13 - 1);
    assert_eq!(e127_broken(&line, true, &held, &[], now), Vec::<String>::new());
    assert_eq!(e127_broken(&line, false, &held, &[], now).len(), 1);
    let stale = format!("stale since {} (x)", now - pn13 - 1);
    assert_eq!(e127_broken(&stale, false, &held, &[], now).len(), 1);
    let young = format!("unverified since {} (x)", now - pn13 + 1);
    assert!(e127_broken(&young, false, &held, &[], now).is_empty());
    assert!(e127_broken("verified seq 3 at 1", false, &held, &[], now).is_empty());
    let before = [("m1".to_string(), "abcdef0123456789".to_string(), 4_999)];
    assert!(e127_broken("verified seq 3 at 1", true, &held, &before, now).is_empty());
    let after = [("m2".to_string(), "abcdef0123456789".to_string(), 5_001)];
    let b = e127_broken("verified seq 3 at 1", true, &held, &after, now);
    assert_eq!(b.len(), 1);
    assert!(b[0].contains("m2"), "{b:?}");
    let other = [("m3".to_string(), "0000000000000000".to_string(), 9_999)];
    assert!(e127_broken("verified seq 3 at 1", true, &held, &other, now).is_empty());
    let mut f = healthy();
    f.insert("revocation_invariants".into(), fresh(json!({"broken": b})));
    assert!(clear(&f, &[], &[], now)["bad"].to_string().contains("revocation_invariants"));
}

// D-2 (provisional A): the not-built quota value does not hold CLEAR back.
#[test]
fn d2_quota_not_built_does_not_hold_clear() {
    let f = healthy();
    assert_eq!(f["quota"]["value"], QUOTA_NOT_BUILT);
    assert_eq!(clear(&f, &[], &[], 10_000_000)["state"], "CLEAR");
}

// E-57 (V-8): idle 48 h with nothing pending is never bad, its age is shown.
#[test]
fn e57_idle_48h_with_nothing_pending_is_not_bad() {
    let now = 200 * 3_600_000;
    let rows = vec![row("m0", "STORED", now - 48 * 3_600_000), row("m0", "HANDED_OVER", now - 48 * 3_600_000)];
    let p = pending(&rows, None);
    let v = last_surface(&rows, &p, Some(now - 48 * 3_600_000), &BTreeMap::new(), &wall_age(now));
    assert_eq!(v["bad"], false);
    assert_eq!(v["age_ms"], 48 * 3_600_000);
}

#[test]
fn e57_bad_exactly_after_ready_plus_two_ticks() {
    let ready = 1_000_000;
    let rows = vec![row("m1", "STORED", ready - 5_000)];
    let p = pending(&rows, None);
    let e = BTreeMap::new();
    assert_eq!(last_surface(&rows, &p, Some(ready), &e, &wall_age(ready + SURFACE_AFTER_MS))["bad"], false);
    let v = last_surface(&rows, &p, Some(ready), &e, &wall_age(ready + SURFACE_AFTER_MS + 1));
    assert_eq!(v["bad"], true);
    assert_eq!(v["overdue"][0]["by"], "ready");
}

#[test]
fn e57_stored_after_ready_counts_from_storing() {
    let ready = 1_000_000;
    let stored = ready + 100_000;
    let rows = vec![row("m1", "STORED", stored)];
    let p = pending(&rows, None);
    let e = BTreeMap::new();
    assert_eq!(last_surface(&rows, &p, Some(ready), &e, &wall_age(stored + SURFACE_AFTER_MS))["bad"], false);
    assert_eq!(last_surface(&rows, &p, Some(ready), &e, &wall_age(stored + SURFACE_AFTER_MS + 1))["bad"], true);
}

// V-8 hook misses persist (r1 MAJOR 7) and keep the first time.
#[test]
fn e57_bad_two_ticks_after_a_hook_call_that_missed_it_and_across_a_restart() {
    let dir = tempfile_dir();
    let store = crate::store::Store::open(&dir, "fx").unwrap();
    record_hook_call(&store, &["m1".to_string()], 1_000);
    record_hook_call(&store, &["m1".to_string()], 9_000); // a later call keeps the first time
    // "restart": the store is closed and opened again; the record is still there
    drop(store);
    let store = crate::store::Store::open(&dir, "fx").unwrap();
    let missed = hook_missed(&store);
    assert_eq!(missed.get("m1"), Some(&1_000));
    let rows = vec![row("m1", "STORED", 0)];
    let p = pending(&rows, None);
    assert_eq!(last_surface(&rows, &p, None, &missed, &wall_age(1_000 + SURFACE_AFTER_MS))["bad"], false);
    let v = last_surface(&rows, &p, None, &missed, &wall_age(1_000 + SURFACE_AFTER_MS + 1));
    assert_eq!(v["bad"], true);
    assert_eq!(v["overdue"][0]["by"], "hook call");
    // handed over: no longer pending, the record is dropped at the next call
    record_hook_call(&store, &[], 10_000);
    assert!(hook_missed(&store).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn e57_no_ready_no_hook_call_is_not_bad_however_old() {
    let rows = vec![row("m1", "STORED", 0)];
    let p = pending(&rows, None);
    assert_eq!(last_surface(&rows, &p, None, &BTreeMap::new(), &wall_age(100 * 3_600_000))["bad"], false);
}

#[test]
fn only_a_handed_over_line_is_surfacing() {
    let rows = vec![row("m1", "STORED", 0), row("m1", "CLAIMED", 10), row("m1", "HANDOVER_UNCONFIRMED", 20)];
    let p = pending(&rows, None);
    let v = last_surface(&rows, &p, Some(0), &BTreeMap::new(), &wall_age(1_000_000));
    assert!(v["last_surface_ms"].is_null());
    assert_eq!(v["bad"], true);
}
