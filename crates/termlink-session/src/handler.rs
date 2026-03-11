use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tokio::sync::Mutex;

use termlink_protocol::control::{self, KeyEntry};
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use crate::events::EventBus;
use crate::executor;
use crate::pty::PtySession;
use crate::registration::Registration;
use crate::scrollback::ScrollbackBuffer;

/// Session context passed to handlers, containing registration and optional PTY state.
pub struct SessionContext {
    pub registration: Registration,
    /// Path to the on-disk registration JSON file (for persistence after updates).
    pub registration_path: Option<std::path::PathBuf>,
    /// Scrollback buffer from PTY session (None for non-PTY sessions).
    pub scrollback: Option<Arc<Mutex<ScrollbackBuffer>>>,
    /// PTY session for input injection (None for non-PTY sessions).
    pub pty: Option<Arc<PtySession>>,
    /// Event bus for structured cross-session messaging.
    pub events: Arc<Mutex<EventBus>>,
    /// Key-value store for session metadata accessible via RPC.
    pub kv: HashMap<String, serde_json::Value>,
}

impl SessionContext {
    /// Create a context for a non-PTY session.
    pub fn new(registration: Registration) -> Self {
        Self {
            registration,
            registration_path: None,
            scrollback: None,
            pty: None,
            events: Arc::new(Mutex::new(EventBus::new())),
            kv: HashMap::new(),
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
            registration_path: None,
            scrollback,
            pty: Some(pty),
            events: Arc::new(Mutex::new(EventBus::new())),
            kv: HashMap::new(),
        }
    }

    /// Set the path to the on-disk registration JSON file for persistence.
    pub fn with_registration_path(mut self, path: std::path::PathBuf) -> Self {
        self.registration_path = Some(path);
        self
    }
}

impl From<Registration> for SessionContext {
    fn from(reg: Registration) -> Self {
        Self::new(reg)
    }
}

impl From<(Registration, std::path::PathBuf)> for SessionContext {
    fn from((reg, path): (Registration, std::path::PathBuf)) -> Self {
        Self::new(reg).with_registration_path(path)
    }
}

/// Check if a request requires mutable (write) access to session context.
pub fn needs_write(req: &Request) -> bool {
    matches!(
        req.method.as_str(),
        control::method::SESSION_UPDATE | control::method::KV_SET | control::method::KV_DELETE
    )
}

/// Dispatch a mutable request (requires write lock on session context).
pub async fn dispatch_mut(req: &Request, ctx: &mut SessionContext) -> Option<RpcResponse> {
    if req.is_notification() {
        return None;
    }
    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
    let response = match req.method.as_str() {
        control::method::SESSION_UPDATE => handle_session_update(id, &req.params, ctx),
        control::method::KV_SET => handle_kv_set(id, &req.params, ctx),
        control::method::KV_DELETE => handle_kv_delete(id, &req.params, ctx),
        _ => {
            // Fall through to immutable dispatch for anything else
            return dispatch(req, ctx).await;
        }
    };
    Some(response)
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
        control::method::COMMAND_EXECUTE => {
            handle_command_execute(id, &req.params, ctx.registration.allowed_commands.as_deref())
                .await
        }
        control::method::COMMAND_INJECT => handle_command_inject(id, &req.params, ctx).await,
        control::method::COMMAND_SIGNAL => {
            handle_command_signal(id, &req.params, &ctx.registration)
        }
        control::method::COMMAND_RESIZE => {
            handle_command_resize(id, &req.params, ctx)
        }
        control::method::EVENT_EMIT => handle_event_emit(id, &req.params, ctx).await,
        control::method::EVENT_POLL => handle_event_poll(id, &req.params, ctx).await,
        control::method::EVENT_TOPICS => handle_event_topics(id, ctx).await,
        control::method::KV_GET => handle_kv_get(id, &req.params, ctx),
        control::method::KV_LIST => handle_kv_list(id, ctx),
        _ => ErrorResponse::method_not_found(id, &req.method).into(),
    };

    Some(response)
}

