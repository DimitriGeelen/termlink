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
//! processes from the hub. Cross-process safety comes from the shared writer
//! (`audit_append`, T-3615): each line is ONE `O_APPEND` write, and rotation
//! holds a `flock` on `<sink>.lock` and re-checks the size, so two processes can
//! no longer both rotate and clobber the `.1` backup. The record shape stays an
//! append and never a read-modify-write.
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
//!   5. otherwise `unknown`. `background` is ONLY ever declared (rule 1): the
//!      loops we own carry `TERMLINK_WORKER_CLASS=background` (every
//!      `.context/cron/*.crontab`, the notify/waker loops, slice 3). Slice 3 also
//!      guessed `background` for any caller with no agent marker and either
//!      `INVOCATION_ID` or a non-tty stdin; the external review (T-3615 A2) showed
//!      that guess also catches a non-Claude harness's shell tool, an MCP server
//!      such a harness starts, and a human piping into termlink, i.e. it discounts
//!      real use as observer effect, the undercount this module exists to avoid.
//!      Such callers are now `unknown`, and the rule that matched is recorded.
//!
//!   `class_rule` (T-3615) names the rule that fired: `explicit`, `fw-dispatch`,
//!   `sidecar-unit`, `systemd-agent`, `entrypoint-cli`, `entrypoint-sdk`, and for
//!   `unknown`: `unmarked-service` (systemd, no agent marker), `unmarked-no-tty`
//!   (no agent marker, stdin not a terminal: cron, daemons, other harnesses' tools,
//!   pipes), or `no-rule` (a terminal, or a partly-marked harness).
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
//! the behaviour is today's (the primary runtime dir). A caller that cannot use
//! that dir (a non-root caller and a `drwx------ root` hub dir, as on .107; or any
//! dir it cannot write) writes a per-user sink, `$HOME/.termlink/`, which the
//! rollup also reads (review item 31; see [`resolve_sink_dir_for`]). This applies to the audit
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
    let dir = resolve_sink_dir_for(
        explicit,
        &crate::discovery::all_runtime_dirs(),
        probe_hub,
        sink_writable,
        user_sink_dir(std::env::var_os("HOME").map(PathBuf::from)),
    );
    let p = dir.join(FILE_NAME);
    let _ = PATH.set(Some(p.clone()));
    Some(p)
}

/// T-3615: pure core of the one-sink-per-host rule for a caller that can write
/// wherever it is pointed (see [`resolve_sink_dir_for`] for the full rule).
pub fn resolve_sink_dir(
    explicit: Option<PathBuf>,
    candidates: &[PathBuf],
    has_hub: impl Fn(&Path) -> bool,
) -> PathBuf {
    resolve_sink_dir_for(
        explicit,
        candidates,
        |d| if has_hub(d) { HubProbe::Present } else { HubProbe::Absent },
        |_| true,
        None,
    )
}

/// What a candidate runtime dir says about this host's hub socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubProbe {
    /// `hub.sock` is there.
    Present,
    /// The dir cannot be searched by this caller (`stat` says permission denied):
    /// the host's hub may well live there, but it is not ours to write.
    Hidden,
    /// No hub socket.
    Absent,
}

/// T-3615 (review item 31): the one-sink-per-host rule plus the non-root path.
/// Order: an explicit override; else the first candidate with a hub; else, if a
/// candidate is [`HubProbe::Hidden`], the per-user sink; else the primary
/// candidate. Whatever is chosen, a dir this caller cannot write falls back to
/// the per-user sink (`user_dir`, i.e. `$HOME/.termlink`) when there is one.
///
/// Why per-user and not a group-writable shared sink: on .107 the hub dir is
/// `drwx------ root`, so every non-root caller's rows were dropped (the write
/// error is swallowed by contract). A shared group-writable file does not stay
/// shared: rotation recreates it with the ROTATING process's umask (022 for
/// most callers, so 0644), after which every other user's appends fail again,
/// silently, until someone fixes the mode. It would also need a group created
/// and every agent user added to it on every host. A per-user file needs no
/// admin step, cannot be broken by another user's umask, and never widens the
/// hub dir. The cost moves to the reader: the rollup (root, via cron) also reads
/// `<home>/.termlink/invocation-audit.jsonl` for every account on the host.
pub fn resolve_sink_dir_for(
    explicit: Option<PathBuf>,
    candidates: &[PathBuf],
    probe: impl Fn(&Path) -> HubProbe,
    writable: impl Fn(&Path) -> bool,
    user_dir: Option<PathBuf>,
) -> PathBuf {
    let primary = || {
        candidates
            .first()
            .cloned()
            .unwrap_or_else(crate::discovery::runtime_dir)
    };
    let chosen = if let Some(dir) = explicit {
        dir
    } else {
        let probes: Vec<HubProbe> = candidates.iter().map(|d| probe(d)).collect();
        if let Some(i) = probes.iter().position(|p| *p == HubProbe::Present) {
            candidates[i].clone()
        } else if probes.contains(&HubProbe::Hidden)
            && let Some(u) = &user_dir
        {
            return u.clone();
        } else {
            primary()
        }
    };
    match user_dir {
        Some(u) if !writable(&chosen) => u,
        _ => chosen,
    }
}

