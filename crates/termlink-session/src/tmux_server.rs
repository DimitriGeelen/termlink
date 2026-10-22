//! Which tmux server a TermLink session lives on (T-3552, SEV-1 T-3549 ruling A).
//!
//! The host's DEFAULT tmux server is shared by every project and by the human, and whoever starts
//! it seeds its global environment. Each project therefore gets its own server, addressed only
//! with `tmux -L <name>`. The name is derived from the project (the same rule `termlink register`
//! enforces, [`crate::child_env::project_of`]) and validated before use: TermLink never passes
//! `-S`, and never hands tmux a name this module did not build (an unchecked path to `-S` once
//! stripped the execute bits from `/`, T-3425). Interface: `docs/reports/T-3552-per-project-tmux-interface.md`.

use sha2::{Digest, Sha256};
use std::path::Path;

/// The server for sessions whose working directory lies in no project.
pub const HOST_SERVER: &str = "tl-host";

/// Longest slug kept from the project directory's name.
const SLUG_MAX: usize = 24;

/// The project directory's basename as a slug: lowercase, runs of anything outside `[a-z0-9]`
/// collapsed to one `-`, trimmed, cut to [`SLUG_MAX`]. `p` when nothing usable is left.
fn slug(name: &str) -> String {
    let mut s = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            s.push(c);
        } else if !s.ends_with('-') {
            s.push('-');
        }
    }
    let s: String = s.trim_matches('-').chars().take(SLUG_MAX).collect();
    let s = s.trim_end_matches('-').to_string();
    if s.is_empty() { "p".to_string() } else { s }
}

/// `tl-<slug>-<hash8>` for a project root: `hash8` is the first 8 hex digits of SHA-256 of the
/// canonical root, so two checkouts with the same basename never share a server.
pub fn server_name_for_root(root: &Path) -> String {
    let canon = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let digest = Sha256::digest(canon.to_string_lossy().as_bytes());
    let hash8: String = digest.iter().take(4).map(|b| format!("{b:02x}")).collect();
    let base = canon.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    format!("tl-{}-{hash8}", slug(&base))
}

/// The server for a session whose working directory is `dir`: its project's server, or
/// [`HOST_SERVER`] when `dir` lies in no project.
pub fn server_name_for(dir: &Path) -> String {
    match crate::child_env::project_of(dir) {
        Some(root) => server_name_for_root(&root),
        None => HOST_SERVER.to_string(),
    }
}

/// True when `name` is a server name TermLink may pass to `tmux -L`: [`HOST_SERVER`], or
/// `tl-<slug>-<hash8>` with a 1..=24 character `[a-z0-9-]` slug and exactly 8 hex digits.
pub fn is_valid_server_name(name: &str) -> bool {
    if name == HOST_SERVER {
        return true;
    }
    let Some(rest) = name.strip_prefix("tl-") else { return false };
    let Some((slug, hash)) = rest.rsplit_once('-') else { return false };
    let slug_ok = !slug.is_empty()
        && slug.len() <= SLUG_MAX
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let hash_ok = hash.len() == 8 && hash.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
    slug_ok && hash_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_is_safe_and_bounded() {
        assert_eq!(slug("010-termlink"), "010-termlink");
        assert_eq!(slug("My Project (v2)"), "my-project-v2");
        assert_eq!(slug("../../etc"), "etc");
        assert_eq!(slug("..."), "p");
        assert_eq!(slug(&"x".repeat(40)).len(), SLUG_MAX);
        assert!(!slug("abcdefghijklmnopqrstuvw-yz").ends_with('-'), "a cut never leaves a trailing dash");
    }

    #[test]
    fn names_are_per_project_and_valid() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("proj");
        let b = d.path().join("other").join("proj"); // same basename, different checkout
        for p in [&a, &b] {
            std::fs::create_dir_all(p.join("sub")).unwrap();
            std::fs::write(p.join(".framework.yaml"), "").unwrap();
        }
        let na = server_name_for(&a.join("sub"));
        let nb = server_name_for(&b);
        assert!(na.starts_with("tl-proj-") && nb.starts_with("tl-proj-"), "{na} {nb}");
        assert_ne!(na, nb, "two checkouts with one basename must not share a server");
        assert_eq!(na, server_name_for(&a), "a subdirectory is on its project's server");
        assert!(is_valid_server_name(&na) && is_valid_server_name(&nb));
    }

    #[test]
    fn no_project_means_the_host_server() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(server_name_for(d.path()), HOST_SERVER);
    }

    #[test]
    fn a_worktree_shares_its_main_checkouts_server() {
        let d = tempfile::tempdir().unwrap();
        let main = d.path().join("main");
        let wt = d.path().join("wt");
        std::fs::create_dir_all(main.join(".git/worktrees/wt")).unwrap();
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(main.join(".framework.yaml"), "").unwrap();
        std::fs::write(wt.join(".framework.yaml"), "").unwrap();
        std::fs::write(wt.join(".git"), format!("gitdir: {}/.git/worktrees/wt\n", main.display())).unwrap();
        assert_eq!(server_name_for(&wt), server_name_for(&main));
    }

    #[test]
    fn unsafe_names_are_refused() {
        for bad in [
            "", "default", "tl-", "tl-x", "tl--0123abcd", "tl-a-0123abc", "tl-a-0123abcde", "tl-a-0123ABCD",
            "tl-a/b-0123abcd", "tl-../x-0123abcd", "/tmp/tmux-0/default", "tl-a b-0123abcd",
            "tl-abcdefghijklmnopqrstuvwxy-0123abcd",
        ] {
            assert!(!is_valid_server_name(bad), "{bad:?} must be refused");
        }
        assert!(is_valid_server_name("tl-010-termlink-0123abcd"));
        assert!(is_valid_server_name(HOST_SERVER));
    }
}
