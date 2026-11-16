//! T-3810: `channel post --ensure-topic` must not call `channel.create` for a topic that exists.
//! Field case (2026-11-16): a /be-reachable heartbeat on greenfield posted every 30 s with
//! --ensure-topic through a scoped Interact token; every post first tried `channel.create`
//! (needs Control), was refused, and the .107 hub logged ~2,880 WARN lines a day.
//!
//! Hermetic: a throwaway hub on a unix socket in a temp runtime dir; the hub's own
//! `rpc-audit.jsonl` (one line per method) is the counter.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path).env("TERMLINK_IDENTITY_DIR", dir.path.join("id")).args(args).stdin(Stdio::null());
    c
}

fn creates(dir: &TestDir) -> usize {
    std::fs::read_to_string(dir.path.join("rpc-audit.jsonl"))
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains("\"method\":\"channel.create\""))
        .count()
}

fn wait_for_creates(dir: &TestDir, want: usize) -> usize {
    // the audit write is queued (best-effort); give it a moment to land
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let n = creates(dir);
        if n >= want || Instant::now() > end {
            return n;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn ensure_topic_post_to_an_existing_topic_calls_no_create() {
    let dir = TestDir::new("t3810");
    let hub = cmd(&dir, &["hub", "start"]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
    let _hub = ProcessGuard::new(hub, "t3810-hub");
    let deadline = Instant::now() + Duration::from_secs(20);
    while !dir.path.join("hub.sock").exists() {
        assert!(Instant::now() < deadline, "hub socket never appeared");
        std::thread::sleep(Duration::from_millis(50));
    }

    // an absent topic IS still created by --ensure-topic (behaviour kept)
    let out = cmd(&dir, &["channel", "post", "agent-presence-t3810", "--ensure-topic", "--payload", "hb"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let after_first = wait_for_creates(&dir, 1);
    assert_eq!(after_first, 1, "the first post creates the missing topic");

    // three more posts to the now-existing topic: no further create
    for _ in 0..3 {
        let out = cmd(&dir, &["channel", "post", "agent-presence-t3810", "--ensure-topic", "--payload", "hb"]).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    }
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(creates(&dir), 1, "an existing topic is never re-created");
}
