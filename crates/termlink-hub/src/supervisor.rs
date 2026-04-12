//! Session supervision loop for the hub daemon.
//!
//! Periodically polls all registered sessions for liveness and cleans up
//! stale registrations (dead process, missing socket). Emits `session.exited`
//! events to all live sessions before cleanup, enabling dispatch orchestrators
//! to detect worker crashes without polling.

use std::path::Path;
use std::time::Duration;

use serde_json::json;
use tokio::sync::watch;

use termlink_protocol::control;
use termlink_session::{client, discovery, liveness, manager};

/// Default supervision interval.
pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(30);

/// Topic for session lifecycle exit events.
pub const SESSION_EXITED_TOPIC: &str = "session.exited";

/// Run the session supervision loop.
///
/// Polls all registered sessions every `interval` and removes stale ones.
/// Emits `session.exited` events before cleanup.
/// Stops when the shutdown signal is received.
pub async fn run(interval: Duration, mut shutdown_rx: watch::Receiver<bool>) {
    tracing::info!(
        interval_secs = interval.as_secs(),
        "Session supervisor started"
    );

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                // T-987: sweep all candidate session dirs
                let dirs = discovery::all_sessions_dirs();
                if dirs.is_empty() {
                    // Fall back to default dir even if it doesn't exist yet
                    sweep(&discovery::sessions_dir()).await;
                } else {
                    for dir in &dirs {
                        sweep(dir).await;
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("Session supervisor shutting down");
                    break;
                }
            }
        }
    }
}

