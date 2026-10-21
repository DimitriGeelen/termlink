//! Operator notices (T-3521): one failing-first test per notice kind, driven through the real
//! store API with injected time and a fake registry. Temp dirs only (T-3293).
use super::answer::ResolveRequest;
use super::sweep::SweepObservation;
use super::*;
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

fn req(a: &Allocation, prior: &str, to: &str) -> TransitionRequest {
    TransitionRequest {
        dispatch_id: a.dispatch_id.clone(),
        attempt: a.attempt,
        nonce: a.nonce.clone(),
        supervisor_token: Some(a.supervisor_token.clone()),
        expected_prior: prior.into(),
        to: to.into(),
        supervisor_gen: 1,
        worker_timeout_ms: Some(3_600_000),
        ..Default::default()
    }
}

fn done_req(a: &Allocation, prior: &str, digest: &str, persisted_at: Option<i64>) -> TransitionRequest {
    TransitionRequest {
        evidence_digest: Some(digest.into()),
        exit_code: Some(0),
        persisted_at,
        handback: Some(Handback {
            attempt: Some(a.attempt),
            nonce: Some(a.nonce.clone()),
            status: Some("complete".into()),
            summary: None,
        }),
        ..req(a, prior, "done")
    }
}

fn live(a: &Allocation) -> SweepObservation {
    SweepObservation { listing_complete: true, live: [(a.dispatch_id.clone(), a.attempt)].into_iter().collect() }
}

fn nobody() -> SweepObservation {
    SweepObservation { listing_complete: true, live: HashSet::new() }
}

/// Kinds of every notice on the attempt, whoever it is for.
fn all_kinds(s: &WcStore, a: &Allocation) -> Vec<String> {
    s.notice_list(&a.dispatch_id, a.attempt)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["kind"].as_str().unwrap().to_string())
        .collect()
}

/// Kinds of the operator's notices only.
fn kinds(s: &WcStore, a: &Allocation) -> Vec<String> {
    let all = s.notice_list(&a.dispatch_id, a.attempt).unwrap();
    all.as_array()
        .unwrap()
        .iter()
        .filter(|n| n["audience"] == "operator")
        .map(|n| n["kind"].as_str().unwrap().to_string())
        .collect()
}

fn count(s: &WcStore, a: &Allocation) -> i64 {
    s.operator_notice_count(&a.dispatch_id, None).unwrap()
}

/// A running attempt, seen live at 2s, that the sweep then loses (end seen twice, G passed).
fn lost_attempt(s: &WcStore) -> (Allocation, i64) {
    let a = alloc(s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    s.sweep(&live(&a), 2_000).unwrap();
    s.sweep(&nobody(), 2_000 + SWEEP).unwrap();
    s.sweep(&nobody(), 2_000 + 2 * SWEEP).unwrap();
    let lost_at = 2_000 + SWEEP + GRACE_MS;
    let r = s.sweep(&nobody(), lost_at).unwrap();
    assert!(r.transitions.iter().any(|t| t.to == "lost"));
    (a, lost_at)
}

#[test]
fn done_conflict_refusal_raises_one_notice_and_a_t8_replay_none() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    s.transition(&done_req(&a, "running", "E1", None), 2_000).unwrap();
    assert_eq!(count(&s, &a), 0, "a normal T3 raises nothing");
    let replay = s.transition(&done_req(&a, "running", "E1", None), 3_000).unwrap();
    assert!(replay.deduped);
    assert_eq!(count(&s, &a), 0, "T8 with the same evidence is a response, not an event");
    let e = s.transition(&done_req(&a, "running", "E2", None), 4_000).unwrap_err();
    assert!(matches!(e, WcError::Fence { reason: FenceReason::DoneConflict, .. }));
    assert_eq!(kinds(&s, &a), vec!["done_conflict"]);
    assert_eq!(count(&s, &a), 1);
}

#[test]
fn late_t5_refusal_raises_a_notice_and_an_in_time_t5_none() {
    let (_d, s) = store();
    let (a, lost_at) = lost_attempt(&s);
    let e = s.transition(&done_req(&a, "lost", "E1", Some(lost_at + 1)), lost_at + 5_000).unwrap_err();
    assert!(matches!(e, WcError::Fence { reason: FenceReason::ForbiddenTransition, .. }));
    assert_eq!(kinds(&s, &a), vec!["late_evidence"]);
    s.transition(&done_req(&a, "lost", "E1", Some(lost_at)), lost_at + 6_000).unwrap();
    assert_eq!(count(&s, &a), 1, "the in-time T5 adds none");
}

#[test]
fn other_refusals_raise_no_notice() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    let mut stale_gen = req(&a, "running", "running");
    stale_gen.supervisor_gen = 9;
    assert!(s.transition(&stale_gen, 2_000).is_err());
    let mut cas = done_req(&a, "dispatched", "E1", None);
    cas.expected_prior = "dispatched".into();
    assert!(s.transition(&cas, 3_000).is_err());
    assert_eq!(count(&s, &a), 0);
}

