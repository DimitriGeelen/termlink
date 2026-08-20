//! T-2607 — unified identity/trust-plane directory resolution.
//!
//! The client identity/trust plane — the TOFU `known_hubs` store
//! ([`crate::tofu::known_hubs_path`]), the durable offline outbound queue
//! ([`crate::offline_queue::default_queue_path`]), and the await-ack retry DB
//! ([`crate::ack_retry::default_tracker_path`]) — must live in ONE private,
//! per-user directory.
//!
//! Before T-2607 each helper independently did `HOME.unwrap_or("/tmp")`, which
//! silently relocated trust material to a world-writable, reboot-volatile
//! `/tmp/.termlink` whenever `HOME` was unset (a systemd unit without
//! `Environment=HOME` / `User=`, an `env -i` invocation, a minimal container).
//! That is three defects at once: a Reliability-#2 "silent failure" (no error
//! surfaced), a Portability-#4 environment coupling, and — because `/tmp` is
//! world-readable/writable — a cross-user pin-tamper / collision vector on the
//! trust store. `tofu.rs` additionally ignored `TERMLINK_IDENTITY_DIR`, so the
//! trust store could not be co-located with the rest of the plane.
//!
//! Resolution order (identical for every identity-plane path helper):
//!   1. `$TERMLINK_IDENTITY_DIR`     — explicit override, honored everywhere
//!   2. `$XDG_STATE_HOME/termlink`   — XDG Base Directory spec
//!   3. `$HOME/.termlink`            — conventional
//!   4. last resort (all unset):     — loud, UID-namespaced, NOT world-writable
//!      `<tmpdir>/termlink-<uid>`  created `0700`, with a one-time
//!      `tracing::error!` warning that pins will not survive reboot.
//!
//! Step 4 never uses the shared world-writable `/tmp/.termlink`; it mirrors the
//! UID-namespaced [`crate::discovery`] `runtime_dir()` fallback so a mis-configured
//! host degrades to a private, locked-down (mode `0700`) directory instead of a
//! shared one. The resolver is **infallible** by design (T-2607 Decision:
//! Option B): the ~30 existing call sites — including the infallible
//! [`crate::tofu::KnownHubStore::default_store`] — need no `Result` ripple, and
//! the "no silent failure" requirement is met by the loud error log plus the
//! locked-down permissions rather than by a hard refusal that would break every
//! legitimate `HOME`-less invocation.

use std::path::{Path, PathBuf};
use std::sync::Once;

/// Pure resolution core — no env reads, no filesystem side effects — so the
/// policy is deterministically unit-testable without touching process globals
/// (mirrors the T-2269 `match_profile_by_address` extraction: env mutation in
/// tests is racy under parallel execution).
///
/// Returns the resolved identity DIRECTORY plus `last_resort = true` when it
/// fell through to the UID-namespaced tmp path (the loud branch). An
/// exported-but-empty env value (`HOME=`) is treated as unset, never as the
/// root directory `/`.
fn resolve_identity_dir_from(
    identity_dir: Option<&str>,
    xdg_state_home: Option<&str>,
    home: Option<&str>,
    temp_dir: &Path,
    uid: u32,
) -> (PathBuf, bool) {
    if let Some(d) = identity_dir.filter(|s| !s.is_empty()) {
        // TERMLINK_IDENTITY_DIR is used verbatim as the dir (mirrors the
        // existing offline_queue convention — no `.termlink` suffix).
        return (PathBuf::from(d), false);
    }
    if let Some(x) = xdg_state_home.filter(|s| !s.is_empty()) {
        return (PathBuf::from(x).join("termlink"), false);
    }
    if let Some(h) = home.filter(|s| !s.is_empty()) {
        return (PathBuf::from(h).join(".termlink"), false);
    }
    (temp_dir.join(format!("termlink-{uid}")), true)
}

/// Resolve the private per-user identity/trust directory. See the module docs
/// for the resolution ladder.
///
/// Infallible. On the last-resort branch it emits a one-time loud
/// `tracing::error!` and creates the directory with mode `0700`, so trust
/// material is never left in a world-writable location and the fallback is
/// never silent.
pub fn resolve_identity_dir() -> PathBuf {
    let identity = std::env::var("TERMLINK_IDENTITY_DIR").ok();
    let xdg = std::env::var("XDG_STATE_HOME").ok();
    let home = std::env::var("HOME").ok();
    let temp = std::env::temp_dir();
    let uid = unsafe { libc::getuid() };

    let (dir, last_resort) = resolve_identity_dir_from(
        identity.as_deref(),
        xdg.as_deref(),
        home.as_deref(),
        &temp,
        uid,
    );

    if last_resort {
        warn_last_resort_once(&dir);
        harden_last_resort_dir(&dir);
    }
    dir
}

