//! Per-tool and per-verb invocation telemetry written to an audit sink.
//! T-2996 (value-review C-45): per-tool / per-verb invocation telemetry.
//!
//! Lives in `termlink-session` since T-3033 so the session daemon can record
//! through it; `termlink_hub::invocation_audit` re-exports it, so hub, MCP and
//! CLI call sites are unchanged.
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
//!
//! # Caller attribution (T-3615)
//!
//! Every record also says WHO called, so the fleet's usage can be split by caller
//! (the T-3612 Q6 question: what do interactive sessions, dispatched workers,
//! reviewers and standing workers each actually use?). Attribution comes from the
//! process environment ONLY, read once per process and cached; it never reads
//! message content, arguments or files. Fields are only ADDED after `name`, so a
//! reader of the old `{ts,surface,name}` shape keeps working, and an old row (no
//! `class`) is distinguishable from an attributed one.
//!
//! - `class` — one of `interactive`, `dispatched`, `standing`, `reviewer`,
//!   `background`, `unknown`, decided by the FIRST rule that matches:
//!   1. `TERMLINK_WORKER_CLASS` set to one of those six values: an explicit
//!      declaration. This is how a launcher the rules below cannot see declares
//!      itself (a standing `claude -n` session under a unit, a non-Anthropic seat).
//!   2. `FW_DISPATCHED_WORKER=1` — written into every `fw termlink dispatch`
//!      worker's env.sh (termlink.sh, T-3959): `reviewer` when the dispatch is a
//!      review (`FW_REVIEW_WORKER=1`, written for task_type review, or
//!      `FW_REVIEWER_IN_DISPATCH=1`, the reviewer's own dispatch sentinel), else
//!      `dispatched`. Checked before rule 3 because a worker inherits its
//!      parent's other markers.
//!   3. `TERMLINK_SIDECAR_SUPERVISOR` set (the arc-011 sidecar unit
//!      `termlink-sidecar@<project>--<agent>` sets it to `systemd`; this is how the
//!      W1 pair claude-termlink / claude-termlink-alt run), or `INVOCATION_ID` set
//!      WITHOUT `FW_CLAUDE_FW_SUPERVISED` AND with an agent marker (`AI_AGENT` or
//!      `CLAUDE_CODE_ENTRYPOINT`: an agent systemd started as a service) -> `standing`.
//!      `claude-fw` also runs under systemd (a `run-*.scope`, so `INVOCATION_ID` is
//!      set) but marks itself `FW_CLAUDE_FW_SUPERVISED=1`: that is an operator's
//!      session, not a standing worker.
//!   4. `CLAUDE_CODE_ENTRYPOINT`: `cli` (the interactive TUI) -> `interactive`;
//!      `sdk-*` (headless `claude -p` fired by an orchestrator outside
//!      `fw termlink dispatch`) -> `dispatched`.
//!   5. No agent marker at all (neither `AI_AGENT` nor `CLAUDE_CODE_ENTRYPOINT`)
//!      AND either `INVOCATION_ID` set (a systemd service with no agent in it) or
//!      stdin not a terminal (cron, a `nohup`/`setsid` loop, a daemon: their stdin
//!      is `/dev/null` or a file) -> `background`. This is the heuristic for the
//!      callers we do not own (the vendored sidecar `watcher.py`, other projects'
//!      loops); the ones we own declare `TERMLINK_WORKER_CLASS=background` via
//!      rule 1 (every `.context/cron/*.crontab`, the notify/waker loops).
//!      Measured before this rule (2026-10-28, .107): 88% of attributed rows were
//!      `unknown`, nearly all `channel unread` / `pty output` / `ping` polling
//!      from such loops. An agent's Bash tool also has a non-tty stdin, but it
//!      carries `AI_AGENT`, so it never lands here.
//!   6. otherwise `unknown` (a human typing `termlink` at a terminal, or a
//!      harness that marks itself only partly).
//!
//!   `TERMLINK_AGENT_ID` is deliberately NOT a class signal: this project sets
//!   `TERMLINK_AGENT_ID=claude-termlink` in its Claude settings env (runme action
//!   8, T-3303), so every session and every worker it spawns carries it.
//! - `harness` — `AI_AGENT` verbatim (e.g. `claude-code_2-1-311_agent` from an
//!   agent's Bash tool, `..._harness` from the MCP server/hooks the harness spawns).
//! - `model` — `TERMLINK_MODEL`, else `ANTHROPIC_MODEL`; omitted when neither is set.
//! - `worker_kind` — `TERMLINK_WORKER_KIND`; omitted when unset.
//! - `worker` — `FW_SIDECAR_AGENT_ID` (a dispatched worker's dispatch name, which
//!   is also its `.context/dispatch/<name>/` dir), so the rollup can join usage to
//!   the dispatch's meta.json for task, model and kind.
//! - `task` — `TERMLINK_TASK`, else `FW_TASK_ID`; omitted when unset.
//! - `project` — basename of `TERMLINK_PROJECT`, else `PROJECT_ROOT`, else
//!   `CLAUDE_PROJECT_DIR`, else `PWD`.
//!
//! # One sink per host (T-3615)
//!
//! Dispatched workers and other callers often run without `TERMLINK_RUNTIME_DIR`,
//! so the four-step [`crate::discovery::runtime_dir`] sent their records to
//! `$TMPDIR/termlink-<uid>` or `/tmp/termlink-<uid>`: a stray, volatile sink nobody
//! read (26k rows found on .107). The rule now: an explicit `TERMLINK_RUNTIME_DIR`
//! (or [`init`]) still wins; otherwise the sink is the runtime dir of the hub this
//! host runs, i.e. the first of [`crate::discovery::all_runtime_dirs`] (the same
//! candidate list hub discovery and the supervisor scan, primary first, then
//! `/var/lib/termlink`) that holds a `hub.sock`. The hub's runtime dir is the one
//! directory every caller on the host agrees on whatever its env, and it is the
//! persistent one where the hub runs under systemd. If no candidate holds a hub,
//! the behaviour is today's (the primary runtime dir). This applies to the audit
//! sink only; session and hub discovery are unchanged. Cost: at most one `stat`
//! per candidate, once per process.

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
/// T-3033 (C-30/C-31): session-daemon RPCs (`kv.*`, `session.*`) that never pass
/// the hub's `rpc_audit`. Recorded at `handler::dispatch_scoped`.
pub const SURFACE_SESSION_RPC: &str = "session-rpc";

