//! Answer path and operator resolve (T-3503, S3) with injected time, temp dirs only
//! (T-3293). Cases: CC-10 parent-session loss, CC-11 answer delivered and acked,
//! CC-12 answer TTL expiry bounces, CC-15 unknown resolution by operator.
use super::answer::{AckRequest, AnswerRequest, AskRequest, ResolveRequest, ANSWER_TTL_MS, resolve_at};
use super::sweep::SweepObservation;
use super::*;
use serde_json::Value;

fn store() -> (tempfile::TempDir, WcStore) {
    let d = tempfile::tempdir().unwrap();
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    (d, s)
}

/// An attempt that is `running` at t=1000.
fn running(s: &WcStore) -> Allocation {
    let a = s.allocate(&AllocateRequest::default(), 0).unwrap();
    s.transition(
        &TransitionRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: a.attempt,
            nonce: a.nonce.clone(),
            supervisor_token: Some(a.supervisor_token.clone()),
            expected_prior: "dispatched".into(),
            to: "running".into(),
            supervisor_gen: 1,
            ..Default::default()
        },
        1_000,
    )
    .unwrap();
    a
}

fn ask(a: &Allocation, q: &str) -> AskRequest {
    AskRequest {
        dispatch_id: a.dispatch_id.clone(),
        attempt: a.attempt,
        nonce: a.nonce.clone(),
        question_id: Some(q.into()),
        body: "which branch?".into(),
    }
}

fn answer(a: &Allocation, cmd: &str, q: Option<&str>) -> AnswerRequest {
    AnswerRequest {
        dispatch_id: a.dispatch_id.clone(),
        attempt: a.attempt,
        supervisor_token: Some(a.supervisor_token.clone()),
        cmd_id: Some(cmd.into()),
        question_id: q.map(str::to_string),
        body: "main".into(),
    }
}

fn ack(a: &Allocation, cmd: &str, state: &str) -> AckRequest {
    AckRequest {
        dispatch_id: a.dispatch_id.clone(),
        attempt: a.attempt,
        nonce: a.nonce.clone(),
        cmd_id: cmd.into(),
        state: state.into(),
    }
}

fn reason_of(e: WcError) -> &'static str {
    match e {
        WcError::Fence { reason, .. } => reason.as_str(),
        other => panic!("expected a fence refusal, got {other:?}"),
    }
}

fn answers(s: &WcStore, a: &Allocation) -> Vec<Value> {
    s.get(&a.dispatch_id).unwrap()["attempts"][a.attempt as usize - 1]["answers"]
        .as_array()
        .unwrap()
        .clone()
}

