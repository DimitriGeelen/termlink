// Shared git-derived version scheme for every crate's build.rs (T-3774).
//
// `include!("../version-derive.rs")` from crates/<crate>/build.rs. One copy, so the four
// crates linked into one binary can never report two different versions again.
//
// Scheme (input: `git describe --tags --long --abbrev=9 --match 'v[0-9]*'`):
//
//   v0.12.11-0-ga2fcb16f8   (on the tag)        -> 0.12.11
//   v0.12.11-4-ga2fcb16f8   (4 commits after)   -> 0.12.11+4.ga2fcb16f8
//   v0.13.0-rc1-2-gdeadbeef (pre-release tag)    -> 0.13.0-rc1+2.gdeadbeef
//
// A release reports exactly its tag. A dev build reports the release it sits on, the number
// of commits since it and its commit, as SemVer build metadata. That string is unique per
// commit and orders after its base tag and before the next one under `sort -V` and under
// scripts/lib/version-cmp.sh (which breaks the SemVer build-metadata tie on N). It is the
// same shape the framework itself uses for FW_VERSION between releases (AEF T-4055).
//
// The scheme it replaces (`major.minor.N`) collided with real releases and went DOWN at every
// tag: v0.12.11+4 reported 0.12.4, the number of the release v0.12.4 (PL-243).

/// Map `git describe --long` output to the version string. `None` on anything unparseable,
/// so the caller falls back to the Cargo.toml version rather than inventing one.
#[allow(dead_code)]
pub fn version_from_describe(desc: &str) -> Option<String> {
    let desc = desc.trim();
    let desc = desc.strip_prefix('v').unwrap_or(desc);
    // Split from the right: <base>-<N>-g<sha>. The base itself may contain '-' (rc tags).
    let mut parts = desc.rsplitn(3, '-');
    let sha_part = parts.next()?;
    let n_part = parts.next()?;
    let base = parts.next()?;
    let sha = sha_part.strip_prefix('g')?;
    if sha.is_empty() || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n: u64 = n_part.parse().ok()?;
    let core = base.split('-').next()?;
    let nums: Vec<&str> = core.split('.').collect();
    if nums.len() != 3 || nums.iter().any(|s| s.is_empty() || !s.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    if n == 0 {
        Some(base.to_string())
    } else {
        Some(format!("{base}+{n}.g{sha}"))
    }
}

/// Run git and derive the version for the current checkout (`None` outside a git repo or
/// with no `v*` tag in reach).
#[allow(dead_code)]
pub fn git_derived_version() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["describe", "--tags", "--long", "--abbrev=9", "--match", "v[0-9]*"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    version_from_describe(&String::from_utf8(output.stdout).ok()?)
}

#[cfg(test)]
mod tests {
    use super::version_from_describe as v;

    #[test]
    fn exact_tag_is_the_tag() {
        assert_eq!(v("v0.12.11-0-ga2fcb16f8").as_deref(), Some("0.12.11"));
    }

    #[test]
    fn after_tag_names_base_count_and_commit() {
        assert_eq!(v("v0.12.11-4-ga2fcb16f8\n").as_deref(), Some("0.12.11+4.ga2fcb16f8"));
    }

    #[test]
    fn dev_build_never_collides_with_an_older_release() {
        // The T-3774 case: v0.12.11 + 4 used to report "0.12.4", the old release's number.
        let got = v("v0.12.11-4-ga2fcb16f8").unwrap();
        assert_ne!(got, "0.12.4");
        assert!(got.starts_with("0.12.11+"), "{got}");
        // and v0.12.10 + 1 used to report "0.12.1"
        assert!(v("v0.12.10-1-g60f3d57f7").unwrap().starts_with("0.12.10+1."));
    }

    #[test]
    fn same_count_different_commit_is_a_different_version() {
        assert_ne!(v("v0.12.11-4-gaaaaaaaaa"), v("v0.12.11-4-gbbbbbbbbb"));
    }

    #[test]
    fn prerelease_tag_keeps_its_suffix() {
        assert_eq!(v("v0.13.0-rc1-0-gdeadbeef0").as_deref(), Some("0.13.0-rc1"));
        assert_eq!(v("v0.13.0-rc1-2-gdeadbeef0").as_deref(), Some("0.13.0-rc1+2.gdeadbeef0"));
    }

    #[test]
    fn garbage_is_none_not_a_guess() {
        for bad in ["", "v0.12.11", "a2fcb16f8", "v0.12-4-gabc", "v0.12.11-x-gabc", "v0.12.11-4-abc", "v0.12.11-4-gxyz"] {
            assert_eq!(v(bad), None, "{bad:?}");
        }
    }
}
