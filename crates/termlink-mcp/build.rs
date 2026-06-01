//! Build script: derive version from git tags so `termlink_version` MCP
//! tool returns the same `version`/`commit`/`target` as `termlink version
//! --json` (T-1912 — T-1909 third-catch convergence).
//!
//! Mirror of `crates/termlink-cli/build.rs`. The CLI and MCP server share
//! the same `env!("CARGO_PKG_VERSION")` / `option_env!("GIT_COMMIT")` /
//! `option_env!("BUILD_TARGET")` pattern. Without this script the
//! `termlink-mcp` crate reports its own Cargo.toml version (0.9.0) and
//! `unknown` for commit/target — operator-visible drift from the CLI.
//!
//! If a fourth crate needs identical build-info behavior, extract these
//! contents to a shared `build-info` helper crate. With three copies the
//! duplication cost is bounded and the indirection cost of extraction is
//! not yet justified.
//!
//! See `crates/termlink-cli/build.rs` for the detailed rerun-trigger
//! rationale (T-1057) — the same logic applies here.

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

    if let Some(version) = git_derived_version() {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
    }

    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        && output.status.success()
    {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("cargo:rustc-env=GIT_COMMIT={hash}");
    }

    if let Ok(target) = std::env::var("TARGET") {
        println!("cargo:rustc-env=BUILD_TARGET={target}");
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
        Some(desc.to_string())
    }
}
