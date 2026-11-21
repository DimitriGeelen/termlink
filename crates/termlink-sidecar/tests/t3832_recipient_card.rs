//! T-3832 (design 3.6.3, 3.8.2): the sender's status shows the recipient's reachability card, and past the
//! card's expiry (PN-9) a message that would read STUCK or UNKNOWN reads WAITING FOR RECIPIENT instead.
//! Hermetic: a store in a throwaway estate, no hub; the card read is written the way the monitor writes it.
use serde_json::{Value, json};
use termlink_sidecar::api;
use termlink_sidecar::deadline::{Deadlines, OVERDUE_HANDOVER};
use termlink_sidecar::liveness::{CARD_EXPIRE_MS, CARDS_STATE};
use termlink_sidecar::store::{LineExtra, Store};
use termlink_sidecar::testkit::Estate;

const D: Deadlines = Deadlines::R44E;

/// A sender store with message `m` to agent-b at the given peer stages, and (when given) agent-b's card
/// sampled `card_age` ms before the view. Returns the message's status `offset` ms after its POSTED line.
fn view(peer_stages: &[(&str, u64)], card_age: Option<u64>, offset: u64) -> Value {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-a").unwrap();
    let sent = LineExtra { reason: Some("priority=0".into()), ..Default::default() };
    s.append_event_unless("c", "SENT", "m", "fp-a", "d", sent, |_| false).unwrap();
    s.append_event("c", "POSTED", "m", "fp-a", "d").unwrap();
    termlink_sidecar::revocation::record_sent_to(&s, "m", "agent-b");
    let posted = s.chain_lines("c").unwrap().into_iter().find(|l| l.event == "POSTED").and_then(|l| l.ts_ms).unwrap();
    for (st, off) in peer_stages {
        let extra = LineExtra { peer_ts_ms: Some(posted + off), ..Default::default() };
        s.append_event_unless("c", st, "m", "fp-b", "d", extra, |_| false).unwrap();
    }
    let now = posted + offset;
    if let Some(age) = card_age {
        let card = json!({"receiver_up": true, "right_hub": true, "adapter_present": true, "last_surface_ms": null});
        let read = json!({"agent-b": {"read_ms": now, "sample_ms": now - age, "card": card}});
        s.write_state(CARDS_STATE, &read.to_string()).unwrap();
    }
    api::status_view(&json!({"op": "status", "msg_id": "m"}), &s, "agent-a", None, now, &D)["messages"][0].clone()
}

fn label(v: &Value) -> Option<&str> {
    v.get("label").and_then(Value::as_str)
}

#[test]
fn a_fresh_card_is_shown_and_changes_no_label() {
    let v = view(&[("RECEIVED", 0)], Some(1_000), 61_000);
    assert_eq!(label(&v), Some("STUCK"), "{v}");
    assert_eq!(v["recipient_card"]["state"], "fresh");
    assert_eq!(v["recipient_card"]["card"]["receiver_up"], true);
}

/// 3.8.2: received, never stored, past the accept deadline is STUCK while the recipient is up; once its
/// card has expired the recipient is offline and the message waits for it. Negative controls: no card read
/// at all keeps STUCK (nothing is known), and a fresh card keeps STUCK.
#[test]
fn past_the_card_expiry_stuck_reads_waiting_for_recipient() {
    let off = CARD_EXPIRE_MS + 1_000;
    let v = view(&[("RECEIVED", 0)], Some(off), off + 61_000);
    assert_eq!(label(&v), Some("WAITING FOR RECIPIENT"), "{v}");
    assert!(v.get("stuck_step").is_none(), "{v}");
    assert!(v["reason"].as_str().unwrap_or("").contains("recipient offline"), "{v}");
    assert_eq!(v["recipient_card"]["state"], "expired");
    assert_eq!(label(&view(&[("RECEIVED", 0)], None, off + 61_000)), Some("STUCK"));
    assert_eq!(label(&view(&[("RECEIVED", 0)], Some(1_000), off + 61_000)), Some("STUCK"));
    // On the recipient's own overdue notice, too.
    let n = view(&[("RECEIVED", 0), ("STORED", 0), (OVERDUE_HANDOVER, 90_000)], Some(off), off + 100_000);
    assert_eq!(label(&n), Some("WAITING FOR RECIPIENT"), "{n}");
}

/// Posted with nothing back reads UNKNOWN (no signal); past the card expiry it waits for the recipient.
/// A card older than 2 x PN-9 still waits, and its card reads unknown, never dead.
#[test]
fn past_the_card_expiry_unknown_reads_waiting_and_an_old_card_is_never_dead() {
    let off = CARD_EXPIRE_MS + 1_000;
    assert_eq!(label(&view(&[], None, 61_000)), Some("UNKNOWN"));
    assert_eq!(label(&view(&[], Some(off), off + 61_000)), Some("WAITING FOR RECIPIENT"));
    let old = view(&[], Some(3 * CARD_EXPIRE_MS), 3 * CARD_EXPIRE_MS + 61_000);
    assert_eq!(label(&old), Some("WAITING FOR RECIPIENT"), "{old}");
    assert_eq!(old["recipient_card"]["state"], "unknown");
    assert!(!old.to_string().contains("dead"), "{old}");
}
