//! T-3462: the sender keeps a message until the recipient reports STORED, resends
//! it on the R-44.e accept deadline with growing gaps, then gives up loudly.
//! Hermetic: testkit estate (temp dir, throwaway hub) only.

use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use termlink_sidecar::deadline::Deadlines;
use termlink_sidecar::send::*;
use termlink_sidecar::store::{ChainLine, LineExtra, Store, now_ms};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire;

struct Rig {
    e: Estate,
    store: Arc<Store>,
    sender: Sender,
}

fn cfg(e: &Estate) -> SendConfig {
    SendConfig {
        agent: "agent-a".into(),
        instance: "inst-a".into(),
        identity: e.identity("a").unwrap(),
        hub: e.hub_addr(),
        declared_hub_id: e.hub_id().unwrap(),
        queue_path: e.root().join("outbox-a.sqlite"),
        timeout: Duration::from_secs(3),
        pump_interval: None,
    }
}

fn rig() -> Rig {
    let e = Estate::create().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let sender = Sender::new(cfg(&e), store.clone()).unwrap();
    Rig { e, store, sender }
}

fn events(r: &Rig, conv: &str, msg_id: &str) -> Vec<String> {
    r.store.chain_lines(conv).unwrap().into_iter().filter(|l| l.msg_id == msg_id).map(|l| l.event).collect()
}

fn count(r: &Rig, conv: &str, msg_id: &str, ev: &str) -> usize {
    events(r, conv, msg_id).iter().filter(|e| *e == ev).count()
}

/// The recipient's sidecar reported `stage` (what pull.rs records on the sender's chain).
fn peer_reports(r: &Rig, conv: &str, msg_id: &str, stage: &str) {
    let peer = r.e.identity("b").unwrap().fingerprint().to_owned();
    let sent: ChainLine = r.store.chain_lines(conv).unwrap().into_iter().find(|l| l.event == "SENT" && l.msg_id == msg_id).unwrap();
    r.store
        .append_event_unless(conv, stage, msg_id, &peer, &sent.digest, LineExtra { peer_ts_ms: Some(now_ms()), ..Default::default() }, |_| false)
        .unwrap();
}

fn inbox_len(r: &Rig, to: &str) -> usize {
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
        Ok(v) => v["messages"].as_array().map(Vec::len).unwrap_or(0),
        Err(_) => 0,
    }
}

const DL: Deadlines = Deadlines::R44E;

#[test]
fn resend_posted_message_is_kept_durably_until_stored() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-1"), "hello", None).unwrap();
    assert_eq!(r.sender.outbox_len(), 0, "the outbox row is gone once the hub accepted it");
    assert_eq!(r.sender.awaiting(), vec![s.msg_id.clone()], "but the sender still holds it");
    assert!(r.e.root().join("outbox-a.awaiting").join(format!("{}.json", s.msg_id)).exists());
}

#[test]
fn resend_stored_report_releases_the_message_and_it_is_never_resent() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-2"), "hello", None).unwrap();
    peer_reports(&r, "k-2", &s.msg_id, "RECEIVED");
    // RECEIVED alone is not acceptance: past the deadline it is resent.
    let t = now_ms() + DL.accept(false) + 1_000;
    assert_eq!(r.sender.resend_overdue(t, &DL).resent, vec![s.msg_id.clone()]);
    peer_reports(&r, "k-2", &s.msg_id, "STORED");
    let rep = r.sender.resend_overdue(t + 3_600_000, &DL);
    assert_eq!(rep.released, vec![s.msg_id.clone()]);
    assert!(rep.resent.is_empty() && rep.gave_up.is_empty(), "{rep:?}");
    assert!(r.sender.awaiting().is_empty());
    assert_eq!(count(&r, "k-2", &s.msg_id, RESENT), 1);
    assert_eq!(r.sender.resend_overdue(t + 7_200_000, &DL), ResendReport::default());
}

#[test]
fn resend_a_later_stage_also_counts_as_stored() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-3"), "hello", None).unwrap();
    peer_reports(&r, "k-3", &s.msg_id, "HANDED_OVER");
    let rep = r.sender.resend_overdue(now_ms() + 3_600_000, &DL);
    assert_eq!(rep.released, vec![s.msg_id.clone()]);
    assert_eq!(count(&r, "k-3", &s.msg_id, RESENT), 0);
}

#[test]
fn resend_after_the_normal_accept_deadline_with_the_same_id_and_no_duplicate_on_the_hub() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-4"), "hello", None).unwrap();
    let base = now_ms();
    assert_eq!(r.sender.resend_overdue(base + DL.accept(false) / 2, &DL), ResendReport::default(), "not before the deadline");
    let rep = r.sender.resend_overdue(base + DL.accept(false) + 1_000, &DL);
    assert_eq!(rep.resent, vec![s.msg_id.clone()]);
    assert_eq!(r.sender.outbox_len(), 0, "the resend was posted");
    assert_eq!(count(&r, "k-4", &s.msg_id, RESENT), 1);
    assert_eq!(count(&r, "k-4", &s.msg_id, POSTED), 1, "POSTED is written once");
    // Same client_msg_id: the hub's dedupe absorbs the copy.
    assert_eq!(inbox_len(&r, "agent-b"), 1);
    assert_eq!(r.sender.awaiting(), vec![s.msg_id.clone()], "still held until STORED");
}

#[test]
fn resend_urgent_uses_the_fifteen_second_deadline() {
    let r = rig();
    let n = r.sender.send("agent-b", "inst-b", Some("k-5n"), "normal", Some("3")).unwrap();
    let u = r.sender.send("agent-b", "inst-b", Some("k-5u"), "urgent", Some("7")).unwrap();
    let rep = r.sender.resend_overdue(now_ms() + DL.accept(true) + 1_000, &DL);
    assert_eq!(rep.resent, vec![u.msg_id.clone()], "only the urgent one is due at 16 s");
    assert_eq!(count(&r, "k-5n", &n.msg_id, RESENT), 0);
}

