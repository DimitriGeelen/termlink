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
}

// T-3774: one scheme for every crate, see crates/version-derive.rs.
include!("../version-derive.rs");
