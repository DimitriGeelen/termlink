//! B-16 (T-3697, arc-011 plan unit P-7): the doorbell D+ and the 300 s watchdog K-33.
//! One test (or more) per E-n of 05 6.1 at tier 1 (simulated clock, fake ports), the
//! store integration (ATTEMPTED never HANDED_OVER, the claim is not bypassed, the kill
//! switch), and one live run against a real TermLink PTY session (session binding for
//! S1 and `command.notify`; the SI-1 line typed into a real shell). Hermetic: temp dirs
//! and a test estate only; no real production path, no hub.

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Mutex;
use termlink_sidecar::doorbell::*;
use termlink_sidecar::meter::Conversation;

// ------------------------------------------------------------------ fake ports

#[derive(Default)]
struct St {
    mono: u64,
    /// Added to `mono` on every `input_activity` call (time spent in the checks).
    step_ms: u64,
    switch: bool,
    /// `ready()` answers this from the given call number (1-based) on.
    ready: Option<ReadyIn>,
    ready_after: Option<(usize, Option<ReadyIn>)>,
    ready_calls: usize,
    fg: Option<Fg>,
    fg_after: Option<(usize, Option<Fg>)>,
    ia_calls: usize,
    veto: Option<(Veto, bool)>,
    conv: Option<Conversation>,
    conv_after: Option<(usize, Conversation)>,
    conv_calls: usize,
    fail_append: bool,
    fail_admin: bool,
    notify_result: Option<Notified>,
    /// A turn starts at the keystrokes (E-14).
    ready_busy_on_notify: bool,
    log: Vec<Value>,
    admin: Vec<Value>,
    notifies: Vec<(String, Vec<String>, u64)>,
    attempted: Vec<(String, Via, String)>,
    unconfirmed: Vec<String>,
    attention: BTreeMap<(String, String), String>,
    ids: u64,
}

struct Fake(Mutex<St>);

const HPID: u32 = 4242;
fn harness() -> Harness {
    Harness { pid: HPID, exe: Some("/opt/claude/claude".into()), script: None }
}
fn fg_ok() -> Fg {
    Fg { leader_pid: Some(HPID), exe: Some("/opt/claude/claude".into()), script: None, client_last: None }
}
fn ready(state: &str, seq: u64) -> ReadyIn {
    ReadyIn { state: state.into(), seq, current_epoch: true, rx_mono: 0 }
}

impl Fake {
    fn new() -> Self {
        Fake(Mutex::new(St { mono: 10_000_000, switch: true, ready: Some(ready("READY", 1)), fg: Some(fg_ok()), veto: Some((Veto::Quiet, false)), conv: Some(Conversation::None), notify_result: Some(Notified::Typed), ..Default::default() }))
    }
    fn s(&self) -> std::sync::MutexGuard<'_, St> {
        self.0.lock().unwrap()
    }
    fn now(&self) -> u64 {
        self.s().mono
    }
    fn advance(&self, ms: u64) {
        self.s().mono += ms;
    }
    fn notifies(&self) -> usize {
        self.s().notifies.len()
    }
}

impl Ports for Fake {
    fn mono(&self) -> u64 {
        self.s().mono
    }
    fn switch_on(&self) -> bool {
        self.s().switch
    }
    fn ready(&self, _: &str) -> Option<ReadyIn> {
        let mut s = self.s();
        s.ready_calls += 1;
        match &s.ready_after {
            Some((n, r)) if s.ready_calls >= *n => r.clone(),
            _ => s.ready.clone(),
        }
    }
    fn conversation(&self, _: &str, _: u64) -> Conversation {
        let mut s = self.s();
        s.conv_calls += 1;
        match &s.conv_after {
            Some((n, c)) if s.conv_calls >= *n => c.clone(),
            _ => s.conv.clone().unwrap(),
        }
    }
    fn input_activity(&self, _: &str) -> Result<Fg, String> {
        let mut s = self.s();
        s.ia_calls += 1;
        let step = s.step_ms;
        s.mono += step;
        let fg = match &s.fg_after {
            Some((n, f)) if s.ia_calls >= *n => f.clone(),
            _ => s.fg.clone(),
        };
        fg.ok_or_else(|| "no reply".into())
    }
    fn veto(&self, _: &str, _: u32) -> (Veto, bool) {
        self.s().veto.clone().unwrap()
    }
    fn append(&self, row: &Value, admin: bool) -> Result<(), String> {
        let mut s = self.s();
        if s.fail_append || (admin && s.fail_admin) {
            return Err("disk full".into());
        }
        if admin {
            s.admin.push(row.clone());
        }
        s.log.push(row.clone());
        Ok(())
    }
    fn notify(&self, session: &str, ids: &[String], count: u64) -> Notified {
        let mut s = self.s();
        s.notifies.push((session.into(), ids.to_vec(), count));
        if s.ready_busy_on_notify {
            s.ready = Some(ready("BUSY", 99));
        }
        s.notify_result.clone().unwrap()
    }
    fn attempted(&self, ids: &[String], via: Via, ring_id: &str) {
        let mut s = self.s();
        for i in ids {
            s.attempted.push((i.clone(), via, ring_id.into()));
        }
    }
    fn unconfirmed(&self, id: &str) {
        self.s().unconfirmed.push(id.into());
    }
    fn attention(&self, kind: &str, key: &str, detail: &str) {
        self.s().attention.entry((kind.into(), key.into())).or_insert_with(|| detail.into());
    }
    fn new_id(&self) -> String {
        let mut s = self.s();
        s.ids += 1;
        format!("ring-{}", s.ids)
    }
}