/// T-3033: which session-daemon methods are recorded. Only the two surfaces the
/// value review found unobservable. `event.poll`/`event.subscribe` long-poll
/// loops are deliberately excluded: they are high-rate and answer no open usage
/// question, so they would only crowd the rotation cap.
pub fn is_recorded_session_method(method: &str) -> bool {
    method.starts_with("kv.") || method.starts_with("session.")
}

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
    let explicit = std::env::var_os("TERMLINK_RUNTIME_DIR").map(PathBuf::from);
    let dir = resolve_sink_dir(
        explicit,
        &crate::discovery::all_runtime_dirs(),
        |d| d.join("hub.sock").exists(),
    );
    let p = dir.join(FILE_NAME);
    let _ = PATH.set(Some(p.clone()));
    Some(p)
}

/// T-3615: pure core of the one-sink-per-host rule (see the module doc).
/// `candidates` is primary-first (as [`crate::discovery::all_runtime_dirs`]
/// returns it); `has_hub` says whether a dir holds this host's hub socket.
/// Explicit override wins; else the first candidate with a hub; else the
/// primary candidate (today's behaviour).
pub fn resolve_sink_dir(
    explicit: Option<PathBuf>,
    candidates: &[PathBuf],
    has_hub: impl Fn(&Path) -> bool,
) -> PathBuf {
    if let Some(dir) = explicit {
        return dir;
    }
    if let Some(dir) = candidates.iter().find(|d| has_hub(d)) {
        return dir.clone();
    }
    candidates
        .first()
        .cloned()
        .unwrap_or_else(crate::discovery::runtime_dir)
}

