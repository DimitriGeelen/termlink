//! T-3366 (arc-011 P-10, B-1): E-37 sidecar half and E-166 (FCW-84) against a throwaway hub.
//! The sidecar pins the hub's signing key on first verified contact, keeps it across a hub
//! restart, and after a key swap refuses every hub-signed object with a needs-attention entry
//! naming both fingerprints. It never re-pins silently and never holds mail for it.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use termlink_sidecar::hub_key_pin::{self, KeyState};
use termlink_sidecar::send::*;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

fn sender_cfg(e: &Estate, hub_id: &str) -> SendConfig {
    SendConfig {
        agent: "agent-a".into(),
        instance: "inst-a".into(),
        identity: e.identity("a").unwrap(),
        hub: e.hub_addr(),
        declared_hub_id: hub_id.into(),
        queue_path: e.root().join("outbox-a.sqlite"),
        timeout: Duration::from_secs(5),
        pump_interval: None,
    }
}

fn hub_version(e: &Estate) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(
            &e.hub_addr(),
            "hub.version",
            json!({}),
            Duration::from_secs(10),
        ))
        .unwrap();
    termlink_session::client::unwrap_result(resp).unwrap()
}

fn attention_kinds(store: &Store) -> Vec<(String, String)> {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| (v["kind"].as_str().unwrap_or("").to_string(), v["detail"].as_str().unwrap_or("").to_string()))
        .collect()
}

#[test]
fn e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named() {
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let sender = Sender::new(sender_cfg(&e, &id), store.clone()).unwrap();

    // first verified contact (local socket): pinned, and said so
    assert!(sender.check_pin().is_ok());
    let k1 = hub_version(&e)["signing_key"].as_str().unwrap().to_string();
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name(&id)).unwrap().as_deref(), Some(k1.as_str()));
    assert!(attention_kinds(&store).iter().any(|(k, d)| k == "hub-signing-key-pinned" && d.contains(&k1)));
    // OBS-37: the admin row on the store's admin chain names the pinned key
    let rows = store.chain_lines(termlink_sidecar::api::ADMIN_CONV).unwrap();
    assert!(rows.iter().any(|l| l.event == "hub-key-pinned" && l.msg_id == id), "no admin row");
    assert_eq!(hub_key_pin::verified_hub_key(&store, &id, &hub_version(&e)).as_deref(), Some(k1.as_str()));

    // restart: same key, still pinned, nothing changed
    e.restart_hub().unwrap();
    assert!(sender.check_pin().is_ok());
    assert_eq!(hub_key_pin::status(&store, Some(&id))["state"], "pinned");
    assert!(!attention_kinds(&store).iter().any(|(k, _)| k == "hub-signing-key-changed"));

    // swap the hub's key file (a second valid key), restart
    e.stop_hub();
    let key = e.runtime_dir().join("hub.signing.key");
    std::fs::write(&key, format!("{}\n", "7".repeat(64))).unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    e.restart_hub().unwrap();
    let k2 = hub_version(&e)["signing_key"].as_str().unwrap().to_string();
    assert_ne!(k1, k2, "the fixture must really swap the key");

    // mail is not held (the hub id still matches) ...
    assert!(sender.check_pin().is_ok());
    // ... but every hub-signed object is refused, the pin is unchanged, and the entry names both
    assert_eq!(hub_key_pin::verified_hub_key(&store, &id, &hub_version(&e)), None);
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name(&id)).unwrap().as_deref(), Some(k1.as_str()));
    let changed: Vec<_> = attention_kinds(&store).into_iter().filter(|(k, _)| k == "hub-signing-key-changed").collect();
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert!(changed[0].1.contains(&k1) && changed[0].1.contains(&k2), "{changed:?}");
    let st = hub_key_pin::status(&store, Some(&id));
    assert_eq!((st["state"].as_str(), st["pinned"].as_str(), st["offered"].as_str()), (Some("mismatch"), Some(k1.as_str()), Some(k2.as_str())));

    // coalesced: a second contact the same day adds no second entry
    assert!(sender.check_pin().is_ok());
    assert_eq!(attention_kinds(&store).iter().filter(|(k, _)| k == "hub-signing-key-changed").count(), 1);

    // SI-25: the hub's private seed appears nowhere in the sidecar's store
    let seed = "7".repeat(64);
    for entry in walk(&e.store_root()) {
        let body = std::fs::read(&entry).unwrap_or_default();
        assert!(!String::from_utf8_lossy(&body).contains(&seed), "seed found in {}", entry.display());
    }
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { out.extend(walk(&p)); } else { out.push(p); }
        }
    }
    out
}

#[test]
fn e166_fcw84_untrusted_first_contact_pins_nothing_and_no_offer_changes_nothing() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let fp = format!("ed25519:{}", "a".repeat(64));
    let v = json!({"hub_id": "0123456789abcdef", "signing_key": fp});

    // not over the local socket: no pin, unverified, hub-signed data refused
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &v, false), KeyState::Unverified(fp.clone()));
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name("0123456789abcdef")).unwrap(), None);
    assert_eq!(hub_key_pin::verified_hub_key(&store, "0123456789abcdef", &v), None);
    assert_eq!(hub_key_pin::status(&store, Some("0123456789abcdef"))["state"], "unverified");

    // a hub that predates T-3366 offers no key: nothing pinned, nothing refused as a mismatch
    let old = json!({"hub_id": "0123456789abcdef"});
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &old, true), KeyState::NotOffered);
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name("0123456789abcdef")).unwrap(), None);

    // a malformed offer is never pinned, even over the trusted path
    let bad = json!({"hub_id": "0123456789abcdef", "signing_key": "ed25519:nothex"});
    assert!(matches!(hub_key_pin::observe(&store, "0123456789abcdef", &bad, true), KeyState::Unverified(_)));
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name("0123456789abcdef")).unwrap(), None);

    // and the trusted path does pin a well-formed offer
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &v, true), KeyState::NewlyPinned(fp.clone()));
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &v, true), KeyState::Pinned(fp));
}

