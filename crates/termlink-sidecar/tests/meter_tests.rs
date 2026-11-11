//! B-27 hermetic tests: the activity meter (AL-1 intake, AL-3 S1/S2/S3), the
//! conversation signal (AL-4) and status `activity` (E-15..E-19, E-9 meter half).
//! Simulated clocks, a fake S1 source and a fake ledger; no hub, no PTY.
//! Each test names the negative control that turns it red.

use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use termlink_sidecar::meter::*;

const A: &str = "agent-b";
const T: u64 = 30_000;

#[derive(Default)]
struct FakeS1(Mutex<HashMap<String, Result<S1Reply, String>>>);
struct S1Box(Arc<FakeS1>);
impl S1Source for S1Box {
    fn read(&self, s: &str) -> Result<S1Reply, String> {
        self.0.0.lock().unwrap().get(s).cloned().unwrap_or(Err("unbound".into()))
    }
}

struct FakeLedger(Arc<Mutex<Result<HashSet<String>, String>>>);
impl Ledger for FakeLedger {
    fn owed(&self, conv: &str, _: u64) -> Result<bool, String> {
        self.0.lock().unwrap().as_ref().map(|s| s.contains(conv)).map_err(Clone::clone)
    }
}

struct Rig {
    m: Meter,
    now: Arc<AtomicU64>,
    s1: Arc<FakeS1>,
    owed: Arc<Mutex<Result<HashSet<String>, String>>>,
}

fn rig() -> Rig {
    let now = Arc::new(AtomicU64::new(1_000_000));
    let s1 = Arc::new(FakeS1::default());
    let owed = Arc::new(Mutex::new(Ok(HashSet::new())));
    let (n1, n2) = (now.clone(), now.clone());
    let m = Meter::new(A, Box::new(S1Box(s1.clone())), Box::new(FakeLedger(owed.clone())))
        .with_clocks(Arc::new(move || n1.load(Ordering::SeqCst)), Arc::new(move || n2.load(Ordering::SeqCst)));
    Rig { m, now, s1, owed }
}

impl Rig {
    fn at(&self) -> u64 {
        self.now.load(Ordering::SeqCst)
    }
    fn adv(&self, ms: u64) {
        self.now.fetch_add(ms, Ordering::SeqCst);
    }
    fn rep(&self, v: Value) -> Intake {
        self.m.intake(v.to_string().as_bytes())
    }
    fn ev(&self, sess: &str, event: &str, at_ms: u64, extra: Value) -> Intake {
        let mut v = json!({"schema": SCHEMA, "harness": "fake", "agent_id": A, "session_id": sess, "event": event, "at_ms": at_ms});
        for (k, x) in extra.as_object().unwrap() {
            v[k] = x.clone();
        }
        self.rep(v)
    }
    /// S1 counters as the session reports them: `last_mono` per source.
    fn set_s1(&self, sess: &str, client: Option<u64>, notify: Option<u64>, inject: Option<u64>) {
        let mut last = BTreeMap::new();
        last.insert("client".into(), client);
        last.insert("client_auto".into(), None);
        last.insert("notify".into(), notify);
        last.insert("inject".into(), inject);
        let r = S1Reply { boot_id: termlink_session::input_meter::boot_id(), last, foreground: Value::Null };
        self.s1.0.lock().unwrap().insert(sess.into(), Ok(r));
    }
    fn act(&self, sess: &str) -> Value {
        self.m.tick();
        self.m.status_view()["sessions"][sess].clone()
    }
}

// ---- E-15 (sidecar half): S1 moves on client input only -------------------------------------

/// Control: a meter that took `max(client, notify, inject)` as S1 moves s1_at on the doorbell.
#[test]
fn e15_s1_moves_on_client_input_only() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    let t0 = r.at();
    r.set_s1("s", None, Some(t0), Some(t0));
    let a = r.act("s");
    assert_eq!(a["s1"], "known");
    assert_eq!(a["s1_at"], Value::Null, "notify/inject keystrokes are not S1: {a}");
    assert_ne!(a["source"], "s1");
    r.adv(1000);
    r.set_s1("s", Some(r.at()), Some(t0), Some(t0));
    let a = r.act("s");
    assert_eq!(a["s1_at"], json!(r.at()));
    assert_eq!(a["source"], "s1");
}

