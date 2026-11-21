//! T-3834 (RK-1) at L4, on a real hub: fx-a and fx-b are carded, the operator revokes fx-a's key, and fx-b
//! sends to fx-a. fx-b's status shows the message as WAITING FOR RECIPIENT with the reason "key revoked,
//! operator action needed", fx-b's store gets ONE `recipient-key-revoked` entry (not one per pass), a message
//! sent before the revocation to the same, then-healthy recipient carried no such reason, and the SENT chain
//! line is unchanged (the recipient is kept outside the hashed chain).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use termlink_sidecar::api::{Api, ApiConfig, Scope};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::revocation::{RECIPIENT_REVOKED_KIND, RECIPIENT_REVOKED_REASON};
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

fn entries(store: &Store, kind: &str) -> usize {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["kind"] == kind)
        .count()
}

struct Side {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
}

fn side(e: &Estate, id: &str, who: &str, peer: &str) -> Side {
    let agent = format!("fx-{who}");
    let store = Arc::new(Store::open(&e.store_root(), &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: format!("inst-{who}"),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.to_string(),
                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let mut roster = Roster::new();
    roster.add(e.identity(peer).unwrap().verifying_key());
    let rx = Receiver::new(
        ReceiveConfig {
            agent: agent.clone(),
            instance: format!("inst-{who}"),
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

/// The status view of one message, through fx-b's real status socket.
fn message(e: &Estate, store: &Arc<Store>, msg_id: &str) -> Value {
    let api = Api::start(ApiConfig::new("fx-b", &e.runtime_dir()), store.clone()).unwrap();
    let mut s = UnixStream::connect(api.path(Scope::Status)).unwrap();
    writeln!(s, "{}", json!({"op": "status", "msg_id": msg_id})).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    let v: Value = serde_json::from_str(&line).unwrap();
    v["messages"].as_array().and_then(|ms| ms.iter().find(|m| m["msg_id"] == msg_id).cloned()).unwrap_or(Value::Null)
}

/// The SENT line of `msg_id` on `conv`, as stored (hash included).
fn sent_line(store: &Store, conv: &str, msg_id: &str) -> String {
    store
        .chain_lines(conv)
        .unwrap()
        .into_iter()
        .find(|l| l.event == "SENT" && l.msg_id == msg_id)
        .map(|l| format!("{} {} {} {:?} {}", l.seq, l.msg_id, l.sender, l.reason, l.hash))
        .unwrap()
}

#[test]
fn t3834_sender_names_a_revoked_recipient_key_once() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    for a in ["fx-a", "fx-b"] {
        let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/{a}")}));
        let who = a.trim_start_matches("fx-");
        let put = rpc(&e, "hub.key_card_put", json!({"agent": a, "project": "proj-p",
            "signing_key": format!("ed25519-pub:{}", e.identity(who).unwrap().public_key_hex())}));
        assert_eq!(put["result"]["ok"], true, "{put}");
    }
    let _a = side(&e, &id, "a", "b");
    let b = side(&e, &id, "b", "a");

    // Control: before any revocation, a message to fx-a carries no revoked-key reason.
    let m0 = b.sender.send("fx-a", "inst-a", Some("c0"), "before", None).unwrap();
    let _ = b.rx.pass();
    let v0 = message(&e, &b.store, &m0.msg_id);
    assert!(v0.get("reason").is_none() || v0["reason"].is_null(), "no reason while fx-a's key is healthy: {v0}");
    assert_eq!(entries(&b.store, RECIPIENT_REVOKED_KIND), 0);

    // The operator revokes fx-a's key; fx-b sends to fx-a.
    let fp = termlink_session::signed_state::key_fingerprint(&e.identity("a").unwrap().verifying_key());
    let r = rpc(&e, "hub.revoke", json!({"key": fp, "project": "proj-p", "agent": "fx-a", "reason": "key exposed"}));
    assert_eq!(r["result"]["seq"], 1, "{r}");
    let m1 = b.sender.send("fx-a", "inst-a", Some("c1"), "after", None).unwrap();
    let sent_before = sent_line(&b.store, "c1", &m1.msg_id);

    // Two passes: the first takes the new revocation list, the second sees fx-a's current key on it.
    let _ = b.rx.pass();
    let _ = b.rx.pass();
    let v1 = message(&e, &b.store, &m1.msg_id);
    assert_eq!(v1["label"], "WAITING FOR RECIPIENT", "{v1}");
    assert_eq!(v1["reason"], RECIPIENT_REVOKED_REASON, "{v1}");
    assert!(v1.get("delivered").is_none(), "never called delivered: {v1}");
    assert_eq!(entries(&b.store, RECIPIENT_REVOKED_KIND), 1, "one entry for (fx-a, its key)");

    // The earlier message to the same recipient now waits for the same reason; still ONE entry.
    let _ = b.rx.pass();
    assert_eq!(message(&e, &b.store, &m0.msg_id)["reason"], RECIPIENT_REVOKED_REASON);
    assert_eq!(entries(&b.store, RECIPIENT_REVOKED_KIND), 1, "coalesced across passes and messages");

    // The recipient is kept outside the chain: the SENT line is byte-identical.
    assert_eq!(sent_line(&b.store, "c1", &m1.msg_id), sent_before);
    assert!(b.store.verify_chain("c1").is_ok());
}