fn hex(n: u64) -> String {
    format!("{n:032x}")
}

fn msg(n: u64) -> Msg {
    Msg { id: hex(n), conv: "c1".into(), urgent: false, stored_mono: 0, last_pull_mono: None, attempts: 0, bound: None }
}

fn facts(name: &str) -> SessFacts {
    SessFacts {
        name: name.into(),
        ended: false,
        harness: Some(harness()),
        registered: true,
        pty_launched: true,
        s1: S1In::Known(None),
        s2_known: true,
        acts: (None, None, None),
        ready: Some(ready("READY", 1)),
        conversation: Conversation::None,
        last_submit: None,
        role_mail: true,
        bound_mail: true,
    }
}

type Flip = Box<dyn Fn(&Fake)>;

fn no_hand(_: &str) -> bool {
    false
}

// ------------------------------------------------------------------ E-1

/// E-1: the 16 combinations of (a) mail, (b) READY, (c) no typing, (d) conversation none.
#[test]
fn e1_ring_gate_sixteen_rows() {
    let now = 10_000_000;
    let mut table = vec![];
    for bits in 0..16u8 {
        let (a, b, c, d) = (bits & 8 != 0, bits & 4 != 0, bits & 2 != 0, bits & 1 != 0);
        let pending = if a { vec![msg(1)] } else { vec![] };
        let r = ready(if b { "READY" } else { "BUSY" }, 1);
        let s1 = S1In::Known(if c { Some(now - 31_000) } else { Some(now - 5_000) });
        let conv = if d { Conversation::None } else { Conversation::Operator };
        let dec = decide(&GateIn { now, pending: &pending, role_mail: true, bound_mail: true, session: "s", pty_launched: true, harness_registered: true, ready: Some(&r), s1: &s1, conversation: &conv, schedule: Ok(()) });
        table.push((bits, matches!(dec, Decision::Ring { .. }) as u8));
    }
    let expected: Vec<(u8, u8)> = (0..16).map(|b| (b, (b == 15) as u8)).collect();
    assert_eq!(table, expected);
    // Absent READY, unknown S1, unknown conversation, another epoch: all hold (fail closed).
    let p = vec![msg(1)];
    let r_old = ReadyIn { current_epoch: false, ..ready("READY", 1) };
    for (r, s1, conv) in [
        (None, S1In::Known(None), Conversation::None),
        (Some(&r_old), S1In::Known(None), Conversation::None),
        (Some(&ready("READY", 1)), S1In::Unknown, Conversation::None),
        (Some(&ready("READY", 1)), S1In::Known(None), Conversation::Unknown("ledger".into())),
    ] {
        let d = decide(&GateIn { now, pending: &p, role_mail: true, bound_mail: true, session: "s", pty_launched: true, harness_registered: true, ready: r, s1: &s1, conversation: &conv, schedule: Ok(()) });
        assert!(matches!(d, Decision::Hold(_)), "{d:?}");
    }
    // agent(C): only C's mail; another conversation's holds.
    let c = Conversation::Agent(vec!["c2".into()]);
    let r = ready("READY", 1);
    let s1 = S1In::Known(None);
    let d = decide(&GateIn { now, pending: &p, role_mail: true, bound_mail: true, session: "s", pty_launched: true, harness_registered: true, ready: Some(&r), s1: &s1, conversation: &c, schedule: Ok(()) });
    assert_eq!(d, Decision::Hold("conversation"));
    // FCW-20: mail stored less than a tick ago waits for the hook path.
    let fresh = vec![Msg { stored_mono: now - 1_000, ..msg(1) }];
    let d = decide(&GateIn { now, pending: &fresh, role_mail: true, bound_mail: true, session: "s", pty_launched: true, harness_registered: true, ready: Some(&r), s1: &s1, conversation: &Conversation::None, schedule: Ok(()) });
    assert_eq!(d, Decision::Hold("nothing"));
}

/// E-1.4: at most one ring per agent per tick, and every decision is recorded (FCW-26).
#[test]
fn e1_one_session_per_agent_per_tick_and_decisions_recorded() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut a = facts("sess-a");
    a.acts = (Some(1), None, None);
    let mut b = facts("sess-b");
    b.acts = (Some(2), None, None);
    let rep = db.tick(&f, &[a, b], &[msg(1)], &no_hand);
    assert_eq!(rep.decisions.len(), 2);
    assert_eq!(f.notifies(), 1);
    assert_eq!(f.s().notifies[0].0, "sess-b", "the larger last activity is rung");
}

// ------------------------------------------------------------------ E-2

fn ring_once(f: &Fake) -> (RingOut, Value) {
    let snap = Snap { ready_seq: 1, convs: vec!["c1".into()] };
    type_ring(f, "s", &harness(), &[hex(1)], 1, Some(&snap), Via::Doorbell, json!({}))
}

