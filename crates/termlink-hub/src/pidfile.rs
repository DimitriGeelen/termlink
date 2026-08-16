//! Pidfile management for the hub daemon.
//!
//! Provides write/read/validate/remove lifecycle for `hub.pid` in the runtime directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use termlink_session::discovery;
use termlink_session::liveness;

/// Return the well-known hub pidfile path: `runtime_dir()/hub.pid`.
pub fn hub_pidfile_path() -> PathBuf {
    discovery::runtime_dir().join("hub.pid")
}

/// Status of an existing pidfile.
#[derive(Debug, PartialEq, Eq)]
pub enum PidfileStatus {
    /// No pidfile exists.
    NotRunning,
    /// Pidfile exists but the process is dead (stale).
    Stale(u32),
    /// Pidfile exists and the process is alive.
    Running(u32),
}

/// Check the status of the hub pidfile.
pub fn check(pidfile: &Path) -> PidfileStatus {
    match read_pid(pidfile) {
        None => PidfileStatus::NotRunning,
        Some(pid) => {
            if liveness::process_exists(pid) {
                PidfileStatus::Running(pid)
            } else {
                PidfileStatus::Stale(pid)
            }
        }
    }
}

/// Write the current process PID to the pidfile.
///
/// Creates parent directories if needed. Overwrites any existing pidfile.
pub fn write(pidfile: &Path) -> io::Result<()> {
    if let Some(parent) = pidfile.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(pidfile, format!("{}", std::process::id()))
}

/// Remove the pidfile if it exists.
pub fn remove(pidfile: &Path) {
    let _ = fs::remove_file(pidfile);
}

/// Acquire the pidfile for this process.
///
/// Returns `Ok(())` if the pidfile was written successfully.
/// Returns `Err` if another hub is already running.
/// Cleans up stale pidfiles automatically.
pub fn acquire(pidfile: &Path) -> Result<(), AcquireError> {
    match check(pidfile) {
        PidfileStatus::NotRunning => {
            write(pidfile).map_err(AcquireError::Io)?;
            Ok(())
        }
        PidfileStatus::Stale(old_pid) => {
            tracing::info!(stale_pid = old_pid, "Cleaning stale hub pidfile");
            remove(pidfile);
            write(pidfile).map_err(AcquireError::Io)?;
            Ok(())
        }
        PidfileStatus::Running(pid) => Err(AcquireError::AlreadyRunning(pid)),
    }
}

/// Is something actually LISTENING on `socket` right now?
///
/// T-2767. The pidfile is an *assertion* about liveness; the socket is the
/// *evidence*. They disagree whenever the pidfile is lost while the hub keeps
/// running — which is not hypothetical: on 2026-08-16 a hub supervised by
/// systemd was serving on `/var/lib/termlink/hub.sock` with no pidfile naming
/// it, so `check()` returned `NotRunning` and a second hub started cleanly and
/// took over both the pidfile and the socket. The result was a split brain on
/// ONE host — unix-socket clients reaching one instance, TCP clients the other,
/// with the same topic names resolving differently.
///
/// A successful connect proves a listener is accepting. `ECONNREFUSED` means
/// the socket FILE is left over from an unclean shutdown with nothing behind
/// it — the stale case, which must still start (that is what the pidfile-only
/// check was written for, and breaking it would trade one failure for another).
fn socket_has_listener(socket: &Path) -> bool {
    if !socket.exists() {
        return false;
    }
    std::os::unix::net::UnixStream::connect(socket).is_ok()
}

/// Acquire the pidfile, refusing if a hub is LIVE on `socket` even when the
/// pidfile does not say so.
///
/// This is [`acquire`] plus the evidence check. Prefer it at every real start
/// path; `acquire` is retained for callers that have no socket to probe.
pub fn acquire_with_socket(pidfile: &Path, socket: &Path) -> Result<(), AcquireError> {
    // An alive PID in the pidfile is already conclusive — report it as before,
    // because naming the PID is more actionable than naming the socket.
    if let PidfileStatus::Running(pid) = check(pidfile) {
        return Err(AcquireError::AlreadyRunning(pid));
    }
    // Pidfile says nobody is home. Believe the socket, not the file.
    if socket_has_listener(socket) {
        return Err(AcquireError::SocketAlive(socket.to_path_buf()));
    }
    acquire(pidfile)
}

