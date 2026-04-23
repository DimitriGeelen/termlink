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
        control::method::KV_SET => handle_kv_set(id, &req.params, ctx).await,
        control::method::KV_DELETE => handle_kv_delete(id, &req.params, ctx).await,
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
        control::method::QUERY_STATUS => handle_query_status(id, ctx).await,
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
        control::method::EVENT_SUBSCRIBE => handle_event_subscribe(id, &req.params, ctx).await,
        control::method::EVENT_TOPICS => handle_event_topics(id, ctx).await,
        control::method::PTY_MODE => handle_pty_mode(id, ctx).await,
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

    if let Some(add_roles) = params.get("add_roles").and_then(|r| r.as_array()) {
        for role in add_roles.iter().filter_map(|r| r.as_str()) {
            if !ctx.registration.roles.contains(&role.to_string()) {
                ctx.registration.roles.push(role.to_string());
            }
        }
        if !add_roles.is_empty() {
            changed.push("roles");
        }
    }

    if let Some(remove_roles) = params.get("remove_roles").and_then(|r| r.as_array()) {
        let to_remove: Vec<String> = remove_roles
            .iter()
            .filter_map(|r| r.as_str().map(String::from))
            .collect();
        ctx.registration.roles.retain(|r| !to_remove.contains(r));
        if !to_remove.is_empty() {
            changed.push("roles");
        }
    }

    // Persist to disk if path is configured
    if !changed.is_empty()
        && let Some(ref path) = ctx.registration_path
            && let Err(e) = ctx.registration.write_atomic(path) {
                tracing::error!(error = %e, "Failed to persist registration after session.update");
                return ErrorResponse::new(
                    id,
                    control::error_code::INJECTION_FAILED,
                    &format!("Update applied in-memory but disk persistence failed: {e}"),
                )
                .into();
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

async fn handle_query_status(id: serde_json::Value, ctx: &SessionContext) -> RpcResponse {
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

    // Add terminal mode if PTY is available
    if let Some(pty) = &ctx.pty
        && let Ok(mode) = pty.terminal_mode().await {
            result["terminal_mode"] = json!({
                "canonical": mode.canonical,
                "echo": mode.echo,
                "raw": mode.raw,
                "alternate_screen": mode.alternate_screen,
            });
        }

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
///   strip_ansi: bool (optional) — strip ANSI escape sequences from output
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

    let strip_ansi = params
        .get("strip_ansi")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let final_output = if strip_ansi {
        strip_ansi_codes(&output_str)
    } else {
        output_str.into_owned()
    };

    Response::success(
        id,
        json!({
            "output": final_output,
            "bytes_len": output.len(),
            "total_buffered": sb.len(),
        }),
    )
    .into()
}

/// Strip ANSI escape sequences and carriage returns from a string.
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch.is_ascii_alphabetic() || ch == 'K' || ch == 'J' || ch == 'H' {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\x07' {
                            break;
                        }
                        if ch == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else if c == '\r' {
            continue;
        } else {
            result.push(c);
        }
    }
    result
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
/// Writes each KeyEntry as a separate PTY write. Non-text entries (special keys
/// like Enter) are preceded by a small delay so that raw-mode TUIs (e.g. ink)
/// see them as individual keypresses rather than pasted text.
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

    // Parse optional inter-key delay (default 10ms, 0 = no delay)
    let delay_ms = params
        .get("inject_delay_ms")
        .or_else(|| params.get("payload").and_then(|p| p.get("inject_delay_ms")))
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    // Resolve each entry individually for separate writes
    let mut resolved: Vec<(Vec<u8>, bool)> = Vec::with_capacity(keys.len());
    let mut total_bytes = 0usize;
    for entry in &keys {
        let bytes = match executor::resolve_key_entry(entry) {
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
        let is_special = !matches!(entry, KeyEntry::Text(_));
        total_bytes += bytes.len();
        resolved.push((bytes, is_special));
    }

    // If PTY session is available, inject with per-entry writes
    if let Some(pty) = &ctx.pty {
        for (i, (bytes, is_special)) in resolved.iter().enumerate() {
            // Delay before special keys (not before the first entry)
            if *is_special && i > 0 && delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            if let Err(e) = pty.write(bytes).await {
                return ErrorResponse::new(
                    id,
                    control::error_code::INJECTION_FAILED,
                    &format!("PTY write failed: {e}"),
                )
                .into();
            }
        }

        // Poll for terminal mode changes after injection
        emit_mode_change_if_needed(ctx).await;

        return Response::success(
            id,
            json!({
                "status": "injected",
                "bytes_len": total_bytes,
            }),
        )
        .into();
    }

    // No PTY — report resolved keys without injection
    Response::success(
        id,
        json!({
            "status": "resolved",
            "bytes_len": total_bytes,
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

    let (events, gap_detected, events_lost) = if since_seq == u64::MAX {
        // No since param — return everything, optionally filtered by topic
        let events = if let Some(topic) = topic_filter {
            bus.all_by_topic(topic)
        } else {
            bus.all()
        };
        (events, false, 0u64)
    } else {
        let result = if let Some(topic) = topic_filter {
            bus.poll_topic(topic, since_seq)
        } else {
            bus.poll(since_seq)
        };
        (result.events, result.gap_detected, result.events_lost)
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

    let mut result = json!({
        "events": events_json,
        "count": events_json.len(),
        "next_seq": bus.next_seq(),
    });

    if gap_detected {
        result["gap_detected"] = json!(true);
        result["events_lost"] = json!(events_lost);
    }

    Response::success(id, result).into()
}

/// Handle `event.subscribe` — long-poll for events using broadcast channel.
///
/// Acquires a broadcast receiver, then waits up to `timeout_ms` (default 5000)
/// for events. Returns as soon as at least one event arrives or the timeout
/// expires. Optional `topic` parameter filters events by topic.
///
/// Optional `since` parameter enables cursor-based replay: historical events
/// with seq > since are included before live events, providing catch-up +
/// live delivery in a single RPC call. Without `since`, only live events
/// are returned.
///
/// This is dramatically lower latency than `event.poll` (which requires a
/// fixed sleep interval on the client side) because the server blocks until
/// an event actually arrives.
async fn handle_event_subscribe(
    id: serde_json::Value,
    params: &serde_json::Value,
    ctx: &SessionContext,
) -> RpcResponse {
    let timeout_ms = params
        .get("timeout_ms")
        .and_then(|t| t.as_u64())
        .unwrap_or(5000);

    let topic_filter = params
        .get("topic")
        .and_then(|t| t.as_str())
        .map(String::from);

    let max_events = params
        .get("max_events")
        .and_then(|m| m.as_u64())
        .unwrap_or(100) as usize;

    let since_param = params
        .get("since")
        .and_then(|s| s.as_u64());

    // Acquire subscriber + optionally replay historical events (single lock).
    let (mut rx, historical, gap_detected, events_lost) = {
        let bus = ctx.events.lock().await;
        let rx = bus.subscribe();

        if let Some(since_seq) = since_param {
            let poll_result = if let Some(ref topic) = topic_filter {
                bus.poll_topic(topic, since_seq)
            } else {
                bus.poll(since_seq)
            };
            let hist: Vec<serde_json::Value> = poll_result
                .events
                .into_iter()
                .take(max_events)
                .map(|e| {
                    json!({
                        "seq": e.seq,
                        "topic": e.topic,
                        "payload": e.payload,
                        "timestamp": e.timestamp,
                    })
                })
                .collect();
            (rx, hist, poll_result.gap_detected, poll_result.events_lost)
        } else {
            (rx, Vec::new(), false, 0u64)
        }
    };

    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    let mut collected: Vec<serde_json::Value> = historical;
    let mut lagged: u64 = 0;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() || collected.len() >= max_events {
            break;
        }

        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        // Apply topic filter
                        if topic_filter.as_ref().is_some_and(|t| event.topic != *t) {
                            continue;
                        }
                        collected.push(json!({
                            "seq": event.seq,
                            "topic": event.topic,
                            "payload": event.payload,
                            "timestamp": event.timestamp,
                        }));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        lagged += n;
                        // Continue receiving — some events were dropped
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break; // EventBus was dropped
                    }
                }
            }
            _ = tokio::time::sleep(remaining) => {
                break; // Timeout
            }
        }
    }

    // Compute next_seq from the highest seq in collected events
    let next_seq = collected
        .iter()
        .filter_map(|e| e["seq"].as_u64())
        .max()
        .map(|s| s + 1);

    let mut result = json!({
        "events": collected,
        "count": collected.len(),
    });

    if let Some(ns) = next_seq {
        result["next_seq"] = json!(ns);
    }

    if lagged > 0 {
        result["lagged"] = json!(lagged);
    }

    if gap_detected {
        result["gap_detected"] = json!(true);
        result["events_lost"] = json!(events_lost);
    }

    Response::success(id, result).into()
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
async fn handle_kv_set(
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

    let replaced = ctx.kv.insert(key.clone(), value.clone()).is_some();

    {
        let mut bus = ctx.events.lock().await;
        bus.emit(
            "kv.change",
            json!({
                "key": key,
                "value": value,
                "op": "set",
                "replaced": replaced,
            }),
        );
    }

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
async fn handle_kv_delete(
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

    {
        let mut bus = ctx.events.lock().await;
        bus.emit(
            "kv.change",
            json!({
                "key": key,
                "value": serde_json::Value::Null,
                "op": "delete",
                "deleted": deleted,
            }),
        );
    }

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

/// Handle `pty.mode` — return the current terminal mode flags.
///
/// Queries tcgetattr on the PTY master fd to determine canonical/echo/raw state,
/// and includes alternate screen buffer tracking.
async fn handle_pty_mode(id: serde_json::Value, ctx: &SessionContext) -> RpcResponse {
    let pty = match &ctx.pty {
        Some(pty) => pty,
        None => {
            return ErrorResponse::new(
                id,
                control::error_code::CAPABILITY_NOT_SUPPORTED,
                "No PTY session — terminal mode detection not available. Use `register --shell` for PTY-backed sessions.",
            )
            .into();
        }
    };

    match pty.terminal_mode().await {
        Ok(mode) => Response::success(
            id,
            json!({
                "canonical": mode.canonical,
                "echo": mode.echo,
                "raw": mode.raw,
                "alternate_screen": mode.alternate_screen,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::new(
            id,
            control::error_code::CAPABILITY_NOT_SUPPORTED,
            &format!("Failed to read terminal mode: {e}"),
        )
        .into(),
    }
}

/// Emit a `pty.mode-change` event if the terminal mode has changed.
///
/// Called after inject operations to detect mode transitions.
async fn emit_mode_change_if_needed(ctx: &SessionContext) {
    let pty = match &ctx.pty {
        Some(pty) => pty,
        None => return,
    };

    match pty.poll_mode_change().await {
        Ok(Some((current, previous, password_hint))) => {
            let mut payload = json!({
                "canonical": current.canonical,
                "echo": current.echo,
                "raw": current.raw,
                "alternate_screen": current.alternate_screen,
            });
            if let Some(prev) = previous {
                payload["previous"] = json!({
                    "canonical": prev.canonical,
                    "echo": prev.echo,
                    "raw": prev.raw,
                    "alternate_screen": prev.alternate_screen,
                });
            }
            if password_hint {
                payload["password_prompt_hint"] = json!(true);
            }
            let mut bus = ctx.events.lock().await;
            bus.emit("pty.mode-change", payload);
        }
        Ok(None) => {} // No change
        Err(e) => {
            tracing::debug!(error = %e, "Failed to poll terminal mode change");
        }
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
    async fn command_inject_multi_entry_resolves_separately() {
        let ctx = test_ctx();
        // Text + special key should resolve to correct total bytes
        let req = Request::new(
            "command.inject",
            json!("inj-multi"),
            json!({
                "keys": [
                    {"type": "text", "value": "hello"},
                    {"type": "key", "value": "Enter"}
                ],
                "inject_delay_ms": 0
            }),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "resolved");
            // "hello" (5 bytes) + Enter (1 byte) = 6
            assert_eq!(resp.result["bytes_len"], 6);
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn command_inject_custom_delay() {
        let ctx = test_ctx();
        let req = Request::new(
            "command.inject",
            json!("inj-delay"),
            json!({
                "keys": [{"type": "text", "value": "x"}],
                "inject_delay_ms": 50
            }),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["status"], "resolved");
            assert_eq!(resp.result["bytes_len"], 1);
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
    async fn query_output_strip_ansi() {
        let ctx = test_ctx_with_scrollback(b"\x1b[32mgreen\x1b[0m text\r\n");
        let req = Request::new("query.output", json!("out-strip-1"), json!({"lines": 10, "strip_ansi": true}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert_eq!(resp.result["output"], "green text\n");
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn query_output_strip_ansi_false_preserves() {
        let input = b"\x1b[32mgreen\x1b[0m";
        let ctx = test_ctx_with_scrollback(input);
        let req = Request::new("query.output", json!("out-strip-2"), json!({"lines": 10, "strip_ansi": false}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            let output = resp.result["output"].as_str().unwrap();
            assert!(output.contains("\x1b[32m"));
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

    #[tokio::test]
    async fn event_poll_gap_detection() {
        // Use a small-capacity event bus to trigger overflow
        let reg = test_registration();
        let ctx = SessionContext {
            registration: reg,
            registration_path: None,
            scrollback: None,
            pty: None,
            events: Arc::new(Mutex::new(crate::events::EventBus::with_capacity(3))),
            kv: HashMap::new(),
        };

        // Emit 5 events (buffer capacity 3 → seqs 0,1 evicted, buffer holds 2,3,4)
        for i in 0..5 {
            let req = Request::new(
                "event.emit",
                json!("ge"),
                json!({"topic": format!("e{i}"), "payload": {}}),
            );
            dispatch(&req, &ctx).await.unwrap();
        }

        // Poll with since=0 → should detect gap (events at seq 1 lost)
        let req = Request::new("event.poll", json!("gp-1"), json!({"since": 0}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert!(resp.result["gap_detected"].as_bool().unwrap());
            assert_eq!(resp.result["events_lost"], 1);
            assert_eq!(resp.result["count"], 3);
        } else {
            panic!("Expected success for gap poll");
        }

        // Poll with since=1 → no gap (oldest is 2, since+1=2)
        let req = Request::new("event.poll", json!("gp-2"), json!({"since": 1}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert!(resp.result.get("gap_detected").is_none());
            assert_eq!(resp.result["count"], 3);
        } else {
            panic!("Expected success");
        }

        // Poll without since → no gap info (returns all, no cursor check)
        let req = Request::new("event.poll", json!("gp-3"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();

        if let RpcResponse::Success(resp) = resp {
            assert!(resp.result.get("gap_detected").is_none());
            assert_eq!(resp.result["count"], 3);
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn concurrent_pollers_see_all_events() {
        let reg = test_registration();
        let ctx = SessionContext::new(reg);
        let events = ctx.events.clone();

        let num_pollers = 4;
        let num_events: usize = 50;

        // Emit all events first
        {
            let mut bus = events.lock().await;
            for i in 0..num_events {
                bus.emit("test.event", json!({"index": i}));
            }
        }

        // Now spawn concurrent pollers — each independently polls the same bus
        let mut poller_handles = Vec::new();

        for poller_id in 0..num_pollers {
            let events_clone = events.clone();
            poller_handles.push(tokio::spawn(async move {
                let mut cursor: u64 = 0;
                let mut seen = Vec::new();
                let mut iterations = 0;

                // Use a cursor-based approach: poll in batches until caught up
                // Start with cursor meaning "give me everything > cursor"
                // We use a special first poll to get seq 0
                let bus = events_clone.lock().await;
                let all = bus.all();
                for event in &all {
                    seen.push(event.seq);
                }
                if let Some(last) = all.last() {
                    cursor = last.seq;
                }
                drop(bus);

                // Do additional polls to verify no events missed
                while iterations < 10 {
                    iterations += 1;
                    let bus = events_clone.lock().await;
                    let result = bus.poll(cursor);
                    assert!(
                        !result.gap_detected,
                        "Poller {poller_id} detected gap at cursor {cursor}"
                    );
                    for event in &result.events {
                        seen.push(event.seq);
                        cursor = event.seq;
                    }
                    drop(bus);
                    tokio::task::yield_now().await;
                }

                seen
            }));
        }

        // Wait for pollers and verify all saw all events
        for handle in poller_handles {
            let seen = handle.await.unwrap();
            assert_eq!(
                seen.len(),
                num_events,
                "Poller missed events: saw {} of {}",
                seen.len(),
                num_events
            );
            // Verify no duplicates in a single poller's view
            let mut sorted = seen.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), seen.len(), "Poller saw duplicate events");
            // Verify all seq numbers present
            for i in 0..num_events as u64 {
                assert!(sorted.contains(&i), "Poller missing seq {i}");
            }
        }
    }

    // --- strip_ansi_codes tests ---

    #[test]
    fn strip_ansi_plain_text_passthrough() {
        assert_eq!(strip_ansi_codes("hello world"), "hello world");
        assert_eq!(strip_ansi_codes(""), "");
        assert_eq!(strip_ansi_codes("line1\nline2\n"), "line1\nline2\n");
    }

    #[test]
    fn strip_ansi_csi_color_codes() {
        // SGR (Select Graphic Rendition) — color codes
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi_codes("\x1b[1;32mbold green\x1b[0m"), "bold green");
        assert_eq!(
            strip_ansi_codes("\x1b[38;5;196mextended\x1b[0m"),
            "extended"
        );
    }

    #[test]
    fn strip_ansi_csi_cursor_movement() {
        // Cursor up, down, forward, back, erase
        assert_eq!(strip_ansi_codes("\x1b[2Aup two"), "up two");
        assert_eq!(strip_ansi_codes("\x1b[10Bdown ten"), "down ten");
        assert_eq!(strip_ansi_codes("before\x1b[Kafter"), "beforeafter");
        assert_eq!(strip_ansi_codes("before\x1b[2Jafter"), "beforeafter");
        assert_eq!(strip_ansi_codes("\x1b[5;10Hpositioned"), "positioned");
    }

    #[test]
    fn strip_ansi_osc_title_setting() {
        // OSC with BEL terminator
        assert_eq!(
            strip_ansi_codes("\x1b]0;My Terminal Title\x07rest"),
            "rest"
        );
        // OSC with ST (ESC \) terminator
        assert_eq!(
            strip_ansi_codes("\x1b]0;Title\x1b\\rest"),
            "rest"
        );
    }

    #[test]
    fn strip_ansi_carriage_return() {
        assert_eq!(strip_ansi_codes("line\r\n"), "line\n");
        assert_eq!(strip_ansi_codes("overwrite\rvisible"), "overwritevisible");
        assert_eq!(strip_ansi_codes("\r"), "");
    }

    #[test]
    fn strip_ansi_mixed_content() {
        let input = "\x1b[1;34m$ \x1b[0mecho \x1b[32m\"hello\"\x1b[0m\r\nhello\r\n";
        let expected = "$ echo \"hello\"\nhello\n";
        assert_eq!(strip_ansi_codes(input), expected);
    }

    #[test]
    fn strip_ansi_bare_escape_consumed() {
        // A bare ESC followed by something other than [ or ] should consume ESC + next char
        assert_eq!(strip_ansi_codes("\x1bXrest"), "rest");
    }

    // --- needs_write tests ---

    #[test]
    fn needs_write_identifies_mutable_methods() {
        let mutable = [
            control::method::SESSION_UPDATE,
            control::method::KV_SET,
            control::method::KV_DELETE,
        ];
        for method in &mutable {
            let req = Request::new(method, json!(1), json!({}));
            assert!(needs_write(&req), "{method} should require write lock");
        }
    }

    #[test]
    fn needs_write_rejects_read_methods() {
        let read_only = [
            "termlink.ping",
            control::method::QUERY_STATUS,
            control::method::QUERY_CAPABILITIES,
            control::method::QUERY_OUTPUT,
            control::method::COMMAND_EXECUTE,
            control::method::EVENT_EMIT,
            control::method::EVENT_POLL,
            control::method::KV_GET,
            control::method::KV_LIST,
        ];
        for method in &read_only {
            let req = Request::new(method, json!(1), json!({}));
            assert!(!needs_write(&req), "{method} should NOT require write lock");
        }
    }

    // --- KV error case tests ---

    #[tokio::test]
    async fn kv_set_missing_key_returns_error() {
        let mut ctx = test_ctx();
        let req = Request::new("kv.set", json!("e-1"), json!({"value": "no-key"}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error for kv.set without key");
        }
    }

    #[tokio::test]
    async fn kv_set_missing_value_returns_error() {
        let mut ctx = test_ctx();
        let req = Request::new("kv.set", json!("e-2"), json!({"key": "no-value"}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error for kv.set without value");
        }
    }

    #[tokio::test]
    async fn kv_get_missing_key_returns_error() {
        let ctx = test_ctx();
        let req = Request::new("kv.get", json!("e-3"), json!({}));
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error for kv.get without key");
        }
    }

    #[tokio::test]
    async fn kv_delete_missing_key_returns_error() {
        let mut ctx = test_ctx();
        let req = Request::new("kv.delete", json!("e-4"), json!({}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, standard_error::INVALID_PARAMS);
        } else {
            panic!("Expected error for kv.delete without key");
        }
    }

    // --- session.update roles test ---

    #[tokio::test]
    async fn session_update_roles() {
        let mut ctx = test_ctx();
        assert_eq!(ctx.registration.roles, vec!["coder"]);

        let req = Request::new(
            "session.update",
            json!("upd-r"),
            json!({"roles": ["reviewer", "deployer"]}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            let updated = r.result["updated"].as_array().unwrap();
            assert!(updated.iter().any(|c| c == "roles"));
            let roles = r.result["roles"].as_array().unwrap();
            assert_eq!(roles.len(), 2);
        } else {
            panic!("Expected success for session.update");
        }
        assert_eq!(ctx.registration.roles, vec!["reviewer", "deployer"]);
    }

    #[tokio::test]
    async fn session_update_add_roles() {
        let mut ctx = test_ctx();
        assert_eq!(ctx.registration.roles, vec!["coder"]);

        let req = Request::new(
            "session.update",
            json!("upd-ar"),
            json!({"add_roles": ["reviewer"]}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert!(r.result["updated"].as_array().unwrap().iter().any(|c| c == "roles"));
        } else {
            panic!("Expected success");
        }
        assert_eq!(ctx.registration.roles, vec!["coder", "reviewer"]);
    }

    #[tokio::test]
    async fn session_update_remove_roles() {
        let mut ctx = test_ctx();
        ctx.registration.roles = vec!["coder".into(), "reviewer".into(), "deployer".into()];

        let req = Request::new(
            "session.update",
            json!("upd-rr"),
            json!({"remove_roles": ["reviewer"]}),
        );
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert!(r.result["updated"].as_array().unwrap().iter().any(|c| c == "roles"));
        } else {
            panic!("Expected success");
        }
        assert_eq!(ctx.registration.roles, vec!["coder", "deployer"]);
    }

    #[tokio::test]
    async fn session_update_add_roles_dedup() {
        let mut ctx = test_ctx();
        assert_eq!(ctx.registration.roles, vec!["coder"]);

        let req = Request::new(
            "session.update",
            json!("upd-ard"),
            json!({"add_roles": ["coder", "reviewer"]}),
        );
        dispatch_mut(&req, &mut ctx).await.unwrap();
        // "coder" should not be duplicated
        assert_eq!(ctx.registration.roles, vec!["coder", "reviewer"]);
    }

    // --- dispatch_mut fallthrough and notification tests ---

    #[tokio::test]
    async fn dispatch_mut_falls_through_to_immutable() {
        let mut ctx = test_ctx();
        // termlink.ping is a read-only method — dispatch_mut should fall through to dispatch
        let req = Request::new("termlink.ping", json!("ft-1"), json!({}));
        let resp = dispatch_mut(&req, &mut ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["id"], ctx.registration.id.as_str());
        } else {
            panic!("Expected success from fallthrough dispatch");
        }
    }

    #[tokio::test]
    async fn dispatch_mut_notification_returns_none() {
        let mut ctx = test_ctx();
        let req = Request::notification("session.update", json!({"tags": ["ignored"]}));
        let resp = dispatch_mut(&req, &mut ctx).await;
        assert!(resp.is_none());
    }

    #[tokio::test]
    async fn event_subscribe_receives_events() {
        let ctx = test_ctx();

        // Emit an event in the background after a brief delay
        let events = ctx.events.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut bus = events.lock().await;
            bus.emit("test.event", json!({"data": "hello"}));
        });

        let req = Request::new(
            "event.subscribe",
            json!("sub-1"),
            json!({"timeout_ms": 2000}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            let count = r.result["count"].as_u64().unwrap();
            assert_eq!(count, 1);
            let events = r.result["events"].as_array().unwrap();
            assert_eq!(events[0]["topic"], "test.event");
            assert_eq!(events[0]["payload"]["data"], "hello");
            // Verify next_seq is present for cursor-based following
            assert!(r.result["next_seq"].is_u64(), "next_seq should be present when events are returned");
        } else {
            panic!("Expected success from event.subscribe");
        }
    }

    #[tokio::test]
    async fn event_subscribe_timeout_returns_empty() {
        let ctx = test_ctx();

        let req = Request::new(
            "event.subscribe",
            json!("sub-2"),
            json!({"timeout_ms": 100}),
        );
        let start = tokio::time::Instant::now();
        let resp = dispatch(&req, &ctx).await.unwrap();
        let elapsed = start.elapsed();

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 0);
            assert!(r.result["events"].as_array().unwrap().is_empty());
            // next_seq should be absent when no events were received
            assert!(r.result["next_seq"].is_null(), "next_seq should be absent when no events");
        } else {
            panic!("Expected success from event.subscribe");
        }

        // Should have waited approximately the timeout duration
        assert!(elapsed >= Duration::from_millis(90));
        assert!(elapsed < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn event_subscribe_topic_filter() {
        let ctx = test_ctx();

        let events = ctx.events.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            let mut bus = events.lock().await;
            bus.emit("noise", json!({}));
            bus.emit("target.topic", json!({"found": true}));
            bus.emit("more.noise", json!({}));
        });

        let req = Request::new(
            "event.subscribe",
            json!("sub-3"),
            json!({"timeout_ms": 2000, "topic": "target.topic"}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 1);
            let events = r.result["events"].as_array().unwrap();
            assert_eq!(events[0]["topic"], "target.topic");
            assert_eq!(events[0]["payload"]["found"], true);
        } else {
            panic!("Expected success from event.subscribe");
        }
    }

    #[tokio::test]
    async fn event_subscribe_since_replays_history() {
        let ctx = test_ctx();

        // Pre-populate events in the buffer
        {
            let mut bus = ctx.events.lock().await;
            bus.emit("evt.a", json!({"n": 1})); // seq 0
            bus.emit("evt.b", json!({"n": 2})); // seq 1
            bus.emit("evt.c", json!({"n": 3})); // seq 2
        }

        // Subscribe with since=0 should replay events with seq > 0 (i.e., seq 1 and 2)
        let req = Request::new(
            "event.subscribe",
            json!("sub-since-1"),
            json!({"timeout_ms": 100, "since": 0}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            let events = r.result["events"].as_array().unwrap();
            assert_eq!(events.len(), 2, "should replay 2 events after seq 0");
            assert_eq!(events[0]["topic"], "evt.b");
            assert_eq!(events[1]["topic"], "evt.c");
            assert!(r.result["next_seq"].is_u64(), "next_seq should be present");
            assert_eq!(r.result["next_seq"], 3);
            // No gap expected
            assert!(r.result["gap_detected"].is_null());
        } else {
            panic!("Expected success from event.subscribe with since");
        }
    }

    #[tokio::test]
    async fn event_subscribe_since_with_topic_filter() {
        let ctx = test_ctx();

        {
            let mut bus = ctx.events.lock().await;
            bus.emit("noise", json!({}));       // seq 0
            bus.emit("target", json!({"a": 1})); // seq 1
            bus.emit("noise", json!({}));        // seq 2
            bus.emit("target", json!({"a": 2})); // seq 3
        }

        // since=0 + topic filter should only return "target" events
        let req = Request::new(
            "event.subscribe",
            json!("sub-since-topic"),
            json!({"timeout_ms": 100, "since": 0, "topic": "target"}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            let events = r.result["events"].as_array().unwrap();
            assert_eq!(events.len(), 2, "should only have 'target' events");
            assert_eq!(events[0]["payload"]["a"], 1);
            assert_eq!(events[1]["payload"]["a"], 2);
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn event_subscribe_since_no_matching_events() {
        let ctx = test_ctx();

        {
            let mut bus = ctx.events.lock().await;
            bus.emit("evt", json!({})); // seq 0
        }

        // since=0 means events with seq > 0 — there are none
        let req = Request::new(
            "event.subscribe",
            json!("sub-since-none"),
            json!({"timeout_ms": 100, "since": 0}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            // Only seq 0 exists, since > 0 returns nothing from history
            // and timeout expires with no live events
            assert_eq!(r.result["count"], 0);
        } else {
            panic!("Expected success");
        }
    }

    #[tokio::test]
    async fn event_subscribe_without_since_skips_history() {
        let ctx = test_ctx();

        // Pre-populate events
        {
            let mut bus = ctx.events.lock().await;
            bus.emit("old.event", json!({}));
            bus.emit("old.event", json!({}));
        }

        // Without since, should NOT return historical events
        let req = Request::new(
            "event.subscribe",
            json!("sub-no-since"),
            json!({"timeout_ms": 100}),
        );
        let resp = dispatch(&req, &ctx).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 0, "without since, historical events should not be included");
        } else {
            panic!("Expected success");
        }
    }

}