/// E-2: each re-check, flipped between the gate and the keystrokes, stops the keystrokes.
#[test]
fn e2_rechecks_each_fail_closed_and_clean_run_logs_interval() {
    let cases: Vec<(&str, Flip)> = vec![
        ("kill-switch", Box::new(|f: &Fake| f.s().switch = false)),
        ("ready", Box::new(|f: &Fake| f.s().ready = Some(ready("BUSY", 2)))),
        ("foreground", Box::new(|f: &Fake| f.s().fg = Some(Fg { leader_pid: Some(77), exe: Some("/bin/bash".into()), script: None, client_last: None }))),
        ("veto:cpu", Box::new(|f: &Fake| f.s().veto = Some((Veto::Active("cpu".into()), false)))),
        ("typing", Box::new(|f: &Fake| {
            let now = f.now();
            f.s().fg = Some(Fg { client_last: Some(now), ..fg_ok() })
        })),
        ("conversation", Box::new(|f: &Fake| f.s().conv = Some(Conversation::Operator))),
    ];
    for (check, flip) in cases {
        let f = Fake::new();
        flip(&f);
        let (out, row) = ring_once(&f);
        assert_eq!(out, RingOut::Held(check.into()), "{check}");
        assert_eq!(f.notifies(), 0, "{check}: no keystrokes");
        assert_eq!(row["outcome"], format!("held:{check}"));
        assert!(f.s().log.is_empty(), "{check}: a held ring writes no intent (CR-6.9)");
    }
    let f = Fake::new();
    f.s().step_ms = 5;
    let (out, row) = ring_once(&f);
    assert!(matches!(out, RingOut::Done { notified: Notified::Typed, .. }), "{out:?}");
    assert_eq!(f.notifies(), 1);
    assert!(row["check_to_keystroke_ms"].is_u64() && row["t_first_check"].is_u64() && row["t_last_check"].is_u64());
    assert!(row["first_check_to_keystroke_ms"].as_u64() >= row["check_to_keystroke_ms"].as_u64(), "measured from the earliest check too (CR-6.2)");
    let intent = &f.s().log[0];
    assert_eq!(intent["phase"], "intent");
}

/// E-2.6: READY to BUSY, or the foreground to a shell, during the veto window: held at the re-read.
#[test]
fn e2_6_reread_after_the_veto() {
    // ready(): call 1 = the pre-veto re-read; call 2 = after the veto.
    let f = Fake::new();
    f.s().ready_after = Some((2, Some(ready("BUSY", 2))));
    assert_eq!(ring_once(&f).0, RingOut::Held("ready".into()));
    assert_eq!(f.notifies(), 0);
    let f = Fake::new();
    f.s().fg_after = Some((2, Some(Fg { leader_pid: Some(77), exe: Some("/bin/bash".into()), script: None, client_last: None })));
    assert_eq!(ring_once(&f).0, RingOut::Held("foreground".into()));
    assert_eq!(f.notifies(), 0);
}

/// E-2.7: the foreground check is the harness pid and (exe, script), never `comm`.
#[test]
fn e2_7_is_harness_binding() {
    let node = Harness { pid: 10, exe: Some("/usr/bin/node".into()), script: Some("/x/@anthropic-ai/claude-code/cli.js".into()) };
    let fg = |pid: u32, exe: &str, script: Option<&str>| Fg { leader_pid: Some(pid), exe: Some(exe.into()), script: script.map(Into::into), client_last: None };
    assert!(is_harness(&node, &fg(10, "/usr/bin/node", Some("/x/@anthropic-ai/claude-code/cli.js"))), "(a) the interpreter harness");
    assert!(!is_harness(&node, &fg(10, "/usr/bin/node", Some("/tmp/other.js"))), "(b) another script");
    assert!(!is_harness(&node, &fg(11, "/usr/bin/node", Some("/x/@anthropic-ai/claude-code/cli.js"))), "(c) a restarted harness (new leader)");
    assert!(!is_harness(&harness(), &fg(HPID, "/usr/bin/python3", None)), "(d) renamed to the harness's comm");
    assert!(registered(&node, &[]));
    assert!(!registered(&Harness { script: Some("/tmp/other.js".into()), ..node.clone() }, &[]));
    assert!(registered(&harness(), &[]));
    assert!(!registered(&Harness { pid: 1, exe: Some("/bin/bash".into()), script: None }, &[]));
}

// ------------------------------------------------------------------ E-3

