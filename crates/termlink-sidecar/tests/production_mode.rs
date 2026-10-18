//! Tests of the production start mode against the estate rule, in fresh temp dirs.
//! PH-2 row P1 (T-3365): the production start mode next to the PH-1 estate rule.
//! Everything here lives in fresh temp dirs; no real production path is touched.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use termlink_sidecar::estate::{self, EstateError, Mode, PRODUCTION_MAGIC, PRODUCTION_MARKER};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "tl-p1-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&d).unwrap();
    d
}

/// A production root with a genuine, owner-only-writable marker.
fn production_root(tag: &str) -> PathBuf {
    let r = tmp(tag);
    let m = r.join(PRODUCTION_MARKER);
    fs::write(&m, format!("{PRODUCTION_MAGIC}\n")).unwrap();
    fs::set_permissions(&m, fs::Permissions::from_mode(0o644)).unwrap();
    r
}

#[test]
fn production_root_is_accepted_with_real_home_and_runtime_outside_it() {
    let root = production_root("ok");
    let (home, runtime) = (tmp("home"), tmp("rt"));
    let store = root.join("stores");
    fs::create_dir_all(&store).unwrap();
    let m = estate::guard_any(&home, &runtime, &store).unwrap();
    assert_eq!(m, Mode::Production(root.canonicalize().unwrap()));
    assert_eq!(m.as_str(), "production");
    for d in [root, home, runtime] {
        let _ = fs::remove_dir_all(d);
    }
}

#[test]
fn an_unmarked_store_is_refused() {
    let (home, runtime, store) = (tmp("h"), tmp("r"), tmp("s"));
    let r = estate::guard_any(&home, &runtime, &store);
    assert!(matches!(r, Err(EstateError::NotMarked(_))), "{r:?}");
    for d in [home, runtime, store] {
        let _ = fs::remove_dir_all(d);
    }
}

#[test]
fn a_forged_production_marker_is_not_production() {
    let root = tmp("forged");
    fs::write(root.join(PRODUCTION_MARKER), "termlink-sidecar-production v0\n").unwrap();
    let r = estate::guard_any(&root, &root, &root);
    assert!(matches!(r, Err(EstateError::NotMarked(_))), "{r:?}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_production_marker_others_can_rewrite_is_refused() {
    let root = production_root("gw");
    fs::set_permissions(root.join(PRODUCTION_MARKER), fs::Permissions::from_mode(0o664)).unwrap();
    let r = estate::guard_any(&root, &root, &root);
    assert!(matches!(r, Err(EstateError::NotMarked(_))), "{r:?}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_store_with_home_inside_a_test_estate_is_refused() {
    let e = Estate::create_without_hub().unwrap();
    let root = production_root("mixed");
    let r = estate::guard_any(&e.home(), &tmp("rt2"), &root);
    assert!(matches!(r, Err(EstateError::Escapes { what: "HOME", .. })), "{r:?}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_estate_rule_is_unchanged_under_guard_any() {
    let e = Estate::create_without_hub().unwrap();
    let m = estate::guard_any(&e.home(), &e.runtime_dir(), &e.store_root()).unwrap();
    assert_eq!(m, Mode::Estate(e.root().canonicalize().unwrap()));
    let e2 = Estate::create_without_hub().unwrap();
    let r = estate::guard_any(&e.home(), &e2.runtime_dir(), &e.store_root());
    assert!(matches!(r, Err(EstateError::Escapes { .. })), "{r:?}");
}

#[test]
fn a_store_opens_in_production_mode_and_says_so() {
    let root = production_root("store");
    let s = Store::open(&root, "agent-p1").unwrap();
    assert_eq!(s.mode().as_str(), "production");
    drop(s);
    let _ = fs::remove_dir_all(root);
}
