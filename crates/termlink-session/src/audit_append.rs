//! Size-capped O_APPEND line writer shared by the hub's rpc audit and the invocation audit sinks.
//! Size-capped O_APPEND line writer shared by the hub's `rpc_audit` and the
//! `invocation_audit` sink (T-2251; moved here by T-3033 so the session daemon
//! can use it without depending on termlink-hub).
//!
//! Cross-process safety: each line goes out as ONE `O_APPEND` write (line and
//! newline in one buffer), so concurrent writers in different processes cannot
//! interleave, and rotation is serialised across processes by a `flock` on
//! `<path>.lock` (T-3615). Callers must never read-modify-write the file.
use std::path::{Path, PathBuf};

/// The rotated-backup path for `path` — append `.1` to the file name
/// (e.g. `rpc-audit.jsonl` → `rpc-audit.jsonl.1`). A single backup generation.
pub fn rotated_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".1");
    PathBuf::from(os)
}

/// The cross-process rotation lock for `path`: `<path>.lock` beside it (T-3615).
pub fn lock_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".lock");
    PathBuf::from(os)
}

/// T-2251 / T-3615: capped append. If `max_bytes > 0` and the current file is
/// already at/over the cap, rotate it (rename → `.1`, overwriting any prior
/// backup) before appending to a fresh file. `max_bytes == 0` disables rotation
/// (append-forever). Checking size BEFORE the write bounds the live file to
/// `cap + one line` and the backup to the same — total ~2× cap.
///
/// T-3615 (external review, synthesis item 30) fixed two ways this lost rows:
/// - The line and its `\n` went out as TWO writes, so a concurrent writer could
///   land between them and glue two records into one unparseable line. The line
///   and newline are now one buffer and one `write_all`.
/// - Rotation was only serialised by a process-local mutex. Two processes that
///   both saw the file over the cap could both rename: the second rename moved
///   the fresh, nearly empty file over the full `.1` backup (a whole generation,
///   up to 32 MiB of invocation rows or 100 MiB of rpc rows, gone). Rotation now
///   takes an exclusive `flock` on `<path>.lock` and re-checks the size under
///   it, so only the first process renames. Writers that do not rotate take no
///   lock; `O_APPEND` keeps their writes whole, and a write that races a rename
///   lands in the backup (same inode), never lost. If the lock file cannot be
///   opened the rotation is skipped (the file grows past the cap for that call)
///   rather than renamed unlocked.
pub fn append_line_capped(path: &Path, line: &str, max_bytes: u64) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    if max_bytes > 0
        && let Ok(meta) = std::fs::metadata(path)
        && meta.len() >= max_bytes
    {
        rotate_locked(path, max_bytes);
    }
    let mut buf = Vec::with_capacity(line.len() + 1);
    buf.extend_from_slice(line.as_bytes());
    buf.push(b'\n');
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(&buf)?;
    Ok(())
}

