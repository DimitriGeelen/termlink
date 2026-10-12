//! B7 tests for C-6 (pull.rs) against a throwaway hub. Hermetic: testkit estate only.

use base64::Engine as _;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::*;
use termlink_sidecar::send::*;
use termlink_sidecar::store::{ChainLine, ChainStatus, Step, Store, StoreError, arm, disarm};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::{self, Msg, Roster, Stage};

const A: &str = "agent-a";
const B: &str = "agent-b";
const INST_A: &str = "inst-a";
const INST_B: &str = "inst-b";

/// Each test starts its own hub. On a loaded host, eight hubs starting at once
/// miss the 5 s `hub.version` bound, so the hub tests here run one at a time.
static HUB_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    HUB_SERIAL.lock().unwrap_or_else(|p| p.into_inner())
}

struct Agent {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
}

fn roster(e: &Estate) -> Roster {
    let mut r = Roster::new();
    r.add(e.identity("a").unwrap().verifying_key());
    r.add(e.identity("b").unwrap().verifying_key());
    r
}

fn agent(e: &Estate, who: &str, name: &str, inst: &str, declared: &str) -> Agent {
    let store = Arc::new(Store::open(&e.store_root(), name).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: name.into(),
                instance: inst.into(),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: declared.into(),
                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
                timeout: Duration::from_secs(15),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: name.into(),
            instance: inst.into(),
            hub: e.hub_addr(),
            declared_hub_id: declared.into(),
            roster: roster(e),
            timeout: Duration::from_secs(15),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster(e))),
    )
    .unwrap();
    Agent { store, sender, rx }
}

fn pair() -> (Estate, Agent, Agent) {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let a = agent(&e, "a", A, INST_A, &id);
    let b = agent(&e, "b", B, INST_B, &id);
    (e, a, b)
}

fn lines(s: &Store, conv: &str, msg_id: &str) -> Vec<ChainLine> {
    s.chain_lines(conv).unwrap().into_iter().filter(|l| l.msg_id == msg_id).collect()
}

fn names(ls: &[ChainLine]) -> Vec<String> {
    ls.iter().map(|l| l.event.clone()).collect()
}

fn count(s: &Store, conv: &str, event: &str) -> usize {
    s.chain_lines(conv).map(|ls| ls.iter().filter(|l| l.event == event).count()).unwrap_or(0)
}

fn caught_up(r: PassResult) -> usize {
    match r {
        PassResult::CaughtUp { handled } => handled,
        other => panic!("expected CaughtUp, got {other:?}"),
    }
}

fn rpc(e: &Estate, m: &str, params: Value) -> Result<Value, String> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), m, params, Duration::from_secs(5)))
        .map_err(|x| x.to_string())?;
    termlink_session::client::unwrap_result(resp)
}

/// Post raw bytes to a topic. The hub-level signature is made with `hub_signer`
/// over `payload` (so the hub accepts); `meta` is whatever the test wants the
/// receiver to see.
fn raw_post(e: &Estate, topic: &str, msg_type: &str, payload: &[u8], ts: i64, hub_signer: &Identity, meta: BTreeMap<String, String>) {
    let _ = rpc(e, "channel.create", json!({"name": topic}));
    let sig = hub_signer.sign(&canonical_sign_bytes(topic, msg_type, payload, None, ts));
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    rpc(
        e,
        "channel.post",
        json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "ts": ts,
            "sender_id": hub_signer.fingerprint(),
            "sender_pubkey_hex": hub_signer.public_key_hex(),
            "signature_hex": hex(&sig.to_bytes()),
            "metadata": meta,
        }),
    )
    .expect("hub accepts the post");
}

fn meta_of(s: &wire::Signed) -> BTreeMap<String, String> {
    BTreeMap::from([
        (wire::META_PK.to_owned(), s.signer_pk.clone()),
        (wire::META_SIG.to_owned(), s.sig.clone()),
    ])
}

