use std::path::{Path, PathBuf};

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::watch;

use termlink_protocol::jsonrpc::{ErrorResponse, Request, RpcResponse};
use termlink_session::auth::PeerCredentials;
use termlink_session::discovery;

use crate::pidfile;
use crate::remote_store;
use crate::router;
use crate::supervisor;

/// Return the well-known hub socket path: `runtime_dir()/hub.sock`.
pub fn hub_socket_path() -> PathBuf {
    discovery::runtime_dir().join("hub.sock")
}

/// A handle to signal the hub to shut down gracefully.
#[derive(Clone)]
pub struct ShutdownHandle {
    tx: watch::Sender<bool>,
}

impl ShutdownHandle {
    /// Signal the hub to shut down. The accept loop will stop and
    /// active connections will be given time to complete.
    pub fn shutdown(&self) {
        let _ = self.tx.send(true);
    }
}

/// Start the hub server, binding to the given socket path.
///
/// Returns a [`ShutdownHandle`] that can be used to trigger graceful shutdown.
/// The server will:
/// 1. Stop accepting new connections
/// 2. Wait up to 5 seconds for active connections to complete
/// 3. Remove pidfile and socket file
///
/// Acquires a pidfile to prevent multiple hub instances. The pidfile is removed
/// on clean shutdown. Stale pidfiles from crashed hubs are cleaned automatically.
pub async fn run(socket_path: &Path) -> std::io::Result<ShutdownHandle> {
    run_with_tcp(socket_path, None).await
}

/// Start the hub server with optional TCP listener.
///
/// When `tcp_addr` is provided (e.g., "0.0.0.0:9100"), the hub listens on
/// both the Unix socket and the TCP address simultaneously.
pub async fn run_with_tcp(
    socket_path: &Path,
    tcp_addr: Option<&str>,
) -> std::io::Result<ShutdownHandle> {
    let pidfile_path = pidfile::hub_pidfile_path();

    // Acquire pidfile (prevents double-start, cleans stale)
    pidfile::acquire(&pidfile_path).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::AddrInUse, e.to_string())
    })?;

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let unix_listener = UnixListener::bind(socket_path)?;
    tracing::info!(path = %socket_path.display(), "Hub listening on Unix");

    // Optional TCP listener
    let tcp_listener = if let Some(addr) = tcp_addr {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            std::io::Error::new(e.kind(), format!("Failed to bind TCP {}: {}", addr, e))
        })?;
        let local_addr = listener.local_addr()?;
        tracing::info!(%local_addr, "Hub listening on TCP");
        Some(listener)
    } else {
        None
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let handle = ShutdownHandle { tx: shutdown_tx };

    // Initialize the remote session store
    let remote_store = router::init_remote_store();

    // Start the session supervisor
    let supervisor_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        supervisor::run(supervisor::DEFAULT_INTERVAL, supervisor_rx).await;
    });

    // Start the remote store reaper (expires stale remote sessions)
    let reaper_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        remote_store::run_reaper(remote_store, remote_store::REAPER_INTERVAL, reaper_rx).await;
    });

    let socket_path_owned = socket_path.to_path_buf();
    tokio::spawn(async move {
        run_accept_loop(unix_listener, tcp_listener, shutdown_rx).await;

        // Cleanup on exit
        let _ = std::fs::remove_file(&socket_path_owned);
        pidfile::remove(&pidfile_path);
        tracing::info!("Hub shut down cleanly");
    });

    Ok(handle)
}

/// Start the hub server and block until shutdown.
///
/// This is the simple API for CLI usage — starts the server and waits
/// for the shutdown handle to be triggered.
pub async fn run_blocking(socket_path: &Path, tcp_addr: Option<&str>) -> std::io::Result<()> {
    let pidfile_path = pidfile::hub_pidfile_path();

    // Acquire pidfile (prevents double-start, cleans stale)
    pidfile::acquire(&pidfile_path).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::AddrInUse, e.to_string())
    })?;

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let unix_listener = UnixListener::bind(socket_path)?;
    tracing::info!(path = %socket_path.display(), "Hub listening on Unix");

    let tcp_listener = if let Some(addr) = tcp_addr {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            std::io::Error::new(e.kind(), format!("Failed to bind TCP {}: {}", addr, e))
        })?;
        let local_addr = listener.local_addr()?;
        tracing::info!(%local_addr, "Hub listening on TCP");
        Some(listener)
    } else {
        None
    };

    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    run_accept_loop(unix_listener, tcp_listener, shutdown_rx).await;

    // Cleanup on exit
    pidfile::remove(&pidfile_path);
    Ok(())
}