/// E-3 tier 1: every line matches the SI-1 expression; every invalid input is refused.
#[test]
fn e3_si1_line_fuzz() {
    let re = |l: &str| {
        let Some(rest) = l.strip_prefix("sidecar: ") else { return false };
        let Some((n, rest)) = rest.split_once(" peer messages waiting, ids ") else { return false };
        let Some(ids) = rest.strip_suffix(". The prompt hook shows them as untrusted data.") else { return false };
        let ids: Vec<&str> = ids.split(' ').collect();
        !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && (1..=4).contains(&ids.len()) && ids.iter().all(|i| i.len() == 32 && i.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
    };
    let nasty = ["\n", "\x1b[2J", "$(id)", "`id`", ";", "\u{0430}", "A", " ", ""];
    let mut seed: u64 = 0x05ee_db16;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let (mut ok, mut refused) = (0, 0);
    for _ in 0..100_000 {
        let n = (next() % 6) as usize;
        let mut ids = vec![];
        for _ in 0..n {
            let mut id = format!("{:032x}", next() as u128 * next() as u128);
            match next() % 8 {
                0 => id.truncate(31),
                1 => id.push('a'),
                2 => id = id.to_uppercase(),
                3 => id.replace_range(5..6, nasty[(next() % nasty.len() as u64) as usize]),
                _ => {}
            }
            ids.push(id);
        }
        let count = match next() % 4 {
            0 => 0,
            1 => n as u64,
            2 => 1_000_001,
            _ => n as u64 + next() % 1000,
        };
        match compose(&ids, count) {
            Ok(l) => {
                assert!(re(&l), "{l:?}");
                assert!(l.is_ascii() && !l.contains('\n'));
                ok += 1;
            }
            Err(_) => refused += 1,
        }
    }
    assert!(ok > 1000 && refused > 1000, "{ok} {refused}");
}

/// E-3 tier 2 (local shells): typed into a shell the line starts nothing (exit 127).
#[test]
fn e3_line_typed_into_shells_is_command_not_found() {
    let line = compose(&[hex(1), hex(2)], 2).unwrap();
    for sh in ["bash", "sh", "zsh", "dash"] {
        if std::process::Command::new("sh").args(["-c", &format!("command -v {sh}")]).output().map(|o| o.stdout.is_empty()).unwrap_or(true) {
            continue;
        }
        let o = std::process::Command::new(sh).arg("-c").arg(&line).output().unwrap();
        assert_eq!(o.status.code(), Some(127), "{sh}: {o:?}");
    }
}

// ------------------------------------------------------------------ E-4

/// E-4 tier 1: a ring is ATTEMPTED, never HANDED_OVER; 90 s without evidence labels it once.
#[test]
fn e4_ring_is_attempted_then_unconfirmed() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    assert_eq!(f.s().attempted.len(), 1);
    assert_eq!(f.s().attempted[0].1, Via::Doorbell);
    for _ in 0..3 {
        f.advance(30_000);
        db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    }
    assert_eq!(f.s().unconfirmed, vec![hex(1)], "labelled once at 90 s");
    f.advance(30_000);
    db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    assert_eq!(f.s().unconfirmed.len(), 1);
}

// ------------------------------------------------------------------ E-5

/// E-5.6: a submitting adapter is rung at ticks 0, 4, 12, 32, 52, 72, 92, 112 (60 min).
#[test]
fn e5_6_backoff_schedule_with_submits() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut fa = facts("s");
    let mut rung = vec![];
    for t in 0..120u64 {
        let rep = db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        let rang = matches!(rep.rung, Some((_, RingOut::Done { .. })));
        if rang {
            rung.push(t);
        }
        // The adapter submits every typed line within the tick, the watchdog's too (ARC4
        // 6.9.7.e: it may fire in back-off and leaves the step unchanged), with proof of absence.
        if rang || matches!(rep.watchdog, Some((_, RingOut::Done { .. }))) {
            let at = f.now() + 1_000;
            fa.last_submit = Some((at, vec![hex(1)]));
            fa.acts.1 = Some(at);
        }
        f.advance(30_000);
    }
    assert_eq!(rung, vec![0, 4, 12, 32, 52, 72, 92, 112]);
}

/// E-5.6 / E-7.7: a non-submitting adapter: one ring, one watchdog line, then STALLED.
#[test]
fn e5_6_unsubmitted_then_one_watchdog_then_stalled() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let fa = facts("s");
    for _ in 0..120 {
        db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        f.advance(30_000);
    }
    let vias: Vec<Via> = f.s().attempted.iter().map(|a| a.1).collect();
    assert_eq!(vias, vec![Via::Doorbell, Via::Watchdog], "one ring, one watchdog line, nothing more");
    assert_eq!(db.sched("s").unwrap().state, RingState::Stalled);
    let stalled = f.s().attention.keys().filter(|(k, _)| k == "session-stalled").count();
    assert_eq!(stalled, 1);
    assert_eq!(db.h11.e_unsubmitted, 2);
    // A submit ends the episode: STALLED -> BACKOFF, and the normal ring that is due resumes.
    let mut fa = facts("s");
    fa.last_submit = Some((f.now(), vec![]));
    let before = f.notifies();
    db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
    assert_ne!(db.sched("s").unwrap().state, RingState::Stalled);
    assert_eq!(f.notifies(), before + 1);
}

/// E-5.7: at MAX_ATTEMPTS no further line, one needs-attention entry per message.
#[test]
fn e5_7_max_attempts_bound() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let m = Msg { attempts: MAX_ATTEMPTS, ..msg(1) };
    for _ in 0..30 {
        db.tick(&f, &[facts("s")], std::slice::from_ref(&m), &no_hand);
        f.advance(30_000);
    }
    assert_eq!(f.notifies(), 0);
    assert_eq!(f.s().attention.keys().filter(|(k, _)| k == "doorbell-max-attempts").count(), 1);
}

/// E-5.1: no re-type on top of an unsubmitted line; the schedule state machine itself.
#[test]
fn e5_schedule_table() {
    let mut s = Sched::default();
    assert_eq!(s.may_ring(0), Ok(()));
    s.on_typed(0, 0, Via::Doorbell, "r1", &[]);
    assert_eq!(s.may_ring(1), Err("unsubmitted"));
    assert_eq!(s.on_tick(30_000), None, "not yet more than one tick");
    assert!(matches!(s.on_tick(30_001), Some(SchedEvent::Unsubmitted(_))));
    assert_eq!(s.may_ring(100), Err("unsubmitted"));
    s.on_submit(40_000);
    assert_eq!(s.state, RingState::Backoff);
    assert_eq!(s.may_ring(3), Err("backoff"));
    assert_eq!(s.may_ring(4), Ok(()));
    s.on_typed(4, 120_000, Via::Watchdog, "w", &[]);
    assert_eq!(s.k, 0, "a watchdog line never moves k");
    assert!(matches!(s.on_tick(160_000), Some(SchedEvent::Stalled(_))));
    s.on_typed(5, 170_000, Via::Watchdog, "w2", &[]);
    assert_eq!(s.state, RingState::Stalled, "nothing typed into a STALLED session");
    s.on_progress();
    assert_eq!(s.state, RingState::Stalled, "progress does not end STALLED");
    s.on_session_start();
    assert_eq!(s.state, RingState::Idle);
}

