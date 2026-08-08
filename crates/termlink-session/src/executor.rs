use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Result of executing a shell command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    /// T-2529: `true` iff stdout or stderr hit `MAX_OUTPUT_BYTES` and the child was
    /// killed to stop unbounded capture into the daemon heap. `#[serde(default)]`
    /// keeps the wire format backward-compatible with pre-T-2529 producers.
    #[serde(default)]
    pub truncated: bool,
}

impl ExecResult {
    /// Canonical JSON for the exec round-trip surfaced at every boundary
    /// (`command.execute` RPC, `termlink exec --json` CLI, MCP `termlink_run` /
    /// `termlink_batch_run`). T-2537: `truncated` MUST be one of the emitted
    /// fields — without it a caller receiving `exit_code:-1` cannot tell a
    /// T-2529 cap-hit from a signal kill (both render `-1`), and in the ~64 KiB
    /// band around the cap a truncated result even reads as `exit_code:0`.
    /// Callers that add envelope fields (`ok`, `command`, `session_id`,
    /// `elapsed_ms`) merge them on top of these core fields.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
            "truncated": self.truncated,
        })
    }
}

/// Maximum command length to prevent abuse (64 KiB).
const MAX_COMMAND_LEN: usize = 65_536;

/// Maximum captured output per stream (stdout / stderr) before the child is killed and
/// the result flagged `truncated` (16 MiB — mirrors the bus `MAX_LINE_BYTES` /
/// `MAX_PAYLOAD_SIZE` convention). T-2529: `Command::output()` reads the child's ENTIRE
/// stdout+stderr into `Vec<u8>` with no bound, so a single `execute` of an unbounded
/// producer (`yes`, `cat /dev/zero`) — or an ACCIDENTAL `cat biglog` / `find /` — grew
/// the long-lived hub/session daemon heap at pipe speed until OOM. This caps it.
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

/// Default exec timeout applied when the caller supplies none.
const DEFAULT_EXEC_TIMEOUT_SECS: u64 = 30;

/// Hard upper bound on a caller-supplied exec timeout (1 hour). T-2530: the timeout
/// arrives as a caller-controlled JSON u64 (`handler.rs` `payload.timeout` →
/// `Duration::from_secs`, reachable via Execute-scope RPC / `termlink_remote_exec` /
/// MCP `termlink_exec` / `termlink_dispatch`) and was previously applied verbatim.
/// An unbounded timeout defeats the timeout: a low-output long-runner (`sleep
/// 99999999`) is never reclaimed — the session task and child are held indefinitely —
/// and `Duration::from_secs(u64::MAX)` fed to `tokio::time::timeout` risks an
/// `Instant + Duration` overflow panic. Mirrors the repo "clamp every caller numeric
/// param" convention (the size twin, `MAX_OUTPUT_BYTES`, shipped in T-2529).
/// Overridable via `TERMLINK_MAX_EXEC_TIMEOUT_SECS` (clamped `[1, 86_400]`) for
/// operator tuning / tests. Synchronous captured-output exec has no legitimate need
/// to run longer than this — long work belongs on an async session, not a blocking
/// `execute`.
const MAX_EXEC_TIMEOUT_SECS: u64 = 3_600;