/// FCW-12 / FCW-11: a failed read or another boot is unknown, never silence;
/// no watchdog. Control: treat Err as Known{client: None} and the watchdog reads available.
#[test]
fn fcw12_s1_unreadable_or_other_boot_is_unknown() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    let a = r.act("s");
    assert!(a["s1"].as_str().unwrap().starts_with("unknown"), "{a}");
    assert_eq!(a["s1_at"], "unknown");
    assert_eq!(a["watchdog"], "unavailable: S1 unknown");
    let mut last = BTreeMap::new();
    last.insert("client".to_string(), Some(5));
    r.s1.0.lock().unwrap().insert("s".into(), Ok(S1Reply { boot_id: Some("another-boot".into()), last, foreground: Value::Null }));
    let a = r.act("s");
    assert!(a["s1"].as_str().unwrap().contains("another boot"), "{a}");
    r.set_s1("s", None, None, None);
    assert_eq!(r.act("s")["watchdog"], "available");
}

// ---- E-16: S2 and the strict intake -----------------------------------------------------------

/// Every valid event moves s2_at. Control: drop the S2 stamp in intake and each step stays put.
#[test]
fn e16_each_valid_event_moves_s2() {
    let r = rig();
    let evs: Vec<(&str, Value)> = vec![
        ("session_start", json!({"interactive": true})),
        ("prompt_submit", json!({"origin": "human", "basis": "inferred"})),
        ("tool_start", json!({"tool": "Bash"})),
        ("tool_end", json!({"tool": "Bash"})),
        ("turn_stop", json!({})),
        ("ready", json!({"state": "READY"})),
        ("evidence", json!({"nonce": "n1", "record_ref": "r"})),
        ("complete", json!({"turn_boundary": true})),
        ("session_end", json!({})),
    ];
    r.set_s1("s", None, None, None);
    for (i, (e, x)) in evs.into_iter().enumerate() {
        r.adv(1000);
        assert_eq!(r.ev("s", e, 10 + i as u64, x), Intake::Applied, "{e}");
        let a = r.act("s");
        assert_eq!(a["s2_at"], json!(r.at()), "{e} must move S2: {a}");
    }
    assert_eq!(r.m.counters().unparseable, 0);
}

/// FCW-13: no report and no transcript -> S2 unknown, watchdog unavailable.
#[test]
fn fcw13_s2_unknown_without_reports() {
    let r = rig();
    r.m.on_handed_over("s", "c"); // unknown session: not even created
    r.m.legacy_hook("s", "post-tool-use", None);
    r.set_s1("s", None, None, None);
    // A built hook call is a report (D-2): S2 known.
    assert_eq!(r.act("s")["watchdog"], "available");
    let r = rig();
    // A session seen only through S3 does not exist for the meter.
    r.m.on_sent(Some("x"), "c");
    assert_eq!(r.m.counters().s3_unattributed, 1);
    assert!(r.m.status_view()["sessions"].get("x").is_none());
}

fn malformed() -> Vec<Vec<u8>> {
    let ok = |f: &dyn Fn(&mut Value)| {
        let mut v = json!({"schema": SCHEMA, "agent_id": A, "session_id": "s", "event": "ready", "state": "READY", "at_ms": 5});
        f(&mut v);
        v.to_string().into_bytes()
    };
    vec![
        ok(&|v| {
            v.as_object_mut().unwrap().remove("state");
        }), // control: a mutant defaulting a missing state to READY
        ok(&|v| v["state"] = json!("ready")),
        ok(&|v| v["schema"] = json!("termlink.adapter/1")),
        ok(&|v| v["schema"] = json!("TERMLINK.ADAPTER/2")),
        ok(&|v| v["event"] = json!("Ready")),
        ok(&|v| v["agent_id"] = json!("agent-x")),
        ok(&|v| v["session_id"] = json!("a b")),
        ok(&|v| v["session_id"] = json!("x".repeat(129))),
        ok(&|v| v["at_ms"] = json!(-1)),
        ok(&|v| v["at_ms"] = json!("5")),
        ok(&|v| v["at_ms"] = json!(9_007_199_254_740_992u64)),
        ok(&|v| v["pad"] = json!("x".repeat(1 << 20))),
        br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"BUSY","state":"READY","at_ms":5}"#.to_vec(),
        br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"READY","at_ms":5.0}"#.to_vec(),
        br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"READY","at_ms":5e0}"#.to_vec(),
        br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"READY","at_ms":-0}"#.to_vec(),
        br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"READY","at_ms":5,"x":[[[[[[[[[1]]]]]]]]]}"#.to_vec(),
        b"{\"schema\":\"termlink.adapter/2\",\"agent_id\":\"agent-b\",\"session_id\":\"s\",\"event\":\"ready\",\"state\":\"READ\xffY\",\"at_ms\":5}".to_vec(),
        b"[1,2]".to_vec(),
        b"".to_vec(),
        b"not json".to_vec(),
        br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"READY","at_ms":5} trailing"#.to_vec(),
    ]
}