// ------------------------------------------------------------------ E-6 (L2 part)

/// E-6 (L2 part): a replayed line whose ids were already handed over is counted stray.
#[test]
fn e6_stray_replay_is_counted() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut fa = facts("s");
    db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
    f.advance(10_000);
    fa.last_submit = Some((f.now(), vec![hex(1)]));
    db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &|_| true);
    assert_eq!(db.h11.b_stray, 1);
    assert!(f.s().log.iter().any(|r| r["h11"] == "b"));
}

// ------------------------------------------------------------------ E-7

fn silent_busy() -> SessFacts {
    let mut fa = facts("s");
    fa.ready = Some(ready("BUSY", 1));
    fa.conversation = Conversation::Agent(vec!["other".into()]);
    fa
}

/// E-7: the watchdog fires after 300 s of silence even with READY busy, with its admin row.
#[test]
fn e7_watchdog_fires_at_300s_independent_of_ready() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let fa = silent_busy();
    let start = f.now();
    let mut fired_at = None;
    for _ in 0..12 {
        let rep = db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        if fired_at.is_none() && matches!(rep.watchdog, Some((_, RingOut::Done { .. }))) {
            fired_at = Some(f.now() - start);
        }
        f.advance(30_000);
    }
    assert_eq!(fired_at, Some(300_000), "no keystroke before 300 s; one at 300 s (the first tick at or after)");
    assert_eq!(f.notifies(), 1);
    let s = f.s();
    let intent = s.admin.iter().find(|r| r["phase"] == "intent").expect("the admin row is the intent");
    for k in ["kind", "session", "at", "silent_for_s", "ready_flag", "ready_age_s", "last_activity", "checks", "message_ids", "ring_id"] {
        assert!(intent.get(k).is_some(), "admin row lacks {k}: {intent}");
    }
    assert_eq!(intent["kind"], WATCHDOG_EVENT);
    assert_eq!(intent["ready_flag"], "busy");
    let outcome = s.admin.iter().find(|r| r["phase"] == "outcome").unwrap();
    assert_eq!(outcome["outcome"], "typed");
    assert_eq!(outcome["ring_id"], intent["ring_id"]);
    assert_eq!(s.attempted[0].1, Via::Watchdog);
    assert_eq!(s.attention.keys().filter(|(k, _)| k == "ready-busy-while-silent").count(), 1);
}

/// E-7.4: a second fire the same day adds no second ready-busy entry (coalesced).
#[test]
fn e7_ready_busy_entry_coalesced_per_day() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut fa = silent_busy();
    for t in 0..40 {
        if t == 12 {
            fa.last_submit = Some((f.now(), vec![]));
        }
        db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        f.advance(30_000);
    }
    assert_eq!(db.h11.f_fires, 2, "a submit ended the first episode, so a second fire came");
    assert_eq!(f.s().attention.keys().filter(|(k, _)| k == "ready-busy-while-silent").count(), 1);
}

/// E-7.6 (F-1 ruling A): a client write after the silence was found and before the fire holds it.
#[test]
fn e7_6_typing_recheck_after_the_veto_holds_the_watchdog() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let fa = silent_busy();
    for _ in 0..10 {
        db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        f.advance(30_000);
    }
    let base = f.s().ia_calls;
    let now = f.now();
    // The second input_activity read of this fire (after the veto) sees a client write.
    f.s().fg_after = Some((base + 2, Some(Fg { client_last: Some(now), ..fg_ok() })));
    let rep = db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
    assert_eq!(rep.watchdog.map(|w| w.1), Some(RingOut::Held("typing".into())));
    assert_eq!(f.notifies(), 0);
    assert!(f.s().admin.iter().any(|r| r["outcome"] == "held:typing"));
}

/// E-7.8: two silent sessions due in one tick: one fire (larger last activity), the other waits.
#[test]
fn e7_8_one_fire_per_agent_per_tick() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut a = silent_busy();
    a.name = "sa".into();
    let mut b = silent_busy();
    b.name = "sb".into();
    b.acts = (Some(f.now() - 400_000), None, None);
    a.acts = (Some(f.now() - 500_000), None, None);
    let mut fires_per_tick = vec![];
    for _ in 0..12 {
        let before = f.notifies();
        db.tick(&f, &[a.clone(), b.clone()], &[msg(1)], &no_hand);
        fires_per_tick.push(f.notifies() - before);
        f.advance(30_000);
    }
    assert!(fires_per_tick.iter().all(|n| *n <= 1), "{fires_per_tick:?}");
    assert_eq!(f.s().notifies[0].0, "sb");
}

// ------------------------------------------------------------------ E-8

