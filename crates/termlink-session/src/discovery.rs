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

/// Outcome of scanning the candidate session directories (T-2791).
///
/// Two OUTCOMES, deliberately not one list. `usable` is what a caller can
/// enumerate; `unreadable` is a candidate that exists but which this process
/// is not permitted to read. Collapsing them is the defect this type exists
/// to prevent — see `scan_sessions_dirs`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SessionsDirScan {
    /// Directories that exist and can be enumerated.
    pub usable: Vec<PathBuf>,
    /// Directories that exist but are not readable by this process
    /// (`EACCES`, typically a `0700` dir owned by another uid).
    pub unreadable: Vec<PathBuf>,
}

impl SessionsDirScan {
    /// True when at least one candidate was hidden by permissions — the
    /// signal that "no sessions found" must NOT be reported as a complete
    /// and empty inventory.
    pub fn has_unreadable(&self) -> bool {
        !self.unreadable.is_empty()
    }
}

/// Scan the candidate session directories, distinguishing ABSENT from
/// PRESENT-BUT-UNREADABLE (T-2791).
///
/// `all_sessions_dirs()` previously filtered on `Path::is_dir()`, which
/// returns `false` when the underlying `stat` fails — including `EACCES`.
/// That made a runtime dir the caller cannot read indistinguishable from one
/// that does not exist, so a non-root process pointed at a `0700` root-owned
/// `TERMLINK_RUNTIME_DIR` discovered zero sessions and every surface above
/// reported a confident, complete, empty inventory (999-AEF OBS-302).
///
/// A missing directory is still filtered silently — that is the legitimate
/// intent of the original filter, and the common case (`/tmp/termlink-$UID`
/// simply not created yet). A permission denial is *reported*, because it
/// means the answer is unknown rather than empty. Directive #2: a plausible
/// wrong answer is worse than an error.
/// What the scan decided about one candidate directory (T-2791).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateOutcome {
    /// Exists and can be enumerated.
    Usable,
    /// Exists but this process may not read it — the answer is UNKNOWN,
    /// not empty.
    Unreadable,
    /// Absent (or not a directory): filtered silently, as it always was.
    Skip,
}

/// Pure decision for one candidate directory (T-2791).
///
/// Kept free of filesystem access so the `EACCES` branch is testable — the
/// process running the tests is frequently root, and root bypasses the very
/// permission check this function exists to honour, so a chmod-based test
/// would silently never exercise it. Same shape and reasoning as
/// `decide_unix_peer` in `termlink-hub::server` (T-2448).
///
/// `stat` is `Ok(is_dir)` or the `ErrorKind` from `fs::metadata`.
/// `read_dir` is consulted only when `stat` reports a directory, because
/// traversing a directory (`--x`) and listing it (`r--`) are separate
/// permissions and only an actual `read_dir` proves the latter.
pub(crate) fn classify_candidate(
    stat: Result<bool, std::io::ErrorKind>,
    read_dir: Option<Result<(), std::io::ErrorKind>>,
) -> CandidateOutcome {
    use std::io::ErrorKind::PermissionDenied;

    match stat {
        Ok(true) => match read_dir {
            Some(Err(PermissionDenied)) => CandidateOutcome::Unreadable,
            // Any other read_dir error (ENOTDIR after a race, EIO, ...) is
            // transient or exotic: keep the dir so the caller's own read
            // surfaces the real error rather than us guessing here.
            _ => CandidateOutcome::Usable,
        },
        // Exists but is not a directory — indistinguishable from absent for
        // our purposes, and not a permission problem.
        Ok(false) => CandidateOutcome::Skip,
        Err(PermissionDenied) => CandidateOutcome::Unreadable,
        // NotFound and everything else: silently skipped, as before.
        Err(_) => CandidateOutcome::Skip,
    }
}

