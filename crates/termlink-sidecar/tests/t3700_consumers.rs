//! T-3700 (arc-011 P-12, B-2, 06a AL-30) at L4 on a real hub: E-66 (a second consumer is flagged, refused
//! after bind; 17 readers evict the oldest with a row; a second copy is seen within one check), E-66.6 (a
//! read-only consumer counts), E-133 (the per-tick check fires), E-169 B-2 part (an unreadable list is
//! `unknown`, never "one"), operator rulings E (proven/claimed, bind by key) and D (peek through the sidecar).

use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use serde_json::{Value, json};
use termlink_session::agent_identity::Identity;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

const INBOX_BOUND: i64 = -32049;

fn rpc(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    serde_json::to_value(&resp).unwrap()
}

struct Side {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
}

fn side(e: &Estate, id: &str, who: &str, instance: &str) -> Side {
    let agent = format!("fx-{who}");
    let root = e.store_root().join(instance);
    std::fs::create_dir_all(&root).unwrap();
    let store = Arc::new(Store::open(&root, &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: instance.to_string(),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.to_string(),
                queue_path: e.root().join(format!("outbox-{instance}.sqlite")),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let roster = Roster::new();
    let rx = Receiver::new(
        ReceiveConfig {
            agent: agent.clone(),
            instance: instance.to_string(),
            hub: e.hub_addr(),
            declared_hub_id: id.to_string(),
            roster: roster.clone(),
            timeout: Duration::from_secs(10),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster)),
    )
    .unwrap();
    Side { store, sender, rx }
}

fn put_card(e: &Estate, agent: &str, who: &str) {
    let r = rpc(e, "hub.key_card_put", json!({"agent": agent, "project": "proj-p",
        "signing_key": format!("ed25519-pub:{}", e.identity(who).unwrap().public_key_hex())}));
    assert_eq!(r["result"]["ok"], true, "{r}");
}

fn line(store: &Store) -> String {
    termlink_sidecar::consumers::status(store)["consumers"].as_str().unwrap_or("").to_string()
}

fn attention_count(store: &Store) -> usize {
    store.attention_entries().iter().filter(|l| l.contains(termlink_sidecar::consumers::ATTENTION_KIND)).count()
}

/// A raw read of `topic` by a client that is not a sidecar, with an optional `reader` object.
fn raw_read(e: &Estate, topic: &str, reader: Option<Value>) -> Value {
    let mut p = json!({"topic": topic, "cursor": 0, "limit": 5});
    if let Some(r) = reader {
        p["reader"] = r;
    }
    rpc(e, "channel.subscribe", p)
}

/// A reader object signed by `key` for `agent`/`instance` at cursor 0.
fn signed_reader(id: &str, topic: &str, key: &Identity, agent: &str, instance: &str) -> Value {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let b = termlink_session::inbox_read::read_sign_bytes(id, topic, "channel.subscribe", 0, ts, agent, instance);
    json!({"agent": agent, "instance": instance, "ts_ms": ts, "signer_pk": key.public_key_hex(), "sig": key.sign_hex(&b)})
}

fn consumers(e: &Estate, topic: &str) -> Vec<Value> {
    let r = rpc(e, "inbox.consumers", json!({"topic": topic}));
    r["result"]["consumers"].as_array().cloned().unwrap_or_else(|| panic!("inbox.consumers: {r}"))
}

fn admin_rows(e: &Estate, event: &str) -> usize {
    let p = e.runtime_dir().join("rpc-admin.jsonl");
    std::fs::read_to_string(p).unwrap_or_default().lines().filter(|l| l.contains(&format!("\"{event}\""))).count()
}

#[test]
fn e66_second_consumer_flagged_then_refused_after_bind() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    put_card(&e, "fx-a", "a");
    let a = side(&e, &id, "a", "inst-a");

    // fx-a's own signed read: proven, one consumer
    let _ = a.rx.pass();
    a.rx.check_consumers_now();
    assert_eq!(line(&a.store), "one (proven)");

    // E-66.6: fx-z only reads (never acks), unsigned; it is a consumer and is flagged at the next check
    let r = raw_read(&e, &topic, Some(json!({"agent": "fx-z", "instance": "raw"})));
    assert!(r.get("result").is_some(), "unbound read passes: {r}");
    a.rx.check_consumers_now();
    let l = line(&a.store);
    assert_eq!(l, "inbox consumed by two identities: fx-a (proven), fx-z (claimed: unsigned)");
    assert_eq!(attention_count(&a.store), 1);
    a.rx.check_consumers_now();
    assert_eq!(attention_count(&a.store), 1, "coalesced per (inbox, reader, day)");
    println!("E-133 PASS (the per-tick check fired on the injected second consumer)");

    // the operator binds fx-a's inbox to fx-a (runme, local socket)
    let b = rpc(&e, "hub.inbox_bind", json!({"topic": topic, "agent": "fx-a"}));
    assert_eq!(b["result"]["ok"], true, "{b}");
    assert_eq!(admin_rows(&e, "inbox-bind"), 1);

    // fx-z unsigned, fx-z naming fx-a unsigned, an unnamed peek: all refused with a reason
    for reader in [Some(json!({"agent": "fx-z"})), Some(json!({"agent": "fx-a", "instance": "x"})), None] {
        let r = raw_read(&e, &topic, reader.clone());
        assert_eq!(r["error"]["code"], INBOX_BOUND, "{reader:?}: {r}");
        assert!(r["error"]["message"].as_str().unwrap().contains("inbox bound to fx-a"), "{r}");
    }
    assert!(admin_rows(&e, "inbox-bound-refused") >= 1);
    // fx-z signing with its own key while naming fx-a: refused as a card mismatch
    let z = e.identity("z").unwrap();
    let r = raw_read(&e, &topic, Some(signed_reader(&id, &topic, &z, "fx-a", "x")));
    assert_eq!(r["error"]["code"], -32044, "{r}");

    // fx-a keeps reading its own inbox, and its peek works through the bind (ruling D)
    let _ = a.sender.send("fx-a", "inst-a", Some("c1"), "to myself", None);
    assert!(matches!(a.rx.pass(), termlink_sidecar::pull::PassResult::CaughtUp { .. }));
    let p = a.sender.peek_inbox(0, 10).unwrap();
    assert!(p["result"]["messages"].as_array().is_some(), "{p}");

    // other inboxes are untouched by the bind
    let other = format!("inbox:{id}/fx-b");
    let _ = rpc(&e, "channel.create", json!({"name": other}));
    assert!(raw_read(&e, &other, None).get("result").is_some());

    // unbind reopens
    let u = rpc(&e, "hub.inbox_unbind", json!({"topic": topic}));
    assert_eq!(u["result"]["was_bound"], true, "{u}");
    assert!(raw_read(&e, &topic, None).get("result").is_some());
    println!("E-66 PASS (flagged naming both, read-only consumer counted, refused after bind with reason and row)");
}

