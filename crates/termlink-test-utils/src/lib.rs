//! Shared test utilities for the TermLink workspace.
//!
//! Provides common helpers to reduce boilerplate across test files:
//! - [`TestDir`] — Unique temp directories with auto-cleanup
//! - [`ProcessGuard`] — RAII process management (kill on drop)
//! - [`wait_for_socket`] / [`wait_for_data_socket`] — Socket polling
//! - [`start_session`] — Session fixture with accept loop

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use termlink_session::handler::SessionContext;
use termlink_session::manager::Session;
use termlink_session::registration::{Registration, SessionConfig};
use termlink_session::server;

static DIR_COUNTER: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// TestDir — unique temp directory with optional auto-cleanup
// ---------------------------------------------------------------------------

/// A unique temporary directory for test isolation.
///
/// Creates `/tmp/tl-test-{counter}-{name}` with a `sessions/` subdirectory.
/// Cleans up on drop unless `keep()` is called.
pub struct TestDir {
    pub path: PathBuf,
    cleanup: bool,
}

impl TestDir {
    /// Create a new unique test directory.
    pub fn new(name: &str) -> Self {
        let n = DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-test-{n}-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sessions")).unwrap();
        Self {
            path: dir,
            cleanup: true,
        }
    }

    /// Path to the sessions subdirectory.
    pub fn sessions_dir(&self) -> PathBuf {
        self.path.join("sessions")
    }

    /// Prevent cleanup on drop (useful for debugging failed tests).
    #[allow(dead_code)]
    pub fn keep(&mut self) {
        self.cleanup = false;
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

impl AsRef<Path> for TestDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

// ---------------------------------------------------------------------------
// ProcessGuard — RAII child process management
// ---------------------------------------------------------------------------

/// RAII guard that kills a child process on drop.
///
/// Guarantees cleanup even on test panic.
pub struct ProcessGuard {
    child: Child,
    #[allow(dead_code)]
    name: String,
}

impl ProcessGuard {
    pub fn new(child: Child, name: &str) -> Self {
        Self {
            child,
            name: name.to_string(),
        }
    }

    /// Access the child process (e.g., to check stdout).
    pub fn child(&mut self) -> &mut Child {
        &mut self.child
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// Socket polling
// ---------------------------------------------------------------------------

/// Wait until at least one `.sock` file appears in a directory.
///
/// Returns the path to the first socket found, or an error on timeout.
pub fn wait_for_socket(sessions_dir: &Path, timeout: Duration) -> Result<PathBuf, String> {
    let start = Instant::now();
    loop {
        if let Ok(entries) = std::fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "sock") {
                    return Ok(path);
                }
            }
        }
        if start.elapsed() > timeout {
            return Err(format!(
                "No socket appeared in {sessions_dir:?} within {timeout:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Wait until a `.sock.data` file appears in a directory.
///
/// Returns the path to the data socket, or an error on timeout.
pub fn wait_for_data_socket(sessions_dir: &Path, timeout: Duration) -> Result<PathBuf, String> {
    let start = Instant::now();
    loop {
        if let Ok(entries) = std::fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_string_lossy().ends_with(".sock.data") {
                    return Ok(path);
                }
            }
        }
        if start.elapsed() > timeout {
            return Err(format!(
                "No data socket appeared in {sessions_dir:?} within {timeout:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

// ---------------------------------------------------------------------------
// Session fixtures
// ---------------------------------------------------------------------------

/// Shared session type used by the server accept loop.
pub type SharedSession = Arc<RwLock<SessionContext>>;

/// Register a session and start its accept loop in the background.
///
/// Returns the join handle and a clone of the registration for assertions.
pub async fn start_session(
    sessions_dir: &Path,
    name: &str,
    roles: Vec<String>,
) -> (tokio::task::JoinHandle<()>, Registration) {
    let config = SessionConfig {
        display_name: Some(name.into()),
        roles,
        ..Default::default()
    };
    let session = Session::register_in(config, sessions_dir)
        .await
        .unwrap();

    let (registration, listener, _sessions_dir) = session.into_parts();
    let reg = registration.clone();
    let ctx = SessionContext::new(registration);
    let shared = Arc::new(RwLock::new(ctx));

    let handle = tokio::spawn(async move {
        server::run_accept_loop(listener, shared).await;
    });

    // Give the accept loop a moment to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    (handle, reg)
}

/// Build a Command for the `termlink` binary with isolated runtime dir.
///
/// Requires the binary to be built (use `cargo_bin!("termlink")` for the path).
pub fn termlink_cmd(binary: &Path, runtime_dir: &Path) -> Command {
    let mut cmd = Command::new(binary);
    cmd.env("TERMLINK_RUNTIME_DIR", runtime_dir);
    cmd.env("RUST_LOG", "");
    cmd
}

// ---------------------------------------------------------------------------
// Tests for the test utils themselves
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_creates_sessions_subdir() {
        let dir = TestDir::new("unit-test");
        assert!(dir.sessions_dir().exists());
        let path = dir.path.clone();
        drop(dir);
        // Should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_dir_keep_prevents_cleanup() {
        let mut dir = TestDir::new("keep-test");
        dir.keep();
        let path = dir.path.clone();
        drop(dir);
        assert!(path.exists());
        // Manual cleanup
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_dir_unique_names() {
        let d1 = TestDir::new("a");
        let d2 = TestDir::new("a");
        assert_ne!(d1.path, d2.path);
    }

    #[test]
    fn process_guard_kills_on_drop() {
        use std::process::Stdio;
        let child = Command::new("sleep")
            .arg("60")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let pid = child.id();
        let guard = ProcessGuard::new(child, "sleep");
        drop(guard);
        // Process should be dead
        let result = unsafe { libc::kill(pid as i32, 0) };
        assert_ne!(result, 0);
    }

    #[tokio::test]
    async fn start_session_creates_socket() {
        let dir = TestDir::new("session-fixture");
        let (_handle, reg) = start_session(&dir.sessions_dir(), "test-sess", vec![]).await;
        assert!(reg.socket.exists());
        _handle.abort();
    }
}