/// E-8: the watchdog's own checks hold it, one admin row per hold kind.
#[test]
fn e8_watchdog_checks_hold_with_rows() {
    let cases: Vec<(&str, Flip)> = vec![
        ("kill-switch", Box::new(|f: &Fake| f.s().switch = false)),
        ("foreground", Box::new(|f: &Fake| f.s().fg = Some(Fg { leader_pid: Some(5), exe: Some("/bin/bash".into()), script: None, client_last: None }))),
        ("veto:cpu", Box::new(|f: &Fake| f.s().veto = Some((Veto::Active("cpu".into()), false)))),
    ];
    for (check, flip) in cases {
        let f = Fake::new();
        flip(&f);
        let mut db = Doorbell::new();
        let fa = silent_busy();
        for _ in 0..15 {
            db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
            f.advance(30_000);
        }
        assert_eq!(f.notifies(), 0, "{check}");
        let rows = f.s().admin.iter().filter(|r| r["outcome"] == format!("held:{check}")).count();
        assert_eq!(rows, 1, "{check}: one row per hold kind per silence episode");
    }
}

// ------------------------------------------------------------------ E-9

/// E-9: arrivals keep coming; the watchdog still fires at 300 s.
#[test]
fn e9_inflow_is_not_activity() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let fa = silent_busy();
    let start = f.now();
    let mut pending = vec![msg(1)];
    let mut fired = None;
    for t in 0..12u64 {
        pending.push(msg(100 + t));
        let rep = db.tick(&f, std::slice::from_ref(&fa), &pending, &no_hand);
        if fired.is_none() && matches!(rep.watchdog, Some((_, RingOut::Done { .. }))) {
            fired = Some(f.now() - start);
        }
        f.advance(30_000);
    }
    assert_eq!(fired, Some(300_000));
}

/// E-9.6 on the real meter: a repeated `ready {BUSY}` does not move S2; session_start
/// carries the harness pid the binding uses.
#[test]
fn e9_6_repeated_ready_busy_does_not_move_s2_on_the_meter() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    let clock = Arc::new(AtomicU64::new(1_000));
    let c2 = clock.clone();
    let m = termlink_sidecar::meter::Meter::new("agent-x", Box::new(termlink_sidecar::meter::Unbound), Box::new(termlink_sidecar::meter::NoLedger))
        .with_clocks(Arc::new(move || c2.load(Ordering::SeqCst)), Arc::new(|| 1));
    let rep = |ev: &str, extra: Value| {
        let mut v = json!({"schema": "termlink.adapter/2", "event": ev, "agent_id": "agent-x", "session_id": "s1", "at_ms": 1});
        for (k, x) in extra.as_object().unwrap() {
            v[k] = x.clone();
        }
        m.intake(v.to_string().as_bytes())
    };
    rep("session_start", json!({"interactive": true, "harness_pid": 4321}));
    rep("ready", json!({"state": "BUSY"}));
    let s2 = m.doorbell_facts(1_000)[0].acts.1;
    for i in 1..30 {
        clock.store(1_000 + i * 10_000, Ordering::SeqCst);
        rep("ready", json!({"state": "BUSY"}));
    }
    let f = &m.doorbell_facts(clock.load(Ordering::SeqCst))[0];
    assert_eq!(f.acts.1, s2, "S2 does not move on a repeated ready with an unchanged state");
    assert_eq!(f.harness_pid, Some(4321));
}

// ------------------------------------------------------------------ E-10

/// E-10: S2 unknown means no watchdog, and status says so.
#[test]
fn e10_s2_unknown_watchdog_unavailable() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut fa = silent_busy();
    fa.s2_known = false;
    for _ in 0..20 {
        db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        f.advance(30_000);
    }
    assert_eq!(f.notifies(), 0);
    let st = db.status_view(true);
    assert_eq!(st["sessions"]["s"]["watchdog"], "unavailable (S2 unknown)");
}

// ------------------------------------------------------------------ E-11

/// E-11: safety and liveness of the watchdog over 10^4 seeded traces of 2 simulated hours.
#[test]
fn e11_watchdog_bound_and_liveness_property() {
    let mut seed: u64 = 0xb16_e11;
    let mut rnd = move |n: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % n
    };
    for trace in 0..10_000u64 {
        let f = Fake::new();
        let mut db = Doorbell::new();
        let mut fa = silent_busy();
        let start = f.now();
        let mut last_event = start; // the baseline: first seen
        let mut last_fire: Option<u64> = None;
        let mut pending = true;
        let mut stalled;
        let mut quiet_since: Option<u64> = None; // pending, switch on, unvetoed, not stalled
        for _ in 0..240 {
            // random events strictly between ticks
            let now = f.now();
            if rnd(10) == 0 {
                let at = now - rnd(29_000);
                fa.acts.1 = Some(at.max(fa.acts.1.unwrap_or(0)));
                last_event = last_event.max(at);
            }
            let switch = rnd(20) != 0;
            let veto = rnd(15) == 0;
            f.s().switch = switch;
            f.s().veto = Some((if veto { Veto::Active("cpu".into()) } else { Veto::Quiet }, false));
            if rnd(40) == 0 {
                pending = !pending;
            }
            stalled = db.sched("s").map(|s| s.state) == Some(RingState::Stalled);
            if stalled && rnd(8) == 0 {
                fa.last_submit = Some((now - 1, vec![]));
                last_event = last_event.max(now - 1);
                fa.acts.1 = Some(now - 1);
                let _ = &stalled; // the submit ends the episode: re-read next tick
            }
            let p = if pending { vec![msg(1)] } else { vec![] };
            let before = f.notifies();
            db.tick(&f, std::slice::from_ref(&fa), &p, &no_hand);
            let fired = f.notifies() > before;
            let since = [Some(last_event), last_fire].into_iter().flatten().max().unwrap();
            if fired {
                // safety: 300 s of no event and no earlier fire
                assert!(now - since >= 300_000, "trace {trace}: fired after {} ms of silence", now - since);
                last_fire = Some(now);
                quiet_since = None;
            }
            stalled = db.sched("s").map(|s| s.state) == Some(RingState::Stalled);
            let eligible = pending && switch && !veto && !stalled;
            if eligible && !fired {
                let q = *quiet_since.get_or_insert(now);
                // liveness: silent, eligible for 300 s + 1 tick => a fire happened
                assert!(!(now - since >= 330_000 && now - q >= 30_000), "trace {trace}: no fire after {} ms of silence", now - since);
            } else {
                quiet_since = None;
            }
            f.advance(30_000);
        }
    }
}

