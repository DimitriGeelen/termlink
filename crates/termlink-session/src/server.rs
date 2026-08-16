use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::RwLock;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};

use crate::auth::{self, PeerCredentials, PermissionScope};
use crate::handler::{self, SessionContext};

/// Shared session state accessible by connection handlers.
pub type SharedSession = Arc<RwLock<SessionContext>>;

/// Handle a single client connection on the control plane socket.
///
/// Reads newline-delimited JSON-RPC requests, checks per-method permission scope,
/// dispatches authorized requests, and writes newline-delimited JSON-RPC responses.
///
/// If the session has a `token_secret`, the initial scope is `Observe` and clients
/// must authenticate via `auth.token` to upgrade their scope. Without a `token_secret`,
/// same-UID connections get `Execute` scope (legacy behavior).
pub async fn handle_connection(
    stream: UnixStream,
    session: SharedSession,
    initial_scope: PermissionScope,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let mut granted_scope = initial_scope;

    // Read token_secret from session registration (for auth.token validation)
    let token_secret = {
        let ctx = session.read().await;
        ctx.registration.token_secret.clone()
    };

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => {
                // Handle auth.token specially — upgrades connection scope
                if req.method == control::method::AUTH_TOKEN {
                    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
                    handle_auth_token(&req, &token_secret, &mut granted_scope, id)
                } else {
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
                                control::error_code::AUTH_DENIED,
                                &format!(
                                    "Permission denied: method '{}' requires '{}' scope, connection has '{}'",
                                    req.method, required, granted_scope
                                ),
                            )
                            .into(),
                        )
                    } else {
                        // T-2521: single source of truth for session-lock scope.
                        // event.subscribe is dispatched detached from the session
                        // lock so it can't deadlock concurrent kv.set/kv.delete.
                        handler::dispatch_scoped(&session, &req).await
                    }
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

/// Handle an `auth.token` request — validate the token and upgrade connection scope.
fn handle_auth_token(
    req: &Request,
    token_secret: &Option<String>,
    granted_scope: &mut PermissionScope,
    id: serde_json::Value,
) -> Option<RpcResponse> {
    let secret = match token_secret {
        Some(s) => s,
        None => {
            // No token secret configured — auth.token is not supported
            return Some(
                ErrorResponse::new(
                    id,
                    control::error_code::AUTH_DENIED,
                    "Token authentication not configured for this session",
                )
                .into(),
            );
        }
    };

    // Decode the hex secret
    let secret_bytes: auth::TokenSecret = match hex_to_bytes(secret) {
        Some(b) => b,
        None => {
            tracing::error!("Invalid token_secret in registration (not valid hex)");
            return Some(
                ErrorResponse::internal_error(id, "Internal auth configuration error").into(),
            );
        }
    };

    // Extract the token string from params
    let token_str = match req.params.get("token").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return Some(
                ErrorResponse::new(
                    id,
                    control::error_code::AUTH_REQUIRED,
                    "Missing 'token' parameter",
                )
                .into(),
            );
        }
    };

    // Validate the token
    match auth::validate_token(&secret_bytes, token_str, None) {
        Ok((payload, scope)) => {
            *granted_scope = scope;
            tracing::info!(
                scope = %scope,
                session_id = %payload.session_id,
                "Connection authenticated via token"
            );
            Some(
                Response::success(
                    id,
                    serde_json::json!({
                        "authenticated": true,
                        "scope": scope.to_string(),
                    }),
                )
                .into(),
            )
        }
        Err(e) => {
            tracing::warn!(error = %e, "Token validation failed");
            Some(
                ErrorResponse::new(
                    id,
                    control::error_code::AUTH_DENIED,
                    &format!("Token validation failed: {e}"),
                )
                .into(),
            )
        }
    }
}

