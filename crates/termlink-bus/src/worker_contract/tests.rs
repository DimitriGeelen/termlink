//! Conformance cases CC-04/05/06/07/14/16/18/19 (+CC-23/24 credential and T5 bound)
//! with injected time and a temp dir only (T-3293).
use super::*;

fn store() -> (tempfile::TempDir, WcStore) {
    let d = tempfile::tempdir().unwrap();
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    (d, s)
}

fn alloc(s: &WcStore, now: i64) -> Allocation {
    s.allocate(&AllocateRequest::default(), now).unwrap()
}

fn t2(a: &Allocation) -> TransitionRequest {
    TransitionRequest {
        dispatch_id: a.dispatch_id.clone(),
        attempt: a.attempt,
        nonce: a.nonce.clone(),
        supervisor_token: Some(a.supervisor_token.clone()),
        expected_prior: "dispatched".into(),
        to: "running".into(),
        supervisor_gen: 1,
        ..Default::default()
    }
}

fn t3(a: &Allocation, digest: &str) -> TransitionRequest {
    TransitionRequest {
        expected_prior: "running".into(),
        to: "done".into(),
        evidence_digest: Some(digest.into()),
        exit_code: Some(0),
        handback: Some(Handback {
            attempt: Some(a.attempt),
            nonce: Some(a.nonce.clone()),
            status: Some("complete".into()),
            summary: None,
            commission: Some(a.dispatch_id.clone()),
        }),
        ..t2(a)
    }
}

fn reason(e: WcError) -> &'static str {
    match e {
        WcError::Fence { reason, .. } => reason.as_str(),
        other => panic!("expected fence, got {other:?}"),
    }
}

fn history_len(s: &WcStore, id: &str) -> usize {
    let v = s.get(id).unwrap();
    v["attempts"].as_array().unwrap().iter().map(|a| a["history"].as_array().unwrap().len()).sum()
}

fn mark_lost(s: &WcStore, id: &str, attempt: i64, ts: i64) {
    // The sweep (S2) writes T4; S1 tests set the state directly.
    let c = s.conn.lock().unwrap();
    c.execute("UPDATE wc_attempts SET state='lost' WHERE dispatch_id=?1 AND attempt=?2", params![id, attempt]).unwrap();
    c.execute(
        "INSERT INTO wc_history (dispatch_id, attempt, from_state, to_state, writer, ts) VALUES (?1,?2,'running','lost','hub',?3)",
        params![id, attempt, ts],
    )
    .unwrap();
}

#[test]
fn tables_created_by_bus_open_and_idempotent() {
    let d = tempfile::tempdir().unwrap();
    for _ in 0..2 {
        let bus = crate::Bus::open(d.path()).unwrap();
        let a = bus.wc_allocate(&AllocateRequest::default(), 1000).unwrap();
        assert!(bus.wc_get(&a.dispatch_id).is_ok());
    }
}

