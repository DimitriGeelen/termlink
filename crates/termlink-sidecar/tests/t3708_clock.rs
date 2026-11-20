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

    // a stamp written after the jumps is aged plainly
    let late = store::now_ms() - 1_000;
    assert!((1_000..3_000).contains(&fields::age_ms(late, store::now_ms())));
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
}