/// Handle `session.update` — update session tags, display_name, or roles at runtime.
fn handle_session_update(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &mut SessionContext,
) -> RpcResponse {
    let mut changed = Vec::new();

    if let Some(tags) = params.get("tags").and_then(|t| t.as_array()) {
        ctx.registration.tags = tags
            .iter()
            .filter_map(|t| t.as_str().map(String::from))
            .collect();
        changed.push("tags");
    }

    if let Some(add_tags) = params.get("add_tags").and_then(|t| t.as_array()) {
        for tag in add_tags.iter().filter_map(|t| t.as_str()) {
            if !ctx.registration.tags.contains(&tag.to_string()) {
                ctx.registration.tags.push(tag.to_string());
            }
        }
        if !add_tags.is_empty() {
            changed.push("tags");
        }
    }

    if let Some(remove_tags) = params.get("remove_tags").and_then(|t| t.as_array()) {
        let to_remove: Vec<String> = remove_tags
            .iter()
            .filter_map(|t| t.as_str().map(String::from))
            .collect();
        ctx.registration.tags.retain(|t| !to_remove.contains(t));
        if !to_remove.is_empty() {
            changed.push("tags");
        }
    }

    if let Some(name) = params.get("display_name").and_then(|n| n.as_str()) {
        ctx.registration.display_name = name.to_string();
        changed.push("display_name");
    }

    if let Some(roles) = params.get("roles").and_then(|r| r.as_array()) {
        ctx.registration.roles = roles
            .iter()
            .filter_map(|r| r.as_str().map(String::from))
            .collect();
        changed.push("roles");
    }

    // Persist to disk if path is configured
    if !changed.is_empty() {
        if let Some(ref path) = ctx.registration_path {
            if let Err(e) = ctx.registration.write_atomic(path) {
                tracing::error!(error = %e, "Failed to persist registration after session.update");
                return ErrorResponse::new(
                    id,
                    control::error_code::INJECTION_FAILED,
                    &format!("Update applied in-memory but disk persistence failed: {e}"),
                )
                .into();
            }
        }
    }

    Response::success(
        id,
        json!({
            "updated": changed,
            "tags": ctx.registration.tags,
            "display_name": ctx.registration.display_name,
            "roles": ctx.registration.roles,
        }),
    )
    .into()
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
        "roles": reg.roles,
        "tags": reg.tags,
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
    allowed_commands: Option<&[String]>,
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

    match executor::execute(command, cwd, env.as_ref(), timeout, allowed_commands).await {
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

async fn handle_event_emit(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    let topic = match params
        .get("topic")
        .and_then(|t| t.as_str())
    {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => {
            return ErrorResponse::new(
                id,
                termlink_protocol::jsonrpc::standard_error::INVALID_PARAMS,
                "Missing required param: topic (non-empty string)",
            )
            .into();
        }
    };

    let payload = params
        .get("payload")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let mut bus = ctx.events.lock().await;
    let seq = bus.emit(topic.clone(), payload);

    Response::success(
        id,
        json!({
            "status": "emitted",
            "seq": seq,
            "topic": topic,
        }),
    )
    .into()
}

async fn handle_event_poll(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    // If "since" is not provided, return all events.
    // If provided, return events with seq > since.
    let since_param = params
        .get("since")
        .and_then(|s| s.as_u64());

    let topic_filter = params
        .get("topic")
        .and_then(|t| t.as_str());

    let bus = ctx.events.lock().await;

    // Use a sentinel below any valid seq to mean "all events"
    let since_seq = since_param.unwrap_or(u64::MAX);

    let events: Vec<&crate::events::Event> = if since_seq == u64::MAX {
        // No since param — return everything, optionally filtered by topic
        if let Some(topic) = topic_filter {
            bus.all_by_topic(topic)
        } else {
            bus.all()
        }
    } else if let Some(topic) = topic_filter {
        bus.poll_topic(topic, since_seq)
    } else {
        bus.poll(since_seq)
    };

    let events_json: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            json!({
                "seq": e.seq,
                "topic": e.topic,
                "payload": e.payload,
                "timestamp": e.timestamp,
            })
        })
        .collect();

    Response::success(
        id,
        json!({
            "events": events_json,
            "count": events_json.len(),
            "next_seq": bus.next_seq(),
        }),
    )
    .into()
}

