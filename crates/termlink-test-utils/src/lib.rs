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
    // T-3755: the child must not inherit the caller's project or identity. A
    // PROJECT_ROOT naming another project than the test's cwd makes `register`
    // refuse (T-3551, `refuse_foreign_project_env`), and 46 cli_integration
    // tests then time out waiting for a socket. TERMLINK_AGENT_ID & co. from an
    // agent shell would likewise give the test session that agent's identity.
    for (key, _) in std::env::vars_os() {
        if let Some(k) = key.to_str()
            && is_scrubbed_var(k)
        {
            cmd.env_remove(k);
        }
    }
    cmd.env("TERMLINK_RUNTIME_DIR", runtime_dir);
    cmd.env("RUST_LOG", "");
    // T-3492: the empty-list runtime-dir hint must not read the real dirs.
    cmd.env("TERMLINK_HINT_CANDIDATE_DIRS", "");
    cmd
}

/// T-3755: variables [`termlink_cmd`] never passes on from the test process: the
/// project variables register judges (`child_env::PROJECT_VARS`), every
/// `TERMLINK_*` (identity, agent id, audit switches; the helper sets the ones a
/// test needs), and the harness's `CLAUDE*` / `FW_*` session markers.
pub fn is_scrubbed_var(name: &str) -> bool {
    termlink_session::child_env::PROJECT_VARS.contains(&name)
        || name.starts_with("TERMLINK_")
        || name.starts_with("CLAUDE")
        || name.starts_with("FW_")
}

/// The build-profile directory the running test binary lives in, e.g.
/// `<target-dir>/debug` for `<target-dir>/debug/deps/parity-1a2b`.
///
/// T-3679: this is where cargo put every binary of THIS build, whatever
/// `--target-dir` / `CARGO_TARGET_DIR` / `--profile` the run used. Returns
/// `None` when the current executable is not in a `deps/` directory (not a
/// cargo test binary).
fn current_profile_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let deps = exe.parent()?;
    if deps.file_name()? != "deps" {
        return None;
    }
    deps.parent().map(Path::to_path_buf)
}

/// Find the `termlink` CLI binary path for cross-crate integration tests.
///
/// CARGO_BIN_EXE_termlink is only set when the binary's own crate runs its
/// tests — cross-crate tests (e.g. `termlink-mcp/tests/parity.rs` invoking
/// the CLI for MCP/CLI parity checks, T-1909) need another way. Order:
/// 1. `TERMLINK_BIN` env var, an explicit override (CI / cross-compile);
/// 2. (T-3679) `termlink` next to the running test binary: the same target
///    dir and profile as this build, so a custom `--target-dir` is honoured
///    and a stale `target/release/termlink` is never preferred over the
///    binary this very `cargo test` just built;
/// 3. fallback: walk up from CARGO_MANIFEST_DIR to the workspace root and try
///    `target/release/termlink`, then `target/debug/termlink`.
///
/// **Note (T-1912):** This function does NOT rebuild the CLI binary. If
/// you depend on git-derived version metadata being coherent between the
/// CLI binary and the freshly-recompiled MCP test crate (e.g. comparing
/// commit hashes), use [`find_termlink_bin_fresh`] instead.
pub fn find_termlink_bin() -> Result<PathBuf, String> {
    if let Ok(override_path) = std::env::var("TERMLINK_BIN") {
        let p = PathBuf::from(override_path);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("$TERMLINK_BIN points at {p:?} which does not exist"));
    }
    if let Some(profile_dir) = current_profile_dir() {
        let candidate = profile_dir.join("termlink");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR not set — call from inside a cargo test".to_string())?;
    let mut cursor = PathBuf::from(manifest_dir);
    loop {
        if cursor.join("Cargo.lock").exists() {
            break;
        }
        if !cursor.pop() {
            return Err("workspace root (Cargo.lock) not found".into());
        }
    }
    for profile in &["release", "debug"] {
        let candidate = cursor.join("target").join(profile).join("termlink");
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "termlink binary not found in {cursor:?}/target/{{release,debug}}/. Run `cargo build -p termlink` first."
    ))
}

