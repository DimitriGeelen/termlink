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

/// F-22 (CR-6.20 / CR-B27.3): the tick's S1 reads share one deadline. Six slow
/// sessions (each read 600 ms) under a 200 ms budget: the tick returns well
/// within one serial read set, the slow ones read unknown this tick, the fast
/// one is known, and a slow read still in flight is not started again.
/// Control: read serially without a budget and the tick takes >= 3.6 s and every
/// slow session reads known.
#[test]
fn f22_s1_reads_share_a_tick_deadline() {
    struct Slow(Arc<FakeS1>);
    impl S1Source for Slow {
        fn read(&self, s: &str) -> Result<S1Reply, String> {
            if s.starts_with("slow") {
                std::thread::sleep(std::time::Duration::from_millis(600));
            }
            self.0.0.lock().unwrap().get(s).cloned().unwrap_or(Err("unbound".into()))
        }
    }
    let now = Arc::new(AtomicU64::new(1_000_000));
    let s1 = Arc::new(FakeS1::default());
    let (n1, n2) = (now.clone(), now.clone());
    let m = Meter::new(A, Box::new(Slow(s1.clone())), Box::new(FakeLedger(Arc::new(Mutex::new(Ok(HashSet::new()))))))
        .with_clocks(Arc::new(move || n1.load(Ordering::SeqCst)), Arc::new(move || n2.load(Ordering::SeqCst)))
        .with_tick_budget(200);
    let r = Rig { m, now, s1, owed: Arc::new(Mutex::new(Ok(HashSet::new()))) };
    let mut names: Vec<String> = (0..6).map(|i| format!("slow{i}")).collect();
    names.push("fast".into());
    for n in &names {
        r.ev(n, "session_start", 1, json!({"interactive": true}));
        r.set_s1(n, Some(r.at()), None, None);
    }
    let t0 = std::time::Instant::now();
    r.m.tick();
    let took = t0.elapsed();
    assert!(took < std::time::Duration::from_millis(1500), "the tick took {took:?}");
    let v = r.m.status_view();
    assert_eq!(v["sessions"]["fast"]["s1"], "known", "{v}");
    for n in &names[..6] {
        let s = v["sessions"][n]["s1"].as_str().unwrap();
        assert!(s.contains("tick budget"), "{n}: {s}");
        assert_eq!(v["sessions"][n]["watchdog"], "unavailable: S1 unknown");
    }
    // The slow reads are still running: the next tick does not start them again.
    r.m.tick();
    let v = r.m.status_view();
    assert!(v["sessions"]["slow0"]["s1"].as_str().unwrap().contains("has not returned"), "{v}");
    assert_eq!(v["sessions"]["fast"]["s1"], "known");
}

// ---- E-16: S2 and the strict intake -----------------------------------------------------------

