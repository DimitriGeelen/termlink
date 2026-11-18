//! T-3703 (arc-011 P-15, B-5) slice B at L4: a real hub binary in a throwaway runtime dir.
//! E-43 (self-revocation by the old key, a request by another key refused, the traces of every revocation),
//! E-167 (no admin row → no revocation; a list that cannot be written → the previous list stays), the hub-side
//! post check (design v2 5.8, KEY_REVOKED) and the restore → RECONCILE → reissue path (design v2 5.7).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use base64::Engine;
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_protocol::control::error_code;
use termlink_session::agent_identity::Identity;
use termlink_session::signed_state::{self as ss, J};
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
    let _ = std::fs::remove_file(dir.path.join("hub.sock"));
    let child = cmd(dir, &["hub", "start"]).stdout(log("hub.out.log")).stderr(log("hub.err.log")).spawn().expect("spawn hub");
    let g = ProcessGuard::new(child, tag);
    let deadline = Instant::now() + Duration::from_secs(20);
    while UnixStream::connect(dir.path.join("hub.sock")).is_err() {
        assert!(Instant::now() < deadline, "{tag}: hub never accepted");
        std::thread::sleep(Duration::from_millis(50));
    }
    g
}

fn code(v: &serde_json::Value) -> Option<i64> {
    v.get("error").and_then(|e| e.get("code")).and_then(|c| c.as_i64())
}

fn hub_key(sock: &Path) -> ss::VerifyingKey {
    let v = rpc(sock, "hub.version", serde_json::json!({}));
    let r = &v["result"];
    let pk = r["signing_public_key"].as_str().expect("hub.version offers signing_public_key (design v2 5.1)");
    let fp = ss::fingerprint_of_hex(pk).unwrap();
    assert_eq!(Some(fp.as_str()), r["signing_key"].as_str(), "sha256(public key) equals the offered fingerprint");
    ss::verifying_key_of_hex(pk).unwrap()
}

/// The signed head and list, verified under the hub key (AL-20).
fn head_seq(sock: &Path, pk: &ss::VerifyingKey) -> i64 {
    let h = rpc(sock, "hub.revocation_head", serde_json::json!({}));
    let text = h["result"]["text"].as_str().unwrap_or_else(|| panic!("head: {h}"));
    let h = ss::parse_and_verify(text.as_bytes(), pk, "termlink.revocation-head/1").unwrap();
    let l = rpc(sock, "hub.revocation_get", serde_json::json!({}));
    let l = ss::parse_and_verify(l["result"]["text"].as_str().unwrap().as_bytes(), pk, "termlink.revocation/1").unwrap();
    assert_eq!(h.get("digest"), l.get("digest"), "the head names the list it heads");
    h.get("seq").and_then(J::as_int).unwrap()
}

fn list_keys(sock: &Path, pk: &ss::VerifyingKey) -> Vec<(String, String, String)> {
    let l = rpc(sock, "hub.revocation_get", serde_json::json!({}));
    let l = ss::parse_and_verify(l["result"]["text"].as_str().unwrap().as_bytes(), pk, "termlink.revocation/1").unwrap();
    let Some(J::Arr(es)) = l.get("entries") else { return vec![] };
    es.iter()
        .map(|e| {
            let s = |n: &str| e.get(n).and_then(J::as_str).unwrap_or("").to_string();
            (s("key"), s("agent"), s("by"))
        })
        .collect()
}

