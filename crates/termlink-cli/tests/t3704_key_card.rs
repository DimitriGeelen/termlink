//! T-3704 (arc-011 P-16, B-6): E-69 post half and E-169's card case at L4 — a real hub binary in a
//! throwaway runtime dir. Cards are written with `termlink hub key-card put` (the runme path);
//! posts are signed `channel.post` RPCs over the hub socket, so the hub sees exactly what a
//! sidecar (or a forger holding hub access) would send.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use base64::Engine;
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::Identity;
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path).env("TERMLINK_IDENTITY_DIR", dir.path.join("id")).args(args).stdin(Stdio::null());
    c
}

fn rpc(sock: &Path, method: &str, params: serde_json::Value) -> serde_json::Value {
    let mut s = UnixStream::connect(sock).expect("connect hub");
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    s.write_all(format!("{req}\n").as_bytes()).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).expect("json reply")
}

fn start(dir: &TestDir, tag: &str) -> ProcessGuard {
    let log = |n: &str| std::fs::OpenOptions::new().create(true).append(true).open(dir.path.join(n)).unwrap();
    let child = cmd(dir, &["hub", "start"]).stdout(log("hub.out.log")).stderr(log("hub.err.log")).spawn().expect("spawn hub");
    let g = ProcessGuard::new(child, tag);
    let deadline = Instant::now() + Duration::from_secs(20);
    while UnixStream::connect(dir.path.join("hub.sock")).is_err() {
        assert!(Instant::now() < deadline, "{tag}: hub never accepted");
        std::thread::sleep(Duration::from_millis(50));
    }
    g
}

/// A signed arc-011 post: `from_agent` claimed in the payload, signed by `key`.
fn post(sock: &Path, key: &Identity, from_agent: &str, msg_type: &str) -> serde_json::Value {
    let topic = "inbox:0000000000000000/fx-target";
    let payload = serde_json::json!({"v": 1, "kind": "msg", "from_agent": from_agent, "body": "hello"}).to_string();
    let ts = 1_800_000_000_000i64;
    let bytes = canonical_sign_bytes(topic, msg_type, payload.as_bytes(), None, ts);
    rpc(
        sock,
        "channel.post",
        serde_json::json!({
            "topic": topic, "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
            "ts": ts, "sender_id": key.fingerprint(), "sender_pubkey_hex": key.public_key_hex(),
            "signature_hex": key.sign_hex(&bytes),
        }),
    )
}

fn code(v: &serde_json::Value) -> Option<i64> {
    v.get("error").and_then(|e| e.get("code")).and_then(|c| c.as_i64())
}

fn put(dir: &TestDir, agent: &str, project: &str, key: &Identity, replace: bool) -> std::process::Output {
    let k = format!("ed25519-pub:{}", key.public_key_hex());
    let mut a = vec!["hub", "key-card", "put", "--agent", agent, "--project", project, "--signing-key", &k, "--json"];
    if replace {
        a.push("--replace");
    }
    cmd(dir, &a).output().unwrap()
}

