use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tokio::sync::Mutex;

use termlink_protocol::control::{self, KeyEntry};
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use crate::executor;
use crate::pty::PtySession;
use crate::registration::Registration;
use crate::scrollback::ScrollbackBuffer;

/// Session context passed to handlers, containing registration and optional PTY state.
pub struct SessionContext {
    pub registration: Registration,
    /// Scrollback buffer from PTY session (None for non-PTY sessions).
    pub scrollback: Option<Arc<Mutex<ScrollbackBuffer>>>,
    /// PTY session for input injection (None for non-PTY sessions).
    pub pty: Option<Arc<PtySession>>,
}

impl SessionContext {
    /// Create a context for a non-PTY session.
    pub fn new(registration: Registration) -> Self {
        Self {
            registration,
            scrollback: None,
            pty: None,
        }
    }

    /// Create a context with PTY session state.
    pub fn with_pty(
        registration: Registration,
        pty: Arc<PtySession>,
    ) -> Self {
        let scrollback = Some(pty.scrollback());
        Self {
            registration,
            scrollback,
            pty: Some(pty),
        }
    }
}

impl From<Registration> for SessionContext {
    fn from(reg: Registration) -> Self {
        Self::new(reg)
    }
}

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// Returns `None` for notifications (no response expected).
pub async fn dispatch(req: &Request, ctx: &SessionContext) -> Option<RpcResponse> {
    if req.is_notification() {
        tracing::debug!(method = %req.method, "Received notification");
        return None;
    }

    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let response = match req.method.as_str() {
        "termlink.ping" => handle_ping(id, &ctx.registration),
        control::method::QUERY_STATUS => handle_query_status(id, ctx),
        control::method::QUERY_CAPABILITIES => handle_query_capabilities(id, &ctx.registration),
        control::method::QUERY_OUTPUT => handle_query_output(id, &req.params, ctx).await,
        control::method::SESSION_HEARTBEAT => handle_heartbeat(id, &ctx.registration),
        control::method::COMMAND_EXECUTE => handle_command_execute(id, &req.params).await,
        control::method::COMMAND_INJECT => handle_command_inject(id, &req.params, ctx).await,
        control::method::COMMAND_SIGNAL => {
            handle_command_signal(id, &req.params, &ctx.registration)
        }
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

fn handle_query_status(id: serde_json::Value, ctx: &SessionContext) -> RpcResponse {
    let reg = &ctx.registration;
    let mut result = json!({
        "id": reg.id.as_str(),
        "display_name": reg.display_name,
        "state": reg.state,
        "pid": reg.pid,
        "created_at": reg.created_at,
        "heartbeat_at": reg.heartbeat_at,
        "capabilities": reg.capabilities,
        "metadata": reg.metadata,
    });

    // Add PTY info if available
    result["has_pty"] = json!(ctx.pty.is_some());

    Response::success(id, result).into()
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

/// Handle `query.output` — return scrollback buffer snapshot.
///
/// Params:
///   lines: number (optional) — last N lines (default: 50)
///   bytes: number (optional) — last N bytes (overrides lines if both given)
async fn handle_query_output(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    let scrollback = match &ctx.scrollback {
        Some(sb) => sb,
        None => {
            return ErrorResponse::new(
                id,
                control::error_code::OUTPUT_UNAVAILABLE,
                "No PTY session — output capture not available. Use `register --shell` for PTY-backed sessions.",
            )
            .into();
        }
    };

    let sb = scrollback.lock().await;

    let output = if let Some(bytes) = params.get("bytes").and_then(|b| b.as_u64()) {
        sb.last_n_bytes(bytes as usize)
    } else {
        let lines = params
            .get("lines")
            .and_then(|l| l.as_u64())
            .unwrap_or(50);
        sb.last_n_lines(lines as usize)
    };

    let output_str = String::from_utf8_lossy(&output);

    Response::success(
        id,
        json!({
            "output": output_str,
            "bytes_len": output.len(),
            "total_buffered": sb.len(),
        }),
    )
    .into()
}

/// Handle `command.execute` — spawn a shell command and return output.
async fn handle_command_execute(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let command = match params
        .get("command")
        .or_else(|| params.get("payload").and_then(|p| p.get("command")))
        .and_then(|c| c.as_str())
    {
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

/// Handle `command.inject` — inject keystrokes into the PTY.
///
/// If a PTY session is active, writes resolved bytes directly to the PTY master.
/// Otherwise, resolves keys and reports the result (no injection target).
async fn handle_command_inject(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
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

    let bytes = match executor::resolve_keys(&keys) {
        Ok(b) => b,
        Err(e) => {
            return ErrorResponse::new(
                id,
                control::error_code::INJECTION_FAILED,
                &format!("Key resolution failed: {e}"),
            )
            .into();
        }
    };

    // If PTY session is available, inject directly
    if let Some(pty) = &ctx.pty {
        match pty.write(&bytes).await {
            Ok(()) => {
                return Response::success(
                    id,
                    json!({
                        "status": "injected",
                        "bytes_len": bytes.len(),
                    }),
                )
                .into();
            }
            Err(e) => {
                return ErrorResponse::new(
                    id,
                    control::error_code::INJECTION_FAILED,
                    &format!("PTY write failed: {e}"),
                )
                .into();
            }
        }
    }

    // No PTY — report resolved keys without injection
    Response::success(
        id,
        json!({
            "status": "resolved",
            "bytes_len": bytes.len(),
            "note": "No PTY session. Use `register --shell` for PTY-backed injection.",
        }),
    )
    .into()
}

/// Handle `command.signal` — send a POSIX signal to the session's process.
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

    fn test_ctx() -> SessionContext {
        SessionContext::new(test_registration())
    }

    fn test_ctx_with_scrollback(data: &[u8]) -> SessionContext {
        let reg = test_registration();
        let mut sb = ScrollbackBuffer::new(4096);
        sb.append(data);
        SessionContext {
            registration: reg,
            scrollback: Some(Arc::new(Mutex::new(sb))),
            pty: None,
        }
    }

    #[tokio::test]
    async fn ping_returns_id() {
        let ctx = test_ctx();
        let req = Request::new("termlink.ping", json!("req-1"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.id, json!("req-1"));
            assert_eq!(resp.result["id"], ctx.registration.id.as_str());
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn query_status_returns_state() {
        let ctx = test_ctx();
        let req = Request::new("query.status", json!("req-2"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["state"], "initializing");
            assert_eq!(resp.result["has_pty"], false);
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn query_capabilities_returns_caps() {
        let ctx = test_ctx();
        let req = Request::new("query.capabilities", json!("req-3"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let caps = resp.result["capabilities"].as_array().unwrap();
            assert_eq!(caps.len(), 3);
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn heartbeat_returns_state() {
        let ctx = test_ctx();
        let req = Request::new("session.heartbeat", json!("req-4"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert!(resp.result["heartbeat_at"].is_string());
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let ctx = test_ctx();
        let req = Request::new("unknown.method", json!("req-5"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::METHOD_NOT_FOUND);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn notification_returns_none() {
        let ctx = test_ctx();
        let req = Request::notification("event.state_change", json!({"state": "busy"}));
        let resp = dispatch(&req, &ctx).await;
        assert!(resp.is_none());
    }

    #[tokio::test]
    async fn command_execute_echo() {
        let ctx = test_ctx();
        let req = Request::new(
            "command.execute",
            json!("exec-1"),
            json!({"command": "echo test123"}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["exit_code"], 0);
            assert!(resp.result["stdout"].as_str().unwrap().contains("test123"));
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_execute_missing_command() {
        let ctx = test_ctx();
        let req = Request::new("command.execute", json!("exec-2"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn command_inject_resolves_keys_no_pty() {
        let ctx = test_ctx();
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
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "resolved");
            assert_eq!(resp.result["bytes_len"], 3); // "ls" + 0x0D
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_inject_unknown_key() {
        let ctx = test_ctx();
        let req = Request::new(
            "command.inject",
            json!("inj-2"),
            json!({
                "keys": [
                    {"type": "key", "value": "NonexistentKey"}
                ]
            }),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::INJECTION_FAILED);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn command_signal_to_self() {
        let ctx = test_ctx();
        let req = Request::new(
            "command.signal",
            json!("sig-1"),
            json!({"signal": 0}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "sent");
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_signal_missing_param() {
        let ctx = test_ctx();
        let req = Request::new("command.signal", json!("sig-2"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn query_output_no_pty_returns_error() {
        let ctx = test_ctx();
        let req = Request::new("query.output", json!("out-1"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::OUTPUT_UNAVAILABLE);
        } else {
            panic!("Expected error response for no PTY");
        }
    }

    #[tokio::test]
    async fn query_output_returns_scrollback() {
        let ctx = test_ctx_with_scrollback(b"line1\nline2\nline3\n");
        let req = Request::new("query.output", json!("out-2"), json!({"lines": 2}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let output = resp.result["output"].as_str().unwrap();
            assert!(output.contains("line2"));
            assert!(output.contains("line3"));
            assert!(!output.contains("line1"));
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn query_output_by_bytes() {
        let ctx = test_ctx_with_scrollback(b"abcdefghij");
        let req = Request::new("query.output", json!("out-3"), json!({"bytes": 5}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["output"], "fghij");
            assert_eq!(resp.result["bytes_len"], 5);
        } else {
            panic!("Expected success response");
        }
    }
}
