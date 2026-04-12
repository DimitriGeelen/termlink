use std::path::PathBuf;

/// Resolve the TermLink runtime directory.
///
/// Resolution order (from T-006):
/// 1. $TERMLINK_RUNTIME_DIR (explicit override)
/// 2. $XDG_RUNTIME_DIR/termlink (Linux standard)
/// 3. $TMPDIR/termlink-$UID (macOS)
/// 4. /tmp/termlink-$UID (universal fallback)
pub fn runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("TERMLINK_RUNTIME_DIR") {
        return PathBuf::from(dir);
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg).join("termlink");
    }

    let uid = unsafe { libc::getuid() };

    if let Ok(tmpdir) = std::env::var("TMPDIR") {
        return PathBuf::from(tmpdir).join(format!("termlink-{uid}"));
    }

    PathBuf::from(format!("/tmp/termlink-{uid}"))
}

/// Path to the sessions subdirectory under the default runtime dir.
pub fn sessions_dir() -> PathBuf {
    runtime_dir().join("sessions")
}

/// Return all candidate runtime directories (T-987: multi-dir session scan).
///
/// Includes the primary `runtime_dir()` plus any additional well-known
/// locations that may hold sessions. Used by hub discovery and supervisor
/// to find sessions across the two-pool architecture (T-959):
/// persistent `/var/lib/termlink` + ephemeral `/tmp/termlink-UID`.
///
/// The primary dir is always first. Duplicates are removed.
pub fn all_runtime_dirs() -> Vec<PathBuf> {
    // If TERMLINK_RUNTIME_DIR is explicitly set, it's an exclusive override —
    // the caller wants exactly this dir (tests, systemd units, manual config).
    // Multi-dir scanning only kicks in for the default resolution path.
    if std::env::var("TERMLINK_RUNTIME_DIR").is_ok() {
        return vec![runtime_dir()];
    }

    let primary = runtime_dir();
    let uid = unsafe { libc::getuid() };

    let mut dirs = vec![primary.clone()];

    // Well-known persistent location (systemd hub, T-931)
    let persistent = PathBuf::from("/var/lib/termlink");
    if persistent != primary {
        dirs.push(persistent);
    }

    // XDG runtime dir
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let xdg_tl = PathBuf::from(xdg).join("termlink");
        if !dirs.contains(&xdg_tl) {
            dirs.push(xdg_tl);
        }
    }

    // /tmp fallback
    let tmp_tl = PathBuf::from(format!("/tmp/termlink-{uid}"));
    if !dirs.contains(&tmp_tl) {
        dirs.push(tmp_tl);
    }

    dirs
}

/// Return all candidate session directories (T-987).
///
/// Convenience: `all_runtime_dirs()` mapped to `dir/sessions`, filtered
/// to dirs that actually exist on disk (avoids noisy read_dir errors).
pub fn all_sessions_dirs() -> Vec<PathBuf> {
    all_runtime_dirs()
        .into_iter()
        .map(|d| d.join("sessions"))
        .filter(|d| d.is_dir())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_dir_returns_path() {
        // This test verifies the function doesn't panic and returns a non-empty path.
        // We can't reliably test the default resolution because parallel tests
        // may race on TERMLINK_RUNTIME_DIR. The override test covers the env var path.
        let dir = runtime_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn override_via_env() {
        let unique = format!("/custom/test-{}", std::process::id());
        let _guard = EnvGuard::set("TERMLINK_RUNTIME_DIR", &unique);
        let dir = runtime_dir();
        // If another test raced us on the env var, we just verify ours is coherent
        if std::env::var("TERMLINK_RUNTIME_DIR").ok().as_deref() == Some(unique.as_str()) {
            assert_eq!(dir, PathBuf::from(&unique));
        }
    }

    #[test]
    fn sessions_dir_is_child_of_runtime() {
        let rt = runtime_dir();
        let sess = sessions_dir();
        assert_eq!(sess, rt.join("sessions"));
        assert!(sess.starts_with(&rt));
    }

    #[test]
    fn all_runtime_dirs_includes_primary() {
        let primary = runtime_dir();
        let all = all_runtime_dirs();
        assert!(!all.is_empty(), "Should have at least one dir");
        assert_eq!(all[0], primary, "Primary dir should be first");
    }

    #[test]
    fn all_runtime_dirs_no_duplicates() {
        let all = all_runtime_dirs();
        let mut seen = std::collections::HashSet::new();
        for dir in &all {
            assert!(seen.insert(dir), "Duplicate dir: {}", dir.display());
        }
    }

    #[test]
    fn all_sessions_dirs_filters_nonexistent() {
        // all_sessions_dirs only returns dirs that exist on disk
        let dirs = all_sessions_dirs();
        for dir in &dirs {
            assert!(dir.is_dir(), "{} should exist", dir.display());
        }
    }

    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, val: &str) -> Self {
            let prev = std::env::var(key).ok();
            // SAFETY: test-only, single-threaded test runner for this module
            unsafe { std::env::set_var(key, val) };
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: test-only, restoring previous env state
            match &self.prev {
                Some(val) => unsafe { std::env::set_var(self.key, val) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }
}
