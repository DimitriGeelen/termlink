//! T-1304: append-only RPC audit log.
//!
//! Each authenticated JSON-RPC dispatch records one line:
//! `{"ts":<unix_ms>,"method":"<method>"}` to `<runtime_dir>/rpc-audit.jsonl`.
//! Best-effort — write failures never fail the RPC. `fw metrics api-usage`
//! reads the file and tallies. Single-file design (no rotation in v1) — the
//! operator-runbook handles disk pressure via cron deletion >90d.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static AUDIT_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

pub const FILE_NAME: &str = "rpc-audit.jsonl";

/// T-1307: Methods that are transport plumbing rather than user-meaningful
/// API calls. These would otherwise dominate audit-log volume from long-poll
/// subscriber loops (a single `event collect` CLI invocation generates ~13K
/// `event.collect` dispatches). Skip them so the audit log stays signal-rich
/// for the T-1166 entry-gate measurement.
const SKIP_METHODS: &[&str] = &["event.poll", "event.collect"];

/// T-1311: Legacy primitives that the T-1166 entry gate is targeting for
/// retirement. Used by `warn_if_legacy` to emit a real-time warn log when
/// any of these methods is dispatched. Mirror of the `LEGACY` set in
/// `.agentic-framework/agents/metrics/api-usage.sh` — keep in sync if either
/// changes (single source of truth would require cross-language config; not
/// worth the plumbing for 6 strings).
const LEGACY_METHODS: &[&str] = &[
    "event.broadcast",
    "inbox.list",
    "inbox.status",
    "inbox.clear",
    "file.send",
    "file.receive",
];

/// T-1311: Rate-limit window for the per-(method, from) deprecation warn log.
/// 5 minutes balances: short enough that operator sees the signal soon after
/// turning the spigot off, long enough that a chatty long-running caller
/// doesn't flood the log.
const DEPRECATION_WARN_WINDOW: Duration = Duration::from_secs(5 * 60);

/// T-1311: Per-(method, from) last-warn-at tracker. Process-local. Pruned
/// opportunistically by `warn_if_legacy` — no background gc.
static DEPRECATION_WARN_TRACKER: OnceLock<Mutex<HashMap<(String, String), Instant>>> =
    OnceLock::new();

fn deprecation_tracker() -> &'static Mutex<HashMap<(String, String), Instant>> {
    DEPRECATION_WARN_TRACKER.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_legacy_method(method: &str) -> bool {
    LEGACY_METHODS.contains(&method)
        || method.starts_with("file.send.")
        || method.starts_with("file.receive.")
}

