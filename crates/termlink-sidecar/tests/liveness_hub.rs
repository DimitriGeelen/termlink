//! T-3646: the pinned main's liveness note against a throwaway hub. Hermetic: testkit estate only.
//!
//! The main (agent-a) posts its note; a peer (agent-b) reads it from the hub and judges it.
//! `cycle(hub, now)` takes the clock as a parameter, so stale and stuck are driven by moving
//! `now`, never by sleeping.

use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use termlink_sidecar::book::Book;
use termlink_sidecar::liveness::*;
use termlink_sidecar::send::*;
use termlink_sidecar::store::{Store, now_ms};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

const STALE: u64 = 1_000;

fn sender(e: &Estate, who: &str, agent: &str, hub_id: &str) -> (Arc<Store>, Arc<Sender>) {
    let store = Arc::new(Store::open(&e.store_root(), agent).unwrap());
    let s = Sender::new(
        SendConfig {
            agent: agent.into(),
            instance: format!("{agent}.w1"),
            identity: e.identity(who).unwrap(),
            hub: e.hub_addr(),
            declared_hub_id: hub_id.into(),
            queue_path: e.root().join(format!("outbox-{agent}.sqlite")),
            timeout: Duration::from_secs(3),
            pump_interval: None,
        },
        store.clone(),
    )
    .unwrap();
    (store, Arc::new(s))
}

fn monitor(agent: &str, book: Book, roster: Roster, store: Arc<Store>, sender: Arc<Sender>, hub_id: &str) -> Monitor {
    Monitor {
        book,
        agent: agent.into(),
        instance: format!("{agent}.w1"),
        hub_id: hub_id.into(),
        roster,
        own_fingerprint: sender.fingerprint().to_owned(),
        store,
        sender,
        shared: Shared::new(),
        every_ms: NOTE_EVERY_MS,
        stale_after_ms: STALE,
    }
}

fn kinds(s: &Store) -> Vec<String> {
    s.attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|v| v["kind"].as_str().map(str::to_owned))
        .filter(|k| k.starts_with("main-"))
        .collect()
}

#[test]
fn liveness_hub_alive_then_stuck_then_stale_then_unknown_when_the_hub_is_gone() {
    let mut e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();
    let (a_store, a_sender) = sender(&e, "a", "agent-a", &hub_id);
    let (b_store, b_sender) = sender(&e, "b", "agent-b", &hub_id);
    let mut roster = Roster::new();
    roster.add(e.identity("a").unwrap().verifying_key());
    roster.add(e.identity("b").unwrap().verifying_key());
    let a_book = Book { peers: BTreeMap::from([("agent-b".into(), "agent-b.w1".into())]), main: Some("agent-a".into()) };
    let b_book = Book { peers: BTreeMap::from([("agent-a".into(), "agent-a.w1".into())]), main: Some("agent-a".into()) };
    let a = monitor("agent-a", a_book, roster.clone(), a_store.clone(), a_sender.clone(), &hub_id);
    let b = monitor("agent-b", b_book, roster.clone(), b_store.clone(), b_sender.clone(), &hub_id);
    let hub = RpcHub::new(e.hub_addr(), Duration::from_secs(3)).unwrap();

    // Before the main has ever posted: unknown, never alive.
    let t0 = now_ms();
    assert_eq!(b.cycle(&hub, t0).state, "unknown");

    // The main posts; both sides read it as alive, with the cursor it reported.
    a_store.write_state(termlink_sidecar::pull::CURSOR_STATE, "0").unwrap();
    assert_eq!(a.cycle(&hub, t0).state, "alive");
    let v = b.cycle(&hub, t0);
    assert_eq!((v.state, v.drained_up_to), ("alive", Some(0)));
    let shown = termlink_sidecar::liveness::status_view(Some(&b.book), Some((&b.shared, NOTE_EVERY_MS)), t0);
    assert_eq!(shown["state"], "alive");

    // Mail for the main that it never drains: once the main reports with that mail older than
    // the threshold, the peer sees stuck and alarms once.
    b_sender.send("agent-a", "agent-a.w1", None, "hello main", None).unwrap();
    let sent_at = now_ms();
    let later = sent_at + 5 * STALE;
    assert_eq!(a.cycle(&hub, later).state, "stuck");
    assert_eq!(b.cycle(&hub, later).state, "stuck");
    assert_eq!(b.cycle(&hub, later).state, "stuck");
    assert_eq!(kinds(&b_store), vec![ALERT_STUCK]);

    // The main goes silent: its last note ages past the threshold.
    let v = b.cycle(&hub, later + 2 * STALE);
    assert_eq!(v.state, "stale");
    assert_eq!(v.note_age_ms, Some(2 * STALE));
    assert_eq!(kinds(&b_store), vec![ALERT_STUCK, ALERT_STALE]);

    // It comes back having drained its inbox: alive, and one recovery entry.
    let back = later + 3 * STALE;
    a_store.write_state(termlink_sidecar::pull::CURSOR_STATE, "1").unwrap();
    assert_eq!(a.cycle(&hub, back).state, "alive");
    let v = b.cycle(&hub, back);
    assert_eq!((v.state, v.drained_up_to), ("alive", Some(1)));
    assert_eq!(kinds(&b_store), vec![ALERT_STUCK, ALERT_STALE, ALERT_RECOVERED]);

    // The hub goes away: unknown, never alive, and no alarm churn.
    e.stop_hub();
    let v = b.cycle(&hub, back);
    assert_eq!(v.state, "unknown");
    assert!(v.reason.contains("hub unreadable"), "{}", v.reason);
    assert_eq!(kinds(&b_store).len(), 3);
}
