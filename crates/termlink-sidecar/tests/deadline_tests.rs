//! Tests of the 30 second tick and the receiver overdue deadlines on a simulated clock.
//! PH-2 row P2 (T-3428): the 30 s tick and the R-44.e deadlines, on a simulated
//! clock. Sender labels from the hub record; the receiver's overdue notices.
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::api;
use termlink_sidecar::deadline::{Deadlines, OVERDUE_ANSWER, OVERDUE_HANDOVER, TICK_MS};
use termlink_sidecar::deliver::Deliverer;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::{LineExtra, Store};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::{Msg, Roster};

const D: Deadlines = Deadlines::R44E;

#[test]
fn deadline_table_is_r44e() {
    assert_eq!(TICK_MS, 30_000, "R-17: one tick is 30 s");
    assert_eq!((D.accept(false), D.accept(true)), (60_000, 15_000));
    assert_eq!((D.handover(false), D.handover(true)), (2 * TICK_MS, TICK_MS));
    assert_eq!((D.answer(false), D.answer(true)), (3_600_000, 300_000));
}

// ---- sender labels -------------------------------------------------------------

/// A sender store with one message `m` at the stages given. Every deadline is
/// measured from the sender's own POSTED line (`status_at` reads it from the
/// store), so the clock the tests reason about is that line's recorded time, not
/// the wall clock taken before setup. A peer stage is given as an offset from
/// that same instant. The view returned is taken `offset` ms after it.
///
/// Why not `now_ms()` taken before setup: the store stamps each line with the
/// wall clock when it is written. On a loaded host the setup alone can take more
/// than a second, so a "61 s after the post" view was really 60 s and the
/// 60 s accept deadline had not yet passed. Measuring from the stamped line
/// keeps the asserted property identical and makes the verdict independent of
/// how long setup took.
fn sender_view(priority: i64, peer_stages: &[(&str, u64)]) -> impl Fn(u64) -> Value + use<> {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-a").unwrap();
    let (me, peer) = ("fp-a", "fp-b");
    let sent = LineExtra { reason: Some(format!("priority={priority}")), ..Default::default() };
    s.append_event_unless("c", "SENT", "m", me, "d", sent, |_| false).unwrap();
    s.append_event("c", "POSTED", "m", me, "d").unwrap();
    let posted = s.chain_lines("c").unwrap().into_iter().find(|l| l.event == "POSTED").and_then(|l| l.ts_ms).expect("POSTED is timestamped");
    for (st, off) in peer_stages {
        let extra = LineExtra { peer_ts_ms: Some(posted + off), ..Default::default() };
        s.append_event_unless("c", st, "m", peer, "d", extra, |_| false).unwrap();
    }
    move |offset: u64| {
        let _keep = &e;
        api::status_view(&json!({"op": "status", "msg_id": "m"}), &s, "agent-a", None, posted + offset, &D)["messages"][0].clone()
    }
}

fn label(v: &Value) -> (Option<&str>, Option<&str>) {
    (v.get("label").and_then(Value::as_str), v.get("stuck_step").and_then(Value::as_str))
}

#[test]
fn posted_with_nothing_back_is_unknown_after_the_accept_deadline_urgent_sooner() {
    let normal = sender_view(0, &[]);
    assert_eq!(label(&normal(20_000)), (None, None), "within 60 s: no label");
    assert_eq!(label(&normal(61_000)), (Some("UNKNOWN"), None), "liveness unknown: never STUCK");
    let urgent = sender_view(7, &[]);
    assert_eq!(label(&urgent(20_000)), (Some("UNKNOWN"), None), "urgent: 15 s");
}

#[test]
fn received_never_stored_is_stuck_not_accepted() {
    let v = sender_view(0, &[("RECEIVED", 0)]);
    assert_eq!(label(&v(30_000)), (None, None));
    assert_eq!(label(&v(61_000)), (Some("STUCK"), Some("not accepted")));
    let u = sender_view(9, &[("RECEIVED", 0)]);
    assert_eq!(label(&u(16_000)), (Some("STUCK"), Some("not accepted")));
}

#[test]
fn stored_waits_for_the_recipient_until_its_overdue_notice() {
    let v = sender_view(0, &[("RECEIVED", 0), ("STORED", 0)]);
    // A busy or absent recipient: no age makes it STUCK on the sender's side.
    assert_eq!(label(&v(24 * 3_600_000)), (Some("WAITING FOR RECIPIENT"), None));
    let n = sender_view(0, &[("RECEIVED", 0), ("STORED", 0), (OVERDUE_HANDOVER, 90_000)]);
    assert_eq!(label(&n(100_000)), (Some("STUCK"), Some("not handed over")));
}

#[test]
fn handed_over_unanswered_is_unknown_past_the_deadline_and_stuck_on_notice() {
    let base = [("RECEIVED", 0), ("STORED", 0), ("HANDED_OVER", 0)];
    let v = sender_view(0, &base);
    assert_eq!(label(&v(30 * 60_000)), (None, None), "within the hour");
    assert_eq!(label(&v(61 * 60_000)), (Some("UNKNOWN"), None));
    let u = sender_view(5, &base);
    assert_eq!(label(&u(6 * 60_000)), (Some("UNKNOWN"), None), "urgent: 5 min");
    let mut with_notice = base.to_vec();
    with_notice.push((OVERDUE_ANSWER, 61 * 60_000));
    let n = sender_view(0, &with_notice);
    assert_eq!(label(&n(62 * 60_000)), (Some("STUCK"), Some("not answered")));
    // Answered: nothing to say.
    let mut replied = base.to_vec();
    replied.push(("REPLIED", 60_000));
    let r = sender_view(0, &replied);
    assert_eq!(label(&r(2 * 3_600_000)), (None, None));
}