// ------------------------------------------------------------------ E-12

/// E-12 (tier 1): the switch off stops both typed paths within the tick.
#[test]
fn e12_kill_switch_stops_doorbell_and_watchdog() {
    let f = Fake::new();
    f.s().switch = false;
    let mut db = Doorbell::new();
    for fa in [facts("s"), silent_busy()] {
        for _ in 0..15 {
            db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
            f.advance(30_000);
        }
    }
    assert_eq!(f.notifies(), 0);
    // Switched on again: the normal rules apply.
    f.s().switch = true;
    let mut db = Doorbell::new();
    db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    assert_eq!(f.notifies(), 1);
}

// ------------------------------------------------------------------ E-14 (limit)

/// E-14: a turn starting after the last re-check still receives the line; counted by H-11 (d).
#[test]
fn e14_limit_check_to_keystroke_race() {
    let f = Fake::new();
    f.s().ready_busy_on_notify = true;
    let mut db = Doorbell::new();
    let mut fa = facts("s");
    db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
    assert_eq!(f.notifies(), 1, "the race is narrowed, not closed: the line arrives");
    fa.last_submit = Some((f.now() + 500, vec![]));
    f.advance(1_000);
    db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
    assert_eq!(db.h11.d_near_turn, 1, "and is counted as a ring within 2 s of a turn start");
    println!("LIMIT E-14: a turn starting between the last re-check and the keystrokes receives the line (RR-19, AR-2)");
}

// ------------------------------------------------------------------ E-121

/// E-121 (tier 1): the ring decision never reads the screen; READY BUSY never rings.
#[test]
fn e121_screen_is_never_an_input() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    let mut fa = facts("s");
    fa.ready = Some(ready("BUSY", 1));
    for _ in 0..9 {
        let rep = db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        assert_eq!(rep.decisions[0].1, "hold:not-ready");
        f.advance(30_000);
    }
    assert_eq!(f.notifies(), 0);
    let src = include_str!("../src/doorbell.rs");
    for needle in ["query.output", "command.output", "QUERY_OUTPUT", "scrollback"] {
        assert!(!src.contains(needle), "the ring path must not read output ({needle})");
    }
}

// ------------------------------------------------------------------ E-159

/// E-159: one ring per agent; no intent no keystroke; a notify timeout; a restart.
#[test]
fn e159_ring_durability() {
    // no intent, no keystroke (full disk)
    let f = Fake::new();
    f.s().fail_append = true;
    assert_eq!(ring_once(&f).0, RingOut::Held("intent-unrecorded".into()));
    assert_eq!(f.notifies(), 0);
    // the watchdog's admin row unwritable: no keystroke (FCW-42, E-160)
    let f = Fake::new();
    f.s().fail_admin = true;
    let (out, _) = type_ring(&f, "s", &harness(), &[hex(1)], 1, None, Via::Watchdog, json!({}));
    assert_eq!(out, RingOut::Held("intent-unrecorded".into()));
    assert_eq!(f.notifies(), 0);
    // timeout: unknown, ATTEMPTED, no re-ring until a submit
    let f = Fake::new();
    f.s().notify_result = Some(Notified::Unknown);
    let mut db = Doorbell::new();
    db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    assert_eq!(f.s().attempted.len(), 1);
    for _ in 0..8 {
        f.advance(30_000);
        db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    }
    assert_eq!(f.notifies(), 1, "unknown is treated as typed: no re-ring");
    // restart: rebuilt from the ring log
    let log = vec![
        json!({"phase": "intent", "ring_id": "r1", "session": "a", "via": "doorbell", "ids": [hex(1)], "t_last_check": 100}),
        json!({"phase": "intent", "ring_id": "r2", "session": "b", "via": "watchdog", "ids": [hex(1)], "t_last_check": 100}),
        json!({"phase": "outcome", "ring_id": "r2", "outcome": "typed"}),
        json!({"phase": "intent", "ring_id": "r3", "session": "c", "via": "doorbell", "ids": [hex(1)], "t_last_check": 100}),
        json!({"phase": "outcome", "ring_id": "r3", "outcome": "typed"}),
        json!({"phase": "intent", "ring_id": "r4", "session": "d", "via": "doorbell", "ids": [hex(1)], "t_last_check": 100}),
        json!({"phase": "outcome", "ring_id": "r4", "outcome": "failed(no pty)"}),
    ];
    let mut db = Doorbell::new();
    db.restore(&log, &|s| (s == "c").then_some(200));
    assert_eq!(db.sched("a").unwrap().state, RingState::Unsubmitted, "intent without outcome");
    assert_eq!(db.sched("b").unwrap().state, RingState::Stalled, "a watchdog line without a submit");
    assert_eq!(db.sched("c").unwrap().state, RingState::Idle, "submitted after the line");
    assert_eq!(db.sched("d").unwrap().state, RingState::Idle, "a failed notify typed nothing");
}

