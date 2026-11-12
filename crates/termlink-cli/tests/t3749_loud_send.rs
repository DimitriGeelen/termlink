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

/// (d) ring20 msg 6dabf379: a controller registered without TERMLINK_RUNTIME_DIR
/// landed in $XDG_RUNTIME_DIR/termlink while the hub served /var/lib/termlink,
/// and the session was invisible for 10 days. Same layout here: a real hub in
/// `hubdir` (standing in for /var/lib/termlink via the test seam), the session
/// resolving to `xdg/termlink`. Register must refuse and name both dirs; with
/// TERMLINK_RUNTIME_DIR pointing at the hub's dir it must register (control).
#[test]
fn register_refuses_when_running_hub_serves_another_runtime_dir() {
    let dir = TestDir::new("t3749-regdir");
    let hubdir = dir.path.join("hubdir");
    let xdg = dir.path.join("xdg");
    std::fs::create_dir_all(&hubdir).unwrap();
    std::fs::create_dir_all(&xdg).unwrap();
    let base = |rt: Option<&std::path::Path>| {
        let mut c = Command::new(cargo::cargo_bin!("termlink"));
        c.env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", &dir.path)
            .env("TERMLINK_IDENTITY_DIR", dir.path.join("id"))
            .env("TERMLINK_HINT_CANDIDATE_DIRS", "")
            .env("XDG_RUNTIME_DIR", &xdg)
            .env("TERMLINK_TEST_HUB_ALT_DIR", &hubdir)
            .env("RUST_LOG", "")
            .stdin(Stdio::null());
        if let Some(rt) = rt {
            c.env("TERMLINK_RUNTIME_DIR", rt);
        }
        c
    };
    let hub = base(Some(&hubdir))
        .args(["hub", "start"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn hub");
    let _hub = ProcessGuard::new(hub, "t3749-regdir-hub");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !hubdir.join("hub.pid").exists() || !hubdir.join("hub.sock").exists() {
        assert!(Instant::now() < deadline, "hub never started");
        std::thread::sleep(Duration::from_millis(50));
    }

    // Spawn with a deadline: a register that does NOT refuse runs forever, and
    // must fail this test rather than hang it (mutant-run lesson).
    let child = base(None)
        .args(["register", "--name", "t3749-mis", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("register");
    let mut mis = ProcessGuard::new(child, "t3749-regdir-mis");
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(st) = mis.child().try_wait().unwrap() {
            break st;
        }
        assert!(Instant::now() < deadline, "register did not refuse (still running)");
        std::thread::sleep(Duration::from_millis(50));
    };
    let mut so = String::new();
    use std::io::Read as _;
    mis.child().stdout.take().unwrap().read_to_string(&mut so).unwrap();
    assert_ne!(status.code(), Some(0), "must refuse: {so}");
    assert!(so.contains("refusing to register"), "{so}");
    assert!(so.contains(&xdg.join("termlink").display().to_string()), "session dir: {so}");
    assert!(so.contains(&hubdir.display().to_string()), "hub dir: {so}");
    let sessions = xdg.join("termlink").join("sessions");
    let registered = std::fs::read_dir(&sessions)
        .map(|d| d.flatten().any(|e| e.path().extension().is_some_and(|x| x == "json")))
        .unwrap_or(false);
    assert!(!registered, "nothing may be registered on refusal");

    // Positive control: same hub, session in the hub's dir — registers.
    let child = base(Some(&hubdir))
        .args(["register", "--name", "t3749-ok"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("register ok");
    let mut ok = ProcessGuard::new(child, "t3749-regdir-ok");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let found = std::fs::read_dir(hubdir.join("sessions"))
            .map(|d| d.flatten().any(|e| e.path().extension().is_some_and(|x| x == "json")))
            .unwrap_or(false);
        if found {
            break;
        }
        assert!(ok.child().try_wait().unwrap().is_none(), "control register exited early");
        assert!(Instant::now() < deadline, "control session never registered");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// (c) `fleet join`: a throwaway TCP hub on an ephemeral 127.0.0.1 port, temp
/// HOME and identity dir. Negative control first: a wrong --fingerprint must
/// refuse (exit 1, step named) and write no profile. Then the real join must
/// probe, pin, write the secret 0600, add the profile and prove a round trip;
/// the secret itself never appears in the output.
#[test]
fn fleet_join_refuses_wrong_fingerprint_then_joins_with_round_trip() {
    let dir = TestDir::new("t3749-join");
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let tcp = format!("127.0.0.1:{port}");
    let hub = cmd(&dir, &["hub", "start", "--tcp", &tcp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn hub");
    let _hub = ProcessGuard::new(hub, "t3749-join-hub");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !dir.path.join("hub.secret").exists() || std::net::TcpStream::connect(&tcp).is_err() {
        assert!(Instant::now() < deadline, "hub never came up");
        std::thread::sleep(Duration::from_millis(50));
    }
    let secret = std::fs::read_to_string(dir.path.join("hub.secret")).unwrap().trim().to_string();
    let src = format!("file:{}", dir.path.join("hub.secret").display());
    let hubs_toml = dir.path.join(".termlink").join("hubs.toml");

    let bad = "sha256:".to_string() + &"0".repeat(64);
    let (out, so, se) = run(
        &dir,
        &["fleet", "join", &tcp, "--name", "t3749", "--bootstrap-from", &src, "--fingerprint", &bad],
    );
    assert_eq!(out.status.code(), Some(1), "stdout={so} stderr={se}");
    assert!(se.contains("[FAIL] pin") && se.contains("mismatch"), "{se}");
    let toml_now = std::fs::read_to_string(&hubs_toml).unwrap_or_default();
    assert!(!toml_now.contains("[hubs.t3749]"), "refused join wrote a profile: {toml_now}");

    let (out, so, se) = run(&dir, &["fleet", "join", &tcp, "--name", "t3749", "--bootstrap-from", &src]);
    assert_eq!(out.status.code(), Some(0), "stdout={so} stderr={se}");
    for step in ["probe", "pin", "credential", "profile", "round-trip"] {
        assert!(so.contains(&format!("[OK]   {step}")), "missing step {step}: {so}");
    }
    assert!(so.contains("JOINED"), "{so}");
    assert!(!so.contains(&secret) && !se.contains(&secret), "secret leaked to output");
    let toml_now = std::fs::read_to_string(&hubs_toml).unwrap();
    assert!(toml_now.contains("[hubs.t3749]") && toml_now.contains(&tcp), "{toml_now}");
    let sf = dir.path.join(".termlink").join("secrets").join("t3749.hex");
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(std::fs::metadata(&sf).unwrap().permissions().mode() & 0o777, 0o600);

    // The joined profile is usable by name for a real send.
    let (out, so, se) = run(
        &dir,
        &["channel", "post", "inbox:t3749", "--payload", "hi", "--ensure-topic", "--hub", "t3749"],
    );
    assert_eq!(out.status.code(), Some(0), "stdout={so} stderr={se}");
    assert!(so.contains("Posted to inbox:t3749"), "{so}");
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
