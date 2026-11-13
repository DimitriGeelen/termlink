//! T-3757 step 1: every CLI founding-verb caller declares `from`, end to end.
//!
//! The session writes a use-log row (T-3702) for every `command.inject` /
//! `command.execute`; these tests read that row and check `caller.from` is the
//! identity the CLI declared. The owner-only tests are the control: a session
//! started with `TERMLINK_OWNER_ONLY=<owner>` accepts the owner's stamped call
//! and refuses another declared caller, which only works if `from` really
//! reaches the session.

use std::process::{Command, Stdio};
use std::time::Duration;

use assert_cmd::cargo;
use termlink_test_utils::{wait_for_socket, ProcessGuard, TestDir};

fn termlink_cmd(runtime_dir: &std::path::Path) -> Command {
    termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), runtime_dir)
}

fn start(runtime_dir: &std::path::Path, name: &str, shell: bool, owner: Option<&str>) -> ProcessGuard {
    let mut c = termlink_cmd(runtime_dir);
    c.args(["register", "--name", name]);
    if shell {
        c.arg("--shell");
    }
    if let Some(o) = owner {
        c.env("TERMLINK_OWNER_ONLY", o);
    }
    let child = c.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("spawn register");
    ProcessGuard::new(child, name)
}

/// Every use-log row under `<sessions>/use-log/`.
fn rows(dir: &TestDir) -> Vec<serde_json::Value> {
    let d = dir.sessions_dir().join("use-log");
    let mut out = Vec::new();
    for e in std::fs::read_dir(&d).unwrap_or_else(|e| panic!("no use-log dir {}: {e}", d.display())).flatten() {
        for l in std::fs::read_to_string(e.path()).unwrap().lines() {
            out.push(serde_json::from_str(l).unwrap());
        }
    }
    out
}

fn froms(dir: &TestDir, verb: &str) -> Vec<String> {
    rows(dir)
        .into_iter()
        .filter(|r| r["verb"] == verb)
        .map(|r| r["caller"]["from"].as_str().unwrap_or("<none>").to_string())
        .collect()
}

#[test]
fn t3757_exec_declares_termlink_from() {
    let dir = TestDir::new("t3757-exec");
    let _g = start(&dir.path, "w", false, None);
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(10)).unwrap();
    let out = termlink_cmd(&dir.path).env("TERMLINK_FROM", "agent-x").args(["exec", "w", "echo hi"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(froms(&dir, "command.execute"), vec!["agent-x"]);
}

#[test]
fn t3757_pty_inject_and_inject_declare_agent_id() {
    let dir = TestDir::new("t3757-inject");
    let _g = start(&dir.path, "sh", true, None);
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(10)).unwrap();
    termlink_test_utils::wait_for_data_socket(&dir.sessions_dir(), Duration::from_secs(10)).unwrap();
    for args in [&["pty", "inject", "sh", "true", "--enter"][..], &["inject", "sh", "true", "--enter"][..]] {
        let out = termlink_cmd(&dir.path).env("TERMLINK_AGENT_ID", "agent-y").args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }
    assert_eq!(froms(&dir, "command.inject"), vec!["agent-y", "agent-y"]);
}

#[test]
fn t3757_no_identity_still_names_the_user() {
    let dir = TestDir::new("t3757-anon");
    let _g = start(&dir.path, "w", false, None);
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(10)).unwrap();
    let out = termlink_cmd(&dir.path).env("USER", "opx").args(["exec", "w", "true"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(froms(&dir, "command.execute"), vec!["user:opx"], "no row without a named caller");
}

#[test]
fn t3757_owner_only_owner_accepted_other_refused() {
    let dir = TestDir::new("t3757-owner");
    let _g = start(&dir.path, "w", false, Some("owner-a"));
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(10)).unwrap();
    let ok = termlink_cmd(&dir.path).env("TERMLINK_AGENT_ID", "owner-a").args(["exec", "w", "echo owner-ran"]).output().unwrap();
    assert!(ok.status.success(), "owner refused: {}", String::from_utf8_lossy(&ok.stderr));
    assert!(String::from_utf8_lossy(&ok.stdout).contains("owner-ran"));
    // Negative control: another declared caller is refused and logged as refused.
    let bad = termlink_cmd(&dir.path).env("TERMLINK_AGENT_ID", "intruder").args(["exec", "w", "echo no"]).output().unwrap();
    assert!(!bad.status.success(), "another caller was let through");
    let r: Vec<(String, String)> = rows(&dir)
        .into_iter()
        .map(|r| (r["caller"]["from"].as_str().unwrap_or("").into(), r["decision"].as_str().unwrap_or("").into()))
        .collect();
    assert_eq!(r, vec![("owner-a".into(), "allowed".into()), ("intruder".into(), "refused-owner".into())]);
}