/// Unparseable reports count and never read as READY, human or S2.
#[test]
fn e16_malformed_reports_are_counted_and_change_nothing() {
    let r = rig();
    let bad = malformed();
    for b in &bad {
        assert!(matches!(r.m.intake(b), Intake::Ignored(_)), "accepted: {}", String::from_utf8_lossy(&b[..b.len().min(200)]));
    }
    assert_eq!(r.m.counters().unparseable, bad.len() as u64);
    assert!(r.m.ready("s").is_none(), "no malformed report made a READY record");
    assert!(r.m.status_view()["sessions"].get("s").is_none(), "no session, no S2");
    assert!(r.m.counters().reasons.len() <= 16);
    // A well-formed one at depth 8 is fine.
    let deep = br#"{"schema":"termlink.adapter/2","agent_id":"agent-b","session_id":"s","event":"ready","state":"READY","at_ms":5,"x":[[[[[[[1]]]]]]]}"#;
    assert_eq!(r.m.intake(deep), Intake::Applied);
}

/// E-16.3: 10^5 fuzzed objects (byte flips, truncations, splices of the valid
/// report); none crashes, none applied reads as READY unless it is a valid one.
#[test]
fn e16_fuzz_one_hundred_thousand_reports() {
    let r = rig();
    let base = json!({"schema": SCHEMA, "agent_id": A, "session_id": "f", "event": "ready", "state": "BUSY", "at_ms": 1}).to_string().into_bytes();
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut rnd = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    let mut applied = 0u64;
    for i in 0..100_000u64 {
        let mut b = base.clone();
        match i % 4 {
            0 => {
                let p = (rnd() as usize) % b.len();
                b[p] = rnd() as u8;
            }
            1 => b.truncate((rnd() as usize) % b.len()),
            2 => {
                let p = (rnd() as usize) % b.len();
                let pool = b"{}[]\",:-0e\\\xff";
                b.insert(p, pool[(rnd() as usize) % pool.len()]);
            }
            _ => {
                let p = (rnd() as usize) % b.len();
                let q = (rnd() as usize) % b.len();
                b.swap(p, q);
            }
        }
        let before = r.m.counters().unparseable;
        match r.m.intake(&b) {
            Intake::Ignored(_) => assert!(r.m.counters().unparseable + r.m.counters().session_table_full > before),
            _ => applied += 1,
        }
    }
    let c = r.m.counters();
    assert_eq!(c.unparseable + c.session_table_full + applied, 100_000, "every object is counted or applied: {c:?}");
    assert!(c.unparseable > 50_000, "{c:?}");
    // Whatever was applied came from a report that still said BUSY or a valid state.
    if let Some(rd) = r.m.ready("f") {
        assert!(["BUSY", "READY", "NOT_RUNNING"].contains(&rd.state.as_str()));
    }
}

