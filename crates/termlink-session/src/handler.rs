use std::collections::HashMap;
use std::time::Duration;

use serde_json::json;

use termlink_protocol::control::{self, KeyEntry};
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use crate::executor;
use crate::registration::Registration;

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// Returns `None` for notifications (no response expected).
pub async fn dispatch(req: &Request, registration: &Registration) -> Option<RpcResponse> {
    if req.is_notification() {
        tracing::debug!(method = %req.method, "Received notification");
        return None;
    }

    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let response = match req.method.as_str() {
        "termlink.ping" => handle_ping(id, registration),
        control::method::QUERY_STATUS => handle_query_status(id, registration),
        control::method::QUERY_CAPABILITIES => handle_query_capabilities(id, registration),
        control::method::SESSION_HEARTBEAT => handle_heartbeat(id, registration),
        control::method::COMMAND_EXECUTE => handle_command_execute(id, &req.params).await,
        control::method::COMMAND_INJECT => handle_command_inject(id, &req.params),
        control::method::COMMAND_SIGNAL => handle_command_signal(id, &req.params, registration),
        _ => ErrorResponse::method_not_found(id, &req.method).into(),
    };

    Some(response)
}

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

/// Handle `command.execute` — spawn a shell command and return output.
///
/// Params:
///   command: string (required) — shell command to run
///   cwd: string (optional) — working directory
///   env: object (optional) — additional environment variables
///   timeout: number (optional) — timeout in seconds (default: 30)
async fn handle_command_execute(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let command = match params.get("command").or_else(|| {
        params.get("payload").and_then(|p| p.get("command"))
    }).and_then(|c| c.as_str()) {
        Some(c) => c,
        None => {
            return ErrorResponse::new(
                id,
                termlink_protocol::jsonrpc::standard_error::INVALID_PARAMS,
                "Missing required param: command",
            )
            .into();
        }
    };

    let payload = params.get("payload").unwrap_or(params);
    let cwd = payload.get("cwd").and_then(|c| c.as_str());

    let env: Option<HashMap<String, String>> = payload
        .get("env")
        .and_then(|e| e.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });

    let timeout = payload
        .get("timeout")
        .and_then(|t| t.as_u64())
        .map(Duration::from_secs);

    match executor::execute(command, cwd, env.as_ref(), timeout).await {
        Ok(result) => Response::success(
            id,
            json!({
                "exit_code": result.exit_code,
                "stdout": result.stdout,
                "stderr": result.stderr,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::new(
            id,
            control::error_code::INJECTION_FAILED,
            &format!("Execution failed: {e}"),
        )
        .into(),
    }
}

/// Handle `command.inject` — resolve key entries to bytes.
///
/// Params:
///   keys: array of KeyEntry (required)
///
/// For now, returns the resolved bytes as base64 since we don't have
/// a PTY to inject into yet. The actual PTY injection will come in T-007.
fn handle_command_inject(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let keys_value = params
        .get("keys")
        .or_else(|| params.get("payload").and_then(|p| p.get("keys")));

    let keys: Vec<KeyEntry> = match keys_value {
        Some(k) => match serde_json::from_value(k.clone()) {
            Ok(keys) => keys,
            Err(e) => {
                return ErrorResponse::new(
                    id,
                    termlink_protocol::jsonrpc::standard_error::INVALID_PARAMS,
                    &format!("Invalid keys format: {e}"),
                )
                .into();
            }
        },
        None => {
            return ErrorResponse::new(
                id,
                termlink_protocol::jsonrpc::standard_error::INVALID_PARAMS,
                "Missing required param: keys",
            )
            .into();
        }
    };

    match executor::resolve_keys(&keys) {
        Ok(bytes) => Response::success(
            id,
            json!({
                "status": "resolved",
                "bytes_len": bytes.len(),
                "note": "PTY injection not yet implemented (T-007). Keys resolved successfully.",
            }),
        )
        .into(),
        Err(e) => ErrorResponse::new(
            id,
            control::error_code::INJECTION_FAILED,
            &format!("Key resolution failed: {e}"),
        )
        .into(),
    }
}

/// Handle `command.signal` — send a POSIX signal to the session's process.
///
/// Params:
///   signal: number (required) — signal number (e.g., 2 for SIGINT)
fn handle_command_signal(
    id: serde_json::Value,
    params: &serde_json::Value,
    reg: &Registration,
) -> RpcResponse {
    let signal = match params
        .get("signal")
        .or_else(|| params.get("payload").and_then(|p| p.get("signal")))
        .and_then(|s| s.as_i64())
    {
        Some(s) => s as i32,
        None => {
            return ErrorResponse::new(
                id,
                termlink_protocol::jsonrpc::standard_error::INVALID_PARAMS,
                "Missing required param: signal (number)",
            )
            .into();
        }
    };

    match executor::send_signal(reg.pid, signal) {
        Ok(()) => Response::success(
            id,
            json!({
                "status": "sent",
                "signal": signal,
                "pid": reg.pid,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::new(
            id,
            control::error_code::SIGNAL_FAILED,
            &format!("Signal failed: {e}"),
        )
        .into(),
    }
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

    #[tokio::test]
    async fn ping_returns_id() {
        let reg = test_registration();
        let req = Request::new("termlink.ping", json!("req-1"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.id, json!("req-1"));
            assert_eq!(resp.result["id"], reg.id.as_str());
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn query_status_returns_state() {
        let reg = test_registration();
        let req = Request::new("query.status", json!("req-2"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["state"], "initializing");
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn query_capabilities_returns_caps() {
        let reg = test_registration();
        let req = Request::new("query.capabilities", json!("req-3"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let caps = resp.result["capabilities"].as_array().unwrap();
            assert_eq!(caps.len(), 3);
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn heartbeat_returns_state() {
        let reg = test_registration();
        let req = Request::new("session.heartbeat", json!("req-4"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert!(resp.result["heartbeat_at"].is_string());
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let reg = test_registration();
        let req = Request::new("unknown.method", json!("req-5"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::METHOD_NOT_FOUND);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn notification_returns_none() {
        let reg = test_registration();
        let req = Request::notification("event.state_change", json!({"state": "busy"}));
        let resp = dispatch(&req, &reg).await;
        assert!(resp.is_none());
    }

    #[tokio::test]
    async fn command_execute_echo() {
        let reg = test_registration();
        let req = Request::new(
            "command.execute",
            json!("exec-1"),
            json!({"command": "echo test123"}),
        );
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["exit_code"], 0);
            assert!(resp.result["stdout"].as_str().unwrap().contains("test123"));
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_execute_missing_command() {
        let reg = test_registration();
        let req = Request::new("command.execute", json!("exec-2"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn command_inject_resolves_keys() {
        let reg = test_registration();
        let req = Request::new(
            "command.inject",
            json!("inj-1"),
            json!({
                "keys": [
                    {"type": "text", "value": "ls"},
                    {"type": "key", "value": "Enter"}
                ]
            }),
        );
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "resolved");
            assert_eq!(resp.result["bytes_len"], 3); // "ls" + 0x0D
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_inject_unknown_key() {
        let reg = test_registration();
        let req = Request::new(
            "command.inject",
            json!("inj-2"),
            json!({
                "keys": [
                    {"type": "key", "value": "NonexistentKey"}
                ]
            }),
        );
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::INJECTION_FAILED);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn command_signal_to_self() {
        let reg = test_registration();
        // Signal 0 = check process existence, doesn't actually signal
        let req = Request::new(
            "command.signal",
            json!("sig-1"),
            json!({"signal": 0}),
        );
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "sent");
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_signal_missing_param() {
        let reg = test_registration();
        let req = Request::new("command.signal", json!("sig-2"), json!({}));
        let resp = dispatch(&req, &reg).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error response");
        }
    }
}
