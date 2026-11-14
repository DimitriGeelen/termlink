//! T-3788 (T-3786 GO A): a non-root user with a hub link and no local hub socket uses
//! the link's hub over TCP, with its own credential for that address, for channel
//! commands given no `--hub` (the mail hook's inbox read and `channel post`).
//! Controls: no link keeps the old outcome; a half link is an error; root never falls back.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use termlink_test_utils::{ProcessGuard, TestDir};

fn bin() -> &'static Path {
    cargo::cargo_bin!("termlink")
}

/// A command run as the hub's owner (runtime dir and HOME = the hub dir).
fn hub_cmd(hub: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(bin(), &hub.path);
    c.env("HOME", &hub.path).env("TERMLINK_IDENTITY_DIR", hub.path.join("id")).args(args).stdin(Stdio::null());
    c
}

/// A command run as the linked user (own HOME, own runtime dir, "non-root").
fn user_cmd(user: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(bin(), &user.path.join("rt"));
    c.env("HOME", &user.path)
        .env("TERMLINK_IDENTITY_DIR", user.path.join("id"))
        .env("TERMLINK_TEST_TREAT_AS_NON_ROOT", "1")
        .args(args)
        .stdin(Stdio::null());
    c
}

fn out(mut c: Command) -> (i32, String, String) {
    let o = c.output().expect("run termlink");
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into())
}

struct Setup {
    hub: TestDir,
    user: TestDir,
    addr: String,
    _hub_proc: ProcessGuard,
}

/// A TCP hub, and a user whose `~/.termlink` holds the link (claim principal `tester`,
/// prefix `t-`). With `link = false` the user has no link files.
fn setup(tag: &str, link: bool) -> Setup {
    let hub = TestDir::new(&format!("t3788-{tag}-hub"));
    let user = TestDir::new(&format!("t3788-{tag}-user"));
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{port}");
    let p = hub_cmd(&hub, &["hub", "start", "--tcp", &addr]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("hub");
    let guard = ProcessGuard::new(p, "t3788-hub");
    let deadline = Instant::now() + Duration::from_secs(15);
    while !hub.path.join("hub.secret").exists() || std::net::TcpStream::connect(&addr).is_err() {
        assert!(Instant::now() < deadline, "hub never came up");
        std::thread::sleep(Duration::from_millis(50));
    }
    let dot = user.path.join(".termlink");
    std::fs::create_dir_all(&dot).unwrap();
    if link {
        let tok = dot.join("hub-register.token");
        let (rc, so, se) = out(hub_cmd(
            &hub,
            &["hub", "token", "--out", tok.to_str().unwrap(), "--register-principal", "tester", "--register-prefix", "t-", "--ttl", "600", "--json"],
        ));
        assert_eq!(rc, 0, "mint: {so} {se}");
        std::fs::write(dot.join("hub-register.addr"), format!("{addr}\n")).unwrap();
    }
    Setup { hub, user, addr, _hub_proc: guard }
}


/// The user's own hubs.toml profile for the hub address, carrying its own Interact
/// token (what runme 22c installs for dimitri-mint-dev on .107).
fn give_user_a_profile(s: &Setup) {
    let dot = s.user.path.join(".termlink");
    let tok = dot.join("interact.token");
    let (rc, so, se) = out(hub_cmd(&s.hub, &["hub", "token", "--out", tok.to_str().unwrap(), "--scope", "interact", "--ttl", "600", "--json"]));
    assert_eq!(rc, 0, "mint interact: {so} {se}");
    std::fs::write(dot.join("hubs.toml"), format!("[hubs.local]\naddress = \"{}\"\ntoken_file = \"{}\"\n", s.addr, tok.display())).unwrap();
}

#[test]
fn linked_user_without_a_local_hub_posts_and_reads_over_its_hub_link() {
    let s = setup("rt", true);
    give_user_a_profile(&s);
    // The inbox exists (as a real one does); creating topics needs Control, which this user's token lacks.
    let (rc, so, se) = out(hub_cmd(&s.hub, &["channel", "create", "inbox:t3788"]));
    assert_eq!(rc, 0, "create: {so} {se}");
    let (rc, so, se) = out(user_cmd(&s.user, &["channel", "post", "inbox:t3788", "--payload", "t3788-sentinel"]));
    assert_eq!(rc, 0, "post: {so} {se}");
    assert!(so.contains("Posted to inbox:t3788"), "{so}");
    assert!(se.contains("using this user's hub link") && se.contains(&s.addr), "explicit note: {se}");
    let (rc, so, se) = out(user_cmd(&s.user, &["channel", "subscribe", "inbox:t3788", "--json"]));
    assert_eq!(rc, 0, "subscribe: {so} {se}");
    assert!(so.contains("dDM3ODgtc2VudGluZWw="), "read back over TCP (payload_b64 of t3788-sentinel): {so}");
    // The hub owner sees the same post on its hub: it really went there.
    let (rc, so, _) = out(hub_cmd(&s.hub, &["channel", "subscribe", "inbox:t3788", "--json"]));
    assert_eq!(rc, 0);
    assert!(so.contains("dDM3ODgtc2VudGluZWw="), "{so}");
}

#[test]
fn without_a_link_the_old_outcome_stays() {
    let s = setup("nolink", false);
    let (rc, so, se) = out(user_cmd(&s.user, &["channel", "subscribe", "inbox:t3788", "--json"]));
    assert_ne!(rc, 0, "{so} {se}");
    assert!(format!("{so}{se}").contains("Hub is not running"), "{so} {se}");
    assert!(!se.contains("hub link"), "{se}");
}

#[test]
fn a_half_provisioned_link_is_an_error_not_a_silent_fallback() {
    let s = setup("half", true);
    std::fs::remove_file(s.user.path.join(".termlink").join("hub-register.token")).unwrap();
    let (rc, so, se) = out(user_cmd(&s.user, &["channel", "subscribe", "inbox:t3788", "--json"]));
    assert_ne!(rc, 0, "{so} {se}");
    let all = format!("{so}{se}");
    assert!(all.contains("half-provisioned") && all.contains("hub-register.token"), "{all}");
}

#[test]
fn root_never_falls_back_even_with_link_files() {
    let s = setup("root", true);
    give_user_a_profile(&s);
    let mut c = termlink_test_utils::termlink_cmd(bin(), &s.user.path.join("rt"));
    c.env("HOME", &s.user.path).env("TERMLINK_IDENTITY_DIR", s.user.path.join("id"))
        .args(["channel", "subscribe", "inbox:t3788", "--json"]).stdin(Stdio::null());
    // No TERMLINK_TEST_TREAT_AS_NON_ROOT: the runner's own uid decides. As root, no fallback.
    // SAFETY: geteuid has no preconditions.
    if unsafe { libc::geteuid() } != 0 {
        return; // a non-root runner cannot exercise the root branch
    }
    let (rc, so, se) = out(c);
    assert_ne!(rc, 0, "{so} {se}");
    assert!(format!("{so}{se}").contains("Hub is not running"), "{so} {se}");
}
