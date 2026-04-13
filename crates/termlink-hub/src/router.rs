use std::sync::OnceLock;
use std::time::Duration;

use serde_json::json;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};
use termlink_protocol::TransportAddr;

use termlink_session::client;
use termlink_session::manager;

use crate::aggregator::{EventAggregator, SessionTarget};
use crate::remote_store::RemoteStore;

/// Per-target timeout for broadcast/collect operations.
const PER_TARGET_TIMEOUT: Duration = Duration::from_secs(5);

/// Global remote session store (initialized once by the hub server).
static REMOTE_STORE: OnceLock<RemoteStore> = OnceLock::new();

/// Global event aggregator (T-966).
static AGGREGATOR: OnceLock<EventAggregator> = OnceLock::new();

/// Initialize the global remote store. Called once by the hub server.
pub fn init_remote_store() -> RemoteStore {
    let store = RemoteStore::new();
    let _ = REMOTE_STORE.set(store.clone());
    store
}

/// Initialize the global event aggregator. Called once by the hub server.
pub fn init_aggregator() {
    let _ = AGGREGATOR.set(EventAggregator::new(4096));
}

/// Get the global event aggregator.
pub(crate) fn aggregator() -> Option<&'static EventAggregator> {
    AGGREGATOR.get()
}

/// Get the global remote store (returns None if not initialized).
pub(crate) fn remote_store() -> Option<&'static RemoteStore> {
    REMOTE_STORE.get()
}

/// Route a JSON-RPC request to the appropriate handler.
///
/// Hub-local methods (session.discover) are handled directly.
/// All other methods are forwarded to the target session specified in params.target.
pub async fn route(req: &Request) -> Option<RpcResponse> {
    if req.is_notification() {
        tracing::debug!(method = %req.method, "Hub received notification (ignoring)");
        return None;
    }

    let id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let response = match req.method.as_str() {
        control::method::SESSION_DISCOVER => handle_discover(id, &req.params).await,
        control::method::EVENT_BROADCAST => handle_event_broadcast(id, &req.params).await,
        control::method::EVENT_COLLECT => handle_event_collect(id, &req.params).await,
        control::method::EVENT_SUBSCRIBE if is_hub_level(&req.params) => {
            handle_hub_subscribe(id, &req.params).await
        }
        control::method::EVENT_EMIT_TO => handle_event_emit_to(id, &req.params).await,
        control::method::ORCHESTRATOR_ROUTE => handle_orchestrator_route(id, &req.params).await,
        control::method::ORCHESTRATOR_BYPASS_STATUS => handle_bypass_status(id),
        control::method::ORCHESTRATOR_BYPASS_INVALIDATE => handle_bypass_invalidate(id, &req.params),
        "session.register_remote" => handle_register_remote(id, &req.params),
        "session.heartbeat" => handle_heartbeat(id, &req.params),
        "session.deregister_remote" => handle_deregister_remote(id, &req.params),
        "inbox.list" => handle_inbox_list(id, &req.params),
        "inbox.status" => handle_inbox_status(id),
        "inbox.clear" => handle_inbox_clear(id, &req.params),
        _ => forward_to_target(req, id).await,
    };

    Some(response)
}

/// Handle `session.discover` — list/filter registered sessions.
///
/// Optional params: { tags?: [string], roles?: [string], capabilities?: [string], name?: string }
/// All filters use AND logic. Omitted filters match everything.
async fn handle_discover(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    match manager::list_sessions(false) {
        Ok(sessions) => {
            let tag_filter: Vec<String> = params
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let role_filter: Vec<String> = params
                .get("roles")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let cap_filter: Vec<String> = params
                .get("capabilities")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            let name_filter = params.get("name").and_then(|n| n.as_str());

            let mut entries: Vec<serde_json::Value> = sessions
                .iter()
                .filter(|s| {
                    tag_filter.iter().all(|t| s.tags.contains(t))
                        && role_filter.iter().all(|r| s.roles.contains(r))
                        && cap_filter.iter().all(|c| s.capabilities.contains(c))
                        && name_filter.is_none_or(|n| {
                            s.display_name.to_lowercase().contains(&n.to_lowercase())
                        })
                })
                .map(|s| {
                    json!({
                        "id": s.id.as_str(),
                        "display_name": s.display_name,
                        "state": s.state,
                        "capabilities": s.capabilities,
                        "roles": s.roles,
                        "tags": s.tags,
                        "pid": s.pid,
                    })
                })
                .collect();

            // Include remote (TCP) sessions from the in-memory store
            if let Some(store) = remote_store() {
                let remote_entries: Vec<serde_json::Value> = store
                    .list_live()
                    .iter()
                    .filter(|e| {
                        tag_filter.iter().all(|t| e.tags.contains(t))
                            && role_filter.iter().all(|r| e.roles.contains(r))
                            && cap_filter.iter().all(|c| e.capabilities.contains(c))
                            && name_filter.is_none_or(|n| {
                                e.display_name.to_lowercase().contains(&n.to_lowercase())
                            })
                    })
                    .map(|e| e.to_json())
                    .collect();
                entries.extend(remote_entries);
            }

            Response::success(id, json!({ "sessions": entries })).into()
        }
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("Discovery failed: {e}")).into()
        }
    }
}

/// Handle `event.broadcast` — emit an event to multiple sessions (fan-out).
///
/// Params: { topic, payload, targets?: [string] }
/// If targets is omitted, broadcasts to all live sessions.
async fn handle_event_broadcast(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let topic = match params.get("topic").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                -32602,
                "Missing 'topic' in params",
            )
            .into();
        }
    };

    let payload = params
        .get("payload")
        .cloned()
        .unwrap_or(json!({}));

    // Resolve target sessions
    let registrations = if let Some(targets) = params.get("targets").and_then(|t| t.as_array()) {
        let mut regs = Vec::new();
        for t in targets {
            if let Some(name) = t.as_str() {
                match manager::find_session(name) {
                    Ok(r) => regs.push(r),
                    Err(_) => {
                        tracing::warn!(target = name, "Broadcast: target not found, skipping");
                    }
                }
            }
        }
        regs
    } else {
        // All live sessions
        match manager::list_sessions(false) {
            Ok(sessions) => sessions
                .iter()
                .filter_map(|s| manager::find_session(s.id.as_str()).ok())
                .collect(),
            Err(e) => {
                return ErrorResponse::internal_error(
                    id,
                    &format!("Failed to list sessions: {e}"),
                )
                .into();
            }
        }
    };

    let targeted = registrations.len();
    let topic_owned = topic.to_string();

    // Dispatch to all targets concurrently with per-target timeout
    let mut join_set = tokio::task::JoinSet::new();

    for reg in registrations {
        let emit_params = json!({
            "topic": topic_owned,
            "payload": payload,
        });
        let addr = reg.addr.to_transport_addr();

        join_set.spawn(async move {
            let result = tokio::time::timeout(
                PER_TARGET_TIMEOUT,
                client::rpc_call_addr(&addr, control::method::EVENT_EMIT, emit_params),
            )
            .await;

            match result {
                Ok(Ok(resp)) => client::unwrap_result(resp).is_ok(),
                Ok(Err(_)) => false,   // RPC error
                Err(_) => false,        // Timeout
            }
        });
    }

    let mut succeeded = 0u64;
    let mut failed = 0u64;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(true) => succeeded += 1,
            _ => failed += 1,
        }
    }

    Response::success(
        id,
        json!({
            "topic": topic_owned,
            "targeted": targeted,
            "succeeded": succeeded,
            "failed": failed,
        }),
    )
    .into()
}

/// Handle `event.emit_to` — push an event directly to a target session's event bus.
///
/// Params: { target: string, topic: string, payload?: value, from?: string }
/// The hub resolves the target session, enriches the payload with sender info,
/// and forwards an `event.emit` RPC to the target's socket. This is a unicast
/// push — the sender does not need to know the target's socket path.
async fn handle_event_emit_to(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let target = match params.get("target").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                -32602,
                "Missing 'target' in params",
            )
            .into();
        }
    };

    let topic = match params.get("topic").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                -32602,
                "Missing 'topic' in params",
            )
            .into();
        }
    };

    let payload = params
        .get("payload")
        .cloned()
        .unwrap_or(json!({}));

    let from = params.get("from").and_then(|f| f.as_str());

    // Resolve target session (local first, then remote)
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(_) => {
            // Check remote store
            if let Some(store) = remote_store()
                && let Some(_remote) = store.get(target) {
                    return ErrorResponse::new(
                        id,
                        control::error_code::CAPABILITY_NOT_SUPPORTED,
                        "emit_to for remote (TCP) sessions is not yet supported",
                    )
                    .into();
                }

            // T-988: For file events, spool to inbox instead of erroring
            if let Ok(true) = crate::inbox::deposit(target, topic, &payload, from) {
                return Response::success(id, json!({
                    "ok": true,
                    "spooled": true,
                    "target": target,
                    "message": format!("Target '{}' offline — file event spooled to inbox", target),
                }))
                .into();
            }

            return ErrorResponse::new(
                id,
                control::error_code::SESSION_NOT_FOUND,
                &format!("Target session '{}' not found", target),
            )
            .into();
        }
    };

    // Enrich payload with sender info for traceability
    let enriched_payload = if let Some(sender) = from {
        let mut p = payload.clone();
        if let Some(obj) = p.as_object_mut() {
            obj.insert("_from".to_string(), json!(sender));
        } else {
            p = json!({ "_data": payload, "_from": sender });
        }
        p
    } else {
        payload
    };

    let emit_params = json!({
        "topic": topic,
        "payload": enriched_payload,
    });

    let addr = reg.addr.to_transport_addr();
    let result = tokio::time::timeout(
        PER_TARGET_TIMEOUT,
        client::rpc_call_addr(&addr, control::method::EVENT_EMIT, emit_params),
    )
    .await;

    match result {
        Ok(Ok(resp)) => {
            match client::unwrap_result(resp) {
                Ok(mut result) => {
                    // Add target info to response
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("target".to_string(), json!(target));
                        if let Some(sender) = from {
                            obj.insert("from".to_string(), json!(sender));
                        }
                    }
                    Response::success(id, result).into()
                }
                Err(e) => {
                    ErrorResponse::internal_error(
                        id,
                        &format!("Target session rejected emit: {e}"),
                    )
                    .into()
                }
            }
        }
        Ok(Err(e)) => {
            ErrorResponse::internal_error(
                id,
                &format!("Failed to connect to target session '{}': {e}", target),
            )
            .into()
        }
        Err(_) => {
            ErrorResponse::internal_error(
                id,
                &format!("Timeout emitting to target session '{}'", target),
            )
            .into()
        }
    }
}

