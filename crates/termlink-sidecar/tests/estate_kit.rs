//! Tests of the estate guard and the disposable test estate harness.
use std::fs;
use termlink_sidecar::estate::{self, EstateError};
use termlink_sidecar::testkit::Estate;

#[test]
fn estate_guard_accepts_marked_tree() {
    let e = Estate::create_without_hub().unwrap();
    let r = estate::guard(&e.home(), &e.runtime_dir(), &e.store_root()).unwrap();
    assert_eq!(r, e.root().canonicalize().unwrap());
}

#[test]
fn estate_guard_refuses_unmarked_and_escaping_paths() {
    let e = Estate::create_without_hub().unwrap();
    let outside = std::env::temp_dir().join("tl-unmarked-probe");
    fs::create_dir_all(&outside).unwrap();
    let r = estate::guard(&outside, &e.runtime_dir(), &e.store_root());
    assert!(matches!(r, Err(EstateError::NotMarked(_))), "{r:?}");
    let r = estate::guard(&e.home(), &outside, &e.store_root());
    assert!(matches!(r, Err(EstateError::NotMarked(_))), "{r:?}");
    // A second marked estate is a different tree: mixing them is refused.
    let e2 = Estate::create_without_hub().unwrap();
    let r = estate::guard(&e.home(), &e2.runtime_dir(), &e.store_root());
    assert!(matches!(r, Err(EstateError::Escapes { .. })), "{r:?}");
    let _ = fs::remove_dir_all(&outside);
}

#[test]
fn estate_guard_rejects_forged_marker() {
    let e = Estate::create_without_hub().unwrap();
    fs::write(e.root().join(".termlink-estate"), "not the magic\n").unwrap();
    let r = estate::guard(&e.home(), &e.runtime_dir(), &e.store_root());
    assert!(matches!(r, Err(EstateError::NotMarked(_))), "{r:?}");
}

#[test]
fn estate_sidecar_binary_refuses_outside_estate_exit_2() {
    let bin = env!("CARGO_BIN_EXE_termlink-sidecar");
    let out = std::process::Command::new(bin)
        .arg("start")
        .env_clear()
        .env("HOME", "/tmp")
        .env("TERMLINK_RUNTIME_DIR", "/tmp")
        .env("TERMLINK_SIDECAR_STORE", "/tmp")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing to start"));
    // Unset environment is refused too.
    let out = std::process::Command::new(bin)
        .arg("start")
        .env_clear()
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn estate_sidecar_binary_starts_inside_estate() {
    let e = Estate::create_without_hub().unwrap();
    let out = e
        .command(env!("CARGO_BIN_EXE_termlink-sidecar"))
        .arg("start")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
}

#[test]
fn estate_env_is_isolated_and_keys_exist() {
    let e = Estate::create_without_hub().unwrap();
    let env = e.env();
    for k in ["HOME", "TERMLINK_RUNTIME_DIR", "XDG_RUNTIME_DIR"] {
        assert!(env[k].starts_with(e.root().to_str().unwrap()), "{k}");
    }
    assert!(env.contains_key("TERMLINK_ESTATE_CANARY"));
    for k in ["a.key", "b.key"] {
        assert!(e.root().join("keys").join(k).is_file());
    }
    assert!(e.root().join("proj-a/TASK.md").is_file());
}

#[test]
fn estate_secret_scan_finds_planted_canary_only() {
    let e = Estate::create_without_hub().unwrap();
    assert!(e.secret_scan(e.root()).is_empty());
    fs::write(
        e.store_root().join("leak.log"),
        format!("x {} y", e.canary()),
    )
    .unwrap();
    assert_eq!(
        e.secret_scan(e.root()),
        vec![e.store_root().join("leak.log")]
    );
}

#[test]
fn estate_hub_starts_audits_clean_and_tears_down_without_survivors() {
    let mut e = Estate::create().expect("estate with hub");
    assert!(e.runtime_dir().join("hub.sock").exists());
    let root = e.root().to_path_buf();
    assert_eq!(e.path_open_audit(), Vec::<std::path::PathBuf>::new());
    // the canary is exported to children but must not appear in the tree
    assert!(e.secret_scan(e.root()).is_empty());
    let survivors = e.teardown();
    assert!(survivors.is_empty(), "{survivors:?}");
    assert!(!root.exists());
}