#[test]
fn secrets_stored_only_as_sha256() {
    let (_d, s) = store();
    let a = alloc(&s, 1000);
    let (n, t): (String, String) = s
        .conn
        .lock()
        .unwrap()
        .query_row("SELECT nonce_sha256, supervisor_token_sha256 FROM wc_attempts", [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert_eq!(n, sha256_hex(&a.nonce));
    assert_eq!(t, sha256_hex(&a.supervisor_token));
    let got = s.get(&a.dispatch_id).unwrap().to_string();
    assert!(!got.contains(&a.nonce) && !got.contains(&a.supervisor_token));
}

// CC-18
#[test]
fn cc18_caller_chosen_dispatch_id() {
    let (_d, s) = store();
    let x = "D-01J9ZZZZZZZZZZZZZZZZZZZZZZ".to_string();
    assert!(is_valid_dispatch_id(&x));
    let a = s.allocate(&AllocateRequest { dispatch_id: Some(x.clone()), ..Default::default() }, 1).unwrap();
    assert_eq!(a.dispatch_id, x);
    match s.allocate(&AllocateRequest { dispatch_id: Some(x.clone()), ..Default::default() }, 2) {
        Err(WcError::IdTaken(id)) => assert_eq!(id, x),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        s.allocate(&AllocateRequest { dispatch_id: Some("D-nope".into()), ..Default::default() }, 3),
        Err(WcError::BadId(_))
    ));
    let auto = alloc(&s, 4);
    assert!(is_valid_dispatch_id(&auto.dispatch_id));
    assert_ne!(auto.dispatch_id, x);
}

// CC-06
#[test]
fn cc06_retry_while_old_worker_survives() {
    let (_d, s) = store();
    let a1 = alloc(&s, 10);
    s.transition(&t2(&a1), 11).unwrap();
    let again = AllocateRequest { attempt_of: Some(a1.dispatch_id.clone()), ..Default::default() };
    assert_eq!(reason(s.allocate(&again, 12).unwrap_err()), "cas_mismatch");
    assert_eq!(s.get(&a1.dispatch_id).unwrap()["attempts"].as_array().unwrap().len(), 1);
    mark_lost(&s, &a1.dispatch_id, 1, 20);
    let a2 = s.allocate(&again, 21).unwrap();
    assert_eq!(a2.attempt, 2);
    assert_ne!(a2.nonce, a1.nonce);
    // late running-path write from A1 (T-RECLAIM with the A1 nonce): A1 is not current, so it is
    // stale_attempt before the CAS is even looked at (§3.3 fence order, T-3529).
    let mut late = t2(&a1);
    late.expected_prior = "running".into();
    late.supervisor_gen = 2;
    assert_eq!(reason(s.transition(&late, 22).unwrap_err()), "stale_attempt");
    let v = s.get(&a1.dispatch_id).unwrap();
    assert_eq!(v["attempts"][1]["state"], "dispatched");
}

// CC-06 (T-3529): a reclaim naming a stale attempt is `stale_attempt` even though the
// transition itself is not legal (the attempt is lost, and the write says prior=lost).
// Fence order per contract §3.3: attempt currency is tested before transition legality.
#[test]
fn cc06_reclaim_of_stale_attempt_is_stale_not_forbidden() {
    let (_d, s) = store();
    let a1 = alloc(&s, 10);
    s.transition(&t2(&a1), 11).unwrap();
    mark_lost(&s, &a1.dispatch_id, 1, 20);
    let a2 = s.allocate(&AllocateRequest { attempt_of: Some(a1.dispatch_id.clone()), ..Default::default() }, 21).unwrap();
    assert_eq!(a2.attempt, 2);
    // The vector's reclaim: expected_prior is the attempt's own state (lost), to=running, gen+1.
    let reclaim = TransitionRequest {
        expected_prior: "lost".into(),
        supervisor_gen: 2,
        ..t2(&a1)
    };
    assert_eq!(reason(s.transition(&reclaim, 22).unwrap_err()), "stale_attempt");
    assert_eq!(s.get(&a1.dispatch_id).unwrap()["attempts"][0]["state"], "lost");
}

#[test]
fn stale_attempt_on_running_path() {
    let (_d, s) = store();
    let a1 = alloc(&s, 10);
    // A1 is dispatched (non-terminal) so no A2 can exist; make A1 non-current by force.
    mark_lost(&s, &a1.dispatch_id, 1, 15);
    let a2 = s.allocate(&AllocateRequest { attempt_of: Some(a1.dispatch_id.clone()), ..Default::default() }, 16).unwrap();
    // revive A1 as 'dispatched' to exercise the stale_attempt guard directly
    s.conn.lock().unwrap().execute("UPDATE wc_attempts SET state='dispatched' WHERE attempt=1", []).unwrap();
    assert_eq!(reason(s.transition(&t2(&a1), 17).unwrap_err()), "stale_attempt");
    assert_eq!(s.get(&a2.dispatch_id).unwrap()["attempts"][1]["state"], "dispatched");
}

// CC-07
#[test]
fn cc07_stale_attempt_write_rejected() {
    let (_d, s) = store();
    let a1 = alloc(&s, 10);
    let mut r1 = t2(&a1);
    s.transition(&r1, 11).unwrap();
    // A1 finishes its run: evidence persisted at 12, lost entry at 20.
    mark_lost(&s, &a1.dispatch_id, 1, 20);
    let a2 = s.allocate(&AllocateRequest { attempt_of: Some(a1.dispatch_id.clone()), ..Default::default() }, 21).unwrap();
    s.transition(&t2(&a2), 22).unwrap();
    // zombie A1 supervisor submits "a completion of A2" with n1 -> bad_nonce
    let mut bad = t3(&a2, "X");
    bad.nonce = a1.nonce.clone();
    assert_eq!(reason(s.transition(&bad, 23).unwrap_err()), "bad_nonce");
    // the same with n1 for A1: accepted as T5 (persisted before the lost entry), A2 untouched
    r1 = t3(&a1, "E1");
    r1.persisted_at = Some(12);
    let out = s.transition(&r1, 24).unwrap();
    assert_eq!(out.record["state"], "done");
    let v = s.get(&a1.dispatch_id).unwrap();
    assert_eq!(v["attempts"][1]["state"], "running");
    assert_eq!(v["attempts"][1]["evidence_digest"], Value::Null);
}

// CC-24 (T5 bound)
#[test]
fn cc24_late_t5_postdating_lost_is_forbidden() {
    let (_d, s) = store();
    let a = alloc(&s, 10);
    s.transition(&t2(&a), 11).unwrap();
    mark_lost(&s, &a.dispatch_id, 1, 50);
    let mut late = t3(&a, "E");
    late.persisted_at = Some(51);
    assert_eq!(reason(s.transition(&late, 60).unwrap_err()), "forbidden_transition");
    late.persisted_at = None;
    assert_eq!(reason(s.transition(&late, 61).unwrap_err()), "forbidden_transition");
    late.persisted_at = Some(50);
    assert_eq!(s.transition(&late, 62).unwrap().record["state"], "done");
}

// CC-04
#[test]
fn cc04_supervisor_crash_after_commit_is_idempotent() {
    let (_d, s) = store();
    let a = alloc(&s, 10);
    s.transition(&t2(&a), 11).unwrap();
    let done = s.transition(&t3(&a, "X"), 12).unwrap();
    assert!(!done.deduped);
    let n = history_len(&s, &a.dispatch_id);
    let again = s.transition(&t3(&a, "X"), 13).unwrap();
    assert!(again.deduped);
    assert_eq!(again.record["state"], "done");
    assert_eq!(history_len(&s, &a.dispatch_id), n);
}

// CC-05
#[test]
fn cc05_duplicate_finalisation() {
    let (_d, s) = store();
    let a = alloc(&s, 10);
    s.transition(&t2(&a), 11).unwrap();
    s.transition(&t3(&a, "X"), 12).unwrap();
    let before = s.get(&a.dispatch_id).unwrap()["attempts"][0].clone();
    assert!(s.transition(&t3(&a, "X"), 13).unwrap().deduped);
    assert_eq!(reason(s.transition(&t3(&a, "Y"), 14).unwrap_err()), "done_conflict");
    let after = s.get(&a.dispatch_id).unwrap()["attempts"][0].clone();
    assert_eq!(after["evidence_digest"], "X");
    assert_eq!(after["state"], before["state"]);
    assert_eq!(after["outcome"], before["outcome"]);
    // the conflict is logged on the history (outside the lifecycle)
    let h = after["history"].as_array().unwrap();
    assert_eq!(h.last().unwrap()["fence_reason"], "done_conflict");
}

// CC-14
#[test]
fn cc14_stale_supervisor_generation() {
    let (_d, s) = store();
    let a = alloc(&s, 10);
    s.transition(&t2(&a), 11).unwrap();
    let mut reclaim = t2(&a);
    reclaim.expected_prior = "running".into();
    reclaim.supervisor_gen = 2;
    s.transition(&reclaim, 12).unwrap();
    let old = t3(&a, "X"); // gen 1
    assert_eq!(reason(s.transition(&old, 13).unwrap_err()), "stale_gen");
    let mut new = t3(&a, "X");
    new.supervisor_gen = 2;
    assert_eq!(s.transition(&new, 14).unwrap().record["state"], "done");
    // reclaim must claim exactly prev+1
    let b = alloc(&s, 20);
    s.transition(&t2(&b), 21).unwrap();
    let mut skip = t2(&b);
    skip.expected_prior = "running".into();
    skip.supervisor_gen = 3;
    assert_eq!(reason(s.transition(&skip, 22).unwrap_err()), "stale_gen");
}

// CC-16 + CC-19
#[test]
// T-3577: the inline tuple type documents the table's columns; a type alias would hide them.
#[allow(clippy::type_complexity)]
fn cc16_outcome_derivation_and_cc19_paused() {
    let cases: [(&str, Option<i64>, bool, Option<&str>, bool, &str); 6] = [
        // name, exit, refused, hb status, hb nonce ok, expected
        ("clean", Some(0), false, Some("complete"), true, "succeeded"),
        ("exit1", Some(1), false, Some("complete"), true, "failed"),
        ("refused", Some(3), true, Some("blocked"), true, "refused"),
        ("wrong_nonce", Some(0), false, Some("complete"), false, "handback-malformed"),
        ("absent", Some(0), false, None, true, "handback-malformed"),
        ("paused", Some(0), false, Some("paused"), true, "paused"),
    ];
    for (name, exit, refused, status, nonce_ok, want) in cases {
        let (_d, s) = store();
        let a = alloc(&s, 10);
        s.transition(&t2(&a), 11).unwrap();
        let mut r = t3(&a, "D");
        r.exit_code = exit;
        r.refused = refused;
        r.handback = status.map(|st| Handback {
            attempt: Some(1),
            nonce: Some(if nonce_ok { a.nonce.clone() } else { "other".into() }),
            status: Some(st.into()),
            summary: None,
            commission: Some(a.dispatch_id.clone()),
        });
        let out = s.transition(&r, 12).unwrap();
        assert_eq!(out.record["state"], "done", "{name}");
        assert_eq!(out.record["outcome"], want, "{name}");
        assert_eq!(out.record["exit_code"], json!(exit), "{name}");
        if name == "paused" {
            // CC-19: A2 may be allocated after a paused done
            let n = s.allocate(&AllocateRequest { attempt_of: Some(a.dispatch_id.clone()), ..Default::default() }, 13).unwrap();
            assert_eq!(n.attempt, 2);
        }
    }
}

// T-3599: §3.3 rule 1 commission clause (ring20 N4). A hand-back naming another dispatch
// is malformed even with the right attempt and nonce; the own id, bare or with a
// `<backend>:` prefix, passes. T-3600: an absent commission is malformed too (§2).
#[test]
fn handback_commission_must_name_this_dispatch() {
    let (_d, s) = store();
    let probe = alloc(&s, 10);
    let own = probe.dispatch_id.clone();
    let cases: [(&str, Option<String>, &str); 4] = [
        ("own-bare", Some(own.clone()), "succeeded"),
        ("own-prefixed", Some(format!("ext-backend:{own}")), "succeeded"),
        ("absent", None, "handback-malformed"),
        ("foreign", Some("D-someone-else".into()), "handback-malformed"),
    ];
    let mut t = 20;
    for (name, commission, want) in cases {
        let a = if name == "own-bare" { probe.clone() } else { t += 10; alloc(&s, t) };
        s.transition(&t2(&a), t + 1).unwrap();
        let mut r = t3(&a, "D");
        r.exit_code = Some(0);
        let commission = commission.map(|c| c.replace(&own, &a.dispatch_id));
        r.handback = Some(Handback {
            attempt: Some(1),
            nonce: Some(a.nonce.clone()),
            status: Some("complete".into()),
            summary: None,
            commission,
        });
        let out = s.transition(&r, t + 2).unwrap();
        assert_eq!(out.record["outcome"], want, "{name}");
    }
}

// CC-16 supervisor-refusal / CC-21 / CC-25: only the refused flag outranks the handback check.
#[test]
fn refused_wins_over_absent_handback_but_unrefused_absent_is_malformed() {
    // (name, from dispatched?, exit, refused, want)
    let cases = [
        ("cc16_refused_absent", false, 3, true, "refused"),
        ("cc21_refused_before_start_absent", true, 3, true, "refused"),
        ("cc25_unrefused_absent", false, 143, false, "handback-malformed"),
    ];
    for (name, from_dispatched, exit, refused, want) in cases {
        let (_d, s) = store();
        let a = alloc(&s, 10);
        if !from_dispatched {
            s.transition(&t2(&a), 11).unwrap();
        }
        let mut r = t3(&a, "D");
        if from_dispatched {
            r.expected_prior = "dispatched".into();
        }
        r.exit_code = Some(exit);
        r.refused = refused;
        r.handback = None;
        let out = s.transition(&r, 12).unwrap();
        assert_eq!(out.record["state"], "done", "{name}");
        assert_eq!(out.record["outcome"], want, "{name}");
        assert_eq!(out.record["exit_code"], json!(exit), "{name}");
    }
}

// CC-23
#[test]
fn nonce_alone_cannot_complete() {
    let (_d, s) = store();
    let a = alloc(&s, 10);
    s.transition(&t2(&a), 11).unwrap();
    let mut r = t3(&a, "X");
    r.supervisor_token = None;
    assert_eq!(reason(s.transition(&r, 12).unwrap_err()), "bad_credential");
    r.supervisor_token = Some("wrong".into());
    assert_eq!(reason(s.transition(&r, 13).unwrap_err()), "bad_credential");
    assert_eq!(s.get(&a.dispatch_id).unwrap()["attempts"][0]["state"], "running");
    assert_eq!(s.transition(&t3(&a, "X"), 14).unwrap().record["state"], "done");
}

#[test]
fn cas_forbidden_and_unknown_cases() {
    let (_d, s) = store();
    let a = alloc(&s, 10);
    // T3b from dispatched is valid
    let mut r = t3(&a, "X");
    r.expected_prior = "dispatched".into();
    assert_eq!(s.transition(&r, 11).unwrap().record["state"], "done");
    // wrong expected prior
    let b = alloc(&s, 12);
    let mut bad = t3(&b, "X");
    bad.expected_prior = "lost".into();
    assert_eq!(reason(s.transition(&bad, 13).unwrap_err()), "cas_mismatch");
    // hub-only targets are forbidden here
    for to in ["lost", "unknown", "dispatched"] {
        let mut f = t2(&b);
        f.to = to.into();
        assert_eq!(reason(s.transition(&f, 14).unwrap_err()), "forbidden_transition", "{to}");
    }
    // unknown dispatch / attempt
    let mut nf = t2(&b);
    nf.dispatch_id = "D-00000000000000000000000000".into();
    assert!(matches!(s.transition(&nf, 15), Err(WcError::NotFound(_))));
    assert!(matches!(s.get("D-00000000000000000000000000"), Err(WcError::NotFound(_))));
    // T7b: unknown -> done, no persisted_at bound
    let c = alloc(&s, 20);
    s.transition(&t2(&c), 21).unwrap();
    s.conn.lock().unwrap().execute("UPDATE wc_attempts SET state='unknown' WHERE dispatch_id=?1", params![c.dispatch_id]).unwrap();
    assert_eq!(s.transition(&t3(&c, "Z"), 22).unwrap().record["state"], "done");
}