/// Perform a single supervision sweep: list sessions, check liveness,
/// emit `session.exited` events for dead sessions, then clean up.
pub async fn sweep(sessions_dir: &Path) {
    let sessions = match manager::list_sessions_in(sessions_dir, true) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!(error = %e, "Supervisor: could not list sessions");
            return;
        }
    };

    // Partition into alive and dead
    let mut dead = Vec::new();
    let mut alive = Vec::new();
    for reg in &sessions {
        if liveness::is_alive(reg) {
            alive.push(reg);
        } else {
            dead.push(reg);
        }
    }

    if dead.is_empty() {
        return;
    }

    // Emit session.exited to all live sessions for each dead session
    for dead_reg in &dead {
        tracing::warn!(
            session_id = %dead_reg.id,
            pid = dead_reg.pid,
            name = ?dead_reg.display_name,
            "Supervisor: detected dead session, emitting session.exited"
        );

        let payload = json!({
            "session_id": dead_reg.id.as_str(),
            "display_name": dead_reg.display_name,
            "pid": dead_reg.pid,
            "reason": "process_died",
            "tags": dead_reg.tags,
        });

        // Fan-out to all live sessions (best-effort, don't block on failures)
        let emit_params = json!({
            "topic": SESSION_EXITED_TOPIC,
            "payload": payload,
        });

        for live_reg in &alive {
            let addr = live_reg.addr.to_transport_addr();
            let params = emit_params.clone();
            // Fire-and-forget with short timeout — don't let slow sessions block sweep
            let result = tokio::time::timeout(
                Duration::from_secs(2),
                client::rpc_call_addr(&addr, control::method::EVENT_EMIT, params),
            )
            .await;

            if let Ok(Err(e)) = &result {
                tracing::debug!(
                    target_session = %live_reg.id,
                    error = %e,
                    "Failed to deliver session.exited event"
                );
            }
        }
    }

    // Now clean up dead sessions
    for dead_reg in &dead {
        liveness::cleanup_stale(dead_reg, sessions_dir);
    }

    tracing::info!(
        cleaned = dead.len(),
        total = sessions.len(),
        "Supervisor sweep complete (session.exited emitted)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use termlink_session::handler::SessionContext;
    use termlink_session::identity::SessionId;
    use termlink_session::registration::{Registration, SessionConfig};
    use termlink_session::server;
    use tokio::sync::RwLock;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_sessions_dir() -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::path::PathBuf::from(format!(
            "/tmp/tl-supervisor-{}-{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Create a fake session registration file with a specific PID.
    fn create_fake_session(sessions_dir: &Path, name: &str, pid: u32) -> SessionId {
        let id = SessionId::generate();
        let socket_path = sessions_dir.join(format!("{id}.sock"));
        // Create a fake socket file so is_alive checks it
        std::fs::write(&socket_path, b"").unwrap();

        let config = SessionConfig {
            display_name: Some(name.into()),
            ..Default::default()
        };
        let mut reg = Registration::new(id.clone(), config, socket_path);
        reg.pid = pid;

        // Write the JSON registration file
        let json_path = sessions_dir.join(format!("{id}.json"));
        let json = serde_json::to_string_pretty(&reg).unwrap();
        std::fs::write(&json_path, json).unwrap();

        id
    }

    #[tokio::test]
    async fn sweep_cleans_dead_sessions() {
        let dir = test_sessions_dir();

        // Create a session with a dead PID
        let dead_id = create_fake_session(&dir, "dead-session", 4_000_000);

        // Create a session with our own (alive) PID
        let alive_id = create_fake_session(&dir, "alive-session", std::process::id());

        // Sweep
        sweep(&dir).await;

        // Dead session should be cleaned
        let dead_json = dir.join(format!("{dead_id}.json"));
        let dead_sock = dir.join(format!("{dead_id}.sock"));
        assert!(!dead_json.exists(), "Dead session JSON should be removed");
        assert!(!dead_sock.exists(), "Dead session socket should be removed");

        // Alive session should remain
        let alive_json = dir.join(format!("{alive_id}.json"));
        let alive_sock = dir.join(format!("{alive_id}.sock"));
        assert!(alive_json.exists(), "Alive session JSON should remain");
        assert!(alive_sock.exists(), "Alive session socket should remain");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sweep_empty_dir_is_ok() {
        let dir = test_sessions_dir();
        sweep(&dir).await; // Should not panic
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sweep_nonexistent_dir_is_ok() {
        let dir = std::path::PathBuf::from("/tmp/tl-supervisor-nonexistent-dir");
        let _ = std::fs::remove_dir_all(&dir);
        sweep(&dir).await; // Should not panic
    }

    #[tokio::test]
    async fn supervisor_respects_shutdown() {
        let (_tx, rx) = watch::channel(false);
        let tx_clone = _tx.clone();

        let handle = tokio::spawn(async move {
            run(Duration::from_millis(50), rx).await;
        });

        // Let it run a few cycles
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Signal shutdown
        tx_clone.send(true).unwrap();

        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Supervisor should stop on shutdown signal");
    }

    /// Start a real session with accept loop for integration tests.
    async fn start_real_session(
        sessions_dir: &Path,
        name: &str,
    ) -> (tokio::task::JoinHandle<()>, Registration) {
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

        tokio::time::sleep(Duration::from_millis(10)).await;
        (handle, reg)
    }

    #[tokio::test]
    async fn sweep_emits_session_exited_to_live_sessions() {
        let dir = test_sessions_dir();

        // Start a real session (the observer)
        let (handle, observer_reg) = start_real_session(&dir, "observer").await;

        // Create a fake dead session
        let _dead_id = create_fake_session(&dir, "dead-worker", 4_000_000);

        // Sweep — should emit session.exited to observer, then clean dead session
        sweep(&dir).await;

        // Give a moment for the event to be delivered
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Poll the observer's event bus for session.exited
        let resp = client::rpc_call(
            observer_reg.socket_path(),
            "event.poll",
            json!({"topic": "session.exited"}),
        )
        .await
        .expect("Should poll observer events");

        let result = client::unwrap_result(resp).expect("Should get poll result");
        let events = result["events"].as_array().expect("Should have events array");

        assert!(
            !events.is_empty(),
            "Observer should have received session.exited event"
        );

        let event = &events[0];
        assert_eq!(event["topic"], "session.exited");
        assert_eq!(event["payload"]["display_name"], "dead-worker");
        assert_eq!(event["payload"]["reason"], "process_died");
        assert_eq!(event["payload"]["pid"], 4_000_000);

        handle.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sweep_no_event_for_alive_sessions() {
        let dir = test_sessions_dir();

        // Start a real session (the observer)
        let (handle, observer_reg) = start_real_session(&dir, "observer2").await;

        // No dead sessions — just the observer

        // Sweep — should do nothing
        sweep(&dir).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Poll — should have no session.exited events
        let resp = client::rpc_call(
            observer_reg.socket_path(),
            "event.poll",
            json!({"topic": "session.exited"}),
        )
        .await
        .expect("Should poll");

        let result = client::unwrap_result(resp).expect("Should get poll result");
        let events = result["events"].as_array().expect("Should have events array");
        assert!(
            events.is_empty(),
            "No session.exited events should be emitted when all sessions are alive"
        );

        handle.abort();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