/// Resolve the effective upper bound on an exec timeout, honoring the env override.
fn max_exec_timeout() -> Duration {
    let secs = std::env::var("TERMLINK_MAX_EXEC_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(1, 86_400))
        .unwrap_or(MAX_EXEC_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Clamp a caller-supplied exec timeout to the `[_, max_exec_timeout()]` band,
/// applying `DEFAULT_EXEC_TIMEOUT_SECS` when absent. T-2530.
fn effective_exec_timeout(timeout: Option<Duration>) -> Duration {
    timeout
        .unwrap_or(Duration::from_secs(DEFAULT_EXEC_TIMEOUT_SECS))
        .min(max_exec_timeout())
}

/// Validate a command string for safe shell execution.
/// Rejects null bytes, invalid UTF-8 control sequences, and oversized commands.
fn validate_command(command: &str) -> Result<(), ExecError> {
    if command.is_empty() {
        return Err(ExecError::Validation("command must not be empty".into()));
    }
    if command.len() > MAX_COMMAND_LEN {
        return Err(ExecError::Validation(format!(
            "command too long ({} bytes, max {})",
            command.len(),
            MAX_COMMAND_LEN
        )));
    }
    if command.contains('\0') {
        return Err(ExecError::Validation(
            "command must not contain null bytes".into(),
        ));
    }
    Ok(())
}

/// Validate a command against an allowlist of prefix patterns.
///
/// If `allowed_commands` is `None`, all commands are allowed (backward compatible).
/// If `Some(&[])` (empty list), NO commands are allowed.
/// Otherwise, the command must start with at least one of the allowed prefixes.
fn validate_allowlist(command: &str, allowed_commands: Option<&[String]>) -> Result<(), ExecError> {
    if let Some(prefixes) = allowed_commands {
        let trimmed = command.trim_start();
        let allowed = prefixes
            .iter()
            .any(|prefix| trimmed.starts_with(prefix.as_str()));
        if !allowed {
            return Err(ExecError::NotAllowed(command.to_string()));
        }
    }
    Ok(())
}

/// Execute a shell command with optional timeout, working directory, and env vars.
///
/// # Security
/// The command string is passed to `sh -c` and executed as a shell command.
/// Callers must ensure commands are trusted or come from authenticated sources.
/// Input validation rejects null bytes and oversized commands, but does NOT
/// sanitize shell metacharacters — that is the caller's responsibility.
///
/// If `allowed_commands` is provided, the command must match at least one prefix.
pub async fn execute(
    command: &str,
    cwd: Option<&str>,
    env: Option<&std::collections::HashMap<String, String>>,
    timeout: Option<Duration>,
    allowed_commands: Option<&[String]>,
) -> Result<ExecResult, ExecError> {
    execute_capped(command, cwd, env, timeout, allowed_commands, MAX_OUTPUT_BYTES).await
}

/// T-2529: `execute` with an explicit per-stream output cap. `Command::output()` reads
/// the child's ENTIRE stdout+stderr into memory with no bound — so instead we spawn with
/// piped stdio and read both streams CONCURRENTLY in a chunked `select!` loop, killing
/// the child the instant either buffer exceeds `max_output_bytes` (so an unbounded
/// producer can never grow the daemon heap past the cap, and a flood on ONE stream does
/// not block on the OTHER stream's EOF). `execute` delegates with `MAX_OUTPUT_BYTES`; the
/// cap is a parameter so tests inject a tiny bound without emitting 16 MiB.
async fn execute_capped(
    command: &str,
    cwd: Option<&str>,
    env: Option<&std::collections::HashMap<String, String>>,
    timeout: Option<Duration>,
    allowed_commands: Option<&[String]>,
    max_output_bytes: usize,
) -> Result<ExecResult, ExecError> {
    use tokio::io::AsyncReadExt;

    validate_command(command)?;
    validate_allowlist(command, allowed_commands)?;

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    if let Some(env_vars) = env {
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Kill the spawned child if the run future is dropped (e.g. on timeout). Without
    // this, a timed-out command orphans the child, which keeps running to its natural
    // completion — a resource leak on the "control terminal sessions" path (`termlink
    // exec`, session RPC, MCP exec/batch all route through here). T-2509.
    cmd.kill_on_drop(true);

    let timeout_dur = effective_exec_timeout(timeout);

    let run = async {
        let mut child = cmd.spawn().map_err(ExecError::Spawn)?;
        let mut cout = child.stdout.take().expect("stdout piped above");
        let mut cerr = child.stderr.take().expect("stderr piped above");

        let mut obuf: Vec<u8> = Vec::new();
        let mut ebuf: Vec<u8> = Vec::new();
        let mut chunk_o = [0u8; 8192];
        let mut chunk_e = [0u8; 8192];
        let mut o_done = false;
        let mut e_done = false;
        let mut truncated = false;

        // Read whichever stream is ready; check the cap after each chunk. A chunked
        // select loop (not `join!` on `read_to_end`) is what lets a flood on one stream
        // trigger an immediate kill instead of blocking on the other stream's EOF.
        while !(o_done && e_done) {
            tokio::select! {
                r = cout.read(&mut chunk_o), if !o_done => match r {
                    Ok(0) => o_done = true,
                    Ok(n) => obuf.extend_from_slice(&chunk_o[..n]),
                    Err(e) => return Err(ExecError::Spawn(e)),
                },
                r = cerr.read(&mut chunk_e), if !e_done => match r {
                    Ok(0) => e_done = true,
                    Ok(n) => ebuf.extend_from_slice(&chunk_e[..n]),
                    Err(e) => return Err(ExecError::Spawn(e)),
                },
            }
            if obuf.len() > max_output_bytes || ebuf.len() > max_output_bytes {
                truncated = true;
                obuf.truncate(max_output_bytes);
                ebuf.truncate(max_output_bytes);
                // Stop the (now pipe-blocked) child; we will read no more from it.
                let _ = child.kill().await;
                break;
            }
        }

        let status = child.wait().await.map_err(ExecError::Spawn)?;

        Ok::<ExecResult, ExecError>(ExecResult {
            exit_code: status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&obuf).into_owned(),
            stderr: String::from_utf8_lossy(&ebuf).into_owned(),
            truncated,
        })
    };

    tokio::time::timeout(timeout_dur, run)
        .await
        .map_err(|_| ExecError::Timeout(timeout_dur))?
}

/// Resolve a symbolic key name to its raw byte sequence.
pub fn resolve_key(name: &str) -> Option<Vec<u8>> {
    match name {
        // Control characters
        "Ctrl+A" | "ctrl+a" => Some(vec![0x01]),
        "Ctrl+B" | "ctrl+b" => Some(vec![0x02]),
        "Ctrl+C" | "ctrl+c" => Some(vec![0x03]),
        "Ctrl+D" | "ctrl+d" => Some(vec![0x04]),
        "Ctrl+E" | "ctrl+e" => Some(vec![0x05]),
        "Ctrl+F" | "ctrl+f" => Some(vec![0x06]),
        "Ctrl+G" | "ctrl+g" => Some(vec![0x07]),
        "Ctrl+H" | "ctrl+h" => Some(vec![0x08]),
        "Ctrl+K" | "ctrl+k" => Some(vec![0x0B]),
        "Ctrl+L" | "ctrl+l" => Some(vec![0x0C]),
        "Ctrl+N" | "ctrl+n" => Some(vec![0x0E]),
        "Ctrl+O" | "ctrl+o" => Some(vec![0x0F]),
        "Ctrl+P" | "ctrl+p" => Some(vec![0x10]),
        "Ctrl+Q" | "ctrl+q" => Some(vec![0x11]),
        "Ctrl+R" | "ctrl+r" => Some(vec![0x12]),
        "Ctrl+S" | "ctrl+s" => Some(vec![0x13]),
        "Ctrl+T" | "ctrl+t" => Some(vec![0x14]),
        "Ctrl+U" | "ctrl+u" => Some(vec![0x15]),
        "Ctrl+V" | "ctrl+v" => Some(vec![0x16]),
        "Ctrl+W" | "ctrl+w" => Some(vec![0x17]),
        "Ctrl+X" | "ctrl+x" => Some(vec![0x18]),
        "Ctrl+Y" | "ctrl+y" => Some(vec![0x19]),
        "Ctrl+Z" | "ctrl+z" => Some(vec![0x1A]),
        "Ctrl+\\" | "ctrl+\\" => Some(vec![0x1C]),

        // Special keys
        "Enter" | "enter" | "Return" | "return" => Some(vec![0x0D]),
        "Tab" | "tab" => Some(vec![0x09]),
        "Backspace" | "backspace" => Some(vec![0x7F]),
        "Escape" | "escape" | "Esc" | "esc" => Some(vec![0x1B]),
        "Delete" | "delete" | "Del" | "del" => Some(vec![0x1B, 0x5B, 0x33, 0x7E]),
        "Space" | "space" => Some(vec![0x20]),

        // Arrow keys (ANSI)
        "Up" | "up" => Some(vec![0x1B, 0x5B, 0x41]),
        "Down" | "down" => Some(vec![0x1B, 0x5B, 0x42]),
        "Right" | "right" => Some(vec![0x1B, 0x5B, 0x43]),
        "Left" | "left" => Some(vec![0x1B, 0x5B, 0x44]),

        // Home/End
        "Home" | "home" => Some(vec![0x1B, 0x5B, 0x48]),
        "End" | "end" => Some(vec![0x1B, 0x5B, 0x46]),

        _ => None,
    }
}

/// Resolve a KeyEntry (from T-005 protocol) to raw bytes.
pub fn resolve_key_entry(entry: &termlink_protocol::control::KeyEntry) -> Result<Vec<u8>, String> {
    match entry {
        termlink_protocol::control::KeyEntry::Text(text) => Ok(text.as_bytes().to_vec()),
        termlink_protocol::control::KeyEntry::Key(name) => {
            resolve_key(name).ok_or_else(|| format!("Unknown key: {name}"))
        }
        termlink_protocol::control::KeyEntry::Raw(b64) => {
            // Simple base64 decode (no dependency needed for basic cases)
            base64_decode(b64).map_err(|e| format!("Invalid base64: {e}"))
        }
    }
}

/// Resolve a sequence of KeyEntries to a single byte buffer.
pub fn resolve_keys(entries: &[termlink_protocol::control::KeyEntry]) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    for entry in entries {
        buf.extend(resolve_key_entry(entry)?);
    }
    Ok(buf)
}

/// Send a POSIX signal to a process.
pub fn send_signal(pid: u32, signal: i32) -> Result<(), ExecError> {
    let ret = unsafe { libc::kill(pid as i32, signal) };
    if ret == 0 {
        Ok(())
    } else {
        Err(ExecError::Signal(
            signal,
            std::io::Error::last_os_error(),
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("command timed out after {0:?}")]
    Timeout(Duration),

    #[error("failed to spawn command: {0}")]
    Spawn(std::io::Error),

    #[error("failed to send signal {0}: {1}")]
    Signal(i32, std::io::Error),

    #[error("command validation failed: {0}")]
    Validation(String),

    #[error("command not in allowlist: {0}")]
    NotAllowed(String),
}

/// Minimal base64 decoder (avoids adding a dependency for this).
fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let input = input.trim_end_matches('=');
    let mut buf = Vec::with_capacity(input.len() * 3 / 4);
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;

    for c in input.bytes() {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'\n' | b'\r' | b' ' => continue,
            _ => return Err("invalid base64 character"),
        };
        accum = (accum << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((accum >> bits) as u8);
            accum &= (1 << bits) - 1;
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use termlink_protocol::control::KeyEntry;

    #[test]
    fn validate_rejects_empty_command() {
        assert!(matches!(
            validate_command(""),
            Err(ExecError::Validation(_))
        ));
    }

    // T-2537: the canonical exec response MUST carry `truncated` so every
    // boundary (RPC/CLI/MCP) that serializes an ExecResult can disambiguate a
    // T-2529 cap-hit from a signal kill. Load-bearing: deleting the
    // `"truncated"` line from `ExecResult::to_json` makes this fail.
    #[test]
    fn exec_response_json_includes_truncated() {
        let capped = ExecResult {
            exit_code: -1,
            stdout: "partial".to_string(),
            stderr: String::new(),
            truncated: true,
        };
        let v = capped.to_json();
        assert_eq!(
            v.get("truncated"),
            Some(&serde_json::Value::Bool(true)),
            "capped exec must surface truncated=true (T-2537)"
        );
        assert_eq!(v["exit_code"], serde_json::json!(-1));

        let clean = ExecResult {
            exit_code: 0,
            stdout: "ok".to_string(),
            stderr: String::new(),
            truncated: false,
        };
        assert_eq!(
            clean.to_json().get("truncated"),
            Some(&serde_json::Value::Bool(false)),
            "clean exec must surface truncated=false, not omit it (T-2537)"
        );
    }

    #[test]
    fn validate_rejects_null_bytes() {
        assert!(matches!(
            validate_command("echo hello\0world"),
            Err(ExecError::Validation(_))
        ));
    }

    #[test]
    fn validate_rejects_oversized_command() {
        let long_cmd = "x".repeat(MAX_COMMAND_LEN + 1);
        assert!(matches!(
            validate_command(&long_cmd),
            Err(ExecError::Validation(_))
        ));
    }

    #[test]
    fn validate_accepts_normal_commands() {
        assert!(validate_command("echo hello").is_ok());
        assert!(validate_command("ls -la | grep foo").is_ok());
        assert!(validate_command("cat file.txt && echo done").is_ok());
    }

    #[tokio::test]
    async fn execute_echo() {
        let result = execute("echo hello", None, None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello");
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn execute_with_cwd() {
        let result = execute("pwd", Some("/tmp"), None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        // macOS resolves /tmp to /private/tmp
        assert!(
            result.stdout.trim() == "/tmp" || result.stdout.trim() == "/private/tmp"
        );
    }

    #[tokio::test]
    async fn execute_with_env() {
        let mut env = std::collections::HashMap::new();
        env.insert("MY_VAR".into(), "my_value".into());
        let result = execute("echo $MY_VAR", None, Some(&env), None, None)
            .await
            .unwrap();
        assert_eq!(result.stdout.trim(), "my_value");
    }

    #[tokio::test]
    async fn execute_captures_stderr() {
        let result = execute("echo err >&2", None, None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stderr.trim(), "err");
    }

    #[tokio::test]
    async fn execute_nonzero_exit() {
        let result = execute("exit 42", None, None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[tokio::test]
    async fn execute_timeout() {
        let result = execute(
            "sleep 10",
            None,
            None,
            Some(Duration::from_millis(100)),
            None,
        )
        .await;
        assert!(matches!(result, Err(ExecError::Timeout(_))));
    }

    #[tokio::test]
    async fn execute_timeout_kills_child() {
        // Regression (T-2509): a timed-out command must have its child KILLED, not
        // left orphaned to run to completion. The command sleeps, then would create
        // a marker file. If the child survives the timeout, the marker appears.
        let dir = std::env::temp_dir();
        let marker = dir.join(format!("tl-t2509-{}.marker", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        let cmd = format!("sleep 1 && touch {}", marker.display());

        let result = execute(&cmd, None, None, Some(Duration::from_millis(150)), None).await;
        assert!(matches!(result, Err(ExecError::Timeout(_))));

        // Wait well past the child's natural completion (1s sleep). If kill_on_drop
        // did NOT fire, the orphaned child would create the marker within this window.
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let leaked = marker.exists();
        let _ = std::fs::remove_file(&marker);
        assert!(
            !leaked,
            "timed-out child was not killed — it survived and created the marker (kill_on_drop missing)"
        );
    }

    // ── T-2529: captured output must be bounded so a huge-output command cannot
    // OOM the long-lived daemon. `Command::output()` (the old impl) had no cap. ──

    #[tokio::test]
    async fn execute_truncates_over_cap() {
        // A BOUNDED producer that still exceeds a tiny injected cap: 500 KB of NUL.
        // Bounded on purpose so the load-bearing revert proof (cap check disabled →
        // full read) buffers 500 KB, not gigabytes. Fixed impl: truncated at the first
        // chunk. Reverted impl: reads all 500 KB, truncated=false → this test FAILS.
        let cap = 256usize;
        let res = execute_capped(
            "head -c 500000 /dev/zero",
            None,
            None,
            Some(Duration::from_secs(10)),
            None,
            cap,
        )
        .await
        .expect("exec ok");
        assert!(
            res.truncated,
            "output exceeding the cap must be flagged truncated (T-2529)"
        );
        assert!(
            res.stdout.len() <= cap,
            "captured stdout must be bounded to the cap ({cap}); got {}",
            res.stdout.len()
        );
    }

    #[tokio::test]
    async fn execute_no_truncate_under_cap() {
        let res = execute_capped("echo hi", None, None, None, None, 1024)
            .await
            .expect("exec ok");
        assert!(!res.truncated, "small output must not be flagged truncated");
        assert_eq!(res.stdout.trim(), "hi");
    }

    #[tokio::test]
    async fn execute_kills_infinite_producer_promptly() {
        // The real DoS scenario: an UNBOUNDED producer must be killed at the cap and
        // return promptly (not after the 30s timeout, and without OOMing us). The outer
        // 5s bound asserts promptness — the chunked kill fires on the first over-cap
        // chunk. (Not the revert-proof target — a reverted cap would buffer `yes`
        // without limit; the bounded test above is the safe revert target.)
        let res = tokio::time::timeout(
            Duration::from_secs(5),
            execute_capped("yes", None, None, Some(Duration::from_secs(30)), None, 4096),
        )
        .await
        .expect("execute_capped must return promptly for an unbounded producer, not hang")
        .expect("exec ok");
        assert!(res.truncated, "an infinite producer must be flagged truncated");
        assert!(res.stdout.len() <= 4096, "captured output must be capped");
    }

    // ── T-2530: a caller-supplied exec timeout must be clamped to a sane upper bound.
    // An unbounded timeout defeats the timeout (a low-output long-runner is never
    // reclaimed) and `Duration::from_secs(u64::MAX)` risks a tokio overflow panic. ──

    /// Serializes the few tests that mutate `TERMLINK_MAX_EXEC_TIMEOUT_SECS`.
    static EXEC_TIMEOUT_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn effective_exec_timeout_clamps_huge_value() {
        let _g = EXEC_TIMEOUT_ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("TERMLINK_MAX_EXEC_TIMEOUT_SECS") };
        // The load-bearing assertion: an astronomically large caller value is clamped
        // to the default max (3600s), NOT applied verbatim. Temp-reverting the `.min()`
        // in `effective_exec_timeout` makes this fail (returns u64::MAX seconds).
        assert_eq!(
            effective_exec_timeout(Some(Duration::from_secs(u64::MAX))),
            Duration::from_secs(MAX_EXEC_TIMEOUT_SECS),
            "a huge caller timeout must be clamped to the max, not applied verbatim"
        );
    }

    #[test]
    fn effective_exec_timeout_default_and_passthrough() {
        let _g = EXEC_TIMEOUT_ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("TERMLINK_MAX_EXEC_TIMEOUT_SECS") };
        // Absent → default; a sub-max value passes through unchanged.
        assert_eq!(
            effective_exec_timeout(None),
            Duration::from_secs(DEFAULT_EXEC_TIMEOUT_SECS)
        );
        assert_eq!(
            effective_exec_timeout(Some(Duration::from_secs(5))),
            Duration::from_secs(5)
        );
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)] // env must stay set across the exec await; lock serializes it
    async fn exec_huge_timeout_is_reclaimed_not_hung() {
        // Behavioral proof that the clamp neutralizes the infinite-timeout DoS AND
        // does not panic: with the max clamped tiny (1s) via env, a command that
        // requests `u64::MAX` seconds and would otherwise hang forever is reclaimed by
        // the clamped deadline. The outer 6s bound asserts it returns; the inner result
        // must be `Err(Timeout)`. Without the clamp, `execute_capped` would await
        // `sleep 30` (or panic on the u64::MAX Instant addition) and blow the 6s bound.
        let _g = EXEC_TIMEOUT_ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("TERMLINK_MAX_EXEC_TIMEOUT_SECS", "1") };
        let res = tokio::time::timeout(
            Duration::from_secs(6),
            execute("sleep 30", None, None, Some(Duration::from_secs(u64::MAX)), None),
        )
        .await;
        unsafe { std::env::remove_var("TERMLINK_MAX_EXEC_TIMEOUT_SECS") };
        let inner = res.expect("clamped exec must return well within 6s, not hang");
        assert!(
            matches!(inner, Err(ExecError::Timeout(_))),
            "a huge caller timeout must be clamped and time out, got {inner:?}"
        );
    }

    #[test]
    fn resolve_known_keys() {
        assert_eq!(resolve_key("Enter"), Some(vec![0x0D]));
        assert_eq!(resolve_key("Ctrl+C"), Some(vec![0x03]));
        assert_eq!(resolve_key("Up"), Some(vec![0x1B, 0x5B, 0x41]));
        assert_eq!(resolve_key("Tab"), Some(vec![0x09]));
        assert!(resolve_key("UnknownKey").is_none());
    }

    #[test]
    fn resolve_key_entries() {
        let entries = vec![
            KeyEntry::Text("ls -la".into()),
            KeyEntry::Key("Enter".into()),
        ];
        let bytes = resolve_keys(&entries).unwrap();
        assert_eq!(&bytes[..6], b"ls -la");
        assert_eq!(bytes[6], 0x0D); // Enter
    }

    #[test]
    fn resolve_raw_base64() {
        // 0x03 (Ctrl+C) = "Aw==" in base64
        let entry = KeyEntry::Raw("Aw==".into());
        let bytes = resolve_key_entry(&entry).unwrap();
        assert_eq!(bytes, vec![0x03]);
    }

    #[test]
    fn base64_decode_basic() {
        assert_eq!(base64_decode("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(base64_decode("Aw==").unwrap(), vec![0x03]);
    }

    #[test]
    fn send_signal_to_self() {
        // Signal 0 checks process existence without actually sending a signal
        send_signal(std::process::id(), 0).unwrap();
    }

    #[test]
    fn send_signal_to_nonexistent() {
        let result = send_signal(4_000_000, 0);
        assert!(result.is_err());
    }

    // --- Allowlist tests ---

    #[test]
    fn allowlist_none_allows_everything() {
        assert!(validate_allowlist("rm -rf /", None).is_ok());
    }

    #[test]
    fn allowlist_empty_blocks_everything() {
        let empty: Vec<String> = vec![];
        assert!(matches!(
            validate_allowlist("echo hello", Some(&empty)),
            Err(ExecError::NotAllowed(_))
        ));
    }

    #[test]
    fn allowlist_matching_prefix_allows() {
        let allowed = vec!["echo".into(), "ls".into()];
        assert!(validate_allowlist("echo hello", Some(&allowed)).is_ok());
        assert!(validate_allowlist("ls -la", Some(&allowed)).is_ok());
    }

    #[test]
    fn allowlist_non_matching_prefix_blocks() {
        let allowed = vec!["echo".into(), "ls".into()];
        assert!(matches!(
            validate_allowlist("rm -rf /", Some(&allowed)),
            Err(ExecError::NotAllowed(_))
        ));
    }

    #[test]
    fn allowlist_trims_leading_whitespace() {
        let allowed = vec!["echo".into()];
        assert!(validate_allowlist("  echo hello", Some(&allowed)).is_ok());
    }

    #[tokio::test]
    async fn execute_with_allowlist_allows_matching() {
        let allowed = vec!["echo".into()];
        let result = execute("echo allowed", None, None, None, Some(&allowed))
            .await
            .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "allowed");
    }

    #[tokio::test]
    async fn execute_with_allowlist_blocks_non_matching() {
        let allowed = vec!["echo".into()];
        let result = execute("rm -rf /", None, None, None, Some(&allowed)).await;
        assert!(matches!(result, Err(ExecError::NotAllowed(_))));
    }
}
