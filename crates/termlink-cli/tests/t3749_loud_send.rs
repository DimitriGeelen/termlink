//! T-3749: a send that did not reach a hub must fail loudly at the sender.
//!
//! Field case (ring20 T-2657, 2026-11-12): a message from the operator's Mac to
//! the ring20 management agent never arrived and the sender saw no error. Two of
//! the silent shapes, reproduced on .107 before the fix:
//!   1. no reachable hub: `channel post` queued the message in the local offline
//!      queue, printed "Queued to ..." and exited 0;
//!   2. an unusable `--hub` (`tcp://host:port`, a typo'd profile name, a bad
//!      port) fell through to a relative unix path, then the same queue, exit 0.
//!
//! Every case runs against a temp runtime dir, HOME and identity dir; the only
//! hub is a throwaway one on a unix socket inside the temp dir.

use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path)
        .env("TERMLINK_IDENTITY_DIR", dir.path.join("id"))
        .args(args)
        .stdin(Stdio::null());
    c
}

fn run(dir: &TestDir, args: &[&str]) -> (Output, String, String) {
    let out = cmd(dir, args).output().expect("run termlink");
    let so = String::from_utf8_lossy(&out.stdout).to_string();
    let se = String::from_utf8_lossy(&out.stderr).to_string();
    (out, so, se)
}

#[test]
fn post_with_no_reachable_hub_exits_3_and_says_not_delivered() {
    let dir = TestDir::new("t3749-queued");
    let (out, so, se) = run(&dir, &["channel", "post", "inbox:t3749", "--payload", "hi"]);
    assert_eq!(out.status.code(), Some(3), "stdout={so} stderr={se}");
    assert!(so.contains("Queued to inbox:t3749"), "{so}");
    assert!(se.contains("NOT DELIVERED"), "{se}");

    let (out, so, se) =
        run(&dir, &["channel", "post", "inbox:t3749", "--payload", "hi", "--json"]);
    assert_eq!(out.status.code(), Some(3), "stdout={so} stderr={se}");
    let v: serde_json::Value = serde_json::from_str(&so).expect("json");
    assert_eq!(v["delivered"], false);
    assert_eq!(v["ok"], false);
}

#[test]
fn post_with_unusable_hub_arg_fails_with_named_reason_and_queues_nothing() {
    let dir = TestDir::new("t3749-badhub");
    for (hub, reason) in [
        ("tcp://127.0.0.1:9100", "wrong scheme"),
        ("ring20-managment", "no profile"),
        ("somehost:notaport", "not a valid port"),
    ] {
        let (out, so, se) =
            run(&dir, &["channel", "post", "inbox:t3749", "--payload", "hi", "--hub", hub]);
        assert_eq!(out.status.code(), Some(1), "{hub}: stdout={so} stderr={se}");
        assert!(se.contains(reason), "{hub}: {se}");
        assert!(!so.contains("Queued"), "{hub}: must not queue: {so}");
    }
}

/// Positive control: with a live hub the same post exits 0 and says Posted, so
/// the two tests above are measuring the failure, not a broken command.
#[test]
fn post_with_live_hub_exits_0() {
    let dir = TestDir::new("t3749-live");
    let hub = cmd(&dir, &["hub", "start"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn hub");
    let _hub = ProcessGuard::new(hub, "t3749-hub");
    let sock = dir.path.join("hub.sock");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sock.exists() {
        assert!(Instant::now() < deadline, "hub socket never appeared");
        std::thread::sleep(Duration::from_millis(50));
    }
    let sock = sock.to_string_lossy().to_string();
    let (out, so, se) = run(
        &dir,
        &["channel", "post", "inbox:t3749", "--payload", "hi", "--ensure-topic", "--hub", &sock],
    );
    assert_eq!(out.status.code(), Some(0), "stdout={so} stderr={se}");
    assert!(so.contains("Posted to inbox:t3749"), "{so}");
    assert!(!se.contains("NOT DELIVERED"), "{se}");
}
