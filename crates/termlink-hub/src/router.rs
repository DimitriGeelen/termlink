use std::sync::OnceLock;
use std::time::Duration;

use serde_json::json;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};
use termlink_protocol::TransportAddr;

use termlink_session::client;
use termlink_session::manager;

use crate::remote_store::RemoteStore;

/// Per-target timeout for broadcast/collect operations.
const PER_TARGET_TIMEOUT: Duration = Duration::from_secs(5);

/// Global remote session store (initialized once by the hub server).
static REMOTE_STORE: OnceLock<RemoteStore> = OnceLock::new();

/// Initialize the global remote store. Called once by the hub server.
pub fn init_remote_store() -> RemoteStore {
    let store = RemoteStore::new();
    let _ = REMOTE_STORE.set(store.clone());
    store
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
        control::method::ORCHESTRATOR_ROUTE => handle_orchestrator_route(id, &req.params).await,
        "session.register_remote" => handle_register_remote(id, &req.params),
        "session.heartbeat" => handle_heartbeat(id, &req.params),
        "session.deregister_remote" => handle_deregister_remote(id, &req.params),
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

    let topic_filter = params.get("topic").and_then(|t| t.as_str());

    // Dispatch polls concurrently with per-target timeout
    let mut join_set = tokio::task::JoinSet::new();

    for reg in registrations {
        let sid = reg.id.to_string();
        let display_name = reg.display_name.clone();
        let addr = reg.addr.to_transport_addr();
        let since_map = since_map.clone();
        let topic_filter = topic_filter.map(String::from);

        join_set.spawn(async move {
            let mut poll_params = json!({});
            if let Some(seq_val) = since_map.get(&sid) {
                poll_params["since"] = seq_val.clone();
            }
            if let Some(t) = &topic_filter {
                poll_params["topic"] = json!(t);
            }

            let result = tokio::time::timeout(
                PER_TARGET_TIMEOUT,
                client::rpc_call_addr(&addr, control::method::EVENT_POLL, poll_params),
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

    let session_id = store.register(display_name, host, port, pid, roles, tags, capabilities);
    tracing::info!(id = %session_id, "Remote session registered");

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
/// Params: {
///   selector: { tags?: [...], roles?: [...], capabilities?: [...], name?: string },
///   method: string,       // RPC method to call on the specialist
///   params: object,       // params to pass to the specialist
///   timeout_secs?: number // per-target timeout (default: 5)
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

    // Try candidates in order (failover)
    let mut last_error = String::new();
    for reg in candidates.drain(..) {
        let addr = reg.addr.to_transport_addr();
        let result = tokio::time::timeout(timeout, async {
            let mut c = client::Client::connect_addr(&addr).await?;
            c.call(&method, id.clone(), forward_params.clone()).await
        })
        .await;

        match result {
            Ok(Ok(RpcResponse::Success(resp))) => {
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
                last_error = format!("{}: {}", reg.display_name, e.error.message);
                tracing::debug!(
                    target = reg.display_name,
                    error = %e.error.message,
                    "orchestrator.route: candidate returned error, trying next"
                );
            }
            Ok(Err(e)) => {
                last_error = format!("{}: {}", reg.display_name, e);
                tracing::debug!(
                    target = reg.display_name,
                    error = %e,
                    "orchestrator.route: candidate connection failed, trying next"
                );
            }
            Err(_) => {
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
            "All {} candidate(s) failed. Last: {}",
            total_candidates, last_error
        ),
    )
    .into()
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
        let remote_id = store.register(
            "tcp-target".into(),
            "127.0.0.1".into(),
            tcp_port,
            None,
            vec![],
            vec![],
            vec![],
        );

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
}
