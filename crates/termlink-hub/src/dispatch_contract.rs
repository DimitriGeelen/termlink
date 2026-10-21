//! Shared worker contract, hub backend, slice S1 (T-3501): `dispatch.allocate`,
//! `dispatch.transition`, `dispatch.get`. The store and the fencing live in
//! `termlink_bus::worker_contract`; this module is the JSON-RPC edge.
//!
//! Scopes: allocate and transition are Execute (deny-by-default for unlisted methods),
//! get is Observe (listed in `server.rs`). Time is read here and passed down, so the
//! store is testable with injected time.

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::{json, Value};
use termlink_bus::Bus;
use termlink_bus::worker_contract::sweep::SweepObservation;
use termlink_bus::worker_contract::answer::{
    resolve_at, AckRequest, AnswerRequest, AskRequest, ResolveRequest,
};
use termlink_bus::worker_contract::{
    AllocateRequest, Handback, StopRequest, TransitionRequest, WcError,
};
use termlink_protocol::control::error_code;
use termlink_protocol::jsonrpc::{ErrorResponse, Response, RpcResponse};
use termlink_session::{liveness, manager};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn s(params: &Value, k: &str) -> Option<String> {
    params.get(k).and_then(|v| v.as_str()).map(str::to_string)
}

fn i(params: &Value, k: &str) -> Option<i64> {
    params.get(k).and_then(|v| v.as_i64())
}

fn bus_or_err(id: &Value) -> Result<&'static Bus, RpcResponse> {
    crate::channel::bus().ok_or_else(|| {
        ErrorResponse::internal_error(id.clone(), "channel bus not initialized").into()
    })
}

fn err(id: Value, e: WcError) -> RpcResponse {
    match e {
        WcError::IdTaken(d) => ErrorResponse::with_data(
            id,
            error_code::DISPATCH_ID_TAKEN,
            &format!("dispatch_id {d} is already used; no other id is substituted"),
            json!({"dispatch_id": d, "reason": "id_taken"}),
        )
        .into(),
        WcError::BadId(d) => ErrorResponse::new(
            id,
            -32602,
            &format!("dispatch_id {d:?} is not a well-formed D-<ulid>"),
        )
        .into(),
        WcError::NotFound(d) => ErrorResponse::with_data(
            id,
            error_code::DISPATCH_NOT_FOUND,
            &format!("dispatch not found: {d}"),
            json!({"dispatch_id": d}),
        )
        .into(),
        WcError::Fence { reason, detail } => ErrorResponse::with_data(
            id,
            error_code::FENCE_REJECTED,
            &format!("write refused by the worker-contract fence: {}", reason.as_str()),
            json!({"reason": reason.as_str(), "detail": detail}),
        )
        .into(),
        WcError::Invalid(m) => ErrorResponse::new(id, -32602, &m).into(),
        WcError::Bus(e) => ErrorResponse::internal_error(id, &format!("dispatch store: {e}")).into(),
    }
}

pub async fn handle_dispatch_allocate(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => allocate_with(b, id, params, now_ms()),
        Err(r) => r,
    }
}

pub async fn handle_dispatch_transition(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => transition_with(b, id, params, now_ms()),
        Err(r) => r,
    }
}

pub async fn handle_dispatch_stop(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => stop_with(b, id, params, now_ms()),
        Err(r) => r,
    }
}

pub async fn handle_dispatch_get(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => get_with(b, id, params),
        Err(r) => r,
    }
}

pub async fn handle_dispatch_ask(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => ask_with(b, id, params, now_ms()),
        Err(r) => r,
    }
}

pub async fn handle_dispatch_answer(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => answer_with(b, id, params, now_ms()),
        Err(r) => r,
    }
}

pub async fn handle_dispatch_ack(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => ack_with(b, id, params, now_ms()),
        Err(r) => r,
    }
}

