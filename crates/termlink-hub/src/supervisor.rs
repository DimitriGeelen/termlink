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

/// T-3293: an orphaned `<id>.sock.data` must be at least this old before the
/// sweep reaps it. A session writes its files in sequence, so a data socket can
/// briefly exist before its registration JSON; 10 minutes is far beyond that.
pub const ORPHAN_DATA_SOCKET_GRACE: Duration = Duration::from_secs(600);

/// Topic for session lifecycle exit events.
pub const SESSION_EXITED_TOPIC: &str = "session.exited";

/// Run the session supervision loop.
///
/// Polls all registered sessions every `interval` and removes stale ones.
/// Emits `session.exited` events before cleanup.
/// Stops when the shutdown signal is received.
pub async fn run(interval: Duration, shutdown_rx: watch::Receiver<bool>) {
    run_with(interval, shutdown_rx, sweep_targets, true).await
}

/// The real sweep targets: every candidate session dir (or the default one),
/// plus the legacy `/tmp/termlink-<uid>` pool.
fn sweep_targets() -> Vec<std::path::PathBuf> {
    // T-987: sweep all candidate session dirs
    let mut dirs = discovery::all_sessions_dirs();
    if dirs.is_empty() {
        // Fall back to default dir even if it doesn't exist yet
        dirs.push(discovery::sessions_dir());
    }
    let uid = unsafe { libc::getuid() };
    with_legacy_pool(dirs, std::path::PathBuf::from(format!("/tmp/termlink-{uid}/sessions")))
}

/// T-3295: add the legacy pool to the sweep targets when it exists.
///
/// A hub started with `TERMLINK_RUNTIME_DIR` (every systemd unit) scans ONLY that
/// dir — `all_runtime_dirs` treats the override as exclusive. Sessions launched
/// without the variable (e.g. from a tmux server started under `env -i`) register
/// in `/tmp/termlink-<uid>` instead, where no janitor ever looked: 275 live ones on
/// one host, so any that crashed or were SIGKILLed would leak there forever. The
/// sweep only removes dead registrations and aged orphans, so adding it is safe.
fn with_legacy_pool(
    mut dirs: Vec<std::path::PathBuf>,
    legacy: std::path::PathBuf,
) -> Vec<std::path::PathBuf> {
    if legacy.is_dir() && !dirs.contains(&legacy) {
        dirs.push(legacy);
    }
    dirs
}

/// The supervision loop with its host-state inputs injected.
///
/// T-3293: `run` resolves the REAL session dirs and inbox. A test that drove
/// `run` swept whatever `TERMLINK_RUNTIME_DIR` pointed at on the test host — and
/// once the sweep reaped orphan data sockets, a `cargo test` deleted ~10,000 real
/// files on the machine running it. Tests pass their own dirs and skip the inbox.
pub async fn run_with(
    interval: Duration,
    mut shutdown_rx: watch::Receiver<bool>,
    dirs: impl Fn() -> Vec<std::path::PathBuf>,
    inbox_housekeeping: bool,
) {
    tracing::info!(
        interval_secs = interval.as_secs(),
        "Session supervisor started"
    );

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                for dir in dirs() {
                    sweep(&dir).await;
                }
                if inbox_housekeeping {
                    // T-988: deliver pending inbox files + clean expired
                    deliver_inbox().await;
                    let expired = crate::inbox::cleanup_expired(crate::inbox::DEFAULT_EXPIRY);
                    if expired > 0 {
                        tracing::info!(expired, "Supervisor: cleaned expired inbox entries");
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

/// Check inbox for pending files and deliver to sessions that are now online (T-988).
async fn deliver_inbox() {
    let targets = match crate::inbox::list_all_targets() {
        Ok(t) => t,
        Err(_) => return,
    };

    for (target, _count) in targets {
        // Try to find the session — if alive, deliver pending files
        if let Ok(reg) = manager::find_session(&target)
            && liveness::is_alive(&reg)
        {
            let addr = reg.addr.to_transport_addr();
            let delivered = crate::inbox::deliver_pending(&target, &addr).await;
            if delivered > 0 {
                tracing::info!(
                    target = %target,
                    delivered = delivered,
                    "Supervisor: delivered inbox files to now-online session"
                );
            }
        }
    }
}

/// Perform a single supervision sweep: list sessions, check liveness,
/// emit `session.exited` events for dead sessions, then clean up.
pub async fn sweep(sessions_dir: &Path) {
    // T-3293: reap orphaned data-plane sockets on EVERY sweep — before the
    // early return below, which fires whenever no session died this cycle and
    // would otherwise leave orphans from earlier deaths untouched forever.
    let reap = liveness::reap_orphan_data_sockets(sessions_dir, ORPHAN_DATA_SOCKET_GRACE);
    if reap.removed > 0 {
        tracing::info!(
            reaped = reap.removed,
            dir = %sessions_dir.display(),
            "Supervisor: reaped orphaned data-plane sockets (<id>.sock.data without a registration)"
        );
    }
    if reap.failed > 0 {
        // T-3299: say it. Under systemd ProtectSystem=strict a dir outside
        // ReadWritePaths is read-only, and every delete there failed silently.
        tracing::warn!(
            failed = reap.failed,
            dir = %sessions_dir.display(),
            first_error = reap.first_error.as_deref().unwrap_or("?"),
            "Supervisor: could NOT remove orphaned data-plane sockets — is this dir writable by the hub (systemd ReadWritePaths)?"
        );
    }

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

    /// T-3293: an orphaned data socket is reaped by the sweep even when no
    /// session died this cycle — the early-return path that used to skip it.
    #[tokio::test]
    async fn sweep_reaps_old_orphan_data_socket_even_with_no_deaths() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("tl-gone.sock.data");
        let young = dir.path().join("tl-fresh.sock.data");
        std::fs::write(&old, b"").unwrap();
        std::fs::write(&young, b"").unwrap();
        let aged = std::time::SystemTime::now() - (ORPHAN_DATA_SOCKET_GRACE + Duration::from_secs(60));
        std::fs::File::options().write(true).open(&old).unwrap().set_modified(aged).unwrap();

        sweep(dir.path()).await;

        assert!(!old.exists(), "orphan older than the grace must be reaped");
        assert!(young.exists(), "orphan younger than the grace must survive");
    }

    #[test]
    fn legacy_pool_is_added_once_and_only_when_present() {
        let primary = tempfile::tempdir().unwrap();
        let legacy = tempfile::tempdir().unwrap();
        let p = primary.path().to_path_buf();
        let l = legacy.path().to_path_buf();

        assert_eq!(with_legacy_pool(vec![p.clone()], l.clone()), vec![p.clone(), l.clone()]);
        assert_eq!(with_legacy_pool(vec![p.clone(), l.clone()], l.clone()), vec![p.clone(), l.clone()],
            "never duplicated");
        let absent = l.join("does-not-exist");
        assert_eq!(with_legacy_pool(vec![p.clone()], absent), vec![p], "absent pool not added");
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

        // T-3293: never the real session dirs or inbox — see `run_with`.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let handle = tokio::spawn(async move {
            run_with(Duration::from_millis(50), rx, move || vec![path.clone()], false).await;
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
