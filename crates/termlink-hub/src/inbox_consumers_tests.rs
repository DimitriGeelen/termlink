//! T-3700 unit tests for the inbox consumer list (AL-30, rulings E and D).

use super::*;
use ed25519_dalek::{Signer, SigningKey};

const HUB: &str = "cacc73ea32b121dd";
const T: &str = "inbox:cacc73ea32b121dd/fx-a";
const NOW: i64 = 1_800_000_000_000;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn proof(k: &SigningKey, agent: &str, instance: &str, topic: &str, pos: u64, ts: i64) -> ReadProof {
    let bytes = read_sign_bytes(HUB, topic, "channel.subscribe", pos, ts, agent, instance);
    ReadProof {
        agent: agent.into(),
        instance: instance.into(),
        ts_ms: ts,
        signer_pk: Some(hex(k.verifying_key().as_bytes())),
        sig: Some(hex(&k.sign(&bytes).to_bytes())),
    }
}

/// Cards: fx-a -> key 1, fx-b -> key 2. Revoked: whatever `rev` holds.
fn with_lookups<R>(rev: BTreeSet<String>, f: impl FnOnce(&Lookups<'_>) -> R) -> R {
    let card_key = |a: &str| -> Result<Option<[u8; 32]>, String> {
        Ok(match a {
            "fx-a" => Some(*key(1).verifying_key().as_bytes()),
            "fx-b" => Some(*key(2).verifying_key().as_bytes()),
            _ => None,
        })
    };
    let revoked = move || -> Result<BTreeSet<String>, String> { Ok(rev.clone()) };
    let l = Lookups { hub_id: Some(HUB), card_key: &card_key, revoked: &revoked };
    f(&l)
}

fn assess_one(p: Option<&ReadProof>) -> Result<Reader, Refusal> {
    with_lookups(BTreeSet::new(), |l| assess(T, "channel.subscribe", 7, p, "local", NOW, l))
}

#[test]
fn signed_read_by_carded_key_is_proven() {
    let r = assess_one(Some(&proof(&key(1), "fx-a", "fx-a.w1", T, 7, NOW))).unwrap();
    assert_eq!(r.strength, Strength::Proven);
    assert_eq!(r.agent, "fx-a");
    assert_eq!(r.key_fp.as_deref(), Some(termlink_session::signed_state::key_fingerprint(&key(1).verifying_key()).as_str()));
}

#[test]
fn no_reader_is_unnamed_claimed_with_transport() {
    let r = assess_one(None).unwrap();
    assert_eq!((r.agent.as_str(), r.instance.as_str(), r.strength.clone()), (UNNAMED, "local", Strength::Claimed));
}

#[test]
fn named_unsigned_reader_is_claimed() {
    let p = ReadProof { agent: "fx-a".into(), instance: "x".into(), ts_ms: NOW, signer_pk: None, sig: None };
    let r = assess_one(Some(&p)).unwrap();
    assert_eq!(r.strength, Strength::Claimed);
    assert_eq!(r.why, "unsigned");
}

#[test]
fn bad_signature_is_refused_never_downgraded() {
    let mut p = proof(&key(1), "fx-a", "i", T, 7, NOW);
    p.ts_ms += 1; // signature no longer covers the bytes
    let e = assess_one(Some(&p)).unwrap_err();
    assert_eq!(e.code, error_code::CHANNEL_SIGNATURE_INVALID);
}

#[test]
fn signature_for_another_topic_or_position_does_not_verify() {
    let p = proof(&key(1), "fx-a", "i", "inbox:cacc73ea32b121dd/fx-b", 7, NOW);
    assert!(assess_one(Some(&p)).is_err());
    let p = proof(&key(1), "fx-a", "i", T, 8, NOW);
    assert!(assess_one(Some(&p)).is_err());
}

#[test]
fn signed_by_a_key_not_on_the_named_card_is_refused() {
    // fx-z signs with its own key but names fx-a
    let e = assess_one(Some(&proof(&key(9), "fx-a", "i", T, 7, NOW))).unwrap_err();
    assert_eq!(e.code, error_code::CHANNEL_KEY_CARD_MISMATCH);
}

#[test]
fn signed_without_a_card_is_claimed() {
    let r = assess_one(Some(&proof(&key(9), "fx-z", "i", T, 7, NOW))).unwrap();
    assert_eq!((r.strength, r.why.as_str()), (Strength::Claimed, "no key card"));
}

#[test]
fn revoked_key_is_refused() {
    let fp = termlink_session::signed_state::key_fingerprint(&key(1).verifying_key());
    let e = with_lookups([fp].into_iter().collect(), |l| {
        assess(T, "channel.subscribe", 7, Some(&proof(&key(1), "fx-a", "i", T, 7, NOW)), "local", NOW, l)
    })
    .unwrap_err();
    assert_eq!(e.code, error_code::KEY_REVOKED);
}

#[test]
fn skewed_clock_is_claimed_not_refused() {
    let p = proof(&key(1), "fx-a", "i", T, 7, NOW - PROOF_WINDOW_MS - 1);
    let r = assess_one(Some(&p)).unwrap();
    assert_eq!((r.strength, r.why.as_str()), (Strength::Claimed, "clock skew"));
    // inside the window proves
    let p = proof(&key(1), "fx-a", "i", T, 7, NOW - PROOF_WINDOW_MS);
    assert_eq!(assess_one(Some(&p)).unwrap().strength, Strength::Proven);
}

#[test]
fn no_hub_identity_means_claimed() {
    let card_key = |_: &str| -> Result<Option<[u8; 32]>, String> { Ok(None) };
    let revoked = || -> Result<BTreeSet<String>, String> { Ok(BTreeSet::new()) };
    let l = Lookups { hub_id: None, card_key: &card_key, revoked: &revoked };
    let r = assess(T, "channel.subscribe", 7, Some(&proof(&key(1), "fx-a", "i", T, 7, NOW)), "local", NOW, &l).unwrap();
    assert_eq!(r.strength, Strength::Claimed);
}

#[test]
fn parse_reader_rules() {
    assert_eq!(parse_reader(&serde_json::json!({})).unwrap(), None);
    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "Bad Name"}})).is_err());
    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "sig": "00"}})).is_err());
    // r1 CC 1 / OC 1: the instance is required and may not be the ack sentinel
    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a"}})).is_err());
    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "instance": "-"}})).is_err());
    let p = parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "instance": "i", "ts_ms": 5}})).unwrap().unwrap();
    assert_eq!((p.agent.as_str(), p.ts_ms), ("fx-a", 5));
}

