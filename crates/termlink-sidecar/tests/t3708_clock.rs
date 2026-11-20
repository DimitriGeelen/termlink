//! T-3708 (E-59, r1 MAJOR 9 / BLOCKER): the deadline ages the sidecar uses do not move with a wall jump, through the
//! real clock path (`store::test_shift_wall_ms` shifts what `store::now_ms` returns). Own binary: the shift is
//! process-wide, so nothing else may run in this process.

use termlink_sidecar::fields;
use termlink_sidecar::store;

#[test]
fn t3708_e59_deadline_ages_ignore_wall_jumps_on_the_real_clock_path() {
    fields::look();
    let stamp = store::now_ms() - 10_000; // stored 10 s ago, before any jump
    assert!((10_000..12_000).contains(&fields::age_ms(stamp, store::now_ms())));

    store::test_shift_wall_ms(3_600_000); // +1 h
    fields::look();
    let a = fields::age_ms(stamp, store::now_ms());
    assert!(a < 15_000, "a +1 h jump aged the stamp to {a} ms: a deadline would fire early");

    store::test_shift_wall_ms(-7_200_000); // then -2 h
    fields::look();
    let b = fields::age_ms(stamp, store::now_ms());
    assert!((10_000..15_000).contains(&b), "after -2 h the stamp's age is {b} ms: a deadline would fire late");

    // a stamp written (registered) after the jumps is aged plainly
    let late = fields::stamp_now();
    assert!(fields::age_ms(late, store::now_ms()) < 2_000);
    store::test_shift_wall_ms(3_600_000);
    fields::look();

    // the receiver's hand-over deadline (deliver.rs) across a +1 h jump: no notice fires early
    receiver_deadline_ignores_a_forward_jump();
}

fn receiver_deadline_ignores_a_forward_jump() {
    use std::sync::Arc;
    use std::time::Duration;
    use termlink_sidecar::deadline::Deadlines;
    use termlink_sidecar::deliver::Deliverer;
    use termlink_sidecar::policy::BuiltIn;
    use termlink_sidecar::send::{SendConfig, Sender};
    use termlink_sidecar::store::Store;
    use termlink_sidecar::testkit::Estate;
    use termlink_sidecar::wire::{Msg, Roster};
    let Ok(e) = Estate::create() else { panic!("no test hub: E-59 deliver part not run (not a PASS)") };
    let ka = e.identity("a").unwrap();
    let mut roster = Roster::new();
    roster.add(ka.verifying_key());
    let st = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: "agent-b".into(),
                instance: "inst-b".into(),
                identity: e.identity("b").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: e.hub_id().unwrap(),
                queue_path: e.root().join("outbox-b.sqlite"),
                timeout: Duration::from_secs(15),
                pump_interval: None,
            },
            st.clone(),
        )
        .unwrap(),
    );
    let d = Deliverer::new("agent-b", st.clone(), Box::new(BuiltIn::new(roster)), Some(sender));
    let m = Msg {
        from: ka.fingerprint().to_owned(),
        from_agent: "agent-a".into(),
        from_instance: "inst-a".into(),
        to_agent: "agent-b".into(),
        to_instance: "inst-b".into(),
        conv: "c".into(),
        msg_id: "h1".into(),
        seq: 0,
        priority: "0".into(),
        ts_ms: 1,
        body: "b".into(),
    };
    st.append_event(&m.conv, "RECEIVED", &m.msg_id, &m.from, &m.digest()).unwrap();
    st.put(&m.from, &m.conv, &m.msg_id, &m.payload_bytes()).unwrap();
    fields::look();
    let now = store::now_ms();
    d.mark_ready("s1", now).unwrap();
    assert_eq!(d.check_overdue(store::now_ms(), &Deadlines::R44E), 0);
    store::test_shift_wall_ms(3_600_000);
    fields::look();
    let fired = d.check_overdue(store::now_ms(), &Deadlines::R44E);
    store::test_shift_wall_ms(-3_600_000);
    fields::look();
    assert_eq!(fired, 0, "a +1 h wall jump fired the hand-over deadline early");

    // r2 MAJOR 2 (the reviewer's sequence): stored, the wall goes back 1 h, the agent becomes ready 29 minutes
    // later (on the new, earlier wall), and is checked 30 s after that. Readiness is the later event, 30 s ago:
    // nothing may fire. Ordering the two wall stamps instead picks the storing stamp and reads ~29.5 min.
    let m2 = Msg { msg_id: "h2".into(), ..m };
    st.append_event(&m2.conv, "RECEIVED", &m2.msg_id, &m2.from, &m2.digest()).unwrap();
    st.put(&m2.from, &m2.conv, &m2.msg_id, &m2.payload_bytes()).unwrap();
    store::test_shift_wall_ms(-3_600_000);
    fields::look();
    let ready = store::now_ms() + 29 * 60_000;
    d.mark_ready("s2", ready).unwrap();
    let fired2 = d.check_overdue(ready + 30_000, &Deadlines::R44E);
    store::test_shift_wall_ms(3_600_000);
    fields::look();
    assert_eq!(fired2, 0, "after a -1 h jump the hand-over deadline fired 30 s after readiness");

    // r2 MAJOR 1 (mutant m5e): a chain event written AFTER a backward jump is registered with that jump, so its
    // age is plain; an unregistered stamp would be "corrected" by the jump and read an hour old
    store::test_shift_wall_ms(-3_600_000);
    fields::look();
    st.append_event("c", "NOTE", "h3", &ka.fingerprint().to_owned(), "d").unwrap();
    let ts = st.chain_lines("c").unwrap().into_iter().rev().find(|l| l.msg_id == "h3").and_then(|l| l.ts_ms).unwrap();
    let a = fields::age_ms(ts, store::now_ms());
    store::test_shift_wall_ms(3_600_000);
    fields::look();
    assert!(a < 5_000, "a stamp written after a jump was aged as if before it: {a} ms");
}
