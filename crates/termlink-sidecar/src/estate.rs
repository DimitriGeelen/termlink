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
            Self::NotMarked(p) => write!(f, "no valid {MARKER} marker above {p}"),
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