/// FCW-3: an older `ready` is dropped; a tie keeps BUSY. Control: drop the
/// at_ms comparison and the stale READY wins.
#[test]
fn fcw3_ready_ordering() {
    let r = rig();
    assert_eq!(r.ev("s", "ready", 100, json!({"state": "BUSY"})), Intake::Applied);
    assert_eq!(r.ev("s", "ready", 99, json!({"state": "READY"})), Intake::Dropped("ready-reordered"));
    assert_eq!(r.m.ready("s").unwrap().state, "BUSY");
    assert_eq!(r.ev("s", "ready", 100, json!({"state": "READY"})), Intake::Dropped("tie keeps BUSY"));
    assert_eq!(r.m.ready("s").unwrap().state, "BUSY");
    assert_eq!(r.ev("s", "ready", 101, json!({"state": "READY"})), Intake::Applied);
    let rd = r.m.ready("s").unwrap();
    assert_eq!((rd.state.as_str(), rd.seq, rd.epoch), ("READY", 2, r.m.epoch()));
    assert_eq!(r.m.counters().ready_reordered, 1);
}

/// FCW-4/FCW-5: the receiver only downgrades an origin. Controls: accept
/// "Human" case-insensitively; skip the PTY-counter override.
#[test]
fn fcw4_fcw5_prompt_origin_checks() {
    let r = rig();
    let origin = |r: &Rig| r.m.last_prompt("s").map(|p| (p.origin, p.basis));
    r.ev("s", "prompt_submit", 1, json!({"origin": "Human", "basis": "inferred"}));
    assert_eq!(origin(&r), Some((Origin::Unknown, "absent".into())));
    r.ev("s", "prompt_submit", 2, json!({"origin": "human", "basis": "doorbell"}));
    assert_eq!(origin(&r), Some((Origin::Unknown, "absent".into())), "basis not allowed for human");
    assert_eq!(r.m.counters().origin_inconsistent, 2);
    // A line the receiver typed this tick, reported human: automated.
    r.m.note_typed("s", &["m-1".into()]);
    r.ev("s", "prompt_submit", 3, json!({"origin": "human", "basis": "inferred", "doorbell_ids": ["m-1"]}));
    assert_eq!(origin(&r), Some((Origin::Automated, "doorbell".into())));
    // human/pty-input with the last PTY input from notify or inject: automated.
    let t = r.at();
    r.set_s1("s", Some(t - 50), Some(t - 10), None);
    r.ev("s", "prompt_submit", 4, json!({"origin": "human", "basis": "pty-input"}));
    assert_eq!(origin(&r), Some((Origin::Automated, "pty-input".into())));
    r.set_s1("s", Some(t - 50), None, Some(t - 5));
    r.ev("s", "prompt_submit", 5, json!({"origin": "human", "basis": "pty-input"}));
    assert_eq!(origin(&r), Some((Origin::Automated, "pty-input".into())));
    assert_eq!(r.m.counters().origin_overridden, 3);
    // Last input from the client: human stays human.
    r.set_s1("s", Some(t - 1), Some(t - 10), None);
    r.ev("s", "prompt_submit", 6, json!({"origin": "human", "basis": "pty-input"}));
    assert_eq!(origin(&r), Some((Origin::Human, "pty-input".into())));
    // Counters unreadable: cannot confirm pty-input -> inferred, never upgraded.
    r.s1.0.lock().unwrap().clear();
    r.ev("s", "prompt_submit", 7, json!({"origin": "human", "basis": "pty-input"}));
    assert_eq!(origin(&r), Some((Origin::Human, "inferred".into())));
    r.ev("s", "prompt_submit", 8, json!({"origin": "automated", "basis": "harness-mark"}));
    assert_eq!(origin(&r), Some((Origin::Automated, "harness-mark".into())));
}

/// FCW-7: 64 live sessions; the 65th is refused.
#[test]
fn fcw7_session_table_is_capped() {
    let r = rig();
    for i in 0..MAX_SESSIONS {
        assert_eq!(r.ev(&format!("s{i}"), "turn_stop", 1, json!({})), Intake::Applied);
    }
    assert_eq!(r.ev("one-more", "turn_stop", 1, json!({})), Intake::Ignored("session-table-full".into()));
    assert_eq!(r.m.counters().session_table_full, 1);
    // An ended session frees a slot.
    r.ev("s0", "session_end", 2, json!({}));
    assert_eq!(r.ev("one-more", "turn_stop", 1, json!({})), Intake::Applied);
}

