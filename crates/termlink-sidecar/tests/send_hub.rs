//! B6 tests for C-5 (send.rs) against a throwaway hub. Hermetic: testkit estate only.

use base64::Engine as _;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};
use termlink_sidecar::api::*;
use termlink_sidecar::send::*;
use termlink_sidecar::store::{ChainLine, Store, now_ms};
use termlink_sidecar::testkit::Estate;

struct Rig {
    e: Estate,
    store: Arc<Store>,
    sender: Arc<Sender>,
}

fn sender_cfg(e: &Estate, hub_id: &str, hub: termlink_protocol::TransportAddr) -> SendConfig {
    SendConfig {
        agent: "agent-a".into(),
        instance: "inst-a".into(),
        identity: e.identity("a").unwrap(),
        hub,
        declared_hub_id: hub_id.into(),
        queue_path: e.root().join("outbox-a.sqlite"),
        timeout: Duration::from_secs(3),
        pump_interval: None,
    }
}

fn rig() -> Rig {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let sender = Arc::new(Sender::new(sender_cfg(&e, &id, e.hub_addr()), store.clone()).unwrap());
    Rig { e, store, sender }
}

fn events(r: &Rig, conv: &str, msg_id: &str) -> Vec<ChainLine> {
    r.store.chain_lines(conv).unwrap().into_iter().filter(|l| l.msg_id == msg_id).collect()
}

fn names(ls: &[ChainLine]) -> Vec<&str> {
    ls.iter().map(|l| l.event.as_str()).collect()
}