// ---- record / consumers / bind ----

fn dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn proven(agent: &str, instance: &str, n: u8) -> Reader {
    Reader {
        agent: agent.into(),
        instance: instance.into(),
        strength: Strength::Proven,
        why: String::new(),
        key_fp: Some(termlink_session::signed_state::key_fingerprint(&key(n).verifying_key())),
    }
}

fn claimed_r(agent: &str) -> Reader {
    claimed(agent, "local", "unsigned")
}

fn rows() -> (Vec<serde_json::Value>, impl FnMut(serde_json::Value) -> Result<(), String>) {
    (Vec::new(), |_r| Ok(()))
}

fn listed(d: &Path, topic: &str, now: u64) -> Vec<(String, String, String)> {
    let v = consumers(d, topic, DEFAULT_WINDOW_S, now).unwrap();
    v["consumers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["agent"].as_str().unwrap().to_string(),
                e["instance"].as_str().unwrap().to_string(),
                e["strength"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn every_action_records_including_read_only() {
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap(); // reads only, never acks (FCW-209)
    record(d.path(), T, &proven("fx-b", ACK_INSTANCE, 2), Action::Ack, n, &mut log).unwrap();
    let l = listed(d.path(), T, n);
    assert_eq!(l.len(), 3);
    assert!(l.contains(&("fx-z".into(), "local".into(), "claimed".into())));
    assert!(l.contains(&("fx-a".into(), "w1".into(), "proven".into())));
}

#[test]
fn window_hides_old_readers() {
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n - 2 * 86_400_000, &mut log).unwrap();
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
    assert_eq!(listed(d.path(), T, n).len(), 1);
}

#[test]
fn seventeenth_reader_evicts_the_oldest_with_a_row() {
    let d = dir();
    let mut logged = Vec::new();
    let mut log = |r: serde_json::Value| {
        logged.push(r);
        Ok(())
    };
    let n = NOW as u64;
    for i in 0..17u64 {
        record(d.path(), T, &claimed_r(&format!("fx-{i}")), Action::Read, n + i, &mut log).unwrap();
    }
    let l = listed(d.path(), T, n + 100);
    assert_eq!(l.len(), MAX_PER_TOPIC);
    assert!(!l.iter().any(|e| e.0 == "fx-0"));
    assert!(logged.iter().any(|r| r["event"] == "inbox-consumer-evicted" && r["agent"] == "fx-0"));
}

#[test]
fn state_survives_a_restart() {
    let d = dir();
    let (_, mut log) = rows();
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, NOW as u64, &mut log).unwrap();
    reset(d.path());
    assert_eq!(listed(d.path(), T, NOW as u64).len(), 1);
}

#[test]
fn unreadable_list_is_an_error_never_empty() {
    let d = dir();
    std::fs::write(consumers_path(d.path()), b"{not json").unwrap();
    reset(d.path());
    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now_wall_ms()).is_err());
}