/// Check if a request is hub-level (no `target` param, or `aggregate: true`).
fn is_hub_level(params: &serde_json::Value) -> bool {
    params.get("target").is_none()
        || params.get("aggregate").and_then(|a| a.as_bool()).unwrap_or(false)
}

/// Handle hub-level `event.subscribe` — return aggregated events from all sessions (T-966).
///
/// Params: { timeout_ms?: u64, topic?: string }
/// No `target` param = hub-level aggregation. With `target` param = forwarded to session.
async fn handle_hub_subscribe(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    let agg = match aggregator() {
        Some(a) => a,
        None => {
            return ErrorResponse::internal_error(id, "Event aggregator not initialized").into();
        }
    };

    let timeout_ms = params
        .get("timeout_ms")
        .and_then(|t| t.as_u64())
        .unwrap_or(5000);
    let topic_filter = params.get("topic").and_then(|t| t.as_str());

    let events = agg
        .collect(Duration::from_millis(timeout_ms), topic_filter)
        .await;

    let json_events: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            json!({
                "session": e.session_id,
                "session_name": e.session_name,
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
            "events": json_events,
            "count": json_events.len(),
            "sessions": agg.session_count().await,
        }),
    )
    .into()
}

/// Handle `event.collect` — poll events from multiple sessions (fan-in).
///
/// Params: { targets?: [string], since?: {session_id: seq}, topic?: string }
/// If targets is omitted, collects from all live sessions.
async fn handle_event_collect(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    // Resolve target sessions
    let registrations = if let Some(targets) = params.get("targets").and_then(|t| t.as_array()) {
        let mut regs = Vec::new();
        for t in targets {
            if let Some(name) = t.as_str()
                && let Ok(r) = manager::find_session(name) {
                    regs.push(r);
                }
        }
        regs
    } else {
        match manager::list_sessions(false) {
            Ok(sessions) => sessions
                .iter()
                .filter_map(|s| manager::find_session(s.id.as_str()).ok())
                .collect(),
            Err(e) => {
                return ErrorResponse::internal_error(
                    id,
                    &format!("Failed to list sessions: {e}"),
                )
                .into();
            }
        }
    };

    let since_map = params
        .get("since")
        .and_then(|s| s.as_object())
        .cloned()
        .unwrap_or_default();

    // Global since_default: used as fallback when no per-session cursor exists.
    // Enables --since flag at CLI level to replay history from a sequence number.
    let since_default = params.get("since_default").and_then(|s| s.as_u64());

    let topic_filter = params.get("topic").and_then(|t| t.as_str());

    // Optional timeout_ms: when set, use event.subscribe (server-side blocking)
    // instead of event.poll (instant snapshot). This eliminates polling latency
    // for callers that would otherwise sleep between collect calls.
    let subscribe_timeout_ms = params
        .get("timeout_ms")
        .and_then(|t| t.as_u64());

    // Dispatch polls concurrently with per-target timeout
    let mut join_set = tokio::task::JoinSet::new();
    let num_targets = registrations.len().max(1) as u64;

    for reg in registrations {
        let sid = reg.id.to_string();
        let display_name = reg.display_name.clone();
        let addr = reg.addr.to_transport_addr();
        let since_map = since_map.clone();
        let topic_filter = topic_filter.map(String::from);

        join_set.spawn(async move {
            // Choose RPC method based on timeout_ms parameter
            let (method, rpc_params) = if let Some(timeout_ms) = subscribe_timeout_ms {
                let per_session_timeout = timeout_ms / num_targets;
                let effective_timeout = per_session_timeout.max(100); // at least 100ms
                let mut p = json!({"timeout_ms": effective_timeout});
                if let Some(seq_val) = since_map.get(&sid) {
                    p["since"] = seq_val.clone();
                } else if let Some(default_seq) = since_default {
                    p["since"] = json!(default_seq);
                }
                if let Some(t) = &topic_filter {
                    p["topic"] = json!(t);
                }
                (control::method::EVENT_SUBSCRIBE, p)
            } else {
                let mut p = json!({});
                if let Some(seq_val) = since_map.get(&sid) {
                    p["since"] = seq_val.clone();
                } else if let Some(default_seq) = since_default {
                    p["since"] = json!(default_seq);
                }
                if let Some(t) = &topic_filter {
                    p["topic"] = json!(t);
                }
                (control::method::EVENT_POLL, p)
            };

            let result = tokio::time::timeout(
                PER_TARGET_TIMEOUT,
                client::rpc_call_addr(&addr, method, rpc_params),
            )
            .await;

            match result {
                Ok(Ok(resp)) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        let mut events = Vec::new();
                        if let Some(ev_array) = result["events"].as_array() {
                            for event in ev_array {
                                let mut enriched = event.clone();
                                enriched["session"] = json!(&sid);
                                enriched["session_name"] = json!(&display_name);
                                events.push(enriched);
                            }
                        }
                        let next_seq = result.get("next_seq").cloned();
                        Some((sid, events, next_seq))
                    } else {
                        None
                    }
                }
                Ok(Err(e)) => {
                    tracing::debug!(session = %sid, error = %e, "Collect: failed to reach session");
                    None
                }
                Err(_) => {
                    tracing::debug!(session = %sid, "Collect: timeout reaching session");
                    None
                }
            }
        });
    }

    let mut all_events: Vec<serde_json::Value> = Vec::new();
    let mut cursors = json!({});

    while let Some(result) = join_set.join_next().await {
        if let Ok(Some((sid, events, next_seq))) = result {
            all_events.extend(events);
            if let Some(next) = next_seq {
                cursors[sid] = next;
            }
        }
    }

    // Sort by timestamp, then seq
    all_events.sort_by(|a, b| {
        let ta = a["timestamp"].as_u64().unwrap_or(0);
        let tb = b["timestamp"].as_u64().unwrap_or(0);
        ta.cmp(&tb)
            .then_with(|| {
                let sa = a["seq"].as_u64().unwrap_or(0);
                let sb = b["seq"].as_u64().unwrap_or(0);
                sa.cmp(&sb)
            })
    });

    Response::success(
        id,
        json!({
            "events": all_events,
            "count": all_events.len(),
            "cursors": cursors,
        }),
    )
    .into()
}

/// Handle `session.register_remote` — register a TCP session in the hub's memory.
///
/// Params: { display_name, host, port, pid?, roles?, tags?, capabilities? }
fn handle_register_remote(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let store = match remote_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "Remote store not initialized").into();
        }
    };

    let host = match params.get("host").and_then(|h| h.as_str()) {
        Some(h) => h.to_string(),
        None => return ErrorResponse::new(id, -32602, "Missing 'host' in params").into(),
    };
    let port = match params.get("port").and_then(|p| p.as_u64()) {
        Some(p) => p as u16,
        None => return ErrorResponse::new(id, -32602, "Missing 'port' in params").into(),
    };
    let display_name = params
        .get("display_name")
        .and_then(|n| n.as_str())
        .unwrap_or("remote")
        .to_string();
    let pid = params.get("pid").and_then(|p| p.as_u64()).map(|p| p as u32);
    let roles = extract_string_array(params, "roles");
    let tags = extract_string_array(params, "tags");
    let capabilities = extract_string_array(params, "capabilities");

    let display_name_clone = display_name.clone();
    let host_clone = host.clone();
    let session_id = store.register(crate::remote_store::RemoteSessionInfo { display_name, host, port, pid, roles, tags, capabilities });
    tracing::info!(id = %session_id, "Remote session registered");

    // T-966: Subscribe aggregator to this session's event bus
    if let Some(agg) = aggregator() {
        let target = SessionTarget {
            id: session_id.clone(),
            display_name: display_name_clone,
            addr: TransportAddr::tcp(host_clone, port),
        };
        tokio::spawn(async move {
            agg.add_session(target).await;
        });
    }

    Response::success(id, json!({ "id": session_id })).into()
}

/// Handle `session.heartbeat` — refresh TTL for a remote session.
///
/// Params: { id }
fn handle_heartbeat(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let store = match remote_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "Remote store not initialized").into();
        }
    };

    let session_id = match params.get("id").and_then(|i| i.as_str()) {
        Some(i) => i,
        None => return ErrorResponse::new(id, -32602, "Missing 'id' in params").into(),
    };

    if store.heartbeat(session_id) {
        Response::success(id, json!({ "ok": true })).into()
    } else {
        ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            &format!("Remote session '{}' not found", session_id),
        )
        .into()
    }
}