/// Error returned when acquiring a pidfile fails.
#[derive(Debug)]
pub enum AcquireError {
    /// Another hub instance is already running with this PID.
    AlreadyRunning(u32),
    /// A hub is accepting on the socket even though the pidfile did not say so
    /// (T-2767) — a lost or stale pidfile in front of a live hub.
    SocketAlive(PathBuf),
    /// I/O error writing the pidfile.
    Io(io::Error),
}

impl std::fmt::Display for AcquireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning(pid) => {
                write!(f, "Hub is already running (PID {pid}). Use 'termlink hub stop' to stop it.")
            }
            Self::SocketAlive(sock) => {
                // Name the evidence and both remediations: the operator cannot
                // act on "already running" alone when the pidfile is the thing
                // that went missing, and a unit-supervised hub must not be
                // stopped with a bare `hub stop`.
                write!(
                    f,
                    "A hub is already accepting on {} (the pidfile did not name it — \
                     lost or stale pidfile in front of a live hub). Starting here would \
                     take over the socket and split the substrate in two. \
                     Check 'systemctl status termlink-hub' first: if it is unit-supervised, \
                     leave it alone; otherwise stop it with 'termlink hub stop'.",
                    sock.display()
                )
            }
            Self::Io(e) => write!(f, "Failed to write pidfile: {e}"),
        }
    }
}

impl std::error::Error for AcquireError {}

