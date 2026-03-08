use serde_json::json;

use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use crate::registration::Registration;

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// Returns `None` for notifications (no response expected).
pub fn dispatch(req: &Request, registration: &Registration) -> Option<RpcResponse> {
    if req.is_notification() {
        // Notifications don't get responses per JSON-RPC 2.0 spec
        tracing::debug!(method = %req.method, "Received notification");
        return None;
    }

    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let response = match req.method.as_str() {
        "termlink.ping" => handle_ping(id.clone(), registration),
        "query.status" => handle_query_status(id.clone(), registration),
        "query.capabilities" => handle_query_capabilities(id.clone(), registration),
        "session.heartbeat" => handle_heartbeat(id.clone(), registration),
        _ => ErrorResponse::method_not_found(id, &req.method).into(),
    };

    Some(response)
}

/// Handle `termlink.ping` — liveness verification.
/// Returns the session ID so the caller can confirm identity.
fn handle_ping(id: serde_json::Value, reg: &Registration) -> RpcResponse {
    Response::success(
        id,
        json!({
            "id": reg.id.as_str(),
            "state": reg.state,
            "display_name": reg.display_name,
        }),
    )
    .into()
}

/// Handle `query.status` — session state and metadata.
fn handle_query_status(id: serde_json::Value, reg: &Registration) -> RpcResponse {
    Response::success(
        id,
        json!({
            "id": reg.id.as_str(),
            "display_name": reg.display_name,
            "state": reg.state,
            "pid": reg.pid,
            "created_at": reg.created_at,
            "heartbeat_at": reg.heartbeat_at,
            "metadata": reg.metadata,
        }),
    )
    .into()
}

/// Handle `query.capabilities` — what this session supports.
fn handle_query_capabilities(id: serde_json::Value, reg: &Registration) -> RpcResponse {
    Response::success(
        id,
        json!({
            "id": reg.id.as_str(),
            "capabilities": reg.capabilities,
            "roles": reg.roles,
            "version": reg.version,
        }),
    )
    .into()
}

/// Handle `session.heartbeat` — liveness probe with timestamp update.
fn handle_heartbeat(id: serde_json::Value, reg: &Registration) -> RpcResponse {
    Response::success(
        id,
        json!({
            "id": reg.id.as_str(),
            "state": reg.state,
            "heartbeat_at": reg.heartbeat_at,
        }),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::SessionId;
    use crate::registration::SessionConfig;
    use std::path::PathBuf;
    use termlink_protocol::jsonrpc::standard_error;

    fn test_registration() -> Registration {
        let id = SessionId::generate();
        Registration::new(
            id,
            SessionConfig {
                display_name: Some("test-session".into()),
                capabilities: vec!["inject".into(), "command".into(), "query".into()],
                roles: vec!["coder".into()],
            },
            PathBuf::from("/tmp/test.sock"),
        )
    }

    #[test]
    fn ping_returns_id() {
        let reg = test_registration();
        let req = Request::new("termlink.ping", json!("req-1"), json!({}));
        let resp = dispatch(&req, &reg).unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.id, json!("req-1"));
            assert_eq!(resp.result["id"], reg.id.as_str());
            assert_eq!(resp.result["display_name"], "test-session");
        } else {
            panic!("Expected success response");
        }
    }

    #[test]
    fn query_status_returns_state() {
        let reg = test_registration();
        let req = Request::new("query.status", json!("req-2"), json!({}));
        let resp = dispatch(&req, &reg).unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["state"], "initializing");
            assert_eq!(resp.result["display_name"], "test-session");
            assert!(resp.result["pid"].is_number());
        } else {
            panic!("Expected success response");
        }
    }

    #[test]
    fn query_capabilities_returns_caps() {
        let reg = test_registration();
        let req = Request::new("query.capabilities", json!("req-3"), json!({}));
        let resp = dispatch(&req, &reg).unwrap();

        if let RpcResponse::Success(resp) = resp {
            let caps = resp.result["capabilities"].as_array().unwrap();
            assert_eq!(caps.len(), 3);
            assert!(caps.contains(&json!("inject")));
            assert_eq!(resp.result["roles"], json!(["coder"]));
        } else {
            panic!("Expected success response");
        }
    }

    #[test]
    fn heartbeat_returns_state() {
        let reg = test_registration();
        let req = Request::new("session.heartbeat", json!("req-4"), json!({}));
        let resp = dispatch(&req, &reg).unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["id"], reg.id.as_str());
            assert!(resp.result["heartbeat_at"].is_string());
        } else {
            panic!("Expected success response");
        }
    }

    #[test]
    fn unknown_method_returns_error() {
        let reg = test_registration();
        let req = Request::new("unknown.method", json!("req-5"), json!({}));
        let resp = dispatch(&req, &reg).unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::METHOD_NOT_FOUND);
            assert!(err.error.message.contains("unknown.method"));
        } else {
            panic!("Expected error response");
        }
    }

    #[test]
    fn notification_returns_none() {
        let reg = test_registration();
        let req = Request::notification("event.state_change", json!({"state": "busy"}));
        let resp = dispatch(&req, &reg);
        assert!(resp.is_none());
    }
}
