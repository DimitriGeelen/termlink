//! B10 tests for C-11 (hook.rs, daemon.rs) and the sender labels (FC-5, FC-6, FC-7).
//! Hermetic: testkit estate only. `hooks_scripted_demo` is also what
//! `scripts/arc011-ph1-demo.sh --scripted` runs (R-69: the demo is the test).

use serde_json::{Value, json};
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use termlink_sidecar::api::{self, Scope, STUCK_AFTER_MS};
use termlink_sidecar::daemon::Config;
use termlink_sidecar::deliver::{Deliverer, READY_MAX_AGE_MS};
use termlink_sidecar::hook;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::store::{LineExtra, Store};
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");

fn wait_for(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

// ---- FC-5: ready flag ---------------------------------------------------------

#[test]
fn hooks_ready_flag_reads_not_ready_unless_present_parseable_and_fresh() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let d = Deliverer::new("agent-b", store.clone(), Box::new(BuiltIn::new(Roster::new())), None);
    let now = 10_000_000_000u64;
    assert!(!d.is_ready("s-missing", now, READY_MAX_AGE_MS), "missing");
    store.write_state("ready-s-empty", "").unwrap();
    assert!(!d.is_ready("s-empty", now, READY_MAX_AGE_MS), "empty");
    store.write_state("ready-s-garbage", "yes please").unwrap();
    assert!(!d.is_ready("s-garbage", now, READY_MAX_AGE_MS), "unparseable");
    std::fs::create_dir(store.dir().join("state.ready-s-dir")).unwrap();
    assert!(!d.is_ready("s-dir", now, READY_MAX_AGE_MS), "unreadable");
    d.mark_ready("s-old", now - READY_MAX_AGE_MS - 1).unwrap();
    assert!(!d.is_ready("s-old", now, READY_MAX_AGE_MS), "stale");
    d.mark_ready("s-future", now + 60_000).unwrap();
    assert!(!d.is_ready("s-future", now, READY_MAX_AGE_MS), "from the future");
    d.mark_ready("s-ok", now - 1000).unwrap();
    assert!(d.is_ready("s-ok", now, READY_MAX_AGE_MS), "fresh");
}

// ---- FC-6, FC-7: sender labels ------------------------------------------------

#[test]
fn hooks_sender_sees_waiting_for_recipient_then_stuck_never_delivered() {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-a").unwrap();
    let (me, peer) = ("fp-a", "fp-b");
    s.append_event("c", "SENT", "m1", me, "d").unwrap();
    s.append_event("c", "POSTED", "m1", me, "d").unwrap();
    let stored_at = 5_000_000u64;
    for st in ["RECEIVED", "STORED"] {
        let extra = LineExtra { peer_ts_ms: Some(stored_at), ..Default::default() };
        s.append_event_unless("c", st, "m1", peer, "d", extra, |_| false).unwrap();
    }
    let view = |now: u64| {
        let v = api::status_view(&json!({"op": "status", "msg_id": "m1"}), &s, "agent-a", None, now, STUCK_AFTER_MS);
        v["messages"][0].clone()
    };
    // B is idle or has no hook: storage is not delivery (FC-7).
    let w = view(stored_at + 60_000);
    assert_eq!((w["stage"].clone(), w["label"].clone()), (json!("STORED"), json!("WAITING FOR RECIPIENT")), "{w}");
    assert!(w.get("delivered").is_none());
    // Past the deadline on a simulated clock: STUCK (FC-6).
    assert_eq!(view(stored_at + STUCK_AFTER_MS + 1)["label"], "STUCK");
    // Handed over: no waiting label any more.
    let extra = LineExtra { peer_ts_ms: Some(stored_at + 10), ..Default::default() };
    s.append_event_unless("c", "HANDED_OVER", "m1", peer, "d", extra, |_| false).unwrap();
    let h = view(stored_at + STUCK_AFTER_MS + 1);
    assert_eq!((h["stage"].clone(), h["delivered"].clone()), (json!("HANDED_OVER"), json!(true)), "{h}");
    assert!(h.get("label").is_none(), "{h}");
}

// ---- fail-open turn -----------------------------------------------------------