/// Everything on the recipient's inbox topic, straight from the hub.
fn inbox(r: &Rig, to: &str) -> Vec<Value> {
    let topic = inbox_topic(&r.e.hub_id().unwrap(), to);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(
            &r.e.hub_addr(),
            "channel.subscribe",
            json!({"topic": topic, "cursor": 0, "limit": 100}),
            Duration::from_secs(5),
        ))
        .unwrap();
    match termlink_session::client::unwrap_result(resp) {
        Ok(v) => v["messages"].as_array().cloned().unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn wait_for<F: FnMut() -> bool>(mut f: F) -> bool {
    let end = Instant::now() + Duration::from_secs(10);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn send_hub_posts_to_the_recipient_inbox_and_records_sent_then_posted_with_times() {
    let r = rig();
    let before = now_ms();
    let s = r.sender.send("agent-b", "inst-b", Some("conv-1"), "hello", None).unwrap();
    let after = now_ms();
    assert!((before..=after).contains(&s.ts_ms));
    assert_eq!(r.sender.outbox_len(), 0, "the inline pump drained the outbox");
    let ev = events(&r, "conv-1", &s.msg_id);
    assert_eq!(names(&ev), ["SENT", "POSTED"]);
    assert!(ev.iter().all(|l| l.ts_ms.is_some_and(|t| t >= before)), "every stage record carries its time");
    // The hub holds exactly that signed payload (the hub verified the signature to accept it).
    let msgs = inbox(&r, "agent-b");
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    let payload = base64::engine::general_purpose::STANDARD.decode(msgs[0]["payload_b64"].as_str().unwrap()).unwrap();
    assert_eq!(termlink_sidecar::wire::digest_hex(&payload), ev[0].digest);
    assert_eq!(msgs[0]["sender_id"], ev[0].sender);
    assert_eq!(msgs[0]["msg_type"], "sidecar.msg.v1");
}

#[test]
fn send_hub_api_send_answers_sent_only_and_status_shows_the_rows() {
    let mut r = rig();
    let mut cfg = ApiConfig::new("agent-a", &r.e.runtime_dir());
    cfg.hub_id = Some(r.sender.declared_hub_id().to_owned());
    cfg.sender = Some(r.sender.clone());
    let api = Api::start(cfg, r.store.clone()).unwrap();
    let call = |scope: Scope, req: Value| -> Value {
        use std::io::{BufRead, BufReader, Write};
        let mut s = std::os::unix::net::UnixStream::connect(api.path(scope)).unwrap();
        writeln!(s, "{req}").unwrap();
        let mut line = String::new();
        BufReader::new(s).read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    };
    let v = call(Scope::Send, json!({"op":"send","to":"agent-b","to_instance":"inst-b","conv":"conv-api","body":"hi","priority":3}));
    assert_eq!(v["stage"], "SENT", "{v}");
    assert!(v["msg_id"].as_str().is_some_and(|m| m.len() == 32));
    assert!(!v.to_string().to_lowercase().contains("deliver"), "{v}");
    let st = call(Scope::Status, json!({"op":"status","conv":"conv-api"}));
    let rows = st["rows"].as_array().unwrap();
    assert!(rows.iter().all(|row| row["ts_ms"].is_u64()), "{st}");
    assert_eq!(st["messages"][0]["stage"], "SENT");
    assert!(st["messages"][0].get("delivered").is_none());
    assert_eq!(st["outbox"]["pending"], 0);
    // Bad input is refused, nothing queued.
    let bad = call(Scope::Send, json!({"op":"send","to":"agent-b","to_instance":"inst-b","body":"x","priority":"high"}));
    assert_eq!(bad["code"], "bad-request", "{bad}");
    let missing = call(Scope::Send, json!({"op":"send","to":"agent-b","body":"x"}));
    assert_eq!(missing["code"], "bad-request");
    assert_eq!(r.sender.outbox_len(), 0);
    drop(api);
    r.e.stop_hub();
}

#[test]
fn send_hub_hub_down_waits_then_flows_on_without_loss_or_duplicate() {
    let mut r = rig();
    r.e.stop_hub();
    let s = r.sender.send("agent-b", "inst-b", Some("conv-w"), "while down", None).unwrap();
    // SENT was returned: the message is in the durable outbox, and says WAITING.
    assert_eq!(r.sender.outbox_len(), 1);
    assert_eq!(names(&events(&r, "conv-w", &s.msg_id)), ["SENT", "WAITING"]);
    assert_eq!(r.sender.pump(), PumpResult::Waiting);
    assert_eq!(names(&events(&r, "conv-w", &s.msg_id)), ["SENT", "WAITING"], "WAITING is written once");
    // A second message while down queues behind the first.
    let s2 = r.sender.send("agent-b", "inst-b", Some("conv-w"), "also down", None).unwrap();
    assert_eq!(r.sender.outbox_len(), 2);
    assert_eq!(s2.seq, 1);
    r.e.restart_hub().unwrap();
    assert_eq!(r.sender.pump(), PumpResult::Drained);
    assert_eq!(r.sender.outbox_len(), 0);
    assert_eq!(names(&events(&r, "conv-w", &s.msg_id)), ["SENT", "WAITING", "POSTED"]);
    assert_eq!(names(&events(&r, "conv-w", &s2.msg_id)), ["SENT", "WAITING", "POSTED"]);
    // Pumping again posts nothing more.
    assert_eq!(r.sender.pump(), PumpResult::Drained);
    let msgs = inbox(&r, "agent-b");
    assert_eq!(msgs.len(), 2, "{msgs:?}");
}

#[test]
fn send_hub_background_pump_flows_on_by_itself() {
    let mut r = rig();
    let id = r.sender.declared_hub_id().to_owned();
    r.e.stop_hub();
    let mut cfg = sender_cfg(&r.e, &id, r.e.hub_addr());
    cfg.queue_path = r.e.root().join("outbox-bg.sqlite");
    cfg.pump_interval = Some(Duration::from_millis(200));
    let sender = Sender::new(cfg, r.store.clone()).unwrap();
    let s = sender.send("agent-b", "inst-b", Some("conv-bg"), "later", None).unwrap();
    assert_eq!(sender.outbox_len(), 1);
    r.e.restart_hub().unwrap();
    assert!(wait_for(|| sender.outbox_len() == 0), "background pump did not drain the outbox");
    assert!(names(&events(&r, "conv-w-none", "x")).is_empty());
    assert!(names(&events(&r, "conv-bg", &s.msg_id)).contains(&"POSTED"));
}

#[test]
fn send_hub_wrong_hub_id_is_refused_and_nothing_is_queued_or_posted() {
    let r = rig();
    let wrong = Sender::new(
        {
            let mut c = sender_cfg(&r.e, "0000000000000000", r.e.hub_addr());
            c.queue_path = r.e.root().join("outbox-wrong.sqlite");
            c
        },
        r.store.clone(),
    )
    .unwrap();
    let err = wrong.send("agent-b", "inst-b", Some("conv-x"), "nope", None).unwrap_err();
    assert_eq!(err.code(), "hub-id-mismatch", "{err}");
    assert!(err.to_string().contains("0000000000000000"));
    assert_eq!(wrong.outbox_len(), 0);
    assert!(r.store.chain_lines("conv-x").unwrap().is_empty(), "no SENT without an outbox row");
    assert!(inbox(&r, "agent-b").is_empty());
    assert!(matches!(wrong.check_pin(), Err(PinError::Mismatch { .. })));
}

#[test]
fn send_hub_a_message_queued_before_the_hub_changed_is_held_not_posted() {
    // Two hubs with different ids: the sender is pointed at the second one.
    let r = rig();
    let other = Estate::create().unwrap();
    assert_ne!(r.e.hub_id().unwrap(), other.hub_id().unwrap());
    let id = r.e.hub_id().unwrap();
    // Queue while the declared hub is reachable-looking but down: use a dead address first.
    let dead = termlink_protocol::TransportAddr::unix(r.e.root().join("nothing.sock"));
    let mut cfg = sender_cfg(&r.e, &id, dead);
    cfg.queue_path = r.e.root().join("outbox-held.sqlite");
    let first = Sender::new(cfg, r.store.clone()).unwrap();
    let s = first.send("agent-b", "inst-b", Some("conv-h"), "held", None).unwrap();
    assert_eq!(first.outbox_len(), 1);
    drop(first);
    // Reopen the same outbox, now pointing at the OTHER hub with the same declared id.
    let mut cfg = sender_cfg(&r.e, &id, other.hub_addr());
    cfg.queue_path = r.e.root().join("outbox-held.sqlite");
    let second = Sender::new(cfg, r.store.clone()).unwrap();
    let res = second.pump();
    assert!(matches!(res, PumpResult::Held(_)), "{res:?}");
    assert_eq!(second.outbox_len(), 1, "the message stays in the outbox");
    assert!(second.held().is_some());
    assert!(r.store.attention_entries().iter().any(|a| a.contains("hub-id-mismatch")));
    assert_eq!(names(&events(&r, "conv-h", &s.msg_id)), ["SENT", "WAITING"]);
    // Nothing reached the other hub.
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(
            &other.hub_addr(),
            "channel.subscribe",
            json!({"topic": inbox_topic(&id, "agent-b"), "cursor": 0, "limit": 10}),
            Duration::from_secs(5),
        ))
        .unwrap();
    let got = termlink_session::client::unwrap_result(resp).map(|v| v["messages"].as_array().map(Vec::len).unwrap_or(0));
    assert!(matches!(got, Err(_) | Ok(0)), "{got:?}");
}

#[test]
fn send_hub_touches_no_production_path() {
    let r = rig();
    r.sender.send("agent-b", "inst-b", Some("conv-p"), "x", None).unwrap();
    assert!(r.e.path_open_audit().is_empty());
}