/// Worker classes a record can carry (T-3615). See the module doc for the rules.
pub const CLASSES: [&str; 6] =
    ["interactive", "dispatched", "standing", "reviewer", "background", "unknown"];

/// Caller attribution for one process (T-3615). Built once from the env.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    pub class: &'static str,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub worker_kind: Option<String>,
    pub worker: Option<String>,
    pub task: Option<String>,
    pub project: Option<String>,
}

fn nonempty(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Pure classifier for a caller whose stdin is a terminal (so only env decides).
pub fn attribution_from(get: impl Fn(&str) -> Option<String>) -> Attribution {
    attribution_with(get, true)
}

/// Pure classifier behind the cached attribution: `get` looks one env var up,
/// `stdin_tty` says whether stdin is a terminal (rule 5, `background`).
/// Split out so every class can be tested without mutating process env (L-006).
pub fn attribution_with(get: impl Fn(&str) -> Option<String>, stdin_tty: bool) -> Attribution {
    let get = |k: &str| nonempty(get(k));
    let is1 = |k: &str| get(k).as_deref() == Some("1");
    let agent_marked = get("AI_AGENT").is_some() || get("CLAUDE_CODE_ENTRYPOINT").is_some();
    let systemd_service = get("INVOCATION_ID").is_some();
    let class = if let Some(c) = get("TERMLINK_WORKER_CLASS")
        .and_then(|v| CLASSES.iter().copied().find(|c| *c == v.to_ascii_lowercase()))
    {
        c
    } else if is1("FW_DISPATCHED_WORKER") {
        if is1("FW_REVIEW_WORKER") || is1("FW_REVIEWER_IN_DISPATCH") {
            "reviewer"
        } else {
            "dispatched"
        }
    } else if get("TERMLINK_SIDECAR_SUPERVISOR").is_some()
        || (systemd_service && agent_marked && get("FW_CLAUDE_FW_SUPERVISED").is_none())
    {
        "standing"
    } else {
        match get("CLAUDE_CODE_ENTRYPOINT").as_deref() {
            Some("cli") => "interactive",
            Some(e) if e.starts_with("sdk") => "dispatched",
            _ if !agent_marked && (systemd_service || !stdin_tty) => "background",
            _ => "unknown",
        }
    };
    let project = get("TERMLINK_PROJECT")
        .or_else(|| get("PROJECT_ROOT"))
        .or_else(|| get("CLAUDE_PROJECT_DIR"))
        .or_else(|| get("PWD"))
        .map(|p| {
            let t = p.trim_end_matches('/');
            Path::new(t)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| t.to_string())
        })
        .filter(|s| !s.is_empty());
    Attribution {
        class,
        harness: get("AI_AGENT"),
        model: get("TERMLINK_MODEL").or_else(|| get("ANTHROPIC_MODEL")),
        worker_kind: get("TERMLINK_WORKER_KIND"),
        worker: get("FW_SIDECAR_AGENT_ID"),
        task: get("TERMLINK_TASK").or_else(|| get("FW_TASK_ID")),
        project,
    }
}