fn msg(from: &Identity, from_agent: &str, to_instance: &str, conv: &str, id: &str, body: &str) -> Msg {
    Msg {
        from: from.fingerprint().to_owned(),
        from_agent: from_agent.into(),
        from_instance: "inst-a".into(),
        to_agent: B.into(),
        to_instance: to_instance.into(),
        conv: conv.into(),
        msg_id: id.into(),
        seq: 0,
        priority: "0".into(),
        ts_ms: 1,
        body: body.into(),
    }
}

/// Sign `m` with `key` and post it to B's inbox carrying the matching metadata.
fn post_signed(e: &Estate, key: &Identity, m: &Msg) {
    let topic = inbox_topic(&e.hub_id().unwrap(), B);
    let ts = termlink_sidecar::store::now_ms() as i64;
    let s = wire::sign(key, &topic, ts, m);
    raw_post(e, &topic, wire::MSG_TYPE, &s.payload, ts, key, meta_of(&s));
}

#[test]
fn receive_hub_message_is_received_then_stored_and_sender_shows_both_with_receiver_times() {
    let _serial = serial();
    let (_e, a, b) = pair();
    let s = a.sender.send(B, INST_B, Some("conv-1"), "hello", None).unwrap();
    assert_eq!(caught_up(b.rx.pass()), 1);

    let bl = lines(&b.store, "conv-1", &s.msg_id);
    assert_eq!(names(&bl), ["RECEIVED", "STORED"]);
    assert!(bl.iter().all(|l| l.sender == a.sender.fingerprint()));
    // The record is durable and is exactly the payload a signed.
    let (_, payload) = b.store.read_record("conv-1", bl[1].seq, &s.msg_id).unwrap();
    assert_eq!(wire::digest_hex(&payload), bl[1].digest);

    // A reads its own inbox: both stage records, verified, with B's times.
    assert_eq!(caught_up(a.rx.pass()), 2);
    let al = lines(&a.store, "conv-1", &s.msg_id);
    assert_eq!(names(&al), ["SENT", "POSTED", "RECEIVED", "STORED"]);
    for (stage, mine) in al[2..].iter().zip(&bl) {
        assert_eq!(stage.sender, b.sender.fingerprint());
        assert_eq!(stage.peer_ts_ms, mine.ts_ms, "the sender shows the receiver's own time");
        assert_eq!(stage.digest, al[0].digest);
    }
    assert!(matches!(a.store.verify_chain("conv-1").unwrap(), ChainStatus::Intact { .. }));
    assert!(matches!(b.store.verify_chain("conv-1").unwrap(), ChainStatus::Intact { .. }));

    // Nothing new: a second pass on either side changes nothing.
    assert_eq!(caught_up(b.rx.pass()), 0);
    assert_eq!(caught_up(a.rx.pass()), 0);
    assert_eq!(lines(&a.store, "conv-1", &s.msg_id).len(), 4);
}

