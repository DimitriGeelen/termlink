use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::RwLock;

use termlink_protocol::jsonrpc::{ErrorResponse, Request, RpcResponse};

use crate::auth::{self, PeerCredentials, PermissionScope};
use crate::handler::{self, SessionContext};

/// Shared session state accessible by connection handlers.
pub type SharedSession = Arc<RwLock<SessionContext>>;

/// Handle a single client connection on the control plane socket.
///
/// Reads newline-delimited JSON-RPC requests, checks per-method permission scope,
/// dispatches authorized requests, and writes newline-delimited JSON-RPC responses.
pub async fn handle_connection(
    stream: UnixStream,
    session: SharedSession,
    granted_scope: PermissionScope,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => {
                // Check permission scope before dispatching
                let required = auth::method_scope(&req.method);
                if !granted_scope.satisfies(required) {
                    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
                    tracing::warn!(
                        method = %req.method,
                        required = %required,
                        granted = %granted_scope,
                        "Permission denied: insufficient scope"
                    );
                    Some(
                        ErrorResponse::new(
                            id,
                            -32603,
                            &format!(
                                "Permission denied: method '{}' requires '{}' scope, connection has '{}'",
                                req.method, required, granted_scope
                            ),
                        )
                        .into(),
                    )
                } else if handler::needs_write(&req) {
                    let mut ctx = session.write().await;
                    handler::dispatch_mut(&req, &mut ctx).await
                } else {
                    let ctx = session.read().await;
                    handler::dispatch(&req, &ctx).await
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to parse JSON-RPC request");
                Some(ErrorResponse::parse_error().into())
            }
        };

        // Send response (if not a notification)
        if let Some(resp) = response {
            let mut json = serde_json::to_string(&resp).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to serialize response");
                let err: RpcResponse = ErrorResponse::internal_error(
                    serde_json::Value::Null,
                    "serialization error",
                )
                .into();
                serde_json::to_string(&err).unwrap()
            });
            json.push('\n');

            if let Err(e) = writer.write_all(json.as_bytes()).await {
                tracing::debug!(error = %e, "Failed to write response, client disconnected");
                break;
            }
        }
    }
}

