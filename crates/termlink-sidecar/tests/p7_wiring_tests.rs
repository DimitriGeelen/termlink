//! Tests of the operator key register and the project address book, using temp dirs and a test estate.
//! PH-2 row P7 (T-3438): the operator's key register (P7-2, PR-1a) and the
//! project address book (SI-20 pinned main, SI-21 own-project default, R-62
//! exact instance, P7-4 provisioned instance ids). Temp dirs and a test estate
//! only; no real production path is touched.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::book::Book;
use termlink_sidecar::daemon::{self, Config};
use termlink_sidecar::estate::{PRODUCTION_MAGIC, PRODUCTION_MARKER};
use termlink_sidecar::hook;
use termlink_sidecar::testkit::Estate;

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "tl-p7-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&d).unwrap();
    d
}

fn production_root(tag: &str) -> PathBuf {
    let r = tmp(tag);
    let m = r.join(PRODUCTION_MARKER);
    fs::write(&m, format!("{PRODUCTION_MAGIC}\n")).unwrap();
    fs::set_permissions(&m, fs::Permissions::from_mode(0o644)).unwrap();
    r
}

fn book_for(me: &str) -> Book {
    let (peer, inst) = if me == "claude-termlink" {
        ("claude-termlink-alt", "claude-termlink-alt.w1-test")
    } else {
        ("claude-termlink", "claude-termlink.w1-test")
    };
    Book { peers: BTreeMap::from([(peer.to_string(), inst.to_string())]), main: Some("claude-termlink".into()) }
}

fn cfg(root: &Path, key_file: PathBuf, key_register: Option<PathBuf>, book: Book) -> Config {
    Config {
        agent: "claude-termlink".into(),
        instance: "claude-termlink.w1-test".into(),
        hub: termlink_protocol::TransportAddr::unix(root.join("no-hub.sock")),
        hub_id: "0000000000000000".into(),
        key_file,
        roster: vec![],
        queue_path: root.join("outbox.sqlite"),
        poll_ms: 200,
        timeout_ms: 2_000,
        claude_projects: None,
        key_register,
        book,
    }
}

fn err_of(r: Result<daemon::Running, String>) -> String {
    match r {
        Ok(_) => panic!("start was accepted"),
        Err(e) => e,
    }
}

fn pubkey_of(key: &Path) -> String {
    termlink_session::agent_identity::Identity::load_or_create_from_file(key).unwrap().public_key_hex()
}

#[test]
fn production_refuses_a_missing_key_file_and_never_mints_one() {
    let root = production_root("nokey");
    let key = root.join("keys").join("claude-termlink.key");
    let reg = root.join("claude-termlink.pub");
    fs::write(&reg, "00".repeat(32)).unwrap();
    let e = err_of(daemon::start(&cfg(&root, key.clone(), Some(reg), book_for("claude-termlink")), &root, &root, &root));
    assert!(e.contains("never mints a key"), "{e}");
    assert!(!key.exists(), "a production start must not create a key file");
}

#[test]
fn production_refuses_without_a_register_entry() {
    let root = production_root("noreg");
    let key = root.join("claude-termlink.key");
    pubkey_of(&key); // creates it, as the operator's provisioning would
    let e = err_of(daemon::start(&cfg(&root, key, None, book_for("claude-termlink")), &root, &root, &root));
    assert!(e.contains("key_register"), "{e}");
}

#[test]
fn a_key_that_differs_from_the_register_is_a_refused_start() {
    let root = production_root("mismatch");
    let key = root.join("claude-termlink.key");
    pubkey_of(&key);
    let reg = root.join("claude-termlink.pub");
    let other = root.join("other.key");
    fs::write(&reg, pubkey_of(&other)).unwrap();
    let e = err_of(daemon::start(&cfg(&root, key, Some(reg), book_for("claude-termlink")), &root, &root, &root));
    assert!(e.contains("does not match the key register"), "{e}");
}

