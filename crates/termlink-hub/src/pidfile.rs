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
///
/// T-3340: "Running" means the PID is alive AND is a termlink process. A pidfile
/// whose PID was reused by an unrelated process reads Stale, so `hub restart` /
/// `hub stop` never signal that process.
pub fn check(pidfile: &Path) -> PidfileStatus {
    match read_pid(pidfile) {
        None => PidfileStatus::NotRunning,
        Some(pid) => {
            if liveness::process_exists(pid) && is_termlink_process(pid) {
                PidfileStatus::Running(pid)
            } else {
                PidfileStatus::Stale(pid)
            }
        }
    }
}

/// T-3340: is `pid` a termlink process? Reads the process command line where the
/// platform exposes one (procfs). Without procfs (macOS) it cannot tell, and
/// answers true so behaviour there is unchanged: existence alone, as before.
pub fn is_termlink_process(pid: u32) -> bool {
    let proc_root = Path::new("/proc");
    if !proc_root.join("self").join("cmdline").exists() {
        return true;
    }
    match fs::read(proc_root.join(pid.to_string()).join("cmdline")) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).contains("termlink"),
        Err(_) => false,
    }
}

/// T-3340: every runtime dir a hub for this user may live in, in the order the
/// binary resolves them (`discovery::runtime_dir`), plus the systemd-managed
/// `/var/lib/termlink` (T-935). Deduplicated.
pub fn candidate_runtime_dirs() -> Vec<PathBuf> {
    let uid = unsafe { libc::getuid() };
    let mut dirs = vec![PathBuf::from("/var/lib/termlink")];
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join("termlink"));
    }
    if let Ok(tmpdir) = std::env::var("TMPDIR") {
        dirs.push(PathBuf::from(tmpdir).join(format!("termlink-{uid}")));
    }
    dirs.push(PathBuf::from(format!("/tmp/termlink-{uid}")));
    let mut seen = Vec::new();
    for d in dirs {
        if !seen.contains(&d) {
            seen.push(d);
        }
    }
    seen
}

/// T-3340: live hubs in `dirs` other than the one in `own_dir`, as (dir, pid).
pub fn other_live_hubs(own_dir: &Path, dirs: &[PathBuf]) -> Vec<(PathBuf, u32)> {
    dirs.iter()
        .filter(|d| d.as_path() != own_dir)
        .filter_map(|d| match check(&d.join("hub.pid")) {
            PidfileStatus::Running(pid) => Some((d.clone(), pid)),
            _ => None,
        })
        .collect()
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

    /// T-3340: a pidfile naming a live process that is not termlink reads Stale.
    /// PID 1 is always alive and is init/systemd, never termlink. Skipped where
    /// the platform has no procfs (the check then cannot tell, by design).
    #[test]
    fn check_reused_pid_of_non_termlink_process_is_stale() {
        if !Path::new("/proc/self/cmdline").exists() {
            return;
        }
        let path = test_pidfile();
        fs::write(&path, "1").unwrap();
        assert_eq!(check(&path), PidfileStatus::Stale(1));
        let _ = fs::remove_file(&path);
    }

    /// T-3340: our own test binary is a termlink process (termlink_hub-*), so a
    /// pidfile naming it reads Running.
    #[test]
    fn check_own_termlink_pid_is_running() {
        let path = test_pidfile();
        write(&path).unwrap();
        assert_eq!(check(&path), PidfileStatus::Running(std::process::id()));
        let _ = fs::remove_file(&path);
    }

    /// T-3340: other_live_hubs finds a live hub in another dir and ignores its own
    /// dir, empty dirs and stale pidfiles.
    #[test]
    fn other_live_hubs_finds_only_live_hubs_elsewhere() {
        let base = std::env::temp_dir().join(format!("tl-3340-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed)));
        let own = base.join("own");
        let live = base.join("live");
        let stale = base.join("stale");
        let empty = base.join("empty");
        for d in [&own, &live, &stale, &empty] {
            fs::create_dir_all(d).unwrap();
        }
        fs::write(own.join("hub.pid"), std::process::id().to_string()).unwrap();
        fs::write(live.join("hub.pid"), std::process::id().to_string()).unwrap();
        fs::write(stale.join("hub.pid"), "4194303").unwrap(); // above pid_max default: never alive
        let dirs = vec![own.clone(), live.clone(), stale.clone(), empty.clone()];
        let found = other_live_hubs(&own, &dirs);
        assert_eq!(found, vec![(live.clone(), std::process::id())]);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn candidate_runtime_dirs_include_systemd_and_tmp_default() {
        let dirs = candidate_runtime_dirs();
        let uid = unsafe { libc::getuid() };
        assert_eq!(dirs[0], PathBuf::from("/var/lib/termlink"));
        assert!(dirs.contains(&PathBuf::from(format!("/tmp/termlink-{uid}"))));
        let mut dedup = dirs.clone();
        dedup.dedup();
        assert_eq!(dedup.len(), dirs.len());
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