/// Read PID from a pidfile, returning None if the file doesn't exist or can't be parsed.
fn read_pid(pidfile: &Path) -> Option<u32> {
    fs::read_to_string(pidfile)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_pidfile() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(format!(
            "/tmp/tl-pidfile-test-{}-{}.pid",
            std::process::id(),
            n
        ))
    }

    #[test]
    fn check_no_pidfile() {
        let path = test_pidfile();
        let _ = fs::remove_file(&path);
        assert_eq!(check(&path), PidfileStatus::NotRunning);
    }

    #[test]
    fn write_and_read() {
        let path = test_pidfile();
        write(&path).unwrap();
        let pid = read_pid(&path).unwrap();
        assert_eq!(pid, std::process::id());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn check_running() {
        let path = test_pidfile();
        write(&path).unwrap();
        assert_eq!(check(&path), PidfileStatus::Running(std::process::id()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn check_stale() {
        let path = test_pidfile();
        // Write a PID that definitely doesn't exist
        fs::write(&path, "4000000").unwrap();
        assert_eq!(check(&path), PidfileStatus::Stale(4_000_000));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn acquire_fresh() {
        let path = test_pidfile();
        let _ = fs::remove_file(&path);
        acquire(&path).unwrap();
        assert_eq!(check(&path), PidfileStatus::Running(std::process::id()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn acquire_cleans_stale() {
        let path = test_pidfile();
        fs::write(&path, "4000000").unwrap();
        acquire(&path).unwrap();
        assert_eq!(check(&path), PidfileStatus::Running(std::process::id()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn acquire_rejects_running() {
        let path = test_pidfile();
        // Write our own PID (definitely alive)
        write(&path).unwrap();
        let result = acquire(&path);
        assert!(matches!(result, Err(AcquireError::AlreadyRunning(_))));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn remove_nonexistent_is_ok() {
        let path = test_pidfile();
        let _ = fs::remove_file(&path);
        remove(&path); // Should not panic
    }

    #[test]
    fn corrupt_pidfile_treated_as_not_running() {
        let path = test_pidfile();
        fs::write(&path, "not-a-number").unwrap();
        assert_eq!(check(&path), PidfileStatus::NotRunning);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn empty_pidfile_treated_as_not_running() {
        let path = test_pidfile();
        fs::write(&path, "").unwrap();
        assert_eq!(check(&path), PidfileStatus::NotRunning);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn whitespace_only_pidfile_treated_as_not_running() {
        let path = test_pidfile();
        fs::write(&path, "  \n  \t  ").unwrap();
        assert_eq!(check(&path), PidfileStatus::NotRunning);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn pid_with_trailing_newline_parses() {
        let path = test_pidfile();
        fs::write(&path, format!("{}\n", std::process::id())).unwrap();
        assert_eq!(check(&path), PidfileStatus::Running(std::process::id()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn overflow_pid_treated_as_not_running() {
        let path = test_pidfile();
        // u32::MAX + 1 overflows
        fs::write(&path, "4294967296").unwrap();
        assert_eq!(check(&path), PidfileStatus::NotRunning);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn negative_pid_treated_as_not_running() {
        let path = test_pidfile();
        fs::write(&path, "-1").unwrap();
        assert_eq!(check(&path), PidfileStatus::NotRunning);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn acquire_error_display() {
        let already = AcquireError::AlreadyRunning(12345);
        assert!(already.to_string().contains("12345"));
        assert!(already.to_string().contains("already running"));

        let io_err = AcquireError::Io(io::Error::new(io::ErrorKind::PermissionDenied, "nope"));
        assert!(io_err.to_string().contains("nope"));
    }

    #[test]
    fn acquire_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(AcquireError::AlreadyRunning(1));
        assert!(!err.to_string().is_empty());
    }

    // --- T-2767: the pidfile is an assertion, the socket is the evidence -----

    fn tmp_socket_path() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(format!(
            "/tmp/termlink-test-sock-{}-{}.sock",
            std::process::id(),
            n
        ))
    }

    /// The regression. A hub is LIVE on the socket while the pidfile does not
    /// name it — exactly the 2026-08-16 condition. Starting must refuse.
    #[test]
    fn live_socket_with_no_pidfile_refuses() {
        let sock = tmp_socket_path();
        let pidfile = test_pidfile();
        let _listener = std::os::unix::net::UnixListener::bind(&sock).unwrap();
        assert_eq!(check(&pidfile), PidfileStatus::NotRunning);

        let err = acquire_with_socket(&pidfile, &sock)
            .expect_err("a live listener must block acquisition even with no pidfile");
        match err {
            AcquireError::SocketAlive(p) => assert_eq!(p, sock),
            other => panic!("expected SocketAlive, got {other:?}"),
        }

        // LOAD-BEARING: the pre-fix predicate is still available, and on this
        // exact input it SUCCEEDS — which is the bug. Pinning the contrast here
        // means the test cannot pass for the wrong reason, and reverting the
        // call sites to `acquire` reintroduces a failure this suite reports.
        assert!(
            acquire(&pidfile).is_ok(),
            "pidfile-only acquire is expected to (wrongly) succeed here; if this \
             starts failing, the contrast this regression pins has changed"
        );

        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_file(&pidfile);
    }

    /// The case the pidfile-only check existed for: recovery after an unclean
    /// shutdown. A leftover socket FILE with nothing listening must still start.
    #[test]
    fn stale_socket_file_with_no_listener_still_starts() {
        let sock = tmp_socket_path();
        let pidfile = test_pidfile();
        // A socket file with no listener behind it — connect gives ECONNREFUSED.
        {
            let l = std::os::unix::net::UnixListener::bind(&sock).unwrap();
            drop(l);
        }
        assert!(sock.exists(), "the leftover socket file must remain for this test");

        acquire_with_socket(&pidfile, &sock)
            .expect("a dead socket file must not block startup — that is the unclean-shutdown case");

        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_file(&pidfile);
    }

    /// An absent socket path is not a listener.
    #[test]
    fn absent_socket_does_not_block() {
        let sock = tmp_socket_path();
        let pidfile = test_pidfile();
        assert!(!sock.exists());
        acquire_with_socket(&pidfile, &sock).expect("no socket at all must start cleanly");
        let _ = std::fs::remove_file(&pidfile);
    }

    /// A live PID in the pidfile still reports the PID, not the socket — naming
    /// the process is more actionable than naming the file.
    #[test]
    fn live_pidfile_still_reports_already_running() {
        let sock = tmp_socket_path();
        let pidfile = test_pidfile();
        let _listener = std::os::unix::net::UnixListener::bind(&sock).unwrap();
        write(&pidfile).unwrap(); // our own PID: alive by construction

        match acquire_with_socket(&pidfile, &sock) {
            Err(AcquireError::AlreadyRunning(pid)) => assert_eq!(pid, std::process::id()),
            other => panic!("expected AlreadyRunning, got {other:?}"),
        }

        let _ = std::fs::remove_file(&sock);
        let _ = std::fs::remove_file(&pidfile);
    }

    /// Directive #2: the refusal must be actionable, not a bare failure.
    #[test]
    fn socket_alive_message_names_socket_and_both_remediations() {
        let msg = AcquireError::SocketAlive(PathBuf::from("/var/lib/termlink/hub.sock")).to_string();
        assert!(msg.contains("/var/lib/termlink/hub.sock"), "must name the evidence: {msg}");
        assert!(msg.contains("systemctl status termlink-hub"), "must name the supervised case: {msg}");
        assert!(msg.contains("termlink hub stop"), "must name the unsupervised case: {msg}");
    }
}
