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

/// T-3832 (design 3.6.1): the note a P-24 sidecar posts still parses with the note struct as it was
/// before P-24 (no `card` field), so an old reader is not broken by the card.
#[test]
fn a_p24_note_parses_with_the_pre_p24_struct() {
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct PreP24Note {
        agent: String,
        instance: String,
        sample_ms: u64,
        drained_up_to: u64,
        receiving: String,
    }
    let n = Note {
        agent: "a".into(),
        instance: "a.w1".into(),
        sample_ms: 5,
        drained_up_to: 2,
        receiving: "on".into(),
        card: Some(Card { receiver_up: true, right_hub: true, adapter_present: false, last_surface_ms: Some(4) }),
    };
    let body = serde_json::to_vec(&n).unwrap();
    assert!(String::from_utf8_lossy(&body).contains("\"card\""));
    let old: PreP24Note = serde_json::from_slice(&body).unwrap();
    assert_eq!((old.sample_ms, old.drained_up_to), (5, 2));
    // and a pre-P-24 note parses with the new struct, as a note without a card
    let pre = br#"{"agent":"a","instance":"a.w1","sample_ms":5,"drained_up_to":2,"receiving":"on"}"#;
    assert_eq!(serde_json::from_slice::<Note>(pre).unwrap().card, None);
}

/// T-3832 (design 3.6.3): fresh within PN-9, expired past it, unknown past 2 x PN-9 (never dead), and
/// unknown for anything without a card.
#[test]
fn card_view_is_fresh_then_expired_then_unknown_never_dead() {
    let card = serde_json::json!({"receiver_up": true, "right_hub": true, "adapter_present": true, "last_surface_ms": 1});
    let read = serde_json::json!({"read_ms": 1_000, "sample_ms": 1_000, "card": card});
    let at = |now: u64| card_view(Some(&read), now, CARD_EXPIRE_MS);
    assert_eq!(at(1_000 + CARD_EXPIRE_MS)["state"], "fresh");
    assert_eq!(at(1_000 + CARD_EXPIRE_MS)["card"], card);
    assert_eq!(at(1_000 + CARD_EXPIRE_MS + 1)["state"], "expired");
    assert!(at(1_000 + CARD_EXPIRE_MS + 1)["card"].is_null());
    assert_eq!(at(1_000 + 2 * CARD_EXPIRE_MS)["state"], "expired");
    assert_eq!(at(1_000 + 2 * CARD_EXPIRE_MS + 1)["state"], "unknown");
    for now in [0, 1_000, 10 * CARD_EXPIRE_MS, u64::MAX] {
        assert_ne!(at(now)["state"], "dead");
    }
    assert_eq!(card_view(None, 5, CARD_EXPIRE_MS)["state"], "unknown");
    let err = serde_json::json!({"read_ms": 5, "error": "hub unreadable: refused"});
    assert_eq!(card_view(Some(&err), 5, CARD_EXPIRE_MS)["state"], "unknown");
}

/// T-3832 (design 3.6.1, 3.6.3): a sidecar that is not the main posts its own note with a card, and a
/// sender that sent to it reads that card from the hub; a stranger's key is not believed.
#[test]
fn every_sidecar_posts_a_card_and_a_sender_reads_its_recipients_card() {
    let e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();
    let (a_store, a_sender) = sender(&e, "a", "agent-a", &hub_id);
    let (b_store, b_sender) = sender(&e, "b", "agent-b", &hub_id);
    let mut roster = Roster::new();
    roster.add(e.identity("a").unwrap().verifying_key());
    roster.add(e.identity("b").unwrap().verifying_key());
    // No main pinned at all: the monitor still posts its own card.
    let a = monitor("agent-a", Book::default(), roster.clone(), a_store.clone(), a_sender.clone(), &hub_id);
    let b = monitor("agent-b", Book::default(), roster.clone(), b_store.clone(), b_sender.clone(), &hub_id);
    let hub = RpcHub::new(e.hub_addr(), Duration::from_secs(3)).unwrap();
    b_sender.send("agent-a", "agent-a.w1", None, "hello a", None).unwrap();
    let t0 = now_ms();
    assert_eq!(a.cycle(&hub, t0).state, "unknown"); // no main: unknown, and the card is posted anyway
    b.cycle(&hub, t0);
    let cards = stored_cards(&b_store);
    let r = cards.get("agent-a").expect("b read the card of the agent it sent to");
    assert_eq!(r["sample_ms"], t0, "{r}");
    assert!(r["card"]["receiver_up"].is_boolean() && r["card"]["right_hub"].is_boolean(), "{r}");
    assert_eq!(card_view(Some(r), t0, CARD_EXPIRE_MS)["state"], "fresh");
    // Signed by a key b does not know: unknown, never a card.
    let stranger = monitor("agent-b", Book::default(), Roster::new(), b_store.clone(), b_sender.clone(), &hub_id);
    stranger.read_cards(&hub, t0);
    let r = stored_cards(&b_store).get("agent-a").cloned().unwrap();
    assert!(r["error"].as_str().unwrap_or("").contains("unknown key"), "{r}");
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
