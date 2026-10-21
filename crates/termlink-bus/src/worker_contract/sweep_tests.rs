//! Sweep conformance (T-3502, S2) with injected time and a fake registry: temp dirs only
//! (T-3293). Cases follow the T-3502 ACs and design §4: the end-observed-twice rule with
//! grace G, the registry epoch (hub restart, unreadable listing), the deadlines, and the
//! `lost` upgrade and no-resurrection rules.
use super::*;
use super::sweep::SweepObservation;
use std::collections::HashSet;

const SWEEP: i64 = 30_000;

fn store() -> (tempfile::TempDir, WcStore) {
    let d = tempfile::tempdir().unwrap();
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    (d, s)
}

fn alloc(s: &WcStore, now: i64) -> Allocation {
    s.allocate(&AllocateRequest::default(), now).unwrap()
}

/// T2: dispatched -> running at `now`, with a worker timeout (ms).
fn t2(s: &WcStore, a: &Allocation, now: i64, worker_timeout_ms: i64) {
    s.transition(
        &TransitionRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: a.attempt,
            nonce: a.nonce.clone(),
            supervisor_token: Some(a.supervisor_token.clone()),
            expected_prior: "dispatched".into(),
            to: "running".into(),
            supervisor_gen: 1,
            worker_timeout_ms: Some(worker_timeout_ms),
            ..Default::default()
        },
        now,
    )
    .unwrap();
}

/// T3: running -> done with evidence (the supervisor's own write).
fn t3_done(s: &WcStore, a: &Allocation, now: i64) {
    s.transition(
        &TransitionRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: a.attempt,
            nonce: a.nonce.clone(),
            supervisor_token: Some(a.supervisor_token.clone()),
            expected_prior: "running".into(),
            to: "done".into(),
            supervisor_gen: 1,
            evidence_digest: Some("E1".into()),
            exit_code: Some(0),
            handback: Some(Handback {
                attempt: Some(a.attempt),
                nonce: Some(a.nonce.clone()),
                status: Some("complete".into()),
                summary: None,
            }),
            ..Default::default()
        },
        now,
    )
    .unwrap();
}

fn live(a: &Allocation) -> SweepObservation {
    SweepObservation { listing_complete: true, live: [(a.dispatch_id.clone(), a.attempt)].into_iter().collect() }
}

fn nobody() -> SweepObservation {
    SweepObservation { listing_complete: true, live: HashSet::new() }
}

fn unreadable() -> SweepObservation {
    SweepObservation { listing_complete: false, live: HashSet::new() }
}

fn state(s: &WcStore, a: &Allocation) -> String {
    s.get(&a.dispatch_id).unwrap()["attempts"][0]["state"].as_str().unwrap().to_string()
}

fn hub_rows(s: &WcStore, a: &Allocation) -> Vec<Value> {
    s.get(&a.dispatch_id).unwrap()["attempts"][0]["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["writer"] == "hub")
        .cloned()
        .collect()
}

fn to_states(s: &WcStore, a: &Allocation) -> Vec<String> {
    s.get(&a.dispatch_id).unwrap()["attempts"][0]["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["to"].as_str().unwrap().to_string())
        .collect()
}

/// A running attempt whose binding was seen live at `t`, epoch 0 (a fresh store).
fn running_seen(s: &WcStore, worker_timeout_ms: i64, seen_at: i64) -> Allocation {
    let a = alloc(s, 0);
    t2(s, &a, 1_000, worker_timeout_ms);
    let r = s.sweep(&live(&a), seen_at).unwrap();
    assert!(r.transitions.is_empty(), "a live sighting writes nothing for running");
    a
}

// --- CC-02 / CC-01 shape: end observed twice, lost only after grace G -----------------

#[test]
fn absent_twice_then_lost_only_after_grace() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    s.sweep(&nobody(), 2_000 + SWEEP).unwrap(); // absence 1: end observed
    s.sweep(&nobody(), 2_000 + 2 * SWEEP).unwrap(); // absence 2: twice, but G not elapsed
    assert_eq!(state(&s, &a), "running");
    s.sweep(&nobody(), 2_000 + 2 * SWEEP + 2 * SWEEP).unwrap();
    assert_eq!(state(&s, &a), "running", "end first observed at 32s: lost is not due before 152s");
    let r = s.sweep(&nobody(), 32_000 + GRACE_MS).unwrap();
    assert_eq!(state(&s, &a), "lost");
    assert!(r.transitions.iter().any(|t| t.to == "lost" && t.cause == "end_observed"));
    let rows = hub_rows(&s, &a);
    assert!(rows.iter().any(|h| h["to"] == "lost" && h["from"] == "running" && h["cause"] == "end_observed"));
}

#[test]
fn one_absence_is_never_enough_for_lost() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    s.sweep(&nobody(), 2_000 + 10 * GRACE_MS).unwrap();
    assert_eq!(state(&s, &a), "running", "a single absent sweep, however late, does not end the worker");
}

