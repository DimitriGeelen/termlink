//! T-2996 (value-review C-45): per-tool / per-verb invocation telemetry.
//!
//! # Why this exists when `rpc_audit` already counts calls
//!
//! `rpc_audit` records every authenticated JSON-RPC dispatch by METHOD. Every
//! usage question the value review actually asks is about TOOLS and VERBS, and
//! the two do not correspond:
//!
//! - `termlink_agent_top_reacted`, `termlink_agent_top_replied` and
//!   `termlink_agent_top_repliers` all dispatch `channel.subscribe` with the
//!   identical `{topic, cursor, limit}` shape. In `rpc-audit.jsonl` they are
//!   indistinguishable from each other and from any ordinary subscribe.
//! - They PAGE, so one tool call emits many dispatches. The method count is not
//!   merely too coarse to attribute — it is not even proportional to tool usage.
//!
//! Measured on this host's live log: `channel.subscribe` is the single largest
//! method at 403,555 dispatches. That number cannot tell you whether ANY of the
//! 28 off-charter analytics tools was ever called, which is why every review run
//! capped its usage verdicts at UNMEASURED (reading D).
//!
//! # Why append-only, and why that is not a style choice
//!
//! These counts are the evidence base for DELETING tools (C-01/IW-1, C-02). The
//! dangerous failure direction is therefore UNDERCOUNT: a tool that is used but
//! reads as zero gets deleted. So the one property this sink must have is that
//! it does not silently lose records under concurrency.
//!
//! The repo contains both patterns for this exact problem:
//!
//! - `rpc_audit` appends whole lines with `O_APPEND`. POSIX guarantees an
//!   `O_APPEND` write smaller than `PIPE_BUF` is atomic, so concurrent writers
//!   — including writers in DIFFERENT PROCESSES — cannot interleave or clobber
//!   each other. This is the property that makes it safe, not the mutex.
//! - `.agentic-framework/lib/hook-telemetry.sh::_fw_telemetry_increment` does an
//!   unlocked read-modify-TRUNCATE-write. Measured under 8 concurrent writers
//!   (T-2982): 477-479 of 480 increments lost, because `>` truncates on open so
//!   a racing writer reads a near-empty file and writes back ~1. It is currently
//!   corrupting `.context/working/.hook-counter` in this repo.
//!
//! This module follows the first. Note carefully that the process-local `Mutex`
//! below does NOT serialise across processes — MCP and CLI are separate
//! processes from the hub. It guards only the rotation rename. Cross-process
//! safety comes from `O_APPEND` line atomicity, which is why the record shape is
//! an append and never a read-modify-write.
//!
//! # Scope
//!
//! Recording is best-effort: any failure here is swallowed, because losing a
//! telemetry line must never fail a user's tool call. Set
//! `TERMLINK_INVOCATION_AUDIT=0` to disable recording entirely.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Sink file name, written under the resolved runtime dir.
pub const FILE_NAME: &str = "invocation-audit.jsonl";

/// Rotation cap. Deliberately smaller than `rpc_audit`'s 100 MiB: one line per
/// tool call is far lower volume than one per RPC dispatch. `0` disables
/// rotation (append forever), matching `rpc_audit`'s contract.
pub const DEFAULT_MAX_BYTES: u64 = 32 * 1024 * 1024;

/// Surface discriminator — which entry point the invocation came through.
/// Recorded explicitly so a future CLI/session-daemon instrument can share the
/// sink without the reader having to guess from the name.
pub const SURFACE_MCP: &str = "mcp";
pub const SURFACE_CLI: &str = "cli";

static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static MAX_BYTES: OnceLock<u64> = OnceLock::new();
static WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn write_lock() -> &'static Mutex<()> {
    WRITE_LOCK.get_or_init(|| Mutex::new(()))
}

/// Explicit init, mirroring `rpc_audit::init`. Callers that know their runtime
/// dir (the hub) use this; MCP and CLI rely on the lazy resolution in
/// [`current_path`] instead, so they need no startup wiring.
pub fn init(runtime_dir: &Path) {
    let _ = PATH.set(Some(runtime_dir.join(FILE_NAME)));
    let _ = MAX_BYTES.set(read_max_bytes_env());
}

