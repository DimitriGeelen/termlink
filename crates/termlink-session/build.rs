//! Build script: derive version from git tags, so `SessionMetadata::termlink_version`
//! records the binary that registered the session rather than a frozen constant.
//!
//! # Why this crate needs one (T-2744)
//!
//! `registration.rs` writes `env!("CARGO_PKG_VERSION")` into every session's
//! metadata. Without this script that expands to *this crate's* `Cargo.toml`
//! version — `0.9.0`, unchanged since it was written — so every session ever
//! registered recorded the same string regardless of which binary produced it.
//! Measured before the fix: live sessions reported `0.9.0` while
//! `termlink --version` reported `0.11.720`.
//!
//! `termlink-cli` and `termlink-mcp` already carried this derivation. This crate
//! did not, and it is the one that persists the value to disk for other tools to
//! read. Keep the three in step: a crate that reports a user-visible version and
//! has no build.rs is reporting its Cargo.toml, which is not the project's
//! version scheme.
//!
//! Semantics match the sibling scripts exactly:
//! - Tagged commit `v0.8.0` → `0.8.0`
//! - 5 commits after tag → `0.8.5`
//! - No tags / not a git repo → falls back to the Cargo.toml version
//!
//! # Re-run triggers
//!
//! Mirrors `termlink-cli/build.rs`, including the T-1057 fix. Watching only
//! `.git/HEAD` is not enough: that file holds `ref: refs/heads/main` and is not
//! rewritten on commit, so build.rs never re-ran and the version froze at
//! whatever the first build saw. `.git/logs/HEAD` is the trigger that catches
//! every HEAD movement. Missing paths are tolerated by cargo (a shallow clone or
//! tarball build simply means "never changed").

use std::process::Command;

const GIT_RERUN_PATHS: &[&str] = &[
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

    // Cargo sets CARGO_PKG_VERSION in the build script's own environment to the
    // Cargo.toml value, before any override below applies. Re-exporting it under
    // a second name gives the test suite something to compare against, so the
    // regression test can assert "the recorded version is not the Cargo.toml
    // constant" without hardcoding a version literal that would itself go stale.
    if let Ok(cargo_toml_version) = std::env::var("CARGO_PKG_VERSION") {
        println!("cargo:rustc-env=CARGO_TOML_VERSION={cargo_toml_version}");
    }

    if let Some(version) = git_derived_version() {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
        // Lets the test tell "derivation ran" apart from "no git here, fell back
        // to Cargo.toml" — a tarball or shallow build is a legitimate fallback,
        // not a regression, and must not fail the suite.
        println!("cargo:rustc-env=VERSION_IS_GIT_DERIVED=1");
    }
}

fn git_derived_version() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--match", "v[0-9]*"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let desc = String::from_utf8(output.stdout).ok()?.trim().to_string();
    let desc = desc.strip_prefix('v').unwrap_or(&desc);

    if desc.contains('-') {
        // v0.8.0-47-gabcdef → 0.8.47
        let parts: Vec<&str> = desc.splitn(3, '-').collect();
        if parts.len() >= 2 {
            let base = parts[0];
            let commits = parts[1];
            let major_minor = base.rsplitn(2, '.').last()?;
            Some(format!("{major_minor}.{commits}"))
        } else {
            None
        }
    } else {
        // Exact tag: v0.8.0 → 0.8.0
        Some(desc.to_string())
    }
}