/// S2 (a) moves on turn events and READY state changes only (F-3, CR-6.3 / CR-B27.1).
/// Control: stamp S2 on every valid report (the pre-F-3 rule) and the "repeat",
/// `evidence` and `complete` rows move S2, going red.
#[test]
fn e16_f3_s2_moves_on_turn_events_and_ready_changes_only() {
    let r = rig();
    // (event, extra, moves S2)
    let evs: Vec<(&str, Value, bool)> = vec![
        ("session_start", json!({"interactive": true}), true),
        ("prompt_submit", json!({"origin": "human", "basis": "inferred"}), true),
        ("ready", json!({"state": "BUSY"}), true),
        ("ready", json!({"state": "BUSY"}), false),
        ("tool_start", json!({"tool": "Bash"}), true),
        ("ready", json!({"state": "BUSY"}), false),
        ("tool_end", json!({"tool": "Bash"}), true),
        ("evidence", json!({"nonce": "n1", "record_ref": "r"}), false),
        ("complete", json!({"turn_boundary": true}), false),
        ("turn_stop", json!({}), true),
        ("ready", json!({"state": "READY"}), true),
        ("ready", json!({"state": "READY"}), false),
        ("session_end", json!({}), true),
    ];
    r.set_s1("s", None, None, None);
    let mut s2 = None;
    for (i, (e, x, moves)) in evs.into_iter().enumerate() {
        r.adv(1000);
        assert_eq!(r.ev("s", e, 10 + i as u64, x), Intake::Applied, "{e}");
        let a = r.act("s");
        if moves {
            s2 = Some(r.at());
        }
        assert_eq!(a["s2_at"], json!(s2), "{e} (#{i}) moves S2 = {moves}: {a}");
    }
    assert_eq!(r.m.counters().unparseable, 0);
    // An adapter that only repeats `ready {BUSY}` leaves S2 silent, so the watchdog can see it.
    let r = rig();
    r.set_s1("s", None, None, None);
    r.ev("s", "ready", 1, json!({"state": "BUSY"}));
    let first = r.at();
    for i in 0..10 {
        r.adv(T);
        r.ev("s", "ready", 2 + i, json!({"state": "BUSY"}));
    }
    let a = r.act("s");
    assert_eq!(a["s2_at"], json!(first), "{a}");
    assert_eq!(a["watchdog"], "available");
    // The built hooks: the four turn hooks move S2, any other hook name does not.
    let r = rig();
    r.m.legacy_hook("s", "post-tool-use", None);
    let t = r.at();
    r.adv(1000);
    r.m.legacy_hook("s", "notification", None);
    assert_eq!(r.act("s")["s2_at"], json!(t));
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

/// F-4 (CR-6.4 / CR-B27.2): the counter check runs for every `human` origin of a
/// PTY session, basis `inferred` included (the Claude Code default), and
/// `client_auto` bytes after the doorbell line do not make it human.
/// Control: run the check for basis `pty-input` only and the `inferred` rows stay human.
#[test]
fn f4_counter_check_for_every_human_origin() {
    let r = rig();
    let origin = |r: &Rig| r.m.last_prompt("s").map(|p| (p.origin, p.basis));
    let t = r.at();
    // The last PTY input before the prompt came from notify: the doorbell typed it.
    r.set_s1("s", Some(t - 50), Some(t - 10), None);
    r.ev("s", "prompt_submit", 1, json!({"origin": "human", "basis": "inferred"}));
    assert_eq!(origin(&r), Some((Origin::Automated, "pty-input".into())));
    r.set_s1("s", Some(t - 50), None, Some(t - 5));
    r.ev("s", "prompt_submit", 2, json!({"origin": "human", "basis": "inferred"}));
    assert_eq!(origin(&r), Some((Origin::Automated, "pty-input".into())));
    // A terminal reply (client_auto) after the doorbell line is not a person typing.
    r.set_s1("s", Some(t - 50), Some(t - 10), None);
    r.s1.0.lock().unwrap().get_mut("s").unwrap().as_mut().unwrap().last.insert("client_auto".into(), Some(t - 1));
    r.ev("s", "prompt_submit", 3, json!({"origin": "human", "basis": "inferred"}));
    assert_eq!(origin(&r), Some((Origin::Automated, "pty-input".into())), "client_auto is ignored");
    assert_eq!(r.m.counters().origin_overridden, 3);
    // Client input last: human stays human with its basis.
    r.set_s1("s", Some(t - 1), Some(t - 10), None);
    r.ev("s", "prompt_submit", 4, json!({"origin": "human", "basis": "inferred"}));
    assert_eq!(origin(&r), Some((Origin::Human, "inferred".into())));
    // No PTY counters (an unbound or non-PTY session): `inferred` stays as reported.
    r.s1.0.lock().unwrap().clear();
    r.ev("s", "prompt_submit", 5, json!({"origin": "human", "basis": "inferred"}));
    assert_eq!(origin(&r), Some((Origin::Human, "inferred".into())));
    assert_eq!(r.m.counters().origin_overridden, 3);
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
    // Cargo's per-build scratch dir for integration tests: inside the build's own target dir, never the
    // source tree (which is read-only on node1, T-3677) and never /tmp.
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("t3687-meter-{}", std::process::id()));
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

/// F-28 (CR-6.25 / CR-B27.6): a transcript whose lstat fails is not a silent S2.
/// With no fresh report S2 is unknown and there is no watchdog; a fresh report
/// keeps it known; the transcript readable again makes it known.
/// Control: keep `s2_known = reports > 0 || transcript` and the watchdog reads
/// available on an unreadable transcript.
#[test]
fn f28_unreadable_transcript_is_s2_unknown_without_a_fresh_report() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("t3687-f28-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let root = dir.join("projects");
    std::fs::create_dir_all(root.join("p")).unwrap();
    let tr = root.join("p").join("s.jsonl");
    std::fs::write(&tr, "{}\n").unwrap();
    let mut r = rig();
    r.m = r.m.with_transcript_root(root.clone());
    r.ev("s", "session_start", 1, json!({"interactive": true, "transcript_path": tr.to_string_lossy()}));
    r.set_s1("s", None, None, None);
    assert_eq!(r.act("s")["watchdog"], "available");
    std::fs::remove_file(&tr).unwrap();
    // The session_start report is fresh: S2 stays known.
    r.adv(T);
    let a = r.act("s");
    assert_eq!(a["watchdog"], "available", "a fresh report keeps S2 known: {a}");
    // No report for longer than the watchdog window: unknown, never silence.
    r.adv(S2_FRESH_MS);
    let a = r.act("s");
    assert_eq!(a["s2_at"], "unknown", "{a}");
    assert_eq!(a["watchdog"], "unavailable: S2 unknown (transcript unreadable and no fresh report)");
    assert_ne!(a["source"], "s2");
    // A new report makes S2 known again (an `evidence` report: fresh, though it does not move S2).
    r.ev("s", "evidence", 2, json!({"nonce": "n1"}));
    assert_eq!(r.act("s")["watchdog"], "available");
    // The transcript back: known without a fresh report.
    std::fs::write(&tr, "{}\n").unwrap();
    r.adv(S2_FRESH_MS + 1);
    assert_eq!(r.act("s")["watchdog"], "available");
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
    // A prompt this epoch (automated), so the F-26 restart rule does not apply.
    r.ev("s", "prompt_submit", 2, json!({"origin": "automated", "basis": "doorbell"}));
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

/// F-26 (CR-6.23 / CR-B27.4): `last_prompt` is lost on a sidecar restart, so a
/// session with no prompt this epoch and S2 within 120 s reads `operator`
/// (unknown fails toward a person); without S2 in 120 s it reads none.
/// Control: drop the restart rule and the new meter reads `none` while the
/// person's session is still active.
#[test]
fn f26_no_prompt_this_epoch_with_recent_s2_is_operator() {
    // Before the restart: a human prompt, a running turn.
    let old = rig();
    old.ev("s", "prompt_submit", 1, json!({"origin": "human", "basis": "inferred"}));
    assert_eq!(cv(&old, "s"), Conversation::Operator);
    // After the restart: a new meter (new epoch) sees only the next turn event.
    let r = rig();
    r.ev("s", "tool_end", 2, json!({"tool": "Bash"}));
    assert!(r.m.last_prompt("s").is_none());
    let t = r.at();
    assert_eq!(cv(&r, "s"), Conversation::Operator, "no prompt this epoch, S2 now");
    r.now.store(t + V19_MS, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::Operator, "at 120 s");
    r.now.store(t + V19_MS + 1, Ordering::SeqCst);
    assert_eq!(cv(&r, "s"), Conversation::None, "S2 older than 120 s");
    // Transcript growth or a later turn event renews it; a prompt this epoch ends the rule.
    r.ev("s", "tool_start", 3, json!({"tool": "Bash"}));
    assert_eq!(cv(&r, "s"), Conversation::Operator);
    r.ev("s", "prompt_submit", 4, json!({"origin": "automated", "basis": "doorbell"}));
    assert_eq!(cv(&r, "s"), Conversation::None, "an automated prompt this epoch is not a person");
}

// ---- E-19: status activity is fresh or unknown ---------------------------------------------------

/// After two ticks without a meter tick every field reads unknown. Control:
/// serve the cached snapshot without the stamp check and the frozen read shows numbers.
#[test]
fn e19_stale_status_reads_unknown() {
    let r = rig();
    r.ev("s", "session_start", 1, json!({"interactive": true}));
    r.ev("s", "prompt_submit", 2, json!({"origin": "automated", "basis": "doorbell"}));
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