/// Who is calling `resolve_on_host` (T-3527, spec §3.3 T7d speed bump).
#[derive(Debug, Clone, Default)]
pub struct ResolveSession {
    /// Agent-environment markers were present (the CLI reads `CLAUDECODE=1`).
    pub agent: bool,
    /// An explicit override reason, recorded in history when an agent session resolves.
    pub override_reason: Option<String>,
}

/// The refusal an agent session gets without an override. Named so a caller can match it.
pub const AGENT_SESSION_REFUSAL: &str = "agent_session";

/// T-3527: the T7d speed bump. Returns the actor to record (empty: the operator), or the
/// refusal for an agent session that gave no non-empty override reason.
pub fn resolve_gate(session: &ResolveSession) -> Result<String, String> {
    let over = session.override_reason.as_deref().map(str::trim).filter(|r| !r.is_empty());
    match (session.agent, over) {
        (false, _) => Ok(String::new()),
        (true, Some(r)) => Ok(format!("agent session, override: {r}")),
        (true, None) => Err(format!(
            "refused ({AGENT_SESSION_REFUSAL}): this looks like an agent session (CLAUDECODE=1). \
             T7d resolve is the operator's; this check is a speed bump, not identity. To resolve \
             anyway, pass --override \"<why>\", which is recorded in the attempt's history"
        )),
    }
}

/// T-3503 (S3), T7d: the operator resolves an `unknown` attempt. Hub-host only: this is
/// called by the CLI on the host that runs the hub, and nothing in the router reaches it.
/// Until a signed operator identity exists (T-3493), being on the host is the identity.
/// T-3527: an agent session is refused unless it carries a non-empty override reason, which
/// is then recorded in the history row. This is a speed bump, not identity: an agent can set
/// the override, and the record shows that it did.
pub fn resolve_on_host(
    bus_root: &std::path::Path,
    dispatch_id: &str,
    attempt: i64,
    as_state: &str,
    reason: &str,
    session: &ResolveSession,
) -> Result<Value, String> {
    let actor = resolve_gate(session)?;
    let req = ResolveRequest {
        dispatch_id: dispatch_id.to_string(),
        attempt,
        as_state: as_state.to_string(),
        reason: reason.to_string(),
        actor,
    };
    resolve_at(bus_root, &req, now_ms()).map_err(|e| match e {
        WcError::NotFound(d) => format!("not found: {d}"),
        WcError::Invalid(m) => m,
        WcError::Fence { reason, detail } => format!("refused ({}): {detail}", reason.as_str()),
        other => format!("{other:?}"),
    })
}

/// T-3502 (S2): what the sweep sees of the registrations under `dirs`. A dir that
/// cannot be listed makes the listing incomplete (the hub bumps the epoch); a dir that
/// does not exist is empty, not unreadable (`list_sessions_in` says so itself).
pub(crate) fn scan_bindings(dirs: &[PathBuf]) -> SweepObservation {
    let mut obs = SweepObservation { listing_complete: true, live: HashSet::new() };
    for dir in dirs {
        match manager::list_sessions_in(dir, true) {
            Ok(regs) => {
                for reg in regs.iter().filter(|r| liveness::is_alive(r)) {
                    if let Some(b) = &reg.worker_binding {
                        obs.live.insert((b.dispatch_id.clone(), i64::from(b.attempt)));
                    }
                }
            }
            Err(e) => {
                obs.listing_complete = false;
                tracing::warn!(dir = %dir.display(), error = %e, "dispatch sweep: sessions dir unreadable; epoch bumped, no absence counted");
            }
        }
    }
    obs
}