/// A `termlink.revocation-request/1` for `key_of`'s key, signed by `signer`.
fn request(key_of: &Identity, signer: &Identity, at_ms: i64) -> String {
    let pk_hex = key_of.public_key_hex();
    let mut o = J::Obj(vec![
        ("schema".into(), J::Str("termlink.revocation-request/1".into())),
        ("key".into(), J::Str(ss::fingerprint_of_hex(&pk_hex).unwrap())),
        ("public_key".into(), J::Str(pk_hex)),
        ("project".into(), J::Str("claimed".into())),
        ("agent".into(), J::Str("claimed".into())),
        ("reason".into(), J::Str("retired".into())),
        ("at".into(), J::Int(at_ms)),
    ]);
    let d = ss::digest(&o).unwrap();
    o.set("digest", J::Str(d.clone()));
    o.set("signature", J::Str(signer.sign_hex(&ss::signing_input("termlink.revocation-request/1", &d))));
    serde_json::to_string(&o.to_serde()).unwrap()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

fn post(sock: &Path, key: &Identity, from_agent: &str) -> serde_json::Value {
    let topic = "inbox:0000000000000000/fx-target";
    let msg_type = "sidecar.msg.v1";
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

fn put(dir: &TestDir, agent: &str, key: &Identity) {
    let k = format!("ed25519-pub:{}", key.public_key_hex());
    let o = cmd(dir, &["hub", "key-card", "put", "--agent", agent, "--project", "010-termlink", "--signing-key", &k, "--json"])
        .output()
        .unwrap();
    assert!(o.status.success(), "card put: {}", String::from_utf8_lossy(&o.stderr));
}

fn admin(dir: &TestDir) -> String {
    std::fs::read_to_string(dir.path.join("rpc-admin.jsonl")).unwrap_or_default()
}

#[test]
fn e43_self_revocation_forged_request_operator_revoke_and_the_post_check() {
    let dir = TestDir::new("t3703-rev");
    let _g = start(&dir, "t3703-rev-hub");
    let sock = dir.path.join("hub.sock");
    let pk = hub_key(&sock);
    assert_eq!(head_seq(&sock, &pk), 0, "the empty list is served signed at seq 0");

    let created = rpc(&sock, "channel.create", serde_json::json!({"name": "inbox:0000000000000000/fx-target"}));
    assert!(created.get("result").is_some(), "{created}");
    let a = Identity::generate();
    let b = Identity::generate();
    put(&dir, "fx-a", &a);
    put(&dir, "fx-b", &b);
    assert!(cmd(&dir, &["hub", "key-card", "mode", "enforce"]).output().unwrap().status.success());
    let p0 = post(&sock, &a, "fx-a");
    assert!(p0.get("error").is_none(), "carded, not revoked: stored: {p0}");

    // fx-a revokes its own key (by: self), names from the card, not from the request
    let r = rpc(&sock, "hub.revocation_request", serde_json::json!({"request": request(&a, &a, now_ms())}));
    assert_eq!(r["result"]["seq"], 1, "{r}");
    assert_eq!(r["result"]["agent"], "fx-a");
    let keys = list_keys(&sock, &pk);
    assert_eq!(keys, vec![(ss::fingerprint_of_hex(&a.public_key_hex()).unwrap(), "fx-a".into(), "self".into())]);
    // the three traces: the admin rows (intent before, committed after), the re-signed list, the attention flag
    let log = admin(&dir);
    let intent = log.lines().position(|l| l.contains("\"hub.revocation_request\"") && l.contains("\"intent\""));
    let commit = log.lines().position(|l| l.contains("\"revocation-committed\"") && l.contains("\"seq\":1"));
    assert!(matches!((intent, commit), (Some(i), Some(c)) if i < c), "intent row before committed row:\n{log}");
    assert!(log.lines().any(|l| l.contains("\"revocation-committed\"") && l.contains("\"attention\":true")));
    assert_eq!(head_seq(&sock, &pk), 1);

    // the hub's post check now refuses fx-a's key (design v2 5.8)
    assert_eq!(code(&post(&sock, &a, "fx-a")), Some(error_code::KEY_REVOKED));
    assert!(post(&sock, &b, "fx-b").get("error").is_none(), "fx-b unaffected");

    // E-43 second case: a request naming fx-b's key but signed by another key (fx-a's) is refused
    let forged = rpc(&sock, "hub.revocation_request", serde_json::json!({"request": request(&b, &a, now_ms())}));
    assert_eq!(code(&forged), Some(error_code::REVOCATION_REFUSED), "{forged}");
    assert!(admin(&dir).contains("\"revocation-request-refused\""), "the refused request is logged");
    assert_eq!(head_seq(&sock, &pk), 1, "nothing issued");
    // a stale request by the right key is refused too
    let stale = rpc(&sock, "hub.revocation_request", serde_json::json!({"request": request(&b, &b, now_ms() - 3_600_000)}));
    assert_eq!(code(&stale), Some(error_code::REVOCATION_REFUSED));

    // the operator revokes fx-b through the CLI (runme path); a repeat is idempotent
    let pkb = b.public_key_hex();
    let args = ["hub", "revocation", "revoke", "--public-key", &pkb, "--project", "010-termlink", "--agent", "fx-b", "--reason", "drill", "--json"];
    let o = cmd(&dir, &args).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!((v["seq"].as_i64(), v["already"].as_bool()), (Some(2), Some(false)));
    let v: serde_json::Value = serde_json::from_slice(&cmd(&dir, &args).output().unwrap().stdout).unwrap();
    assert_eq!((v["seq"].as_i64(), v["already"].as_bool()), (Some(2), Some(true)), "re-revoke: no new seq");
    assert_eq!(list_keys(&sock, &pk)[1].2, "operator");
}

#[test]
fn e167_no_admin_row_no_revocation_and_an_unwritable_list_keeps_the_previous_one() {
    let dir = TestDir::new("t3703-e167");
    let mut g = start(&dir, "t3703-e167-hub");
    let sock = dir.path.join("hub.sock");
    let pk = hub_key(&sock);
    let a = Identity::generate();
    let pka = a.public_key_hex();
    let revoke = |agent: &str, pk_hex: &str| {
        cmd(&dir, &["hub", "revocation", "revoke", "--public-key", pk_hex, "--project", "p", "--agent", agent, "--reason", "e167"])
            .output()
            .unwrap()
    };
    // (1) the admin log cannot be written: a directory where the file goes
    let log = dir.path.join("rpc-admin.jsonl");
    let _ = std::fs::remove_file(&log);
    std::fs::create_dir(&log).unwrap();
    std::fs::write(log.join("x"), b"x").unwrap();
    let o = revoke("fx-a", &pka);
    assert!(!o.status.success(), "no admin row, no revocation");
    assert_eq!(head_seq(&sock, &pk), 0, "no new sequence number");
    std::fs::remove_dir_all(&log).unwrap();
    assert!(revoke("fx-a", &pka).status.success());
    assert_eq!(head_seq(&sock, &pk), 1);
    // (2) the new list cannot be written: a directory where the hub's temporary file goes
    let pid = g.child().id();
    let blocker = dir.path.join(format!(".revocations.json.tmp.{pid}"));
    std::fs::create_dir(&blocker).unwrap();
    std::fs::write(blocker.join("x"), b"x").unwrap();
    let b = Identity::generate();
    let o = revoke("fx-b", &b.public_key_hex());
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("FCW-92"), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(head_seq(&sock, &pk), 1, "previous list served");
}

#[test]
fn a_restored_list_is_refused_until_reissue() {
    let dir = TestDir::new("t3703-restore");
    let sock = dir.path.join("hub.sock");
    let (k1, k2) = (Identity::generate(), Identity::generate());
    let backup;
    {
        let _g = start(&dir, "t3703-restore-hub-1");
        let pk = hub_key(&sock);
        let rv = |i: &Identity, agent: &str| {
            let h = i.public_key_hex();
            assert!(cmd(&dir, &["hub", "revocation", "revoke", "--public-key", &h, "--project", "p", "--agent", agent, "--reason", "r"])
                .output()
                .unwrap()
                .status
                .success());
        };
        rv(&k1, "fx-a");
        backup = std::fs::read(dir.path.join("revocations.json")).unwrap();
        rv(&k2, "fx-b");
        assert_eq!(head_seq(&sock, &pk), 2);
    }
    std::fs::write(dir.path.join("revocations.json"), &backup).unwrap(); // the restore
    let _g = start(&dir, "t3703-restore-hub-2");
    let pk = hub_key(&sock);
    let h = rpc(&sock, "hub.revocation_head", serde_json::json!({}));
    assert_eq!(code(&h), Some(error_code::REVOCATION_RECONCILE), "a list behind the log is not served: {h}");
    assert!(admin(&dir).contains("\"revocation-reconcile\""));
    let o = cmd(&dir, &["hub", "revocation", "reissue", "--json"]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(head_seq(&sock, &pk), 3, "reissued above every seq a sidecar may hold");
    assert_eq!(list_keys(&sock, &pk).len(), 2, "the entry the restore lost is back");
}