pub fn scan_sessions_dirs() -> SessionsDirScan {
    let mut scan = SessionsDirScan::default();

    for dir in all_runtime_dirs() {
        let sessions = dir.join("sessions");

        // `metadata` follows symlinks and fails with EACCES when any parent
        // component is not traversable — which is exactly the `0700` case,
        // where the denial is on the parent rather than on `sessions` itself.
        let stat = std::fs::metadata(&sessions)
            .map(|md| md.is_dir())
            .map_err(|e| e.kind());

        let read_dir = if stat == Ok(true) {
            Some(std::fs::read_dir(&sessions).map(|_| ()).map_err(|e| e.kind()))
        } else {
            None
        };

        match classify_candidate(stat, read_dir) {
            CandidateOutcome::Usable => scan.usable.push(sessions),
            CandidateOutcome::Unreadable => scan.unreadable.push(sessions),
            CandidateOutcome::Skip => {}
        }
    }

    scan
}

/// Return all candidate session directories (T-987).
///
/// Convenience: `all_runtime_dirs()` mapped to `dir/sessions`, filtered to
/// dirs that exist AND can be enumerated.
///
/// This drops the `unreadable` half of [`scan_sessions_dirs`], so a caller
/// that needs to tell "no sessions" from "cannot see the sessions" must use
/// [`scan_sessions_dirs`] directly (T-2791).
pub fn all_sessions_dirs() -> Vec<PathBuf> {
    scan_sessions_dirs().usable
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

    // ── T-2791: ABSENT and PRESENT-BUT-UNREADABLE are different answers ──────
    //
    // The pre-T-2791 scan was `.filter(|d| d.is_dir())`. `Path::is_dir()`
    // returns `false` when the underlying stat fails, so EACCES and ENOENT
    // both collapsed to "dropped" — a non-root process pointed at a 0700
    // root-owned runtime dir discovered zero sessions, and every surface
    // above reported a confident, COMPLETE, empty inventory (999-AEF
    // OBS-302). These pin the distinction the old filter could not express.

    #[test]
    fn permission_denied_at_stat_is_unreadable_not_skipped() {
        // The load-bearing case. Under the old `is_dir()` filter this path
        // was indistinguishable from ENOENT below.
        assert_eq!(
            classify_candidate(Err(std::io::ErrorKind::PermissionDenied), None),
            CandidateOutcome::Unreadable
        );
    }

    #[test]
    fn permission_denied_at_read_dir_is_unreadable() {
        // Traversable (`--x`) but not listable (`r--`): stat succeeds and
        // reports a directory, and only read_dir reveals the denial. The old
        // filter never called read_dir at all, so it reported this as usable
        // and the emptiness surfaced one layer up as "no sessions".
        assert_eq!(
            classify_candidate(Ok(true), Some(Err(std::io::ErrorKind::PermissionDenied))),
            CandidateOutcome::Unreadable
        );
    }

    #[test]
    fn absent_dir_is_still_skipped_silently() {
        // The legitimate intent of the original filter, preserved: an
        // uncreated /tmp/termlink-$UID must not become a reported problem.
        assert_eq!(
            classify_candidate(Err(std::io::ErrorKind::NotFound), None),
            CandidateOutcome::Skip
        );
    }

    #[test]
    fn existing_listable_dir_is_usable() {
        assert_eq!(
            classify_candidate(Ok(true), Some(Ok(()))),
            CandidateOutcome::Usable
        );
    }

    #[test]
    fn existing_non_directory_is_skipped() {
        // A file where a directory was expected is not a permission problem.
        assert_eq!(classify_candidate(Ok(false), None), CandidateOutcome::Skip);
    }

    #[test]
    fn non_permission_read_dir_error_stays_usable() {
        // Exotic/transient errors keep the dir so the CALLER's own read
        // surfaces the real error, rather than this function guessing that
        // the dir is inaccessible.
        assert_eq!(
            classify_candidate(Ok(true), Some(Err(std::io::ErrorKind::Other))),
            CandidateOutcome::Usable
        );
    }

    #[test]
    fn all_sessions_dirs_drops_the_unreadable_half() {
        // `all_sessions_dirs()` is the back-compat surface and returns only
        // `usable` — this pins that a caller needing to tell "no sessions"
        // from "cannot see the sessions" must use `scan_sessions_dirs()`.
        let scan = scan_sessions_dirs();
        assert_eq!(all_sessions_dirs(), scan.usable);
        assert_eq!(scan.has_unreadable(), !scan.unreadable.is_empty());
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
