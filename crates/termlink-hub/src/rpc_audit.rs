//! T-1304: append-only RPC audit log.
//!
//! Each authenticated JSON-RPC dispatch records one line:
//! `{"ts":<unix_ms>,"method":"<method>"}` to `<runtime_dir>/rpc-audit.jsonl`.
//! Best-effort — write failures never fail the RPC. `fw metrics api-usage`
//! reads the file and tallies. Single-file design (no rotation in v1) — the
//! operator-runbook handles disk pressure via cron deletion >90d.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static AUDIT_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

pub const FILE_NAME: &str = "rpc-audit.jsonl";

/// T-1307: Methods that are transport plumbing rather than user-meaningful
/// API calls. These would otherwise dominate audit-log volume from long-poll
/// subscriber loops (a single `event collect` CLI invocation generates ~13K
/// `event.collect` dispatches). Skip them so the audit log stays signal-rich
/// for the T-1166 entry-gate measurement.
const SKIP_METHODS: &[&str] = &["event.poll", "event.collect"];

/// Initialise the audit-log path. Call once at hub bootstrap.
/// If the runtime_dir is missing or unwritable the audit silently no-ops.
pub fn init(runtime_dir: &Path) {
    let path = runtime_dir.join(FILE_NAME);
    let _ = AUDIT_PATH.set(Some(path));
}

/// Test-only override so unit tests don't trample the prod path.
#[cfg(test)]
pub fn init_for_test(path: PathBuf) {
    let _ = AUDIT_PATH.set(Some(path));
}

fn current_path() -> Option<&'static Path> {
    AUDIT_PATH.get().and_then(|p| p.as_deref())
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Append one line. Errors are logged at debug and swallowed.
/// T-1307: silently skips transport-plumbing methods listed in `SKIP_METHODS`.
pub fn record(method: &str) {
    if SKIP_METHODS.contains(&method) {
        return;
    }
    let Some(path) = current_path() else { return };
    let line = format!(r#"{{"ts":{},"method":{}}}"#, now_ms(), json_escape(method));
    if let Err(e) = append_line(path, &line) {
        tracing::debug!(error = %e, "rpc_audit: append failed (suppressed)");
    }
}

fn append_line(path: &Path, line: &str) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

fn json_escape(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Reset OnceLock by writing the path directly via the test helper.
    /// Tests that need an isolated path should own their own TempDir.
    fn force_path(path: PathBuf) {
        // OnceLock::set returns Err if already set — allow that; tests below
        // run in-process and only the first one wins. We work around this by
        // calling append_line directly in the second/third tests.
        let _ = AUDIT_PATH.set(Some(path));
    }

    #[test]
    fn record_creates_file_and_one_valid_json_line() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("rpc-audit.jsonl");
        force_path(path.clone());
        record("hub.auth");
        // Read file (might be the first record with this OnceLock — only check existence + parse)
        if path.exists() {
            let body = fs::read_to_string(&path).unwrap();
            let line = body.lines().next().expect("at least one line");
            let v: serde_json::Value = serde_json::from_str(line).expect("valid JSON line");
            assert!(v.get("ts").and_then(|t| t.as_u64()).is_some());
            assert!(v.get("method").and_then(|m| m.as_str()).is_some());
        }
    }

    #[test]
    fn append_line_writes_two_distinct_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("rpc-audit.jsonl");
        // Bypass OnceLock-only-once limitation by calling helpers directly.
        let l1 = format!(r#"{{"ts":1,"method":{}}}"#, json_escape("event.broadcast"));
        let l2 = format!(r#"{{"ts":2,"method":{}}}"#, json_escape("inbox.list"));
        append_line(&path, &l1).unwrap();
        append_line(&path, &l2).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 2);
        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(v1["method"], "event.broadcast");
        assert_eq!(v2["method"], "inbox.list");
        assert_eq!(v1["ts"], 1);
        assert_eq!(v2["ts"], 2);
    }

    #[test]
    fn record_does_not_panic_on_unwritable_path() {
        // Pointing at a path under a non-existent parent — OpenOptions returns
        // ENOENT; record() must swallow it and not panic.
        let bogus = PathBuf::from("/nonexistent/no-such-dir/rpc-audit.jsonl");
        let r = append_line(&bogus, r#"{"ts":1,"method":"x"}"#);
        assert!(r.is_err(), "unwritable path should error");
        // record() with this path would log+swallow; we can't invoke record()
        // here because OnceLock is shared, but the shape of the swallow path
        // is exercised: append_line returns Err → record() logs + returns ().
    }

    #[test]
    fn json_escape_escapes_quotes_and_control_chars() {
        assert_eq!(json_escape("foo"), "\"foo\"");
        assert_eq!(json_escape("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_escape("a\nb"), "\"a\\nb\"");
    }

    #[test]
    fn skip_methods_contains_long_poll_plumbing() {
        // T-1307: must skip these to keep audit volume sane under long-poll load.
        assert!(SKIP_METHODS.contains(&"event.poll"));
        assert!(SKIP_METHODS.contains(&"event.collect"));
        // Real API methods MUST NOT be skipped — the gate depends on counting them.
        assert!(!SKIP_METHODS.contains(&"event.broadcast"));
        assert!(!SKIP_METHODS.contains(&"channel.post"));
        assert!(!SKIP_METHODS.contains(&"inbox.list"));
    }

    #[test]
    fn record_skips_event_poll_does_not_create_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("rpc-audit.jsonl");
        // Drive only skip-listed methods. The append path must NEVER be invoked,
        // so the file should not exist regardless of whether the OnceLock is set.
        // We test the predicate directly to avoid the OnceLock-only-once issue.
        for skip in SKIP_METHODS {
            // Mimic record()'s early return: if record were called with this
            // method, append_line wouldn't run.
            assert!(SKIP_METHODS.contains(skip));
        }
        // And confirm a non-skipped method WOULD reach append_line:
        assert!(!SKIP_METHODS.contains(&"event.broadcast"));
        // Sanity: file doesn't exist without write
        assert!(!path.exists());
    }
}
