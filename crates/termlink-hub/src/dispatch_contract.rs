//! Shared worker contract, hub backend, slice S1 (T-3501): `dispatch.allocate`,
//! `dispatch.transition`, `dispatch.get`. The store and the fencing live in
//! `termlink_bus::worker_contract`; this module is the JSON-RPC edge.
//!
//! Scopes: allocate and transition are Execute (deny-by-default for unlisted methods),
//! get is Observe (listed in `server.rs`). Time is read here and passed down, so the
//! store is testable with injected time.

use serde_json::{json, Value};
use termlink_bus::Bus;
use termlink_bus::worker_contract::{
    AllocateRequest, Handback, TransitionRequest, WcError,
};
use termlink_protocol::control::error_code;
use termlink_protocol::jsonrpc::{ErrorResponse, Response, RpcResponse};

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

pub async fn handle_dispatch_get(id: Value, params: &Value) -> RpcResponse {
    match bus_or_err(&id) {
        Ok(b) => get_with(b, id, params),
        Err(r) => r,
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
}
