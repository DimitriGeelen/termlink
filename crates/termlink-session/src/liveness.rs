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

/// Remove stale registration artifacts (socket + JSON files).
pub fn cleanup_stale(reg: &Registration, sessions_dir: &Path) {
    let json_path = Registration::json_path(sessions_dir, &reg.id);
    if let Some(path) = reg.addr.as_unix_path() {
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_file(&json_path);
    tracing::info!(
        session_id = %reg.id,
        pid = reg.pid,
        "Cleaned stale session registration"
    );
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
}
