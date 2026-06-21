use std::sync::Arc;

use tokio::net::UnixListener;
use tokio::sync::RwLock;

use crate::handler::SessionContext;
use crate::manager::Session;
use crate::registration::{Registration, SessionConfig};
use crate::server;

/// A TermLink endpoint running in the current process.
///
/// Creates a Unix socket and RPC server that makes this process discoverable
/// via the hub and able to participate in the event mesh (emit, poll, emit-to,
/// collect). Does NOT provide PTY capabilities (inject, output, stream) — for
/// that, use `Session::register` with `--shell`.
///
/// # Usage
///
/// ```no_run
/// use termlink_session::endpoint::Endpoint;
/// use termlink_session::registration::SessionConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let endpoint = Endpoint::start(SessionConfig {
///     display_name: Some("my-agent".into()),
///     tags: vec!["agent".into()],
///     ..Default::default()
/// }).await?;
///
/// println!("Registered: {} at {}", endpoint.id(), endpoint.socket_path().display());
///
/// // Run until shutdown signal
/// endpoint.run_until_shutdown().await;
/// # Ok(())
/// # }
/// ```
pub struct Endpoint {
    registration: Registration,
    listener: UnixListener,
    sessions_dir: std::path::PathBuf,
    session_id: crate::SessionId,
}

impl Endpoint {
    /// Start an endpoint in the current process.
    ///
    /// Registers with the sessions directory, creates a Unix socket,
    /// and prepares the RPC server. Call `run_until_shutdown()` or
    /// `run()` to start accepting connections.
    pub async fn start(config: SessionConfig) -> Result<Self, crate::SessionError> {
        let session = Session::register(config).await?;
        let id = session.id().clone();
        let sessions_dir = crate::discovery::sessions_dir();
        let (registration, listener, _) = session.into_parts();

        Ok(Self {
            registration,
            listener,
            sessions_dir,
            session_id: id,
        })
    }

    /// Start an endpoint in a specific sessions directory (for testing).
    pub async fn start_in(
        config: SessionConfig,
        sessions_dir: &std::path::Path,
    ) -> Result<Self, crate::SessionError> {
        let session = Session::register_in(config, sessions_dir).await?;
        let id = session.id().clone();
        let (registration, listener, sd) = session.into_parts();

        Ok(Self {
            registration,
            listener,
            sessions_dir: sd,
            session_id: id,
        })
    }

    /// Session ID.
    pub fn id(&self) -> &crate::SessionId {
        &self.session_id
    }

    /// Socket path for this endpoint.
    pub fn socket_path(&self) -> &std::path::Path {
        self.registration.socket_path()
    }

    /// The registration data.
    pub fn registration(&self) -> &Registration {
        &self.registration
    }

    /// Consume the endpoint, returning parts. Suppresses Drop cleanup.
    fn into_parts(self) -> (Registration, UnixListener, std::path::PathBuf, crate::SessionId) {
        let this = std::mem::ManuallyDrop::new(self);
        // SAFETY: ManuallyDrop suppresses the destructor, so we can move fields
        // out via ptr::read without double-free. Each field is read exactly once,
        // and the ManuallyDrop wrapper is never dropped (no destructor runs).
        unsafe {
            let registration = std::ptr::read(&this.registration);
            let listener = std::ptr::read(&this.listener);
            let sessions_dir = std::ptr::read(&this.sessions_dir);
            let session_id = std::ptr::read(&this.session_id);
            (registration, listener, sessions_dir, session_id)
        }
    }

    /// Run the RPC server until Ctrl+C or SIGHUP, then clean up.
    pub async fn run_until_shutdown(self) {
        let (registration, listener, sessions_dir, session_id) = self.into_parts();
        let socket_path = registration.socket_path().to_path_buf();
        let json_path = Registration::json_path(&sessions_dir, &session_id);
        let ctx = SessionContext::new(registration)
            .with_registration_path(json_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        // T-2235: keep heartbeat_at fresh for event-only (--self) endpoints, mirroring
        // the PTY register path (T-2230). As a never-completing select branch it is
        // cancelled automatically when the accept loop or shutdown signal wins.
        let shared_hb = shared.clone();

        tokio::select! {
            _ = server::run_accept_loop(listener, shared) => {}
            _ = tokio::signal::ctrl_c() => {}
            _ = heartbeat_loop(shared_hb) => {}
        }

        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&json_path);
    }

