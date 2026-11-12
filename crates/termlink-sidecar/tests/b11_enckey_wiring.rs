//! T-3696 (B-11, K-24; E-48, E-144, E-168 wiring): the encryption key register check runs inside a real
//! `daemon::start`. A test estate with its own hub: a matching register starts and status shows
//! `keys.encryption.register: match` (E-144); a swapped register refuses the start with both fingerprints (E-48);
//! no register starts in the rollback state (`not provisioned`, the negative control: the check is opt-in per
//! agent, nothing working is switched off). A production root whose register is not root-owned refuses (E-168).
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::daemon::{self, Config};
use termlink_sidecar::enckey;
use termlink_sidecar::estate::{PRODUCTION_MAGIC, PRODUCTION_MARKER};
use termlink_sidecar::hook;
use termlink_sidecar::testkit::Estate;

const AGENT: &str = "claude-b11";

fn status(e: &Estate) -> Value {
    hook::call(&api::socket_path(&e.runtime_dir(), AGENT, Scope::Status), &json!({"op": "status"}), Duration::from_secs(20)).unwrap()
}

fn cfg(e: &Estate, reg: Option<PathBuf>) -> Config {
    Config {
        agent: AGENT.into(),
        instance: format!("{AGENT}.w1-test"),
        hub: e.hub_addr(),
        hub_id: e.hub_id().unwrap(),
        key_file: e.root().join("keys").join("b11.key"),
        roster: vec![],
        queue_path: e.root().join("outbox.sqlite"),
        poll_ms: 200,
        timeout_ms: 2_000,
        claude_projects: None,
        key_register: None,
        book: Default::default(),
        main_project: None,
        enc_key_register: reg,
    }
}

fn store_dir(e: &Estate) -> PathBuf {
    e.store_root().join("sidecar").join(AGENT)
}

fn write_reg(p: &Path, body: &str) {
    fs::write(p, body).unwrap();
    fs::set_permissions(p, fs::Permissions::from_mode(0o644)).unwrap();
}

#[test]
fn e144_e48_register_check_runs_at_start() {
    // E-144: provisioning wrote both halves; the start succeeds and status says match.
    let e = Estate::create().unwrap();
    let k = enckey::load_key(&enckey::key_path(&store_dir(&e)), false).unwrap();
    let reg = e.root().join("b11.x25519.pub");
    write_reg(&reg, &k.public_hex());
    let running = daemon::start(&cfg(&e, Some(reg.clone())), &e.store_root(), &e.runtime_dir(), &e.home()).unwrap();
    let s = status(&e);
    assert_eq!(s["keys"]["encryption"]["register"], "match", "{s}");
    assert_eq!(s["keys"]["encryption"]["fingerprint"], json!(k.fingerprint()));
    drop(running);

    // E-48: the register swapped for another agent's key refuses the start, naming both fingerprints.
    let other = enckey::EncKey::generate();
    write_reg(&reg, &other.public_hex());
    let err = match daemon::start(&cfg(&e, Some(reg)), &e.store_root(), &e.runtime_dir(), &e.home()) {
        Ok(_) => panic!("a swapped register was accepted"),
        Err(err) => err,
    };
    assert!(err.contains("FCW-109") && err.contains(&k.fingerprint()) && err.contains(&other.fingerprint()), "{err}");

    // Negative control / rollback: no register configured starts, status says not provisioned.
    let running = daemon::start(&cfg(&e, None), &e.store_root(), &e.runtime_dir(), &e.home()).unwrap();
    assert_eq!(status(&e)["keys"]["encryption"]["state"], "not provisioned");
    drop(running);
}

#[test]
fn e168_production_register_not_root_owned_refuses_start() {
    let root = std::env::temp_dir().join(format!(
        "tl-b11-prod-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let m = root.join(PRODUCTION_MARKER);
    fs::write(&m, format!("{PRODUCTION_MAGIC}\n")).unwrap();
    fs::set_permissions(&m, fs::Permissions::from_mode(0o644)).unwrap();
    // The signing identity passes (P7-2), so the refusal below is the encryption register's.
    let key = root.join("b11.key");
    let pk = termlink_session::agent_identity::Identity::load_or_create_from_file(&key).unwrap().public_key_hex();
    let sreg = root.join("b11.pub");
    fs::write(&sreg, &pk).unwrap();
    let reg = root.join("b11.x25519.pub");
    write_reg(&reg, &"11".repeat(32));
    // SAFETY: geteuid has no preconditions. As root, hand the entry to `nobody`; otherwise it is ours, not root's.
    if unsafe { libc::geteuid() } == 0 {
        std::os::unix::fs::chown(&reg, Some(65534), None).unwrap();
    }
    let c = Config {
        agent: AGENT.into(),
        instance: format!("{AGENT}.w1-test"),
        hub: termlink_protocol::TransportAddr::unix(root.join("no-hub.sock")),
        hub_id: "0000000000000000".into(),
        key_file: key,
        roster: vec![],
        queue_path: root.join("outbox.sqlite"),
        poll_ms: 200,
        timeout_ms: 2_000,
        claude_projects: None,
        key_register: Some(sreg),
        book: Default::default(),
        main_project: None,
        enc_key_register: Some(reg),
    };
    let err = match daemon::start(&c, &root, &root, &root) {
        Ok(_) => panic!("a non-root register was accepted in production"),
        Err(err) => err,
    };
    assert!(err.contains("FCW-110") && err.contains("b11.x25519.pub") && err.contains("not 0"), "{err}");
}