fn read_max_bytes_env() -> u64 {
    std::env::var("TERMLINK_INVOCATION_AUDIT_MAX_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_BYTES)
}

fn max_bytes() -> u64 {
    MAX_BYTES.get().copied().unwrap_or_else(read_max_bytes_env)
}

/// `TERMLINK_INVOCATION_AUDIT=0` disables recording. Any other value (or unset)
/// leaves it on: the instrument exists to be on by default, since a review that
/// has to ask an operator to enable telemetry first gets no telemetry.
fn enabled() -> bool {
    enabled_from(std::env::var("TERMLINK_INVOCATION_AUDIT").ok().as_deref())
}

/// Pure predicate behind [`enabled`], split out so it can be tested without
/// mutating process env in a shared test binary (which would race sibling
/// tests). Only the exact string `"0"` disables.
fn enabled_from(v: Option<&str>) -> bool {
    !matches!(v, Some("0"))
}

/// Resolve the sink path, lazily if `init` was never called.
///
/// The runtime dir is resolved by the binary's OWN four-step order
/// (`termlink_session::discovery::runtime_dir`) rather than a hardcoded
/// `/tmp/termlink-0`. T-2729 is the cautionary case: the preflight check
/// hardcoded that path and consequently inspected a directory the hub was not
/// using, reporting PASS on the one failure mode it existed to catch.
fn current_path() -> Option<PathBuf> {
    if let Some(p) = PATH.get() {
        return p.clone();
    }
    let p = termlink_session::discovery::runtime_dir().join(FILE_NAME);
    let _ = PATH.set(Some(p.clone()));
    Some(p)
}

fn json_escape(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Record one invocation. Best-effort and infallible by contract: every error
/// path is swallowed so a telemetry failure can never fail the caller's tool.
pub fn record(surface: &str, name: &str) {
    if !enabled() {
        return;
    }
    let Some(path) = current_path() else {
        return;
    };
    record_to(&path, surface, name);
}

/// Testable core of [`record`] with an explicit destination.
pub fn record_to(path: &Path, surface: &str, name: &str) {
    let line = format!(
        r#"{{"ts":{},"surface":{},"name":{}}}"#,
        now_ms(),
        json_escape(surface),
        json_escape(name)
    );
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _guard = write_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Errors are intentionally dropped — see the module contract.
    let _ = crate::rpc_audit::append_line_capped(path, &line, max_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn read_lines(p: &Path) -> Vec<String> {
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn records_one_line_per_invocation_with_surface_and_name() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(FILE_NAME);
        record_to(&p, SURFACE_MCP, "termlink_agent_top_reacted");
        record_to(&p, SURFACE_MCP, "termlink_agent_top_repliers");
        let lines = read_lines(&p);
        assert_eq!(lines.len(), 2, "one line per invocation");
        assert!(lines[0].contains(r#""name":"termlink_agent_top_reacted""#));
        assert!(lines[1].contains(r#""name":"termlink_agent_top_repliers""#));
        assert!(lines[0].contains(r#""surface":"mcp""#));
    }

    /// The whole point of the instrument: two tools that are INDISTINGUISHABLE
    /// in `rpc-audit.jsonl` (both dispatch `channel.subscribe`) must be
    /// distinguishable here. If this ever fails, the sink has regressed to the
    /// resolution that left C-01 undecidable.
    #[test]
    fn distinguishes_tools_that_share_one_rpc_method() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(FILE_NAME);
        for _ in 0..3 {
            record_to(&p, SURFACE_MCP, "termlink_agent_top_reacted");
        }
        record_to(&p, SURFACE_MCP, "termlink_agent_top_replied");
        let lines = read_lines(&p);
        let reacted = lines
            .iter()
            .filter(|l| l.contains("top_reacted"))
            .count();
        let replied = lines
            .iter()
            .filter(|l| l.contains("top_replied"))
            .count();
        assert_eq!(reacted, 3);
        assert_eq!(replied, 1);
    }

    /// Records must survive a name containing JSON metacharacters rather than
    /// producing an unparseable line. The episodic-store corruption (T-2805)
    /// was exactly this class: a writer that escaped too little, discovered 29
    /// files later because nothing ever parsed what it wrote.
    #[test]
    fn escapes_names_so_every_line_stays_parseable() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(FILE_NAME);
        record_to(&p, SURFACE_CLI, r#"weird"name\with/meta"#);
        for line in read_lines(&p) {
            let v: serde_json::Value =
                serde_json::from_str(&line).expect("every emitted line must parse as JSON");
            assert_eq!(v["name"], r#"weird"name\with/meta"#);
        }
    }

    /// Best-effort contract: an unwritable destination must not panic.
    #[test]
    fn unwritable_destination_is_swallowed_not_panicked() {
        // A path whose parent cannot be created (a file used as a directory).
        let td = TempDir::new().unwrap();
        let blocker = td.path().join("blocker");
        std::fs::write(&blocker, b"x").unwrap();
        let p = blocker.join("nested").join(FILE_NAME);
        record_to(&p, SURFACE_MCP, "termlink_help"); // must not panic
    }

    /// Concurrency guard. This is the direct T-2982 regression test: the
    /// framework's shell counter loses 477-479 of 480 increments under 8
    /// concurrent writers because it read-modify-TRUNCATE-writes. An
    /// append-only sink must lose nothing.
    #[test]
    fn concurrent_writers_lose_no_records() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(FILE_NAME);
        let writers = 8;
        let per_writer = 60;
        std::thread::scope(|s| {
            for w in 0..writers {
                let p = p.clone();
                s.spawn(move || {
                    for _ in 0..per_writer {
                        record_to(&p, SURFACE_MCP, &format!("tool_{w}"));
                    }
                });
            }
        });
        let lines = read_lines(&p);
        assert_eq!(
            lines.len(),
            writers * per_writer,
            "append-only sink must not lose records under concurrency"
        );
        for line in &lines {
            serde_json::from_str::<serde_json::Value>(line)
                .expect("no line may be torn by a concurrent write");
        }
        for w in 0..writers {
            let n = lines
                .iter()
                .filter(|l| l.contains(&format!(r#""name":"tool_{w}""#)))
                .count();
            assert_eq!(n, per_writer, "writer {w} lost records");
        }
    }

    #[test]
    fn only_the_exact_string_zero_disables_recording() {
        assert!(!enabled_from(Some("0")), "\"0\" must disable");
        assert!(enabled_from(None), "unset must leave the instrument ON");
        assert!(enabled_from(Some("1")));
        assert!(enabled_from(Some("")), "empty is not an opt-out");
        assert!(
            enabled_from(Some("false")),
            "only \"0\" opts out - a near-miss must not silently disable telemetry"
        );
    }
}
