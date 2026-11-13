//! Build script that derives the crate version from git tags, falling back to the Cargo.toml version.
//! Build script: derive version from git tags (matching framework T-648 pattern).
//!
//! - Tagged commit `v0.8.0` → version `0.8.0`
//! - 5 commits after tag → version `0.8.0+5.g<sha9>` (T-3774; was `0.8.5`, which collided with release v0.8.5)
//! - No tags / not a git repo → falls back to Cargo.toml version
//!
//! # Re-run triggers (T-1057)
//!
//! Cargo only re-invokes build.rs when one of the paths listed below changes.
//! Each path targets a distinct git event:
//!
//! | Path                       | Changes on                               |
//! |----------------------------|------------------------------------------|
//! | `.git/HEAD`                | `git switch` (symbolic ref rewrite)      |
//! | `.git/logs/HEAD`           | **Every** HEAD movement — commit, merge, |
//! |                            | rebase, reset, pull, switch (appended).  |
//! |                            | This is the critical trigger for picking |
//! |                            | up new commits on the current branch.    |
//! | `.git/refs/heads`          | Directory mtime bumps when a local       |
//! |                            | branch tip is written (covers commits    |
//! |                            | even in worktrees where logs/HEAD may    |
//! |                            | be relocated).                           |
//! | `.git/refs/tags`           | `git tag` / `git tag -d` (new tags move  |
//! |                            | the derived version semver).             |
//! | `.git/packed-refs`         | `git gc` / `git pack-refs` (packs the    |
//! |                            | loose refs into a single file).          |
//!
//! Historical bug: prior to T-1057 we only watched `.git/HEAD` and
//! `.git/refs/tags`. `.git/HEAD` contains `ref: refs/heads/main` as a
//! stable pointer — the file itself is NOT rewritten on commit. Result:
//! `cargo build` never re-ran build.rs after the first invocation, and the
//! binary reported a frozen version number forever. `cargo install --git`
//! happened to work (clean target, fresh build.rs run) but incremental dev
//! builds silently drifted.
//!
//! Missing watched paths are tolerated by cargo — if `.git/logs/HEAD` does
//! not exist (shallow clone, tarball build), it just means "never changed"
//! for rerun purposes. No build failure.

use std::process::Command;

const GIT_RERUN_PATHS: &[&str] = &[
    "../version-derive.rs",
    "../../.git/HEAD",
    "../../.git/logs/HEAD",
    "../../.git/refs/heads",
    "../../.git/refs/tags",
    "../../.git/packed-refs",
];

fn main() {
    for p in GIT_RERUN_PATHS {
        println!("cargo:rerun-if-changed={p}");
    }

    if let Some(version) = git_derived_version() {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
    }

    // Embed git commit hash
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        && output.status.success()
    {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=GIT_COMMIT={hash}");
    }

    // Embed build target triple
    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=BUILD_TARGET={target}");
    }
}

// T-3774: one scheme for every crate, see crates/version-derive.rs.
include!("../version-derive.rs");
