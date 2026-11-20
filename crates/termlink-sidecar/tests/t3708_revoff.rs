//! T-3708 (FCW-120): with the revocation pull switched off, the tracker's pass still stamps the `revocation`
//! field ("off (PH-1 trust)" is a measured value). Own binary: the field registry is process-wide and must start empty.

use std::time::{Duration, Instant};
use termlink_sidecar::fields;
use termlink_sidecar::revocation::{PN13, Tracker};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

#[test]
fn t3708_switched_off_revocation_is_still_stamped() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "fx-b").unwrap();
    store.write_state("revocation-pull", "off").unwrap();
    assert!(fields::global().last(fields::REVOCATION).is_none(), "the registry starts empty in this process");
    let mut t = Tracker::new(&store, "0123456789abcdef", PN13);
    let rpc = |_: &str| -> Result<serde_json::Value, (i64, String)> { Err((0, "no hub in this test".into())) };
    t.tick(&store, &serde_json::json!({}), &rpc, Instant::now(), None);
    assert!(fields::global().last(fields::REVOCATION).is_some(), "an off pass left the revocation field unstamped");
    let _ = Duration::ZERO;
    // r2 MINOR 6: a failed pass stamps the field and rewrites the line, so it never reads "verified" fresh
    store.write_state("revocation-pull", "on").unwrap();
    t.tick(&store, &serde_json::json!({}), &rpc, Instant::now(), None);
    let line = termlink_sidecar::revocation::status(&store)["revocation"].as_str().unwrap().to_string();
    assert!(!line.starts_with("verified"), "a failed pull left the line as {line}");
    assert!(termlink_sidecar::fields::revocation_reading(&line) != "ok", "{line}");
}