/// Like [`find_termlink_bin`], but first runs `cargo build -p termlink`
/// (once per test process) so the binary reflects the current git state.
/// Required for parity tests that compare git-derived metadata (commit,
/// version) between the CLI binary and a freshly-recompiled MCP test crate.
///
/// T-3679: the build uses the SAME target dir and profile as the running
/// test binary. It used to be `--release` into the default target dir: from a
/// cold target dir that is a second full build (~18 min on .107, holding the
/// parity tests' ENV_LOCK the whole time), and with a custom `--target-dir`
/// it built into one directory while `find_termlink_bin` read another. In a
/// `cargo test --workspace` run cargo has already built this binary, so the
/// build is a no-op of a second or two.
///
/// Skips the build when `TERMLINK_BIN` is set (assumes the override is
/// authoritative and may be a cross-compiled artifact).
pub fn find_termlink_bin_fresh() -> Result<PathBuf, String> {
    if std::env::var("TERMLINK_BIN").is_ok() {
        return find_termlink_bin();
    }
    static BUILT: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();
    let build_result = BUILT.get_or_init(|| {
        let mut args: Vec<String> = vec!["build".into(), "-p".into(), "termlink".into(), "--quiet".into()];
        if let Some(profile_dir) = current_profile_dir() {
            let profile = profile_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("debug")
                .to_string();
            if let Some(target_dir) = profile_dir.parent() {
                args.push("--target-dir".into());
                args.push(target_dir.to_string_lossy().into_owned());
            }
            // cargo names the dev profile's directory "debug".
            let profile = if profile == "debug" { "dev".to_string() } else { profile };
            args.push("--profile".into());
            args.push(profile);
        }
        let output = Command::new("cargo")
            .args(&args)
            .output()
            .map_err(|e| format!("spawn cargo build: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "cargo {} failed: stderr={}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    });
    match build_result {
        Ok(()) => find_termlink_bin(),
        Err(e) => Err(e.clone()),
    }
}

// ---------------------------------------------------------------------------
// Tests for the test utils themselves
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// T-3755: what the child actually receives, read back with `env`. The
    /// control variable (not project/identity) must survive, so the test fails
    /// if the helper cleared the whole environment instead of scrubbing.
    #[test]
    fn termlink_cmd_scrubs_project_and_identity_vars() {
        // SAFETY: unique-to-this-test names (PROJECT_ROOT is read by no other
        // test in this crate); set before the child is spawned.
        unsafe {
            std::env::set_var("PROJECT_ROOT", "/t3755/elsewhere");
            std::env::set_var("CLAUDE_PROJECT_DIR", "/t3755/elsewhere");
            std::env::set_var("TERMLINK_AGENT_ID", "t3755-leaked-agent");
            std::env::set_var("FW_T3755_MARK", "leaked");
            std::env::set_var("T3755_CONTROL_KEEP", "kept");
        }
        let rt = Path::new("/t3755/runtime");
        let out = termlink_cmd(Path::new("/usr/bin/env"), rt).output().expect("run env");
        let text = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = text.lines().collect();
        for leaked in ["PROJECT_ROOT=", "CLAUDE_PROJECT_DIR=", "TERMLINK_AGENT_ID=", "FW_T3755_MARK="] {
            assert!(!lines.iter().any(|l| l.starts_with(leaked)), "{leaked} reached the child:\n{text}");
        }
        assert!(lines.contains(&"T3755_CONTROL_KEEP=kept"), "control var lost:\n{text}");
        assert!(lines.contains(&"TERMLINK_RUNTIME_DIR=/t3755/runtime"), "runtime dir not set:\n{text}");
        assert!(lines.contains(&"TERMLINK_HINT_CANDIDATE_DIRS="), "hint dirs not set:\n{text}");
    }

    #[test]
    fn scrub_list_names_project_vars_and_prefixes_only() {
        for v in termlink_session::child_env::PROJECT_VARS {
            assert!(is_scrubbed_var(v), "{v}");
        }
        assert!(is_scrubbed_var("TERMLINK_IDENTITY_DIR"));
        assert!(is_scrubbed_var("CLAUDECODE"));
        for kept in ["HOME", "PATH", "TMPDIR", "RUST_LOG", "FRAMEWORK_ROOT", "XTERMLINK_X"] {
            assert!(!is_scrubbed_var(kept), "{kept} must pass through");
        }
    }

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
        assert!(reg.socket_path().exists());
        _handle.abort();
    }
}
