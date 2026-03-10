//! Session supervision loop for the hub daemon.
//!
//! Periodically polls all registered sessions for liveness and cleans up
//! stale registrations (dead process, missing socket).

use std::path::Path;
use std::time::Duration;

use tokio::sync::watch;

use termlink_session::discovery;
use termlink_session::liveness;
use termlink_session::manager;

/// Default supervision interval.
pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(30);

/// Run the session supervision loop.
///
/// Polls all registered sessions every `interval` and removes stale ones.
/// Stops when the shutdown signal is received.
pub async fn run(interval: Duration, mut shutdown_rx: watch::Receiver<bool>) {
    let sessions_dir = discovery::runtime_dir().join("sessions");
    tracing::info!(
        interval_secs = interval.as_secs(),
        "Session supervisor started"
    );

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                sweep(&sessions_dir);
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

/// Perform a single supervision sweep: list sessions, check liveness, clean stale.
pub fn sweep(sessions_dir: &Path) {
    let sessions = match manager::list_sessions_in(sessions_dir, true) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!(error = %e, "Supervisor: could not list sessions");
            return;
        }
    };

    let mut cleaned = 0;
    for reg in &sessions {
        if !liveness::is_alive(reg) {
            tracing::warn!(
                session_id = %reg.id,
                pid = reg.pid,
                name = ?reg.display_name,
                "Supervisor: detected dead session, cleaning up"
            );
            liveness::cleanup_stale(reg, sessions_dir);
            cleaned += 1;
        }
    }

    if cleaned > 0 {
        tracing::info!(cleaned, total = sessions.len(), "Supervisor sweep complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use termlink_session::identity::SessionId;
    use termlink_session::registration::{Registration, SessionConfig};

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

    #[test]
    fn sweep_cleans_dead_sessions() {
        let dir = test_sessions_dir();

        // Create a session with a dead PID
        let dead_id = create_fake_session(&dir, "dead-session", 4_000_000);

        // Create a session with our own (alive) PID
        let alive_id = create_fake_session(&dir, "alive-session", std::process::id());

        // Sweep
        sweep(&dir);

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

    #[test]
    fn sweep_empty_dir_is_ok() {
        let dir = test_sessions_dir();
        sweep(&dir); // Should not panic
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sweep_nonexistent_dir_is_ok() {
        let dir = std::path::PathBuf::from("/tmp/tl-supervisor-nonexistent-dir");
        let _ = std::fs::remove_dir_all(&dir);
        sweep(&dir); // Should not panic
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
}