#[test]
fn production_refuses_without_an_address_book_and_a_bad_book() {
    let root = production_root("nobook");
    let key = root.join("claude-termlink.key");
    let reg = root.join("claude-termlink.pub");
    fs::write(&reg, pubkey_of(&key)).unwrap();
    let e = err_of(daemon::start(&cfg(&root, key.clone(), Some(reg.clone()), Book::default()), &root, &root, &root));
    assert!(e.contains("address book"), "{e}");
    let root2 = production_root("badbook");
    let mut bad = book_for("claude-termlink");
    bad.main = Some("stranger".into());
    let e = err_of(daemon::start(&cfg(&root2, key, Some(reg), bad), &root2, &root2, &root2));
    assert!(e.contains("pinned main"), "{e}");
}

#[test]
fn a_matching_register_passes_the_key_check() {
    // Everything up to the hub connection succeeds; the start then fails only on
    // the absent hub, never on the key or the book.
    let root = production_root("match");
    let key = root.join("claude-termlink.key");
    let reg = root.join("claude-termlink.pub");
    fs::write(&reg, format!("{}\n", pubkey_of(&key))).unwrap();
    if let Err(e) = daemon::start(&cfg(&root, key, Some(reg), book_for("claude-termlink")), &root, &root, &root) {
        assert!(!e.contains("key") && !e.contains("address book"), "{e}");
    }
}

fn send(e: &Estate, req: Value) -> Value {
    hook::call(&api::socket_path(&e.runtime_dir(), "claude-termlink", Scope::Send), &req, Duration::from_secs(20)).unwrap()
}

#[test]
fn the_address_book_fills_the_instance_keeps_sends_inside_the_project_and_never_redirects() {
    let e = Estate::create().unwrap();
    let key = e.root().join("keys").join("a.key");
    let reg = e.root().join("a.pub");
    fs::write(&reg, e.identity("a").unwrap().public_key_hex()).unwrap();
    let mut c = cfg(e.root(), key, Some(reg), book_for("claude-termlink"));
    c.hub = e.hub_addr();
    c.hub_id = e.hub_id().unwrap();
    let running = daemon::start(&c, &e.store_root(), &e.runtime_dir(), &e.home()).unwrap();

    let ok = send(&e, json!({"op": "send", "to": "claude-termlink-alt", "body": "hello"}));
    assert_eq!(ok["stage"], "SENT", "{ok}");
    let out = send(&e, json!({"op": "send", "to": "penelope", "body": "x"}));
    assert_eq!(out["code"], "not-in-address-book", "{out}");
    let wrong = send(&e, json!({"op": "send", "to": "claude-termlink-alt", "to_instance": "claude-termlink-alt.old", "body": "x"}));
    assert_eq!(wrong["code"], "instance-mismatch", "{wrong}");
    let main = send(&e, json!({"op": "send", "to": "main", "body": "x"}));
    assert_eq!(main["code"], "is-self", "{main}");
    // The message went to the peer's own inbox on the hub (resolved through the book).
    let topic = format!("inbox:{}/claude-termlink-alt", c.hub_id);
    let bin = termlink_sidecar::testkit::termlink_bin().expect("termlink binary for the estate hub");
    let mut posted = false;
    for _ in 0..50 {
        let out = e.command(&bin).args(["channel", "list", "--json"]).output().unwrap();
        if String::from_utf8_lossy(&out.stdout).contains(&topic) {
            posted = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(posted, "no topic {topic} on the estate hub");
    drop(running);
}

#[test]
fn without_a_book_the_estate_send_still_requires_the_instance() {
    let e = Estate::create().unwrap();
    let mut c = cfg(e.root(), e.root().join("keys").join("a.key"), None, Book::default());
    c.hub = e.hub_addr();
    c.hub_id = e.hub_id().unwrap();
    let running = daemon::start(&c, &e.store_root(), &e.runtime_dir(), &e.home()).unwrap();
    let out = send(&e, json!({"op": "send", "to": "claude-termlink-alt", "body": "x"}));
    assert_eq!(out["code"], "bad-request", "{out}");
    drop(running);
}
