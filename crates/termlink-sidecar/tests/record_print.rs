//! Hermetic tests for reply, the record print and the vocabulary of printed labels.
//! B11 tests: reply (BV-6), the record print (BV-7) and the FC-16 vocabulary.
//! Hermetic: testkit estate only.

use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::api::{self, PRE_HANDOVER_VOCAB, Scope};
use termlink_sidecar::deadline::Deadlines;
use termlink_sidecar::deliver::{Deliverer, Pulled, Verdict};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::{LineExtra, Store};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

struct Side {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
    d: Deliverer,
}

fn roster_of(ids: &[&Identity]) -> Roster {
    let mut r = Roster::new();
    for i in ids {
        r.add(i.verifying_key());
    }
    r
}

fn side(e: &Estate, hub_id: &str, who: &str, name: &str, inst: &str, roster: &Roster) -> Side {
    let store = Arc::new(Store::open(&e.store_root(), name).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: name.into(),
                instance: inst.into(),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: hub_id.into(),
                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
                timeout: Duration::from_secs(15),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: name.into(),
            instance: inst.into(),
            hub: e.hub_addr(),
            declared_hub_id: hub_id.into(),
            roster: roster.clone(),
            timeout: Duration::from_secs(15),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster.clone())),
    )
    .unwrap();
    let d = Deliverer::new(name, store.clone(), Box::new(BuiltIn::new(roster.clone())), Some(sender.clone()));
    Side { store, sender, rx, d }
}

fn status(s: &Side, name: &str, id: &str) -> Value {
    api::dispatch_with(&json!({"op": "status", "msg_id": id}), Scope::Status, &s.store, name, None, Some(&s.sender))
}

fn call(s: &Side, name: &str, req: Value) -> Value {
    api::dispatch_full(&req, Scope::Send, &s.store, name, None, Some(&s.sender), Some(&s.d))
}

/// FC-16: a sender-side view of a message with no HANDED_OVER says nothing about delivery.
fn assert_pre_handover_vocab(view: &Value) {
    let stage = view["stage"].as_str().unwrap();
    assert!(PRE_HANDOVER_VOCAB.contains(&stage), "stage {stage:?} not in the allow-list: {view}");
    if let Some(l) = view.get("label") {
        let l = l.as_str().unwrap();
        assert!(PRE_HANDOVER_VOCAB.contains(&l), "label {l:?} not in the allow-list: {view}");
    }
    assert!(view.get("delivered").is_none(), "{view}");
    let summary = json!({"stage": view["stage"], "label": view.get("label")}).to_string().to_lowercase();
    for w in ["deliver", "handed", "read", "seen", "done"] {
        assert!(!summary.contains(w), "{w:?} in {summary}");
    }
}

#[test]
fn record_print_round_trip_shows_all_five_stages_in_order_with_times() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let (ka, kb) = (e.identity("a").unwrap(), e.identity("b").unwrap());
    let roster = roster_of(&[&ka, &kb]);
    let a = side(&e, &id, "a", "agent-a", "inst-a", &roster);
    let b = side(&e, &id, "b", "agent-b", "inst-b", &roster);

    let sent = call(&a, "agent-a", json!({"op": "send", "to": "agent-b", "to_instance": "inst-b", "conv": "rt", "body": "question"}));
    assert_eq!(sent["stage"], "SENT", "{sent}");
    let m1 = sent["msg_id"].as_str().unwrap().to_owned();
    assert_pre_handover_vocab(&json!({"stage": sent["stage"]}));

    b.rx.pass();
    a.rx.pass();
    assert_pre_handover_vocab(&status(&a, "agent-a", &m1)["messages"][0]);

    // B's agent gets it at a tool call; the transcript proves it.
    let Pulled::Frame { frame, .. } = b.d.pull("sess", "post-tool-use", Some("toolu_1")).unwrap() else { panic!("no frame") };
    let t = e.root().join("b.jsonl");
    let rec = json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [frame.text], "toolUseID": "toolu_1"}});
    std::fs::write(&t, format!("{rec}\n")).unwrap();
    b.d.register_session("sess", &t).unwrap();
    assert_eq!(b.d.finalize(&frame.nonce, false).unwrap(), Verdict::HandedOver);

    // B answers it.
    let r = call(&b, "agent-b", json!({"op": "reply", "conv": "rt", "in_reply_to": m1, "body": "answer"}));
    assert_eq!((r["stage"].clone(), r["in_reply_to"].clone()), (json!("SENT"), json!(m1)), "{r}");
    let m2 = r["msg_id"].as_str().unwrap().to_owned();

    // A reads its inbox: B's stage records and B's reply.
    a.rx.pass();
    let v = status(&a, "agent-a", &m1)["messages"][0].clone();
    assert_eq!(v["stage"], "REPLIED", "{v}");
    let record = v["record"].as_array().unwrap();
    let stages: Vec<&str> = record.iter().map(|r| r["stage"].as_str().unwrap()).collect();
    assert_eq!(stages, ["SENT", "RECEIVED", "STORED", "HANDED_OVER", "REPLIED"]);
    let times: Vec<u64> = record.iter().map(|r| r["at_ms"].as_u64().unwrap_or_else(|| panic!("a row is absent: {v}"))).collect();
    assert!(times.windows(2).all(|w| w[0] <= w[1]), "times out of order: {times:?}");
    assert_eq!(record[0]["by"], ka.fingerprint());
    for r in &record[1..] {
        assert_eq!(r["by"], kb.fingerprint(), "peer stages are B's");
    }
    // The reply reached A's own instance as an ordinary stored message.
    let reply_lines: Vec<String> = a.store.chain_lines("rt").unwrap().into_iter().filter(|l| l.msg_id == m2).map(|l| l.event).collect();
    assert_eq!(reply_lines, ["RECEIVED", "STORED"]);
    // B's own view of m1 carries REPLIED too, naming the reply.
    let bl = b.store.chain_lines("rt").unwrap();
    let rep = bl.iter().find(|l| l.event == "REPLIED" && l.msg_id == m1).unwrap();
    assert_eq!(rep.reason.as_deref(), Some(format!("reply={m2}").as_str()));
}

