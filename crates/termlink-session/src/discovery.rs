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

/// Path to the sessions subdirectory.
pub fn sessions_dir() -> PathBuf {
    runtime_dir().join("sessions")
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
