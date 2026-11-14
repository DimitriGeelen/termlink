//! T-3782: the hub forwards calls above observe scope to a TCP worker that handed it a
//! forward token at registration (operator ruling A, 2026-11-14). Who may make such a
//! call is still the hub's own scope check: inject needs Control and exec needs Execute,
//! so an Interact-scope caller is refused before anything is forwarded.

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
    let hub = TestDir::new(&format!("t3782-{tag}-hub"));
    let user = TestDir::new(&format!("t3782-{tag}-user"));
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{port}");
    let p = hub_cmd(&hub, &["hub", "start", "--tcp", &addr]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("hub");
    let guard = ProcessGuard::new(p, "t3782-hub");
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

/// Names of the records the hub knows, as its owner sees them.
fn hub_names(s: &Setup) -> String {
    let secret = s.hub.path.join("hub.secret");
    let (_, so, se) = out(hub_cmd(&s.hub, &["remote", "list", &s.addr, "--secret-file", secret.to_str().unwrap(), "--json"]));
    format!("{so}{se}")
}

fn wait_until(what: &str, secs: u64, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(200));
    }
}


fn start_linked_worker(s: &Setup, name: &str) -> (ProcessGuard, i32) {
    let child = user_cmd(&s.user, &["register", "--shell", "--name", name, "--json"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("register");
    let pid = child.id() as i32;
    let guard = ProcessGuard::new(child, "t3782-worker");
    let want = format!("t-{name}");
    wait_until("worker on the hub", 20, || hub_names(s).contains(&want));
    (guard, pid)
}

fn remote(s: &Setup, args: &[&str]) -> (i32, String, String) {
    let secret = s.hub.path.join("hub.secret");
    let mut a: Vec<&str> = vec!["remote"];
    a.extend_from_slice(args);
    let sf = secret.to_str().unwrap().to_string();
    a.push("--secret-file");
    a.push(&sf);
    out(hub_cmd(&s.hub, &a))
}

/// An Execute caller execs, and a Control caller injects, into a linked worker through
/// the hub; an Interact-scope caller is refused for both; the forward token never
/// appears in the hub's listing.
#[test]
fn hub_forwards_exec_and_inject_to_a_linked_worker_only_for_callers_its_scope_admits() {
    let s = setup("fwd", true);
    let (_w, pid) = start_linked_worker(&s, "fw1");

    let (rc, so, se) = remote(&s, &["exec", &s.addr, "t-fw1", "echo t3782-exec-ok", "--json"]);
    assert_eq!(rc, 0, "exec through the hub: {so} {se}");
    assert!(so.contains("t3782-exec-ok"), "{so}");

    let (rc, so, se) = remote(&s, &["inject", &s.addr, "t-fw1", "echo t3782-inject", "--json"]);
    assert_eq!(rc, 0, "inject through the hub: {so} {se}");

    let (rc, so, se) = remote(&s, &["exec", &s.addr, "t-fw1", "echo nope", "--scope", "interact", "--json"]);
    assert_ne!(rc, 0, "an Interact caller must not exec: {so} {se}");
    assert!(!so.contains("nope\n"), "{so}");
    let (rc, so, se) = remote(&s, &["inject", &s.addr, "t-fw1", "echo nope", "--scope", "interact", "--json"]);
    assert_ne!(rc, 0, "an Interact caller must not inject: {so} {se}");

    let listing = hub_names(&s);
    assert!(!listing.contains("forward_token"), "token field leaked into the listing: {listing}");
    assert!(!listing.contains("eyJ"), "token-like value in the listing: {listing}");

    // SAFETY: the register child we spawned.
    unsafe { libc::kill(pid, libc::SIGTERM) };
    wait_until("worker to leave the hub", 15, || !hub_names(&s).contains("t-fw1"));
}
