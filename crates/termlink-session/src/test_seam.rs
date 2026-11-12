//! T-3686 (06 AL-16, FCW-60; 05 E-163.1): the test-seam gate for the main lease.
//!
//! The `TERMLINK_MAIN_TEST_*` variables shorten the lease, quiet period and cooldown so tests
//! can drive takeovers in seconds. Before T-3686 they were obeyed by every binary, so a
//! leftover variable in a live session's environment could shrink main's lease to 1 s.
//!
//! The rule (06 5.16.c): a seam is honoured only when the process runs in a marked test estate;
//! for `termlink main` that is "HOME and the runtime directory lie in one tree marked by a
//! genuine `.termlink-estate` marker" (the C-1 marker the sidecar testkit writes, see
//! `termlink_sidecar::estate`). Outside an estate any variable with the seam prefix makes the
//! lease answer `unknown` (exit 2) with a reason naming the variable; its value is never read.
//!
//! The marker constants live here (not in `termlink-sidecar`) because this crate sits below
//! the sidecar, the CLI and the MCP server; `termlink_sidecar::estate` re-exports them.

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

/// Marker file name of a test estate, created only by test harnesses.
pub const ESTATE_MARKER: &str = ".termlink-estate";
/// First line of a genuine estate marker.
pub const ESTATE_MARKER_MAGIC: &str = "termlink-sidecar-estate v1";
/// Prefix of every main-lease seam. Any variable with it counts, known name or not.
pub const MAIN_SEAM_PREFIX: &str = "TERMLINK_MAIN_TEST_";

/// Nearest ancestor of `p` (after canonicalising) holding a genuine estate marker.
pub fn marked_estate_root(p: &Path) -> Option<PathBuf> {
    let canon = p.canonicalize().ok()?;
    canon
        .ancestors()
        .find(|anc| {
            std::fs::read_to_string(anc.join(ESTATE_MARKER))
                .map(|s| s.lines().next() == Some(ESTATE_MARKER_MAGIC))
                .unwrap_or(false)
        })
        .map(Path::to_path_buf)
}

/// True when `home` and `runtime` both lie inside the SAME marked estate.
pub fn in_estate(home: Option<&Path>, runtime: &Path) -> bool {
    let Some(home) = home else { return false };
    match (marked_estate_root(home), marked_estate_root(runtime)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// [`in_estate`] for this process: `HOME` and [`crate::discovery::runtime_dir`].
pub fn in_estate_from_env() -> bool {
    let home = std::env::var_os("HOME").filter(|v| !v.is_empty()).map(PathBuf::from);
    in_estate(home.as_deref(), &crate::discovery::runtime_dir())
}

/// Names (never values) of the main-lease seam variables present, sorted.
pub fn main_seams_set<I: IntoIterator<Item = (OsString, OsString)>>(vars: I) -> Vec<String> {
    let mut v: Vec<String> = vars
        .into_iter()
        .map(|(k, _)| k.to_string_lossy().into_owned())
        .filter(|k| k.starts_with(MAIN_SEAM_PREFIX))
        .collect();
    v.sort();
    v.dedup();
    v
}

/// A seam was found outside a test estate. Carries variable names only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeamRefused {
    pub vars: Vec<String>,
}

impl fmt::Display for SeamRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "test seam {} set outside a test estate (HOME and the runtime dir are not in one tree marked {}); \
             the main lease refuses to run with it: unset it (T-3686, FCW-60)",
            self.vars.join(", "),
            ESTATE_MARKER
        )
    }
}

impl std::error::Error for SeamRefused {}

#[cfg(test)]
mod tests {
    use super::*;

    fn mark(dir: &Path, magic: &str) {
        std::fs::write(dir.join(ESTATE_MARKER), format!("{magic}\n")).unwrap();
    }

    fn tmp(name: &str) -> PathBuf {
        let base = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("t3686-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn estate_needs_genuine_marker_over_both_paths() {
        let root = tmp("both");
        let (home, rt) = (root.join("home"), root.join("rt"));
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&rt).unwrap();
        assert!(!in_estate(Some(&home), &rt), "unmarked tree is not an estate");
        mark(&root, "something else");
        assert!(!in_estate(Some(&home), &rt), "a marker with the wrong magic is not an estate");
        mark(&root, ESTATE_MARKER_MAGIC);
        assert!(in_estate(Some(&home), &rt));
        assert!(!in_estate(None, &rt), "no HOME is not an estate");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn estate_refused_when_paths_split_across_trees() {
        let (a, b) = (tmp("split-a"), tmp("split-b"));
        mark(&a, ESTATE_MARKER_MAGIC);
        mark(&b, ESTATE_MARKER_MAGIC);
        assert!(!in_estate(Some(&a), &b), "two different estates are not one estate");
        let plain = tmp("split-plain");
        assert!(!in_estate(Some(&a), &plain), "runtime outside the estate");
        assert!(!in_estate(Some(&plain), &a), "HOME outside the estate");
        for d in [a, b, plain] {
            std::fs::remove_dir_all(d).unwrap();
        }
    }

    #[test]
    fn seams_listed_by_prefix_names_only() {
        let vars = vec![
            ("PATH".into(), "/bin".into()),
            ("TERMLINK_MAIN_TEST_LEASE_MS".into(), "1000".into()),
            ("TERMLINK_MAIN_TEST_SOMETHING_NEW".into(), "x".into()),
            ("TERMLINK_MAIN_PROJECT".into(), "p".into()),
        ];
        assert_eq!(
            main_seams_set(vars),
            vec!["TERMLINK_MAIN_TEST_LEASE_MS".to_string(), "TERMLINK_MAIN_TEST_SOMETHING_NEW".to_string()]
        );
        let msg = SeamRefused { vars: vec!["TERMLINK_MAIN_TEST_LEASE_MS".into()] }.to_string();
        assert!(msg.contains("TERMLINK_MAIN_TEST_LEASE_MS") && msg.contains("outside a test estate"), "{msg}");
    }
}
