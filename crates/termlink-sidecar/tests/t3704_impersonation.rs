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

    let _ = rx.pass();
    let hits: Vec<_> = kinds(&store).into_iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").collect();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].1.contains("fx-a") && hits[0].1.contains("1 refused"), "{}", hits[0].1);

    // the next pass with no new refusal adds nothing
    let _ = rx.pass();
    assert_eq!(kinds(&store).iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").count(), 1);
}