#[test]
fn done_evidence_inside_grace_prevents_lost() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    s.sweep(&nobody(), 32_000).unwrap();
    s.sweep(&nobody(), 62_000).unwrap();
    t3_done(&s, &a, 100_000); // the supervisor's fenced write arrives inside G
    s.sweep(&nobody(), 400_000).unwrap();
    assert_eq!(state(&s, &a), "done");
    assert!(!to_states(&s, &a).contains(&"lost".to_string()));
}

// --- CC-08 shape: registry epoch, hub restart, unreadable listing ---------------------

#[test]
fn hub_restart_between_sweeps_never_produces_lost() {
    let (d, s) = store();
    let a = running_seen(&s, 1_000, 2_000); // deadline_running = 1_000 + 1_000 + 600_000
    s.sweep(&nobody(), 32_000).unwrap();
    drop(s);
    // Hub restarts: a new store on the same meta.db, epoch bumped at start.
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    assert_eq!(s.begin_hub_epoch().unwrap(), 1);
    s.sweep(&nobody(), 62_000).unwrap();
    s.sweep(&nobody(), 200_000).unwrap();
    assert_eq!(state(&s, &a), "running", "absence across an epoch change counts toward nothing");
    s.sweep(&nobody(), 700_000).unwrap(); // past deadline_running: deadlines decide
    assert_eq!(state(&s, &a), "unknown");
    assert!(!to_states(&s, &a).contains(&"lost".to_string()), "epoch change can only lead to unknown");
}