/// T-3502 (S2): one state sweep, run as a step of the supervisor loop. Failures are
/// logged and skipped: the next sweep sees the same state again.
pub(crate) fn sweep_state(bus: &Bus, dirs: &[PathBuf]) {
    sweep_answers(bus, now_ms());
    let obs = scan_bindings(dirs);
    match bus.wc_sweep(&obs, now_ms()) {
        Ok(report) => {
            for t in &report.transitions {
                tracing::info!(
                    dispatch_id = %t.dispatch_id, attempt = t.attempt, from = %t.from, to = %t.to,
                    cause = %t.cause, epoch = report.epoch, "dispatch sweep: state written by hub"
                );
            }
            for n in &report.notices {
                tracing::warn!(epoch = report.epoch, "dispatch sweep: {n}");
            }
        }
        Err(e) => tracing::warn!(error = ?e, "dispatch sweep failed"),
    }
}

/// T-3503 (S3): the answer step of the sweep, `undelivered` and `expired`, from state.
pub(crate) fn sweep_answers(bus: &Bus, now: i64) {
    match bus.wc_sweep_answers(now) {
        Ok(closed) => {
            for c in &closed {
                tracing::warn!(
                    dispatch_id = %c.dispatch_id, attempt = c.attempt, cmd_id = %c.cmd_id,
                    to = %c.to, cause = %c.cause, "dispatch sweep: answer closed by hub"
                );
            }
        }
        Err(e) => tracing::warn!(error = ?e, "dispatch answer sweep failed"),
    }
}

pub(crate) fn ask_with(bus: &Bus, id: Value, params: &Value, now: i64) -> RpcResponse {
    let (Some(dispatch_id), Some(nonce), Some(attempt), Some(body)) =
        (s(params, "dispatch_id"), s(params, "nonce"), i(params, "attempt"), s(params, "body"))
    else {
        return ErrorResponse::new(id, -32602, "dispatch.ask needs dispatch_id, attempt, nonce, body").into();
    };
    let req = AskRequest { dispatch_id, attempt, nonce, question_id: s(params, "question_id"), body };
    match bus.wc_ask(&req, now) {
        Ok(o) => Response::success(id, json!({"question": o.record, "deduped": o.deduped})).into(),
        Err(e) => err(id, e),
    }
}

pub(crate) fn answer_with(bus: &Bus, id: Value, params: &Value, now: i64) -> RpcResponse {
    let (Some(dispatch_id), Some(attempt), Some(body)) =
        (s(params, "dispatch_id"), i(params, "attempt"), s(params, "body"))
    else {
        return ErrorResponse::new(id, -32602, "dispatch.answer needs dispatch_id, attempt, supervisor_token, body").into();
    };
    let req = AnswerRequest {
        dispatch_id,
        attempt,
        supervisor_token: s(params, "supervisor_token"),
        cmd_id: s(params, "cmd_id"),
        question_id: s(params, "question_id"),
        body,
    };
    match bus.wc_answer(&req, now) {
        Ok(o) => Response::success(id, json!({"answer": o.record, "deduped": o.deduped})).into(),
        Err(e) => err(id, e),
    }
}

pub(crate) fn ack_with(bus: &Bus, id: Value, params: &Value, now: i64) -> RpcResponse {
    let (Some(dispatch_id), Some(nonce), Some(attempt), Some(cmd_id), Some(state)) = (
        s(params, "dispatch_id"),
        s(params, "nonce"),
        i(params, "attempt"),
        s(params, "cmd_id"),
        s(params, "state"),
    ) else {
        return ErrorResponse::new(id, -32602, "dispatch.ack needs dispatch_id, attempt, nonce, cmd_id, state").into();
    };
    let req = AckRequest { dispatch_id, attempt, nonce, cmd_id, state };
    match bus.wc_ack(&req, now) {
        Ok(o) => Response::success(id, json!({"answer": o.record, "deduped": o.deduped})).into(),
        Err(e) => err(id, e),
    }
}