/// Handle `session.deregister_remote` — remove a remote session.
///
/// Params: { id }
fn handle_deregister_remote(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let store = match remote_store() {
        Some(s) => s,
        None => {
            return ErrorResponse::internal_error(id, "Remote store not initialized").into();
        }
    };

    let session_id = match params.get("id").and_then(|i| i.as_str()) {
        Some(i) => i,
        None => return ErrorResponse::new(id, -32602, "Missing 'id' in params").into(),
    };

    if store.deregister(session_id) {
        tracing::info!(id = %session_id, "Remote session deregistered");
        // T-966: Remove aggregator subscription
        if let Some(agg) = aggregator() {
            let sid = session_id.to_string();
            tokio::spawn(async move {
                agg.remove_session(&sid).await;
            });
        }
        Response::success(id, json!({ "ok": true })).into()
    } else {
        ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            &format!("Remote session '{}' not found", session_id),
        )
        .into()
    }
}

/// Extract a string array from a JSON value by key, defaulting to empty vec.
fn extract_string_array(params: &serde_json::Value, key: &str) -> Vec<String> {
    params
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Handle `orchestrator.route` — discover a specialist, forward a method, relay the response.
///
/// Combines session.discover + forward into a single atomic call:
///   1. Find sessions matching the selector (tags/roles/capabilities/name)
///   2. Forward the specified method+params to the first matching session
///   3. If the first fails, try the next candidate (failover)
///   4. Return the specialist's response plus routing metadata
///
/// When `task_type` is provided, sessions with a matching `task-type:<type>` tag are
/// sorted before other candidates (preferred but not required). This enables task-aware
/// routing without breaking existing method-based routing.
///
/// Params: {
///   selector: { tags?: [...], roles?: [...], capabilities?: [...], name?: string },
///   method: string,       // RPC method to call on the specialist
///   params: object,       // params to pass to the specialist
///   timeout_secs?: number // per-target timeout (default: 5)
///   task_type?: string    // task workflow type (build/test/audit/review) — prefers
///                         // specialists with matching "task-type:<type>" tag
/// }
///
/// Response: {
///   routed_to: { id, display_name },
///   candidates: number,
///   result: <specialist's response payload>
/// }
async fn handle_orchestrator_route(
    id: serde_json::Value,
    params: &serde_json::Value,
) -> RpcResponse {
    // Extract required method
    let method = match params.get("method").and_then(|m| m.as_str()) {
        Some(m) => m.to_string(),
        None => {
            return ErrorResponse::new(id, -32602, "Missing 'method' in params").into();
        }
    };

    // Check if this is a mutating command (skip bypass for read-write operations)
    let mutating = params
        .get("mutating")
        .and_then(|m| m.as_bool())
        .unwrap_or(false);

    // Optional task workflow type (build/test/audit/review) — prefers matching specialists
    let task_type = params.get("task_type").and_then(|t| t.as_str()).map(String::from);

    // Build the cache/bypass key: "method" or "method::task_type" when task_type is present.
    // This ensures task-type-specific routes are cached separately from generic ones.
    let routing_key = match &task_type {
        Some(tt) => format!("{method}::{tt}"),
        None => method.clone(),
    };

    // Check bypass registry before routing to a specialist (skip for mutating commands)
    if !mutating {
        let registry = crate::bypass::BypassRegistry::load();
        if let Some(entry) = registry.check(&routing_key) {
            tracing::info!(
                method = %method,
                routing_key = %routing_key,
                run_count = entry.run_count,
                "orchestrator.route: bypass registry hit — command is Tier 3"
            );
            return Response::success(
                id,
                json!({
                    "bypassed": true,
                    "command": method,
                    "tier": entry.tier,
                    "run_count": entry.run_count,
                    "task_type": task_type,
                    "note": "routing shortcut, not execution authorization",
                }),
            )
            .into();
        }
    }

    // Layer 2: Check route cache (between bypass and full discovery)
    if !mutating {
        let route_cache = crate::route_cache::RouteCache::load();
        match route_cache.lookup(&routing_key) {
            crate::route_cache::CacheLookup::Hit(entry) => {
                tracing::info!(
                    method = %method,
                    specialist = %entry.specialist,
                    confidence = entry.effective_confidence(),
                    hit_count = entry.hit_count,
                    "orchestrator.route: route cache hit"
                );
                // Use cached route as selector hint — filter by specialist name
                let cached_selector = json!({
                    "name": entry.specialist,
                });
                // Fall through to discovery with the cached selector
                // (we override selector below to prefer the cached specialist)
                let forward_params_inner = params.get("params").cloned().unwrap_or(json!({}));
                let timeout_secs_inner = params
                    .get("timeout_secs")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(5);
                let timeout_inner = Duration::from_secs(timeout_secs_inner);

                let sessions = match manager::list_sessions(false) {
                    Ok(s) => s,
                    Err(e) => {
                        return ErrorResponse::internal_error(
                            id,
                            &format!("Failed to list sessions: {e}"),
                        )
                        .into();
                    }
                };

                let name_filter = cached_selector.get("name").and_then(|n| n.as_str());
                let candidates: Vec<_> = sessions
                    .into_iter()
                    .filter(|s| {
                        name_filter.is_none_or(|n| {
                            s.display_name.to_lowercase().contains(&n.to_lowercase())
                        })
                    })
                    .collect();

                if let Some(reg) = candidates.first() {
                    let addr = reg.addr.to_transport_addr();
                    let session_id = reg.id.as_str().to_string();
                    let result = tokio::time::timeout(timeout_inner, async {
                        let mut c = client::Client::connect_addr(&addr).await?;
                        c.call(&method, id.clone(), forward_params_inner.clone()).await
                    })
                    .await;

                    match result {
                        Ok(Ok(RpcResponse::Success(resp))) => {
                            // Record cache hit (keyed by routing_key for task-type awareness)
                            let cache_path = crate::route_cache::cache_path();
                            let rk_clone = routing_key.clone();
                            if let Ok(mut cache) = std::fs::read_to_string(&cache_path)
                                .ok()
                                .and_then(|d| serde_json::from_str::<crate::route_cache::RouteCache>(&d).ok())
                                .ok_or(())
                                .or_else(|_| Ok::<_, ()>(crate::route_cache::RouteCache::default()))
                            {
                                cache.record_hit(&rk_clone);
                                let _ = cache.save_to(&cache_path);
                            }

                            // Also record in bypass registry
                            if !mutating {
                                let reg_path = crate::bypass::registry_path();
                                let rk_clone2 = routing_key.clone();
                                let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                                    let _ = r.record_orchestrated_run(&rk_clone2, crate::bypass::RunOutcome::Success);
                                });
                            }

                            return Response::success(
                                id,
                                json!({
                                    "routed_to": {
                                        "id": session_id,
                                        "display_name": reg.display_name,
                                    },
                                    "cached_route": true,
                                    "candidates": 1,
                                    "result": resp.result,
                                }),
                            )
                            .into();
                        }
                        _ => {
                            // Cache route failed — invalidate and fall through to full discovery
                            tracing::warn!(
                                method = %method,
                                routing_key = %routing_key,
                                "orchestrator.route: cached route failed, falling through to full discovery"
                            );
                            let cache_path = crate::route_cache::cache_path();
                            let rk_clone = routing_key.clone();
                            if let Ok(mut cache) = std::fs::read_to_string(&cache_path)
                                .ok()
                                .and_then(|d| serde_json::from_str::<crate::route_cache::RouteCache>(&d).ok())
                                .ok_or(())
                                .or_else(|_| Ok::<_, ()>(crate::route_cache::RouteCache::default()))
                            {
                                cache.invalidate(&rk_clone);
                                let _ = cache.save_to(&cache_path);
                            }
                        }
                    }
                }
                // Cached specialist not found or failed — fall through to normal discovery
            }
            crate::route_cache::CacheLookup::Stale(entry) => {
                tracing::debug!(
                    method = %method,
                    specialist = %entry.specialist,
                    confidence = entry.effective_confidence(),
                    "orchestrator.route: route cache stale, proceeding to full discovery"
                );
                // Fall through to normal discovery (stale hint logged but not used)
            }
            crate::route_cache::CacheLookup::Miss => {
                // No cache entry — normal discovery
            }
        }
    }

    let forward_params = params.get("params").cloned().unwrap_or(json!({}));
    let selector = params.get("selector").cloned().unwrap_or(json!({}));
    let timeout_secs = params
        .get("timeout_secs")
        .and_then(|t| t.as_u64())
        .unwrap_or(5);
    let timeout = Duration::from_secs(timeout_secs);

    // Discover candidates using same filter logic as session.discover
    let sessions = match manager::list_sessions(false) {
        Ok(s) => s,
        Err(e) => {
            return ErrorResponse::internal_error(
                id,
                &format!("Failed to list sessions: {e}"),
            )
            .into();
        }
    };

    let tag_filter = extract_string_array(&selector, "tags");
    let role_filter = extract_string_array(&selector, "roles");
    let cap_filter = extract_string_array(&selector, "capabilities");
    let name_filter = selector.get("name").and_then(|n| n.as_str());

    let mut candidates: Vec<_> = sessions
        .into_iter()
        .filter(|s| {
            tag_filter.iter().all(|t| s.tags.contains(t))
                && role_filter.iter().all(|r| s.roles.contains(r))
                && cap_filter.iter().all(|c| s.capabilities.contains(c))
                && name_filter.is_none_or(|n| {
                    s.display_name.to_lowercase().contains(&n.to_lowercase())
                })
        })
        .collect();

    // Task-type preference: sort candidates so sessions with a matching
    // "task-type:<type>" tag appear before others. This is a stable sort —
    // within each group the original order (creation time) is preserved.
    if let Some(ref tt) = task_type {
        let task_type_tag = format!("task-type:{tt}");
        candidates.sort_by_key(|s| if s.tags.contains(&task_type_tag) { 0u8 } else { 1u8 });
    }

    // Also check remote sessions
    if let Some(store) = remote_store() {
        let remote_matches: Vec<_> = store
            .list_live()
            .into_iter()
            .filter(|e| {
                tag_filter.iter().all(|t| e.tags.contains(t))
                    && role_filter.iter().all(|r| e.roles.contains(r))
                    && cap_filter.iter().all(|c| e.capabilities.contains(c))
                    && name_filter.is_none_or(|n| {
                        e.display_name.to_lowercase().contains(&n.to_lowercase())
                    })
            })
            .collect();

        // Convert remote entries to a forwarding attempt below
        for entry in remote_matches {
            // Try remote candidates after local ones
            let addr = TransportAddr::Tcp {
                host: entry.host.clone(),
                port: entry.port,
            };
            let result = tokio::time::timeout(timeout, async {
                let mut c = client::Client::connect_addr(&addr).await?;
                c.call(&method, id.clone(), forward_params.clone()).await
            })
            .await;

            if let Ok(Ok(RpcResponse::Success(resp))) = result {
                return Response::success(
                    id,
                    json!({
                        "routed_to": { "id": entry.id, "display_name": entry.display_name },
                        "candidates": candidates.len() + 1,
                        "result": resp.result,
                    }),
                )
                .into();
            }
        }
    }

    let total_candidates = candidates.len();

    if candidates.is_empty() {
        return ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            "No sessions match the selector",
        )
        .into();
    }

    // Try candidates in order (failover), skipping circuit-opened sessions
    let cb = crate::circuit_breaker::global();
    let mut last_error = String::new();
    let mut skipped_count = 0usize;
    let mut tried_count = 0usize;
    for reg in candidates.drain(..) {
        let session_id = reg.id.as_str().to_string();

        // Skip sessions with open circuits (avoids cascading timeout delays)
        if cb.should_skip(&session_id) {
            skipped_count += 1;
            tracing::debug!(
                session = %session_id,
                "orchestrator.route: circuit open — skipping candidate"
            );
            last_error = format!("{}: circuit open (skipped)", reg.display_name);
            continue;
        }

        tried_count += 1;
        let addr = reg.addr.to_transport_addr();
        let result = tokio::time::timeout(timeout, async {
            let mut c = client::Client::connect_addr(&addr).await?;
            c.call(&method, id.clone(), forward_params.clone()).await
        })
        .await;

        match result {
            Ok(Ok(RpcResponse::Success(resp))) => {
                cb.record_success(&session_id);
                // Record successful orchestrated run (skip for mutating commands)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ =
                        crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                            if r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::Success) {
                                tracing::info!(
                                    routing_key = %rk_clone,
                                    "orchestrator.route: command promoted to bypass registry"
                                );
                            }
                        });

                    // Record route in cache (Layer 2) for future lookups
                    let cache_path = crate::route_cache::cache_path();
                    let rk_for_cache = routing_key.clone();
                    let specialist_name = reg.display_name.clone();
                    let mut route_cache = crate::route_cache::RouteCache::load_from(&cache_path);
                    route_cache.record_route(
                        &rk_for_cache,
                        &specialist_name,
                        crate::route_cache::RequestSchema::default(),
                    );
                    let _ = route_cache.save_to(&cache_path);
                    tracing::debug!(
                        routing_key = %rk_for_cache,
                        specialist = %specialist_name,
                        "orchestrator.route: recorded route in cache"
                    );
                }
                return Response::success(
                    id,
                    json!({
                        "routed_to": {
                            "id": reg.id.as_str(),
                            "display_name": reg.display_name,
                        },
                        "candidates": total_candidates,
                        "result": resp.result,
                    }),
                )
                .into();
            }
            Ok(Ok(RpcResponse::Error(e))) => {
                // RPC error = command failure (the specialist responded with an error)
                // Command failures don't open the circuit (session is alive, just rejected the call)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                        r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::CommandFailure);
                    });
                }
                last_error = format!("{}: {}", reg.display_name, e.error.message);
                tracing::debug!(
                    target = reg.display_name,
                    error = %e.error.message,
                    "orchestrator.route: candidate returned error, trying next"
                );
            }
            Ok(Err(e)) => {
                cb.record_failure(&session_id);
                // Connection error = infra failure (specialist never received the call)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                        r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::InfraFailure);
                    });
                }
                last_error = format!("{}: {}", reg.display_name, e);
                tracing::debug!(
                    target = reg.display_name,
                    error = %e,
                    "orchestrator.route: candidate connection failed, trying next"
                );
            }
            Err(_) => {
                cb.record_failure(&session_id);
                // Timeout = infra failure (specialist didn't respond in time)
                if !mutating {
                    let reg_path = crate::bypass::registry_path();
                    let rk_clone = routing_key.clone();
                    let _ = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
                        r.record_orchestrated_run(&rk_clone, crate::bypass::RunOutcome::InfraFailure);
                    });
                }
                last_error = format!("{}: timeout", reg.display_name);
                tracing::debug!(
                    target = reg.display_name,
                    "orchestrator.route: candidate timed out, trying next"
                );
            }
        }
    }

    ErrorResponse::new(
        id,
        control::error_code::SESSION_NOT_FOUND,
        &format!(
            "All {} candidate(s) failed ({} tried, {} circuit-open skipped). Last: {}",
            total_candidates, tried_count, skipped_count, last_error
        ),
    )
    .into()
}