/// Run the session accept loop, spawning a task for each connection.
///
/// This is the main entry point for a session's control plane server.
/// It runs until the listener is dropped or an unrecoverable error occurs.
pub async fn run_accept_loop(
    listener: tokio::net::UnixListener,
    session: SharedSession,
) {
    // Cache the session owner UID for auth checks
    let owner_uid = {
        let ctx = session.read().await;
        ctx.registration.uid
    };

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                // Extract peer credentials and check UID
                match PeerCredentials::from_tokio_stream(&stream) {
                    Ok(creds) => {
                        if !creds.is_same_user(owner_uid) {
                            tracing::warn!(
                                peer_uid = creds.uid,
                                peer_pid = ?creds.pid,
                                owner_uid = owner_uid,
                                "Rejected connection from different UID"
                            );
                            // Drop the stream — connection closed
                            continue;
                        }
                        tracing::trace!(
                            peer_uid = creds.uid,
                            peer_pid = ?creds.pid,
                            "Accepted authenticated connection"
                        );
                    }
                    Err(e) => {
                        // If credential extraction fails, allow the connection
                        // (graceful degradation on unsupported platforms)
                        tracing::debug!(
                            error = %e,
                            "Could not extract peer credentials, allowing connection"
                        );
                    }
                }

                // Same-UID connections get full access (Execute scope).
                // Future: capability tokens (T-079) will allow scoped access.
                let scope = PermissionScope::Execute;

                let sess = session.clone();
                tokio::spawn(async move {
                    handle_connection(stream, sess, scope).await;
                });
            }
            Err(e) => {
                tracing::error!(error = %e, "Accept failed");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::SessionId;
    use crate::lifecycle::SessionState;
    use crate::registration::{Registration, SessionConfig};
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_socket_path() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(format!("/tmp/tl-srv-{}-{}.sock", std::process::id(), n))
    }

    fn test_session(socket: PathBuf) -> SessionContext {
        let id = SessionId::generate();
        let mut reg = Registration::new(
            id,
            SessionConfig {
                display_name: Some("server-test".into()),
                capabilities: vec!["inject".into(), "query".into()],
                roles: vec![],
                tags: vec![],
            },
            socket,
        );
        reg.state = SessionState::Ready;
        SessionContext::new(reg)
    }

    #[tokio::test]
    async fn end_to_end_ping() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        // Spawn accept loop
        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            run_accept_loop(listener, shared_clone).await;
        });

        // Give server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Connect as client
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Send ping request
        let req = json!({
            "jsonrpc": "2.0",
            "method": "termlink.ping",
            "id": "test-1",
            "params": {}
        });
        let mut msg = serde_json::to_string(&req).unwrap();
        msg.push('\n');
        writer.write_all(msg.as_bytes()).await.unwrap();

        // Read response
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], "test-1");
        assert!(resp["result"]["id"].is_string());
        assert_eq!(resp["result"]["state"], "ready");
        assert_eq!(resp["result"]["display_name"], "server-test");

        // Cleanup
        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn end_to_end_multiple_requests() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            run_accept_loop(listener, shared_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Request 1: query.status
        let req1 = json!({"jsonrpc": "2.0", "method": "query.status", "id": 1, "params": {}});
        writer.write_all(format!("{}\n", req1).as_bytes()).await.unwrap();
        let resp1: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp1["id"], 1);
        assert!(resp1["result"]["pid"].is_number());

        // Request 2: query.capabilities
        let req2 = json!({"jsonrpc": "2.0", "method": "query.capabilities", "id": 2, "params": {}});
        writer.write_all(format!("{}\n", req2).as_bytes()).await.unwrap();
        let resp2: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp2["id"], 2);
        let caps = resp2["result"]["capabilities"].as_array().unwrap();
        assert!(caps.contains(&json!("inject")));

        // Request 3: unknown method
        let req3 = json!({"jsonrpc": "2.0", "method": "foo.bar", "id": 3, "params": {}});
        writer.write_all(format!("{}\n", req3).as_bytes()).await.unwrap();
        let resp3: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp3["id"], 3);
        assert_eq!(resp3["error"]["code"], -32601);

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn malformed_json_returns_parse_error() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            run_accept_loop(listener, shared_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Send malformed JSON
        writer.write_all(b"this is not json\n").await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32700); // Parse error

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn notification_gets_no_response() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            run_accept_loop(listener, shared_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Send notification (no id)
        let notif = json!({"jsonrpc": "2.0", "method": "event.state_change", "params": {"state": "busy"}});
        writer.write_all(format!("{}\n", notif).as_bytes()).await.unwrap();

        // Send a request after to verify the connection is still alive
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "after-notif", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();

        // We should get the ping response (not a response to the notification)
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "after-notif");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn permission_scope_denies_execute_for_observe_connection() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        // Spawn handler with Observe-only scope (not the accept loop)
        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared_clone, PermissionScope::Observe).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Observe-scoped: ping should work (Observe tier)
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "p1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "p1");
        assert!(resp["result"]["id"].is_string(), "Ping should succeed with Observe scope");

        // Observe-scoped: command.execute should be denied (Execute tier)
        let req = json!({"jsonrpc": "2.0", "method": "command.execute", "id": "e1", "params": {"command": "echo hi"}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "e1");
        assert_eq!(resp["error"]["code"], -32603, "Execute should be denied");
        assert!(resp["error"]["message"].as_str().unwrap().contains("Permission denied"));

        // Observe-scoped: command.inject should be denied (Control tier)
        let req = json!({"jsonrpc": "2.0", "method": "command.inject", "id": "i1", "params": {"text": "ls"}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "i1");
        assert_eq!(resp["error"]["code"], -32603, "Inject should be denied");

        // Observe-scoped: event.emit should be denied (Interact tier)
        let req = json!({"jsonrpc": "2.0", "method": "event.emit", "id": "em1", "params": {"topic": "test", "payload": {}}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "em1");
        assert_eq!(resp["error"]["code"], -32603, "Emit should be denied");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn permission_scope_allows_all_for_execute_connection() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        // Spawn handler with full Execute scope
        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared_clone, PermissionScope::Execute).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Execute scope: ping should work
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "p1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"]["id"].is_string(), "Ping should work with Execute scope");

        // Execute scope: event.emit should work (Interact tier, satisfied by Execute)
        let req = json!({"jsonrpc": "2.0", "method": "event.emit", "id": "em1", "params": {"topic": "test", "payload": {}}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"].is_object(), "Emit should work with Execute scope");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }
}