pub(crate) fn allocate_with(bus: &Bus, id: Value, params: &Value, now: i64) -> RpcResponse {
    let dispatcher = params.get("dispatcher");
    let d = |k: &str| dispatcher.and_then(|x| x.get(k)).and_then(|v| v.as_str()).map(str::to_string);
    let req = AllocateRequest {
        dispatch_id: s(params, "dispatch_id"),
        attempt_of: s(params, "attempt_of"),
        dispatcher_project_agent: d("project_agent"),
        dispatcher_session: d("session"),
        claude_session_id: d("claude_session_id"),
        task_id: s(params, "task_id"),
        retry_of_dispatch_id: s(params, "retry_of"),
        worker_kind: s(params, "worker_kind"),
        session_name: s(params, "session_name"),
        spawn_timeout_ms: i(params, "spawn_timeout_ms"),
    };
    match bus.wc_allocate(&req, now) {
        Ok(a) => Response::success(
            id,
            json!({
                "dispatch_id": a.dispatch_id,
                "attempt": a.attempt,
                "nonce": a.nonce,
                "supervisor_token": a.supervisor_token,
                "deadline_dispatched": a.deadline_dispatched,
            }),
        )
        .into(),
        Err(e) => err(id, e),
    }
}

pub(crate) fn transition_with(bus: &Bus, id: Value, params: &Value, now: i64) -> RpcResponse {
    let need = |k: &str| s(params, k);
    let (Some(dispatch_id), Some(nonce), Some(expected_prior), Some(to)) =
        (need("dispatch_id"), need("nonce"), need("expected_prior"), need("to"))
    else {
        return ErrorResponse::new(
            id,
            -32602,
            "dispatch.transition needs dispatch_id, attempt, nonce, expected_prior, to, supervisor_gen",
        )
        .into();
    };
    let (Some(attempt), Some(supervisor_gen)) = (i(params, "attempt"), i(params, "supervisor_gen")) else {
        return ErrorResponse::new(id, -32602, "dispatch.transition needs integer attempt and supervisor_gen").into();
    };
    let result = params.get("result");
    let hb = params.get("handback").filter(|h| h.is_object()).map(|h| Handback {
        attempt: h.get("attempt").and_then(|v| v.as_i64()),
        nonce: h.get("nonce").and_then(|v| v.as_str()).map(str::to_string),
        status: h.get("status").and_then(|v| v.as_str()).map(str::to_string),
        summary: h.get("summary").and_then(|v| v.as_str()).map(str::to_string),
    });
    let req = TransitionRequest {
        dispatch_id,
        attempt,
        nonce,
        supervisor_token: s(params, "supervisor_token"),
        expected_prior,
        to,
        supervisor_gen,
        evidence_digest: s(params, "evidence_digest"),
        exit_code: i(params, "exit_code"),
        refused: params.get("refused").and_then(|v| v.as_bool()).unwrap_or(false),
        result_kind: result.and_then(|r| r.get("kind")).and_then(|v| v.as_str()).map(str::to_string),
        result_value: result.and_then(|r| r.get("value")).and_then(|v| v.as_str()).map(str::to_string),
        persisted_at: result.and_then(|r| r.get("persisted_at")).and_then(|v| v.as_i64()),
        handback: hb,
        worker_timeout_ms: i(params, "worker_timeout_ms"),
        cause: s(params, "cause"),
    };
    match bus.wc_transition(&req, now) {
        Ok(o) => Response::success(id, json!({"record": o.record, "deduped": o.deduped})).into(),
        Err(e) => err(id, e),
    }
}

/// T-3523: `dispatch.stop {dispatch_id, attempt, supervisor_token, cause}`.
pub(crate) fn stop_with(bus: &Bus, id: Value, params: &Value, now: i64) -> RpcResponse {
    let (Some(dispatch_id), Some(attempt), Some(cause)) = (s(params, "dispatch_id"), i(params, "attempt"), s(params, "cause"))
    else {
        return ErrorResponse::new(id, -32602, "dispatch.stop needs dispatch_id, attempt, cause").into();
    };
    let req = StopRequest { dispatch_id, attempt, supervisor_token: s(params, "supervisor_token"), cause };
    match bus.wc_stop(&req, now) {
        Ok(record) => Response::success(id, json!({"record": record})).into(),
        Err(e) => err(id, e),
    }
}