async fn handle_event_topics(
    id: serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    let bus = ctx.events.lock().await;
    let topics = bus.topics();

    Response::success(
        id,
        json!({
            "topics": topics,
            "event_count": bus.len(),
            "next_seq": bus.next_seq(),
        }),
    )
    .into()
}

/// Handle `kv.set` — set a key-value pair in the session's KV store.
fn handle_kv_set(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &mut SessionContext,
) -> RpcResponse {
    let key = match params.get("key").and_then(|k| k.as_str()) {
        Some(k) => k.to_string(),
        None => {
            return ErrorResponse::new(id, -32602, "Missing required field: key").into();
        }
    };

    let value = match params.get("value") {
        Some(v) => v.clone(),
        None => {
            return ErrorResponse::new(id, -32602, "Missing required field: value").into();
        }
    };

    let replaced = ctx.kv.insert(key.clone(), value).is_some();

    Response::success(
        id,
        json!({
            "key": key,
            "replaced": replaced,
        }),
    )
    .into()
}

/// Handle `kv.get` — get a value by key from the session's KV store.
fn handle_kv_get(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    let key = match params.get("key").and_then(|k| k.as_str()) {
        Some(k) => k,
        None => {
            return ErrorResponse::new(id, -32602, "Missing required field: key").into();
        }
    };

    match ctx.kv.get(key) {
        Some(value) => Response::success(
            id,
            json!({
                "key": key,
                "value": value,
                "found": true,
            }),
        )
        .into(),
        None => Response::success(
            id,
            json!({
                "key": key,
                "value": null,
                "found": false,
            }),
        )
        .into(),
    }
}

/// Handle `kv.list` — list all keys in the session's KV store.
fn handle_kv_list(id: serde_json::Value, ctx: &SessionContext) -> RpcResponse {
    let entries: Vec<serde_json::Value> = ctx
        .kv
        .iter()
        .map(|(k, v)| json!({ "key": k, "value": v }))
        .collect();

    Response::success(
        id,
        json!({
            "entries": entries,
            "count": entries.len(),
        }),
    )
    .into()
}

/// Handle `kv.delete` — delete a key from the session's KV store.
fn handle_kv_delete(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &mut SessionContext,
) -> RpcResponse {
    let key = match params.get("key").and_then(|k| k.as_str()) {
        Some(k) => k.to_string(),
        None => {
            return ErrorResponse::new(id, -32602, "Missing required field: key").into();
        }
    };

    let deleted = ctx.kv.remove(&key).is_some();

    Response::success(
        id,
        json!({
            "key": key,
            "deleted": deleted,
        }),
    )
    .into()
}

