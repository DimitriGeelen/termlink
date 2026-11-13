//! T-3781: a non-root user with a hub link (a registration-claim token plus the hub
//! address, under `~/.termlink`) registers its sessions on the main hub over TCP, and
//! `termlink dispatch` works for it without the hub's Unix socket.
//!
//! Every case runs a throwaway TCP hub in one temp dir and the "user" in another (its
//! own HOME and runtime dir). The test runner is often root, so the user side sets the
//! seam `TERMLINK_TEST_TREAT_AS_NON_ROOT=1` (it only ever lowers privilege).

use std::io::{BufRead, BufReader};
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
    let hub = TestDir::new(&format!("t3781-{tag}-hub"));
    let user = TestDir::new(&format!("t3781-{tag}-user"));
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{port}");
    let p = hub_cmd(&hub, &["hub", "start", "--tcp", &addr]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("hub");
    let guard = ProcessGuard::new(p, "t3781-hub");
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

/// `register --shell` by a linked user: one session, local (own runtime dir, full access
/// for its own user) and on the hub as `t-<name>` owned by the claim's principal; gone
/// from the hub after SIGTERM.
#[test]
fn linked_register_shell_is_local_and_on_the_hub_then_deregisters() {
    let s = setup("reg", true);
    let mut child = user_cmd(&s.user, &["register", "--shell", "--name", "w1", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("register");
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if line.starts_with('{') {
                let _ = tx.send(line);
            }
        }
    });
    let pid = child.id() as i32;
    let line = match rx.recv_timeout(Duration::from_secs(20)) {
        Ok(l) => l,
        Err(_) => {
            let _ = child.kill();
            let o = child.wait_with_output().unwrap();
            panic!("no JSON from register; stderr={}", String::from_utf8_lossy(&o.stderr));
        }
    };
    let guard = ProcessGuard::new(child, "t3781-register");
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["ok"], true, "{line}");
    assert_eq!(v["hub_link"]["name"], "t-w1", "{line}");
    assert_eq!(v["hub_link"]["owner"], "tester", "{line}");
    assert_eq!(v["token_auth"], true, "TCP is token-only: {line}");

    assert!(hub_names(&s).contains("t-w1"), "hub does not list t-w1: {}", hub_names(&s));

    // Its own user keeps full access on the Unix socket despite the TCP token secret.
    let (rc, so, se) = out(user_cmd(&s.user, &["exec", "w1", "echo t3781-exec-ok", "--json"]));
    assert_eq!(rc, 0, "exec: {so} {se}");
    assert!(so.contains("t3781-exec-ok"), "exec output: {so}");

    // SAFETY: pid is the register child we spawned.
    unsafe { libc::kill(pid, libc::SIGTERM) };
    wait_until("t-w1 to leave the hub", 15, || !hub_names(&s).contains("t-w1"));
    drop(guard);
}

/// Without a link and without a reachable hub socket, a non-root dispatch refuses and
/// says how to provision a link; nothing is spawned.
#[test]
fn unlinked_non_root_dispatch_refuses_with_a_provisioning_hint() {
    let s = setup("nolink", false);
    let (rc, so, se) = out(user_cmd(&s.user, &["dispatch", "--count", "1", "--topic", "result", "--timeout", "5", "--backend", "background", "--", "true"]));
    assert_ne!(rc, 0, "{so} {se}");
    assert!(se.contains("no hub link") && se.contains("hub-register.addr"), "{se}");
    assert!(!s.user.path.join("rt").join("sessions").exists() || std::fs::read_dir(s.user.path.join("rt").join("sessions")).unwrap().next().is_none(), "a worker was spawned");
}

/// A linked dispatch: two workers register on the hub (as `t-<name>`) and both results
/// are collected from the workers' own sockets.
#[test]
fn linked_dispatch_registers_workers_on_the_hub_and_collects_their_results() {
    let s = setup("disp", true);
    let tl = bin().to_str().unwrap().to_string();
    let script = format!("sleep 3; '{tl}' emit \"$TERMLINK_WORKER_NAME\" result -p '{{\"ok\":1}}'; sleep 6");
    // The seam is passed on to the workers explicitly: the spawned shells run as the
    // test runner (often root), and without it their `register` would skip the link.
    let mut c =user_cmd(&s.user, &["dispatch", "--count", "2", "--topic", "result", "--timeout", "40", "--backend", "background", "--name", "dw", "--env", "TERMLINK_TEST_TREAT_AS_NON_ROOT=1", "--json", "--", "sh", "-c", &script]);
    let child = c.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("dispatch");
    let names = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    // Watch the hub while the dispatch runs: both workers must appear there.
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut seen = false;
    while Instant::now() < deadline {
        let n = hub_names(&s);
        if n.contains("t-dw-1") && n.contains("t-dw-2") {
            seen = true;
            break;
        }
        *names.lock().unwrap() = n;
        std::thread::sleep(Duration::from_millis(300));
    }
    let o = child.wait_with_output().expect("dispatch output");
    let so = String::from_utf8_lossy(&o.stdout);
    let se = String::from_utf8_lossy(&o.stderr);
    assert!(seen, "workers never on the hub; last list: {}; dispatch stderr: {se}", names.lock().unwrap());
    assert_eq!(o.status.code(), Some(0), "stdout={so} stderr={se}");
    let v: serde_json::Value = serde_json::from_str(so.trim()).unwrap_or_default();
    assert_eq!(v["workers_spawned"], 2, "{so}");
    let results = v["results"].as_array().map(|a| a.len()).unwrap_or(0);
    assert_eq!(results, 2, "both results collected: {so}");
}
