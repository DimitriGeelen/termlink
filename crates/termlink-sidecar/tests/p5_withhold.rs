//! P-5 (T-3695, arc-011; ARC4 7.1.a, CR-4.11; 06 AL-12 5.12.c): a session that is not main
//! is withheld ROLE mail only. Mail of a conversation bound to that exact session (R-62: the
//! session sent or replied in it, or had a message of it handed over) reaches it whether or
//! not it is main. E-161 (the B-25 withhold part) and its control: the built pull (withhold
//! all) fails the exact-instance assertion. Hermetic: testkit estate, no hub.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use termlink_session::agent_identity::Identity;
use termlink_session::main_lease::{Folded, Holder, Row, Timing, View, fold, view};
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::deliver::{Deliverer, Verdict, WITHHOLD_ALL_FLAG};
use termlink_sidecar::lease::Shared;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::{Store, now_ms};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::{Msg, Roster};

const B: &str = "agent-b";
const MAIN: &str = "S1-main";
const OTHER: &str = "S2-other";

struct Rig {
    e: Estate,
    store: Arc<Store>,
    d: Deliverer,
    ka: Identity,
    lease: Arc<Shared>,
}

fn held_by(session: &str, agent: &str, generation: u64) -> View {
    let h = Holder { session: session.into(), agent: agent.into(), host: "h".into(), pid: 1 };
    let mut f: Folded = fold(&[]);
    f.generation = generation;
    let row = Row { claim_id: "c".into(), claimer: h.claimer(), claimed_at: 0, claimed_until: i64::MAX };
    view("p", &f, Some(&row), 10, Timing::default())
}

fn rig() -> Rig {
    let e = Estate::create_without_hub().unwrap();
    let ka = e.identity("a").unwrap();
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let mut roster = Roster::new();
    roster.add(ka.verifying_key());
    let d = Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(roster)), None);
    let lease = Shared::new(Some("p".into()), 15_000);
    lease.set(now_ms(), Ok(held_by(MAIN, B, 7)));
    Rig { e, store, d, ka, lease }
}

fn put(r: &Rig, conv: &str, id: &str) {
    let m = Msg {
        from: r.ka.fingerprint().to_owned(),
        from_agent: "agent-a".into(),
        from_instance: "inst-a".into(),
        to_agent: B.into(),
        to_instance: "inst-b".into(),
        conv: conv.into(),
        msg_id: id.into(),
        seq: 0,
        priority: "0".into(),
        ts_ms: 1,
        body: format!("body of {id}"),
    };
    r.store.append_event(conv, "RECEIVED", id, &m.from, &m.digest()).unwrap();
    r.store.put(&m.from, conv, id, &m.payload_bytes()).unwrap();
}

fn pull(r: &Rig, session: &str, tool: &str) -> Value {
    let req = json!({"op": "pull", "session": session, "hook": "post-tool-use", "tool_use_id": tool});
    api::dispatch_leased(&req, Scope::Send, &r.store, B, None, Some(&r.d), Some(&r.lease))
}

fn ids(v: &Value) -> Vec<String> {
    v["msg_ids"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

fn write_transcript(dir: &Path, name: &str, recs: &[Value]) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, recs.iter().map(|r| format!("{r}\n")).collect::<String>()).unwrap();
    p
}

/// E-161.2-4: two sessions of one agent, lease on S1; S2's pending set holds one role message
/// and one exact-instance message. S2 pulls twice in one generation: it receives the
/// exact-instance message only, and the not-main line once. Main still receives the role mail.
#[test]
fn e161_non_main_session_gets_exact_instance_mail_only_and_the_line_once() {
    let r = rig();
    put(&r, "c-role", "m-role");
    put(&r, "c-mine", "m-mine");
    assert!(r.d.bind("c-mine", OTHER).unwrap(), "first binding");

    let first = pull(&r, OTHER, "toolu_1");
    assert_eq!(ids(&first), ["m-mine"], "S2 is handed its own conversation's mail and no role mail: {first}");
    assert_eq!(first["stage"], "CLAIMED");
    assert!(first.get("not_main").is_none(), "a frame does not spend the once-per-generation line: {first}");

    let second = pull(&r, OTHER, "toolu_2");
    assert_eq!(second["why"], "not-main", "{second}");
    assert_eq!(second["notice"], true, "told once: {second}");
    assert!(second["not_main"].as_str().unwrap().contains("role mail is left unread for main"), "{second}");
    let third = pull(&r, OTHER, "toolu_3");
    assert_eq!(third["why"], "not-main");
    assert_eq!(third["notice"], false, "never twice in one generation: {third}");

    let main = pull(&r, MAIN, "toolu_4");
    assert_eq!(ids(&main), ["m-role"], "the role message stayed stored for main: {main}");
}