// ---- receiver: overdue notices -------------------------------------------------

const B: &str = "agent-b";

fn msg(from: &Identity, id: &str, priority: &str) -> Msg {
    Msg {
        from: from.fingerprint().to_owned(),
        from_agent: "agent-a".into(),
        from_instance: "inst-a".into(),
        to_agent: B.into(),
        to_instance: "inst-b".into(),
        conv: "c".into(),
        msg_id: id.into(),
        seq: 0,
        priority: priority.into(),
        ts_ms: 1,
        body: "b".into(),
    }
}

fn own_events(s: &Store, id: &str) -> Vec<String> {
    s.chain_lines("c").unwrap().into_iter().filter(|l| l.msg_id == id && l.peer_ts_ms.is_none()).map(|l| l.event).collect()
}

struct Rx {
    _e: Estate,
    store: Arc<Store>,
    d: Deliverer,
    ka: Identity,
}

fn receiver_rig() -> Rx {
    let e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();
    let ka = e.identity("a").unwrap();
    let mut roster = Roster::new();
    roster.add(ka.verifying_key());
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: B.into(),
                instance: "inst-b".into(),
                identity: e.identity("b").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: hub_id,
                queue_path: e.root().join("outbox-b.sqlite"),
                timeout: Duration::from_secs(15),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let d = Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(roster)), Some(sender));
    Rx { _e: e, store, d, ka }
}

fn put(s: &Store, m: &Msg) {
    s.append_event(&m.conv, "RECEIVED", &m.msg_id, &m.from, &m.digest()).unwrap();
    s.put(&m.from, &m.conv, &m.msg_id, &m.payload_bytes()).unwrap();
}

#[test]
fn receiver_notices_an_overdue_handover_only_while_its_agent_is_ready_once() {
    let r = receiver_rig();
    put(&r.store, &msg(&r.ka, "h1", "0"));
    let now = termlink_sidecar::store::now_ms();
    // Not ready (no session flag): never overdue, however late.
    assert_eq!(r.d.check_overdue(now + 3_600_000, &D), 0);
    // Ready now; within 2 ticks: nothing yet.
    r.d.mark_ready("s1", now).unwrap();
    assert_eq!(r.d.check_overdue(now + 2 * TICK_MS - 1_000, &D), 0);
    // Past 2 ticks after becoming ready: one notice, recorded once.
    assert_eq!(r.d.check_overdue(now + 2 * TICK_MS + 1_000, &D), 1);
    assert_eq!(r.d.check_overdue(now + 3 * TICK_MS, &D), 0, "sent once");
    assert!(own_events(&r.store, "h1").contains(&OVERDUE_HANDOVER.to_string()));
}

#[test]
fn receiver_session_end_clears_readiness_so_nothing_is_overdue() {
    let r = receiver_rig();
    put(&r.store, &msg(&r.ka, "h2", "0"));
    let now = termlink_sidecar::store::now_ms();
    r.d.mark_ready("s1", now).unwrap();
    r.d.mark_unready("s1").unwrap();
    assert_eq!(r.d.ready_since(now), None);
    assert_eq!(r.d.check_overdue(now + 3_600_000, &D), 0);
}

#[test]
fn receiver_urgent_handover_is_overdue_after_one_tick() {
    let r = receiver_rig();
    put(&r.store, &msg(&r.ka, "u1", "7"));
    let now = termlink_sidecar::store::now_ms();
    r.d.mark_ready("s1", now).unwrap();
    assert_eq!(r.d.check_overdue(now + TICK_MS + 1_000, &D), 1);
    // R-37 (PH-2 row P5): an urgent message overdue is an alarm at once.
    let att = r.store.attention_entries().join("\n");
    assert!(att.contains(termlink_sidecar::deadline::URGENT_OVERDUE) && att.contains("u1"), "{att}");
}

#[test]
fn receiver_routine_overdue_is_no_alarm() {
    let r = receiver_rig();
    put(&r.store, &msg(&r.ka, "r1", "0"));
    let now = termlink_sidecar::store::now_ms();
    r.d.mark_ready("s1", now).unwrap();
    assert_eq!(r.d.check_overdue(now + 3 * TICK_MS, &D), 1, "the notice still goes to the sender");
    let att = r.store.attention_entries().join("\n");
    assert!(!att.contains(termlink_sidecar::deadline::URGENT_OVERDUE), "routine accumulates, never alarms: {att}");
}

#[test]
fn receiver_notices_an_unanswered_handover_and_not_an_answered_one() {
    let r = receiver_rig();
    let (a, b) = (msg(&r.ka, "a1", "0"), msg(&r.ka, "a2", "0"));
    put(&r.store, &a);
    put(&r.store, &b);
    for m in [&a, &b] {
        r.store.append_event("c", "HANDED_OVER", &m.msg_id, &m.from, &m.digest()).unwrap();
    }
    r.store.append_event("c", "REPLIED", "a2", &b.from, &b.digest()).unwrap();
    let now = termlink_sidecar::store::now_ms();
    r.d.mark_ready("s1", now + 3_600_000).unwrap();
    assert_eq!(r.d.check_overdue(now + 3_600_000 + 1_000, &D), 1, "only the unanswered one");
    assert!(own_events(&r.store, "a1").contains(&OVERDUE_ANSWER.to_string()));
    assert!(!own_events(&r.store, "a2").contains(&OVERDUE_ANSWER.to_string()));
}