#[test]
fn restart_keeps_state_and_a_live_binding_changes_nothing() {
    let (d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    drop(s);
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    s.begin_hub_epoch().unwrap();
    let r = s.sweep(&live(&a), 40_000).unwrap();
    assert!(r.transitions.is_empty());
    assert_eq!(state(&s, &a), "running");
}

#[test]
fn unreadable_listing_never_produces_lost() {
    let (_d, s) = store();
    let a = running_seen(&s, 1_000, 2_000);
    let mut last = 0;
    for (i, now) in [32_000, 62_000, 152_000, 200_000, 400_000, 700_000].into_iter().enumerate() {
        let r = s.sweep(&unreadable(), now).unwrap();
        assert!(r.epoch > last, "an incomplete listing bumps the epoch (sweep {i})");
        last = r.epoch;
    }
    assert_eq!(state(&s, &a), "unknown", "deadlines decide; the blind sweeps never end the worker");
    assert!(!to_states(&s, &a).contains(&"lost".to_string()));
}

#[test]
fn epoch_is_bumped_at_hub_start_and_persists() {
    let (d, s) = store();
    assert_eq!(s.begin_hub_epoch().unwrap(), 1);
    assert_eq!(s.begin_hub_epoch().unwrap(), 2);
    drop(s);
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    assert_eq!(s.sweep(&nobody(), 1).unwrap().epoch, 2, "reopening does not reset the epoch");
}

// --- T6: deadlines ---------------------------------------------------------------------

#[test]
fn dispatched_never_seen_goes_unknown_at_its_deadline_and_no_further() {
    let (_d, s) = store();
    let a = alloc(&s, 0); // deadline_dispatched = 180_000
    s.sweep(&nobody(), 100_000).unwrap();
    assert_eq!(state(&s, &a), "dispatched");
    let r = s.sweep(&nobody(), 200_000).unwrap();
    assert_eq!(state(&s, &a), "unknown");
    assert!(r.transitions.iter().any(|t| t.cause == "deadline_dispatched"));
    s.sweep(&nobody(), 900_000).unwrap();
    assert_eq!(state(&s, &a), "unknown", "never seen, so no end was observed: unknown waits for T7d");
}

#[test]
fn live_running_past_deadline_goes_unknown_and_a_returning_worker_is_resumed() {
    let (_d, s) = store();
    let a = running_seen(&s, 1_000, 2_000); // deadline_running = 601_000
    s.sweep(&live(&a), 600_000).unwrap();
    assert_eq!(state(&s, &a), "running", "before the deadline a live worker is left alone");
    s.sweep(&live(&a), 700_000).unwrap();
    assert_eq!(state(&s, &a), "unknown");
    let r = s.sweep(&live(&a), 720_000).unwrap(); // T7a
    assert_eq!(state(&s, &a), "running");
    assert!(r.transitions.iter().any(|t| t.from == "unknown" && t.to == "running" && t.cause == "binding_reappeared"));
    // T7a re-arms the deadline, so the next live sweep does not flip it straight back.
    s.sweep(&live(&a), 730_000).unwrap();
    assert_eq!(state(&s, &a), "running");
}

// --- T7c: unknown with end observed twice ----------------------------------------------

#[test]
fn unknown_with_end_observed_twice_goes_lost_after_grace() {
    let (_d, s) = store();
    let a = running_seen(&s, 1_000, 2_000);
    s.sweep(&live(&a), 700_000).unwrap(); // T6 at the sighting -> unknown, seen at this epoch
    assert_eq!(state(&s, &a), "unknown");
    s.sweep(&nobody(), 720_000).unwrap(); // absence 1
    s.sweep(&nobody(), 750_000).unwrap(); // absence 2
    s.sweep(&nobody(), 830_000).unwrap();
    assert_eq!(state(&s, &a), "unknown", "grace runs from the first absence (720s)");
    let r = s.sweep(&nobody(), 840_000).unwrap();
    assert_eq!(state(&s, &a), "lost");
    assert!(r.transitions.iter().any(|t| t.from == "unknown" && t.to == "lost" && t.cause == "end_observed_unknown"));
}

// --- lost is never resurrected; T5 can still upgrade it --------------------------------

#[test]
fn reappearing_binding_after_lost_is_noticed_not_resurrected_and_t5_still_upgrades() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    s.sweep(&nobody(), 32_000).unwrap();
    s.sweep(&nobody(), 62_000).unwrap();
    s.sweep(&nobody(), 152_000).unwrap();
    assert_eq!(state(&s, &a), "lost");

    let r = s.sweep(&live(&a), 200_000).unwrap();
    assert_eq!(state(&s, &a), "lost", "lost -> running is never written");
    assert_eq!(r.notices.len(), 1);
    assert!(hub_rows(&s, &a).iter().any(|h| h["fence_reason"] == "forbidden_transition" && h["cause"] == "binding_reappeared"));

    let r = s.sweep(&live(&a), 230_000).unwrap();
    assert!(r.notices.is_empty(), "the notice is raised once per reappearance");

    // The supervisor's evidence, persisted before the lost entry (152s), still upgrades it (T5).
    let out = s
        .transition(
            &TransitionRequest {
                dispatch_id: a.dispatch_id.clone(),
                attempt: a.attempt,
                nonce: a.nonce.clone(),
                supervisor_token: Some(a.supervisor_token.clone()),
                expected_prior: "lost".into(),
                to: "done".into(),
                supervisor_gen: 1,
                evidence_digest: Some("E1".into()),
                exit_code: Some(0),
                persisted_at: Some(100_000),
                handback: Some(Handback {
                    attempt: Some(a.attempt),
                    nonce: Some(a.nonce.clone()),
                    status: Some("complete".into()),
                    summary: None,
                }),
                ..Default::default()
            },
            240_000,
        )
        .unwrap();
    assert_eq!(out.record["state"], "done");
    assert_eq!(state(&s, &a), "done");
}

// --- every sweep write is a hub history row --------------------------------------------

#[test]
fn every_sweep_write_leaves_a_hub_history_row() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.sweep(&nobody(), 200_000).unwrap(); // dispatched -> unknown by deadline
    // allocate's own `-> dispatched` row is writer hub too; the sweep's row is the last one.
    let rows = hub_rows(&s, &a);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1]["from"], "dispatched");
    assert_eq!(rows[1]["to"], "unknown");
    assert_eq!(rows[1]["cause"], "deadline_dispatched");
}

#[test]
fn a_clean_sweep_writes_nothing() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    let before = s.get(&a.dispatch_id).unwrap();
    let r = s.sweep(&live(&a), 40_000).unwrap();
    assert!(r.transitions.is_empty() && r.notices.is_empty());
    assert_eq!(s.get(&a.dispatch_id).unwrap()["attempts"][0]["history"], before["attempts"][0]["history"]);
}

// --- confirm_end (T-3522, wc/0.3 CC-02 / §3.4): the hub's end confirmation -------------

#[test]
fn confirm_end_refuses_a_done_attempt_with_forbidden_transition() {
    // CC-02: the supervisor's completion (T3) lands first; the hub's later end confirmation is
    // a forbidden transition (done is terminal), and the attempt is not touched.
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    t3_done(&s, &a, 10_000);
    let err = s.confirm_end(&a.dispatch_id, a.attempt, &nobody(), 20_000).unwrap_err();
    assert!(matches!(err, WcError::Fence { reason: FenceReason::ForbiddenTransition, .. }), "{err:?}");
    assert_eq!(state(&s, &a), "done");
}