#[test]
fn bind_refuses_everyone_but_the_bound_key() {
    let d = dir();
    let mut logged = Vec::new();
    let n = NOW as u64;
    {
        let mut log = |r: serde_json::Value| {
            logged.push(r);
            Ok(())
        };
        bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, n, &mut log).unwrap();
        // the bound agent, proven with the bound key
        record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
        // another agent, proven
        assert_eq!(record(d.path(), T, &proven("fx-b", "w1", 2), Action::Ack, n, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
        // the bound agent's NAME, unsigned (ruling E: a name alone never passes)
        let named = claimed("fx-a", "x", "unsigned");
        assert_eq!(record(d.path(), T, &named, Action::Read, n, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
        // the bound agent's name, proven but by another key (a rotated card needs a re-bind)
        assert!(record(d.path(), T, &proven("fx-a", "w1", 3), Action::Claim, n, &mut log).is_err());
        // a key-less peek (ruling D: no exception)
        assert!(record(d.path(), T, &claimed(UNNAMED, "local", "no reader named"), Action::Read, n, &mut log).is_err());
    }
    assert_eq!(logged[0]["event"], "inbox-bind");
    assert!(logged.iter().any(|r| r["event"] == "inbox-bound-refused" && r["reader"] == "fx-b"));
    // other inboxes are not affected
    let (_, mut log) = rows();
    record(d.path(), "inbox:cacc73ea32b121dd/fx-b", &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
}

#[test]
fn bind_needs_a_card_and_an_inbox_topic() {
    let d = dir();
    let (_, mut log) = rows();
    assert!(bind(d.path(), T, "fx-a", None, &no_rev, NOW as u64, &mut log).is_err());
    assert!(bind(d.path(), "chat:x", "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut log).is_err());
}

#[test]
fn bind_writes_its_admin_row_first() {
    let d = dir();
    let mut failing = |_r: serde_json::Value| Err("admin log full".to_string());
    assert!(bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut failing).is_err());
    assert!(!bindings_path(d.path()).exists(), "no row, no bind");
}

#[test]
fn unbind_reopens_and_unreadable_bindings_fail_closed() {
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, n, &mut log).unwrap();
    assert!(unbind(d.path(), T, &mut log).unwrap());
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
    std::fs::write(bindings_path(d.path()), b"garbage").unwrap();
    assert_eq!(record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
}

#[test]
fn ack_reader_from_cards() {
    let cards = || -> Result<Vec<(String, [u8; 32])>, String> { Ok(vec![("fx-a".into(), *key(1).verifying_key().as_bytes())]) };
    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(1).verifying_key(), Some("fx-a"), NOW, NOW, &cards, l));
    assert_eq!((r.agent.as_str(), r.strength), ("fx-a", Strength::Proven));
    // r1 OC 6: an uncarded ack is listed under its claimed from_agent
    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(9).verifying_key(), Some("fx-z"), NOW, NOW, &cards, l));
    assert_eq!((r.agent.as_str(), r.strength), ("fx-z", Strength::Claimed));
    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(9).verifying_key(), Some("Not A Name"), NOW, NOW, &cards, l));
    assert_eq!(r.agent, UNNAMED);
    // r1 CC 7: a replayed old receipt by the carded key is only claimed
    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(1).verifying_key(), Some("fx-a"), NOW - PROOF_WINDOW_MS - 1, NOW, &cards, l));
    assert_eq!((r.strength, r.why.as_str()), (Strength::Claimed, "stale ack"));
}

fn no_rev() -> Result<BTreeSet<String>, String> {
    Ok(BTreeSet::new())
}

#[test]
fn a_claimed_reader_never_merges_into_a_proven_entry() {
    // r1 CC 1 / OC 4: an unsigned reader naming the owner's agent AND instance is its own entry
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    record(d.path(), T, &proven("fx-a", "inst-a", 1), Action::Read, n, &mut log).unwrap();
    record(d.path(), T, &claimed("fx-a", "inst-a", "unsigned"), Action::Read, n + 1, &mut log).unwrap();
    record(d.path(), T, &proven("fx-a", "inst-a", 1), Action::Read, n + 2, &mut log).unwrap();
    let l = listed(d.path(), T, n + 3);
    assert_eq!(l.len(), 2, "{l:?}");
    assert!(l.contains(&("fx-a".into(), "inst-a".into(), "proven".into())));
    assert!(l.contains(&("fx-a".into(), "inst-a".into(), "claimed".into())));
}