/// Handle `orchestrator.bypass_status` — query the bypass registry contents.
fn handle_bypass_status(id: serde_json::Value) -> RpcResponse {
    let registry = crate::bypass::BypassRegistry::load();
    let entries: Vec<_> = registry
        .entries
        .values()
        .map(|e| {
            json!({
                "command": e.command,
                "tier": e.tier,
                "run_count": e.run_count,
                "fail_count": e.fail_count,
                "promoted_at": e.promoted_at,
                "last_run": e.last_run,
            })
        })
        .collect();
    let candidates: Vec<_> = registry
        .candidates
        .iter()
        .map(|(cmd, stats)| {
            json!({
                "command": cmd,
                "success_count": stats.success_count,
                "fail_count": stats.fail_count,
                "remaining": crate::bypass::PROMOTION_THRESHOLD.saturating_sub(stats.success_count),
            })
        })
        .collect();
    Response::success(
        id,
        json!({
            "bypassed_commands": entries,
            "promotion_candidates": candidates,
        }),
    )
    .into()
}

/// Handle `orchestrator.bypass_invalidate` — remove bypass entries by pattern or all.
///
/// Params:
///   - `pattern` (string, optional): substring pattern to match (case-insensitive).
///     If omitted, clears the entire registry.
///   - `all` (bool, optional): if true, clears everything (same as omitting pattern).
fn handle_bypass_invalidate(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let all = params
        .get("all")
        .and_then(|a| a.as_bool())
        .unwrap_or(false);
    let pattern = params.get("pattern").and_then(|p| p.as_str());

    let reg_path = crate::bypass::registry_path();
    let result = crate::bypass::BypassRegistry::locked_update(&reg_path, |r| {
        let removed = if all || pattern.is_none() {
            r.invalidate_all()
        } else if let Some(pat) = pattern {
            r.invalidate(pat)
        } else {
            unreachable!()
        };
        tracing::info!(
            pattern = pattern.unwrap_or("*"),
            removed,
            "orchestrator.bypass_invalidate: cleared bypass entries"
        );
    });

    match result {
        Ok(registry) => Response::success(
            id,
            json!({
                "invalidated": true,
                "remaining_entries": registry.entries.len(),
                "remaining_candidates": registry.candidates.len(),
            }),
        )
        .into(),
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("Failed to update registry: {e}")).into()
        }
    }
}

/// Forward a request to the target session specified in params.target.
async fn forward_to_target(req: &Request, id: serde_json::Value) -> RpcResponse {
    // Extract target from params
    let target = match req.params.get("target").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(
                id,
                termlink_protocol::control::error_code::SESSION_NOT_FOUND,
                "Missing 'target' in params",
            )
            .into();
        }
    };

    // Resolve target: try local FS first, then remote store
    let addr = if let Ok(reg) = manager::find_session(target) {
        reg.addr.to_transport_addr()
    } else if let Some(entry) = remote_store().and_then(|s| {
        // Try by ID first, then by display name
        s.get(target).or_else(|| {
            s.list_live()
                .into_iter()
                .find(|e| e.display_name == target || e.id == target)
        })
    }) {
        TransportAddr::Tcp {
            host: entry.host.clone(),
            port: entry.port,
        }
    } else {
        return ErrorResponse::new(
            id,
            control::error_code::SESSION_NOT_FOUND,
            &format!("Target '{}' not found (local or remote)", target),
        )
        .into();
    };

    // Forward the request, preserving the original request id
    let forward_result = async {
        let mut c = client::Client::connect_addr(&addr).await?;
        c.call(&req.method, id.clone(), req.params.clone()).await
    };
    match forward_result.await {
        Ok(resp) => resp,
        Err(e) => {
            tracing::warn!(
                target = target,
                error = %e,
                "Failed to forward request to session"
            );
            ErrorResponse::new(
                id,
                control::error_code::SESSION_NOT_FOUND,
                &format!("Failed to reach target: {e}"),
            )
            .into()
        }
    }
}

/// Resolve a target string to a transport address.
///
/// Public so the CLI can use direct routing without the hub.
pub fn resolve_target(target: &str) -> Result<TransportAddr, String> {
    // Try local FS first
    if let Ok(r) = manager::find_session(target) {
        return Ok(r.addr.to_transport_addr());
    }
    // Try remote store
    if let Some(entry) = remote_store().and_then(|s| {
        s.get(target).or_else(|| {
            s.list_live()
                .into_iter()
                .find(|e| e.display_name == target || e.id == target)
        })
    }) {
        return Ok(TransportAddr::Tcp {
            host: entry.host,
            port: entry.port,
        });
    }
    Err(format!("Session '{}' not found (local or remote)", target))
}

