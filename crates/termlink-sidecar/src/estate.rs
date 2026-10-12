//! C-1 estate guard: refuse to run unless `HOME`, the runtime dir and the store
//! root all sit inside one throwaway tree marked by [`MARKER`].

use std::fmt;
use std::path::{Path, PathBuf};

/// Marker file name, created only by the test harness.
pub const MARKER: &str = ".termlink-estate";
/// First line of a genuine marker.
pub const MARKER_MAGIC: &str = "termlink-sidecar-estate v1";
/// Env var naming the sidecar store root.
pub const STORE_ENV: &str = "TERMLINK_SIDECAR_STORE";

#[derive(Debug, PartialEq, Eq)]
pub enum EstateError {
    Unset(&'static str),
    Unresolvable(String),
    NotMarked(String),
    Escapes { what: &'static str, root: String },
}

impl fmt::Display for EstateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unset(v) => write!(f, "{v} is not set"),
            Self::Unresolvable(p) => write!(f, "cannot resolve path {p}"),
            Self::NotMarked(p) => write!(f, "no valid {MARKER} (test estate) or {PRODUCTION_MARKER} (production) marker above {p}"),
            Self::Escapes { what, root } => {
                write!(f, "{what} is not inside the marked estate {root}")
            }
        }
    }
}

impl std::error::Error for EstateError {}

fn find_marked_root(p: &Path) -> Result<PathBuf, EstateError> {
    let canon = p
        .canonicalize()
        .map_err(|_| EstateError::Unresolvable(p.display().to_string()))?;
    for anc in canon.ancestors() {
        let m = anc.join(MARKER);
        if let Ok(s) = std::fs::read_to_string(&m)
            && s.lines().next() == Some(MARKER_MAGIC)
        {
            return Ok(anc.to_path_buf());
        }
    }
    Err(EstateError::NotMarked(canon.display().to_string()))
}

/// Check the three locations; returns the estate root.
pub fn guard(home: &Path, runtime: &Path, store: &Path) -> Result<PathBuf, EstateError> {
    let root = find_marked_root(home)?;
    for (what, p) in [("runtime dir", runtime), ("store root", store)] {
        let r = find_marked_root(p)?;
        if r != root {
            return Err(EstateError::Escapes {
                what,
                root: root.display().to_string(),
            });
        }
    }
    Ok(root)
}

/// Production root marker (PH-2 row P1). Created only by the operator's
/// `runme.sh`, never by the test harness.
pub const PRODUCTION_MARKER: &str = ".termlink-sidecar-root";
/// First line of a genuine production marker.
pub const PRODUCTION_MAGIC: &str = "termlink-sidecar-production v1";

/// Where a store may run: inside a throwaway estate (PH-1 rule) or under an
/// operator-marked production root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Estate(PathBuf),
    Production(PathBuf),
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Estate(_) => "estate",
            Mode::Production(_) => "production",
        }
    }
    pub fn root(&self) -> &Path {
        match self {
            Mode::Estate(r) | Mode::Production(r) => r,
        }
    }
}

/// Nearest ancestor of `p` carrying a genuine production marker, if any.
fn find_production_root(p: &Path) -> Option<PathBuf> {
    use std::os::unix::fs::MetadataExt;
    let canon = p.canonicalize().ok()?;
    for anc in canon.ancestors() {
        let m = anc.join(PRODUCTION_MARKER);
        let Ok(s) = std::fs::read_to_string(&m) else { continue };
        if s.lines().next() != Some(PRODUCTION_MAGIC) {
            continue;
        }
        // A marker at `/` would make every path production; a marker another
        // user wrote, or one others can rewrite, is not the operator's.
        let Ok(md) = std::fs::symlink_metadata(&m) else { continue };
        // SAFETY: geteuid has no preconditions and cannot fail.
        let euid = unsafe { libc::geteuid() };
        if anc.parent().is_none() || !md.file_type().is_file() || md.uid() != euid || md.mode() & 0o022 != 0 {
            return None;
        }
        return Some(anc.to_path_buf());
    }
    None
}

/// The PH-2 guard. A store under a production root runs in production mode,
/// provided HOME and the runtime dir are not inside a test estate (the two are
/// never mixed). Anything else must pass the PH-1 estate rule unchanged.
pub fn guard_any(home: &Path, runtime: &Path, store: &Path) -> Result<Mode, EstateError> {
    if let Some(root) = find_production_root(store) {
        for (what, p) in [("HOME", home), ("runtime dir", runtime)] {
            if let Ok(est) = find_marked_root(p) {
                return Err(EstateError::Escapes { what, root: est.display().to_string() });
            }
        }
        return Ok(Mode::Production(root));
    }
    guard(home, runtime, store).map(Mode::Estate)
}

/// Guard using `HOME`, `TERMLINK_RUNTIME_DIR` and `TERMLINK_SIDECAR_STORE`.
pub fn guard_any_from_env() -> Result<Mode, EstateError> {
    let get = |k: &'static str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .ok_or(EstateError::Unset(k))
    };
    guard_any(&get("HOME")?, &get("TERMLINK_RUNTIME_DIR")?, &get(STORE_ENV)?)
}

/// Guard using `HOME`, `TERMLINK_RUNTIME_DIR` and `TERMLINK_SIDECAR_STORE`.
pub fn guard_from_env() -> Result<PathBuf, EstateError> {
    let get = |k: &'static str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .ok_or(EstateError::Unset(k))
    };
    guard(
        &get("HOME")?,
        &get("TERMLINK_RUNTIME_DIR")?,
        &get(STORE_ENV)?,
    )
}
