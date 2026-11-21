//! T-3832 (P-24 slice b, design 3.8.1, E-60) on a real, then stopped, then restarted hub: a sender whose own
//! hub cannot be reached keeps the message durable and shows it as WAITING with the reason "home hub
//! unreachable since T", T being the first failed post after the last success (a later failure does not move
//! it). Nothing claims RECEIVED from the local outbox. Once a post succeeds the reason is gone.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use termlink_sidecar::api::{Api, ApiConfig, Scope};
use termlink_sidecar::send::{HUB_UNREACHABLE_STATE, PumpResult, SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

fn rpc(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    serde_json::to_value(&resp).unwrap()
}

fn message(e: &Estate, store: &Arc<Store>, msg_id: &str) -> Value {
    let api = Api::start(ApiConfig::new("fx-b", &e.runtime_dir()), store.clone()).unwrap();
    let mut s = UnixStream::connect(api.path(Scope::Status)).unwrap();
    writeln!(s, "{}", json!({"op": "status", "msg_id": msg_id})).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    let v: Value = serde_json::from_str(&line).unwrap();
    v["messages"].as_array().and_then(|ms| ms.iter().find(|m| m["msg_id"] == msg_id).cloned()).unwrap_or(Value::Null)
}

fn reached_received(v: &Value) -> bool {
    v["record"].as_array().into_iter().flatten().any(|r| r["stage"] == "RECEIVED" && r.get("absent").is_none())
}

#[test]
fn t3832_waiting_names_the_home_hub_outage_from_its_first_failed_post() {
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-a")}));
    let store = Arc::new(Store::open(&e.store_root(), "fx-b").unwrap());
    let sender = Sender::new(
        SendConfig {
            agent: "fx-b".into(),
            instance: "inst-b".into(),
            identity: e.identity("b").unwrap(),
            hub: e.hub_addr(),
            declared_hub_id: id.clone(),
            queue_path: e.root().join("outbox-b.sqlite"),
            timeout: Duration::from_secs(5),
            pump_interval: None,
        },
        store.clone(),
    )
    .unwrap();

    // Hub up: a normal post leaves no outage record.
    let m0 = sender.send("fx-a", "inst-a", Some("c0"), "while up", None).unwrap();
    assert!(store.read_state(HUB_UNREACHABLE_STATE).unwrap().is_none());
    assert!(message(&e, &store, &m0.msg_id).get("reason").is_none_or(Value::is_null));

    // The hub goes away: the send still returns an id (durable in the outbox) and the message waits with a reason.
    e.stop_hub();
    let m1 = sender.send("fx-a", "inst-a", Some("c1"), "while down", None).unwrap();
    let v1 = message(&e, &store, &m1.msg_id);
    assert_eq!(v1["label"], "WAITING", "{v1}");
    let t = v1["unreachable_since_ms"].as_u64().expect("since when");
    assert_eq!(v1["reason"], format!("home hub unreachable since {t}"), "{v1}");
    assert!(!reached_received(&v1), "nothing claims RECEIVED from the local outbox: {v1}");

    // A later failure does not move T (it is the FIRST failed post after the last success).
    std::thread::sleep(Duration::from_millis(20));
    let m2 = sender.send("fx-a", "inst-a", Some("c1"), "still down", None).unwrap();
    assert!(matches!(sender.pump(), PumpResult::Waiting));
    assert_eq!(message(&e, &store, &m2.msg_id)["unreachable_since_ms"].as_u64(), Some(t));

    // The hub is back: the pump posts, the outage record is gone, and the reason with it.
    e.restart_hub().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while !matches!(sender.pump(), PumpResult::Drained) {
        assert!(std::time::Instant::now() < deadline, "the pump never drained after the hub came back");
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(store.read_state(HUB_UNREACHABLE_STATE).unwrap().is_none(), "cleared by the successful post");
    let v1 = message(&e, &store, &m1.msg_id);
    assert_ne!(v1["label"], "WAITING", "posted now: {v1}");
    assert!(v1.get("reason").is_none_or(Value::is_null), "no outage reason once posted: {v1}");
}