fn fence_rows(s: &WcStore, a: &Allocation) -> Vec<String> {
    let v = s.get(&a.dispatch_id).unwrap();
    v["attempts"][a.attempt as usize - 1]["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|h| h["fence_reason"].as_str().map(str::to_string))
        .collect()
}

/// CC-11: the answer is stored, the worker acks received then applied; each step is one
/// fenced write and a replay is a response, not a regression.
#[test]
fn cc11_answer_delivered_and_acked() {
    let (_d, s) = store();
    let a = running(&s);
    let q = s.ask(&ask(&a, "q1"), 2_000).unwrap();
    assert!(!q.deduped);
    let st = s.answer(&answer(&a, "c1", Some("q1")), 3_000).unwrap();
    assert_eq!(st.record["state"], "stored");
    assert_eq!(st.record["deadline"], 3_000 + ANSWER_TTL_MS);
    let r = s.ack(&ack(&a, "c1", "received"), 4_000).unwrap();
    assert_eq!(r.record["state"], "received");
    let ap = s.ack(&ack(&a, "c1", "applied"), 5_000).unwrap();
    assert_eq!(ap.record["state"], "applied");
    assert_eq!(ap.record["applied_at"], 5_000);
    // A duplicate ack, even a stale `received`, changes nothing.
    assert!(s.ack(&ack(&a, "c1", "received"), 6_000).unwrap().deduped);
    assert!(s.ack(&ack(&a, "c1", "applied"), 6_001).unwrap().deduped);
    assert_eq!(answers(&s, &a)[0]["state"], "applied");
    // The sweep leaves an applied answer alone, however late it runs.
    assert!(s.sweep_answers(9_999_999).unwrap().is_empty());
    // The question knows which command answered it.
    let got = s.get(&a.dispatch_id).unwrap();
    assert_eq!(got["attempts"][0]["questions"][0]["answered_cmd_id"], "c1");
}

#[test]
fn ack_cannot_skip_received_or_go_backwards() {
    let (_d, s) = store();
    let a = running(&s);
    s.answer(&answer(&a, "c1", None), 2_000).unwrap();
    assert_eq!(reason_of(s.ack(&ack(&a, "c1", "applied"), 2_100).unwrap_err()), "cas_mismatch");
    assert_eq!(answers(&s, &a)[0]["state"], "stored");
    assert!(fence_rows(&s, &a).contains(&"cas_mismatch".to_string()));
    assert!(matches!(s.ack(&ack(&a, "nope", "received"), 2_200), Err(WcError::NotFound(_))));
    assert!(matches!(s.ack(&ack(&a, "c1", "weird"), 2_300), Err(WcError::Invalid(_))));
}

#[test]
fn credentials_and_attempt_fence_the_answer_path() {
    let (_d, s) = store();
    let a = running(&s);
    // ask: the nonce is the credential; the supervisor token is not.
    let mut bad = ask(&a, "q1");
    bad.nonce = "wrong".into();
    assert_eq!(reason_of(s.ask(&bad, 2_000).unwrap_err()), "bad_nonce");
    // answer: the supervisor token is the credential; the nonce is not.
    let mut no_token = answer(&a, "c1", None);
    no_token.supervisor_token = None;
    assert_eq!(reason_of(s.answer(&no_token, 2_000).unwrap_err()), "bad_credential");
    no_token.supervisor_token = Some(a.nonce.clone());
    assert_eq!(reason_of(s.answer(&no_token, 2_000).unwrap_err()), "bad_credential");
    // ack: the nonce.
    s.answer(&answer(&a, "c1", None), 2_000).unwrap();
    let mut bad_ack = ack(&a, "c1", "received");
    bad_ack.nonce = a.supervisor_token.clone();
    assert_eq!(reason_of(s.ack(&bad_ack, 2_100).unwrap_err()), "bad_nonce");
    // An answer to a question that was never asked.
    assert_eq!(reason_of(s.answer(&answer(&a, "c2", Some("ghost")), 2_200).unwrap_err()), "cas_mismatch");
    // A stale attempt: allocate attempt 2 once attempt 1 is done, then write to attempt 1.
    s.transition(
        &TransitionRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: 1,
            nonce: a.nonce.clone(),
            supervisor_token: Some(a.supervisor_token.clone()),
            expected_prior: "running".into(),
            to: "done".into(),
            supervisor_gen: 1,
            evidence_digest: Some("E".into()),
            exit_code: Some(0),
            handback: Some(Handback {
                attempt: Some(1),
                nonce: Some(a.nonce.clone()),
                status: Some("complete".into()),
                summary: None,
            }),
            ..Default::default()
        },
        3_000,
    )
    .unwrap();
    // A terminal attempt takes no new question or answer.
    assert_eq!(reason_of(s.ask(&ask(&a, "q9"), 3_100).unwrap_err()), "forbidden_transition");
    assert_eq!(reason_of(s.answer(&answer(&a, "c9", None), 3_100).unwrap_err()), "forbidden_transition");
    let b = s
        .allocate(
            &AllocateRequest { attempt_of: Some(a.dispatch_id.clone()), ..Default::default() },
            4_000,
        )
        .unwrap();
    assert_eq!(b.attempt, 2);
    assert_eq!(reason_of(s.answer(&answer(&a, "c10", None), 4_100).unwrap_err()), "stale_attempt");
}

#[test]
fn replays_are_idempotent_and_conflicts_are_refused() {
    let (_d, s) = store();
    let a = running(&s);
    assert!(!s.ask(&ask(&a, "q1"), 2_000).unwrap().deduped);
    assert!(s.ask(&ask(&a, "q1"), 2_001).unwrap().deduped);
    let mut other = ask(&a, "q1");
    other.body = "different".into();
    assert_eq!(reason_of(s.ask(&other, 2_002).unwrap_err()), "cas_mismatch");
    assert!(!s.answer(&answer(&a, "c1", Some("q1")), 2_100).unwrap().deduped);
    assert!(s.answer(&answer(&a, "c1", Some("q1")), 2_101).unwrap().deduped);
    let mut conflict = answer(&a, "c1", Some("q1"));
    conflict.body = "other".into();
    assert_eq!(reason_of(s.answer(&conflict, 2_102).unwrap_err()), "cas_mismatch");
    assert_eq!(answers(&s, &a).len(), 1);
}

