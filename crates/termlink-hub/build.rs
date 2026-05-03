//! Build script: derive version from git tags, mirroring termlink-cli/build.rs.
//!
//! Without this, `env!("CARGO_PKG_VERSION")` in router.rs::handle_hub_version
//! resolved to the workspace Cargo.toml's hardcoded "0.9.0", so every hub in
//! the fleet reported "0.9.0" to `fleet doctor` regardless of actual freshness
//! (T-1458). The CLI side has had a build.rs since T-648 / T-1057; the hub
//! crate was just missing the equivalent.
//!
//! Re-run trigger paths and rationale match termlink-cli/build.rs verbatim;
//! see that file for the historical bug context (T-1057).

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