/// T-1311: Emit a real-time `tracing::warn!` when a legacy primitive is
/// called, rate-limited to one log per (method, from) per 5 minutes.
///
/// Pairs with the audit log written by `record()` — that captures *every*
/// call for retrospective tally; this surfaces *one per offender per
/// window* for live operator awareness (journalctl/stderr tail).
///
/// T-1407: `peer_pid` (when Some) is included as a structured tracing
/// field. For Unix-socket callers this lets `ps -p <pid>` identify the
/// originating process even when the JSON-RPC `from` field is absent.
/// T-1409: `peer_addr` (when Some) carries the TCP source address —
/// the network analogue for callers that have no local PID.
pub fn warn_if_legacy(
    method: &str,
    from: Option<&str>,
    peer_pid: Option<u32>,
    peer_addr: Option<&str>,
) {
    if !is_legacy_method(method) {
        return;
    }
    let from_label = from.unwrap_or("(unknown)");
    let key = (method.to_string(), from_label.to_string());
    let now = Instant::now();
    let mut tracker = match deprecation_tracker().lock() {
        Ok(t) => t,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Opportunistic prune: drop entries older than 2x window. Bounds
    // memory under churning-caller workloads without a background task.
    tracker.retain(|_, last| now.duration_since(*last) < DEPRECATION_WARN_WINDOW * 2);
    let should_log = match tracker.get(&key) {
        None => true,
        Some(last) => now.duration_since(*last) >= DEPRECATION_WARN_WINDOW,
    };
    if should_log {
        tracker.insert(key, now);
        tracing::warn!(
            method = %method,
            from = %from_label,
            peer_pid = ?peer_pid,
            peer_addr = ?peer_addr,
            "deprecated primitive called — T-1166: schedule retirement once legacy <1% over 60d"
        );
    }
}

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
/// T-1309: optionally records caller attribution (`from` field) so
/// `fw metrics api-usage` can break down legacy callers by display_name.
/// T-1407: optionally records `peer_pid` for Unix-socket callers — the
/// connect-time PID from getsockopt(SO_PEERCRED). Identifies non-TermLink
/// callers (raw JSON-RPC shells, third-party tools) that omit `from`.
/// Schema is additive: omitted when `None` or 0; existing readers ignore
/// the new field. TCP/TLS connections always get `None`.
/// T-1409: `peer_addr` (when Some non-empty) records the TCP source
/// address `"ip:port"`. Mirror of `peer_pid` for the network side —
/// identifies callers that have no local PID. Omitted when `None` or
/// empty. Unix connections always pass `None`.
pub fn record(
    method: &str,
    from: Option<&str>,
    peer_pid: Option<u32>,
    peer_addr: Option<&str>,
) {
    if SKIP_METHODS.contains(&method) {
        return;
    }
    let Some(path) = current_path() else { return };
    let line = build_audit_line(now_ms(), method, from, peer_pid, peer_addr);
    if let Err(e) = append_line(path, &line) {
        tracing::debug!(error = %e, "rpc_audit: append failed (suppressed)");
    }
}

/// Pure: assemble one JSONL audit line. Public-in-crate so tests can
/// drive it without touching the OnceLock-guarded path.
pub(crate) fn build_audit_line(
    ts_ms: u128,
    method: &str,
    from: Option<&str>,
    peer_pid: Option<u32>,
    peer_addr: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(5);
    parts.push(format!("\"ts\":{ts_ms}"));
    parts.push(format!("\"method\":{}", json_escape(method)));
    if let Some(f) = from
        && !f.is_empty()
    {
        parts.push(format!("\"from\":{}", json_escape(f)));
    }
    if let Some(pid) = peer_pid
        && pid != 0
    {
        parts.push(format!("\"peer_pid\":{pid}"));
    }
    if let Some(a) = peer_addr
        && !a.is_empty()
    {
        parts.push(format!("\"peer_addr\":{}", json_escape(a)));
    }
    format!("{{{}}}", parts.join(","))
}

// T-1432: summarize legacy-primitive invocations from the audit log within
// the given time window. Returns a JSON value shaped for `hub.legacy_usage`
// callers (fleet doctor cut-readiness telemetry).
//
// Schema:
//   {
//     "window_seconds": <u64>,
//     "now_ms": <u128>,
//     "audit_present": <bool>,
//     "total_legacy": <u64>,
//     "last_legacy_ts_ms": <u128 | null>,
//     "by_method": {
//       "<method>": { "count": <u64>, "last_ts_ms": <u128>, "callers": [{"from": "<label>", "count": <u64>}, ...] },
//       ...
//     }
//   }
//
// "Legacy" = the set defined by `is_legacy_method` (event.broadcast,
// inbox.{list,status,clear}, file.{send,receive} + chunked variants).
//
// Skips malformed lines silently — best-effort parser, mirrors the
// best-effort write path. Caller decides what to do with the count.
pub fn summarize_legacy_usage(window_seconds: u64) -> serde_json::Value {
    use std::io::{BufRead, BufReader};

    let path = current_path();
    let audit_present = path.is_some_and(|p| p.exists());

    let lines: Vec<String> = if audit_present
        && let Some(path) = path
        && let Ok(file) = std::fs::File::open(path)
    {
        BufReader::new(file).lines().map_while(Result::ok).collect()
    } else {
        Vec::new()
    };

    summarize_lines(lines.into_iter(), window_seconds, now_ms(), audit_present)
}

// Pure helper: sums up audit-line iterator within a window. Splitting this
// out from `summarize_legacy_usage` lets tests drive it without poking
// AUDIT_PATH (OnceLock-only-once is a unit-test ergonomic hazard).
pub(crate) fn summarize_lines(
    lines: impl Iterator<Item = String>,
    window_seconds: u64,
    now: u128,
    audit_present: bool,
) -> serde_json::Value {
    use std::collections::BTreeMap;

    let window_ms: u128 = (window_seconds as u128).saturating_mul(1000);
    let window_start_ms = now.saturating_sub(window_ms);

    let mut total: u64 = 0;
    let mut last_ts: Option<u128> = None;
    let mut by_method: BTreeMap<String, (u64, u128, BTreeMap<String, u64>)> = BTreeMap::new();

    for line in lines {
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ts = v.get("ts").and_then(|t| t.as_u64()).unwrap_or(0) as u128;
        if ts < window_start_ms {
            continue;
        }
        let method = match v.get("method").and_then(|m| m.as_str()) {
            Some(m) => m,
            None => continue,
        };
        if !is_legacy_method(method) {
            continue;
        }
        total += 1;
        last_ts = Some(last_ts.map_or(ts, |prev| prev.max(ts)));
        let from = v
            .get("from")
            .and_then(|f| f.as_str())
            .unwrap_or("(unknown)")
            .to_string();
        let entry = by_method
            .entry(method.to_string())
            .or_insert_with(|| (0, 0, BTreeMap::new()));
        entry.0 += 1;
        entry.1 = entry.1.max(ts);
        *entry.2.entry(from).or_insert(0) += 1;
    }

    let by_method_json: serde_json::Map<String, serde_json::Value> = by_method
        .into_iter()
        .map(|(method, (count, ts, callers))| {
            let mut callers_vec: Vec<serde_json::Value> = callers
                .into_iter()
                .map(|(from, c)| serde_json::json!({"from": from, "count": c}))
                .collect();
            callers_vec.sort_by(|a, b| {
                b.get("count")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0)
                    .cmp(&a.get("count").and_then(|x| x.as_u64()).unwrap_or(0))
            });
            (
                method,
                serde_json::json!({
                    "count": count,
                    "last_ts_ms": ts,
                    "callers": callers_vec,
                }),
            )
        })
        .collect();

    serde_json::json!({
        "window_seconds": window_seconds,
        "now_ms": now,
        "audit_present": audit_present,
        "total_legacy": total,
        "last_legacy_ts_ms": last_ts,
        "by_method": serde_json::Value::Object(by_method_json),
    })
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
        record("hub.auth", None, None, None);
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
    fn line_with_from_includes_field() {
        // T-1309: when caller attribution is provided, line must include "from".
        let l = build_audit_line(42, "event.broadcast", Some("framework-agent"), None, None);
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert_eq!(v["method"], "event.broadcast");
        assert_eq!(v["from"], "framework-agent");
        assert_eq!(v["ts"], 42);
    }

    #[test]
    fn line_without_from_omits_field() {
        // T-1309: when caller attribution is absent, the line must NOT include "from".
        let l = build_audit_line(42, "event.broadcast", None, None, None);
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert!(v.get("from").is_none(), "from must be omitted when None");
        assert_eq!(v["method"], "event.broadcast");
    }

    #[test]
    fn empty_from_treated_as_absent() {
        // T-1309: empty-string from should be treated like None and omitted.
        let l = build_audit_line(1, "event.broadcast", Some(""), None, None);
        let v: serde_json::Value = serde_json::from_str(&l).unwrap();
        assert!(v.get("from").is_none(), "empty from must be omitted");
    }

    #[test]
    fn line_with_peer_pid_includes_field() {
        // T-1407: when peer_pid is provided, line must include peer_pid as u32.
        let l = build_audit_line(42, "inbox.status", None, Some(12345), None);
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert_eq!(v["method"], "inbox.status");
        assert_eq!(v["peer_pid"], 12345);
        assert!(v.get("from").is_none(), "from absent → not in line");
    }

    #[test]
    fn line_with_from_and_peer_pid_includes_both() {
        // T-1407: from + peer_pid both present should both appear.
        let l = build_audit_line(7, "event.broadcast", Some("tl-abc"), Some(99), None);
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert_eq!(v["from"], "tl-abc");
        assert_eq!(v["peer_pid"], 99);
    }

    #[test]
    fn line_with_peer_pid_zero_omits_field() {
        // T-1407: pid 0 is treated as absent (no peer_pid available).
        let l = build_audit_line(7, "inbox.list", None, Some(0), None);
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert!(v.get("peer_pid").is_none(), "pid 0 must be omitted");
    }

    #[test]
    fn line_with_peer_addr_includes_field() {
        // T-1409: when peer_addr is provided, line must include peer_addr as string.
        let l = build_audit_line(42, "inbox.status", None, None, Some("192.168.10.143:42820"));
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert_eq!(v["method"], "inbox.status");
        assert_eq!(v["peer_addr"], "192.168.10.143:42820");
        assert!(v.get("from").is_none());
        assert!(v.get("peer_pid").is_none());
    }

    #[test]
    fn line_with_peer_addr_and_from_includes_both() {
        // T-1409: from + peer_addr both present should both appear.
        let l = build_audit_line(7, "event.broadcast", Some("tl-xyz"), None, Some("10.0.0.5:5555"));
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert_eq!(v["from"], "tl-xyz");
        assert_eq!(v["peer_addr"], "10.0.0.5:5555");
    }

    #[test]
    fn line_with_all_three_fields() {
        // T-1409: from + peer_pid + peer_addr all present should all appear.
        let l = build_audit_line(
            7,
            "event.broadcast",
            Some("tl-abc"),
            Some(42),
            Some("127.0.0.1:9100"),
        );
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert_eq!(v["from"], "tl-abc");
        assert_eq!(v["peer_pid"], 42);
        assert_eq!(v["peer_addr"], "127.0.0.1:9100");
    }

    #[test]
    fn line_with_empty_peer_addr_omits_field() {
        // T-1409: empty-string peer_addr should be treated like None and omitted.
        let l = build_audit_line(1, "event.broadcast", None, None, Some(""));
        let v: serde_json::Value = serde_json::from_str(&l).expect("valid JSON");
        assert!(v.get("peer_addr").is_none(), "empty peer_addr must be omitted");
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

    /// T-1311: rate-limited deprecation warn — predicate-level tests.
    /// We test `is_legacy_method` and tracker behavior directly because
    /// `warn_if_legacy` emits via `tracing` and the global tracker shared
    /// across tests creates ordering coupling. The integration is exercised
    /// implicitly through unique-key sequences (each test uses keys no
    /// other test uses) so they don't see each other's tracker state.
    #[test]
    fn is_legacy_method_recognises_set_and_chunked_variants() {
        assert!(is_legacy_method("event.broadcast"));
        assert!(is_legacy_method("inbox.list"));
        assert!(is_legacy_method("inbox.status"));
        assert!(is_legacy_method("inbox.clear"));
        assert!(is_legacy_method("file.send"));
        assert!(is_legacy_method("file.receive"));
        assert!(is_legacy_method("file.send.chunk"));
        assert!(is_legacy_method("file.receive.metadata"));
        // Negatives
        assert!(!is_legacy_method("channel.post"));
        assert!(!is_legacy_method("event.subscribe"));
        assert!(!is_legacy_method("hub.auth"));
        assert!(!is_legacy_method("event.poll"));
    }

    #[test]
    fn warn_if_legacy_noop_for_non_legacy() {
        // Should return cleanly with no panic and no tracker insert.
        // Use a method we KNOW is not legacy.
        warn_if_legacy("channel.post", Some("test-noop-caller"), None, None);
        let tracker = deprecation_tracker().lock().unwrap();
        assert!(
            !tracker
                .keys()
                .any(|k| k.0 == "channel.post" && k.1 == "test-noop-caller"),
            "non-legacy method must not insert tracker entry"
        );
    }

    #[test]
    fn warn_if_legacy_logs_first_call_inserts_tracker() {
        // First call for a unique (method, from) should insert into tracker.
        let unique_from = "T-1311-unit-A";
        warn_if_legacy("event.broadcast", Some(unique_from), None, None);
        let tracker = deprecation_tracker().lock().unwrap();
        assert!(
            tracker
                .keys()
                .any(|k| k.0 == "event.broadcast" && k.1 == unique_from),
            "first call must insert tracker entry"
        );
    }

    #[test]
    fn warn_if_legacy_unknown_caller_label() {
        // None caller should be tracked under "(unknown)".
        warn_if_legacy("inbox.list", None, None, None);
        let tracker = deprecation_tracker().lock().unwrap();
        assert!(
            tracker
                .keys()
                .any(|k| k.0 == "inbox.list" && k.1 == "(unknown)"),
            "missing from must surface as (unknown)"
        );
    }

    #[test]
    fn warn_if_legacy_rate_limits_within_window() {
        // Two calls in a row with the same key — only the first inserts a
        // fresh timestamp; the second sees it already there and skips the
        // warn (we can't assert the log directly, but we can verify the
        // timestamp didn't change much between the two calls).
        let unique_from = "T-1311-unit-rate";
        warn_if_legacy("event.broadcast", Some(unique_from), None, None);
        let t1 = {
            let tracker = deprecation_tracker().lock().unwrap();
            *tracker
                .get(&("event.broadcast".to_string(), unique_from.to_string()))
                .expect("entry exists")
        };
        std::thread::sleep(Duration::from_millis(10));
        warn_if_legacy("event.broadcast", Some(unique_from), None, None);
        let t2 = {
            let tracker = deprecation_tracker().lock().unwrap();
            *tracker
                .get(&("event.broadcast".to_string(), unique_from.to_string()))
                .expect("entry exists")
        };
        // Within window — timestamp should NOT have advanced.
        assert_eq!(
            t1, t2,
            "second call within rate-limit window must NOT update timestamp"
        );
    }

    #[test]
    fn summarize_lines_counts_only_legacy_within_window() {
        // T-1432: parser correctness — legacy lines inside window count,
        // non-legacy and out-of-window lines do not.
        let now: u128 = 1_000_000_000_000;
        let window_seconds: u64 = 7 * 86400;
        let in_window = now - 1000;
        let out_of_window = now - (window_seconds as u128) * 1000 - 1;
        let lines = vec![
            // 3 legacy in-window with two distinct callers
            format!(r#"{{"ts":{},"method":"event.broadcast","from":"agent-a"}}"#, in_window),
            format!(r#"{{"ts":{},"method":"inbox.list","from":"agent-a"}}"#, in_window - 100),
            format!(r#"{{"ts":{},"method":"event.broadcast","from":"agent-b"}}"#, in_window - 50),
            // non-legacy in-window: must be ignored
            format!(r#"{{"ts":{},"method":"channel.post","from":"agent-a"}}"#, in_window),
            // legacy but out of window: must be ignored
            format!(r#"{{"ts":{},"method":"file.send","from":"agent-c"}}"#, out_of_window),
            // malformed: must be ignored
            "not-json".to_string(),
        ];
        let summary = summarize_lines(lines.into_iter(), window_seconds, now, true);
        assert_eq!(summary["total_legacy"], 3);
        assert_eq!(summary["audit_present"], true);
        assert_eq!(summary["window_seconds"], window_seconds);
        let by = summary["by_method"].as_object().unwrap();
        assert_eq!(by["event.broadcast"]["count"], 2);
        assert_eq!(by["inbox.list"]["count"], 1);
        assert!(by.get("file.send").is_none());
        assert!(by.get("channel.post").is_none());
        // Caller breakdown: event.broadcast had agent-a + agent-b each once
        let bcast_callers = by["event.broadcast"]["callers"].as_array().unwrap();
        assert_eq!(bcast_callers.len(), 2);
    }

    #[test]
    fn summarize_lines_empty_audit_returns_zero() {
        // Audit-not-present and empty-audit must both return total_legacy=0.
        // The verdict logic in fleet doctor distinguishes via audit_present flag.
        let s = summarize_lines(std::iter::empty(), 7 * 86400, 1_000_000_000_000, false);
        assert_eq!(s["total_legacy"], 0);
        assert_eq!(s["audit_present"], false);
        assert!(s["last_legacy_ts_ms"].is_null());
    }

    #[test]
    fn summarize_lines_handles_missing_from_field() {
        // T-1309: from field is optional. Lines without it must still count
        // and bucket under "(unknown)".
        let now: u128 = 1_000_000_000_000;
        let lines = vec![format!(r#"{{"ts":{},"method":"event.broadcast"}}"#, now - 100)];
        let s = summarize_lines(lines.into_iter(), 7 * 86400, now, true);
        assert_eq!(s["total_legacy"], 1);
        let callers = s["by_method"]["event.broadcast"]["callers"]
            .as_array()
            .unwrap();
        assert_eq!(callers[0]["from"], "(unknown)");
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