/// FCW-8 and S2 (b): a transcript inside the project directory moves S2 when
/// it grows, not otherwise; a symlink or a path outside is refused.
/// Control: count every tick as growth and the "no growth" assert goes red.
#[test]
fn fcw8_transcript_growth_and_refusals() {
    let dir = std::env::current_dir().unwrap().join("target").join(format!("t3687-meter-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let root = dir.join("projects");
    std::fs::create_dir_all(root.join("p")).unwrap();
    let outside = dir.join("outside.jsonl");
    std::fs::write(&outside, "x").unwrap();
    let tr = root.join("p").join("s.jsonl");
    std::fs::write(&tr, "{}\n").unwrap();
    let link = root.join("p").join("link.jsonl");
    std::os::unix::fs::symlink(&tr, &link).unwrap();
    let mut r = rig();
    r.m = r.m.with_transcript_root(root.clone());
    for bad in [&link, &outside] {
        r.ev("b", "session_start", 1, json!({"interactive": true, "transcript_path": bad.to_string_lossy()}));
    }
    assert_eq!(r.m.counters().transcript_refused, 2);
    r.ev("s", "session_start", 1, json!({"interactive": true, "transcript_path": tr.to_string_lossy()}));
    r.set_s1("s", None, None, None);
    let start = r.at();
    r.act("s"); // first read of size/mtime
    r.adv(T);
    let a = r.act("s");
    assert_eq!(a["s2_at"], json!(start), "no growth, no S2: {a}");
    std::fs::write(&tr, "{}\n{\"more\":1}\n").unwrap();
    r.adv(T);
    let a = r.act("s");
    assert_eq!(a["s2_at"], json!(r.at()), "growth moves S2: {a}");
    assert_eq!(a["transcript"], true);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- E-17 / E-9: S3 ----------------------------------------------------------------------------

/// S3 moves on an own send from the session and a hand-over to it, not on
/// arrivals (nothing calls the meter on STORED). Control: count unattributed
/// sends against every session and the "x" assert goes red.
#[test]
fn e17_s3_moves_on_send_and_handover_only() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    r.ev("x", "session_start", 1, json!({"interactive": true}));
    r.set_s1("s", None, None, None);
    r.adv(1000);
    r.m.on_sent(None, "c1"); // a send that names no session
    assert_eq!(r.act("s")["s3_at"], Value::Null);
    r.m.on_sent(Some("s"), "c1");
    let t1 = r.at();
    assert_eq!(r.act("s")["s3_at"], json!(t1));
    assert_eq!(r.act("x")["s3_at"], Value::Null, "another session's send is not this one's S3");
    r.adv(1000);
    r.m.on_handed_over("s", "c2");
    let a = r.act("s");
    assert_eq!(a["s3_at"], json!(r.at()));
    assert_eq!(a["source"], "s3");
}

// ---- AL-3: last activity and the watchdog availability -----------------------------------------

#[test]
fn al3_last_is_the_max_of_known_sources() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    let t_s2 = r.at();
    r.adv(5000);
    r.set_s1("s", Some(r.at()), None, None);
    let a = r.act("s");
    assert_eq!((a["last"].clone(), a["source"].clone()), (json!(r.at()), json!("s1")));
    // S1 becomes unreadable: it no longer contributes, S2 is the last known.
    r.s1.0.lock().unwrap().clear();
    let a = r.act("s");
    assert_eq!((a["last"].clone(), a["source"].clone()), (json!(t_s2), json!("s2")));
    assert_eq!(a["watchdog"], "unavailable: S1 unknown");
}

// ---- E-18: conversation signal -----------------------------------------------------------------

fn cv(r: &Rig, s: &str) -> Conversation {
    r.m.conversation(s, r.at())
}

/// operator from a human/unknown prompt until 120 s after its turn stopped;
/// automated never. Rows are identical for every READY value (READY is not
/// an input). Controls: read READY into the signal; treat unknown as automated.
#[test]
fn e18_operator_window_and_independence_from_ready() {
    for ready in ["READY", "BUSY", "NOT_RUNNING", "none"] {
        for (origin, basis, expect_op) in [("human", "inferred", true), ("unknown", "absent", true), ("automated", "doorbell", false)] {
            let r = rig();
            if ready != "none" {
                r.ev("s", "ready", 1, json!({"state": ready}));
            }
            r.ev("s", "prompt_submit", 2, json!({"origin": origin, "basis": basis}));
            r.adv(10_000);
            r.ev("s", "turn_stop", 3, json!({}));
            let stop = r.at();
            let at = |d: u64| {
                r.now.store(stop + d, Ordering::SeqCst);
                cv(&r, "s")
            };
            let want = |b: bool| if b { Conversation::Operator } else { Conversation::None };
            assert_eq!(at(119_000), want(expect_op), "{ready} {origin} 119s");
            assert_eq!(at(120_000), want(expect_op), "{ready} {origin} 120s");
            assert_eq!(at(121_000), Conversation::None, "{ready} {origin} 121s");
        }
    }
}

/// S1 client input after the prompt extends the window; no stop -> 120 s
/// after the last S2 event (FCW-17).
#[test]
fn e18_client_input_extends_and_fcw17_no_stop() {
    let r = rig();
    r.ev("s", "prompt_submit", 2, json!({"origin": "human", "basis": "inferred"}));
    r.ev("s", "turn_stop", 3, json!({}));
    let stop = r.at();
    r.now.store(stop + 100_000, Ordering::SeqCst);
    r.set_s1("s", Some(r.at()), None, None);
    r.m.tick();
    r.now.store(stop + 121_000, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::Operator, "keystroke at 100 s extends to 220 s");
    r.now.store(stop + 221_000, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::None);

    let r = rig();
    r.ev("s", "prompt_submit", 2, json!({"origin": "human", "basis": "inferred"}));
    r.adv(300_000);
    r.ev("s", "tool_end", 3, json!({"tool": "Bash"}));
    let last = r.at();
    r.now.store(last + 120_000, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::Operator, "no turn stop: counted from the last S2 event");
    r.now.store(last + 120_001, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::None);
}

/// agent: an S3 event in a conversation with an owed answer, for 120 s; ends
/// when nothing is owed; a ledger error is unknown (FCW-18).
#[test]
fn e18_agent_conversation_and_fcw18() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    r.m.on_sent(Some("s"), "c1");
    assert_eq!(cv(&r, "s"), Conversation::None, "nothing owed in c1");
    r.owed.lock().unwrap().as_mut().unwrap().insert("c1".into());
    let t = r.at();
    assert_eq!(cv(&r, "s"), Conversation::Agent(vec!["c1".into()]));
    r.now.store(t + 120_000, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::Agent(vec!["c1".into()]));
    r.now.store(t + 120_001, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::None, "120 s without S3 in c1");
    r.m.on_handed_over("s", "c1");
    *r.owed.lock().unwrap() = Err("chain unreadable".into());
    assert!(matches!(cv(&r, "s"), Conversation::Unknown(_)));
}