// ------------------------------------------------------------------ E-160

fn ps_rows(text: &str) -> Vec<PsRow> {
    parse_ps(text).unwrap()
}

/// E-160: an unmeasurable veto holds the watchdog (entry once per session per day); the
/// macOS interim `ps` veto; status `degraded (macOS, ps veto)`.
#[test]
fn e160_unmeasurable_veto_and_macos_ps_veto() {
    let f = Fake::new();
    f.s().veto = Some((Veto::Unmeasurable("no /proc, no ps".into()), false));
    let mut db = Doorbell::new();
    let fa = silent_busy();
    for _ in 0..20 {
        db.tick(&f, std::slice::from_ref(&fa), &[msg(1)], &no_hand);
        f.advance(30_000);
    }
    assert_eq!(f.notifies(), 0, "the watchdog never types blind");
    assert_eq!(f.s().attention.keys().filter(|(k, _)| k == "watchdog-unavailable").count(), 1);
    // The normal ring keeps OQ-16 D 3: unmeasurable is no veto there.
    let f2 = Fake::new();
    f2.s().veto = Some((Veto::Unmeasurable("x".into()), false));
    assert!(matches!(ring_once(&f2).0, RingOut::Done { .. }));

    // ps fixtures: (a) a child sleep, (b) a CPU loop, (c) an idle tree on a socket read.
    let a0 = "100 1 Ss 0:01.00 /opt/claude/claude\n";
    let child = "100 1 Ss 0:01.00 /opt/claude/claude\n200 100 S 0:00.00 sleep 600\n";
    assert_eq!(ps_judge(&ps_rows(a0), &ps_rows(child), 100, &[]), Veto::Active("child".into()));
    let cpu_b = "100 1 Rs 0:01.20 /opt/claude/claude\n";
    assert_eq!(ps_judge(&ps_rows(a0), &ps_rows(cpu_b), 100, &[]), Veto::Active("cpu".into()));
    let idle_b = "100 1 Ss 0:01.01 /opt/claude/claude\n";
    assert_eq!(ps_judge(&ps_rows(a0), &ps_rows(idle_b), 100, &[]), Veto::Quiet, "(c) LIMIT: a network/disk wait is not seen");
    // An idle registry helper does not hold; a running one does.
    let helper = "100 1 Ss 0:01.00 /opt/claude/claude\n300 100 S 0:00.00 node /mcp/server.js\n";
    assert_eq!(ps_judge(&ps_rows(a0), &ps_rows(helper), 100, &["node /mcp/".into()]), Veto::Quiet);
    let helper_r = "100 1 Ss 0:01.00 /opt/claude/claude\n300 100 R 0:00.00 node /mcp/server.js\n";
    assert_eq!(ps_judge(&ps_rows(a0), &ps_rows(helper_r), 100, &["node /mcp/".into()]), Veto::Active("child".into()));
    assert!(matches!(ps_judge(&ps_rows(a0), &ps_rows(a0), 999, &[]), Veto::Unmeasurable(_)));
    assert!(parse_ps("garbage").is_none());

    // Status: the ps veto measured -> degraded (macOS, ps veto).
    let f = Fake::new();
    f.s().veto = Some((Veto::Quiet, true));
    let mut db = Doorbell::new();
    db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    assert_eq!(db.status_view(true)["sessions"]["s"]["watchdog"], DEGRADED_MACOS);
}

/// E-160 tier 2 on this host: the `ps` veto against a real process tree (a running child).
#[test]
fn e160_ps_veto_on_a_real_tree() {
    let mut sh = std::process::Command::new("sh").args(["-c", "sleep 30 & wait"]).spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    let v = measure_ps(sh.id(), &[]);
    let _ = sh.kill();
    let _ = sh.wait();
    assert_eq!(v, Veto::Active("child".into()));
    let mut idle = std::process::Command::new("sleep").arg("30").spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let v = measure_ps(idle.id(), &[]);
    let _ = idle.kill();
    let _ = idle.wait();
    assert_eq!(v, Veto::Quiet, "an idle tree with no child and no CPU is quiet");
}

// ------------------------------------------------------------------ status lines (PD-1)

#[test]
fn status_lines_switch_and_watchdog() {
    let f = Fake::new();
    let mut db = Doorbell::new();
    db.tick(&f, &[facts("s")], &[msg(1)], &no_hand);
    let v = db.status_view(true);
    assert_eq!(v["switch"], "on");
    assert_eq!(v["sessions"]["s"]["watchdog"], "available");
    assert_eq!(v["sessions"]["s"]["state"], "await-submit");
    assert!(v["h11"]["e_unsubmitted"]["of_rings"].is_u64());
    assert_eq!(db.status_view(false)["sessions"]["s"]["watchdog"], "off (doorbell switched off)");
}