#[test]
fn receive_hub_forged_or_misaddressed_messages_are_refused_never_stored() {
    let _serial = serial();
    let (e, a, b) = pair();
    let ka = e.identity("a").unwrap();
    let host = e.identity("host").unwrap();
    let sibling = e.identity("sibling").unwrap();
    let topic = inbox_topic(&e.hub_id().unwrap(), B);

    // 1 No signature in the metadata (a post made outside the sidecar).
    let m1 = msg(&ka, A, INST_B, "f-1", "m1", "no sig");
    raw_post(&e, &topic, wire::MSG_TYPE, &m1.payload_bytes(), 5, &ka, BTreeMap::new());
    // 2 The shared host key, honest about itself but not rostered.
    post_signed(&e, &host, &msg(&host, A, INST_B, "f-2", "m2", "host key"));
    // 3 A sibling key claiming to be A.
    let m3 = msg(&ka, A, INST_B, "f-3", "m3", "sibling");
    let s3 = wire::sign(&sibling, &topic, 7, &m3);
    raw_post(&e, &topic, wire::MSG_TYPE, &s3.payload, 7, &sibling, meta_of(&s3));
    // 4 Altered after signing: A signed one body, the post carries another.
    let s4 = wire::sign(&ka, &topic, 8, &msg(&ka, A, INST_B, "f-4", "m4", "original"));
    let altered = msg(&ka, A, INST_B, "f-4", "m4", "altered").payload_bytes();
    raw_post(&e, &topic, wire::MSG_TYPE, &altered, 8, &ka, meta_of(&s4));
    // 5 Correctly signed by A, but for another instance (SI-26).
    post_signed(&e, &ka, &msg(&ka, A, "inst-ended", "f-5", "m5", "old instance"));

    assert_eq!(caught_up(b.rx.pass()), 5);
    let reasons = b.store.attention_entries().join("\n");
    for (conv, id, code, why) in [
        ("f-1", "m1", "malformed", "no signature"),
        ("f-3", "m3", "from-mismatch", "does not match the from identity"),
        ("f-4", "m4", "bad-signature", "signature does not verify"),
        ("f-5", "m5", "wrong-instance", "instance inst-ended"),
    ] {
        let ls = lines(&b.store, conv, id);
        assert_eq!(names(&ls), ["REFUSED"], "{conv}");
        assert_eq!(ls[0].reason.as_deref(), Some(code), "{conv}");
        assert_eq!(count(&b.store, conv, "STORED"), 0, "{conv}");
        assert!(reasons.contains(why), "{conv}: {why} not in {reasons}");
    }
    // The shared host key is honest about itself but not on the allow-list
    // (OQ-2 ruling E): stored, held for the operator, never delivered.
    let ls = lines(&b.store, "f-2", "m2");
    assert_eq!(names(&ls), ["RECEIVED", "STORED", "HELD"]);
    assert_eq!(ls[2].reason.as_deref(), Some("awaiting-operator"));
    assert!(reasons.contains("is knocking"), "{reasons}");
    // Its notices go to the inbox it names (agent-a, a name it does not own).
    // A sent no m2, so A ignores all three and shows them: a reflected knock
    // changes nothing on A's chains.
    assert_eq!(caught_up(a.rx.pass()), 3);
    assert!(a.store.chain_lines("f-2").map(|l| l.is_empty()).unwrap_or(true), "A recorded a stage for a message it never sent");
    assert_eq!(a.store.attention_entries().iter().filter(|l| l.contains("was not sent by this agent")).count(), 3);
    // The forged and misaddressed ones are still answered with nothing.
    // A re-read (cursor reset) records each refusal once only.
    b.store.write_state(CURSOR_STATE, "0").unwrap();
    assert_eq!(caught_up(b.rx.pass()), 5);
    assert_eq!(names(&lines(&b.store, "f-4", "m4")), ["REFUSED"]);
}

#[test]
fn receive_hub_store_failure_at_every_step_reports_nothing_then_restart_stores_once() {
    let _serial = serial();
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let a = agent(&e, "a", A, INST_A, &id);
    let mut b = Some(agent(&e, "b", B, INST_B, &id));
    for (i, step) in Step::ALL.iter().enumerate() {
        let conv = format!("k-{i}");
        let s = a.sender.send(B, INST_B, Some(&conv), &format!("kill {step:?}"), None).unwrap();
        arm(*step);
        let r = b.as_ref().unwrap().rx.pass();
        disarm();
        assert!(matches!(r, PassResult::Stalled { .. }), "{step:?}: {r:?}");
        // No STORED stage reached the sender.
        a.rx.pass();
        let al = names(&lines(&a.store, &conv, &s.msg_id));
        assert!(!al.contains(&"STORED".to_string()), "{step:?}: {al:?}");
        // "Kill" B and start it again on the same store.
        drop(b.take());
        b = Some(agent(&e, "b", B, INST_B, &id));
        let bb = b.as_ref().unwrap();
        caught_up(bb.rx.pass());
        let bl = names(&lines(&bb.store, &conv, &s.msg_id));
        assert_eq!(bl.iter().filter(|x| *x == "RECEIVED").count(), 1, "{step:?}: {bl:?}");
        assert_eq!(bl.iter().filter(|x| *x == "STORED").count(), 1, "{step:?}: {bl:?}");
        assert!(matches!(bb.store.verify_chain(&conv).unwrap(), ChainStatus::Intact { .. }), "{step:?}");
        caught_up(a.rx.pass());
        let al = names(&lines(&a.store, &conv, &s.msg_id));
        assert_eq!(al, ["SENT", "POSTED", "RECEIVED", "STORED"], "{step:?}");
    }
    // A restart re-reads nothing: the cursor survived.
    let cursor = b.as_ref().unwrap().rx.cursor();
    drop(b.take());
    let bb = agent(&e, "b", B, INST_B, &id);
    assert_eq!(bb.rx.cursor(), cursor);
    assert_eq!(caught_up(bb.rx.pass()), 0);
}

