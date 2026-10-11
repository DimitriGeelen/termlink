//! B4 tests for C-2 (store.rs). Run with `--features failpoints`.
#![cfg(feature = "failpoints")]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use termlink_sidecar::store::*;
use termlink_sidecar::testkit::Estate;

const S: &str = "fp-sender";
const C: &str = "conv-1";

fn fresh() -> (Estate, Store) {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-b").unwrap();
    (e, s)
}

fn rec_count(s: &Store) -> usize {
    fs::read_dir(s.dir().join("msgs").join(C))
        .map(|d| d.filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".rec")).count())
        .unwrap_or(0)
}

#[test]
fn store_layout_and_modes() {
    let (_e, s) = fresh();
    assert!(matches!(s.put(S, C, "m1", b"hello").unwrap(), PutOutcome::Stored { seq: 0 }));
    for d in ["", "log", "msgs", "msgs/conv-1", "claims", "tmp"] {
        let m = fs::metadata(s.dir().join(d)).unwrap().permissions().mode() & 0o777;
        assert_eq!(m, 0o700, "{d}");
    }
    for f in ["lock", "log/conv-1.jsonl", "msgs/conv-1/0-m1.rec"] {
        let m = fs::metadata(s.dir().join(f)).unwrap().permissions().mode() & 0o777;
        assert_eq!(m, 0o600, "{f}");
    }
    let (h, body) = s.read_record(C, 0, "m1").unwrap();
    assert_eq!((h.len, h.sender.as_str(), body.as_slice()), (5, S, &b"hello"[..]));
}

#[test]
fn store_refuses_unmarked_root() {
    let d = std::env::temp_dir().join(format!("tl-store-unmarked-{}", std::process::id()));
    fs::create_dir_all(&d).unwrap();
    let r = Store::open(&d, "a");
    assert!(matches!(r, Err(StoreError::Estate(_))), "{:?}", r.err());
    assert!(!d.join("sidecar").exists());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn store_rejects_path_traversal_names() {
    let (_e, s) = fresh();
    for bad in ["../x", "a/b", "", ".."] {
        assert!(matches!(s.put(S, C, bad, b"x"), Err(StoreError::BadName(_))), "{bad}");
        assert!(matches!(s.put(S, bad, "m", b"x"), Err(StoreError::BadName(_))), "{bad}");
    }
}

/// FC-3: kill at every named step of 4.3, restart, and the message is either
/// absent-and-unreported or present with the same id; never reported-and-missing.
#[test]
fn store_kill_at_every_step_never_reported_and_missing() {
    for step in Step::ALL {
        let e = Estate::create_without_hub().unwrap();
        let root = e.store_root();
        let reported;
        {
            let s = Store::open(&root, "agent-b").unwrap();
            arm(step);
            let r = s.put(S, C, "m1", b"payload");
            disarm();
            reported = r.is_ok();
            assert!(matches!(r, Err(StoreError::Failpoint(x)) if x == step), "{step:?}");
        } // dropped without cleanup: lock released like a dead process
        let s = Store::open(&root, "agent-b").unwrap();
        assert_eq!(fs::read_dir(s.dir().join("tmp")).unwrap().count(), 0, "{step:?}");
        assert!(!reported);
        let present = rec_count(&s) == 1;
        let status = s.verify_chain(C).unwrap();
        match step {
            Step::WriteTmp | Step::FsyncFile | Step::Rename => {
                assert!(!present, "{step:?}: absent");
                assert_eq!(status, ChainStatus::Intact { len: 0, head: GENESIS.into() }, "{step:?}");
            }
            _ => {
                // Present, and recovery made the chain agree with the record.
                assert!(present, "{step:?}: present");
                assert!(matches!(status, ChainStatus::Intact { len: 1, .. }), "{step:?} {status:?}");
                assert_eq!(s.read_record(C, 0, "m1").unwrap().1, b"payload");
            }
        }
        // The retry is idempotent at worst; it never produces a second record.
        let again = s.put(S, C, "m1", b"payload").unwrap();
        assert!(matches!(again, PutOutcome::Stored { .. } | PutOutcome::Duplicate { .. }));
        assert_eq!(rec_count(&s), 1, "{step:?}");
        assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 1, .. }), "{step:?}");
    }
}