#[test]
fn record_print_reply_only_to_a_message_stored_here_and_not_refused() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let ka = e.identity("a").unwrap();
    let roster = roster_of(&[&ka]);
    let b = side(&e, &id, "b", "agent-b", "inst-b", &roster);
    let unknown = call(&b, "agent-b", json!({"op": "reply", "conv": "x", "in_reply_to": "nope", "body": "?"}));
    assert_eq!(unknown["code"], "bad-request", "{unknown}");
    // A stored-but-refused message (e.g. not-allowed) cannot be answered.
    b.store.append_event("y", "STORED", "m9", "fp-host", "d").unwrap();
    b.store.append_event("y", "REFUSED", "m9", "fp-host", "d").unwrap();
    let refused = call(&b, "agent-b", json!({"op": "reply", "conv": "y", "in_reply_to": "m9", "body": "?"}));
    assert_eq!(refused["code"], "bad-request", "{refused}");
    assert!(refused["error"].as_str().unwrap().contains("refused"), "{refused}");
    // Nothing was queued.
    assert_eq!(b.sender.outbox_len(), 0);
}

#[test]
fn record_print_every_pre_handover_state_uses_only_accepted_vocabulary() {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-a").unwrap();
    let (me, peer) = ("fp-a", "fp-b");
    let t0 = 1_000_000u64;
    let peer_line = |conv: &str, ev: &str, id: &str, at: u64| {
        let x = LineExtra { peer_ts_ms: Some(at), ..Default::default() };
        s.append_event_unless(conv, ev, id, peer, "d", x, |_| false).unwrap();
    };
    // waiting: hub unreachable, never posted
    s.append_event("c", "SENT", "w", me, "d").unwrap();
    s.append_event("c", "WAITING", "w", me, "d").unwrap();
    // posted, nothing back
    s.append_event("c", "SENT", "p", me, "d").unwrap();
    s.append_event("c", "POSTED", "p", me, "d").unwrap();
    // received only
    s.append_event("c", "SENT", "r", me, "d").unwrap();
    s.append_event("c", "POSTED", "r", me, "d").unwrap();
    peer_line("c", "RECEIVED", "r", t0);
    // stored
    s.append_event("c", "SENT", "s", me, "d").unwrap();
    s.append_event("c", "POSTED", "s", me, "d").unwrap();
    peer_line("c", "RECEIVED", "s", t0);
    peer_line("c", "STORED", "s", t0);
    // dead-lettered
    s.append_event("c", "SENT", "x", me, "d").unwrap();
    s.append_event("c", "DEAD_LETTER", "x", me, "d").unwrap();
    // stored just now
    let now_real = termlink_sidecar::store::now_ms();
    s.append_event("c", "SENT", "f", me, "d").unwrap();
    s.append_event("c", "POSTED", "f", me, "d").unwrap();
    peer_line("c", "RECEIVED", "f", now_real);
    peer_line("c", "STORED", "f", now_real);
    // Clock now, then past the accept deadline: `p` (posted, nothing back) turns
    // UNKNOWN. Stored messages wait for the recipient however old they are: only
    // the recipient's overdue notice makes them STUCK (R-44.e, PH-2 row P2).
    for (now, label_p, label_f) in [
        (now_real, None, "WAITING FOR RECIPIENT"),
        (now_real + Deadlines::R44E.accept(false) + 60_000, Some("UNKNOWN"), "WAITING FOR RECIPIENT"),
    ] {
        let view = api::status_view(&json!({"op": "status"}), &s, "agent-a", None, now, &Deadlines::R44E);
        let msgs = view["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 6);
        for m in msgs {
            assert_pre_handover_vocab(m);
        }
        let get = |id: &str| msgs.iter().find(|m| m["msg_id"] == id).unwrap().clone();
        assert_eq!(get("w")["label"], "WAITING");
        assert_eq!(get("x")["label"], "REJECTED BY HUB");
        assert_eq!(get("p").get("label").and_then(Value::as_str), label_p);
        assert_eq!(get("s")["label"], "WAITING FOR RECIPIENT", "stored at t0, long ago, no overdue notice");
        assert_eq!(get("f")["label"], label_f);
        assert_eq!(get("r")["stage"], "RECEIVED");
    }
}
