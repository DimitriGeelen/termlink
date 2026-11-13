use anyhow::{Context, Result};
use base64::Engine;

use termlink_protocol::jsonrpc::RpcResponse;
use termlink_session::client;

use crate::commands::remote::RemoteConn;

// T-3667: the pre-T-3667 default was `/tmp/termlink-inbox` (shared /tmp, default
// umask, so normally 0644 and readable by anyone on the target). It is still accepted
// as an explicit `--dest /tmp/termlink-inbox` so existing scripts that `cat` from there
// keep working (nothing working is switched off implicitly); it gets the same 0700/0600
// treatment and ownership check as any other destination.

/// T-3667: default inbox, expanded by the TARGET's shell — per receiving account.
const DEFAULT_INBOX_DIR_SH: &str = "\"$HOME/.termlink/inbox\"";

/// T-3667: the base64 payload travels in this env var of the `command.execute`
/// request, never inside the command string, so it is not in any process's argv
/// (`ps`). `/proc/<pid>/environ` is readable only by the same uid and root.
const PAYLOAD_ENV: &str = "TERMLINK_PUSH_B64";

/// Kernel per-string limit for an env entry is MAX_ARG_STRLEN = 128 KiB including
/// the `NAME=` prefix and NUL; refuse a little below it with an actionable message
/// instead of a bare E2BIG from the remote spawn.
const MAX_PAYLOAD_B64: usize = 130_000;

/// Build the remote shell command that writes the payload (read from `$PAYLOAD_ENV`)
/// to `<dest>/<filename>` and prints the final path on stdout.
///
/// Guarantees on the target (T-3667): `umask 077`; the directory exists, is not a
/// symlink, is owned by the receiving user and is chmod 0700; the file is removed
/// first (no writing through a planted symlink) and ends 0600. `dest = None` means
/// the per-user default `$HOME/.termlink/inbox`. The payload never appears here.
pub(crate) fn build_write_command(dest: Option<&str>, filename: &str) -> String {
    let dir = match dest {
        Some(d) => shell_escape(d),
        None => DEFAULT_INBOX_DIR_SH.to_string(),
    };
    let fname = shell_escape(filename);
    format!(
        "umask 077; d={dir}; f=\"$d\"/{fname}; \
[ -n \"${{{PAYLOAD_ENV}+x}}\" ] || {{ echo 'termlink push: payload env {PAYLOAD_ENV} not received by target' >&2; exit 3; }}; \
mkdir -p \"$d\" && [ -d \"$d\" ] && [ ! -L \"$d\" ] && [ -O \"$d\" ] || {{ echo \"termlink push: inbox $d is not a directory owned by the receiving user\" >&2; exit 4; }}; \
chmod 700 \"$d\" && rm -f \"$f\" && printf '%s' \"${PAYLOAD_ENV}\" | base64 -d > \"$f\" && chmod 600 \"$f\" && printf '%s\\n' \"$f\""
    )
}

/// Push a file or message to a remote session's inbox with PTY notification.
///
/// This is an atomic operation: write file via `command.execute`, then inject
/// a one-line PTY notification so the target agent sees it immediately.
pub(crate) async fn cmd_push(
    conn: &RemoteConn<'_>,
    session: &str,
    file: Option<&str>,
    message: Option<&str>,
    dest: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    super::print_deprecation_warning("remote push", "channel post");
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_push_inner(conn, session, file, message, dest, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote push", timeout_secs);
        }
    }
}

