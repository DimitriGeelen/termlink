//! Size-capped O_APPEND line writer shared by the hub's `rpc_audit` and the
//! `invocation_audit` sink (T-2251; moved here by T-3033 so the session daemon
//! can use it without depending on termlink-hub).
//!
//! Cross-process safety comes from `O_APPEND`: a whole-line write smaller than
//! `PIPE_BUF` is atomic, so concurrent writers in different processes cannot
//! interleave. Callers must never read-modify-write the file.
use std::path::{Path, PathBuf};

/// The rotated-backup path for `path` — append `.1` to the file name
/// (e.g. `rpc-audit.jsonl` → `rpc-audit.jsonl.1`). A single backup generation.
pub fn rotated_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".1");
    PathBuf::from(os)
}

/// T-2251: pure, lock-free capped append. If `max_bytes > 0` and the current
/// file is already at/over the cap, rotate it (rename → `.1`, overwriting any
/// prior backup) before appending to a fresh file. `max_bytes == 0` disables
/// rotation (append-forever). Checking size BEFORE the write bounds the live
/// file to `cap + one line` and the backup to the same — total ~2× cap.
pub fn append_line_capped(path: &Path, line: &str, max_bytes: u64) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    if max_bytes > 0
        && let Ok(meta) = std::fs::metadata(path)
        && meta.len() >= max_bytes
    {
        // Best-effort rotate: a rename failure (e.g. the file vanished under us)
        // must not lose the line — fall through and append to whatever exists.
        let _ = std::fs::rename(path, rotated_path(path));
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}
