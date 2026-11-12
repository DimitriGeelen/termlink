//! Claude Code hook entry points run by `termlink-sidecar hook <event>` from the agent's settings.
//! C-11 Claude adapter: hook entry points (architecture note 6.2, 6.3, 6.4).
//!
//! `termlink-sidecar hook <event>` is what the agent's `settings.json` runs.
//! It reads Claude Code's hook input from stdin and talks to its own sidecar on
//! the send socket named by `TERMLINK_SIDECAR_SOCKET`.
//!
//! | event | does |
//! |---|---|
//! | `post-tool-use` | `pull`; prints the frame as `additionalContext` when one came back, then spawns a detached finalizer |
//! | `session-start` | `ready` (records the session; the transcript is found by the sidecar under its own HOME) |
//! | `vendor-channel` | PreToolUse for `SendMessage`, `Agent`, `Task` (P-26, AL-39, K-20): `vendor-use`, one admin row per call; prints nothing, never blocks |
//! | `stop` | `ready` only: a Stop hook cannot hand text to the agent without blocking the turn, which PH-1 does not do, so mail waits for the next tool call |
//!
//! Fail directions (6.3): the **turn** fails open: any error (no socket,
//! refused, timeout, bad answer) prints nothing, exits 0 and appends one line to
//! `hook-errors.log`, so the agent is never blocked and the message stays
//! STORED where the sender sees it waiting. The **delivery** fails closed: a
//! frame is printed only when the sidecar returned one, and the hook never
//! records a hand-over; the finalizer asks the sidecar, which reads the
//! transcript.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const SOCKET_ENV: &str = "TERMLINK_SIDECAR_SOCKET";
pub const ERRLOG_ENV: &str = "TERMLINK_SIDECAR_HOOK_ERRORS";
pub const FINALIZE_SECS_ENV: &str = "TERMLINK_SIDECAR_FINALIZE_SECS";
/// Bound on one socket exchange from a hook.
pub const HOOK_TIMEOUT: Duration = Duration::from_secs(3);
/// Largest hook input read from stdin.
pub const MAX_INPUT: u64 = 4 * 1024 * 1024;

/// One request/response on the sidecar socket.
pub fn call(socket: &Path, req: &Value, timeout: Duration) -> Result<Value, String> {
    let mut s = UnixStream::connect(socket).map_err(|e| format!("connect {}: {e}", socket.display()))?;
    s.set_read_timeout(Some(timeout)).map_err(|e| e.to_string())?;
    s.set_write_timeout(Some(timeout)).map_err(|e| e.to_string())?;
    writeln!(s, "{req}").map_err(|e| format!("write: {e}"))?;
    let mut line = String::new();
    BufReader::new(s).take(64 * 1024 * 1024).read_line(&mut line).map_err(|e| format!("read: {e}"))?;
    if line.is_empty() {
        return Err("the sidecar closed the connection without answering".into());
    }
    serde_json::from_str(&line).map_err(|e| format!("bad answer: {e}"))
}

/// CR-6.4 / AL-11 (T-3694, E-153): send one `prompt_submit` adapter report and return the
/// origin and basis the lease claim must carry: the receiver-checked ones from the reply. Any
/// failure (no sidecar, refused, a reply without `origin_checked`) is `(unknown, absent)`, so
/// a prompt the receiver could not check never renews main (FCW-195; never the hook's own
/// `human`).
pub fn checked_origin(socket: &Path, report: &Value, timeout: Duration) -> (String, String) {
    let unknown = || ("unknown".to_string(), "absent".to_string());
    let Ok(r) = call(socket, report, timeout) else { return unknown() };
    match (r.get("origin_checked").and_then(Value::as_str), r.get("basis_checked").and_then(Value::as_str)) {
        (Some(o), Some(b)) if matches!(o, "human" | "automated" | "unknown") => (o.to_string(), b.to_string()),
        _ => unknown(),
    }
}

/// Where hook errors go: `$TERMLINK_SIDECAR_HOOK_ERRORS`, else beside the socket.
pub fn errlog_path(socket: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = std::env::var_os(ERRLOG_ENV) {
        return Some(PathBuf::from(p));
    }
    socket.and_then(Path::parent).map(|d| d.join("hook-errors.log"))
}