/// Exact-instance mail is that instance's only: mail bound to S3 is not handed to S2 (not main).
/// Main, as built, still reads every message, so nothing bound to an ended session is stranded.
#[test]
fn p5_mail_bound_to_another_session_is_not_handed_to_a_non_main_session() {
    let r = rig();
    put(&r, "c-s3", "m-s3");
    r.d.bind("c-s3", "S3-gone").unwrap();
    let v = pull(&r, OTHER, "toolu_1");
    assert_eq!(v["why"], "not-main", "{v}");
    let main = pull(&r, MAIN, "toolu_2");
    assert_eq!(ids(&main), ["m-s3"], "{main}");
}

/// Mail to an exact instance reaches it whether or not it is main: the same session, once main,
/// still receives its bound mail (and the role mail with it, as built).
#[test]
fn p5_exact_instance_mail_reaches_its_session_when_it_is_main_too() {
    let r = rig();
    put(&r, "c-main-own", "m-own");
    r.d.bind("c-main-own", MAIN).unwrap();
    let v = pull(&r, MAIN, "toolu_1");
    assert_eq!(ids(&v), ["m-own"], "{v}");
}

/// Rollback in one step (and the E-161.5 control by flag): with `<store>/withhold-all` the
/// pull is the built all-mail withhold, so the exact-instance assertion fails; removing the
/// flag restores P-5. The waiting-mail signal counts the bound message only while it is withheld.
#[test]
fn p5_withhold_all_flag_restores_the_built_pull_and_counts_the_held_mail() {
    let r = rig();
    put(&r, "c-mine", "m-mine");
    r.d.bind("c-mine", OTHER).unwrap();
    assert!(!r.d.waiting_mail(now_ms())[0].role, "bound mail is exact-instance mail (FCW-64)");

    std::fs::write(r.store.dir().join(WITHHOLD_ALL_FLAG), "").unwrap();
    assert!(r.d.withhold_all());
    let held = pull(&r, OTHER, "toolu_1");
    assert_eq!(held["why"], "not-main", "rolled back: all mail withheld from a non-main session: {held}");
    assert_eq!(held["notice"], true);
    assert!(r.d.waiting_mail(now_ms())[0].role, "withheld mail is never silent: it counts as waiting");

    std::fs::remove_file(r.store.dir().join(WITHHOLD_ALL_FLAG)).unwrap();
    let v = pull(&r, OTHER, "toolu_2");
    assert_eq!(ids(&v), ["m-mine"], "{v}");
}

/// The flag fails toward holding: anything at the flag path (even a directory) counts as set.
#[test]
fn p5_withhold_all_flag_anything_present_counts_as_set() {
    let r = rig();
    assert!(!r.d.withhold_all(), "absent flag: P-5 behaviour");
    std::fs::create_dir_all(r.store.dir().join(WITHHOLD_ALL_FLAG)).unwrap();
    assert!(r.d.withhold_all(), "anything present at the flag path is set");
}

/// E-161 with the delivery switch off: a non-main pull refuses `delivery-off`, as main's does,
/// and hands out nothing (the switch is read before the bound pull).
#[test]
fn e161_delivery_off_refuses_the_non_main_pull_too() {
    let r = rig();
    put(&r, "c-mine", "m-mine");
    r.d.bind("c-mine", OTHER).unwrap();
    let off = api::dispatch(&json!({"op": "delivery", "state": "off", "why": "p5 test"}), Scope::Admin, &r.store, B, None);
    assert!(off.get("code").is_none(), "{off}");
    let v = pull(&r, OTHER, "toolu_1");
    assert_eq!(v["why"], "delivery-off", "{v}");
    assert!(v.get("frame").is_none());
}