#[test]
fn resend_gaps_grow_and_the_sender_gives_up_with_a_needs_attention_entry() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-6"), "hello", None).unwrap();
    let d = DL.accept(false);
    let mut t = now_ms();
    for n in 0..MAX_RESENDS {
        let gap = d << n;
        assert!(r.sender.resend_overdue(t + gap - 1_000, &DL).resent.is_empty(), "attempt {n} not before gap {gap}");
        t += gap + 1_000;
        assert_eq!(r.sender.resend_overdue(t, &DL).resent, vec![s.msg_id.clone()], "attempt {n}");
    }
    assert_eq!(count(&r, "k-6", &s.msg_id, RESENT), MAX_RESENDS as usize);
    let last_gap = d << MAX_RESENDS;
    assert!(r.sender.resend_overdue(t + last_gap - 1_000, &DL).gave_up.is_empty());
    let rep = r.sender.resend_overdue(t + last_gap + 1_000, &DL);
    assert_eq!(rep.gave_up, vec![s.msg_id.clone()]);
    assert!(rep.resent.is_empty());
    assert!(r.sender.awaiting().is_empty());
    assert_eq!(count(&r, "k-6", &s.msg_id, NOT_ACCEPTED), 1);
    let attn: Vec<Value> = r.store.attention_entries().iter().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert!(
        attn.iter().any(|a| a["kind"] == ATTN_NOT_ACCEPTED && a["detail"].as_str().unwrap().contains(&s.msg_id)),
        "{attn:?}"
    );
    // Nothing more: bounded, no flooding.
    assert_eq!(r.sender.resend_overdue(t + 100 * last_gap, &DL), ResendReport::default());
    assert_eq!(inbox_len(&r, "agent-b"), 1);
}

#[test]
fn resend_record_survives_a_sender_restart() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-7"), "hello", None).unwrap();
    let Rig { e, store, sender } = r;
    drop(sender);
    let again = Sender::new(cfg(&e), store.clone()).unwrap();
    assert_eq!(again.awaiting(), vec![s.msg_id.clone()]);
    let rep = again.resend_overdue(now_ms() + DL.accept(false) + 1_000, &DL);
    assert_eq!(rep.resent, vec![s.msg_id.clone()]);
}

#[test]
fn resend_stage_records_are_not_held() {
    let r = rig();
    let st = wire::Stage {
        from: r.sender.fingerprint().to_owned(),
        from_agent: "agent-a".into(),
        to_agent: "agent-b".into(),
        conv: "k-8".into(),
        msg_id: "m-1".into(),
        stage: "STORED".into(),
        ts_ms: now_ms() as i64,
        chain_head: String::new(),
    };
    r.sender.post_stage("agent-b", &st).unwrap();
    assert_eq!(r.sender.outbox_len(), 0);
    assert!(r.sender.awaiting().is_empty());
}

/// T-3510: a held record the sender cannot parse is still a held message. It must
/// be kept (never silently dropped) and raised as a needs-attention item.
fn awaiting_dir(r: &Rig) -> std::path::PathBuf {
    r.e.root().join("outbox-a.awaiting")
}

fn attention_for(r: &Rig, kind: &str, needle: &str) -> bool {
    r.store
        .attention_entries()
        .iter()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .any(|a| a["kind"] == kind && a["detail"].as_str().unwrap_or_default().contains(needle))
}

#[test]
fn unreadable_awaiting_corrupt_record_is_kept_and_raised_not_dropped() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-10"), "hello", None).unwrap();
    let rec = awaiting_dir(&r).join(format!("{}.json", s.msg_id));
    std::fs::write(&rec, b"{not json").unwrap();
    assert_eq!(r.sender.awaiting(), vec![s.msg_id.clone()], "the message is still held");
    assert!(attention_for(&r, "awaiting-unreadable", &s.msg_id), "the operator is told");
    assert!(!rec.exists(), "the corrupt record is moved aside, not left to be re-read");
    assert_eq!(std::fs::read(rec.with_extension("unreadable")).unwrap(), b"{not json", "its bytes are kept");
}

#[test]
fn unreadable_awaiting_record_file_is_kept_and_raised_not_dropped() {
    let r = rig();
    let s = r.sender.send("agent-b", "inst-b", Some("k-11"), "hello", None).unwrap();
    // A directory named like a record: listed, matches `.json`, cannot be read as a file
    // (EISDIR), and unlike chmod 000 it fails for root too.
    let rec = awaiting_dir(&r).join(format!("{}.json", s.msg_id));
    std::fs::remove_file(&rec).unwrap();
    std::fs::create_dir(&rec).unwrap();
    assert_eq!(r.sender.awaiting(), vec![s.msg_id.clone()], "the message is still held");
    assert!(attention_for(&r, "awaiting-unreadable", &s.msg_id), "the operator is told");
    assert!(rec.with_extension("unreadable").exists(), "the unreadable record is kept under its quarantine name");
}

#[test]
fn unreadable_awaiting_dir_is_an_error_not_an_empty_list() {
    let r = rig();
    // The awaiting "dir" is a regular file: it exists, but cannot be listed (ENOTDIR).
    std::fs::write(awaiting_dir(&r), b"not a directory").unwrap();
    assert!(r.sender.awaiting().is_empty());
    assert!(attention_for(&r, "awaiting-unreadable", "outbox-a.awaiting"), "an unreadable dir must raise attention");
}