// Review r1 #2: an UNREADABLE pin is never treated as "no pin" (no re-pin over it).
#[test]
fn e37_unreadable_pin_is_not_replaced() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    // the pin's state file is a directory: reading it errors
    std::fs::create_dir_all(store.dir().join(format!("state.{}", hub_key_pin::pin_state_name(h)))).unwrap();
    let v = json!({"hub_id": h, "signing_key": format!("ed25519:{}", "b".repeat(64))});
    assert!(matches!(hub_key_pin::observe(&store, h, &v, true), KeyState::Unverified(_)));
    assert!(store.dir().join(format!("state.{}", hub_key_pin::pin_state_name(h))).is_dir(), "not replaced");
    assert_eq!(hub_key_pin::verified_hub_key(&store, h, &v), None);
    assert!(attention_kinds(&store).iter().any(|(k, _)| k == "hub-signing-key-pin-unreadable"));
}

// Review r1 #2: a pinned hub that stops offering its key is refused and named.
#[test]
fn e37_withdrawn_key_is_named_and_refused() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    let fp = format!("ed25519:{}", "c".repeat(64));
    hub_key_pin::observe(&store, h, &json!({"hub_id": h, "signing_key": fp}), true);
    let old = json!({"hub_id": h});
    assert_eq!(hub_key_pin::observe(&store, h, &old, true), KeyState::NotOffered);
    assert_eq!(hub_key_pin::verified_hub_key(&store, h, &old), None);
    assert!(attention_kinds(&store).iter().any(|(k, d)| k == "hub-signing-key-withdrawn" && d.contains(&fp)));
}

// E-126: status `hub {id, declared, match}`; control: another id reads match=false.
#[test]
fn e126_status_states_the_answering_hub_id_and_whether_it_matches() {
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    assert_eq!(hub_key_pin::hub_status(&store, Some(&id))["id"], Value::Null, "not observed yet");
    let sender = Sender::new(sender_cfg(&e, &id), store.clone()).unwrap();
    assert!(sender.check_pin().is_ok());
    let st = hub_key_pin::hub_status(&store, Some(&id));
    assert_eq!((st["id"].as_str(), st["declared"].as_str(), st["match"].as_bool()), (Some(id.as_str()), Some(id.as_str()), Some(true)));
    e.restart_hub().unwrap();
    assert!(sender.check_pin().is_ok());
    assert_eq!(hub_key_pin::hub_status(&store, Some(&id))["match"], true, "same id after restart");
    // control: a sidecar declared for another hub sees this hub's id, match false
    let other = Sender::new(sender_cfg(&e, "ffffffffffffffff"), store.clone()).unwrap();
    assert!(other.check_pin().is_err());
    let st = hub_key_pin::hub_status(&store, Some("ffffffffffffffff"));
    assert_eq!((st["id"].as_str(), st["match"].as_bool()), (Some(id.as_str()), Some(false)));
}

// AR-1 LIMIT (E-37.5): a rogue key present at the FIRST trusted contact is pinned. Recorded as
// the design's known limit, not a pass: the pin defends against a later swap only.
#[test]
fn e37_ar1_limit_rogue_key_at_first_contact_is_pinned() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let rogue = format!("ed25519:{}", "d".repeat(64));
    let st = hub_key_pin::observe(&store, "0123456789abcdef", &json!({"hub_id": "0123456789abcdef", "signing_key": rogue}), true);
    assert_eq!(st, KeyState::NewlyPinned(rogue));
    eprintln!("LIMIT AR-1: a rogue key present at first trusted contact is pinned (by design)");
}

// Review r2 #2: threads observing DIFFERENT offers on an empty pin end with ONE pin, never replaced;
// the losers see a mismatch, not a silent re-pin.
#[test]
fn e37_concurrent_first_contacts_with_different_offers_pin_once() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let h = "0123456789abcdef";
    let offers: Vec<String> = (0..8).map(|i| format!("ed25519:{}", format!("{i:x}").repeat(64))).collect();
    let hs: Vec<_> = offers
        .iter()
        .cloned()
        .map(|off| {
            let st = store.clone();
            std::thread::spawn(move || hub_key_pin::observe(&st, h, &json!({"hub_id": h, "signing_key": off}), true))
        })
        .collect();
    let rs: Vec<KeyState> = hs.into_iter().map(|t| t.join().unwrap()).collect();
    let pinned = store.read_state(&hub_key_pin::pin_state_name(h)).unwrap().unwrap();
    assert_eq!(rs.iter().filter(|r| matches!(r, KeyState::NewlyPinned(_))).count(), 1, "{rs:?}");
    for r in &rs {
        match r {
            KeyState::NewlyPinned(f) | KeyState::Pinned(f) => assert_eq!(f, &pinned),
            KeyState::Mismatch { pinned: p, .. } => assert_eq!(p, &pinned),
            other => panic!("unexpected {other:?}"),
        }
    }
}

// Review r2 #2: no durable record, no pin.
#[test]
fn e37_no_record_means_no_pin() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    std::fs::create_dir_all(store.dir().join("attention.jsonl")).unwrap(); // the record cannot be written
    let v = json!({"hub_id": h, "signing_key": format!("ed25519:{}", "e".repeat(64))});
    assert!(matches!(hub_key_pin::observe(&store, h, &v, true), KeyState::Unverified(_)));
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name(h)).unwrap(), None);
}
