//! T-3805: `agent inbox` for an identity with no local cursors must not look like
//! "nothing unread". Found 2026-11-16: this session's per-agent identity printed `[]`
//! while `channel unread` reported 999 unread for it.
//!
//! Hermetic: a fresh HOME and runtime dir; the empty-cursor path returns before any hub call.

use std::process::Command;

#[test]
fn inbox_without_cursors_says_untracked_not_empty() {
    let dir = std::env::temp_dir().join(format!("t3805-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let run = |json: bool| {
        let mut args = vec!["agent", "inbox"];
        if json {
            args.push("--json");
        }
        Command::new(env!("CARGO_BIN_EXE_termlink"))
            .args(&args)
            .env("HOME", &dir)
            .env("TERMLINK_RUNTIME_DIR", &dir)
            .env_remove("TERMLINK_AGENT_ID")
            .output()
            .expect("run termlink")
    };

    let out = run(true);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stdout.trim(), "[]", "JSON shape stays an array: {stdout}");
    assert!(stderr.contains("no topics tracked for identity"), "stderr must say untracked: {stderr}");
    assert!(stderr.contains("not \"nothing unread\""), "{stderr}");

    let out = run(false);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("no topics tracked for identity"), "text says untracked: {stdout}");

    let _ = std::fs::remove_dir_all(&dir);
}
