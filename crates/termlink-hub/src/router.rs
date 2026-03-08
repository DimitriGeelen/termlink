use serde_json::json;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use termlink_session::client;
use termlink_session::manager;

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
        control::method::SESSION_DISCOVER => handle_discover(id).await,
        control::method::EVENT_BROADCAST => handle_event_broadcast(id, &req.params).await,
        control::method::EVENT_COLLECT => handle_event_collect(id, &req.params).await,
        _ => forward_to_target(req, id).await,
    };

    Some(response)
}

/// Handle `session.discover` — list all registered sessions.
async fn handle_discover(id: serde_json::Value) -> RpcResponse {
    match manager::list_sessions(false) {
        Ok(sessions) => {
            let entries: Vec<serde_json::Value> = sessions
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id.as_str(),
                        "display_name": s.display_name,
                        "state": s.state,
                        "capabilities": s.capabilities,
                        "roles": s.roles,
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

    let mut succeeded = 0u64;
    let mut failed = 0u64;

    for reg in &registrations {
        let emit_params = json!({
            "topic": topic,
            "payload": payload,
        });

        match client::rpc_call(&reg.socket, control::method::EVENT_EMIT, emit_params).await {
            Ok(resp) => {
                if client::unwrap_result(resp).is_ok() {
                    succeeded += 1;
                } else {
                    failed += 1;
                }
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    Response::success(
        id,
        json!({
            "topic": topic,
            "targeted": registrations.len(),
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

    let mut all_events: Vec<serde_json::Value> = Vec::new();
    let mut cursors = json!({});

    for reg in &registrations {
        let sid = reg.id.as_str();

        let mut poll_params = json!({});
        if let Some(seq_val) = since_map.get(sid) {
            poll_params["since"] = seq_val.clone();
        }
        if let Some(t) = topic_filter {
            poll_params["topic"] = json!(t);
        }

        match client::rpc_call(&reg.socket, control::method::EVENT_POLL, poll_params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let mut enriched = event.clone();
                            enriched["session"] = json!(sid);
                            enriched["session_name"] = json!(&reg.display_name);
                            all_events.push(enriched);
                        }
                    }
                    if let Some(next) = result.get("next_seq") {
                        cursors[sid] = next.clone();
                    }
                }
            }
            Err(_) => {
                tracing::debug!(session = sid, "Collect: failed to reach session");
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
        let mut c = client::Client::connect(&reg.socket).await?;
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
        .map(|r| r.socket)
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

        let reg = session.registration.clone();
        let ctx = SessionContext::new(session.registration);
        let shared = Arc::new(RwLock::new(ctx));
        let listener = session.listener;

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
            &reg.socket,
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
        let resp = client::rpc_call(&r1.socket, "event.poll", json!({})).await.unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "deploy.start");

        let resp = client::rpc_call(&r2.socket, "event.poll", json!({})).await.unwrap();
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
        let resp = client::rpc_call(&r2.socket, "event.poll", json!({})).await.unwrap();
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
            &r1.socket,
            "event.emit",
            json!({"topic": "build.done", "payload": {"id": 1}}),
        ).await.unwrap();

        client::rpc_call(
            &r2.socket,
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
            &r1.socket,
            "event.emit",
            json!({"topic": "a", "payload": {}}),
        ).await.unwrap();
        client::rpc_call(
            &r1.socket,
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