pub(crate) fn get_with(bus: &Bus, id: Value, params: &Value) -> RpcResponse {
    let Some(dispatch_id) = s(params, "dispatch_id") else {
        return ErrorResponse::new(id, -32602, "Missing 'dispatch_id' in params").into();
    };
    match bus.wc_get(&dispatch_id) {
        Ok(v) => Response::success(id, v).into(),
        Err(e) => err(id, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bus() -> (tempfile::TempDir, Bus) {
        let d = tempfile::tempdir().unwrap();
        let b = Bus::open(d.path()).unwrap();
        (d, b)
    }

    fn ok(r: RpcResponse) -> Value {
        match r {
            RpcResponse::Success(s) => s.result,
            RpcResponse::Error(e) => panic!("expected success, got {e:?}"),
        }
    }

    fn fail(r: RpcResponse) -> (i64, Value) {
        match r {
            RpcResponse::Error(e) => (e.error.code, e.error.data.unwrap_or(Value::Null)),
            RpcResponse::Success(s) => panic!("expected error, got {s:?}"),
        }
    }

    fn lifecycle(a: &Value, to: &str, prior: &str) -> Value {
        json!({
            "dispatch_id": a["dispatch_id"], "attempt": a["attempt"], "nonce": a["nonce"],
            "supervisor_token": a["supervisor_token"], "expected_prior": prior, "to": to,
            "supervisor_gen": 1, "evidence_digest": "E1", "exit_code": 0,
            "handback": {"attempt": a["attempt"], "nonce": a["nonce"], "status": "complete"},
        })
    }

    #[test]
    fn allocate_transition_get_round_trip() {
        let (_d, b) = bus();
        let a = ok(allocate_with(&b, json!(1), &json!({"dispatcher": {"session": "claude-master-1"}}), 1_000));
        assert_eq!(a["attempt"], 1);
        assert!(a["nonce"].as_str().unwrap().len() > 8);
        ok(transition_with(&b, json!(2), &lifecycle(&a, "running", "dispatched"), 2_000));
        let done = ok(transition_with(&b, json!(3), &lifecycle(&a, "done", "running"), 3_000));
        assert_eq!(done["record"]["outcome"], "succeeded");
        assert_eq!(done["deduped"], false);
        let again = ok(transition_with(&b, json!(4), &lifecycle(&a, "done", "running"), 4_000));
        assert_eq!(again["deduped"], true);
        let got = ok(get_with(&b, json!(5), &json!({"dispatch_id": a["dispatch_id"]})));
        assert_eq!(got["attempts"][0]["state"], "done");
        assert_eq!(got["dispatcher"]["session"], "claude-master-1");
    }

    /// T-3523 (CC-25): `dispatch.stop` records the cause without a state change; the T3 that
    /// follows carries it; a missing field is -32602 and a wrong token is a fence refusal.
    #[test]
    fn dispatch_stop_records_the_cause_for_the_confirmed_end() {
        let (_d, b) = bus();
        let a = ok(allocate_with(&b, json!(1), &json!({}), 1_000));
        ok(transition_with(&b, json!(2), &lifecycle(&a, "running", "dispatched"), 2_000));
        let (code, _) = fail(stop_with(&b, json!(3), &json!({"dispatch_id": a["dispatch_id"], "attempt": 1}), 2_500));
        assert_eq!(code, -32602);
        let bad = json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "supervisor_token": "nope", "cause": "deadline_stop"});
        let (_, data) = fail(stop_with(&b, json!(4), &bad, 2_600));
        assert_eq!(data["reason"], "bad_credential");
        let req = json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "supervisor_token": a["supervisor_token"], "cause": "deadline_stop"});
        let r = ok(stop_with(&b, json!(5), &req, 3_000));
        assert_eq!(r["record"]["state"], "running");
        assert_eq!(r["record"]["stop_cause"], "deadline_stop");
        ok(transition_with(&b, json!(6), &lifecycle(&a, "done", "running"), 4_000));
        let got = ok(get_with(&b, json!(7), &json!({"dispatch_id": a["dispatch_id"]})));
        let last = got["attempts"][0]["history"].as_array().unwrap().last().unwrap().clone();
        assert_eq!((last["to"].as_str(), last["cause"].as_str()), (Some("done"), Some("deadline_stop")));
        assert_eq!(got["attempts"].as_array().unwrap().len(), 1, "a stop allocates nothing");
    }

    #[test]
    fn dispatch_get_returns_the_attempts_notices() {
        // T-3521: a done-conflict refusal is a durable operator notice, shown by dispatch.get.
        let (_d, b) = bus();
        let a = ok(allocate_with(&b, json!(1), &json!({}), 1_000));
        ok(transition_with(&b, json!(2), &lifecycle(&a, "running", "dispatched"), 2_000));
        ok(transition_with(&b, json!(3), &lifecycle(&a, "done", "running"), 3_000));
        let mut other = lifecycle(&a, "done", "running");
        other["evidence_digest"] = json!("E2");
        let (code, data) = fail(transition_with(&b, json!(4), &other, 4_000));
        assert_ne!(code, 0);
        assert_eq!(data["reason"], "done_conflict");
        let got = ok(get_with(&b, json!(5), &json!({"dispatch_id": a["dispatch_id"]})));
        let n = got["attempts"][0]["notices"].as_array().unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0]["kind"], "done_conflict");
        assert_eq!(n[0]["audience"], "operator");
    }

    #[test]
    fn fence_rejected_carries_reason() {
        let (_d, b) = bus();
        let a = ok(allocate_with(&b, json!(1), &json!({}), 1_000));
        let mut bad = lifecycle(&a, "running", "dispatched");
        bad["nonce"] = json!("wrong");
        let (code, data) = fail(transition_with(&b, json!(2), &bad, 2_000));
        assert_eq!(code, error_code::FENCE_REJECTED);
        assert_eq!(data["reason"], "bad_nonce");
        let mut nocred = lifecycle(&a, "running", "dispatched");
        nocred.as_object_mut().unwrap().remove("supervisor_token");
        let (_, data) = fail(transition_with(&b, json!(3), &nocred, 2_001));
        assert_eq!(data["reason"], "bad_credential");
    }

    #[test]
    fn id_taken_and_not_found_have_their_own_codes() {
        let (_d, b) = bus();
        let want = "D-01J9ZZZZZZZZZZZZZZZZZZZZZZ";
        ok(allocate_with(&b, json!(1), &json!({"dispatch_id": want}), 1));
        let (code, data) = fail(allocate_with(&b, json!(2), &json!({"dispatch_id": want}), 2));
        assert_eq!(code, error_code::DISPATCH_ID_TAKEN);
        assert_eq!(data["dispatch_id"], want);
        let (code, _) = fail(get_with(&b, json!(3), &json!({"dispatch_id": "D-00000000000000000000000000"})));
        assert_eq!(code, error_code::DISPATCH_NOT_FOUND);
        let (code, _) = fail(allocate_with(&b, json!(4), &json!({"dispatch_id": "bad"}), 3));
        assert_eq!(code, -32602);
    }

    /// T-3502: the sweep's view of the registrations. Fake liveness is a dead pid vs
    /// this process; the dirs are temp dirs (T-3293).
    #[test]
    fn scan_bindings_sees_only_live_bound_registrations() {
        use termlink_session::identity::SessionId;
        use termlink_session::registration::{Registration, SessionConfig, WorkerBinding};
        let dir = tempfile::tempdir().unwrap();
        let bound = "D-01J9ZZZZZZZZZZZZZZZZZZZZZZ";
        let mk = |pid: u32, binding: Option<&str>| {
            let id = SessionId::generate();
            let sock = dir.path().join(format!("{id}.sock"));
            std::fs::write(&sock, b"").unwrap();
            let mut reg = Registration::new(id.clone(), SessionConfig::default(), sock);
            reg.pid = pid;
            reg.worker_binding = binding.map(|b| WorkerBinding::parse(b).unwrap());
            let json = serde_json::to_string_pretty(&reg).unwrap();
            std::fs::write(Registration::json_path(dir.path(), &id), json).unwrap();
        };
        mk(std::process::id(), Some(&format!("{bound}:2"))); // live, bound
        mk(4_000_000, Some(&format!("{bound}:3"))); // dead pid: not live
        mk(std::process::id(), None); // live, unbound

        let obs = scan_bindings(&[dir.path().to_path_buf()]);
        assert!(obs.listing_complete);
        assert_eq!(obs.live, HashSet::from([(bound.to_string(), 2)]));

        // A path that exists but is not a directory cannot be listed: incomplete, not empty.
        let file = dir.path().join("not-a-dir");
        std::fs::write(&file, b"").unwrap();
        let obs = scan_bindings(&[file]);
        assert!(!obs.listing_complete);
        assert!(obs.live.is_empty());

        // A dir that does not exist holds no sessions: complete and empty.
        let obs = scan_bindings(&[dir.path().join("absent")]);
        assert!(obs.listing_complete && obs.live.is_empty());
    }

    /// T-3503: the answer path through the RPC edge, end to end (CC-11 shape).
    #[test]
    fn ask_answer_ack_round_trip_over_the_rpc_edge() {
        let (_d, b) = bus();
        let a = ok(allocate_with(&b, json!(1), &json!({}), 1_000));
        ok(transition_with(&b, json!(2), &lifecycle(&a, "running", "dispatched"), 2_000));
        let q = ok(ask_with(
            &b,
            json!(3),
            &json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "nonce": a["nonce"],
                    "question_id": "q1", "body": "which branch?"}),
            3_000,
        ));
        assert_eq!(q["question"]["question_id"], "q1");
        let st = ok(answer_with(
            &b,
            json!(4),
            &json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "supervisor_token": a["supervisor_token"],
                    "cmd_id": "c1", "question_id": "q1", "body": "main"}),
            4_000,
        ));
        assert_eq!(st["answer"]["state"], "stored");
        let ack = |state: &str, now: i64| {
            ack_with(
                &b,
                json!(5),
                &json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "nonce": a["nonce"],
                        "cmd_id": "c1", "state": state}),
                now,
            )
        };
        assert_eq!(ok(ack("received", 5_000))["answer"]["state"], "received");
        assert_eq!(ok(ack("applied", 6_000))["answer"]["state"], "applied");
        // A refusal is FENCE_REJECTED with its reason.
        let (code, data) = fail(answer_with(
            &b,
            json!(6),
            &json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "supervisor_token": "no", "body": "x"}),
            7_000,
        ));
        assert_eq!(code, error_code::FENCE_REJECTED);
        assert_eq!(data["reason"], "bad_credential");
    }

    /// T-3503: the sweep hook bounces an expired answer, from state.
    #[test]
    fn sweep_answers_bounces_an_expired_answer() {
        let (_d, b) = bus();
        let a = ok(allocate_with(&b, json!(1), &json!({}), 1_000));
        ok(transition_with(&b, json!(2), &lifecycle(&a, "running", "dispatched"), 2_000));
        ok(answer_with(
            &b,
            json!(3),
            &json!({"dispatch_id": a["dispatch_id"], "attempt": 1, "supervisor_token": a["supervisor_token"],
                    "cmd_id": "c1", "body": "main"}),
            3_000,
        ));
        sweep_answers(&b, 3_000 + 600_000);
        let got = ok(get_with(&b, json!(4), &json!({"dispatch_id": a["dispatch_id"]})));
        assert_eq!(got["attempts"][0]["answers"][0]["state"], "expired");
    }

    /// T-3503 / CC-15: `dispatch.resolve` has no network surface. The method constant does
    /// not exist, the router does not route it, the capability list does not name it, and
    /// the host entry point refuses anything that is not `unknown`.
    #[test]
    fn resolve_is_host_only_and_needs_an_unknown_attempt() {
        let src = include_str!("router.rs");
        assert!(!src.contains("dispatch.resolve") && !src.contains("DISPATCH_RESOLVE"));
        let ctl = include_str!("../../termlink-protocol/src/control.rs");
        assert!(!ctl.contains("pub const DISPATCH_RESOLVE"));

        let d = tempfile::tempdir().unwrap();
        let bus = Bus::open(d.path()).unwrap();
        let a = ok(allocate_with(&bus, json!(1), &json!({}), 1_000));
        let id = a["dispatch_id"].as_str().unwrap();
        let op = ResolveSession::default();
        // Still `dispatched`: refused, and the refusal names the reason.
        let e = resolve_on_host(d.path(), id, 1, "lost", "operator says so", &op).unwrap_err();
        assert!(e.contains("cas_mismatch"), "{e}");
        // Past the dispatched deadline the sweep makes it `unknown`; then resolve works.
        let past = a["deadline_dispatched"].as_i64().unwrap() + 1;
        bus.wc_sweep(
            &termlink_bus::worker_contract::sweep::SweepObservation { listing_complete: true, ..Default::default() },
            past,
        )
        .unwrap();
        let rec = resolve_on_host(d.path(), id, 1, "done", "checked by hand", &op).unwrap();
        assert_eq!(rec["state"], "done");
        assert_eq!(rec["attested"], true);
        let e = resolve_on_host(d.path(), "D-01J9ZZZZZZZZZZZZZZZZZZZZZZ", 1, "lost", "x", &op).unwrap_err();
        assert!(e.starts_with("not found"), "{e}");
    }

    /// T-3527 (CC-15 speed bump): an agent session is refused without an override and writes
    /// nothing; with an override it resolves and the override is in the history row; the
    /// writer stays `operator`, so the T7d notice still fires.
    #[test]
    fn resolve_refuses_an_agent_session_without_a_recorded_override() {
        let d = tempfile::tempdir().unwrap();
        let bus = Bus::open(d.path()).unwrap();
        let a = ok(allocate_with(&bus, json!(1), &json!({}), 1_000));
        let id = a["dispatch_id"].as_str().unwrap().to_string();
        let past = a["deadline_dispatched"].as_i64().unwrap() + 1;
        bus.wc_sweep(
            &termlink_bus::worker_contract::sweep::SweepObservation { listing_complete: true, ..Default::default() },
            past,
        )
        .unwrap();
        let before = bus.wc_get(&id).unwrap();

        let agent = ResolveSession { agent: true, override_reason: None };
        let e = resolve_on_host(d.path(), &id, 1, "lost", "host reimaged", &agent).unwrap_err();
        assert!(e.contains(AGENT_SESSION_REFUSAL), "{e}");
        let blank = ResolveSession { agent: true, override_reason: Some("  ".into()) };
        assert!(resolve_on_host(d.path(), &id, 1, "lost", "host reimaged", &blank).unwrap_err().contains(AGENT_SESSION_REFUSAL));
        assert_eq!(bus.wc_get(&id).unwrap(), before, "a refused resolve writes nothing");

        let over = ResolveSession { agent: true, override_reason: Some("operator asked me to".into()) };
        let rec = resolve_on_host(d.path(), &id, 1, "lost", "host reimaged", &over).unwrap();
        assert_eq!(rec["state"], "lost");
        let got = bus.wc_get(&id).unwrap();
        let row = got["attempts"][0]["history"].as_array().unwrap().last().unwrap().clone();
        assert_eq!(row["writer"], "operator");
        let cause = row["cause"].as_str().unwrap();
        assert!(cause.contains("override: operator asked me to") && cause.contains("host reimaged"), "{cause}");
    }
}