/// Resolve a target string to a socket path (convenience for Unix-only callers).
pub fn resolve_target_path(target: &str) -> Result<std::path::PathBuf, String> {
    manager::find_session(target)
        .map(|r| r.socket_path().to_path_buf())
        .map_err(|e| e.to_string())
}

/// Handle `inbox.list` — list pending transfers for a target (T-988).
///
/// Params: { target: string }
fn handle_inbox_list(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let target = match params.get("target").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return ErrorResponse::new(id, -32602, "Missing 'target' in params").into();
        }
    };

    match crate::inbox::list_pending(target) {
        Ok(transfers) => {
            Response::success(id, json!({
                "target": target,
                "transfers": transfers,
            }))
            .into()
        }
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("Inbox list error: {e}")).into()
        }
    }
}

/// Handle `inbox.status` — show inbox overview (T-988).
fn handle_inbox_status(id: serde_json::Value) -> RpcResponse {
    match crate::inbox::list_all_targets() {
        Ok(targets) => {
            let total: usize = targets.iter().map(|(_, c)| c).sum();
            Response::success(id, json!({
                "total_transfers": total,
                "targets": targets.iter().map(|(name, count)| json!({
                    "target": name,
                    "pending": count,
                })).collect::<Vec<_>>(),
            }))
            .into()
        }
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("Inbox status error: {e}")).into()
        }
    }
}