fn log_error(log: Option<&Path>, event: &str, why: &str) {
    let Some(log) = log else { return };
    use std::os::unix::fs::OpenOptionsExt;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).mode(0o600).open(log) {
        let _ = writeln!(f, "{} hook {event}: {}", crate::store::now_ms(), why.replace('\n', " "));
    }
}

/// The tools whose use goes over the vendor's own channel and is logged (P-26,
/// AL-39, E-71). The settings matcher is these names joined by `|`.
pub const VENDOR_TOOLS: [&str; 3] = ["SendMessage", "Agent", "Task"];
/// The PreToolUse matcher the settings entry must carry.
pub fn vendor_matcher() -> String {
    VENDOR_TOOLS.join("|")
}

/// The tool a `vendor-channel` hook reports: `--tool` when given, else the hook
/// input's `tool_name`. `None` for a tool outside [`VENDOR_TOOLS`] (not logged).
pub fn vendor_tool<'a>(explicit: Option<&'a str>, input: &'a Value) -> Option<&'a str> {
    let t = explicit.or_else(|| input.get("tool_name").and_then(Value::as_str))?;
    VENDOR_TOOLS.contains(&t).then_some(t)
}

/// The sidecar request for one hook event. Every request carries this host's
/// `boot_id` and `mono_ns` (CLOCK_BOOTTIME on Linux, as a decimal string) so the
/// meter orders hook calls on them, not on a wall clock (04 17.2.a, CR-6.24, P-2);
/// where there is no boot id neither is sent and the receive stamp orders them.
pub fn request(event: &str, session: &str, input: &Value) -> Result<Value, String> {
    let mut r = match event {
        "post-tool-use" => {
            let mut r = json!({"op": "pull", "session": session, "hook": "post-tool-use"});
            if let Some(t) = input.get("tool_use_id").and_then(Value::as_str) {
                r["tool_use_id"] = json!(t);
            }
            r
        }
        "session-start" | "stop" | "session-end" => json!({"op": "ready", "session": session, "hook": event}),
        "vendor-channel" => match vendor_tool(input.get(TOOL_ARG_KEY).and_then(Value::as_str), input) {
            Some(t) => json!({"op": "vendor-use", "session": session, "tool": t}),
            None => return Err(NOT_VENDOR.into()),
        },
        other => return Err(format!("unknown hook event {other:?}")),
    };
    if let (Some(boot), Some(ns)) = (termlink_session::input_meter::boot_id(), mono_ns()) {
        r["boot_id"] = json!(boot);
        r["mono_ns"] = json!(ns.to_string());
    }
    Ok(r)
}

/// A host-monotonic value in ns that keeps counting through suspend (04 17.2.a):
/// CLOCK_BOOTTIME on Linux, CLOCK_MONOTONIC (which counts through sleep) on macOS.
pub fn mono_ns() -> Option<u64> {
    #[cfg(target_os = "linux")]
    let clock = libc::CLOCK_BOOTTIME;
    #[cfg(not(target_os = "linux"))]
    let clock = libc::CLOCK_MONOTONIC;
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes one timespec through a valid pointer.
    let rc = unsafe { libc::clock_gettime(clock, &mut ts) };
    (rc == 0).then(|| (ts.tv_sec as u64).saturating_mul(1_000_000_000).saturating_add(ts.tv_nsec as u64))
}

/// Key under which `main_hook` passes a `--tool` argument into the hook input.
const TOOL_ARG_KEY: &str = "__termlink_tool_arg";
/// `request` answer for a tool that is not a vendor channel: nothing to log.
const NOT_VENDOR: &str = "not a vendor-channel tool";

/// What a hook run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookOutcome {
    /// What to print on stdout (nothing when `None`).
    pub stdout: Option<String>,
    /// A nonce the finalizer must follow.
    pub finalize: Option<String>,
    /// Why nothing was printed, when an error caused it (also logged).
    pub error: Option<String>,
}