// ---- E-19: status activity is fresh or unknown ---------------------------------------------------

/// After two ticks without a meter tick every field reads unknown. Control:
/// serve the cached snapshot without the stamp check and the frozen read shows numbers.
#[test]
fn e19_stale_status_reads_unknown() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    r.set_s1("s", Some(r.at()), None, None);
    let a = r.act("s");
    assert!(a["stamped_at"].is_u64() && a["s1_at"].is_u64() && a["s2_at"].is_u64(), "{a}");
    assert_eq!(a["conversation"], "none");
    r.adv(STALE_MS); // frozen: no tick
    let a = r.m.status_view()["sessions"]["s"].clone();
    assert!(a["s1_at"].is_u64(), "at the bound it is still fresh: {a}");
    r.adv(1);
    let a = r.m.status_view()["sessions"]["s"].clone();
    assert_eq!(a["stale"], true);
    for k in ["s1_at", "s2_at", "s3_at", "last", "source", "conversation", "watchdog"] {
        assert_eq!(a[k], "unknown", "{k} must not show its last value: {a}");
    }
}

/// E-84 negative-control shape: a doorbell line reported human is recorded
/// automated, and a missing READY never reads as ready.
#[test]
fn e84_negative_controls_at_the_receiver() {
    let r = rig();
    r.m.note_typed("s", &["abc".into()]);
    r.ev("s", "prompt_submit", 1, json!({"origin": "human", "basis": "inferred", "doorbell_ids": ["abc"]}));
    assert_eq!(r.m.last_prompt("s").unwrap().origin, Origin::Automated);
    assert!(r.m.ready("s").is_none(), "no ready report: no READY record");
}