#[test]
fn store_torn_chain_line_is_truncated_not_reported() {
    let e = Estate::create_without_hub().unwrap();
    let root = e.store_root();
    {
        let s = Store::open(&root, "agent-b").unwrap();
        arm(Step::AppendChainTorn);
        let _ = s.put(S, C, "m1", b"p");
        disarm();
        let raw = fs::read(s.dir().join("log/conv-1.jsonl")).unwrap();
        assert!(!raw.is_empty() && raw.last() != Some(&b'\n'), "tail must be torn");
    }
    let s = Store::open(&root, "agent-b").unwrap();
    assert_eq!(s.recovery().torn_lines_truncated, vec![C.to_string()]);
    assert_eq!(s.recovery().appended, vec![(C.to_string(), 0)]);
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 1, .. }));
}

#[test]
fn store_chain_line_without_record_is_loud_and_not_restored() {
    let e = Estate::create_without_hub().unwrap();
    let root = e.store_root();
    {
        let s = Store::open(&root, "agent-b").unwrap();
        s.put(S, C, "m1", b"p").unwrap();
        fs::remove_file(s.dir().join("msgs/conv-1/0-m1.rec")).unwrap();
    }
    let s = Store::open(&root, "agent-b").unwrap();
    assert_eq!(s.recovery().missing, vec![(C.to_string(), 0, "m1".to_string())]);
    assert!(s.attention_entries().iter().any(|l| l.contains("chain-line-without-record")));
    // STORED is never re-reported for it.
    assert!(matches!(s.put(S, C, "m1", b"p"), Err(StoreError::MissingRecord { .. })));
}

/// FC-9: a second sidecar on the same store is refused.
#[test]
fn store_second_open_refused_then_allowed_after_drop() {
    let e = Estate::create_without_hub().unwrap();
    let root = e.store_root();
    let first = Store::open(&root, "agent-b").unwrap();
    assert!(matches!(Store::open(&root, "agent-b"), Err(StoreError::Locked)));
    // A different agent has its own lock.
    Store::open(&root, "agent-c").unwrap();
    drop(first);
    Store::open(&root, "agent-b").unwrap();
}

#[test]
fn store_corrupt_record_is_quarantined_on_read_and_never_returned() {
    let (_e, s) = fresh();
    s.put(S, C, "m1", b"original").unwrap();
    let p = s.dir().join("msgs/conv-1/0-m1.rec");
    let mut b = fs::read(&p).unwrap();
    let n = b.len();
    b[n - 1] ^= 0xff;
    fs::write(&p, b).unwrap();
    let r = s.read_record(C, 0, "m1");
    assert!(matches!(r, Err(StoreError::Quarantined(_))), "{:?}", r.err());
    assert!(!p.exists());
    assert!(s.dir().join("msgs/conv-1/0-m1.rec.bad").exists());
    assert!(s.attention_entries().iter().any(|l| l.contains("quarantined")));
}

#[test]
fn store_recovery_quarantines_corrupt_record_without_chain_line() {
    let e = Estate::create_without_hub().unwrap();
    let root = e.store_root();
    {
        let s = Store::open(&root, "agent-b").unwrap();
        arm(Step::AppendChain);
        let _ = s.put(S, C, "m1", b"original");
        disarm();
        let p = s.dir().join("msgs/conv-1/0-m1.rec");
        let mut b = fs::read(&p).unwrap();
        let n = b.len();
        b[n - 1] ^= 1;
        fs::write(&p, b).unwrap();
    }
    let s = Store::open(&root, "agent-b").unwrap();
    assert_eq!(s.recovery().quarantined.len(), 1);
    assert!(s.recovery().appended.is_empty());
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 0, .. }));
}

