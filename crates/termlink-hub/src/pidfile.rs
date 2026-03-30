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

/// Error returned when acquiring a pidfile fails.
#[derive(Debug)]
pub enum AcquireError {
    /// Another hub instance is already running with this PID.
    AlreadyRunning(u32),
    /// I/O error writing the pidfile.
    Io(io::Error),
}

impl std::fmt::Display for AcquireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning(pid) => {
                write!(f, "Hub is already running (PID {pid}). Use 'termlink hub stop' to stop it.")
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
}
