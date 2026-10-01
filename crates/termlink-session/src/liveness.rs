use std::path::Path;

use crate::registration::Registration;

/// Check if a registered session is still alive.
///
/// Uses the hybrid approach from T-006:
/// 1. PID check (fast path, microseconds)
/// 2. Socket file existence check (confirms socket wasn't cleaned)
///
/// Full socket probe + identity ping will be added when the control plane
/// listener is implemented.
pub fn is_alive(reg: &Registration) -> bool {
    // Fast path: check if PID exists
    if !process_exists(reg.pid) {
        return false;
    }

    // Confirm socket file still exists
    // For Unix sockets, check if the socket file exists on disk.
    // For non-Unix transports, skip the file check (will need a different probe).
    match reg.addr.as_unix_path() {
        Some(path) => path.exists(),
        None => true, // non-Unix transport — cannot file-check, assume alive
    }
}

/// Check if a process with the given PID exists.
pub fn process_exists(pid: u32) -> bool {
    // kill(pid, 0) checks existence without sending a signal.
    // Returns 0 if process exists and we have permission to signal it.
    // Returns -1 with ESRCH if process doesn't exist.
    // Returns -1 with EPERM if process exists but we can't signal it (still alive).
    let ret = unsafe { libc::kill(pid as i32, 0) };
    if ret == 0 {
        return true;
    }
    // EPERM means process exists but we lack permission — still alive
    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    errno == libc::EPERM
}

/// Remove stale registration artifacts: control socket, JSON, and data-plane socket.
///
/// T-3293: this used to remove only the socket and the JSON. A `--shell` session
/// also owns `<socket>.data` (the data plane, see `data_server::data_socket_path`),
/// and once the JSON is gone nothing can find that file again — 10,144 of them had
/// accumulated on one host. The data socket is removed from its derived path and,
/// if different, from the path recorded in `metadata.data_socket`.
pub fn cleanup_stale(reg: &Registration, sessions_dir: &Path) {
    let json_path = Registration::json_path(sessions_dir, &reg.id);
    if let Some(path) = reg.addr.as_unix_path() {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(crate::data_server::data_socket_path(path));
    }
    if let Some(ref recorded) = reg.metadata.data_socket {
        let _ = std::fs::remove_file(recorded);
    }
    let _ = std::fs::remove_file(&json_path);
    tracing::info!(
        session_id = %reg.id,
        pid = reg.pid,
        "Cleaned stale session registration"
    );
}

/// Suffix of a data-plane socket next to its control socket (`<id>.sock.data`).
const DATA_SOCKET_SUFFIX: &str = ".sock.data";

/// Reap orphaned data-plane sockets: `<id>.sock.data` files in `sessions_dir` whose
/// `<id>.json` registration no longer exists and whose mtime is older than `grace`.
///
/// T-3293: these are left by sessions that died by any route other than their own
/// clean shutdown, and by every cleaner that predates the fix above. A file with a
/// registration beside it is never touched (the live-or-dead decision belongs to
/// `is_alive` + `cleanup_stale`), and neither is a young one: a session creates its
/// files in sequence, so a just-created data socket can briefly precede its JSON.
/// Returns how many were removed and how many could NOT be (T-3299: a sweep that
/// cannot write — e.g. a read-only mount under systemd `ProtectSystem=strict` —
/// must not look like a sweep with nothing to do).
pub fn reap_orphan_data_sockets(sessions_dir: &Path, grace: std::time::Duration) -> ReapOutcome {
    let mut out = ReapOutcome::default();
    let entries = match std::fs::read_dir(sessions_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(id) = name.strip_suffix(DATA_SOCKET_SUFFIX) else { continue };
        if id.is_empty() || sessions_dir.join(format!("{id}.json")).exists() {
            continue;
        }
        let old_enough = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age >= grace);
        if !old_enough {
            continue;
        }
        match std::fs::remove_file(entry.path()) {
            Ok(()) => out.removed += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                out.failed += 1;
                if out.first_error.is_none() {
                    out.first_error = Some(format!("{}: {e}", entry.path().display()));
                }
            }
        }
    }
    out
}

