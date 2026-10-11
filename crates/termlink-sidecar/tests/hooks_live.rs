//! B10 live check (run on demand, not in CI): one real `claude -p` session with
//! the PostToolUse hook pointed at a sidecar in a throwaway estate.
//!
//!   cargo test -p termlink-sidecar --test hooks_live -- --ignored --nocapture
//!
//! Like the B1 spike: Claude Code runs with the operator's own login and HOME
//! (the estate HOME has no Claude credential; provisioning one for `testdev` is
//! OQ-3, B14), in the estate's disposable project dir, with no project or user
//! settings (`--setting-sources ""`) and the hook given by `--settings`. Its
//! transcript lands in `~/.claude/projects/<encoded estate dir>/`; that one
//! directory, created by this run, is deleted at the end. Nothing else outside
//! the estate is written.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::daemon::Config;
use termlink_sidecar::hook;
use termlink_sidecar::testkit::Estate;

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");

struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        // SAFETY: plain kill(2) on our own child.
        unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) };
        let _ = self.0.wait();
    }
}

fn wait_for(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

fn start(e: &Estate, who: &str, name: &str, inst: &str, hub_id: &str) -> Proc {
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
    assert!(wait_for(30, || api::socket_path(&e.runtime_dir(), name, Scope::Send).exists()));
    Proc(child)
}

fn status(e: &Estate, name: &str, id: &str) -> Value {
    hook::call(&api::socket_path(&e.runtime_dir(), name, Scope::Status), &json!({"op": "status", "msg_id": id}), Duration::from_secs(10))
        .unwrap()
}

/// Claude Code's project directory name for a cwd: every non-alphanumeric byte becomes '-'.
fn encode(p: &std::path::Path) -> String {
    p.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect()
}

#[test]
#[ignore = "runs a real claude -p (operator login, one haiku call); run on demand"]
fn hooks_live_claude_session_gets_the_frame_and_handed_over_is_recorded() {
    let e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();
    let _a = start(&e, "a", "agent-a", "inst-a", &hub_id);
    let _b = start(&e, "b", "agent-b", "inst-b", &hub_id);
    let sent = hook::call(
        &api::socket_path(&e.runtime_dir(), "agent-a", Scope::Send),
        &json!({"op": "send", "to": "agent-b", "to_instance": "inst-b", "conv": "live", "body": "B10 live check: a peer message for agent-b."}),
        Duration::from_secs(20),
    )
    .unwrap();
    let id = sent["msg_id"].as_str().unwrap().to_owned();
    assert!(wait_for(30, || status(&e, "agent-a", &id)["messages"][0]["stage"] == "STORED"));

    // The session id is chosen up front, so the sidecar is told where the
    // transcript will be before Claude starts (the estate HOME is not Claude's).
    let session = {
        let h = termlink_sidecar::wire::mint_msg_id();
        format!("{}-{}-4{}-8{}-{}", &h[0..8], &h[8..12], &h[13..16], &h[17..20], &h[20..32])
    };
    let cwd = e.root().join("proj-b");
    let real_home = PathBuf::from(std::env::var("HOME").expect("HOME"));
    let proj_dir = real_home.join(".claude").join("projects").join(encode(&cwd));
    let transcript = proj_dir.join(format!("{session}.jsonl"));
    let store_b = e.store_root().join("sidecar").join("agent-b");
    let tmp = store_b.join(format!("state.session-{session}.tmp"));
    std::fs::write(&tmp, transcript.to_string_lossy().as_bytes()).unwrap();
    std::fs::rename(&tmp, store_b.join(format!("state.session-{session}"))).unwrap();

    let errlog = e.root().join("hook-errors.log");
    let sock = api::socket_path(&e.runtime_dir(), "agent-b", Scope::Send);
    let cmd = format!(
        "{}='{}' {}='{}' {}=60 '{}' hook post-tool-use",
        hook::SOCKET_ENV,
        sock.display(),
        hook::ERRLOG_ENV,
        errlog.display(),
        hook::FINALIZE_SECS_ENV,
        BIN
    );
    let settings = json!({"hooks": {"PostToolUse": [{"matcher": "*", "hooks": [{"type": "command", "command": cmd}]}]}});
    let settings_path = e.root().join("claude-settings.json");
    std::fs::write(&settings_path, settings.to_string()).unwrap();

    let out = Command::new("claude")
        .args(["-p", "Run the shell command 'echo hi' with the Bash tool, then reply with exactly: done"])
        .args(["--model", "claude-haiku-5-5", "--setting-sources", "", "--session-id", &session])
        .arg("--settings")
        .arg(&settings_path)
        .args(["--allowedTools", "Bash(echo hi)"])
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .output()
        .expect("claude on PATH");
    println!("claude rc={:?} stdout={:?}", out.status.code(), String::from_utf8_lossy(&out.stdout).trim());

    let handed = wait_for(90, || status(&e, "agent-b", &id)["messages"][0]["stage"] == "HANDED_OVER");
    let seen_by_a = handed && wait_for(30, || status(&e, "agent-a", &id)["messages"][0]["stage"] == "HANDED_OVER");

    // Evidence summary from the real transcript.
    let text = std::fs::read_to_string(&transcript).unwrap_or_default();
    let mut kinds = vec![];
    for l in text.lines() {
        let v: Value = serde_json::from_str(l).unwrap_or(Value::Null);
        let k = match v["type"].as_str() {
            Some("attachment") => format!("attachment/{}", v["attachment"]["type"].as_str().unwrap_or("?")),
            Some(t) => t.to_owned(),
            None => "?".into(),
        };
        if l.contains("ARC011-PEER-MAIL") {
            kinds.push(format!("{k} <- carries the frame header"));
        } else {
            kinds.push(k);
        }
    }
    println!("transcript records: {kinds:#?}");
    println!("agent-b status: {}", status(&e, "agent-b", &id));
    println!("agent-a status: {}", status(&e, "agent-a", &id));
    println!("hook errors: {:?}", std::fs::read_to_string(&errlog).unwrap_or_default());

    // Hygiene: remove the one project dir this run created under the real HOME.
    if proj_dir.starts_with(real_home.join(".claude").join("projects")) && encode(&cwd).contains("-tl-estate-") {
        let _ = std::fs::remove_dir_all(&proj_dir);
    }
    assert!(text.contains("ARC011-PEER-MAIL"), "the frame never reached the transcript");
    assert!(
        kinds.iter().any(|k| k == "attachment/hook_additional_context <- carries the frame header"),
        "the frame is not in a hook_additional_context record"
    );
    assert!(handed, "agent-b never recorded HANDED_OVER");
    assert!(seen_by_a, "agent-a never saw HANDED_OVER");
}