#[test]
fn e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused() {
    let dir = TestDir::new("t3704-card");
    let _g = start(&dir, "t3704-card-hub");
    let sock = dir.path.join("hub.sock");
    let (a, b, c) = (Identity::generate(), Identity::generate(), Identity::generate());
    let created = rpc(&sock, "channel.create", serde_json::json!({"name": "inbox:0000000000000000/fx-target"}));
    assert!(created.get("result").is_some(), "{created}");

    // default off: a forgery still passes (the check is not on yet), proving the default.
    assert_eq!(code(&post(&sock, &b, "fx-a", "sidecar.msg.v1")), None, "default off");

    assert!(put(&dir, "fx-a", "proj-p", &a, false).status.success());
    assert!(put(&dir, "fx-b", "proj-p", &b, false).status.success());
    let m = cmd(&dir, &["hub", "key-card", "mode", "enforce"]).output().unwrap();
    assert!(m.status.success(), "{}", String::from_utf8_lossy(&m.stderr));

    // E-69.3: post as fx-a signed with fx-b's key -> refused (FCW-133)
    assert_eq!(code(&post(&sock, &b, "fx-a", "sidecar.msg.v1")), Some(-32044));
    // a holder of hub access with its own fresh key claiming fx-a (BP-3) -> refused
    assert_eq!(code(&post(&sock, &c, "fx-a", "sidecar.stage.v1")), Some(-32044));
    // E-169: an agent with no card -> refused (FCW-135)
    assert_eq!(code(&post(&sock, &c, "fx-z", "sidecar.msg.v1")), Some(-32043));
    // a traversal name never reaches a lookup -> refused (FCW-135)
    assert_eq!(code(&post(&sock, &c, "../x", "sidecar.msg.v1")), Some(-32043));
    // fx-a's own key -> stored
    let ok = post(&sock, &a, "fx-a", "sidecar.msg.v1");
    assert!(ok.get("result").is_some(), "{ok}");
    // another type on inbox:* is never gated (NID)
    assert!(post(&sock, &c, "fx-a", "chat").get("result").is_some());

    // a key change without --replace is refused; with --replace accepted and logged with both keys
    let r = put(&dir, "fx-a", "proj-p", &c, false);
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("replace"), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(put(&dir, "fx-a", "proj-p", &c, true).status.success());
    let admin = std::fs::read_to_string(dir.path.join("rpc-admin.jsonl")).unwrap_or_default();
    assert!(
        admin.lines().any(|l| l.contains("\"key-card-put\"") && l.contains("\"attention\":true")
            && l.contains(&a.public_key_hex()) && l.contains(&c.public_key_hex())),
        "admin log lacks the replace row naming old and new key:\n{admin}"
    );
    assert!(admin.contains("\"key-card-refused\""), "refusals are admin-logged:\n{admin}");

    // per-agent refusal counts for the impersonated agent
    let list = rpc(&sock, "hub.key_card_list", serde_json::json!({}));
    let refused = list["result"]["refusals"]["fx-a"]["refused"].as_u64().unwrap_or(0);
    assert!(refused >= 2, "{list}");

    // log mode: a forgery is stored but counted as would-refuse
    assert!(cmd(&dir, &["hub", "key-card", "mode", "log"]).status().unwrap().success());
    assert!(post(&sock, &b, "fx-a", "sidecar.msg.v1").get("result").is_some(), "log mode stores");
    let list = rpc(&sock, "hub.key_card_list", serde_json::json!({}));
    assert!(list["result"]["refusals"]["fx-a"]["would_refuse"].as_u64().unwrap_or(0) >= 1, "{list}");
}

#[test]
fn e169_a_corrupt_card_store_refuses_every_gated_post() {
    let dir = TestDir::new("t3704-corrupt");
    let _g = start(&dir, "t3704-corrupt-hub");
    let sock = dir.path.join("hub.sock");
    let a = Identity::generate();
    assert!(put(&dir, "fx-a", "proj-p", &a, false).status.success());
    assert!(cmd(&dir, &["hub", "key-card", "mode", "enforce"]).status().unwrap().success());
    std::fs::write(dir.path.join("key-cards.json"), "{broken").unwrap();
    let v = post(&sock, &a, "fx-a", "sidecar.msg.v1");
    assert_eq!(code(&v), Some(-32043), "{v}");
    assert!(v["error"]["message"].as_str().unwrap_or("").contains("corrupt"), "{v}");
    // an invalid setting fails closed, too
    std::fs::write(dir.path.join("key-card-check"), "maybe\n").unwrap();
    std::fs::remove_file(dir.path.join("key-cards.json")).unwrap();
    assert_eq!(code(&post(&sock, &a, "fx-a", "sidecar.msg.v1")), Some(-32043), "no card + invalid setting = enforce");
}
