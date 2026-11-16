//! T-3366 (arc-011 P-10, B-1): E-36 / E-37 hub half / E-166 hub half at L4 — a real hub binary
//! in a throwaway runtime dir. `hub.version` must answer the minted `hub.id`, unchanged across
//! a restart and a certificate rotation; a legacy dir (secret, no id) must refuse to start until
//! `termlink hub adopt-id` gives it the certificate-derived id.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path).env("TERMLINK_IDENTITY_DIR", dir.path.join("id")).args(args).stdin(Stdio::null());
    c
}

fn hub_version(sock: &Path) -> serde_json::Value {
    let mut s = UnixStream::connect(sock).expect("connect hub");
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    s.write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"hub.version\",\"params\":{}}\n").unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    let v: serde_json::Value = serde_json::from_str(&line).expect("json reply");
    v["result"].clone()
}

/// Start a hub (optionally with TCP), wait for its socket, return the guard.
fn start(dir: &TestDir, tag: &str, tcp: bool) -> ProcessGuard {
    let port;
    let mut args = vec!["hub", "start"];
    if tcp {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        port = format!("127.0.0.1:{}", l.local_addr().unwrap().port());
        drop(l);
        args.push("--tcp");
        args.push(&port);
    }
    // SI-25 scan input: the hub's own output is kept in the runtime dir.
    let log = |n: &str| std::fs::OpenOptions::new().create(true).append(true).open(dir.path.join(n)).unwrap();
    let child = cmd(dir, &args).env("RUST_LOG", "info").stdout(log("hub.out.log")).stderr(log("hub.err.log")).spawn().expect("spawn hub");
    let g = ProcessGuard::new(child, tag);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !(dir.path.join("hub.sock").exists() && dir.path.join("hub.pid").exists()) {
        assert!(Instant::now() < deadline, "{tag}: hub socket never appeared");
        std::thread::sleep(Duration::from_millis(50));
    }
    // the socket file may exist a moment before accept; retry one connect
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(dir.path.join("hub.sock")).is_err() {
        assert!(Instant::now() < deadline, "{tag}: hub never accepted");
        std::thread::sleep(Duration::from_millis(50));
    }
    g
}

fn stop(g: ProcessGuard, dir: &TestDir) {
    drop(g);
    let deadline = Instant::now() + Duration::from_secs(10);
    while dir.path.join("hub.pid").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = std::fs::remove_file(dir.path.join("hub.pid"));
    let _ = std::fs::remove_file(dir.path.join("hub.sock"));
}

fn read_id(dir: &TestDir) -> String {
    std::fs::read_to_string(dir.path.join("hub.id")).unwrap().trim_end().to_string()
}

#[test]
fn e36_socket_only_hub_has_a_minted_id_and_hub_version_reads_the_file() {
    let dir = TestDir::new("t3366-sock");
    let g = start(&dir, "t3366-sock-hub", false);
    let v = hub_version(&dir.path.join("hub.sock"));
    let id = read_id(&dir);
    assert_eq!(v["hub_id"], id.as_str(), "{v}");
    assert!(v["signing_key"].as_str().is_some_and(|k| k.starts_with("ed25519:") && k.len() == 8 + 64), "{v}");
    stop(g, &dir);
}

#[test]
fn e36_e37_restart_and_certificate_rotation_keep_id_and_signing_key() {
    let dir = TestDir::new("t3366-rot");
    let g = start(&dir, "t3366-rot-1", true);
    let v1 = hub_version(&dir.path.join("hub.sock"));
    stop(g, &dir);
    // rotate the certificate: remove it; the next TCP start generates a new one
    let fp_before = std::fs::read(dir.path.join("hub.cert.pem")).unwrap();
    std::fs::remove_file(dir.path.join("hub.cert.pem")).unwrap();
    std::fs::remove_file(dir.path.join("hub.key.pem")).unwrap();
    let g = start(&dir, "t3366-rot-2", true);
    let v2 = hub_version(&dir.path.join("hub.sock"));
    assert_ne!(std::fs::read(dir.path.join("hub.cert.pem")).unwrap(), fp_before, "the certificate must really rotate");
    assert_ne!(v1["hub_instance_id"], v2["hub_instance_id"], "instance id follows the certificate");
    assert_eq!(v1["hub_id"], v2["hub_id"], "hub_id survives rotation: {v1} vs {v2}");
    assert_eq!(v1["signing_key"], v2["signing_key"], "signing key survives rotation");
    stop(g, &dir);
    // SI-25 (E-37.4): the private seed appears in no file of the runtime dir but its own,
    // the hub's stdout/stderr logs included.
    let seed = std::fs::read_to_string(dir.path.join("hub.signing.key")).unwrap().trim().to_string();
    assert!(std::fs::metadata(dir.path.join("hub.err.log")).unwrap().len() > 0, "the scan must see real hub output");
    let mut stack = vec![dir.path.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() { stack.push(p); continue; }
            if p.file_name().is_some_and(|n| n == "hub.signing.key") { continue; }
            let body = std::fs::read(&p).unwrap_or_default();
            assert!(!String::from_utf8_lossy(&body).contains(&seed), "seed found in {}", p.display());
        }
    }
}

#[test]
fn e36_fcw79_legacy_dir_refuses_until_adopt_id_then_keeps_the_certificate_id() {
    let dir = TestDir::new("t3366-legacy");
    // build a legacy dir: a TCP start makes secret + cert, then remove the T-3366 files
    let g = start(&dir, "t3366-legacy-1", true);
    let legacy_id = hub_version(&dir.path.join("hub.sock"))["hub_instance_id"]
        .as_str()
        .unwrap()
        .trim_start_matches("sha256:")[..16]
        .to_string();
    stop(g, &dir);
    std::fs::remove_file(dir.path.join("hub.id")).unwrap();
    std::fs::remove_file(dir.path.join("hub.signing.key")).unwrap();

    let out = cmd(&dir, &["hub", "start"]).output().unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "must refuse: {se}");
    assert!(se.contains("hub.id missing beside existing state"), "{se}");
    assert!(!dir.path.join("hub.id").exists(), "the refusal writes no file");

    let out = cmd(&dir, &["hub", "adopt-id", "--dry-run", "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!((v["hub_id"].as_str(), v["id_source"].as_str()), (Some(legacy_id.as_str()), Some("certificate")));
    assert!(!dir.path.join("hub.id").exists(), "dry-run writes nothing");

    let out = cmd(&dir, &["hub", "adopt-id", "--json"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(read_id(&dir), legacy_id, "no inbox renamed");
    let again: serde_json::Value =
        serde_json::from_slice(&cmd(&dir, &["hub", "adopt-id", "--json"]).output().unwrap().stdout).unwrap();
    assert_eq!((again["id_source"].as_str(), again["key_generated"].as_bool()), (Some("kept"), Some(false)));

    let g = start(&dir, "t3366-legacy-2", true);
    assert_eq!(hub_version(&dir.path.join("hub.sock"))["hub_id"], legacy_id.as_str());
    stop(g, &dir);
}