/// CC-12: an answer nobody acked within ANSWER_TTL is bounced by the sweep: `expired`,
/// and the question is marked bounced. Nothing flips it before the sweep runs, and a late
/// ack cannot revive it.
#[test]
fn cc12_answer_ttl_expiry_bounces() {
    let (_d, s) = store();
    let a = running(&s);
    s.ask(&ask(&a, "q1"), 2_000).unwrap();
    s.answer(&answer(&a, "c1", Some("q1")), 3_000).unwrap();
    // Just inside the TTL: the sweep does nothing.
    assert!(s.sweep_answers(3_000 + ANSWER_TTL_MS - 1).unwrap().is_empty());
    assert_eq!(answers(&s, &a)[0]["state"], "stored");
    // Past the deadline but before the sweep: still `stored` (state is the sweep's to move),
    // yet a late ack is refused.
    let late = 3_000 + ANSWER_TTL_MS + 5;
    assert_eq!(reason_of(s.ack(&ack(&a, "c1", "received"), late).unwrap_err()), "cas_mismatch");
    assert_eq!(answers(&s, &a)[0]["state"], "stored");
    // The sweep bounces it, once.
    let closed = s.sweep_answers(late).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!((closed[0].to.as_str(), closed[0].cause.as_str()), ("expired", "answer_ttl"));
    assert!(s.sweep_answers(late + 1).unwrap().is_empty());
    let got = s.get(&a.dispatch_id).unwrap();
    assert_eq!(got["attempts"][0]["answers"][0]["state"], "expired");
    assert_eq!(got["attempts"][0]["answers"][0]["closed_at"], late);
    assert_eq!(got["attempts"][0]["questions"][0]["bounced_at"], late);
    assert_eq!(reason_of(s.ack(&ack(&a, "c1", "received"), late + 2).unwrap_err()), "cas_mismatch");
    // A received answer is not bounced by the clock: delivery happened.
    s.answer(&answer(&a, "c2", None), late + 10).unwrap();
    s.ack(&ack(&a, "c2", "received"), late + 11).unwrap();
    assert!(s.sweep_answers(late + 10 + ANSWER_TTL_MS + 1).unwrap().is_empty());
}

/// An answer still `stored` when its attempt ends can never be delivered: `undelivered`.
#[test]
fn answer_to_an_attempt_that_ended_is_undelivered() {
    let (_d, s) = store();
    let a = running(&s);
    s.answer(&answer(&a, "c1", None), 2_000).unwrap();
    s.transition(
        &TransitionRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: 1,
            nonce: a.nonce.clone(),
            supervisor_token: Some(a.supervisor_token.clone()),
            expected_prior: "running".into(),
            to: "done".into(),
            supervisor_gen: 1,
            evidence_digest: Some("E".into()),
            exit_code: Some(0),
            handback: Some(Handback {
                attempt: Some(1),
                nonce: Some(a.nonce.clone()),
                status: Some("complete".into()),
                summary: None,
            }),
            ..Default::default()
        },
        3_000,
    )
    .unwrap();
    let closed = s.sweep_answers(3_100).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!((closed[0].to.as_str(), closed[0].cause.as_str()), ("undelivered", "attempt_ended"));
    assert_eq!(answers(&s, &a)[0]["state"], "undelivered");
}

/// CC-10: the parent (dispatcher) session is lost. The records live in the store, not in
/// that session, so nothing about the attempt changes and the question stays open; a
/// successor holding the supervisor token answers it, and the worker's ack still lands.
/// With no successor the answer path stays quiet, and the sweep never turns an
/// unanswered question into a state change.
#[test]
fn cc10_parent_session_loss_does_not_lose_the_question_or_the_attempt() {
    let d = tempfile::tempdir().unwrap();
    let db = d.path().join("meta.db");
    let s = WcStore::open(&db).unwrap();
    let a = s
        .allocate(
            &AllocateRequest { dispatcher_session: Some("claude-master-1".into()), ..Default::default() },
            0,
        )
        .unwrap();
    s.transition(
        &TransitionRequest {
            dispatch_id: a.dispatch_id.clone(),
            attempt: 1,
            nonce: a.nonce.clone(),
            supervisor_token: Some(a.supervisor_token.clone()),
            expected_prior: "dispatched".into(),
            to: "running".into(),
            supervisor_gen: 1,
            ..Default::default()
        },
        1_000,
    )
    .unwrap();
    s.ask(&ask(&a, "q1"), 2_000).unwrap();

    // The parent session dies (and the hub restarts with it): a fresh store on the same file.
    drop(s);
    let s = WcStore::open(&db).unwrap();
    // Sweeps with the parent gone and the worker still registered: attempt stays running.
    let obs = SweepObservation {
        listing_complete: true,
        live: std::collections::HashSet::from([(a.dispatch_id.clone(), 1)]),
    };
    s.sweep(&obs, 60_000).unwrap();
    assert!(s.sweep_answers(60_000).unwrap().is_empty());
    let got = s.get(&a.dispatch_id).unwrap();
    assert_eq!(got["attempts"][0]["state"], "running");
    assert_eq!(got["attempts"][0]["questions"][0]["answered_cmd_id"], Value::Null);
    assert_eq!(got["attempts"][0]["questions"][0]["bounced_at"], Value::Null);
    assert_eq!(got["dispatcher"]["session"], "claude-master-1");

    // A successor session with the token answers; the worker acks both steps.
    s.answer(&answer(&a, "c1", Some("q1")), 70_000).unwrap();
    s.ack(&ack(&a, "c1", "received"), 71_000).unwrap();
    s.ack(&ack(&a, "c1", "applied"), 72_000).unwrap();
    assert_eq!(answers(&s, &a)[0]["state"], "applied");
}