fn handle_command_resize(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    let cols = params
        .get("cols")
        .or_else(|| params.get("payload").and_then(|p| p.get("cols")))
        .and_then(|v| v.as_u64());
    let rows = params
        .get("rows")
        .or_else(|| params.get("payload").and_then(|p| p.get("rows")))
        .and_then(|v| v.as_u64());

    let (cols, rows) = match (cols, rows) {
        (Some(c), Some(r)) if c > 0 && r > 0 && c <= u16::MAX as u64 && r <= u16::MAX as u64 => {
            (c as u16, r as u16)
        }
        _ => {
            return ErrorResponse::new(
                id,
                termlink_protocol::jsonrpc::standard_error::INVALID_PARAMS,
                "Missing or invalid params: cols and rows (positive integers required)",
            )
            .into();
        }
    };

    let pty = match &ctx.pty {
        Some(pty) => pty,
        None => {
            return ErrorResponse::new(
                id,
                control::error_code::CAPABILITY_NOT_SUPPORTED,
                "No PTY session — resize requires --shell mode",
            )
            .into();
        }
    };

    match pty.resize(cols, rows) {
        Ok(()) => Response::success(
            id,
            json!({
                "status": "resized",
                "cols": cols,
                "rows": rows,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::new(
            id,
            control::error_code::INJECTION_FAILED,
            &format!("Resize failed: {e}"),
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
                tags: vec![],
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
            registration_path: None,
            scrollback: Some(Arc::new(Mutex::new(sb))),
            pty: None,
            events: Arc::new(Mutex::new(EventBus::new())),
            kv: HashMap::new(),
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

    #[tokio::test]
    async fn command_resize_no_pty_returns_error() {
        let ctx = test_ctx();
        let req = Request::new("command.resize", json!("rsz-1"), json!({"cols": 120, "rows": 40}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::CAPABILITY_NOT_SUPPORTED);
        } else {
            panic!("Expected error response for no PTY");
        }
    }

    #[tokio::test]
    async fn command_resize_missing_params() {
        let ctx = test_ctx();
        let req = Request::new("command.resize", json!("rsz-2"), json!({"cols": 120}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error for missing rows");
        }
    }

    #[tokio::test]
    async fn event_emit_and_poll() {
        let ctx = test_ctx();

        // Emit an event
        let req = Request::new("event.emit", json!("ev-1"), json!({
            "topic": "build.start",
            "payload": {"project": "termlink"}
        }));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "emitted");
            assert_eq!(resp.result["seq"], 0);
            assert_eq!(resp.result["topic"], "build.start");
        } else {
            panic!("Expected success for event.emit");
        }

        // Emit another
        let req = Request::new("event.emit", json!("ev-2"), json!({
            "topic": "test.pass",
            "payload": {"name": "unit_test"}
        }));
        dispatch(&req, &ctx).await.unwrap();

        // Poll all events
        let req = Request::new("event.poll", json!("ep-1"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["count"], 2);
            let events = resp.result["events"].as_array().unwrap();
            assert_eq!(events[0]["topic"], "build.start");
            assert_eq!(events[1]["topic"], "test.pass");
            assert_eq!(resp.result["next_seq"], 2);
        } else {
            panic!("Expected success for event.poll");
        }

        // Poll since seq 0 (should only get seq 1)
        let req = Request::new("event.poll", json!("ep-2"), json!({"since": 0}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["count"], 1);
            let events = resp.result["events"].as_array().unwrap();
            assert_eq!(events[0]["topic"], "test.pass");
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn event_emit_missing_topic() {
        let ctx = test_ctx();
        let req = Request::new("event.emit", json!("ev-err"), json!({"payload": {}}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error for missing topic");
        }
    }

    #[tokio::test]
    async fn event_topics_lists_distinct() {
        let ctx = test_ctx();

        // Emit events on different topics
        for topic in &["build.start", "test.pass", "build.done", "test.pass"] {
            let req = Request::new("event.emit", json!("t"), json!({"topic": topic}));
            dispatch(&req, &ctx).await.unwrap();
        }

        let req = Request::new("event.topics", json!("et-1"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let topics = resp.result["topics"].as_array().unwrap();
            let topic_strs: Vec<&str> = topics.iter().filter_map(|t| t.as_str()).collect();
            assert!(topic_strs.contains(&"build.start"));
            assert!(topic_strs.contains(&"build.done"));
            assert!(topic_strs.contains(&"test.pass"));
            assert_eq!(topics.len(), 3); // distinct
            assert_eq!(resp.result["event_count"], 4);
        } else {
            panic!("Expected success for event.topics");
        }
    }

    #[tokio::test]
    async fn event_poll_by_topic() {
        let ctx = test_ctx();

        let req = Request::new("event.emit", json!("t"), json!({"topic": "a"}));
        dispatch(&req, &ctx).await.unwrap();
        let req = Request::new("event.emit", json!("t"), json!({"topic": "b"}));
        dispatch(&req, &ctx).await.unwrap();
        let req = Request::new("event.emit", json!("t"), json!({"topic": "a"}));
        dispatch(&req, &ctx).await.unwrap();

        // Poll only topic "a"
        let req = Request::new("event.poll", json!("tp"), json!({"topic": "a"}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["count"], 2);
            let events = resp.result["events"].as_array().unwrap();
            assert!(events.iter().all(|e| e["topic"] == "a"));
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn session_update_set_tags() {
        let mut ctx = test_ctx();
        assert!(ctx.registration.tags.is_empty());

        let req = Request::new(
            "session.update",
            json!("u-1"),
            json!({"tags": ["web", "prod"]}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let tags = resp.result["tags"].as_array().unwrap();
            assert_eq!(tags.len(), 2);
            assert!(tags.contains(&json!("web")));
            assert!(tags.contains(&json!("prod")));
        } else {
            panic!("Expected success");
        }
        assert_eq!(ctx.registration.tags, vec!["web", "prod"]);
    }

    #[tokio::test]
    async fn session_update_add_remove_tags() {
        let mut ctx = test_ctx();
        ctx.registration.tags = vec!["a".into(), "b".into(), "c".into()];

        let req = Request::new(
            "session.update",
            json!("u-2"),
            json!({"add_tags": ["d"], "remove_tags": ["b"]}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let tags = resp.result["tags"].as_array().unwrap();
            assert_eq!(tags.len(), 3);
            assert!(tags.contains(&json!("a")));
            assert!(tags.contains(&json!("c")));
            assert!(tags.contains(&json!("d")));
            assert!(!tags.contains(&json!("b")));
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn session_update_persists_to_disk() {
        let dir = std::env::temp_dir().join(format!("tl-persist-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let reg = test_registration();
        let json_path = dir.join("test-persist.json");
        reg.write_atomic(&json_path).unwrap();

        let mut ctx = SessionContext::new(reg).with_registration_path(json_path.clone());

        // Update tags via session.update
        let req = Request::new(
            "session.update",
            json!("p-1"),
            json!({"tags": ["persisted", "test"]}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        assert!(matches!(resp, RpcResponse::Success(_)));

        // Read back from disk
        let on_disk = Registration::read_from(&json_path).unwrap();
        assert_eq!(on_disk.tags, vec!["persisted", "test"]);

        // Update display_name
        let req = Request::new(
            "session.update",
            json!("p-2"),
            json!({"display_name": "disk-name"}),
        );
        dispatch_mut(&req, &mut ctx).await.unwrap();

        let on_disk = Registration::read_from(&json_path).unwrap();
        assert_eq!(on_disk.display_name, "disk-name");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn session_update_display_name() {
        let mut ctx = test_ctx();
        let req = Request::new(
            "session.update",
            json!("u-3"),
            json!({"display_name": "new-name"}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["display_name"], "new-name");
        } else {
            panic!("Expected success");
        }
        assert_eq!(ctx.registration.display_name, "new-name");
    }

    #[tokio::test]
    async fn kv_set_get_list_delete() {
        let mut ctx = test_ctx();

        // Set a key
        let req = Request::new("kv.set", json!("kv-1"), json!({"key": "color", "value": "blue"}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["key"], "color");
            assert!(!r.result["replaced"].as_bool().unwrap());
        } else {
            panic!("Expected success for kv.set");
        }

        // Get the key
        let req = Request::new("kv.get", json!("kv-2"), json!({"key": "color"}));
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["value"], "blue");
            assert!(r.result["found"].as_bool().unwrap());
        } else {
            panic!("Expected success for kv.get");
        }

        // Get missing key
        let req = Request::new("kv.get", json!("kv-3"), json!({"key": "missing"}));
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert!(!r.result["found"].as_bool().unwrap());
        } else {
            panic!("Expected success for kv.get (missing)");
        }

        // Set another key and list
        let req = Request::new("kv.set", json!("kv-4"), json!({"key": "size", "value": 42}));
        dispatch_mut(&req, &mut ctx).await.unwrap();

        let req = Request::new("kv.list", json!("kv-5"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 2);
        } else {
            panic!("Expected success for kv.list");
        }

        // Replace existing key
        let req = Request::new("kv.set", json!("kv-6"), json!({"key": "color", "value": "red"}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert!(r.result["replaced"].as_bool().unwrap());
        } else {
            panic!("Expected success for kv.set (replace)");
        }

        // Delete
        let req = Request::new("kv.delete", json!("kv-7"), json!({"key": "color"}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert!(r.result["deleted"].as_bool().unwrap());
        } else {
            panic!("Expected success for kv.delete");
        }

        // Delete non-existent
        let req = Request::new("kv.delete", json!("kv-8"), json!({"key": "color"}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert!(!r.result["deleted"].as_bool().unwrap());
        } else {
            panic!("Expected success for kv.delete (not found)");
        }

        // List should now have 1 entry
        let req = Request::new("kv.list", json!("kv-9"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 1);
        } else {
            panic!("Expected success for kv.list");
        }
    }
}