#[test]
fn e66_ack_is_a_consumer_and_17_readers_evict_the_oldest() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    put_card(&e, "fx-b", "b");

    // fx-b acks on fx-a's inbox (a signed receipt post): recorded as proven fx-b
    let kb = e.identity("b").unwrap();
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let payload = br#"{"up_to":0}"#;
    let sig = kb.sign_hex(&termlink_protocol::control::channel::canonical_sign_bytes(&topic, "receipt", payload, None, ts));
    let r = rpc(&e, "channel.post", json!({"topic": topic, "msg_type": "receipt",
        "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload), "ts": ts,
        "sender_id": kb.fingerprint(), "sender_pubkey_hex": kb.public_key_hex(), "signature_hex": sig}));
    assert!(r.get("result").is_some(), "{r}");
    let c = consumers(&e, &topic);
    assert!(c.iter().any(|x| x["agent"] == "fx-b" && x["strength"] == "proven" && x["actions"][0] == "ack"), "{c:?}");

    // 17 more named readers: 16 kept, the oldest (fx-b's ack) evicted with an admin row
    for i in 0..17 {
        std::thread::sleep(Duration::from_millis(2));
        let _ = raw_read(&e, &topic, Some(json!({"agent": format!("fx-r{i}")})));
    }
    let c = consumers(&e, &topic);
    assert_eq!(c.len(), 16, "{c:?}");
    assert!(!c.iter().any(|x| x["agent"] == "fx-b"), "the oldest is gone");
    assert!(admin_rows(&e, "inbox-consumer-evicted") >= 2);
    println!("E-66 PASS (ack recorded; 17 readers keep 16, eviction logged)");
}

#[test]
fn v12_second_copy_seen_within_one_check() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-a")}));
    put_card(&e, "fx-a", "a");
    let a1 = side(&e, &id, "a", "inst-a1");
    let a2 = side(&e, &id, "a", "inst-a2");
    let _ = a1.rx.pass();
    let _ = a2.rx.pass();
    a1.rx.check_consumers_now();
    let l = line(&a1.store);
    assert!(l.starts_with("inbox consumed by two identities: fx-a (proven), fx-a (proven) [second copy, instance inst-a2]"), "{l}");
    println!("E-66 PASS (V-12 second copy seen within one check)");
}

#[test]
fn e169_unreadable_consumer_list_is_unknown_never_one() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-a")}));
    put_card(&e, "fx-a", "a");
    // the table cannot be written: its path is a directory
    std::fs::create_dir_all(e.runtime_dir().join("inbox-consumers.json")).unwrap();
    let a = side(&e, &id, "a", "inst-a");
    let _ = a.rx.pass();
    a.rx.check_consumers_now();
    let l = line(&a.store);
    assert!(l.starts_with("unknown ("), "{l}");
    println!("E-169 PASS (unreadable consumer list shown unknown, never one)");
}

#[test]
fn liveness_first_at_is_not_a_consumer() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    put_card(&e, "fx-a", "a");
    let a = side(&e, &id, "a", "inst-a");
    let _ = a.sender.send("fx-a", "inst-a", Some("c1"), "hello", None);
    let r = rpc(&e, "channel.first_at", json!({"topic": topic, "from_offset": 0}));
    assert!(r["result"]["ts"].as_i64().is_some(), "{r}");
    assert!(r["result"].get("messages").is_none(), "no content: {r}");
    assert!(consumers(&e, &topic).is_empty(), "first_at records no consumer");
}