#[test]
fn receive_hub_restart_after_processing_without_cursor_adds_no_second_stage() {
    let _serial = serial();
    let (e, a, b) = pair();
    let s = a.sender.send(B, INST_B, Some("r-1"), "once", None).unwrap();
    caught_up(b.rx.pass());
    // Lose the cursor (as if B died before writing it): everything is re-read.
    b.store.write_state(CURSOR_STATE, "0").unwrap();
    caught_up(b.rx.pass());
    assert_eq!(names(&lines(&b.store, "r-1", &s.msg_id)), ["RECEIVED", "STORED"]);
    // A gets the stage records twice and keeps one of each (forward-only).
    caught_up(a.rx.pass());
    assert_eq!(names(&lines(&a.store, "r-1", &s.msg_id)), ["SENT", "POSTED", "RECEIVED", "STORED"]);
    drop(e);
}

#[test]
fn receive_hub_same_id_with_another_body_is_a_conflict_and_never_stored() {
    let _serial = serial();
    let (e, _a, b) = pair();
    let ka = e.identity("a").unwrap();
    post_signed(&e, &ka, &msg(&ka, A, INST_B, "c-1", "dup", "first body"));
    post_signed(&e, &ka, &msg(&ka, A, INST_B, "c-1", "dup", "second body"));
    assert_eq!(caught_up(b.rx.pass()), 2);
    let bl = lines(&b.store, "c-1", "dup");
    assert_eq!(names(&bl), ["RECEIVED", "STORED", "RECEIVED", "CONFLICT"]);
    let (_, kept) = b.store.read_record("c-1", bl[1].seq, "dup").unwrap();
    assert!(String::from_utf8_lossy(&kept).contains("first body"));
    assert!(b.store.attention_entries().join("\n").contains("second body with another digest"));
}

#[test]
fn receive_hub_second_sidecar_on_the_same_store_is_refused() {
    let _serial = serial();
    let (e, _a, _b) = pair();
    match Store::open(&e.store_root(), B) {
        Err(StoreError::Locked) => {}
        Err(other) => panic!("expected Locked, got {other}"),
        Ok(_) => panic!("a second sidecar opened the same store"),
    }
}

#[test]
fn receive_hub_wrong_hub_id_reads_nothing_and_says_so() {
    let _serial = serial();
    let (e, a, _b) = pair();
    a.sender.send(B, INST_B, Some("p-1"), "pinned", None).unwrap();
    // A receiver for B declared against another hub id, on a fresh store.
    let store = Arc::new(Store::open(&e.store_root(), "agent-b2").unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: B.into(),
                instance: INST_B.into(),
                identity: e.identity("b").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: "not-the-hub-id".into(),
                queue_path: e.root().join("outbox-b2.sqlite"),
                timeout: Duration::from_secs(15),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: B.into(),
            instance: INST_B.into(),
            hub: e.hub_addr(),
            declared_hub_id: "not-the-hub-id".into(),
            roster: roster(&e),
            timeout: Duration::from_secs(15),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender,
        Box::new(BuiltIn::new(roster(&e))),
    )
    .unwrap();
    assert!(matches!(rx.pass(), PassResult::Held(_)));
    assert!(matches!(rx.pass(), PassResult::Held(_)));
    assert_eq!(rx.cursor(), 0);
    assert!(store.conversations().is_empty());
    let att = store.attention_entries();
    assert_eq!(att.iter().filter(|l| l.contains("hub-id-mismatch")).count(), 1, "{att:?}");
}

