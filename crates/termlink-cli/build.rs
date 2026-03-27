//! Build script: derive version from git tags (matching framework T-648 pattern).
//!
//! - Tagged commit `v0.8.0` → version `0.8.0`
//! - 5 commits after tag → version `0.8.5`
//! - No tags / not a git repo → falls back to Cargo.toml version

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/tags");

    if let Some(version) = git_derived_version() {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={version}");
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