    /// Run the RPC server as a background task, returning a handle.
    ///
    /// The handle can be used to abort the server. Cleanup happens on drop.
    pub fn run_background(self) -> EndpointHandle {
        let (registration, listener, sessions_dir, session_id) = self.into_parts();
        let socket_path = registration.socket_path().to_path_buf();
        let json_path = Registration::json_path(&sessions_dir, &session_id);
        let ctx = SessionContext::new(registration)
            .with_registration_path(json_path.clone());
        let shared = Arc::new(RwLock::new(ctx));

        // T-2235: background endpoints heartbeat too. The heartbeat_loop is a
        // never-completing select branch, so it is cancelled when the accept loop
        // ends or the outer task is aborted via EndpointHandle::stop() — no leak.
        let shared_hb = shared.clone();
        let task = tokio::spawn(async move {
            tokio::select! {
                _ = server::run_accept_loop(listener, shared) => {}
                _ = heartbeat_loop(shared_hb) => {}
            }
        });

        EndpointHandle {
            task,
            socket_path,
            json_path,
            session_id,
        }
    }
}

/// T-2235: periodic self-heartbeat for event-only (`--self`) endpoints.
///
/// Mirrors the PTY register-path heartbeat (T-2230): every
/// `TERMLINK_HEARTBEAT_INTERVAL_SECS` seconds (default 30) it touches
/// `heartbeat_at` both in-memory (read by `query.status`) and on-disk (read by
/// the hub's directory sweep). Loops forever and is intended to be used as a
/// never-completing branch in a `tokio::select!`, so it is cancelled cleanly
/// when the accept loop or shutdown signal wins.
async fn heartbeat_loop(shared: Arc<RwLock<SessionContext>>) {
    let interval_secs = std::env::var("TERMLINK_HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(30);
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
    ticker.tick().await; // consume the immediate first tick
    loop {
        ticker.tick().await;
        let mut ctx = shared.write().await;
        if let Some(path) = ctx.registration_path.clone() {
            if let Err(e) = ctx.registration.touch_heartbeat(&path) {
                tracing::warn!(error = %e, "T-2235: endpoint heartbeat touch failed");
            }
        }
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        let json_path = Registration::json_path(&self.sessions_dir, &self.session_id);
        let _ = std::fs::remove_file(self.registration.socket_path());
        let _ = std::fs::remove_file(&json_path);
    }
}

/// Handle to a background endpoint. Cleans up on drop.
pub struct EndpointHandle {
    task: tokio::task::JoinHandle<()>,
    socket_path: std::path::PathBuf,
    json_path: std::path::PathBuf,
    session_id: crate::SessionId,
}

impl EndpointHandle {
    /// Session ID.
    pub fn id(&self) -> &crate::SessionId {
        &self.session_id
    }

    /// Socket path.
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }

    /// Stop the endpoint and clean up.
    pub fn stop(self) {
        self.task.abort();
        // Drop handles cleanup
    }
}

