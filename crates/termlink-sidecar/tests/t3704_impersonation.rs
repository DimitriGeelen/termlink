//! T-3704 (arc-011 P-16, B-6; design v2 C-12): the agent another session tries to impersonate
//! learns of it. A forged arc-011 post claiming `fx-a` is refused at the hub by the key-card check;
//! fx-a's own sidecar reads the hub's per-agent refusal count on its receive tick and raises one
//! needs-attention entry, once per day, and holds no mail for it.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::{Value, json};
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

fn rpc(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    serde_json::to_value(&resp).unwrap()
}

fn kinds(store: &Store) -> Vec<(String, String)> {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| (v["kind"].as_str().unwrap_or("").to_string(), v["detail"].as_str().unwrap_or("").to_string()))
        .collect()
}

#[test]
fn a_refused_post_claiming_to_be_me_raises_one_attention_entry() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    std::fs::write(e.runtime_dir().join("key-card-check"), "enforce\n").unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "fx-a").unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: "fx-a".into(),
                instance: "inst-a".into(),
                identity: e.identity("a").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.clone(),
                queue_path: e.root().join("outbox-a.sqlite"),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: "fx-a".into(),
            instance: "inst-a".into(),
            hub: e.hub_addr(),
            declared_hub_id: id.clone(),
            roster: Roster::new(),
            timeout: Duration::from_secs(10),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender,
        Box::new(BuiltIn::new(Roster::new())),
    )
    .unwrap();

    // no forgery yet: a pass raises nothing
    let _ = rx.pass();
    assert!(!kinds(&store).iter().any(|(k, _)| k == "posts-claiming-to-be-you-refused"));

    // a forger with its own key posts as fx-a; the hub has no card for fx-a and refuses
    let topic = format!("inbox:{id}/fx-b");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    let forger = Identity::generate();
    let payload = json!({"v": 1, "kind": "msg", "from_agent": "fx-a", "body": "forged"}).to_string();
    let ts = 1_800_000_000_000i64;
    let sig = forger.sign_hex(&canonical_sign_bytes(&topic, "sidecar.msg.v1", payload.as_bytes(), None, ts));
    let refused = rpc(
        &e,
        "channel.post",
        json!({"topic": topic, "msg_type": "sidecar.msg.v1",
               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
               "ts": ts, "sender_id": forger.fingerprint(), "sender_pubkey_hex": forger.public_key_hex(),
               "signature_hex": sig}),
    );
    assert_eq!(refused["error"]["code"], -32043, "{refused}");

    // fx-a has no card yet, so the forgery counts in the shared uncarded bucket, not as fx-a:
    // give fx-a a card, then forge again with another key (a carded-agent mismatch)
    let put = rpc(&e, "hub.key_card_put", json!({"agent": "fx-a", "project": "proj-p",
        "signing_key": format!("ed25519-pub:{}", e.identity("a").unwrap().public_key_hex())}));
    assert_eq!(put["result"]["ok"], true, "{put}");
    let sig = forger.sign_hex(&canonical_sign_bytes(&topic, "sidecar.msg.v1", payload.as_bytes(), None, ts + 1));
    let refused = rpc(
        &e,
        "channel.post",
        json!({"topic": topic, "msg_type": "sidecar.msg.v1",
               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
               "ts": ts + 1, "sender_id": forger.fingerprint(), "sender_pubkey_hex": forger.public_key_hex(),
               "signature_hex": sig}),
    );
    assert_eq!(refused["error"]["code"], -32044, "{refused}");

    let _ = rx.pass();
    let hits: Vec<_> = kinds(&store).into_iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").collect();
    let list = rpc(&e, "hub.key_card_list", json!({}));
    assert_eq!(hits.len(), 1, "{hits:?} list={list} all={:?}", kinds(&store));
    assert!(hits[0].1.contains("fx-a") && hits[0].1.contains("(1 since"), "{}", hits[0].1);
    // the card write itself is the operator's to see (ruling A)
    assert!(kinds(&store).iter().any(|(k, d)| k == "key-card-written" && d.contains("fx-a") && d.contains("seq 1")));

    // the next pass with no new refusal adds nothing
    let _ = rx.pass();
    assert_eq!(kinds(&store).iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").count(), 1);
}