/// Rename `path` to its `.1` backup under the cross-process lock, only if it is
/// still at/over the cap once the lock is held. Best-effort: every failure
/// leaves the file in place, so no line is lost.
fn rotate_locked(path: &Path, max_bytes: u64) {
    use std::fs::OpenOptions;
    let lp = lock_path(path);
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lp)
        .or_else(|_| OpenOptions::new().read(true).open(&lp));
    let Ok(lock) = lock else { return };
    if lock.lock().is_err() {
        return;
    }
    if std::fs::metadata(path).is_ok_and(|m| m.len() >= max_bytes) {
        let _ = std::fs::rename(path, rotated_path(path));
    }
    let _ = lock.unlock();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    const CHILD_ENV: &str = "TERMLINK_AUDIT_APPEND_CHILD";

    /// Child half of the cross-process tests: when `TERMLINK_AUDIT_APPEND_CHILD`
    /// is set (`<path>|<cap>|<writer>|<lines>`), append that many lines and print
    /// how often the `.1` backup was seen smaller than the cap. A rotation race
    /// moves a fresh, nearly empty file over the backup, so a backup under the
    /// cap is the observable symptom. A no-op in a normal test run.
    #[test]
    fn child_writer() {
        let Ok(spec) = std::env::var(CHILD_ENV) else { return };
        let parts: Vec<&str> = spec.split('|').collect();
        let path = PathBuf::from(parts[0]);
        let cap: u64 = parts[1].parse().unwrap();
        let (w, n): (usize, usize) = (parts[2].parse().unwrap(), parts[3].parse().unwrap());
        let backup = rotated_path(&path);
        let mut short = 0;
        // ~200 bytes, so a torn line is easy to spot and rotations are frequent.
        let pad = "x".repeat(160);
        for i in 0..n {
            let line = format!(r#"{{"w":{w},"i":{i},"pad":"{pad}"}}"#);
            append_line_capped(&path, &line, cap).unwrap();
            if cap > 0 && std::fs::metadata(&backup).is_ok_and(|m| m.len() < cap) {
                short += 1;
            }
        }
        println!("SHORT_BACKUPS={short}");
    }

    /// Run `procs` child processes of this test binary against one file.
    fn run_children(path: &Path, cap: u64, procs: usize, lines: usize) -> usize {
        let exe = std::env::current_exe().unwrap();
        let kids: Vec<_> = (0..procs)
            .map(|w| {
                Command::new(&exe)
                    .args(["--exact", "audit_append::tests::child_writer", "--nocapture", "--test-threads=1"])
                    .env(CHILD_ENV, format!("{}|{cap}|{w}|{lines}", path.display()))
                    .output_spawn()
            })
            .collect();
        kids.into_iter()
            .map(|k| {
                let out = k.wait_with_output().unwrap();
                assert!(out.status.success(), "child failed: {}", String::from_utf8_lossy(&out.stderr));
                let s = String::from_utf8_lossy(&out.stdout);
                s.lines()
                    .find_map(|l| l.split("SHORT_BACKUPS=").nth(1))
                    .expect("child printed its count")
                    .trim()
                    .parse::<usize>()
                    .unwrap()
            })
            .sum()
    }

    trait SpawnPiped {
        fn output_spawn(&mut self) -> std::process::Child;
    }
    impl SpawnPiped for Command {
        fn output_spawn(&mut self) -> std::process::Child {
            self.stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        }
    }

    fn lines_of(p: &Path) -> Vec<String> {
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// T-3615 item 30: separate PROCESSES, no rotation — every line arrives
    /// exactly once and whole (the old two-write append could glue two records).
    #[test]
    fn concurrent_processes_no_rotation_lose_and_tear_nothing() {
        let td = tempfile::TempDir::new().unwrap();
        let p = td.path().join("audit.jsonl");
        let (procs, lines) = (8, 400);
        run_children(&p, 0, procs, lines);
        let got = lines_of(&p);
        assert_eq!(got.len(), procs * lines, "every line must arrive once");
        for l in &got {
            let v: serde_json::Value = serde_json::from_str(l).expect("no torn or glued line");
            assert!(v.get("w").is_some() && v.get("i").is_some());
        }
        for w in 0..procs {
            let n = got.iter().filter(|l| l.starts_with(&format!(r#"{{"w":{w},"#))).count();
            assert_eq!(n, lines, "writer {w} lost lines");
        }
    }

    /// T-3615 item 30: separate PROCESSES rotating a small cap hundreds of
    /// times. The backup must always be a full generation (never a fresh file
    /// renamed over it), and every kept line must parse.
    #[test]
    fn concurrent_processes_rotating_never_clobber_the_backup() {
        let td = tempfile::TempDir::new().unwrap();
        let p = td.path().join("audit.jsonl");
        let cap = 4096;
        let short = run_children(&p, cap, 8, 600);
        assert_eq!(short, 0, "a backup smaller than the cap means a rotation race clobbered it");
        let backup = rotated_path(&p);
        assert!(std::fs::metadata(&backup).unwrap().len() >= cap);
        for l in lines_of(&p).iter().chain(lines_of(&backup).iter()) {
            serde_json::from_str::<serde_json::Value>(l).expect("every kept line parses");
        }
        assert!(lock_path(&p).exists(), "rotation takes the sidecar lock file");
    }

    #[test]
    fn single_writer_rotates_at_the_cap_and_keeps_one_backup() {
        let td = tempfile::TempDir::new().unwrap();
        let p = td.path().join("a.jsonl");
        for i in 0..10 {
            append_line_capped(&p, &format!("line-{i}-{}", "y".repeat(40)), 100).unwrap();
        }
        let live = lines_of(&p);
        let bak = lines_of(&rotated_path(&p));
        assert!(!live.is_empty() && !bak.is_empty());
        assert!(std::fs::metadata(rotated_path(&p)).unwrap().len() >= 100);
        assert_eq!(live.last().unwrap(), &format!("line-9-{}", "y".repeat(40)));
    }
}