/// An attempt in `unknown` is `unknown`: reach it by the deadline, the way the sweep does.
fn unknown_attempt(s: &WcStore) -> Allocation {
    let a = s.allocate(&AllocateRequest::default(), 0).unwrap();
    let past = a.deadline_dispatched + 1;
    s.sweep(&SweepObservation { listing_complete: true, ..Default::default() }, past).unwrap();
    assert_eq!(s.get(&a.dispatch_id).unwrap()["attempts"][0]["state"], "unknown");
    a
}

fn resolve(a: &Allocation, as_state: &str, reason: &str) -> ResolveRequest {
    ResolveRequest {
        dispatch_id: a.dispatch_id.clone(),
        attempt: a.attempt,
        as_state: as_state.into(),
        reason: reason.into(),
    }
}

/// CC-15: an operator resolves an `unknown` attempt; the reason is in history; the record
/// says it was attested, with no invented outcome; anything but `unknown` is refused.
#[test]
fn cc15_unknown_resolution_by_operator() {
    let (_d, s) = store();
    let a = unknown_attempt(&s);
    assert!(matches!(s.resolve(&resolve(&a, "done", "  "), 9), Err(WcError::Invalid(_))));
    assert!(matches!(s.resolve(&resolve(&a, "running", "x"), 9), Err(WcError::Invalid(_))));
    let rec = s.resolve(&resolve(&a, "done", "worker finished, checked the branch"), 500_000).unwrap();
    assert_eq!(rec["state"], "done");
    assert_eq!(rec["attested"], true);
    assert_eq!(rec["attested_by"], "operator");
    assert_eq!(rec["outcome"], Value::Null);
    let hist = rec["history"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(hist["writer"], "operator");
    assert_eq!((hist["from"].as_str(), hist["to"].as_str()), (Some("unknown"), Some("done")));
    assert!(hist["cause"].as_str().unwrap().contains("worker finished, checked the branch"));
    // No longer unknown: a second resolve is refused and leaves a refusal row.
    assert_eq!(reason_of(s.resolve(&resolve(&a, "lost", "again"), 500_100).unwrap_err()), "cas_mismatch");
    assert!(fence_rows(&s, &a).contains(&"cas_mismatch".to_string()));

    // The other way: unknown -> lost.
    let b = unknown_attempt(&s);
    let rec = s.resolve(&resolve(&b, "lost", "host was reimaged"), 600_000).unwrap();
    assert_eq!(rec["state"], "lost");
    assert_eq!(rec["attested"], false);

    // Only `unknown` qualifies: running and dispatched are refused.
    let c = running(&s);
    assert_eq!(reason_of(s.resolve(&resolve(&c, "lost", "no"), 700_000).unwrap_err()), "cas_mismatch");
    let dsp = s.allocate(&AllocateRequest::default(), 0).unwrap();
    assert_eq!(reason_of(s.resolve(&resolve(&dsp, "lost", "no"), 700_000).unwrap_err()), "cas_mismatch");
    assert!(matches!(s.resolve(&resolve(&dsp, "lost", "x").clone_with_id("D-nope"), 1), Err(WcError::NotFound(_))));
}

impl ResolveRequest {
    fn clone_with_id(&self, id: &str) -> Self {
        Self { dispatch_id: id.into(), ..self.clone() }
    }
}

/// The CLI path opens the store by directory, never creates one, and works on a file the
/// hub keeps open.
#[test]
fn resolve_at_works_on_a_live_store_and_never_creates_one() {
    let d = tempfile::tempdir().unwrap();
    assert!(matches!(resolve_at(d.path(), &ResolveRequest::default(), 1), Err(WcError::NotFound(_))));
    assert!(!d.path().join("meta.db").exists());
    let s = WcStore::open(&d.path().join("meta.db")).unwrap();
    let a = unknown_attempt(&s);
    let rec = resolve_at(d.path(), &resolve(&a, "lost", "gone"), 800_000).unwrap();
    assert_eq!(rec["state"], "lost");
    assert_eq!(s.get(&a.dispatch_id).unwrap()["attempts"][0]["state"], "lost");
}
