//! T-3708 (P-24 slice a): status through a real API socket. Hermetic: testkit estate, no hub.
//! Design 3.0 (additive only), 3.1 (E-55 schema, restart = unknown), 3.9 (latency log), D-2 (quota).

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use termlink_sidecar::api::*;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

/// Keys `status` returned before P-24 (live path): they must stay, with the same shape.
const PRE_P24_KEYS: &[&str] = &[
    "ok", "agent", "hub_id", "hub", "hub_signing_key", "revocation", "consumers", "stages", "rows", "messages", "chains",
    "delivery", "mode", "receiving", "strength", "missing_strength", "notes", "outbox", "supervision", "activity_veto",
    "binary", "main", "keys", "vendor_log", "vendor_log_detail",
];
const P24_KEYS: &[&str] = &["fields", "clear", "strength_host", "latency_log"];
const FIELD_NAMES: &[&str] =
    &["mode", "binary", "hub", "revocation", "consumers", "supervision", "quota", "strength", "skew", "ntp", "last_surface"];

fn status(e: &Estate, store: &Arc<Store>) -> Value {
    let cfg = ApiConfig::new("agent-b", &e.runtime_dir());
    let api = Api::start(cfg, store.clone()).unwrap();
    let mut s = UnixStream::connect(api.path(Scope::Status)).unwrap();
    writeln!(s, "{}", json!({"op": "status"})).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

// One test, sequential: the switch is a process environment variable.
#[test]
fn t3708_status_is_additive_stamped_and_switchable() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let v = status(&e, &store);
    assert_eq!(v["ok"], true);

    // design 3.0: every pre-P-24 key is still there; the old strength map keeps its shape
    for k in PRE_P24_KEYS {
        assert!(v.get(*k).is_some(), "pre-P-24 key {k} missing");
    }
    assert_eq!(v["strength"]["per_agent_accounts"], false);
    assert!(v["missing_strength"].is_array());

    // E-55 schema: each field stamped fresh (value, at_ms, bound_ms) or unknown (shown, last_at_ms)
    for k in P24_KEYS {
        assert!(v.get(*k).is_some(), "P-24 key {k} missing");
    }
    for n in FIELD_NAMES {
        let f = &v["fields"][*n];
        match f["state"].as_str() {
            Some("fresh") => assert!(f.get("value").is_some() && f["at_ms"].is_u64() && f["bound_ms"].is_u64(), "{n}: {f}"),
            Some("unknown") => assert!(f["shown"].as_str().is_some_and(|s| s.starts_with("unknown (")) && f.get("value").is_none(), "{n}: {f}"),
            _ => panic!("field {n} has no state: {f}"),
        }
    }

    // 06a 2.13.f: nothing has been measured in this process (no pull, no heartbeat): unknown, never CLEAR
    for n in ["hub", "revocation", "consumers", "supervision", "skew"] {
        assert_eq!(v["fields"][n]["state"], "unknown", "{n} must read unknown before its producer ran");
    }
    assert_eq!(v["clear"]["state"], "NOT CLEAR");
    assert!(v["clear"]["unknown"].as_array().unwrap().len() >= 5, "{}", v["clear"]);

    // D-2: quota names the absence; computed fields are fresh
    assert_eq!(v["fields"]["quota"]["value"], "absent: receiver quotas not built (P-17)");
    assert_eq!(v["fields"]["mode"]["state"], "fresh");
    assert_eq!(v["strength_host"]["operator_key"], "no");

    // design 3.9: the call left one latency line
    assert_eq!(v["latency_log"], "ok");
    let log = std::fs::read_to_string(store.dir().join(LATENCY_LOG)).unwrap();
    let last: Value = serde_json::from_str(log.lines().last().unwrap()).unwrap();
    assert!(last["ms"].is_u64() && last["clear"] == "NOT CLEAR", "{last}");

    // design 3.0 rollback: the switch hides every new key and leaves the old ones
    // SAFETY (test): this binary has one test; nothing else reads the variable concurrently.
    unsafe { std::env::set_var("TERMLINK_SIDECAR_P24", "off") };
    let off = status(&e, &store);
    unsafe { std::env::remove_var("TERMLINK_SIDECAR_P24") };
    for k in P24_KEYS {
        assert!(off.get(*k).is_none(), "{k} shown with the switch off");
    }
    for k in PRE_P24_KEYS {
        assert!(off.get(*k).is_some(), "pre-P-24 key {k} missing with the switch off");
    }
}