/// FC-15: edit, insert or delete a middle line; the walk names the break.
#[test]
fn store_chain_break_detected_for_edit_insert_delete() {
    let (_e, s) = fresh();
    for i in 0..4 {
        s.put(S, C, &format!("m{i}"), b"x").unwrap();
    }
    let log = s.dir().join("log/conv-1.jsonl");
    let orig = fs::read_to_string(&log).unwrap();
    let lines: Vec<&str> = orig.lines().collect();
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 4, .. }));

    // edit line 1 (change the msg_id, keep the hash)
    let mut v = lines.clone();
    let edited = v[1].replace("\"m1\"", "\"mX\"");
    v[1] = &edited;
    fs::write(&log, v.join("\n") + "\n").unwrap();
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Broken { seq: 1, .. }));

    // delete line 1
    let mut v = lines.clone();
    v.remove(1);
    fs::write(&log, v.join("\n") + "\n").unwrap();
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Broken { seq: 1, .. }));

    // insert a copy of line 0 after line 0
    let mut v = lines.clone();
    v.insert(1, lines[0]);
    fs::write(&log, v.join("\n") + "\n").unwrap();
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Broken { seq: 1, .. }));

    // Tail truncation is not detectable and the store says so.
    fs::write(&log, lines[..3].join("\n") + "\n").unwrap();
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 3, .. }));
    assert!(TAIL_NOTE.contains("not yet detectable"));
}

/// FC-10: same key same digest is idempotent; different digest records CONFLICT.
#[test]
fn store_conflict_recorded_first_record_untouched() {
    let (_e, s) = fresh();
    assert_eq!(s.put(S, C, "m1", b"one").unwrap(), PutOutcome::Stored { seq: 0 });
    assert_eq!(s.put(S, C, "m1", b"one").unwrap(), PutOutcome::Duplicate { seq: 0 });
    let r = s.put(S, C, "m1", b"two").unwrap();
    assert_eq!(r, PutOutcome::Conflict { first_seq: 0, conflict_seq: 1 });
    // Repeating the same conflicting body does not grow the chain.
    assert_eq!(s.put(S, C, "m1", b"two").unwrap(), PutOutcome::Conflict { first_seq: 0, conflict_seq: 1 });
    assert_eq!(s.read_record(C, 0, "m1").unwrap().1, b"one");
    assert_eq!(rec_count(&s), 1);
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 2, .. }));
    // Another sender may reuse the msg_id: different key.
    assert!(matches!(s.put("other", C, "m1", b"two").unwrap(), PutOutcome::Stored { .. }));
}

#[test]
fn store_stage_events_extend_the_chain() {
    let (_e, s) = fresh();
    s.put(S, C, "m1", b"x").unwrap();
    s.append_event(C, "CLAIMED", "m1", S, "d").unwrap();
    s.append_event(C, "HANDED_OVER", "m1", S, "d").unwrap();
    assert!(matches!(s.verify_chain(C).unwrap(), ChainStatus::Intact { len: 3, .. }));
}

/// SI-30: exactly one creator wins, concurrently; the claim survives restart.
#[test]
fn store_claim_exactly_one_winner_and_survives_restart() {
    let e = Estate::create_without_hub().unwrap();
    let root = e.store_root();
    let s = Arc::new(Store::open(&root, "agent-b").unwrap());
    let wins: usize = (0..16)
        .map(|i| {
            let s = s.clone();
            std::thread::spawn(move || s.claim("m1", 1, &format!("nonce-{i}"), 100 + i).unwrap() == ClaimOutcome::Won)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap() as usize)
        .sum();
    assert_eq!(wins, 1);
    // A later attempt is a different file.
    assert_eq!(s.claim("m1", 2, "n2", 200).unwrap(), ClaimOutcome::Won);
    drop(s);
    let s = Store::open(&root, "agent-b").unwrap();
    assert_eq!(s.claim("m1", 1, "late", 300).unwrap(), ClaimOutcome::Held);
    assert_eq!(s.claims_for("m1").unwrap(), vec![1, 2]);
    let body = fs::read_to_string(s.dir().join("claims/m1.1")).unwrap();
    assert!(body.starts_with("nonce-"), "{body}");
}