/// Run one hook event against the sidecar on `socket` with Claude Code's hook
/// input `input`. Never fails: errors become an empty outcome and a log line.
pub fn run(event: &str, input: &str, socket: Option<&Path>, log: Option<&Path>) -> HookOutcome {
    let none = |why: String| {
        log_error(log, event, &why);
        HookOutcome { stdout: None, finalize: None, error: Some(why) }
    };
    let Some(socket) = socket else {
        return none(format!("{SOCKET_ENV} is not set"));
    };
    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return none(format!("hook input is not JSON: {e}")),
    };
    let Some(session) = v.get("session_id").and_then(Value::as_str) else {
        return none("hook input has no session_id".into());
    };
    let req = match request(event, session, &v) {
        Ok(r) => r,
        // A tool the matcher let through that is not a vendor channel: no row, no error.
        Err(why) if why == NOT_VENDOR => return HookOutcome { stdout: None, finalize: None, error: None },
        Err(why) => return none(why),
    };
    let resp = match call(socket, &req, HOOK_TIMEOUT) {
        // A sidecar started before P-2 refuses the order fields: ask again without
        // them (it then orders the call by its receive stamp) rather than lose the call.
        Ok(r) if r.get("code").and_then(Value::as_str) == Some("unknown-field") && req.get("mono_ns").is_some() => {
            let mut plain = req.clone();
            if let Some(o) = plain.as_object_mut() {
                o.remove("boot_id");
                o.remove("mono_ns");
            }
            match call(socket, &plain, HOOK_TIMEOUT) {
                Ok(r) => r,
                Err(e) => return none(e),
            }
        }
        Ok(r) => r,
        Err(e) => return none(e),
    };
    if let Some(code) = resp.get("code").and_then(Value::as_str) {
        return none(format!("sidecar refused ({code}): {}", resp["error"].as_str().unwrap_or("")));
    }
    if event != "post-tool-use" {
        return HookOutcome { stdout: None, finalize: None, error: None };
    }
    match (resp.get("frame").and_then(Value::as_str), resp.get("nonce").and_then(Value::as_str)) {
        (Some(frame), Some(nonce)) => {
            let out = json!({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": frame}});
            HookOutcome { stdout: Some(out.to_string()), finalize: Some(nonce.to_owned()), error: None }
        }
        // T-3654: a session that is not main is told once per holder generation who main is.
        _ if resp.get("none").and_then(Value::as_bool) == Some(true) => {
            let line = resp.get("not_main").and_then(Value::as_str).filter(|_| resp.get("notice").and_then(Value::as_bool) == Some(true));
            let stdout = line.map(|l| json!({"hookSpecificOutput": {"hookEventName": "PostToolUse", "additionalContext": l}}).to_string());
            HookOutcome { stdout, finalize: None, error: None }
        }
        _ => none(format!("unexpected answer: {resp}")),
    }
}

/// T-3463: is the Claude process running this hook itself started by another
/// Claude? A session joins its sidecar only by being LAUNCHED with the socket
/// (operator ruling P7-1 A), but every process an agent starts inherits its
/// environment, so a `claude -p` an agent runs from its own shell would otherwise
/// run these hooks and could be handed that agent's mail.
///
/// Walks the ancestors of `start_pid` under `proc_root`: the first `claude` is the
/// hook's own session; a second one above it means nested. `Some(false)` when the
/// walk reached the top with at most one; `None` when the ancestry cannot be read
/// (no procfs, e.g. macOS), in which case the caller keeps the pre-T-3463
/// behaviour and joins.
pub fn nested_claude_at(proc_root: &Path, start_pid: u32) -> Option<bool> {
    let mut pid = start_pid;
    let mut seen = 0u32;
    for _ in 0..64 {
        if pid <= 1 {
            return Some(false);
        }
        let dir = proc_root.join(pid.to_string());
        let comm = std::fs::read_to_string(dir.join("comm")).ok()?;
        if comm.trim() == "claude" {
            seen += 1;
            if seen >= 2 {
                return Some(true);
            }
        }
        // ppid is the 2nd field after the LAST ')': comm may hold spaces and ')'.
        let stat = std::fs::read_to_string(dir.join("stat")).ok()?;
        let rest = &stat[stat.rfind(')')? + 1..];
        pid = rest.split_whitespace().nth(1)?.parse().ok()?;
    }
    None
}

/// T-3463: [`nested_claude_at`] on this host, starting from the hook's parent.
fn nested_claude() -> Option<bool> {
    let root = Path::new("/proc");
    if !root.join("self").exists() {
        return None;
    }
    nested_claude_at(root, std::os::unix::process::parent_id())
}

/// Read stdin (bounded), run the hook, print, spawn the finalizer. Exit code is
/// always 0: the turn fails open.
pub fn main_hook(event: &str) -> i32 {
    main_hook_with(event, None)
}

/// [`main_hook`] with the `vendor-channel` event's `--tool <name>` (AL-39).
pub fn main_hook_with(event: &str, tool: Option<&str>) -> i32 {
    let socket = std::env::var_os(SOCKET_ENV).map(PathBuf::from);
    let log = errlog_path(socket.as_deref());
    // T-3463: a Claude started by an agent inherited that agent's socket; it never
    // joins. Said once per session (at session-start), not on every tool call.
    if socket.is_some() && nested_claude() == Some(true) {
        if event == "session-start" {
            log_error(log.as_deref(), event, "nested Claude session (started by another Claude): not joining the sidecar (T-3463, P7-1 A)");
        }
        return 0;
    }
    let mut input = String::new();
    if let Err(e) = std::io::stdin().take(MAX_INPUT).read_to_string(&mut input) {
        log_error(log.as_deref(), event, &format!("cannot read stdin: {e}"));
        return 0;
    }
    if let Some(t) = tool {
        // The explicit tool rides in the input, so `run` stays one pure entry point.
        if let Ok(Value::Object(mut o)) = serde_json::from_str::<Value>(&input) {
            o.insert(TOOL_ARG_KEY.into(), json!(t));
            input = Value::Object(o).to_string();
        }
    }
    let out = run(event, &input, socket.as_deref(), log.as_deref());
    if let Some(s) = &out.stdout {
        println!("{s}");
    }
    if let (Some(nonce), Some(sock)) = (&out.finalize, &socket)
        && let Err(e) = spawn_finalizer(nonce, sock)
    {
        log_error(log.as_deref(), event, &format!("cannot start the finalizer for {nonce}: {e}"));
    }
    0
}

/// The arguments the detached finalizer is started with: the re-exec prefix,
/// then `finalize --nonce N`.
pub fn finalizer_args(nonce: &str) -> Vec<String> {
    let mut v: Vec<String> = crate::cli::reexec_prefix().to_vec();
    v.extend(["finalize".to_string(), "--nonce".to_string(), nonce.to_string()]);
    v
}

/// Start `<this binary> [prefix] finalize --nonce N` detached (own session, no
/// stdio), so the hook returns at once and the finalizer outlives it. The prefix
/// is `sidecar` when this runs as `termlink sidecar` (PH-2 row P1).
pub fn spawn_finalizer(nonce: &str, socket: &Path) -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    let exe = std::env::current_exe()?;
    let mut c = std::process::Command::new(exe);
    c.args(finalizer_args(nonce))
        .env(SOCKET_ENV, socket)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    // SAFETY: setsid is async-signal-safe and takes no arguments.
    unsafe {
        c.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    c.spawn().map(|_| ())
}

/// Poll `handed-over` until the attempt is decided or the window ends, then make
/// the final check. Returns the last answer.
pub fn finalize(socket: &Path, nonce: &str, window: Duration, every: Duration) -> Result<Value, String> {
    let end = Instant::now() + window;
    loop {
        let r = call(socket, &json!({"op": "handed-over", "nonce": nonce}), HOOK_TIMEOUT)?;
        if r.get("code").is_some() {
            return Err(format!("sidecar refused: {r}"));
        }
        let decided = r["stage"] == "HANDED_OVER" || r["attempt"].as_str().is_some_and(|a| a.starts_with("closed"));
        if decided {
            return Ok(r);
        }
        if Instant::now() >= end {
            break;
        }
        std::thread::sleep(every);
    }
    call(socket, &json!({"op": "handed-over", "nonce": nonce, "final": true}), HOOK_TIMEOUT)
}

pub fn main_finalize(nonce: &str) -> i32 {
    let socket = std::env::var_os(SOCKET_ENV).map(PathBuf::from);
    let log = errlog_path(socket.as_deref());
    let Some(socket) = socket else {
        log_error(log.as_deref(), "finalize", &format!("{SOCKET_ENV} is not set"));
        return 1;
    };
    let secs = std::env::var(FINALIZE_SECS_ENV).ok().and_then(|s| s.parse().ok()).unwrap_or(60u64);
    match finalize(&socket, nonce, Duration::from_secs(secs), Duration::from_millis(500)) {
        Ok(_) => 0,
        Err(e) => {
            log_error(log.as_deref(), "finalize", &format!("{nonce}: {e}"));
            1
        }
    }
}