#[test]
fn replay_across_hub_or_method_does_not_verify() {
    let k = key(1);
    let other_hub = read_sign_bytes("ffffffffffffffff", T, "channel.subscribe", 7, NOW, "fx-a", "i");
    let other_method = read_sign_bytes(HUB, T, "channel.claim", 7, NOW, "fx-a", "i");
    for b in [other_hub, other_method] {
        let p = ReadProof {
            agent: "fx-a".into(),
            instance: "i".into(),
            ts_ms: NOW,
            signer_pk: Some(hex(k.verifying_key().as_bytes())),
            sig: Some(hex(&k.sign(&b).to_bytes())),
        };
        assert_eq!(assess_one(Some(&p)).unwrap_err().code, error_code::CHANNEL_SIGNATURE_INVALID);
    }
}

#[test]
fn extreme_timestamp_does_not_overflow() {
    let p = proof(&key(1), "fx-a", "i", T, 7, i64::MIN);
    assert_eq!(assess_one(Some(&p)).unwrap().why, "clock skew");
}

#[test]
fn skewed_proof_is_refused_after_a_bind() {
    let d = dir();
    let (_, mut log) = rows();
    bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut log).unwrap();
    let r = assess_one(Some(&proof(&key(1), "fx-a", "i", T, 7, NOW - PROOF_WINDOW_MS - 1))).unwrap();
    assert_eq!(record(d.path(), T, &r, Action::Read, NOW as u64, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
}

#[test]
fn bind_refuses_a_revoked_key_and_a_long_topic() {
    let d = dir();
    let (_, mut log) = rows();
    let fp = termlink_session::signed_state::key_fingerprint(&key(1).verifying_key());
    let rev = move || -> Result<BTreeSet<String>, String> { Ok([fp.clone()].into_iter().collect()) };
    assert!(bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &rev, NOW as u64, &mut log).is_err());
    let long = format!("inbox:{}", "x".repeat(MAX_TOPIC_LEN));
    assert!(bind(d.path(), &long, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut log).is_err());
}

#[test]
fn a_long_topic_is_not_recorded() {
    let d = dir();
    let (_, mut log) = rows();
    let long = format!("inbox:{}", "x".repeat(MAX_TOPIC_LEN));
    record(d.path(), &long, &claimed_r("fx-z"), Action::Read, NOW as u64, &mut log).unwrap();
    assert!(listed(d.path(), &long, NOW as u64).is_empty());
}

#[test]
fn a_full_table_never_evicts_an_inbox_read_inside_the_window() {
    let d = dir();
    let logged = std::cell::RefCell::new(Vec::new());
    let mut log = |r: serde_json::Value| {
        logged.borrow_mut().push(r);
        Ok(())
    };
    let n = NOW as u64;
    for i in 0..MAX_TOPICS {
        record(d.path(), &format!("inbox:h/t{i}"), &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
    }
    record(d.path(), "inbox:h/new", &claimed_r("fx-z"), Action::Read, n + 1, &mut log).unwrap();
    assert!(listed(d.path(), "inbox:h/new", n + 1).is_empty(), "not recorded");
    assert_eq!(listed(d.path(), "inbox:h/t0", n + 1).len(), 1, "the oldest active inbox kept");
    assert!(logged.borrow().iter().any(|r| r["event"] == "inbox-consumer-table-full"));
    // a day later the oldest inbox is out of the window and may be evicted
    let later = n + 2 * 86_400_000;
    record(d.path(), "inbox:h/new", &claimed_r("fx-z"), Action::Read, later, &mut log).unwrap();
    assert_eq!(listed(d.path(), "inbox:h/new", later).len(), 1);
}

#[test]
fn a_corrupt_table_is_moved_aside_and_reported_incomplete() {
    let d = dir();
    std::fs::write(consumers_path(d.path()), b"{not json").unwrap();
    reset(d.path());
    let mut logged = Vec::new();
    let mut log = |r: serde_json::Value| {
        logged.push(r);
        Ok(())
    };
    let now = now_wall_ms();
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, now, &mut log).unwrap();
    assert!(logged.iter().any(|r| r["event"] == "inbox-consumers-reset"), "{logged:?}");
    let aside: Vec<_> = std::fs::read_dir(d.path()).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().contains(".corrupt-")).collect();
    assert_eq!(aside.len(), 1, "the corrupt file is kept, not overwritten");
    // inside the window the history is incomplete: an error, never a clean list
    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now).is_err());
    // once the window no longer reaches back before the reset, the list is served again
    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now + DEFAULT_WINDOW_S * 1000 + 60_000).is_ok());
}

#[test]
fn a_failed_save_makes_the_list_an_error_never_a_list() {
    // E-169 / FCW-125 (m4 survived the first round-2 mutant run: nothing made a SAVE fail after load)
    let d = dir();
    let (_, mut log) = rows();
    let n = now_wall_ms();
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, n).is_ok());
    // the table path becomes a directory: the next save fails
    std::fs::remove_file(consumers_path(d.path())).unwrap();
    std::fs::create_dir(consumers_path(d.path())).unwrap();
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, n).is_err(), "a list that could not be saved is not served");
}
