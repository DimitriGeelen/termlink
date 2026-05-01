use anyhow::{Context, Result};
use base64::Engine;

use termlink_protocol::jsonrpc::RpcResponse;
use termlink_session::client;

use crate::commands::remote::{connect_remote_hub, RemoteConn};

const INBOX_DIR: &str = "/tmp/termlink-inbox";

/// Push a file or message to a remote session's inbox with PTY notification.
///
/// This is an atomic operation: write file via `command.execute`, then inject
/// a one-line PTY notification so the target agent sees it immediately.
pub(crate) async fn cmd_push(
    conn: &RemoteConn<'_>,
    session: &str,
    file: Option<&str>,
    message: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    super::print_deprecation_warning("remote push", "channel post");
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_push_inner(conn, session, file, message, json)).await {
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
    let inbox_path = format!("{INBOX_DIR}/{filename}");

    // Connect once, reuse for both RPCs
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    // Step 1: Create inbox dir + write file via command.execute
    // Use base64 encoding for safe transport — avoids heredoc delimiter injection
    let b64 = base64::engine::general_purpose::STANDARD.encode(&content);
    let write_cmd = format!(
        "mkdir -p {INBOX_DIR} && echo '{}' | base64 -d > {}",
        b64,
        shell_escape(&inbox_path),
    );

    // SECURITY (G-048): the write_cmd contains the base64 payload literal.
    // If the remote shell or its allowlist echoes the command back in stderr/
    // stdout/error.message, bubbling that up would leak the payload. Pass
    // `redact_secrets` so exec_rpc strips any echoed payload bytes before
    // surfacing an error.
    if let Err(e) = exec_rpc(&mut rpc_client, session, &write_cmd, &[&b64]).await {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to deliver file to target inbox: {e}")}));
        }
        return Err(e).context("Failed to deliver file to target inbox");
    }

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
    redact_secrets: &[&str],
) -> Result<String> {
    let params = serde_json::json!({
        "target": session,
        "command": command,
        "timeout": 30,
    });

    match client.call("command.execute", serde_json::json!("exec"), params).await {
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

    match client.call("command.inject", serde_json::json!("inject"), params).await {
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
