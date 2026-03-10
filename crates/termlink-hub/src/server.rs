use std::path::{Path, PathBuf};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use termlink_protocol::jsonrpc::{ErrorResponse, Request, RpcResponse};
use termlink_session::auth::PeerCredentials;
use termlink_session::discovery;

use crate::router;

/// Return the well-known hub socket path: `runtime_dir()/hub.sock`.
pub fn hub_socket_path() -> PathBuf {
    discovery::runtime_dir().join("hub.sock")
}

/// Start the hub server, binding to the given socket path.
///
/// The hub accepts JSON-RPC connections and routes requests via [`router::route`]:
/// - `session.discover` is handled locally (lists all sessions)
/// - All other methods are forwarded to the target session specified in params.target
pub async fn run(socket_path: &Path) -> std::io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    tracing::info!(path = %socket_path.display(), "Hub listening");

    run_accept_loop(listener).await;
    Ok(())
}

/// Accept loop: spawns a task per connection.
///
/// Rejects connections from different UIDs (same security model as session server).
pub async fn run_accept_loop(listener: UnixListener) {
    let owner_uid = unsafe { libc::getuid() };

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
                                "Hub: rejected connection from different UID"
                            );
                            continue;
                        }
                    }
                    Err(e) => {
                        tracing::debug!(
                            error = %e,
                            "Hub: could not extract peer credentials, allowing connection"
                        );
                    }
                }

                tokio::spawn(async move {
                    handle_connection(stream).await;
                });
            }
            Err(e) => {
                tracing::error!(error = %e, "Hub accept failed");
                break;
            }
        }
    }
}

/// Handle a single hub client connection.
///
/// Reads newline-delimited JSON-RPC, routes via [`router::route`],
/// writes newline-delimited JSON-RPC responses.
async fn handle_connection(stream: UnixStream) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => router::route(&req).await,
            Err(e) => {
                tracing::warn!(error = %e, "Hub: failed to parse JSON-RPC request");
                Some(ErrorResponse::parse_error().into())
            }
        };

        if let Some(resp) = response {
            let mut json = serde_json::to_string(&resp).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Hub: failed to serialize response");
                let err: RpcResponse = ErrorResponse::internal_error(
                    serde_json::Value::Null,
                    "serialization error",
                )
                .into();
                serde_json::to_string(&err).unwrap()
            });
            json.push('\n');

            if let Err(e) = writer.write_all(json.as_bytes()).await {
                tracing::debug!(error = %e, "Hub: client disconnected");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::sync::RwLock;

    use termlink_session::handler::SessionContext;
    use termlink_session::manager::Session;
    use termlink_session::registration::SessionConfig;
    use termlink_session::server as session_server;

    use crate::test_util::ENV_LOCK;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-hubsrv-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hub_sock(dir: &Path) -> PathBuf {
        dir.join("hub.sock")
    }

    /// Start a session in the given directory, return its handle and registration.
    async fn start_session(
        sessions_dir: &Path,
        name: &str,
    ) -> (
        tokio::task::JoinHandle<()>,
        termlink_session::Registration,
    ) {
        let config = SessionConfig {
            display_name: Some(name.into()),
            ..Default::default()
        };
        let session = Session::register_in(config, sessions_dir).await.unwrap();
        let (registration, listener, _) = session.into_parts();
        let reg = registration.clone();
        let ctx = SessionContext::new(registration);
        let shared = Arc::new(RwLock::new(ctx));

        let handle = tokio::spawn(async move {
            session_server::run_accept_loop(listener, shared).await;
        });

        // Give it a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        (handle, reg)
    }

    /// Start the hub server on the given socket, return its handle.
    fn start_hub(socket: PathBuf) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            run(&socket).await.unwrap();
        })
    }

    /// Tests discover + forward in a single test to avoid env var races.
    /// Both require TERMLINK_RUNTIME_DIR to point to the test directory.
    #[tokio::test]
    async fn hub_discover_and_forward() {
        let _lock = ENV_LOCK.lock().unwrap();

        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Override sessions dir so router::route → manager finds sessions
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (h1, _) = start_session(&sessions_dir, "hub-test-a").await;
        let (h2, reg_b) = start_session(&sessions_dir, "hub-test-b").await;

        let hub_socket = hub_sock(&dir);
        let hub_handle = start_hub(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // 1. Discover — should list both sessions
        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "d-1",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], "d-1");
        let sessions = resp["result"]["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 2);

        let names: Vec<&str> = sessions
            .iter()
            .filter_map(|s| s["display_name"].as_str())
            .collect();
        assert!(names.contains(&"hub-test-a"));
        assert!(names.contains(&"hub-test-b"));

        // 2. Forward — ping session-b via the hub
        let req = json!({
            "jsonrpc": "2.0",
            "method": "termlink.ping",
            "id": "fwd-1",
            "params": { "target": reg_b.id.as_str() }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "fwd-1");
        assert_eq!(resp["result"]["display_name"], "hub-test-b");
        assert_eq!(resp["result"]["state"], "ready");

        hub_handle.abort();
        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn hub_malformed_json_returns_parse_error() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hub_handle = start_hub(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        writer.write_all(b"not valid json\n").await.unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["error"]["code"], -32700); // Parse error

        hub_handle.abort();
    }

    #[tokio::test]
    async fn hub_missing_target_returns_error() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hub_handle = start_hub(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let req = json!({
            "jsonrpc": "2.0",
            "method": "query.status",
            "id": "no-target",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "no-target");
        assert!(resp["error"]["code"].as_i64().unwrap() < 0);
        assert!(resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Missing"));

        hub_handle.abort();
    }
}
