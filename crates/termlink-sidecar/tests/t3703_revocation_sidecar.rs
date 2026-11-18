//! T-3703 (arc-011 P-15, B-5) slice C at L4: E-39 on a real hub. After `hub.revoke` of fx-a's key, fx-b's
//! sidecar refuses fx-a's next message within one pass (nothing STORED, a signed REVOKED notice to fx-a, which
//! records it), refuses a message from fx-a stored before the revocation and not yet handed over (FCW-200,
//! E-39.6), raises one `key-revoked` entry, and fx-b's own mail to fx-a is not dropped.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
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

fn kinds(store: &Store) -> Vec<String> {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| v["kind"].as_str().unwrap_or("").to_string())
        .collect()
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

/// Every chain line of `conv` as (event, msg_id, reason).
fn lines(store: &Store, conv: &str) -> Vec<(String, String, Option<String>)> {
    store.chain_lines(conv).unwrap_or_default().into_iter().map(|l| (l.event, l.msg_id, l.reason)).collect()
}

#[test]
fn e39_a_revoked_key_is_refused_within_one_pass_and_stored_mail_from_it_is_refused() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    for a in ["fx-a", "fx-b"] {
        let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/{a}")}));
    }
    let a = side(&e, &id, "a", "b");
    let b = side(&e, &id, "b", "a");

    // m1 from fx-a is stored at fx-b before any revocation; not handed over
    let m1 = a.sender.send("fx-b", "inst-b", Some("c1"), "before", None).unwrap();
    let _ = b.rx.pass();
    let c1 = lines(&b.store, "c1");
    assert!(c1.iter().any(|(ev, _, _)| ev == "STORED"), "m1 stored: {c1:?}");
    let st = b.rx.revocation_state();
    assert_eq!(st, termlink_sidecar::revocation::State::Verified, "the empty list verified on the first pass");

    // the operator revokes fx-a's key
    let fp = termlink_session::signed_state::key_fingerprint(&e.identity("a").unwrap().verifying_key());
    let r = rpc(&e, "hub.revoke", json!({"key": fp, "project": "p", "agent": "fx-a", "reason": "key exposed"}));
    assert_eq!(r["result"]["seq"], 1, "{r}");

    // fx-a sends again at once; ONE pass of fx-b refuses it (head read before any message is handled)
    let _m2 = a.sender.send("fx-b", "inst-b", Some("c2"), "after", None).unwrap();
    let _ = b.rx.pass();
    let c2 = lines(&b.store, "c2");
    assert!(c2.iter().any(|(ev, _, r)| ev == "REFUSED" && r.as_deref() == Some("revoked")), "m2 refused: {c2:?}");
    assert!(!c2.iter().any(|(ev, _, _)| ev == "STORED"), "nothing STORED from the revoked key: {c2:?}");
    // E-39.6 / FCW-200: m1, stored before the revocation, is refused and leaves the pending set
    let c1 = lines(&b.store, "c1");
    assert!(c1.iter().any(|(ev, _, r)| ev == "REFUSED" && r.as_deref() == Some("revoked")), "m1 refused: {c1:?}");
    let status = termlink_sidecar::revocation::status(&b.store);
    assert!(status["revocation"].as_str().unwrap().starts_with("verified seq 1 at"), "{status}");
    assert_eq!(status["refused_revoked"], 2, "{status}");
    assert_eq!(kinds(&b.store).iter().filter(|k| *k == "key-revoked").count(), 1);

    // fx-a's side: the signed REVOKED notices arrive and are recorded on its chain
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let _ = a.rx.pass();
        let c2a = lines(&a.store, "c2");
        if c2a.iter().any(|(ev, _, _)| ev == "REVOKED") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "fx-a never recorded the REVOKED notice: {c2a:?}");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(lines(&a.store, "c1").iter().any(|(ev, id, _)| ev == "REVOKED" && *id == m1.msg_id), "m1's notice too");

    // mail to the revoked agent waits (is not dropped): fx-b -> fx-a is stored at fx-a
    b.sender.send("fx-a", "inst-a", Some("c3"), "to the revoked agent", None).unwrap();
    let _ = a.rx.pass();
    assert!(lines(&a.store, "c3").iter().any(|(ev, _, _)| ev == "STORED"), "mail to fx-a kept");
}

/// r2 CC 1: the stale rule runs when the hub cannot be asked at all (`hub.version` fails), through the real pass.
#[test]
fn e41_l4_a_hub_outage_makes_the_list_stale_through_the_receive_pass() {
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-b")}));
    let b = side(&e, &id, "b", "a");
    let window = Duration::from_secs(3);
    b.rx.set_revocation_stale_after(window);
    let _ = b.rx.pass();
    let verified_at = std::time::Instant::now();
    assert_eq!(b.rx.revocation_state(), termlink_sidecar::revocation::State::Verified);
    e.stop_hub();
    let _ = b.rx.pass();
    // timing-safe: "not yet stale" is asserted only when this pass really ran inside the window
    if verified_at.elapsed() < window {
        assert_eq!(b.rx.revocation_state(), termlink_sidecar::revocation::State::Verified, "not yet past PN-13");
    }
    std::thread::sleep((window + Duration::from_millis(500)).saturating_sub(verified_at.elapsed()));
    let _ = b.rx.pass();
    assert_eq!(b.rx.revocation_state(), termlink_sidecar::revocation::State::Stale);
    assert!(termlink_sidecar::revocation::narrowing(&b.store));
    assert!(kinds(&b.store).iter().any(|k| k == "revocation-stale"));
    let st = termlink_sidecar::revocation::status(&b.store);
    assert!(st["revocation"].as_str().unwrap().starts_with("stale since"), "{st}");
}