/// Review r2 MAJOR 6: a genuine sidecar-built message and stage record pass the hub's check in
/// enforce mode when the sender's card holds its key, and a sidecar with another key claiming that
/// agent is refused. Proves the hub reads `from_agent` where the real wire puts it.
#[test]
fn real_sidecar_msg_and_stage_pass_enforce_with_their_card_and_an_impostor_does_not() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    std::fs::write(e.runtime_dir().join("key-card-check"), "enforce\n").unwrap();
    let a_key = e.identity("a").unwrap();
    let put = rpc(&e, "hub.key_card_put", json!({"agent": "fx-a", "project": "proj-p",
        "signing_key": format!("ed25519-pub:{}", a_key.public_key_hex())}));
    assert_eq!(put["result"]["ok"], true, "{put}");
    let topic = format!("inbox:{id}/fx-b");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    let mk = |who: &str, agent: &str| {
        let store = Arc::new(Store::open(&e.store_root(), &format!("{agent}-{who}")).unwrap());
        Sender::new(
            SendConfig {
                agent: agent.into(),
                instance: format!("inst-{who}"),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.clone(),
                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store,
        )
        .unwrap()
    };
    let count = |e: &Estate| -> usize {
        let v = rpc(e, "channel.subscribe", json!({"topic": topic, "cursor": 0, "limit": 100}));
        v["result"]["messages"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    let before = count(&e);
    let a = mk("a", "fx-a");
    a.send("fx-b", "inst-b", Some("c1"), "real message", None).unwrap();
    let st = termlink_sidecar::wire::Stage {
        from: a.fingerprint().to_string(),
        from_agent: "fx-a".into(),
        to_agent: "fx-b".into(),
        conv: "c1".into(),
        msg_id: "00112233445566778899aabbccddeeff".into(),
        stage: "STORED".into(),
        ts_ms: 1_800_000_000_000,
        chain_head: "0".repeat(64),
    };
    a.post_stage("fx-b", &st).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while count(&e) < before + 2 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(count(&e), before + 2, "the real msg and stage of fx-a were both stored in enforce");
    // an impostor sidecar with its own key, configured as fx-a
    let z = mk("z", "fx-a");
    z.send("fx-b", "inst-b", Some("c2"), "impostor", None).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(count(&e), before + 2, "the impostor's post was refused");
}

/// Review r2 MAJOR 2/5: the hub never gates types other than the two arc-011 ones (NID), so the
/// receiver must be the backstop: a record of any other type on the inbox, claiming any agent, is
/// skipped with an entry, never handled as a message.
#[test]
fn an_unlisted_type_on_the_inbox_is_skipped_never_handled_as_mail() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "fx-a").unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: "fx-a".into(),
                instance: "inst-a".into(),
                identity: e.identity("a").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.clone(),
                queue_path: e.root().join("outbox-a.sqlite"),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: "fx-a".into(),
            instance: "inst-a".into(),
            hub: e.hub_addr(),
            declared_hub_id: id.clone(),
            roster: Roster::new(),
            timeout: Duration::from_secs(10),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender,
        Box::new(BuiltIn::new(Roster::new())),
    )
    .unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    let forger = Identity::generate();
    let payload = json!({"v": 1, "kind": "msg", "from_agent": "fx-b", "to_agent": "fx-a", "body": "forged under another type"}).to_string();
    let ts = 1_800_000_000_000i64;
    let sig = forger.sign_hex(&canonical_sign_bytes(&topic, "sidecar.receipt", payload.as_bytes(), None, ts));
    let ok = rpc(
        &e,
        "channel.post",
        json!({"topic": topic, "msg_type": "sidecar.receipt",
               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
               "ts": ts, "sender_id": forger.fingerprint(), "sender_pubkey_hex": forger.public_key_hex(),
               "signature_hex": sig}),
    );
    assert!(ok.get("result").is_some(), "the hub stores other types (NID): {ok}");
    let _ = rx.pass();
    assert!(
        kinds(&store).iter().any(|(k, d)| k == "inbox-unknown-type" && d.contains("sidecar.receipt")),
        "{:?}",
        kinds(&store)
    );
}