/// Binding: first binding wins; the `claim` op's pseudo session binds nothing; a hand-over to
/// a session binds the conversation to it (R-62: bound at creation, never re-bound here).
#[test]
fn p5_binding_first_wins_and_a_hand_over_binds() {
    let r = rig();
    assert!(r.d.bind("c-x", "S-a").unwrap());
    assert!(!r.d.bind("c-x", "S-b").unwrap(), "first binding wins");
    assert_eq!(r.d.bound_session("c-x").as_deref(), Some("S-a"));
    assert!(!r.d.bind("c-y", "claim-op").unwrap());
    assert_eq!(r.d.bound_session("c-y"), None);

    // Hand-over to MAIN (role mail) binds its conversation to MAIN; a later message of that
    // conversation then reaches MAIN even after MAIN has lost the lease.
    put(&r, "c-h", "m-h1");
    let f = pull(&r, MAIN, "toolu_A");
    assert_eq!(ids(&f), ["m-h1"]);
    let t = write_transcript(
        &r.e.root().join("transcripts"),
        "main.jsonl",
        &[
            json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "tool_use", "id": "toolu_A", "name": "Bash", "input": {}}]}}),
            json!({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_A", "content": "ok"}]}}),
            json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [f["frame"].as_str().unwrap()], "hookName": "PostToolUse:Bash", "toolUseID": "toolu_A", "hookEvent": "PostToolUse"}}),
            json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "thanks"}]}}),
        ],
    );
    r.d.register_session(MAIN, &t).unwrap();
    assert_eq!(r.d.finalize(f["nonce"].as_str().unwrap(), false).unwrap(), Verdict::HandedOver);
    assert_eq!(r.d.bound_session("c-h").as_deref(), Some(MAIN));

    r.lease.set(now_ms(), Ok(held_by(OTHER, B, 8)));
    put(&r, "c-h", "m-h2");
    put(&r, "c-role2", "m-role2");
    let v = pull(&r, MAIN, "toolu_B");
    assert_eq!(ids(&v), ["m-h2"], "the former main still gets its conversation, not the role mail: {v}");
}

/// A session that sends in a conversation is bound to it (the send path, hub unreachable: the
/// message is queued, stage SENT, and the binding is written).
#[test]
fn p5_a_send_from_a_session_binds_its_conversation() {
    let r = rig();
    let sender = Sender::new(
        SendConfig {
            agent: B.into(),
            instance: "inst-b".into(),
            identity: r.e.identity("b").unwrap(),
            hub: r.e.hub_addr(),
            declared_hub_id: "hub-x".into(),
            queue_path: r.e.root().join("outbox-b.sqlite"),
            timeout: Duration::from_secs(2),
            pump_interval: None,
        },
        r.store.clone(),
    )
    .unwrap();
    let req = json!({"op": "send", "to": "agent-a", "to_instance": "inst-a", "conv": "c-sent", "body": "hi", "session": OTHER});
    let v = api::dispatch_leased(&req, Scope::Send, &r.store, B, Some(&sender), Some(&r.d), Some(&r.lease));
    assert_eq!(v["stage"], "SENT", "{v}");
    assert_eq!(r.d.bound_session("c-sent").as_deref(), Some(OTHER));
    // The reply in that conversation reaches the non-main sender.
    put(&r, "c-sent", "m-answer");
    let p = pull(&r, OTHER, "toolu_1");
    assert_eq!(ids(&p), ["m-answer"], "{p}");
    // Without a session nothing is bound.
    let req = json!({"op": "send", "to": "agent-a", "to_instance": "inst-a", "conv": "c-anon", "body": "hi"});
    let v = api::dispatch_leased(&req, Scope::Send, &r.store, B, Some(&sender), Some(&r.d), Some(&r.lease));
    assert_eq!(v["stage"], "SENT", "{v}");
    assert_eq!(r.d.bound_session("c-anon"), None);
}

/// NID: with no lease, a vacant one, or main itself pulling, the pull is unchanged.
#[test]
fn p5_main_and_unknown_lease_pull_as_built() {
    let r = rig();
    put(&r, "c-role", "m-role");
    let none_lease = api::dispatch_leased(
        &json!({"op": "pull", "session": OTHER, "hook": "post-tool-use", "tool_use_id": "t"}),
        Scope::Send,
        &r.store,
        B,
        None,
        Some(&r.d),
        None,
    );
    assert_eq!(ids(&none_lease), ["m-role"], "no lease view: delivery as before: {none_lease}");
}