/// `$HOME/.termlink`, the per-user sink dir (beside `queue.log`, `hubs.toml`).
pub fn user_sink_dir(home: Option<PathBuf>) -> Option<PathBuf> {
    home.filter(|h| !h.as_os_str().is_empty()).map(|h| h.join(".termlink"))
}

fn probe_hub(dir: &Path) -> HubProbe {
    match std::fs::metadata(dir.join("hub.sock")) {
        Ok(_) => HubProbe::Present,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => HubProbe::Hidden,
        Err(_) => HubProbe::Absent,
    }
}

/// Can this process append to the sink in `dir`? Tried for real (create the
/// dir, open the file for append), once per process, because `access(2)`-style
/// checks answer for the real uid and miss ACLs and read-only mounts.
fn sink_writable(dir: &Path) -> bool {
    let _ = std::fs::create_dir_all(dir);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(FILE_NAME))
        .is_ok()
}

/// Worker classes a record can carry (T-3615). See the module doc for the rules.
pub const CLASSES: [&str; 6] =
    ["interactive", "dispatched", "standing", "reviewer", "background", "unknown"];

/// Caller attribution for one process (T-3615). Built once from the env.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    pub class: &'static str,
    /// Which rule decided `class` (T-3615, review A2/A8/A20): `explicit`,
    /// `fw-dispatch`, `sidecar-unit`, `systemd-agent`, `entrypoint-cli`,
    /// `entrypoint-sdk`, `unmarked-service`, `unmarked-no-tty`, `no-rule`.
    pub class_rule: &'static str,
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
    let (class, class_rule) = if let Some(c) = get("TERMLINK_WORKER_CLASS")
        .and_then(|v| CLASSES.iter().copied().find(|c| *c == v.to_ascii_lowercase()))
    {
        (c, "explicit")
    } else if is1("FW_DISPATCHED_WORKER") {
        if is1("FW_REVIEW_WORKER") || is1("FW_REVIEWER_IN_DISPATCH") {
            ("reviewer", "fw-dispatch")
        } else {
            ("dispatched", "fw-dispatch")
        }
    } else if get("TERMLINK_SIDECAR_SUPERVISOR").is_some() {
        ("standing", "sidecar-unit")
    } else if systemd_service && agent_marked && get("FW_CLAUDE_FW_SUPERVISED").is_none() {
        ("standing", "systemd-agent")
    } else {
        match get("CLAUDE_CODE_ENTRYPOINT").as_deref() {
            Some("cli") => ("interactive", "entrypoint-cli"),
            Some(e) if e.starts_with("sdk") => ("dispatched", "entrypoint-sdk"),
            // Unmarked callers stay `unknown` (T-3615 review A2): a non-tty stdin
            // is also what a non-Claude harness's shell tool, an MCP server it
            // starts, or a human piping into termlink look like, so guessing
            // `background` would discount real use as observer effect. The rule
            // that would have fired is kept, so the rollup can still split them.
            _ if !agent_marked && systemd_service => ("unknown", "unmarked-service"),
            _ if !agent_marked && !stdin_tty => ("unknown", "unmarked-no-tty"),
            _ => ("unknown", "no-rule"),
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
        class_rule,
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
        let mut out = format!(
            r#","class":{},"class_rule":{}"#,
            json_escape(self.class),
            json_escape(self.class_rule)
        );
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
    fn unmarked_service_is_unknown_with_its_rule() {
        // T-3615 review A2: memwatch.sh-style units with no marker are no longer
        // guessed `background`; the rule that matched is recorded instead.
        let a = attr(&[("INVOCATION_ID", "abc")]);
        assert_eq!((a.class, a.class_rule), ("unknown", "unmarked-service"));
        let b = attr(&[("INVOCATION_ID", "abc"), ("FW_CLAUDE_FW_SUPERVISED", "1")]);
        assert_eq!((b.class, b.class_rule), ("unknown", "unmarked-service"));
    }

    #[test]
    fn unmarked_no_tty_is_unknown_with_its_rule() {
        // cron without the marker, the vendored watcher.py, another harness's shell
        // tool, a human piping into termlink: all look like this, so not `background`.
        for env in [
            vec![],
            vec![("TERM", "xterm-256color"), ("PROJECT_ROOT", "/opt/x")],
            vec![("TERMLINK_AGENT_ID", "claude-termlink")],
        ] {
            let a = attr_notty(&env);
            assert_eq!((a.class, a.class_rule), ("unknown", "unmarked-no-tty"), "{env:?}");
        }
        // Our own loops still declare it and stay background.
        let c = attr_notty(&[("TERMLINK_WORKER_CLASS", "background")]);
        assert_eq!((c.class, c.class_rule), ("background", "explicit"));
    }

    #[test]
    fn class_rule_names_every_path() {
        type Case<'a> = (&'a [(&'a str, &'a str)], bool, &'a str, &'a str);
        let cases: &[Case] = &[
            (&[("TERMLINK_WORKER_CLASS", "reviewer")], true, "reviewer", "explicit"),
            (&[("FW_DISPATCHED_WORKER", "1")], true, "dispatched", "fw-dispatch"),
            (&[("FW_DISPATCHED_WORKER", "1"), ("FW_REVIEW_WORKER", "1")], true, "reviewer", "fw-dispatch"),
            (&[("TERMLINK_SIDECAR_SUPERVISOR", "systemd")], true, "standing", "sidecar-unit"),
            (&[("INVOCATION_ID", "x"), ("AI_AGENT", "a")], true, "standing", "systemd-agent"),
            (&[("CLAUDE_CODE_ENTRYPOINT", "cli")], false, "interactive", "entrypoint-cli"),
            (&[("CLAUDE_CODE_ENTRYPOINT", "sdk-ts")], true, "dispatched", "entrypoint-sdk"),
            (&[("AI_AGENT", "a")], false, "unknown", "no-rule"),
            (&[], true, "unknown", "no-rule"),
        ];
        for (env, tty, class, rule) in cases {
            let m: std::collections::HashMap<String, String> =
                env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            let a = attribution_with(|k| m.get(k).cloned(), *tty);
            assert_eq!((a.class, a.class_rule), (*class, *rule), "{env:?} tty={tty}");
        }
    }

    #[test]
    fn record_carries_the_class_rule() {
        let frag = attr_notty(&[]).json_fragment();
        assert!(frag.starts_with(r#","class":"unknown","class_rule":"unmarked-no-tty""#), "{frag}");
    }

    #[test]
    fn class_background_never_swallows_an_agent() {
        // An agent's Bash tool has a non-tty stdin but carries AI_AGENT: never background.
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

    // ── T-3615 review item 31: the non-root path ────────────────────────────

    fn user() -> Option<PathBuf> {
        Some(PathBuf::from("/home/u/.termlink"))
    }

    #[test]
    fn sink_hidden_hub_dir_sends_a_non_root_caller_to_its_user_sink() {
        // .107: /var/lib/termlink is drwx------ root, so a non-root stat of
        // hub.sock is EACCES. Before: the stray /tmp/termlink-<uid>, read by nobody.
        let c = vec![PathBuf::from("/tmp/termlink-1000"), PathBuf::from("/var/lib/termlink")];
        let probe = |p: &Path| {
            if p == Path::new("/var/lib/termlink") { HubProbe::Hidden } else { HubProbe::Absent }
        };
        assert_eq!(resolve_sink_dir_for(None, &c, probe, |_| true, user()), user().unwrap());
        // No HOME: nothing better than today's primary.
        assert_eq!(resolve_sink_dir_for(None, &c, probe, |_| true, None), c[0]);
    }

    #[test]
    fn sink_unwritable_choice_falls_back_to_the_user_sink() {
        let c = vec![PathBuf::from("/var/lib/termlink")];
        let present = |_: &Path| HubProbe::Present;
        assert_eq!(resolve_sink_dir_for(None, &c, present, |_| false, user()), user().unwrap());
        assert_eq!(
            resolve_sink_dir_for(Some(PathBuf::from("/ro/tl")), &c, present, |_| false, user()),
            user().unwrap(),
            "an explicit dir this caller cannot write would drop every row"
        );
        // Writable: the hub dir wins as before (root, or a hub run by this user).
        assert_eq!(resolve_sink_dir_for(None, &c, present, |_| true, user()), c[0]);
    }

    #[test]
    fn user_sink_dir_is_home_dot_termlink() {
        assert_eq!(user_sink_dir(Some(PathBuf::from("/home/u"))), user());
        assert_eq!(user_sink_dir(Some(PathBuf::new())), None);
        assert_eq!(user_sink_dir(None), None);
    }

    const NONROOT_CHILD_ENV: &str = "TERMLINK_INVOCATION_AUDIT_NONROOT_CHILD";

    /// Child half of the real non-root test (a no-op otherwise): resolve the sink
    /// with the REAL probes against `<root>/hub` (root-owned, 0700, holds hub.sock)
    /// and record one row, then print where it went.
    #[test]
    fn nonroot_child() {
        let Ok(root) = std::env::var(NONROOT_CHILD_ENV) else { return };
        let root = PathBuf::from(root);
        let c = vec![root.join("primary"), root.join("hub")];
        let dir = resolve_sink_dir_for(
            None,
            &c,
            probe_hub,
            sink_writable,
            user_sink_dir(std::env::var_os("HOME").map(PathBuf::from)),
        );
        let path = dir.join(FILE_NAME);
        record_to_with(&path, SURFACE_CLI, "version", r#","class":"unknown""#);
        println!("SINK={}", path.display());
    }

    /// T-3615 item 31, for real: a non-root process (uid 65534 via setpriv) and a
    /// root-only hub dir. Its row must land in its own `$HOME/.termlink` sink, not
    /// be dropped and not go to a stray tmp dir. Runs only as root with setpriv.
    #[test]
    fn nonroot_caller_writes_its_user_sink_for_real() {
        let is_root = unsafe { libc::geteuid() } == 0;
        let setpriv = Path::new("/usr/bin/setpriv");
        if !is_root || !setpriv.exists() {
            eprintln!("skipped: needs root and /usr/bin/setpriv");
            return;
        }
        use std::os::unix::fs::PermissionsExt;
        let td = TempDir::new().unwrap();
        std::fs::set_permissions(td.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        let hub = td.path().join("hub");
        std::fs::create_dir_all(&hub).unwrap();
        std::fs::write(hub.join("hub.sock"), b"").unwrap();
        std::fs::set_permissions(&hub, std::fs::Permissions::from_mode(0o700)).unwrap();
        let home = td.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let cs = std::ffi::CString::new(home.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::chown(cs.as_ptr(), 65534, 65534) }, 0);
        let exe = std::env::current_exe().unwrap();
        // The child must be able to exec the test binary; skip if a parent dir
        // of it is closed to other users (some build hosts).
        let probe = std::process::Command::new(setpriv)
            .args(["--reuid=65534", "--regid=65534", "--clear-groups", "--"])
            .arg(&exe)
            .arg("--list")
            .output()
            .unwrap();
        if !probe.status.success() {
            eprintln!("skipped: uid 65534 cannot exec {}", exe.display());
            return;
        }
        let out = std::process::Command::new(setpriv)
            .args(["--reuid=65534", "--regid=65534", "--clear-groups", "--"])
            .arg(&exe)
            .args(["--exact", "invocation_audit::tests::nonroot_child", "--nocapture", "--test-threads=1"])
            .env(NONROOT_CHILD_ENV, td.path())
            .env("HOME", &home)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let stdout = String::from_utf8_lossy(&out.stdout);
        let want = home.join(".termlink").join(FILE_NAME);
        assert!(stdout.contains(&format!("SINK={}", want.display())), "{stdout}");
        let lines = read_lines(&want);
        assert_eq!(lines.len(), 1, "the non-root row is kept");
        assert!(!hub.join(FILE_NAME).exists(), "the root-only hub dir is never written or widened");
        let mode = std::fs::metadata(&hub).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
    }
}