#[test]
fn t6_and_t6b_by_deadline_raise_a_notice_each() {
    let (_d, s) = store();
    // T6b: dispatched, never registers.
    let never = alloc(&s, 0);
    let past_dispatched = never.deadline_dispatched + 1;
    s.sweep(&nobody(), past_dispatched).unwrap();
    assert_eq!(kinds(&s, &never), vec!["unknown_by_deadline"]);
    // T6: running, not seen, past its running deadline.
    let run = alloc(&s, 0);
    s.transition(&req(&run, "dispatched", "running"), 1_000).unwrap();
    let past_running = 1_000 + 3_600_000 + RUNNING_SLACK_MS + 1;
    s.sweep(&nobody(), past_running).unwrap();
    assert_eq!(kinds(&s, &run), vec!["unknown_by_deadline"]);
    // T4 is not a notice.
    let (lost, _) = lost_attempt(&s);
    assert_eq!(count(&s, &lost), 0, "T4 raises none");
}

#[test]
fn binding_reappearing_after_lost_raises_one_notice() {
    let (_d, s) = store();
    let (a, lost_at) = lost_attempt(&s);
    s.sweep(&live(&a), lost_at + 5_000).unwrap();
    assert_eq!(kinds(&s, &a), vec!["binding_reappeared"]);
    s.sweep(&live(&a), lost_at + 10_000).unwrap();
    assert_eq!(count(&s, &a), 1, "the same reappearance is told once");
}

#[test]
fn operator_resolve_of_unknown_raises_a_notice_and_a_refused_resolve_none() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    let resolve = |as_state: &str, now| {
        s.resolve(
            &ResolveRequest {
                dispatch_id: a.dispatch_id.clone(),
                attempt: a.attempt,
                as_state: as_state.into(),
                reason: "host reimaged".into(),
            },
            now,
        )
    };
    assert!(resolve("lost", 2_000).is_err(), "running is not unknown: refused");
    assert_eq!(count(&s, &a), 0, "a refused resolve raises none");
    s.sweep(&nobody(), 1_000 + 3_600_000 + RUNNING_SLACK_MS + 1).unwrap();
    assert_eq!(count(&s, &a), 1, "the T6 that made it unknown");
    resolve("lost", 4_000_000_000).unwrap();
    assert_eq!(kinds(&s, &a), vec!["unknown_by_deadline", "operator_resolved"]);
}

#[test]
fn reconciliation_request_is_durable_once_and_not_an_operator_notice() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    s.sweep(&live(&a), 2_000).unwrap();
    let r1 = s.sweep(&nobody(), 2_000 + SWEEP).unwrap();
    assert!(r1.reconciliation_requested.is_empty(), "one absence asks nothing");
    let r2 = s.sweep(&nobody(), 2_000 + 2 * SWEEP).unwrap();
    assert_eq!(r2.reconciliation_requested, vec![(a.dispatch_id.clone(), a.attempt)]);
    let r3 = s.sweep(&nobody(), 2_000 + 3 * SWEEP).unwrap();
    assert!(r3.reconciliation_requested.is_empty(), "asked once per absence window");
    assert_eq!(all_kinds(&s, &a), vec!["reconciliation_requested"]);
    assert_eq!(count(&s, &a), 0, "the reconciler is not the operator");
    let n = s.notice_list(&a.dispatch_id, a.attempt).unwrap();
    assert_eq!(n[0]["audience"], "reconciler");
}

#[test]
fn a_new_epoch_does_not_ask_for_reconciliation() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    s.sweep(&live(&a), 2_000).unwrap();
    s.begin_hub_epoch().unwrap();
    s.sweep(&nobody(), 2_000 + SWEEP).unwrap();
    let r = s.sweep(&nobody(), 2_000 + 2 * SWEEP).unwrap();
    assert!(r.reconciliation_requested.is_empty(), "absence across an epoch counts for nothing (I7)");
    assert!(kinds(&s, &a).is_empty());
}

#[test]
fn get_returns_each_attempts_notices() {
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    s.transition(&done_req(&a, "running", "E1", None), 2_000).unwrap();
    let _ = s.transition(&done_req(&a, "running", "E2", None), 3_000);
    let got = s.get(&a.dispatch_id).unwrap();
    let n = &got["attempts"][0]["notices"];
    assert_eq!(n.as_array().unwrap().len(), 1);
    assert_eq!(n[0]["kind"], "done_conflict");
    assert_eq!(n[0]["audience"], "operator");
}

#[test]
fn notice_commits_with_its_history_row_or_not_at_all() {
    // The trigger fires inside the writer's transaction, so every notice has the history row
    // that caused it (history_id), and a refusal's row and notice are one commit.
    let (_d, s) = store();
    let a = alloc(&s, 0);
    s.transition(&req(&a, "dispatched", "running"), 1_000).unwrap();
    s.transition(&done_req(&a, "running", "E1", None), 2_000).unwrap();
    let _ = s.transition(&done_req(&a, "running", "E2", None), 3_000);
    let conn = s.conn.lock().unwrap();
    let orphans: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM wc_notices n WHERE n.history_id IS NOT NULL
               AND NOT EXISTS (SELECT 1 FROM wc_history h WHERE h.id = n.history_id)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(orphans, 0);
    let linked: i64 = conn
        .query_row("SELECT COUNT(*) FROM wc_notices WHERE history_id IS NOT NULL", [], |r| r.get(0))
        .unwrap();
    assert_eq!(linked, 1);
}