/// Result of [`reap_orphan_data_sockets`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReapOutcome {
    /// Orphaned data sockets removed.
    pub removed: usize,
    /// Orphans that could not be removed (permission, read-only filesystem, ...).
    pub failed: usize,
    /// The first failure, for the log line.
    pub first_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_exists() {
        assert!(process_exists(std::process::id()));
    }

    #[test]
    fn nonexistent_pid() {
        // PID 4194304 is above Linux's default pid_max and unlikely to exist
        // On macOS pid_max is 99998
        assert!(!process_exists(4_000_000));
    }

    #[test]
    fn is_alive_dead_pid() {
        use crate::identity::SessionId;
        use crate::registration::SessionConfig;
        use std::path::PathBuf;

        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/nonexistent.sock");
        let mut reg = Registration::new(id, config, socket);
        reg.pid = 4_000_000; // definitely dead

        assert!(!is_alive(&reg));
    }

    #[test]
    fn is_alive_with_missing_socket() {
        use crate::identity::SessionId;
        use crate::registration::SessionConfig;
        use std::path::PathBuf;

        let id = SessionId::generate();
        let config = SessionConfig::default();
        // Socket path that doesn't exist on disk
        let socket = PathBuf::from("/tmp/termlink-test-nonexistent.sock");
        let reg = Registration::new(id, config, socket);
        // PID is current process (alive), but socket doesn't exist
        assert!(!is_alive(&reg));
    }

    #[test]
    fn cleanup_removes_files() {
        use crate::identity::SessionId;
        use crate::registration::SessionConfig;

        let dir = std::env::temp_dir().join(format!(
            "termlink-test-cleanup-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let id = SessionId::generate();
        let socket_path = dir.join(format!("{id}.sock"));
        let json_path = dir.join(format!("{id}.json"));

        // Create fake socket and json files
        std::fs::write(&socket_path, b"fake").unwrap();
        std::fs::write(&json_path, b"fake").unwrap();

        let config = SessionConfig::default();
        let reg = Registration::new(id.clone(), config, socket_path.clone());

        cleanup_stale(&reg, &dir);

        assert!(!socket_path.exists());
        assert!(!json_path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// T-3293: the data-plane socket is the file every cleaner used to leave behind.
    #[test]
    fn cleanup_removes_data_socket_too() {
        use crate::identity::SessionId;
        use crate::registration::SessionConfig;

        let dir = tempfile::tempdir().unwrap();
        let id = SessionId::generate();
        let socket_path = dir.path().join(format!("{id}.sock"));
        let json_path = dir.path().join(format!("{id}.json"));
        let data_path = crate::data_server::data_socket_path(&socket_path);
        let recorded = dir.path().join("elsewhere.data");
        for p in [&socket_path, &json_path, &data_path, &recorded] {
            std::fs::write(p, b"fake").unwrap();
        }
        let mut reg = Registration::new(id, SessionConfig::default(), socket_path.clone());
        reg.metadata.data_socket = Some(recorded.to_string_lossy().into_owned());

        cleanup_stale(&reg, dir.path());

        assert!(!socket_path.exists(), "control socket left behind");
        assert!(!json_path.exists(), "registration JSON left behind");
        assert!(!data_path.exists(), "derived <sock>.data left behind (the T-3293 leak)");
        assert!(!recorded.exists(), "metadata.data_socket path left behind");
    }

    #[test]
    fn reap_removes_old_orphan_data_socket() {
        let dir = tempfile::tempdir().unwrap();
        let orphan = dir.path().join("tl-orphan.sock.data");
        std::fs::write(&orphan, b"").unwrap();

        let n = reap_orphan_data_sockets(dir.path(), std::time::Duration::ZERO);

        assert_eq!(n.removed, 1);
        assert_eq!(n.failed, 0);
        assert!(!orphan.exists());
    }

    #[test]
    fn reap_never_touches_a_registered_session() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("tl-live.sock.data");
        let json = dir.path().join("tl-live.json");
        let sock = dir.path().join("tl-live.sock");
        for p in [&data, &json, &sock] {
            std::fs::write(p, b"").unwrap();
        }

        let n = reap_orphan_data_sockets(dir.path(), std::time::Duration::ZERO);

        assert_eq!(n, ReapOutcome::default());
        assert!(data.exists() && json.exists() && sock.exists());
    }

    /// T-3299: a delete that fails must be counted and reported, not swallowed —
    /// the read-only /tmp under ProtectSystem=strict looked like "nothing to do".
    #[test]
    fn reap_counts_a_failed_delete_instead_of_swallowing_it() {
        let dir = tempfile::tempdir().unwrap();
        // A non-empty DIRECTORY named like an orphan: remove_file fails on it
        // (EISDIR/EPERM), deterministically and without needing root to drop.
        let stuck = dir.path().join("tl-stuck.sock.data");
        std::fs::create_dir(&stuck).unwrap();
        std::fs::write(stuck.join("x"), b"").unwrap();
        let orphan = dir.path().join("tl-ok.sock.data");
        std::fs::write(&orphan, b"").unwrap();

        let n = reap_orphan_data_sockets(dir.path(), std::time::Duration::ZERO);

        assert_eq!(n.removed, 1);
        assert_eq!(n.failed, 1, "the failed delete must be counted");
        assert!(n.first_error.as_deref().is_some_and(|e| e.contains("tl-stuck.sock.data")));
        assert!(!orphan.exists());
    }

    #[test]
    fn reap_spares_a_young_orphan_and_other_files() {
        let dir = tempfile::tempdir().unwrap();
        let young = dir.path().join("tl-new.sock.data");
        let unrelated = dir.path().join("tl-x.sock");
        let odd = dir.path().join(".sock.data");
        for p in [&young, &unrelated, &odd] {
            std::fs::write(p, b"").unwrap();
        }

        let n = reap_orphan_data_sockets(dir.path(), std::time::Duration::from_secs(3600));

        assert_eq!(n, ReapOutcome::default(), "a data socket younger than the grace may precede its JSON");
        assert!(young.exists() && unrelated.exists() && odd.exists());
    }
}
