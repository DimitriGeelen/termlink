//! Bounded, best-effort NDJSON event logs (T-3465, R5a of the T-2250 telemetry plane).
//!
//! Three signals are logged locally by default: offline-queue flaps
//! (`queue.log`, next to `outbound.sqlite`, normally `~/.termlink/`), hub circuit-breaker
//! trips (`circuit-breaker.jsonl` under the hub runtime dir) and reconnects
//! (`reconnect.log`, next to the queue). All go through here:
//!
//! - **Never blocks the caller.** `append` is a `try_send` onto one writer thread
//!   (same shape as the hub's `rpc_audit::enqueue`), so a stalled disk cannot hold a
//!   tokio worker. A full queue drops the line and counts it.
//! - **Bounded.** The writer uses `audit_append::append_line_capped`: at the cap the
//!   file is renamed to `<name>.1` (one backup), so disk use is about 2x the cap.
//!   Cap defaults to 1 MiB; `TERMLINK_EVENT_LOG_MAX_BYTES` overrides, `0` disables rotation.
//! - **Best-effort.** A write failure never reaches the caller; it produces one
//!   warning line (then every 100th) so it is not silent but cannot flood.
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::audit_append::append_line_capped;

/// Default size cap per log file (1 MiB); the backup `.1` may hold the same again.
pub const DEFAULT_MAX_BYTES: u64 = 1024 * 1024;
const QUEUE_CAP: usize = 1024;

enum Msg {
    Line(PathBuf, String, u64),
    Barrier(SyncSender<()>),
}

static TX: OnceLock<Option<SyncSender<Msg>>> = OnceLock::new();
static DROPPED: AtomicU64 = AtomicU64::new(0);
static FAILED: AtomicU64 = AtomicU64::new(0);

/// Lines dropped because the writer queue was full.
pub fn dropped_total() -> u64 {
    DROPPED.load(Ordering::Relaxed)
}

/// Resolved size cap (env override, else [`DEFAULT_MAX_BYTES`]).
pub fn max_bytes() -> u64 {
    std::env::var("TERMLINK_EVENT_LOG_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_BYTES)
}

fn write_one(path: &Path, line: &str, cap: u64) {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = append_line_capped(path, line, cap) {
        let n = FAILED.fetch_add(1, Ordering::Relaxed);
        if n.is_multiple_of(100) {
            eprintln!("# event log: write failed for {}: {e} (best-effort, continuing)", path.display());
        }
    }
}

fn run(rx: Receiver<Msg>) {
    for m in rx {
        match m {
            Msg::Line(p, l, cap) => write_one(&p, &l, cap),
            Msg::Barrier(done) => {
                let _ = done.send(());
            }
        }
    }
}

fn tx() -> Option<&'static SyncSender<Msg>> {
    TX.get_or_init(|| {
        let (tx, rx) = sync_channel::<Msg>(QUEUE_CAP);
        std::thread::Builder::new()
            .name("event-log".into())
            .spawn(move || run(rx))
            .map_err(|e| eprintln!("# event log: writer thread not started ({e}); writing inline"))
            .ok()
            .map(|_| tx)
    })
    .as_ref()
}

/// Queue one NDJSON line for `path` with the default cap. Never blocks, never fails.
pub fn append(path: &Path, line: String) {
    append_capped(path, line, max_bytes());
}

/// As [`append`] with an explicit cap (tests).
pub fn append_capped(path: &Path, line: String, cap: u64) {
    let Some(tx) = tx() else {
        write_one(path, &line, cap);
        return;
    };
    match tx.try_send(Msg::Line(path.to_path_buf(), line, cap)) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
            let n = DROPPED.fetch_add(1, Ordering::Relaxed) + 1;
            if n == 1 || n.is_multiple_of(100) {
                eprintln!("# event log: writer behind, {n} line(s) dropped");
            }
        }
    }
}

/// Block until every line queued so far has been written. For tests and shutdown.
pub fn flush() {
    if let Some(tx) = tx() {
        let (d, r) = sync_channel(1);
        if tx.send(Msg::Barrier(d)).is_ok() {
            let _ = r.recv();
        }
    }
}

/// Current wall-clock as RFC3339 (UTC, second resolution).
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86400, secs % 86400);
    // Civil-from-days (Howard Hinnant).
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_land_and_file_is_bounded_with_one_backup() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.log");
        let line = "x".repeat(99);
        for _ in 0..50 {
            append_capped(&p, line.clone(), 1000);
        }
        flush();
        let live = std::fs::metadata(&p).unwrap().len();
        assert!(live <= 1000 + 101, "live file exceeded cap + one line: {live}");
        let bak = crate::audit_append::rotated_path(&p);
        assert!(bak.exists(), "rotation should have produced a backup");
        assert!(std::fs::metadata(&bak).unwrap().len() <= 1000 + 101);
        // never more than 2 data files (plus the T-3615 rotation lock, which holds no data)
        let data = std::fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| e.as_ref().unwrap().path() != crate::audit_append::lock_path(&p))
            .count();
        assert_eq!(data, 2);
    }

    #[test]
    fn write_failure_is_swallowed() {
        let dir = tempfile::tempdir().unwrap();
        // parent "dir" is a regular file → create_dir_all/open fail
        let f = dir.path().join("file");
        std::fs::write(&f, b"x").unwrap();
        append_capped(&f.join("sub").join("x.log"), "{}".into(), 1000);
        flush(); // must not panic or hang
    }

    #[test]
    fn rfc3339_shape() {
        let s = now_rfc3339();
        assert_eq!(s.len(), 20);
        assert!(s.ends_with('Z') && s.as_bytes()[10] == b'T');
    }
}