impl Attribution {
    /// The JSON fields this attribution adds, each prefixed with `,` so the
    /// fragment appends after `"name":...` inside the record object.
    pub fn json_fragment(&self) -> String {
        let mut out = format!(r#","class":{}"#, json_escape(self.class));
        for (k, v) in [
            ("harness", &self.harness),
            ("model", &self.model),
            ("worker_kind", &self.worker_kind),
            ("worker", &self.worker),
            ("task", &self.task),
            ("project", &self.project),
        ] {
            if let Some(v) = v {
                out.push_str(&format!(r#","{k}":{}"#, json_escape(v)));
            }
        }
        out
    }
}

static ATTRIBUTION: OnceLock<String> = OnceLock::new();

/// This process's attribution fragment, computed from the env on first use.
fn attribution_fragment() -> &'static str {
    ATTRIBUTION.get_or_init(|| {
        use std::io::IsTerminal;
        attribution_with(|k| std::env::var(k).ok(), std::io::stdin().is_terminal()).json_fragment()
    })
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

/// Testable core of [`record`] with an explicit destination. Carries this
/// process's attribution (T-3615).
pub fn record_to(path: &Path, surface: &str, name: &str) {
    record_to_with(path, surface, name, attribution_fragment());
}

/// [`record_to`] with an explicit attribution fragment (from
/// [`Attribution::json_fragment`]), so tests need not touch process env.
pub fn record_to_with(path: &Path, surface: &str, name: &str, attribution: &str) {
    let line = format!(
        r#"{{"ts":{},"surface":{},"name":{}{}}}"#,
        now_ms(),
        json_escape(surface),
        json_escape(name),
        attribution
    );
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _guard = write_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Errors are intentionally dropped — see the module contract.
    let _ = crate::audit_append::append_line_capped(path, &line, max_bytes());
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

    /// T-3033: the session-daemon filter covers exactly the two blind surfaces.
    #[test]
    fn session_filter_records_kv_and_session_only() {
        for m in ["kv.get", "kv.set", "kv.delete", "kv.list", "session.update"] {
            assert!(is_recorded_session_method(m), "{m} must be recorded");
        }
        for m in ["event.poll", "event.subscribe", "command.execute", "query.status", "kvx.get", ""] {
            assert!(!is_recorded_session_method(m), "{m} must not be recorded");
        }
    }

    /// T-3033: a session-rpc record lands in the same sink shape as MCP/CLI.
    #[test]
    fn session_rpc_record_shape() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join(FILE_NAME);
        record_to(&p, SURFACE_SESSION_RPC, "kv.get");
        let lines = read_lines(&p);
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(v["surface"], "session-rpc");
        assert_eq!(v["name"], "kv.get");
    }

    // ── T-3615: attribution and the one-sink rule ─────────────────────────────

    fn attr(pairs: &[(&str, &str)]) -> Attribution {
        let m: std::collections::HashMap<String, String> =
            pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        attribution_from(|k| m.get(k).cloned())
    }

    #[test]
    fn class_dispatched_worker() {
        let a = attr(&[
            ("FW_DISPATCHED_WORKER", "1"),
            ("CLAUDE_CODE_ENTRYPOINT", "sdk-cli"),
            ("TERMLINK_AGENT_ID", "claude-termlink"),
            ("AI_AGENT", "claude-code_2-1-312_agent"),
            ("FW_SIDECAR_AGENT_ID", "t3615-build-1"),
        ]);
        assert_eq!(a.class, "dispatched");
        assert_eq!(a.harness.as_deref(), Some("claude-code_2-1-312_agent"));
        assert_eq!(a.worker.as_deref(), Some("t3615-build-1"));
    }

    #[test]
    fn class_reviewer_from_either_review_marker() {
        let a = attr(&[("FW_DISPATCHED_WORKER", "1"), ("FW_REVIEW_WORKER", "1")]);
        assert_eq!(a.class, "reviewer");
        let b = attr(&[("FW_DISPATCHED_WORKER", "1"), ("FW_REVIEWER_IN_DISPATCH", "1")]);
        assert_eq!(b.class, "reviewer");
    }

    #[test]
    fn class_standing_sidecar_and_systemd_service() {
        let a = attr(&[("TERMLINK_SIDECAR_SUPERVISOR", "systemd"), ("INVOCATION_ID", "x")]);
        assert_eq!(a.class, "standing");
        let b = attr(&[("INVOCATION_ID", "abc"), ("AI_AGENT", "claude-code_2-1-311_agent")]);
        assert_eq!(b.class, "standing", "an agent in a systemd service with no claude-fw marker");
    }

    fn attr_notty(pairs: &[(&str, &str)]) -> Attribution {
        let m: std::collections::HashMap<String, String> =
            pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        attribution_with(|k| m.get(k).cloned(), false)
    }

    #[test]
    fn class_background_systemd_service_without_agent() {
        // memwatch.sh / `termlink register --self` units: systemd, no agent marker.
        assert_eq!(attr(&[("INVOCATION_ID", "abc")]).class, "background");
        assert_eq!(attr(&[("INVOCATION_ID", "abc"), ("FW_CLAUDE_FW_SUPERVISED", "1")]).class,
                   "background", "a non-agent child of a claude-fw scope is still background");
    }

    #[test]
    fn class_background_no_tty_without_agent() {
        // cron, nohup/setsid loops, the vendored sidecar watcher.py (TERM set, stdin /dev/null).
        assert_eq!(attr_notty(&[]).class, "background");
        assert_eq!(attr_notty(&[("TERM", "xterm-256color"), ("PROJECT_ROOT", "/opt/x")]).class,
                   "background");
        assert_eq!(attr_notty(&[("TERMLINK_AGENT_ID", "claude-termlink")]).class, "background");
    }

    #[test]
    fn class_background_never_swallows_an_agent() {
        // An agent's Bash tool has a non-tty stdin but carries AI_AGENT: not background.
        assert_eq!(attr_notty(&[("AI_AGENT", "claude-code_2-1-311_agent")]).class, "unknown");
        assert_eq!(attr_notty(&[("CLAUDE_CODE_ENTRYPOINT", "cli")]).class, "interactive");
        assert_eq!(attr_notty(&[("FW_DISPATCHED_WORKER", "1")]).class, "dispatched");
        assert_eq!(attr_notty(&[("TERMLINK_SIDECAR_SUPERVISOR", "systemd")]).class, "standing");
        // A human at a terminal with no markers stays unknown.
        assert_eq!(attr(&[("TERM", "xterm-256color")]).class, "unknown");
    }

    #[test]
    fn class_background_explicit_declaration() {
        let a = attr(&[("TERMLINK_WORKER_CLASS", "background"), ("CLAUDE_CODE_ENTRYPOINT", "cli")]);
        assert_eq!(a.class, "background");
        assert!(CLASSES.contains(&"background"));
    }

    #[test]
    fn class_interactive_including_claude_fw_scope() {
        let a = attr(&[("CLAUDE_CODE_ENTRYPOINT", "cli"), ("AI_AGENT", "claude-code_2-1-311_agent")]);
        assert_eq!(a.class, "interactive");
        // claude-fw runs in a systemd scope (INVOCATION_ID) but is an operator's session.
        let b = attr(&[
            ("CLAUDE_CODE_ENTRYPOINT", "cli"),
            ("INVOCATION_ID", "b713"),
            ("FW_CLAUDE_FW_SUPERVISED", "1"),
            ("TERMLINK_AGENT_ID", "claude-termlink"),
        ]);
        assert_eq!(b.class, "interactive");
    }

    #[test]
    fn class_headless_sdk_without_fw_dispatch_is_dispatched() {
        assert_eq!(attr(&[("CLAUDE_CODE_ENTRYPOINT", "sdk-cli")]).class, "dispatched");
    }

    #[test]
    fn class_unknown_and_agent_id_is_not_a_signal() {
        assert_eq!(attr(&[]).class, "unknown");
        assert_eq!(attr(&[("TERMLINK_AGENT_ID", "claude-termlink")]).class, "unknown");
    }

    #[test]
    fn class_explicit_override_wins_and_bad_values_fall_through() {
        let a = attr(&[("TERMLINK_WORKER_CLASS", "Standing"), ("CLAUDE_CODE_ENTRYPOINT", "cli")]);
        assert_eq!(a.class, "standing");
        let b = attr(&[("TERMLINK_WORKER_CLASS", "boss"), ("CLAUDE_CODE_ENTRYPOINT", "cli")]);
        assert_eq!(b.class, "interactive", "an unknown declared class is ignored");
        let c = attr(&[("TERMLINK_WORKER_CLASS", "dispatched"), ("TERMLINK_WORKER_KIND", "codex"),
                       ("TERMLINK_MODEL", "gpt-5")]);
        assert_eq!((c.class, c.worker_kind.as_deref(), c.model.as_deref()),
                   ("dispatched", Some("codex"), Some("gpt-5")));
    }

    #[test]
    fn optional_fields_and_project_basename() {
        let a = attr(&[("ANTHROPIC_MODEL", "claude-opus-5-5"), ("FW_TASK_ID", "T-3615"),
                       ("PWD", "/opt/termlink/"), ("AI_AGENT", "  ")]);
        assert_eq!(a.model.as_deref(), Some("claude-opus-5-5"));
        assert_eq!(a.task.as_deref(), Some("T-3615"));
        assert_eq!(a.project.as_deref(), Some("termlink"));
        assert_eq!(a.harness, None, "blank is absent");
        let b = attr(&[("TERMLINK_PROJECT", "/x/010-termlink"), ("PWD", "/tmp")]);
        assert_eq!(b.project.as_deref(), Some("010-termlink"));
    }

    #[test]
    fn attributed_record_keeps_surface_and_name_and_adds_fields() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(FILE_NAME);
        let a = attr(&[("FW_DISPATCHED_WORKER", "1"), ("AI_AGENT", r#"we"ird"#), ("FW_TASK_ID", "T-1")]);
        record_to_with(&p, SURFACE_CLI, "channel subscribe", &a.json_fragment());
        let lines = read_lines(&p);
        let v: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(v["surface"], "cli");
        assert_eq!(v["name"], "channel subscribe");
        assert_eq!(v["class"], "dispatched");
        assert_eq!(v["harness"], r#"we"ird"#);
        assert_eq!(v["task"], "T-1");
        assert!(v.get("model").is_none(), "absent model is omitted, not null");
        assert!(v["ts"].as_u64().is_some());
    }

    #[test]
    fn plain_record_to_carries_a_class() {
        let td = TempDir::new().unwrap();
        let p = td.path().join(FILE_NAME);
        record_to(&p, SURFACE_MCP, "termlink_ping");
        let v: serde_json::Value = serde_json::from_str(&read_lines(&p)[0]).unwrap();
        assert!(CLASSES.contains(&v["class"].as_str().unwrap()));
    }

    #[test]
    fn sink_explicit_runtime_dir_wins() {
        let c = vec![PathBuf::from("/tmp/termlink-0"), PathBuf::from("/var/lib/termlink")];
        let d = resolve_sink_dir(Some(PathBuf::from("/srv/tl")), &c, |_| true);
        assert_eq!(d, PathBuf::from("/srv/tl"));
    }

    #[test]
    fn sink_without_override_prefers_the_hub_dir_over_tmp() {
        let c = vec![PathBuf::from("/var/tmp/termlink-0"), PathBuf::from("/var/lib/termlink"),
                     PathBuf::from("/tmp/termlink-0")];
        let d = resolve_sink_dir(None, &c, |p| p == Path::new("/var/lib/termlink"));
        assert_eq!(d, PathBuf::from("/var/lib/termlink"));
    }

    #[test]
    fn sink_primary_with_hub_is_unchanged_and_no_hub_keeps_today() {
        let c = vec![PathBuf::from("/run/user/0/termlink"), PathBuf::from("/var/lib/termlink")];
        assert_eq!(resolve_sink_dir(None, &c, |_| true), c[0]);
        assert_eq!(resolve_sink_dir(None, &c, |_| false), c[0], "no hub anywhere: today's primary");
    }

    #[test]
    fn sink_uses_a_real_hub_socket_on_disk() {
        let td = TempDir::new().unwrap();
        let stray = td.path().join("stray");
        let hub = td.path().join("hub");
        std::fs::create_dir_all(&hub).unwrap();
        std::fs::write(hub.join("hub.sock"), b"").unwrap();
        let d = resolve_sink_dir(None, &[stray, hub.clone()], |p| p.join("hub.sock").exists());
        assert_eq!(d, hub);
    }
}