static LAST_RESORT_WARNED: Once = Once::new();

fn warn_last_resort_once(dir: &Path) {
    LAST_RESORT_WARNED.call_once(|| {
        tracing::error!(
            identity_dir = %dir.display(),
            "no HOME, XDG_STATE_HOME, or TERMLINK_IDENTITY_DIR set — the identity/trust \
             plane (TOFU pins, offline queue, ack-retry DB) is falling back to a \
             UID-namespaced temp directory; pins will NOT survive reboot. Set HOME or \
             TERMLINK_IDENTITY_DIR to a persistent private directory."
        );
    });
}

/// Create the last-resort directory with mode `0700` so trust material is never
/// world-readable/writable (defect 1b — cross-user pin tamper). Best-effort: if
/// creation fails the situation surfaces later as an ordinary file-open error,
/// which is still loud — the point is only that we never silently produce a
/// world-writable trust store.
fn harden_last_resort_dir(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if std::fs::create_dir_all(dir).is_ok() {
        let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins_and_is_used_verbatim() {
        let (dir, lr) = resolve_identity_dir_from(
            Some("/data/id"),
            Some("/xdg"),
            Some("/home/u"),
            Path::new("/tmp"),
            1000,
        );
        // TERMLINK_IDENTITY_DIR is the dir itself — no `.termlink` suffix.
        assert_eq!(dir, PathBuf::from("/data/id"));
        assert!(!lr);
    }

    #[test]
    fn xdg_state_home_beats_home() {
        let (dir, lr) =
            resolve_identity_dir_from(None, Some("/xdg"), Some("/home/u"), Path::new("/tmp"), 1000);
        assert_eq!(dir, PathBuf::from("/xdg/termlink"));
        assert!(!lr);
    }

    #[test]
    fn home_used_when_no_override_or_xdg() {
        let (dir, lr) =
            resolve_identity_dir_from(None, None, Some("/home/u"), Path::new("/tmp"), 1000);
        // Behavior-preserving: matches the pre-T-2607 `$HOME/.termlink` shape.
        assert_eq!(dir, PathBuf::from("/home/u/.termlink"));
        assert!(!lr);
    }

    #[test]
    fn empty_env_values_are_treated_as_unset_not_as_root() {
        // An exported-but-empty `HOME=` must NOT resolve to `/.termlink`.
        let (dir, lr) =
            resolve_identity_dir_from(Some(""), Some(""), Some(""), Path::new("/var/tmp"), 1000);
        assert_eq!(dir, PathBuf::from("/var/tmp/termlink-1000"));
        assert!(lr);
    }

    // LOAD-BEARING (T-2607): with every env var unset the resolver must NOT land
    // in the shared, world-writable `/tmp/.termlink`. Temp-revert the step-4
    // branch to the old `unwrap_or("/tmp")` + `.termlink` shape and this test
    // FAILS — proving the guard is what keeps trust material out of the shared dir.
    #[test]
    fn all_unset_falls_back_to_uid_namespaced_tmp_never_shared_dot_termlink() {
        let (dir, lr) = resolve_identity_dir_from(None, None, None, Path::new("/tmp"), 4242);
        assert!(lr, "all-unset must flag last_resort (the loud path)");
        assert_eq!(dir, PathBuf::from("/tmp/termlink-4242"));
        // The specific regression guard: never the shared, world-writable dir.
        assert_ne!(dir, PathBuf::from("/tmp/.termlink"));
        assert!(
            !dir.ends_with(".termlink"),
            "must not be the shared .termlink dir"
        );
        assert!(
            dir.to_string_lossy().contains("termlink-4242"),
            "must be UID-namespaced"
        );
    }

    #[test]
    fn tmpdir_is_honored_for_the_last_resort_base() {
        // The impure wrapper feeds `std::env::temp_dir()` (which reads $TMPDIR);
        // the pure core just joins under whatever base it is given.
        let (dir, lr) =
            resolve_identity_dir_from(None, None, None, Path::new("/custom/tmp"), 7);
        assert_eq!(dir, PathBuf::from("/custom/tmp/termlink-7"));
        assert!(lr);
    }
}