fn run_hook(e: &Estate, event: &str, socket: &Path, input: &Value, errlog: &Path) -> (i32, String) {
    let mut c = e
        .command(BIN)
        .args(["hook", event])
        .env(hook::SOCKET_ENV, socket)
        .env(hook::ERRLOG_ENV, errlog)
        .env(hook::FINALIZE_SECS_ENV, "20")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = c.wait_with_output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn hooks_turn_fails_open_with_a_log_line_and_no_output() {
    let e = Estate::create_without_hub().unwrap();
    let log = e.root().join("hook-errors.log");
    let input = json!({"session_id": "s1", "tool_use_id": "toolu_1", "hook_event_name": "PostToolUse"});
    // 1 No sidecar listening.
    let (rc, out) = run_hook(&e, "post-tool-use", &e.runtime_dir().join("nothing.sock"), &input, &log);
    assert_eq!((rc, out.as_str()), (0, ""));
    // 2 A sidecar that accepts and never answers: bounded by the hook timeout.
    let mute = e.runtime_dir().join("mute.sock");
    let l = UnixListener::bind(&mute).unwrap();
    let hold = std::thread::spawn(move || {
        let (s, _) = l.accept().unwrap();
        std::thread::sleep(Duration::from_secs(6));
        drop(s);
    });
    let t0 = Instant::now();
    let (rc, out) = run_hook(&e, "post-tool-use", &mute, &input, &log);
    assert_eq!((rc, out.as_str()), (0, ""));
    assert!(t0.elapsed() < Duration::from_secs(6), "the hook waited past its bound");
    hold.join().unwrap();
    // 3 Input that is not JSON, and an unknown event.
    let r = hook::run("post-tool-use", "not json", Some(&mute), Some(&log));
    assert!(r.stdout.is_none() && r.error.is_some());
    let r = hook::run("pre-tool-use", "{\"session_id\":\"s\"}", Some(&mute), Some(&log));
    assert!(r.stdout.is_none() && r.error.is_some());
    let lines = std::fs::read_to_string(&log).unwrap();
    assert_eq!(lines.lines().count(), 4, "{lines}");
    assert!(lines.contains("connect") && lines.contains("not JSON") && lines.contains("unknown hook event"), "{lines}");
}

// ---- the scripted demo: two real sidecar processes ------------------------------

struct Proc {
    child: Child,
}

impl Drop for Proc {
    fn drop(&mut self) {
        // SAFETY: plain kill(2) on our own child.
        unsafe { libc::kill(self.child.id() as i32, libc::SIGTERM) };
        let _ = self.child.wait();
    }
}

fn start_sidecar(e: &Estate, who: &str, name: &str, inst: &str, hub_id: &str) -> Proc {
    let roster: Vec<String> = ["a", "b"].iter().map(|w| e.identity(w).unwrap().public_key_hex()).collect();
    let cfg = Config {
        agent: name.into(),
        instance: inst.into(),
        hub: e.hub_addr(),
        hub_id: hub_id.into(),
        key_file: e.root().join("keys").join(format!("{who}.key")),
        roster,
        queue_path: e.root().join(format!("outbox-{who}.sqlite")),
        poll_ms: 200,
        timeout_ms: 15_000,
        claude_projects: None,
    };
    let path = e.root().join(format!("sidecar-{who}.json"));
    std::fs::write(&path, serde_json::to_string(&cfg).unwrap()).unwrap();
    let log = std::fs::File::create(e.root().join(format!("sidecar-{who}.log"))).unwrap();
    let child = e
        .command(BIN)
        .args(["start", "--config"])
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .unwrap();
    let sock = api::socket_path(&e.runtime_dir(), name, Scope::Send);
    assert!(wait_for(30, || sock.exists()), "sidecar {name} did not come up");
    Proc { child }
}

fn sock(e: &Estate, name: &str, scope: Scope) -> PathBuf {
    api::socket_path(&e.runtime_dir(), name, scope)
}

fn status(e: &Estate, name: &str, msg_id: &str) -> Value {
    hook::call(&sock(e, name, Scope::Status), &json!({"op": "status", "msg_id": msg_id}), Duration::from_secs(10)).unwrap()
}

#[test]
fn hooks_scripted_demo() {
    let e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();
    let a = start_sidecar(&e, "a", "agent-a", "inst-a", &hub_id);
    let b = start_sidecar(&e, "b", "agent-b", "inst-b", &hub_id);

    // 1 A sends through its own socket.
    let sent = hook::call(
        &sock(&e, "agent-a", Scope::Send),
        &json!({"op": "send", "to": "agent-b", "to_instance": "inst-b", "conv": "demo", "body": "hello from A"}),
        Duration::from_secs(20),
    )
    .unwrap();
    assert_eq!(sent["stage"], "SENT", "{sent}");
    let id = sent["msg_id"].as_str().unwrap().to_owned();

    // 2 B's sidecar receives and stores it; A sees WAITING FOR RECIPIENT.
    assert!(wait_for(30, || status(&e, "agent-a", &id)["messages"][0]["stage"] == "STORED"), "{}", status(&e, "agent-a", &id));
    assert_eq!(status(&e, "agent-a", &id)["messages"][0]["label"], "WAITING FOR RECIPIENT");

    // 3 B's agent starts a session and ends a tool call: the hooks run.
    let errlog = e.root().join("hook-errors.log");
    let session = "5e55f00d-0000-4000-8000-00000000demo";
    let send_b = sock(&e, "agent-b", Scope::Send);
    let (rc, out) = run_hook(&e, "session-start", &send_b, &json!({"session_id": session}), &errlog);
    assert_eq!((rc, out.as_str()), (0, ""));
    let (rc, out) = run_hook(&e, "stop", &send_b, &json!({"session_id": session}), &errlog);
    assert_eq!((rc, out.as_str()), (0, ""), "Stop never pulls");
    let tool = "toolu_demo_1";
    let (rc, out) = run_hook(
        &e,
        "post-tool-use",
        &send_b,
        &json!({"session_id": session, "tool_use_id": tool, "hook_event_name": "PostToolUse", "tool_name": "Bash"}),
        &errlog,
    );
    assert_eq!(rc, 0);
    let printed: Value = serde_json::from_str(out.trim()).unwrap_or_else(|x| panic!("hook printed {out:?}: {x}"));
    assert_eq!(printed["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    let frame = printed["hookSpecificOutput"]["additionalContext"].as_str().unwrap().to_owned();
    assert!(frame.contains("hello from A") && frame.starts_with("ARC011-PEER-MAIL nonce="), "{frame}");

    // 4 The harness writes it into B's transcript (scripted: the record shape of
    //   the B1 spike), under B's own HOME where the sidecar looks for it.
    let proj = e.home().join(".claude").join("projects").join("-estate-proj-b");
    std::fs::create_dir_all(&proj).unwrap();
    let recs = [
        json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "tool_use", "id": tool, "name": "Bash", "input": {}}]}}),
        json!({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": tool, "content": "ok"}]}}),
        json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [frame], "hookName": "PostToolUse:Bash", "toolUseID": tool, "hookEvent": "PostToolUse"}}),
    ];
    let text: String = recs.iter().map(|r| format!("{r}\n")).collect();
    std::fs::write(proj.join(format!("{session}.jsonl")), text).unwrap();

    // 5 The detached finalizer finds the harness record; A sees HANDED_OVER.
    assert!(
        wait_for(40, || status(&e, "agent-a", &id)["messages"][0]["stage"] == "HANDED_OVER"),
        "sender never saw HANDED_OVER: {}\nb: {}\nhook errors: {}",
        status(&e, "agent-a", &id),
        status(&e, "agent-b", &id),
        std::fs::read_to_string(&errlog).unwrap_or_default()
    );
    // Each sidecar's store is where the estate says it is (not the estate root).
    for name in ["agent-a", "agent-b"] {
        assert!(e.store_root().join("sidecar").join(name).join("lock").is_file(), "{name} store misplaced");
        assert!(!e.root().join("sidecar").join(name).exists(), "{name} store at the estate root");
    }
    let bv = status(&e, "agent-b", &id);
    let stages: Vec<&str> = bv["rows"].as_array().unwrap().iter().filter_map(|r| r["stage"].as_str()).collect();
    assert_eq!(stages, ["RECEIVED", "STORED", "CLAIMED", "HANDED_OVER"], "{bv}");
    // A second tool call hands nothing out again.
    let (rc, out) = run_hook(&e, "post-tool-use", &send_b, &json!({"session_id": session, "tool_use_id": "toolu_demo_2"}), &errlog);
    assert_eq!((rc, out.as_str()), (0, ""));
    assert!(std::fs::read_to_string(&errlog).unwrap_or_default().is_empty(), "no hook errors expected");

    // 6 Both sidecars stop cleanly on SIGTERM.
    for (p, name) in [(a, "agent-a"), (b, "agent-b")] {
        let mut p = p;
        // SAFETY: plain kill(2) on our own child.
        unsafe { libc::kill(p.child.id() as i32, libc::SIGTERM) };
        let st = p.child.wait().unwrap();
        assert_eq!(st.code(), Some(0), "{name} did not stop cleanly");
        std::mem::forget(p);
    }
    println!("arc011-ph1 scripted demo: SENT -> RECEIVED -> STORED -> CLAIMED -> HANDED_OVER, both sidecars stopped cleanly");
}