/// Convert a hex string to a 32-byte array.
fn hex_to_bytes(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

/// T-2773: Write a single `AUTH_DENIED` envelope to a just-accepted Unix stream
/// and close, instead of dropping it silently.
///
/// Generic over the writer so the refusal can be asserted against an in-memory
/// buffer — the point of the fix is what the CLIENT receives, so that is what the
/// test needs to be able to read.
async fn write_uid_refusal<S>(stream: &mut S, peer_uid: Option<u32>, owner_uid: u32)
where
    S: tokio::io::AsyncWrite + Unpin,
{
    let envelope = auth::build_uid_refusal(peer_uid, owner_uid, auth::UnixEndpoint::Session);
    if let Ok(mut line) = serde_json::to_vec(&RpcResponse::Error(envelope)) {
        line.push(b'\n');
        let _ = stream.write_all(&line).await;
        let _ = stream.shutdown().await;
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
    // Cache session owner UID and token mode for auth checks
    let (owner_uid, has_token_secret) = {
        let ctx = session.read().await;
        (ctx.registration.uid, ctx.registration.token_secret.is_some())
    };

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                // T-2773: route the same-uid gate through the ONE shared policy
                // (`auth::decide_unix_peer`), the same call the hub accept loop
                // makes. This arm used to be a private copy that failed OPEN on a
                // credential-extraction error while the hub failed closed — and
                // since a session with no `token_secret` grants `Execute` below,
                // that copy handed full command-execution scope to a peer it
                // could not identify.
                match auth::decide_unix_peer(PeerCredentials::from_tokio_stream(&stream), owner_uid)
                {
                    auth::UnixPeerDecision::Accept { peer_pid } => {
                        tracing::trace!(
                            peer_pid = ?peer_pid,
                            "Accepted authenticated connection"
                        );
                    }
                    auth::UnixPeerDecision::Reject { uid_mismatch } => {
                        match uid_mismatch {
                            Some(peer_uid) => tracing::warn!(
                                peer_uid = peer_uid,
                                owner_uid = owner_uid,
                                "Session: rejected Unix connection from different UID"
                            ),
                            None => tracing::warn!(
                                owner_uid = owner_uid,
                                "Session: rejected Unix connection — could not extract \
                                 peer credentials (fail-closed)"
                            ),
                        }
                        // T-2773: tell the REFUSED PARTY why. A bare `continue`
                        // closed the stream with nothing written, so the client
                        // saw only `Connection reset by peer (os error 104)` and
                        // could not tell a policy refusal from a crashed session.
                        // Spawned so a client that never reads cannot stall the
                        // accept loop.
                        tokio::spawn(async move {
                            write_uid_refusal(&mut stream, uid_mismatch, owner_uid).await;
                        });
                        continue;
                    }
                }

                // Scope assignment:
                // - With token_secret: default to Observe, client must auth.token to upgrade
                // - Without token_secret: legacy behavior, same-UID gets Execute
                let scope = if has_token_secret {
                    PermissionScope::Observe
                } else {
                    PermissionScope::Execute
                };

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

    // ---- T-2773: the uid gate on the session control plane ------------------
    //
    // This accept loop used to fail OPEN: a peer whose credentials could not be
    // read was ALLOWED, and — because a session with no `token_secret` is granted
    // `Execute` a few lines further down — that unidentified peer received full
    // command-execution scope. The hub had refused the same case since T-2448.
    // Two copies of one security policy, opposite verdicts.
    //
    // The tests below therefore pin two separate things: that the verdict is
    // fail-closed, and that the refused party can actually READ the refusal
    // (the T-2772 defect, which this server still carried as a bare `continue`).

    fn session_refusal_json(peer_uid: Option<u32>, owner_uid: u32) -> serde_json::Value {
        serde_json::to_value(RpcResponse::Error(auth::build_uid_refusal(
            peer_uid,
            owner_uid,
            auth::UnixEndpoint::Session,
        )))
        .unwrap()
    }

    #[test]
    fn session_uid_gate_fails_closed_when_credentials_cannot_be_read() {
        // The regression that matters: `Err` must reject, not allow. Generic over
        // the error type so this is provable without SO_PEERCRED ever failing.
        assert_eq!(
            auth::decide_unix_peer::<()>(Err(()), 1000),
            auth::UnixPeerDecision::Reject {
                uid_mismatch: None
            },
            "an unidentifiable peer must be refused, never granted Execute"
        );
    }

    #[test]
    fn session_uid_gate_matches_the_hub_for_every_input() {
        // The mechanism behind this defect was divergence between two copies of
        // the gate. There is now one function, and this pins the property that
        // makes divergence impossible: the session server reaches its verdict by
        // calling the same `auth::decide_unix_peer` the hub calls, so identical
        // inputs cannot produce different outcomes.
        let owner = 1000;
        let same = PeerCredentials { uid: owner, gid: 0, pid: Some(42) };
        let other = PeerCredentials { uid: 1001, gid: 0, pid: Some(43) };

        assert_eq!(
            auth::decide_unix_peer::<()>(Ok(same), owner),
            auth::UnixPeerDecision::Accept { peer_pid: Some(42) }
        );
        assert_eq!(
            auth::decide_unix_peer::<()>(Ok(other), owner),
            auth::UnixPeerDecision::Reject { uid_mismatch: Some(1001) }
        );
        assert_eq!(
            auth::decide_unix_peer::<()>(Err(()), owner),
            auth::UnixPeerDecision::Reject { uid_mismatch: None }
        );
    }

    #[test]
    fn session_refusal_names_the_cause_and_a_way_through() {
        let v = session_refusal_json(Some(1001), 0);
        let msg = v["error"]["message"].as_str().unwrap();

        assert!(msg.contains("1001"), "names the peer uid: {msg}");
        assert!(msg.contains("uid 0"), "names the owner uid: {msg}");
        // A session control plane is local-only, so the remediation is NOT the
        // hub's "connect over TCP" — it is to go through the hub with a token.
        assert!(
            msg.contains("capability token"),
            "carries session-appropriate remediation: {msg}"
        );
        assert_eq!(v["error"]["code"], control::error_code::AUTH_DENIED);
        assert_eq!(v["error"]["data"]["reason"], "uid_mismatch");
        assert_eq!(v["error"]["data"]["endpoint"], "session");
    }

    #[test]
    fn session_refusal_distinguishes_unreadable_credentials_from_wrong_user() {
        let v = session_refusal_json(None, 0);
        let msg = v["error"]["message"].as_str().unwrap();

        assert!(
            msg.contains("could not read peer credentials"),
            "names the real cause: {msg}"
        );
        assert!(
            msg.contains("fail-closed"),
            "says the refusal was deliberate, not a fault: {msg}"
        );
        assert_eq!(v["error"]["data"]["reason"], "peer_credentials_unavailable");
        assert!(v["error"]["data"]["peer_uid"].is_null());

        let mismatch = session_refusal_json(Some(1001), 0);
        assert_ne!(
            msg,
            mismatch["error"]["message"].as_str().unwrap(),
            "the two branches must not collapse to one message"
        );
    }

    #[tokio::test]
    async fn session_refusal_is_one_newline_terminated_line_the_client_can_frame() {
        // What the CLIENT receives is the whole point of the fix — asserting on
        // server-side logs would pass just as happily with a silent drop.
        let mut buf: Vec<u8> = Vec::new();
        write_uid_refusal(&mut buf, None, 0).await;

        assert!(!buf.is_empty(), "a refused peer must not get zero bytes");
        assert!(buf.ends_with(b"\n"), "newline-terminated");
        assert_eq!(buf.iter().filter(|b| **b == b'\n').count(), 1, "exactly one");

        let parsed: serde_json::Value =
            serde_json::from_slice(&buf[..buf.len() - 1]).expect("parses as JSON");
        assert_eq!(parsed["error"]["code"], control::error_code::AUTH_DENIED);
        assert_eq!(
            parsed["error"]["data"]["reason"],
            "peer_credentials_unavailable"
        );
    }

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
        assert_eq!(resp["error"]["code"], -32010, "Execute should be denied (AUTH_DENIED)");
        assert!(resp["error"]["message"].as_str().unwrap().contains("Permission denied"));

        // Observe-scoped: command.inject should be denied (Control tier)
        let req = json!({"jsonrpc": "2.0", "method": "command.inject", "id": "i1", "params": {"text": "ls"}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "i1");
        assert_eq!(resp["error"]["code"], -32010, "Inject should be denied (AUTH_DENIED)");

        // Observe-scoped: event.emit should be denied (Interact tier)
        let req = json!({"jsonrpc": "2.0", "method": "event.emit", "id": "em1", "params": {"topic": "test", "payload": {}}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "em1");
        assert_eq!(resp["error"]["code"], -32010, "Emit should be denied (AUTH_DENIED)");

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

    // === Token auth tests (T-087) ===

    /// Helper: create a session with token_secret enabled.
    fn test_session_with_tokens(socket: PathBuf) -> (SessionContext, auth::TokenSecret) {
        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let id = SessionId::generate();
        let mut reg = Registration::new(
            id,
            SessionConfig {
                display_name: Some("token-test".into()),
                capabilities: vec!["inject".into(), "query".into()],
                roles: vec![],
                tags: vec![],
            },
            socket,
        );
        reg.state = SessionState::Ready;
        reg.token_secret = Some(secret_hex);
        (SessionContext::new(reg), secret)
    }

    #[tokio::test]
    async fn auth_token_upgrades_scope() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let (ctx, secret) = test_session_with_tokens(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        // Spawn handler with Observe scope (token mode)
        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared_clone, PermissionScope::Observe).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Before auth: execute should be denied
        let req = json!({"jsonrpc": "2.0", "method": "command.execute", "id": "e1", "params": {"command": "echo hi"}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010, "Execute should be denied before auth");

        // Authenticate with Execute-scoped token
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "auth.token", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["result"]["authenticated"], true);
        assert_eq!(resp["result"]["scope"], "execute");

        // After auth: ping should still work
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "p1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"]["id"].is_string());

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn auth_token_with_observe_scope_allows_only_reads() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let (ctx, secret) = test_session_with_tokens(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared_clone, PermissionScope::Observe).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Authenticate with Observe-only token
        let token = auth::create_token(&secret, PermissionScope::Observe, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "auth.token", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["result"]["scope"], "observe");

        // Ping works (Observe)
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "p1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"]["id"].is_string());

        // Event.emit denied (Interact > Observe)
        let req = json!({"jsonrpc": "2.0", "method": "event.emit", "id": "em1", "params": {"topic": "test", "payload": {}}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010);

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn auth_token_wrong_secret_rejected() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let (ctx, _secret) = test_session_with_tokens(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared_clone, PermissionScope::Observe).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Create token with different secret
        let wrong_secret = auth::generate_secret();
        let token = auth::create_token(&wrong_secret, PermissionScope::Execute, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "auth.token", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010, "Wrong secret should be rejected");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn auth_token_without_secret_configured_rejected() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        // Session WITHOUT token_secret (legacy mode)
        let ctx = test_session(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(stream, shared_clone, PermissionScope::Execute).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Try to authenticate on a session that doesn't support tokens
        let secret = auth::generate_secret();
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "auth.token", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010, "Should reject when no secret configured");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn event_subscribe_over_socket() {
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

        // Emit an event in the background (use read() since EventBus is Arc<Mutex>)
        let shared_emitter = shared.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let ctx = shared_emitter.read().await;
            let mut bus = ctx.events.lock().await;
            bus.emit("e2e.test", serde_json::json!({"value": 42}));
        });

        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Subscribe with 2s timeout
        let req = json!({
            "jsonrpc": "2.0",
            "method": "event.subscribe",
            "id": "sub-e2e",
            "params": {"timeout_ms": 2000}
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();

        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["id"], "sub-e2e");
        assert_eq!(resp["result"]["count"], 1);
        let events = resp["result"]["events"].as_array().unwrap();
        assert_eq!(events[0]["topic"], "e2e.test");
        assert_eq!(events[0]["payload"]["value"], 42);

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn accept_loop_uses_observe_scope_when_token_secret_set() {
        let socket_path = test_socket_path();
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let (ctx, _secret) = test_session_with_tokens(socket_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        let shared_clone = shared.clone();
        let handle = tokio::spawn(async move {
            run_accept_loop(listener, shared_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Ping works (Observe)
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "p1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"]["id"].is_string(), "Ping should work");

        // Execute denied (no token auth yet)
        let req = json!({"jsonrpc": "2.0", "method": "command.execute", "id": "e1", "params": {"command": "echo hi"}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010, "Execute should be denied without token");

        handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }
}