#[test]
fn confirm_end_on_a_never_seen_running_attempt_writes_nothing_at_any_clock() {
    // §3.4 / CC-17: absence counts only for an attempt seen alive at the current epoch. A
    // running attempt whose binding was never observed is acted on by its deadlines only.
    let (_d, s) = store();
    let a = alloc(&s, 0);
    t2(&s, &a, 1_000, 3_600_000);
    for t in [2_000, 32_000, 62_000, 500_000] {
        let r = s.confirm_end(&a.dispatch_id, a.attempt, &nobody(), t).unwrap();
        assert!(r.transitions.is_empty(), "no end was observed at {t}");
    }
    assert_eq!(state(&s, &a), "running");
}

// --- CC-25 (T-3523, §3.3): a backend-initiated stop is an action, not a state ---------------

fn stop(s: &WcStore, a: &Allocation, cause: &str, token: &str, now: i64) -> WcResult<Value> {
    s.stop(
        &StopRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: a.attempt,
            supervisor_token: Some(token.into()),
            cause: cause.into(),
        },
        now,
    )
}

fn last_cause(s: &WcStore, a: &Allocation) -> Value {
    let v = s.get(&a.dispatch_id).unwrap();
    v["attempts"][a.attempt as usize - 1]["history"].as_array().unwrap().last().unwrap()["cause"].clone()
}

#[test]
fn stop_then_confirmed_exit_is_done_and_carries_the_cause() {
    // CC-25 (a): exit confirmed, valid hand-back. The stop writes no row and no state; T3
    // ends the attempt and its history entry carries the stop cause. No new attempt.
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    let rows = s.get(&a.dispatch_id).unwrap()["attempts"][0]["history"].as_array().unwrap().len();
    let rec = stop(&s, &a, "deadline_stop", &a.supervisor_token, 5_000).unwrap();
    assert_eq!(rec["state"], "running");
    assert_eq!(rec["stop_cause"], "deadline_stop");
    assert_eq!(rec["history"].as_array().unwrap().len(), rows, "a stop writes no history row");
    t3_done(&s, &a, 6_000);
    assert_eq!(state(&s, &a), "done");
    assert_eq!(last_cause(&s, &a), "deadline_stop");
    assert_eq!(s.get(&a.dispatch_id).unwrap()["attempts"].as_array().unwrap().len(), 1);
}

#[test]
fn stop_then_no_exit_evidence_is_lost_by_the_sweep_with_the_cause() {
    // CC-25 (c): no confirmed exit. The hub's end confirmation and grace give T4, and the T4
    // entry carries the stop cause in place of the sweep's own.
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    stop(&s, &a, "deadline_stop", &a.supervisor_token, 5_000).unwrap();
    s.confirm_end(&a.dispatch_id, a.attempt, &nobody(), 10_000).unwrap();
    s.sweep(&nobody(), 10_000 + GRACE_MS).unwrap();
    assert_eq!(state(&s, &a), "lost");
    assert_eq!(last_cause(&s, &a), "deadline_stop");
}

#[test]
fn stop_revoked_behaves_the_same() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    stop(&s, &a, "revoked", &a.supervisor_token, 5_000).unwrap();
    t3_done(&s, &a, 6_000);
    assert_eq!(last_cause(&s, &a), "revoked");
}

#[test]
fn stop_refusals_bad_token_bad_cause_terminal_and_conflicting_cause() {
    let (_d, s) = store();
    let a = running_seen(&s, 3_600_000, 2_000);
    let fence_of = |r: WcResult<Value>| match r {
        Err(WcError::Fence { reason, .. }) => reason.as_str(),
        other => panic!("expected a fence refusal, got {other:?}"),
    };
    assert_eq!(fence_of(stop(&s, &a, "deadline_stop", "not-the-token", 3_000)), "bad_credential");
    assert!(matches!(stop(&s, &a, "bored", &a.supervisor_token, 3_000), Err(WcError::Invalid(_))));
    stop(&s, &a, "deadline_stop", &a.supervisor_token, 4_000).unwrap();
    // The same cause again is a no-op; a different one is refused.
    stop(&s, &a, "deadline_stop", &a.supervisor_token, 4_500).unwrap();
    assert_eq!(fence_of(stop(&s, &a, "revoked", &a.supervisor_token, 4_600)), "cas_mismatch");
    t3_done(&s, &a, 5_000);
    assert_eq!(fence_of(stop(&s, &a, "deadline_stop", &a.supervisor_token, 6_000)), "cas_mismatch");
    assert_eq!(state(&s, &a), "done");
}
