use std::time::Duration;

use serde_json::json;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use termlink_session::client;
use termlink_session::manager;

/// Per-target timeout for broadcast/collect operations.
const PER_TARGET_TIMEOUT: Duration = Duration::from_secs(5);

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

            let entries: Vec<serde_json::Value> = sessions
                .iter()
                .filter(|s| {
                    tag_filter.iter().all(|t| s.tags.contains(t))
                        && role_filter.iter().all(|r| s.roles.contains(r))
                        && cap_filter.iter().all(|c| s.capabilities.contains(c))
                        && name_filter.map_or(true, |n| {
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
        let socket = reg.socket_path().to_path_buf();

        join_set.spawn(async move {
            let result = tokio::time::timeout(
                PER_TARGET_TIMEOUT,
                client::rpc_call(&socket, control::method::EVENT_EMIT, emit_params),
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
            if let Some(name) = t.as_str() {
                if let Ok(r) = manager::find_session(name) {
                    regs.push(r);
                }
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
        let socket = reg.socket_path().to_path_buf();
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
                client::rpc_call(&socket, control::method::EVENT_POLL, poll_params),
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

    // Resolve target to a registration
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            return ErrorResponse::new(
                id,
                control::error_code::SESSION_NOT_FOUND,
                &format!("Target not found: {e}"),
            )
            .into();
        }
    };

    // Forward the request via the target's socket, preserving the original request id
    let forward_result = async {
        let mut c = client::Client::connect(reg.socket_path()).await?;
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

/// Resolve a target string to a socket path.
///
/// Public so the CLI can use direct routing without the hub.
pub fn resolve_target(target: &str) -> Result<std::path::PathBuf, String> {
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
        let _lock = ENV_LOCK.lock().unwrap();
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
        let _lock = ENV_LOCK.lock().unwrap();
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
        let _lock = ENV_LOCK.lock().unwrap();
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
        let _lock = ENV_LOCK.lock().unwrap();
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
        let _lock = ENV_LOCK.lock().unwrap();
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
}