/// Accept loop: spawns a task per connection.
///
/// Rejects connections from different UIDs (same security model as session server).
/// Stops accepting when the shutdown signal is received, then waits up to 5 seconds
/// for active connections to complete.
pub async fn run_accept_loop(
    unix_listener: UnixListener,
    tcp_listener: Option<TcpListener>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let owner_uid = unsafe { libc::getuid() };
    let active_connections = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    loop {
        // Select over Unix listener, optional TCP listener, and shutdown signal
        tokio::select! {
            result = unix_listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        // Extract peer credentials and check UID
                        match PeerCredentials::from_tokio_stream(&stream) {
                            Ok(creds) => {
                                if !creds.is_same_user(owner_uid) {
                                    tracing::warn!(
                                        peer_uid = creds.uid,
                                        peer_pid = ?creds.pid,
                                        owner_uid = owner_uid,
                                        "Hub: rejected Unix connection from different UID"
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

                        let counter = active_connections.clone();
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tokio::spawn(async move {
                            handle_connection(stream).await;
                            counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Hub Unix accept failed");
                        break;
                    }
                }
            }

            result = async {
                match tcp_listener.as_ref() {
                    Some(l) => l.accept().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Ok((stream, peer_addr)) => {
                        tracing::info!(
                            %peer_addr,
                            "Hub: TCP connection accepted (no auth — LAN-only)"
                        );

                        let counter = active_connections.clone();
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tokio::spawn(async move {
                            handle_connection(stream).await;
                            counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Hub TCP accept failed");
                        // Don't break — Unix listener can still work
                    }
                }
            }

            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("Hub: shutdown signal received, draining connections");
                    break;
                }
            }
        }
    }

    // Drain: wait up to 5 seconds for active connections to finish
    let drain_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while active_connections.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        if tokio::time::Instant::now() >= drain_deadline {
            let remaining = active_connections.load(std::sync::atomic::Ordering::Relaxed);
            tracing::warn!(remaining, "Hub: drain timeout, forcing shutdown");
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// Handle a single hub client connection.
///
/// Reads newline-delimited JSON-RPC, routes via [`router::route`],
/// writes newline-delimited JSON-RPC responses.
async fn handle_connection<S>(stream: S)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (reader, mut writer) = tokio::io::split(stream);
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

    /// Start the hub server on the given socket with a shutdown handle.
    fn start_hub_with_shutdown(socket: PathBuf) -> (tokio::task::JoinHandle<()>, watch::Sender<bool>) {
        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket);
            let listener = UnixListener::bind(&socket).unwrap();
            run_accept_loop(listener, None, rx).await;
        });
        (handle, tx)
    }

    /// Start the hub server on the given socket, return its handle.
    fn start_hub(socket: PathBuf) -> tokio::task::JoinHandle<()> {
        let (handle, _tx) = start_hub_with_shutdown(socket);
        handle
    }

    /// Tests discover + forward in a single test to avoid env var races.
    /// Both require TERMLINK_RUNTIME_DIR to point to the test directory.
    #[tokio::test]
    async fn hub_discover_and_forward() {
        let _lock = ENV_LOCK.lock().unwrap();
        // Clear remote store to avoid leakage from other tests
        if let Some(s) = crate::router::remote_store() { s.clear(); }

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

    #[tokio::test]
    async fn graceful_shutdown_stops_accept_loop() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (hub_handle, shutdown_tx) = start_hub_with_shutdown(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Verify hub is accepting connections
        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        drop(stream);

        // Signal shutdown
        shutdown_tx.send(true).unwrap();

        // Hub should stop within a reasonable time
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            hub_handle,
        ).await;

        assert!(result.is_ok(), "Hub did not shut down within 3 seconds");
    }

    #[tokio::test]
    async fn graceful_shutdown_drains_active_connection() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (hub_handle, shutdown_tx) = start_hub_with_shutdown(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Connect a client that stays open
        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (_reader, _writer) = stream.into_split();

        // Signal shutdown while connection is active
        shutdown_tx.send(true).unwrap();

        // Hub should still shut down (drain timeout or client disconnect)
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(7),
            hub_handle,
        ).await;

        assert!(result.is_ok(), "Hub did not shut down during drain");
    }

    #[tokio::test]
    async fn hub_dual_listen_unix_and_tcp() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Start hub with both Unix and TCP listeners
        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            // Bind TCP on ephemeral port
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            // Write port to file so test can read it
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        // 1. Connect via Unix and send a request
        let unix_stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = unix_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "unix-1",
            "params": {}
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["id"], "unix-1");
        assert!(resp["result"].is_object(), "Unix connection should get valid response");

        // 2. Connect via TCP and send same request
        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "tcp-1",
            "params": {}
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["id"], "tcp-1");
        assert!(resp["result"].is_object(), "TCP connection should get valid response");

        // Cleanup
        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }
}