fn handle_inbox_clear(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let all = params.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
    let target = params.get("target").and_then(|t| t.as_str());

    if !all && target.is_none() {
        return ErrorResponse::new(id, -32602, "Missing 'target' or 'all' in params").into();
    }

    let cleared = if all {
        crate::inbox::clear_all()
    } else {
        crate::inbox::clear_target(target.unwrap())
    };

    Response::success(id, json!({
        "ok": true,
        "cleared": cleared,
        "target": target.unwrap_or("*"),
    }))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::RwLock;

    use termlink_session::handler::SessionContext;
    use termlink_session::registration::SessionConfig;
    use termlink_session::Registration;
    use termlink_session::server;

    use crate::test_util::ENV_LOCK;
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-hub-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn start_test_session(
        sessions_dir: &Path,
        name: &str,
    ) -> (
        tokio::task::JoinHandle<()>,
        Registration,
    ) {
        let config = SessionConfig {
            display_name: Some(name.into()),
            ..Default::default()
        };
        let session = termlink_session::Session::register_in(config, sessions_dir)
            .await
            .unwrap();

        let session_id = session.id().clone();
        let (registration, listener, _) = session.into_parts();
        let reg = registration.clone();
        let json_path = Registration::json_path(sessions_dir, &session_id);
        let ctx = SessionContext::new(registration)
            .with_registration_path(json_path);
        let shared = Arc::new(RwLock::new(ctx));

        let handle = tokio::spawn(async move {
            server::run_accept_loop(listener, shared).await;
        });

        (handle, reg)
    }

    async fn start_test_session_with_tags(
        sessions_dir: &Path,
        name: &str,
        tags: Vec<String>,
    ) -> (
        tokio::task::JoinHandle<()>,
        Registration,
    ) {
        let config = SessionConfig {
            display_name: Some(name.into()),
            tags,
            ..Default::default()
        };
        let session = termlink_session::Session::register_in(config, sessions_dir)
            .await
            .unwrap();

        let session_id = session.id().clone();
        let (registration, listener, _) = session.into_parts();
        let reg = registration.clone();
        let json_path = Registration::json_path(sessions_dir, &session_id);
        let ctx = SessionContext::new(registration)
            .with_registration_path(json_path);
        let shared = Arc::new(RwLock::new(ctx));

        let handle = tokio::spawn(async move {
            server::run_accept_loop(listener, shared).await;
        });

        (handle, reg)
    }

    #[tokio::test]
    async fn discover_returns_sessions() {
        let dir = test_dir();

        let (h1, _r1) = start_test_session(&dir, "session-a").await;
        let (h2, _r2) = start_test_session(&dir, "session-b").await;

        // Discover using list_sessions_in directly
        let sessions = manager::list_sessions_in(&dir, false).unwrap();
        assert_eq!(sessions.len(), 2);

        let names: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
        assert!(names.contains(&"session-a"));
        assert!(names.contains(&"session-b"));

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn forward_to_target_session() {
        let dir = test_dir();

        let (handle, reg) = start_test_session(&dir, "target-sess").await;

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Send a ping directly to the session (simulating hub forwarding)
        let resp = client::rpc_call(
            reg.socket_path(),
            "termlink.ping",
            json!({}),
        )
        .await
        .unwrap();

        let result = client::unwrap_result(resp).unwrap();
        assert_eq!(result["display_name"], "target-sess");

        handle.abort();
    }

    #[tokio::test]
    async fn forward_missing_target_returns_error() {
        let req = Request::new(
            "query.status",
            json!("req-1"),
            json!({"target": "nonexistent-session"}),
        );

        let resp = route(&req).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::SESSION_NOT_FOUND);
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn broadcast_emits_to_sessions() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "bcast-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "bcast-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Override env so manager finds sessions
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "topic": "deploy.start",
            "payload": {"version": "1.0"},
        });

        let resp = handle_event_broadcast(json!("bc-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["topic"], "deploy.start");
            assert_eq!(r.result["targeted"], 2);
            assert_eq!(r.result["succeeded"], 2);
            assert_eq!(r.result["failed"], 0);
        } else {
            panic!("Expected success response");
        }

        // Verify events landed on each session
        let resp = client::rpc_call(r1.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "deploy.start");

        let resp = client::rpc_call(r2.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "deploy.start");

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn broadcast_with_targets_filters() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "tgt-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "tgt-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Only target session a
        let params = json!({
            "topic": "test.only",
            "payload": {},
            "targets": [r1.id.as_str()],
        });

        let resp = handle_event_broadcast(json!("bc-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["targeted"], 1);
            assert_eq!(r.result["succeeded"], 1);
        } else {
            panic!("Expected success response");
        }

        // Session b should have no events
        let resp = client::rpc_call(r2.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        assert_eq!(result["count"], 0);

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn collect_aggregates_events() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "coll-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "coll-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Emit events to each session directly
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "build.done", "payload": {"id": 1}}),
        ).await.unwrap();

        client::rpc_call(
            r2.socket_path(),
            "event.emit",
            json!({"topic": "test.pass", "payload": {"id": 2}}),
        ).await.unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_event_collect(json!("cl-1"), &json!({})).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 2);
            let events = r.result["events"].as_array().unwrap();
            let topics: Vec<&str> = events.iter().filter_map(|e| e["topic"].as_str()).collect();
            assert!(topics.contains(&"build.done"));
            assert!(topics.contains(&"test.pass"));

            // Each event should have session metadata
            for event in events {
                assert!(event.get("session").is_some());
                assert!(event.get("session_name").is_some());
            }

            // Cursors should be present
            let cursors = r.result["cursors"].as_object().unwrap();
            assert_eq!(cursors.len(), 2);
        } else {
            panic!("Expected success response");
        }

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn collect_with_since_cursors() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "cur-a").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Emit two events
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "a", "payload": {}}),
        ).await.unwrap();
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "b", "payload": {}}),
        ).await.unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Collect with since cursor at seq 0 — should get only event at seq 1
        let sid = r1.id.as_str();
        let params = json!({
            "since": { sid: 0 },
        });
        let resp = handle_event_collect(json!("cl-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["count"], 1);
            let events = r.result["events"].as_array().unwrap();
            assert_eq!(events[0]["topic"], "b");
        } else {
            panic!("Expected success response");
        }

        h1.abort();
    }

    #[tokio::test]
    async fn broadcast_missing_topic_returns_error() {
        let params = json!({"payload": {}});
        let resp = handle_event_broadcast(json!("bc-err"), &params).await;
        if let RpcResponse::Error(err) = resp {
            assert!(err.error.message.contains("Missing"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn discover_with_filters() {
        let _lock = ENV_LOCK.lock().await;
        // Clear remote store to avoid leakage from other tests
        if let Some(s) = super::remote_store() { s.clear(); }
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Register sessions with different tags via session.update
        let (h1, r1) = start_test_session(&sessions_dir, "web-prod").await;
        let (h2, r2) = start_test_session(&sessions_dir, "api-staging").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Tag session 1 as "prod"
        client::rpc_call(
            r1.socket_path(),
            "session.update",
            json!({"tags": ["prod", "web"]}),
        ).await.unwrap();

        // Tag session 2 as "staging"
        client::rpc_call(
            r2.socket_path(),
            "session.update",
            json!({"tags": ["staging", "api"]}),
        ).await.unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Discover with tag filter — only prod
        let resp = handle_discover(json!("d-1"), &json!({"tags": ["prod"]})).await;

        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0]["display_name"], "web-prod");
            assert!(sessions[0]["tags"].as_array().unwrap().contains(&json!("prod")));
        } else {
            panic!("Expected success");
        }

        // Discover with name filter
        let resp = handle_discover(json!("d-2"), &json!({"name": "api"})).await;

        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0]["display_name"], "api-staging");
        } else {
            panic!("Expected success");
        }

        // Discover with no filters — gets both
        let resp = handle_discover(json!("d-3"), &json!({})).await;

        if let RpcResponse::Success(r) = resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert_eq!(sessions.len(), 2);
        } else {
            panic!("Expected success");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn forward_without_target_param_returns_error() {
        let req = Request::new(
            "query.status",
            json!("req-1"),
            json!({}), // no target
        );

        let resp = route(&req).await.unwrap();
        if let RpcResponse::Error(err) = resp {
            assert_eq!(err.error.code, control::error_code::SESSION_NOT_FOUND);
            assert!(err.error.message.contains("Missing"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn register_remote_and_discover() {
        let _lock = ENV_LOCK.lock().await;
        // Initialize the remote store for this test (clear any leftovers)
        let _store = super::init_remote_store();
        if let Some(s) = super::remote_store() { s.clear(); }

        // Register a remote session via RPC handler
        let params = json!({
            "display_name": "remote-worker",
            "host": "192.168.1.50",
            "port": 9001,
            "pid": 12345,
            "tags": ["gpu", "worker"],
            "roles": ["compute"],
        });
        let resp = super::handle_register_remote(json!("reg-1"), &params);
        let session_id = if let RpcResponse::Success(r) = &resp {
            r.result["id"].as_str().unwrap().to_string()
        } else {
            panic!("Expected success response from register_remote");
        };
        assert!(session_id.starts_with("tl-tcp-"));

        // Discover should include the remote session
        let resp = super::handle_discover(json!("d-1"), &json!({})).await;
        if let RpcResponse::Success(r) = &resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            let remote = sessions.iter().find(|s| s["id"] == session_id);
            assert!(remote.is_some(), "Remote session should appear in discover");
            let remote = remote.unwrap();
            assert_eq!(remote["display_name"], "remote-worker");
            assert_eq!(remote["addr"]["type"], "tcp");
            assert_eq!(remote["addr"]["host"], "192.168.1.50");
            assert_eq!(remote["addr"]["port"], 9001);
            assert_eq!(remote["remote"], true);
        } else {
            panic!("Expected success response from discover");
        }

        // Discover with tag filter should find it
        let resp = super::handle_discover(json!("d-2"), &json!({"tags": ["gpu"]})).await;
        if let RpcResponse::Success(r) = &resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert!(sessions.iter().any(|s| s["id"] == session_id));
        } else {
            panic!("Expected success");
        }

        // Heartbeat should work
        let resp = super::handle_heartbeat(json!("hb-1"), &json!({"id": session_id}));
        if let RpcResponse::Success(r) = &resp {
            assert_eq!(r.result["ok"], true);
        } else {
            panic!("Expected success from heartbeat");
        }

        // Deregister
        let resp = super::handle_deregister_remote(json!("dr-1"), &json!({"id": session_id}));
        if let RpcResponse::Success(r) = &resp {
            assert_eq!(r.result["ok"], true);
        } else {
            panic!("Expected success from deregister");
        }

        // Should no longer appear in discover
        let resp = super::handle_discover(json!("d-3"), &json!({})).await;
        if let RpcResponse::Success(r) = &resp {
            let sessions = r.result["sessions"].as_array().unwrap();
            assert!(!sessions.iter().any(|s| s["id"] == session_id));
        } else {
            panic!("Expected success");
        }
    }

    /// Helper: start a hub with Unix + TCP listeners.
    /// Returns (hub_handle, shutdown_tx, hub_socket_path, tcp_port, secret_hex).
    async fn start_hub_with_tcp(
        dir: &Path,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::watch::Sender<bool>,
        PathBuf,
        u16,
        String,
    ) {
        use crate::server::run_accept_loop;
        use tokio::net::{TcpListener, UnixListener};
        use tokio::sync::watch;

        let hub_socket = dir.join("hub.sock");
        let secret = termlink_session::auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        (handle, tx, hub_socket, tcp_port, secret_hex)
    }

    /// Helper: connect to TCP, authenticate, return (lines_reader, writer).
    async fn tcp_connect_and_auth(
        tcp_port: u16,
        secret_hex: &str,
        scope: termlink_session::auth::PermissionScope,
    ) -> (
        tokio::io::Lines<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>,
        tokio::net::tcp::OwnedWriteHalf,
    ) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let secret_vec: Vec<u8> = (0..secret_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&secret_hex[i..i + 2], 16).unwrap())
            .collect();
        let secret_bytes: [u8; 32] = secret_vec.try_into().expect("secret must be 32 bytes");

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = tokio::io::BufReader::new(reader).lines();

        let token = termlink_session::auth::create_token(&secret_bytes, scope, "", 3600);
        let req = json!({
            "jsonrpc": "2.0",
            "method": "hub.auth",
            "id": "auth",
            "params": { "token": token.raw }
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["result"]["authenticated"], true);

        (lines, writer)
    }

    #[tokio::test]
    async fn tcp_broadcast_delivers_to_sessions() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "tcp-bcast-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "tcp-bcast-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        // Connect via TCP and authenticate with Execute scope
        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Execute,
        )
        .await;

        // Broadcast event via TCP connection
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.broadcast",
            "id": "bc-tcp-1",
            "params": {
                "topic": "deploy.tcp",
                "payload": {"from": "remote-machine"},
            }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "bc-tcp-1");
        assert_eq!(resp["result"]["topic"], "deploy.tcp");
        assert_eq!(resp["result"]["targeted"], 2);
        assert_eq!(resp["result"]["succeeded"], 2);
        assert_eq!(resp["result"]["failed"], 0);

        // Verify events landed on each session
        let resp = client::rpc_call(r1.socket_path(), "event.poll", json!({}))
            .await
            .unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "deploy.tcp");
        assert_eq!(events[0]["payload"]["from"], "remote-machine");

        let resp = client::rpc_call(r2.socket_path(), "event.poll", json!({}))
            .await
            .unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "deploy.tcp");

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn tcp_collect_aggregates_events() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "tcp-coll-a").await;
        let (h2, r2) = start_test_session(&sessions_dir, "tcp-coll-b").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Emit events directly to each session
        client::rpc_call(
            r1.socket_path(),
            "event.emit",
            json!({"topic": "build.done", "payload": {"machine": "A"}}),
        )
        .await
        .unwrap();
        client::rpc_call(
            r2.socket_path(),
            "event.emit",
            json!({"topic": "test.pass", "payload": {"machine": "B"}}),
        )
        .await
        .unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        // Connect via TCP and authenticate
        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Execute,
        )
        .await;

        // Collect events via TCP connection
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.collect",
            "id": "cl-tcp-1",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "cl-tcp-1");
        assert_eq!(resp["result"]["count"], 2);
        let events = resp["result"]["events"].as_array().unwrap();
        let topics: Vec<&str> = events.iter().filter_map(|e| e["topic"].as_str()).collect();
        assert!(topics.contains(&"build.done"));
        assert!(topics.contains(&"test.pass"));

        // Each event should have session metadata
        for event in events {
            assert!(event.get("session").is_some());
            assert!(event.get("session_name").is_some());
        }

        // Cursors should be present
        let cursors = resp["result"]["cursors"].as_object().unwrap();
        assert_eq!(cursors.len(), 2);

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn tcp_unauthenticated_broadcast_rejected() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, _secret_hex) =
            start_hub_with_tcp(&dir).await;

        // Connect via TCP without authenticating
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = TokioBufReader::new(reader).lines();

        // Try broadcast — should be rejected
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.broadcast",
            "id": "bc-noauth",
            "params": {"topic": "test", "payload": {}}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Broadcast should require auth");

        // Try collect — should also be rejected
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.collect",
            "id": "cl-noauth",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Collect should require auth");
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn forward_to_remote_session_via_tcp() {
        let _lock = ENV_LOCK.lock().await;
        // Start a real session listening on TCP
        let dir = test_dir();
        // Isolate runtime dir so connect_addr won't find a stale hub.cert.pem
        // from a previous real hub run (T-165 TLS auto-detection).
        // SAFETY: ENV_LOCK ensures single-threaded access to env vars in tests.
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", dir.to_str().unwrap()) };
        let (handle, reg) = start_test_session(&dir, "tcp-target").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Also start a TCP listener that forwards to this session
        // (simulating a remote session reachable via TCP)
        let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let tcp_port = tcp_listener.local_addr().unwrap().port();
        let socket_path = reg.socket_path().to_path_buf();

        // Proxy: accept TCP, forward to Unix session
        let proxy_handle = tokio::spawn(async move {
            loop {
                let (tcp_stream, _) = tcp_listener.accept().await.unwrap();
                let sp = socket_path.clone();
                tokio::spawn(async move {
                    let unix_stream = tokio::net::UnixStream::connect(&sp).await.unwrap();
                    let (mut tcp_r, mut tcp_w) = tokio::io::split(tcp_stream);
                    let (mut unix_r, mut unix_w) = tokio::io::split(unix_stream);
                    tokio::select! {
                        _ = tokio::io::copy(&mut tcp_r, &mut unix_w) => {}
                        _ = tokio::io::copy(&mut unix_r, &mut tcp_w) => {}
                    }
                });
            }
        });

        // Initialize remote store and register the session as remote (clear first)
        let _store = super::init_remote_store();
        let store = super::remote_store().unwrap();
        store.clear();
        let remote_id = store.register(crate::remote_store::RemoteSessionInfo {
            display_name: "tcp-target".into(),
            host: "127.0.0.1".into(),
            port: tcp_port,
            pid: None,
            roles: vec![],
            tags: vec![],
            capabilities: vec![],
        });

        // Forward a ping to the remote session via the router
        let req = Request::new(
            "termlink.ping",
            json!("fwd-tcp-1"),
            json!({"target": &remote_id}),
        );
        let resp = super::route(&req).await.unwrap();
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["display_name"], "tcp-target");
            assert_eq!(r.result["state"], "ready");
        } else {
            panic!("Expected success — forward to remote TCP session should work");
        }

        // Also test lookup by display name
        let req = Request::new(
            "termlink.ping",
            json!("fwd-tcp-2"),
            json!({"target": "tcp-target"}),
        );
        let resp = super::route(&req).await.unwrap();
        // This might resolve to local or remote — either is fine for this test
        assert!(matches!(resp, RpcResponse::Success(_)));

        proxy_handle.abort();
        handle.abort();
    }

    /// T-923 end-to-end: TCP-bound hub + hub.auth + transparent forwarder
    /// to a local session, proving the cross-host routing path that T-924's
    /// call_session() CLI helper will drive.
    #[tokio::test]
    async fn tcp_forward_to_local_session_after_auth() {
        use tokio::io::AsyncWriteExt;

        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (session_handle, _reg) =
            start_test_session(&sessions_dir, "fwd-tcp-local").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // manager::find_session() reads sessions_dir() which is relative to
        // TERMLINK_RUNTIME_DIR; point it at the test runtime.
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Interact,
        )
        .await;

        // Forward termlink.ping through the hub by session display name.
        let req = json!({
            "jsonrpc": "2.0",
            "method": "termlink.ping",
            "id": "fwd-1",
            "params": { "target": "fwd-tcp-local" }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "fwd-1");
        assert!(
            resp.get("result").is_some(),
            "forwarder should return success, got: {resp}"
        );
        assert_eq!(resp["result"]["display_name"], "fwd-tcp-local");
        assert_eq!(resp["result"]["state"], "ready");

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        session_handle.abort();
    }

    /// T-923 scope gap check: the hub rejects a forwarded write-scope method
    /// BEFORE reaching forward_to_target when the connection only holds
    /// Observe scope. This proves forwarded calls are not a scope bypass.
    #[tokio::test]
    async fn tcp_forward_rejected_when_scope_insufficient() {
        use tokio::io::AsyncWriteExt;

        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (session_handle, _reg) =
            start_test_session(&sessions_dir, "fwd-scope-target").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (hub_handle, shutdown_tx, _hub_socket, tcp_port, secret_hex) =
            start_hub_with_tcp(&dir).await;

        let (mut lines, mut writer) = tcp_connect_and_auth(
            tcp_port,
            &secret_hex,
            termlink_session::auth::PermissionScope::Observe,
        )
        .await;

        // kv.set requires Interact — connection only has Observe. The hub
        // must deny the call at the scope gate and MUST NOT forward.
        let req = json!({
            "jsonrpc": "2.0",
            "method": "kv.set",
            "id": "scope-1",
            "params": {
                "target": "fwd-scope-target",
                "key": "k",
                "value": "v"
            }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(
            resp["error"]["code"].as_i64().unwrap_or(0),
            -32010,
            "Observe scope must not reach kv.set forwarder: {resp}"
        );
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap_or("")
                .contains("Permission denied"),
            "expected permission-denied message, got: {resp}"
        );

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        shutdown_tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
        session_handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_discovers_and_forwards() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a session that will be our "specialist"
        let (handle, _reg) = start_test_session(&sessions_dir, "specialist-a").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route a ping to any session matching the name
        let params = json!({
            "selector": { "name": "specialist" },
            "method": "termlink.ping",
            "params": {},
        });

        let resp = handle_orchestrator_route(json!("orch-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["routed_to"]["display_name"], "specialist-a");
            assert_eq!(r.result["candidates"], 1);
            // The forwarded ping should return the session info
            assert_eq!(r.result["result"]["display_name"], "specialist-a");
        } else {
            panic!("Expected success, got error");
        }

        handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_no_match_returns_error() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "selector": { "name": "nonexistent" },
            "method": "termlink.ping",
            "params": {},
        });

        let resp = handle_orchestrator_route(json!("orch-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Error(e) = resp {
            assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            assert!(e.error.message.contains("No sessions match"));
        } else {
            panic!("Expected error for no matching sessions");
        }
    }

    #[tokio::test]
    async fn orchestrator_route_transport_failure_tracked_in_bypass() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start the dead session first (lower created_at, sorted first by list_sessions)
        let (dead_handle, _dead_reg) =
            start_test_session(&sessions_dir, "dead-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Start the live session second
        let (live_handle, _live_reg) =
            start_test_session(&sessions_dir, "live-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Kill the dead session's listener but leave socket file intact.
        // Socket file + our PID = passes liveness check, but connect will fail.
        dead_handle.abort();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
            "timeout_secs": 1,
        });

        let resp = handle_orchestrator_route(json!("orch-transport-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        // Should succeed via the live specialist (failover from dead)
        match &resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["routed_to"]["display_name"], "live-specialist");
            }
            RpcResponse::Error(e) => {
                panic!("Expected success via failover, got error: {}", e.error.message);
            }
        }

        // Check bypass registry — infra failures (connection to dead session) should be
        // invisible. Only the success from the live session should be recorded.
        let reg_path = dir.join("bypass-registry.json");
        let bypass_reg = crate::bypass::BypassRegistry::load_from(&reg_path);

        let stats = bypass_reg.candidates.get("termlink.ping");
        assert!(
            stats.is_some(),
            "termlink.ping should be tracked in bypass candidates"
        );
        let stats = stats.unwrap();
        assert_eq!(
            stats.fail_count, 0,
            "Infra failures should NOT count against fail_count, got {}",
            stats.fail_count
        );
        assert_eq!(
            stats.success_count, 1,
            "Should have 1 success from live specialist"
        );

        live_handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_mutating_skips_bypass_tracking() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (handle, _reg) = start_test_session(&sessions_dir, "specialist-mut").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route 6 times with mutating=true (use termlink.ping — test sessions handle it)
        for i in 0..6 {
            let params = json!({
                "selector": { "name": "specialist" },
                "method": "termlink.ping",
                "params": {},
                "mutating": true,
            });
            let resp =
                handle_orchestrator_route(json!(format!("mut-{i}")), &params).await;
            assert!(
                matches!(resp, RpcResponse::Success(_)),
                "Mutating route should succeed"
            );
        }

        // Check bypass registry — should NOT have tracked termlink.ping
        let reg_path = dir.join("bypass-registry.json");
        let bypass_reg = crate::bypass::BypassRegistry::load_from(&reg_path);
        assert!(
            !bypass_reg.candidates.contains_key("termlink.ping"),
            "Mutating command should NOT be tracked in bypass candidates"
        );
        assert!(
            !bypass_reg.entries.contains_key("termlink.ping"),
            "Mutating command should NOT be promoted to bypass"
        );

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_non_mutating_promotes_normally() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (handle, _reg) = start_test_session(&sessions_dir, "specialist-nm").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route 5 times without mutating flag (default = false)
        for i in 0..5 {
            let params = json!({
                "selector": { "name": "specialist" },
                "method": "termlink.ping",
                "params": {},
            });
            let resp =
                handle_orchestrator_route(json!(format!("nm-{i}")), &params).await;
            assert!(matches!(resp, RpcResponse::Success(_)));
        }

        // Should be promoted after 5 successes
        let reg_path = dir.join("bypass-registry.json");
        let bypass_reg = crate::bypass::BypassRegistry::load_from(&reg_path);
        assert!(
            bypass_reg.entries.contains_key("termlink.ping"),
            "Non-mutating command should be promoted to bypass after 5 runs"
        );

        // 6th call should return bypassed=true
        let params = json!({
            "selector": { "name": "specialist" },
            "method": "termlink.ping",
            "params": {},
        });
        let resp = handle_orchestrator_route(json!("nm-bypass"), &params).await;
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["bypassed"], true);
        } else {
            panic!("Expected bypass response");
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        handle.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_task_type_prefers_tagged_specialist() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a generic specialist (no task-type tag)
        let (h_generic, _) = start_test_session(&sessions_dir, "generic-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Start a build-specialist (with task-type:build tag)
        let (h_build, _) = start_test_session_with_tags(
            &sessions_dir,
            "build-specialist",
            vec!["task-type:build".into()],
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route with task_type=build — should prefer build-specialist
        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
            "task_type": "build",
        });

        let resp = handle_orchestrator_route(json!("tt-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(
                r.result["routed_to"]["display_name"], "build-specialist",
                "Task-type routing should prefer the tagged specialist"
            );
            assert_eq!(r.result["candidates"], 2);
        } else {
            panic!("Expected success, got error");
        }

        h_generic.abort();
        h_build.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_task_type_falls_back_when_no_match() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a test-specialist (tagged for test, not audit)
        let (h_test, _) = start_test_session_with_tags(
            &sessions_dir,
            "test-specialist",
            vec!["task-type:test".into()],
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route with task_type=audit — no specialist has that tag, should fall back
        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
            "task_type": "audit",
        });

        let resp = handle_orchestrator_route(json!("tt-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(
                r.result["routed_to"]["display_name"], "test-specialist",
                "Should fall back to available specialist when no task-type match"
            );
            assert_eq!(r.result["candidates"], 1);
        } else {
            panic!("Expected success via fallback, got error");
        }

        h_test.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_no_task_type_backward_compatible() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Start a generic and a tagged specialist
        let (h_generic, _) = start_test_session(&sessions_dir, "generic-specialist").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let (h_build, _) = start_test_session_with_tags(
            &sessions_dir,
            "build-specialist",
            vec!["task-type:build".into()],
        )
        .await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Route WITHOUT task_type — should succeed with both candidates available
        let params = json!({
            "selector": {},
            "method": "termlink.ping",
            "params": {},
        });

        let resp = handle_orchestrator_route(json!("tt-3"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            // Without task_type, routing succeeds and both candidates are visible
            assert_eq!(r.result["candidates"], 2);
            // Any specialist is fine — the key is routing works without task_type
            let name = r.result["routed_to"]["display_name"].as_str().unwrap();
            assert!(
                name == "generic-specialist" || name == "build-specialist",
                "Expected one of the specialists, got {name}"
            );
        } else {
            panic!("Expected success, got error");
        }

        h_generic.abort();
        h_build.abort();
    }

    #[tokio::test]
    async fn orchestrator_route_missing_method_returns_error() {
        let params = json!({
            "selector": { "name": "anything" },
        });

        let resp = handle_orchestrator_route(json!("orch-3"), &params).await;

        if let RpcResponse::Error(e) = resp {
            assert_eq!(e.error.code, -32602);
            assert!(e.error.message.contains("Missing 'method'"));
        } else {
            panic!("Expected error for missing method");
        }
    }

    // === event.emit_to tests ===

    #[tokio::test]
    async fn emit_to_pushes_event_to_target() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "emit-to-target").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "target": r1.id.as_str(),
            "topic": "task.result",
            "payload": {"status": "done", "output": "42"},
        });

        let resp = handle_event_emit_to(json!("eto-1"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["topic"], "task.result");
            assert_eq!(r.result["target"], r1.id.as_str());
            assert!(r.result["seq"].as_u64().is_some());
        } else {
            panic!("Expected success response, got: {resp:?}");
        }

        // Verify event landed on target
        let resp = client::rpc_call(r1.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "task.result");
        assert_eq!(events[0]["payload"]["status"], "done");

        h1.abort();
    }

    #[tokio::test]
    async fn emit_to_enriches_with_sender() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let (h1, r1) = start_test_session(&sessions_dir, "emit-to-sender").await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "target": r1.id.as_str(),
            "topic": "negotiate.offer",
            "payload": {"format": "json"},
            "from": "worker-1",
        });

        let resp = handle_event_emit_to(json!("eto-2"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["from"], "worker-1");
        } else {
            panic!("Expected success response, got: {resp:?}");
        }

        // Verify sender info is in the event payload
        let resp = client::rpc_call(r1.socket_path(), "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events[0]["payload"]["_from"], "worker-1");
        assert_eq!(events[0]["payload"]["format"], "json");

        h1.abort();
    }

    #[tokio::test]
    async fn emit_to_unknown_target_returns_error() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(dir.join("sessions")).unwrap();

        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let params = json!({
            "target": "nonexistent-session",
            "topic": "test.ping",
        });

        let resp = handle_event_emit_to(json!("eto-3"), &params).await;

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

        if let RpcResponse::Error(e) = resp {
            assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            assert!(e.error.message.contains("nonexistent-session"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn emit_to_missing_params_returns_error() {
        // Missing target
        let params = json!({"topic": "test"});
        let resp = handle_event_emit_to(json!("eto-4a"), &params).await;
        if let RpcResponse::Error(e) = resp {
            assert!(e.error.message.contains("target"));
        } else {
            panic!("Expected error for missing target");
        }

        // Missing topic
        let params = json!({"target": "some-session"});
        let resp = handle_event_emit_to(json!("eto-4b"), &params).await;
        if let RpcResponse::Error(e) = resp {
            assert!(e.error.message.contains("topic"));
        } else {
            panic!("Expected error for missing topic");
        }
    }

    // --- extract_string_array tests ---

    #[test]
    fn extract_string_array_with_strings() {
        let params = json!({"tags": ["alpha", "beta", "gamma"]});
        let result = extract_string_array(&params, "tags");
        assert_eq!(result, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn extract_string_array_missing_key() {
        let params = json!({"other": "value"});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_null_value() {
        let params = json!({"tags": null});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_non_array() {
        let params = json!({"tags": "single-string"});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_mixed_types() {
        // Non-string elements should be filtered out
        let params = json!({"items": ["valid", 42, "also-valid", true, null]});
        let result = extract_string_array(&params, "items");
        assert_eq!(result, vec!["valid", "also-valid"]);
    }

    #[test]
    fn extract_string_array_empty_array() {
        let params = json!({"tags": []});
        let result = extract_string_array(&params, "tags");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_string_array_empty_params() {
        let params = json!({});
        let result = extract_string_array(&params, "anything");
        assert!(result.is_empty());
    }

    // === Inbox RPC Tests (T-1000) ===

    #[tokio::test]
    async fn inbox_status_returns_empty_when_no_transfers() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_inbox_status(json!(1));
        match resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["total_transfers"], 0);
                let targets = r.result["targets"].as_array().unwrap();
                assert!(targets.is_empty());
            }
            RpcResponse::Error(e) => panic!("Expected success, got error: {}", e.error.message),
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inbox_list_requires_target_param() {
        let resp = handle_inbox_list(json!(1), &json!({}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("target"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing target"),
        }
    }

    #[tokio::test]
    async fn inbox_list_returns_empty_for_unknown_target() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let resp = handle_inbox_list(json!(1), &json!({"target": "nonexistent"}));
        match resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["target"], "nonexistent");
                let transfers = r.result["transfers"].as_array().unwrap();
                assert!(transfers.is_empty());
            }
            RpcResponse::Error(e) => panic!("Expected success, got error: {}", e.error.message),
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn inbox_status_reflects_deposited_files() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Deposit a file event into the inbox
        let deposited = crate::inbox::deposit(
            "test-target",
            "file.init",
            &json!({"transfer_id": "xfer-test-1", "filename": "test.txt", "size": 100}),
            Some("sender"),
        );
        assert!(deposited.unwrap_or(false), "Deposit should succeed");

        // Now check status
        let resp = handle_inbox_status(json!(1));
        match resp {
            RpcResponse::Success(r) => {
                assert!(r.result["total_transfers"].as_u64().unwrap() > 0);
                let targets = r.result["targets"].as_array().unwrap();
                assert!(!targets.is_empty());
                assert_eq!(targets[0]["target"], "test-target");
            }
            RpcResponse::Error(e) => panic!("Expected success, got error: {}", e.error.message),
        }

        // And list for that target
        let resp = handle_inbox_list(json!(2), &json!({"target": "test-target"}));
        match resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["target"], "test-target");
                let transfers = r.result["transfers"].as_array().unwrap();
                assert!(!transfers.is_empty());
            }
            RpcResponse::Error(e) => panic!("Expected success, got error: {}", e.error.message),
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&dir);
    }

    // === remote session lifecycle error-path tests (T-1007) ===

    #[test]
    fn heartbeat_missing_id_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_heartbeat(json!(1), &json!({}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("Missing"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing id"),
        }
    }

    #[test]
    fn heartbeat_nonexistent_session_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_heartbeat(json!(1), &json!({"id": "tl-tcp-nonexistent"}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            }
            RpcResponse::Success(_) => panic!("Expected error for nonexistent session"),
        }
    }

    #[test]
    fn deregister_remote_missing_id_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_deregister_remote(json!(1), &json!({}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("Missing"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing id"),
        }
    }

    #[test]
    fn deregister_remote_nonexistent_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_deregister_remote(json!(1), &json!({"id": "tl-tcp-ghost"}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, control::error_code::SESSION_NOT_FOUND);
            }
            RpcResponse::Success(_) => panic!("Expected error for nonexistent session"),
        }
    }

    #[test]
    fn register_remote_missing_host_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_register_remote(json!(1), &json!({"port": 9001}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("host"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing host"),
        }
    }

    #[test]
    fn register_remote_missing_port_returns_error() {
        let _ = super::init_remote_store();
        let resp = super::handle_register_remote(json!(1), &json!({"host": "192.168.1.1"}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
                assert!(e.error.message.contains("port"));
            }
            RpcResponse::Success(_) => panic!("Expected error for missing port"),
        }
    }

    #[tokio::test]
    async fn hub_subscribe_returns_events_structure() {
        super::init_aggregator();
        let params = json!({"timeout_ms": 100});
        let resp = super::handle_hub_subscribe(json!(1), &params).await;
        match resp {
            RpcResponse::Success(r) => {
                assert!(r.result["events"].is_array());
                assert_eq!(r.result["count"], 0);
                assert!(r.result["sessions"].is_number());
            }
            RpcResponse::Error(e) => panic!("Expected success: {}", e.error.message),
        }
    }

    // === inbox.clear RPC tests (T-1005) ===

    #[test]
    fn inbox_clear_requires_target_or_all() {
        let resp = handle_inbox_clear(json!(1), &json!({}));
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, -32602);
            }
            RpcResponse::Success(_) => panic!("Expected error for missing params"),
        }
    }

    #[tokio::test]
    async fn inbox_clear_target_removes_transfers() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Deposit
        crate::inbox::deposit("clear-me", "file.init", &json!({"transfer_id": "x1"}), Some("s"));

        // Clear
        let resp = handle_inbox_clear(json!(1), &json!({"target": "clear-me"}));
        match resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["ok"], true);
                assert_eq!(r.result["target"], "clear-me");
            }
            RpcResponse::Error(e) => panic!("Expected success: {}", e.error.message),
        }

        // Verify empty
        let resp = handle_inbox_status(json!(2));
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["total_transfers"], 0);
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn inbox_clear_all_removes_everything() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        // Deposit to two targets
        crate::inbox::deposit("t1", "file.init", &json!({"transfer_id": "a"}), Some("s"));
        crate::inbox::deposit("t2", "file.init", &json!({"transfer_id": "b"}), Some("s"));

        // Clear all
        let resp = handle_inbox_clear(json!(1), &json!({"all": true}));
        match resp {
            RpcResponse::Success(r) => {
                assert_eq!(r.result["ok"], true);
                assert_eq!(r.result["target"], "*");
            }
            RpcResponse::Error(e) => panic!("Expected success: {}", e.error.message),
        }

        // Verify empty
        let resp = handle_inbox_status(json!(2));
        if let RpcResponse::Success(r) = resp {
            assert_eq!(r.result["total_transfers"], 0);
        }

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
        let _ = std::fs::remove_dir_all(&dir);
    }
}