#[test]
fn receive_hub_forged_or_stray_stage_records_are_ignored_and_shown() {
    let _serial = serial();
    let (e, a, b) = pair();
    let s = a.sender.send(B, INST_B, Some("s-1"), "staged", None).unwrap();
    let topic_a = inbox_topic(&e.hub_id().unwrap(), A);
    let host = e.identity("host").unwrap();
    let kb = e.identity("b").unwrap();
    let stage = |from: &Identity, id: &str, st: &str| Stage {
        from: from.fingerprint().to_owned(),
        from_agent: B.into(),
        to_agent: A.into(),
        conv: "s-1".into(),
        msg_id: id.into(),
        stage: st.into(),
        ts_ms: 42,
        chain_head: "00".into(),
    };
    // 1 Signed by the host key (not rostered): claims STORED for a real message.
    let s1 = wire::sign_stage(&host, &topic_a, 42, &stage(&host, &s.msg_id, "STORED"));
    raw_post(&e, &topic_a, wire::STAGE_TYPE, &s1.payload, 42, &host, meta_of(&s1));
    // 2 Correctly signed by B, but for a message A never sent.
    let s2 = wire::sign_stage(&kb, &topic_a, 43, &stage(&kb, "never-sent", "RECEIVED"));
    raw_post(&e, &topic_a, wire::STAGE_TYPE, &s2.payload, 43, &kb, meta_of(&s2));
    // 3 No signature in the metadata.
    raw_post(&e, &topic_a, wire::STAGE_TYPE, &stage(&kb, &s.msg_id, "STORED").payload_bytes(), 44, &kb, BTreeMap::new());

    assert_eq!(caught_up(a.rx.pass()), 3);
    assert_eq!(names(&lines(&a.store, "s-1", &s.msg_id)), ["SENT", "POSTED"]);
    let att = a.store.attention_entries().join("\n");
    assert!(att.contains("not in the roster"), "{att}");
    assert!(att.contains("was not sent by this agent"), "{att}");
    assert!(att.contains("no signature"), "{att}");

    // A genuine STORED arriving before RECEIVED: RECEIVED afterwards is behind it and dropped.
    let st = wire::sign_stage(&kb, &topic_a, 50, &stage(&kb, &s.msg_id, "STORED"));
    raw_post(&e, &topic_a, wire::STAGE_TYPE, &st.payload, 50, &kb, meta_of(&st));
    let rc = wire::sign_stage(&kb, &topic_a, 51, &stage(&kb, &s.msg_id, "RECEIVED"));
    raw_post(&e, &topic_a, wire::STAGE_TYPE, &rc.payload, 51, &kb, meta_of(&rc));
    caught_up(a.rx.pass());
    assert_eq!(names(&lines(&a.store, "s-1", &s.msg_id)), ["SENT", "POSTED", "STORED"]);
    drop(b);
}

#[test]
fn receive_hub_concurrent_writers_never_fork_a_chain() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), A).unwrap());
    let threads: Vec<_> = (0..8)
        .map(|t| {
            let s = store.clone();
            std::thread::spawn(move || {
                for i in 0..25 {
                    s.append_event("cc-1", "WAITING", &format!("m-{t}-{i}"), "x", "d").unwrap();
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    match store.verify_chain("cc-1").unwrap() {
        ChainStatus::Intact { len, .. } => assert_eq!(len, 200),
        other => panic!("chain forked: {other:?}"),
    }
}