async fn cmd_push_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    file: Option<&str>,
    message: Option<&str>,
    dest: Option<&str>,
    json: bool,
) -> Result<()> {
    // Validate: need either file or message
    if file.is_none() && message.is_none() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Provide a file path or --message (or both)"}));
        }
        anyhow::bail!("Provide a file path or --message (or both)");
    }

    // Determine content and filename
    let (content, filename) = if let Some(path) = file {
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Cannot read file: {path}: {e}")}));
                }
                return Err(e).context(format!("Cannot read file: {path}"));
            }
        };
        let fname = std::path::Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "push-content.txt".to_string());
        (data, fname)
    } else {
        // --message only
        (message.unwrap_or("").to_string(), "push-message.txt".to_string())
    };

    let content_bytes = content.len();

    // T-3667: base64 for safe transport, carried in the request's env map (not argv).
    let b64 = base64::engine::general_purpose::STANDARD.encode(&content);
    if b64.len() > MAX_PAYLOAD_B64 {
        let msg = format!(
            "Payload too large for remote push ({content_bytes} bytes, {} base64; max {MAX_PAYLOAD_B64}). Use `termlink file send` for large files.",
            b64.len()
        );
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    // Connect once, reuse for both RPCs
    let mut rpc_client = match conn.connect().await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    // Step 1: create the private inbox dir + write the file via command.execute
    let write_cmd = build_write_command(dest, &filename);
    let env = serde_json::json!({ PAYLOAD_ENV: b64 });

    // SECURITY (G-048): the payload is no longer in write_cmd, but keep scrubbing
    // it from any error text in case a remote wrapper ever echoes the environment.
    let inbox_path = match exec_rpc(&mut rpc_client, session, &write_cmd, Some(env), &[&b64]).await {
        Ok(stdout) => stdout
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .unwrap_or_else(|| format!("{}/{filename}", dest.unwrap_or("~/.termlink/inbox"))),
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to deliver file to target inbox: {e}")}));
            }
            return Err(e).context("Failed to deliver file to target inbox");
        }
    };

    // Step 2: Inject PTY notification
    let notification = format!(
        "[TERMLINK] Received: {filename} — cat {inbox_path}"
    );

    if let Err(e) = inject_rpc(&mut rpc_client, session, &notification).await {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to inject PTY notification: {e}")}));
        }
        return Err(e).context("Failed to inject PTY notification");
    }

    // Step 3: Report confirmation
    if json {
        let report = serde_json::json!({
            "ok": true,
            "status": "delivered",
            "hub": conn.hub,
            "session": session,
            "file": file,
            "inbox_path": inbox_path,
            "bytes": content_bytes,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Pushed {filename} ({content_bytes} bytes) → {session} on {}", conn.hub);
        println!("  Inbox: {inbox_path}");
    }

    Ok(())
}

/// Strip any occurrence of `secrets[i]` from `s`, replacing each match with a
/// `<redacted N bytes>` marker. Used on error paths where the remote shell may
/// echo back a command that contained sensitive payload bytes (G-048).
pub(crate) fn redact(s: &str, secrets: &[&str]) -> String {
    let mut out = s.to_string();
    for needle in secrets {
        if needle.len() < 8 {
            // Too short to be a unique payload — skip to avoid false replacements.
            continue;
        }
        if out.contains(needle) {
            let marker = format!("<redacted {} bytes>", needle.len());
            out = out.replace(needle, &marker);
        }
    }
    out
}

/// Execute a command on a remote session, returning stdout.
/// Unlike cmd_remote_exec, this captures the result instead of printing/exiting.
///
/// `redact_secrets` is a list of substrings that must be scrubbed from any
/// error-path message. The happy path never surfaces stderr/stdout, so this
/// only affects bail! sites.
async fn exec_rpc(
    client: &mut client::Client,
    session: &str,
    command: &str,
    env: Option<serde_json::Value>,
    redact_secrets: &[&str],
) -> Result<String> {
    let mut params = serde_json::json!({
        "target": session,
        "command": command,
        "timeout": 30,
    });
    if let Some(env) = env {
        params["env"] = env;
    }

    match client.call("command.execute", serde_json::json!("exec"), termlink_session::founding_log::with_declared_from(params)).await {
        Ok(RpcResponse::Success(r)) => {
            let exit_code = r.result["exit_code"].as_i64().unwrap_or(-1);
            let stdout = r.result["stdout"].as_str().unwrap_or("").to_string();
            let stderr = r.result["stderr"].as_str().unwrap_or("");
            if exit_code != 0 {
                let raw = if stderr.is_empty() { stdout.as_str() } else { stderr };
                let safe = redact(raw, redact_secrets);
                anyhow::bail!("Remote command failed (exit {}): {}", exit_code, safe);
            }
            Ok(stdout)
        }
        Ok(RpcResponse::Error(e)) => {
            if e.error.message.contains("not found") || e.error.message.contains("No route") {
                anyhow::bail!("Session '{}' not found on hub", session);
            }
            let safe = redact(&e.error.message, redact_secrets);
            anyhow::bail!("Remote exec failed: {} {}", e.error.code, safe);
        }
        Err(e) => {
            let safe = redact(&e.to_string(), redact_secrets);
            anyhow::bail!("Remote exec error: {}", safe);
        }
    }
}

/// Inject a one-line text + Enter into a remote session's PTY.
async fn inject_rpc(client: &mut client::Client, session: &str, text: &str) -> Result<()> {
    let params = serde_json::json!({
        "target": session,
        "keys": [
            { "type": "text", "value": text },
            { "type": "key", "value": "Enter" },
        ],
        "inject_delay_ms": 10,
    });

    match client.call("command.inject", serde_json::json!("inject"), termlink_session::founding_log::with_declared_from(params)).await {
        Ok(RpcResponse::Success(_)) => Ok(()),
        Ok(RpcResponse::Error(e)) => {
            anyhow::bail!("Inject failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => anyhow::bail!("Inject error: {}", e),
    }
}

/// Escape a string for use in a shell command.
use termlink_protocol::shell_escape;

#[cfg(test)]
mod tests {
    use super::redact;

    // ── T-3667: private inbox, payload off the command line ──────────────────
    use super::{build_write_command, PAYLOAD_ENV};
    const LEGACY_INBOX_DIR: &str = "/tmp/termlink-inbox";
    use base64::Engine;
    use std::collections::HashMap;
    use std::os::unix::fs::PermissionsExt;
    use termlink_session::executor;

    const SECRET: &str = "very-secret-payload-T3667-do-not-leak-0123456789";

    fn b64(s: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(s)
    }

    fn mode(p: &std::path::Path) -> u32 {
        std::fs::symlink_metadata(p).unwrap().permissions().mode() & 0o777
    }

    /// Run the push command exactly as a target session would (session executor,
    /// env map from the request), with HOME pointed at `home`.
    async fn run(cmd: &str, home: &std::path::Path, payload: Option<&str>, extra: &[(&str, String)]) -> executor::ExecResult {
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), home.display().to_string());
        if let Some(p) = payload {
            env.insert(PAYLOAD_ENV.to_string(), b64(p));
        }
        for (k, v) in extra {
            env.insert(k.to_string(), v.clone());
        }
        executor::execute(cmd, None, Some(&env), None, None).await.expect("executor ran")
    }

    #[test]
    fn command_string_never_contains_payload() {
        // The command string is the argv of `sh -c` on the target: if the payload (or
        // its base64) is in it, every local user sees it in `ps`. Pre-T-3667 it was.
        for dest in [None, Some(LEGACY_INBOX_DIR), Some("/srv/x y")] {
            let cmd = build_write_command(dest, "secret.hex");
            assert!(!cmd.contains(SECRET) && !cmd.contains(&b64(SECRET)), "payload in command: {cmd}");
            assert!(cmd.contains(&format!("\"${PAYLOAD_ENV}\"")), "must read payload from env: {cmd}");
            assert!(cmd.starts_with("umask 077;"), "umask must be explicit: {cmd}");
            assert!(!cmd.contains("/tmp") || dest == Some(LEGACY_INBOX_DIR), "default must not be /tmp: {cmd}");
        }
    }

    #[tokio::test]
    async fn default_dest_is_private_per_user_inbox() {
        let home = tempfile::tempdir().unwrap();
        let r = run(&build_write_command(None, "note.txt"), home.path(), Some(SECRET), &[]).await;
        assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
        let dir = home.path().join(".termlink/inbox");
        let f = dir.join("note.txt");
        assert_eq!(std::fs::read_to_string(&f).unwrap(), SECRET);
        assert_eq!(mode(&dir), 0o700, "inbox dir mode");
        assert_eq!(mode(&f), 0o600, "file mode");
        assert_eq!(r.stdout.trim(), f.display().to_string(), "stdout reports the real path");
    }

    #[tokio::test]
    async fn explicit_dest_is_tightened_even_if_preexisting_open() {
        // Old-style location (as created by pre-T-3667 code: 0755 dir, 0644 file).
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("termlink-inbox");
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        let f = dir.join("old.txt");
        std::fs::write(&f, "stale").unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();

        let cmd = build_write_command(Some(dir.to_str().unwrap()), "old.txt");
        let r = run(&cmd, tmp.path(), Some(SECRET), &[]).await;
        assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
        assert_eq!(std::fs::read_to_string(&f).unwrap(), SECRET);
        assert_eq!(mode(&dir), 0o700);
        assert_eq!(mode(&f), 0o600);
    }

    #[tokio::test]
    async fn symlinked_dest_is_refused_and_target_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("elsewhere");
        std::fs::create_dir(&real).unwrap();
        let link = tmp.path().join("inbox-link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let r = run(&build_write_command(Some(link.to_str().unwrap()), "x.txt"), tmp.path(), Some(SECRET), &[]).await;
        assert_eq!(r.exit_code, 4, "stderr: {}", r.stderr);
        assert!(!real.join("x.txt").exists());
    }

    #[tokio::test]
    async fn missing_payload_env_fails_loudly_not_empty_file() {
        let home = tempfile::tempdir().unwrap();
        let r = run(&build_write_command(None, "n.txt"), home.path(), None, &[]).await;
        assert_eq!(r.exit_code, 3, "stderr: {}", r.stderr);
        assert!(!home.path().join(".termlink/inbox/n.txt").exists());
    }

    #[tokio::test]
    async fn awkward_filename_is_quoted() {
        let home = tempfile::tempdir().unwrap();
        let name = "it's a $(touch pwned) file.txt";
        let r = run(&build_write_command(None, name), home.path(), Some(SECRET), &[]).await;
        assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
        assert_eq!(std::fs::read_to_string(home.path().join(".termlink/inbox").join(name)).unwrap(), SECRET);
        assert!(!home.path().join("pwned").exists() && !std::path::Path::new("pwned").exists());
    }

    /// Process-list check: a `base64` shim on PATH records its own argv and its
    /// parent's (`sh -c <command>`) from /proc while the write is in flight.
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn payload_absent_from_every_process_argv() {
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let log = tmp.path().join("argv.log");
        let real = ["/usr/bin/base64", "/bin/base64"].into_iter().find(|p| std::path::Path::new(p).exists()).expect("base64 present");
        let shim = bin.join("base64");
        std::fs::write(&shim, format!(
            "#!/bin/sh\ntr '\\0' ' ' < /proc/$$/cmdline >> '{l}'; echo >> '{l}'; tr '\\0' ' ' < /proc/$PPID/cmdline >> '{l}'; echo >> '{l}'\nexec {real} \"$@\"\n",
            l = log.display()
        )).unwrap();
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap_or_default());

        let r = run(&build_write_command(None, "s.hex"), tmp.path(), Some(SECRET), &[("PATH", path)]).await;
        assert_eq!(r.exit_code, 0, "stderr: {}", r.stderr);
        let seen = std::fs::read_to_string(&log).expect("shim ran (the test would be vacuous otherwise)");
        assert!(seen.contains("base64 -d") && seen.contains("umask 077"), "shim saw both processes: {seen}");
        assert!(!seen.contains(SECRET) && !seen.contains(&b64(SECRET)), "payload leaked into argv: {seen}");
        assert_eq!(std::fs::read_to_string(tmp.path().join(".termlink/inbox/s.hex")).unwrap(), SECRET);
    }

    #[test]
    fn redact_strips_allowlist_rejection_echo() {
        // Simulate stderr from a session allowlist that echoes the rejected command.
        let payload = "QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE=";
        let stderr = format!(
            "command not allowed: mkdir -p /tmp/termlink-inbox && echo '{payload}' | base64 -d > /tmp/termlink-inbox/secret.hex"
        );
        let safe = redact(&stderr, &[payload]);
        assert!(!safe.contains(payload), "payload must not appear in redacted output");
        assert!(safe.contains("<redacted"), "must include redaction marker");
    }

    #[test]
    fn redact_handles_heredoc_shell_error() {
        let payload = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let stderr = format!("bash: syntax error near 'echo {payload} | base64'");
        let safe = redact(&stderr, &[payload]);
        assert!(!safe.contains(payload));
    }

    #[test]
    fn redact_passthrough_when_no_payload() {
        // Stderr-only failure, no payload echoed — message preserved.
        let stderr = "permission denied: /tmp/termlink-inbox";
        let safe = redact(stderr, &["irrelevant-payload-not-present-here-1234567"]);
        assert_eq!(safe, stderr);
    }

    #[test]
    fn redact_skips_short_needles_to_avoid_false_positives() {
        // Short common substrings (<8 chars) must NOT be replaced.
        let stderr = "exit 1: cat: /tmp/termlink-inbox: No such file or directory";
        let safe = redact(stderr, &["exit 1"]);
        assert_eq!(safe, stderr, "short needle should be skipped");
    }

    #[test]
    fn redact_handles_multiple_secrets() {
        let s1 = "first-secret-payload-bytes-1234567890";
        let s2 = "second-secret-payload-bytes-9876543210";
        let combined = format!("error: rejected '{s1}' and also leaked '{s2}'");
        let safe = redact(&combined, &[s1, s2]);
        assert!(!safe.contains(s1));
        assert!(!safe.contains(s2));
    }
}