impl Drop for EndpointHandle {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_file(&self.socket_path);
        let _ = std::fs::remove_file(&self.json_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client;

    fn test_dir() -> std::path::PathBuf {
        // Keep path short to avoid SUN_LEN limit on Unix sockets
        let dir = std::path::PathBuf::from(format!("/tmp/tl-ep-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn endpoint_registers_and_accepts_rpc() {
        let dir = test_dir().join("sessions");
        std::fs::create_dir_all(&dir).unwrap();

        let endpoint = Endpoint::start_in(
            SessionConfig {
                display_name: Some("test-endpoint".into()),
                tags: vec!["test".into()],
                ..Default::default()
            },
            &dir,
        )
        .await
        .unwrap();

        let socket = endpoint.socket_path().to_path_buf();
        let id = endpoint.id().clone();
        let handle = endpoint.run_background();

        // Give server a moment to start accepting
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Ping
        let resp = client::rpc_call(&socket, "termlink.ping", serde_json::json!({}))
            .await
            .unwrap();
        let result = client::unwrap_result(resp).unwrap();
        assert_eq!(result["id"].as_str().unwrap(), id.as_str());
        assert_eq!(result["display_name"].as_str().unwrap(), "test-endpoint");

        // Emit + poll events
        let resp = client::rpc_call(
            &socket,
            "event.emit",
            serde_json::json!({"topic": "test.hello", "payload": {"msg": "hi"}}),
        )
        .await
        .unwrap();
        let result = client::unwrap_result(resp).unwrap();
        assert_eq!(result["topic"], "test.hello");

        let resp = client::rpc_call(&socket, "event.poll", serde_json::json!({}))
            .await
            .unwrap();
        let result = client::unwrap_result(resp).unwrap();
        let events = result["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["topic"], "test.hello");
        assert_eq!(events[0]["payload"]["msg"], "hi");

        handle.stop();
    }

    #[tokio::test]
    async fn endpoint_self_heartbeat_advances() {
        // T-2235: event-only (--self) endpoints must advance heartbeat_at on-disk
        // (the path the hub sweep reads) — otherwise they appear as frozen husks.
        let dir = test_dir().join("hb-sessions");
        std::fs::create_dir_all(&dir).unwrap();

        // Short interval so the test is fast. SAFETY: no test in this module
        // asserts the 30s default; a faster heartbeat is harmless to the others.
        unsafe {
            std::env::set_var("TERMLINK_HEARTBEAT_INTERVAL_SECS", "1");
        }

        let endpoint = Endpoint::start_in(
            SessionConfig {
                display_name: Some("hb-endpoint".into()),
                ..Default::default()
            },
            &dir,
        )
        .await
        .unwrap();
        let id = endpoint.id().clone();
        let json_path = Registration::json_path(&dir, &id);
        let handle = endpoint.run_background();

        let read_hb = |p: &std::path::Path| -> u64 {
            let s = std::fs::read_to_string(p).unwrap();
            let v: serde_json::Value = serde_json::from_str(&s).unwrap();
            v["heartbeat_at"]
                .as_str()
                .unwrap()
                .trim_end_matches('Z')
                .parse::<u64>()
                .unwrap()
        };

        // Initial heartbeat from registration.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let hb0 = read_hb(&json_path);

        // Wait for at least two 1s ticks.
        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
        let hb1 = read_hb(&json_path);

        assert!(
            hb1 > hb0,
            "heartbeat_at must advance for --self endpoint: hb0={hb0} hb1={hb1}"
        );

        handle.stop();
        unsafe {
            std::env::remove_var("TERMLINK_HEARTBEAT_INTERVAL_SECS");
        }
    }

    #[tokio::test]
    async fn endpoint_cleanup_on_drop() {
        let dir = test_dir().join("cleanup-sessions");
        std::fs::create_dir_all(&dir).unwrap();

        let socket_path;
        let json_path;

        {
            let endpoint = Endpoint::start_in(
                SessionConfig {
                    display_name: Some("cleanup-test".into()),
                    ..Default::default()
                },
                &dir,
            )
            .await
            .unwrap();

            socket_path = endpoint.socket_path().to_path_buf();
            json_path = Registration::json_path(&dir, endpoint.id());

            assert!(socket_path.exists());
            assert!(json_path.exists());
        }
        // Endpoint dropped — files should be cleaned up
        assert!(!socket_path.exists(), "Socket should be cleaned up");
        assert!(!json_path.exists(), "JSON sidecar should be cleaned up");
    }

    #[tokio::test]
    async fn endpoint_handle_cleanup_on_drop() {
        let dir = test_dir().join("handle-cleanup");
        std::fs::create_dir_all(&dir).unwrap();

        let endpoint = Endpoint::start_in(
            SessionConfig {
                display_name: Some("handle-test".into()),
                ..Default::default()
            },
            &dir,
        )
        .await
        .unwrap();

        let socket_path = endpoint.socket_path().to_path_buf();
        let handle = endpoint.run_background();

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(socket_path.exists());

        let json_path = Registration::json_path(&dir, handle.id());
        drop(handle);

        // Small delay for cleanup
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert!(!socket_path.exists(), "Socket should be cleaned up after handle drop");
        assert!(!json_path.exists(), "JSON should be cleaned up after handle drop");
    }

    #[tokio::test]
    async fn endpoint_kv_operations() {
        let dir = test_dir().join("kv-sessions");
        std::fs::create_dir_all(&dir).unwrap();

        let endpoint = Endpoint::start_in(
            SessionConfig {
                display_name: Some("kv-test".into()),
                ..Default::default()
            },
            &dir,
        )
        .await
        .unwrap();

        let socket = endpoint.socket_path().to_path_buf();
        let handle = endpoint.run_background();
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Set a key
        let resp = client::rpc_call(
            &socket,
            "kv.set",
            serde_json::json!({"key": "status", "value": "running"}),
        )
        .await
        .unwrap();
        assert!(client::unwrap_result(resp).is_ok());

        // Get the key
        let resp = client::rpc_call(
            &socket,
            "kv.get",
            serde_json::json!({"key": "status"}),
        )
        .await
        .unwrap();
        let result = client::unwrap_result(resp).unwrap();
        assert_eq!(result["value"], "running");

        handle.stop();
    }
}
