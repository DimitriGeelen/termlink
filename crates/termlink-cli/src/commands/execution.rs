use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;

use termlink_session::client;
use termlink_session::handler::SessionContext;
use termlink_session::manager;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

use crate::cli::SpawnBackend;
use crate::util::shell_escape;

pub(crate) async fn cmd_run(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    timeout: u64,
    json: bool,
    command_parts: Vec<String>,
) -> Result<()> {
    use termlink_session::executor;

    let command_str = command_parts
        .iter()
        .map(|part| {
            if part.contains(' ') || part.contains('"') || part.contains('\'') || part.contains('\\') || part.contains('$') || part.contains('`') {
                format!("'{}'", part.replace('\'', "'\\''"))
            } else {
                part.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let config = SessionConfig {
        display_name: name,
        roles,
        tags,
        ..Default::default()
    };

    let session = match termlink_session::Session::register(config).await {
        Ok(s) => s,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": format!("Failed to register ephemeral session: {}", e)}));
                std::process::exit(1);
            }
            return Err(e).context("Failed to register ephemeral session");
        }
    };

    let session_id = session.id().clone();
    let sessions_dir = termlink_session::discovery::sessions_dir();

    if !json {
        eprintln!("Session {} ({}) registered", session.id(), session.display_name());
        eprintln!("Running: {}", command_str);
    }

    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );
    let (registration, listener, _) = session.into_parts();
    let ctx = SessionContext::new(registration.clone())
        .with_registration_path(json_path);
    let shared = Arc::new(RwLock::new(ctx));
    let shared_clone = shared.clone();

    let reg_for_cleanup = registration;

    // Run RPC listener in background so the session is queryable during execution
    let rpc_handle = tokio::spawn(async move {
        server::run_accept_loop(listener, shared_clone).await;
    });

    // Execute the command (CLI-initiated, no allowlist restriction)
    let start = std::time::Instant::now();
    let result = executor::execute(
        &command_str,
        None,
        None,
        Some(std::time::Duration::from_secs(timeout)),
        None,
    )
    .await;

    // Abort RPC listener
    rpc_handle.abort();

    // Cleanup: deregister session
    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );
    let elapsed_ms = start.elapsed().as_millis();

    let _ = std::fs::remove_file(reg_for_cleanup.socket_path());
    let _ = std::fs::remove_file(&json_path);
    if !json {
        eprintln!("Session {} deregistered", session_id);
    }

    match result {
        Ok(exec_result) => {
            if json {
                println!("{}", serde_json::json!({
                    "exit_code": exec_result.exit_code,
                    "stdout": exec_result.stdout,
                    "stderr": exec_result.stderr,
                    "elapsed_ms": elapsed_ms,
                    "session_id": session_id.as_str(),
                    "command": command_str,
                }));
                if exec_result.exit_code != 0 {
                    std::process::exit(exec_result.exit_code);
                }
            } else {
                if !exec_result.stdout.is_empty() {
                    print!("{}", exec_result.stdout);
                }
                if !exec_result.stderr.is_empty() {
                    eprint!("{}", exec_result.stderr);
                }
                if exec_result.exit_code != 0 {
                    std::process::exit(exec_result.exit_code);
                }
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({
                    "error": e.to_string(),
                    "elapsed_ms": elapsed_ms,
                    "session_id": session_id.as_str(),
                    "command": command_str,
                }));
                std::process::exit(1);
            }
            anyhow::bail!("Command failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_request(
    target: &str,
    topic: &str,
    payload: &str,
    reply_topic: &str,
    timeout: u64,
    interval: u64,
    json: bool,
) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
                std::process::exit(1);
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let request_id = format!("req-{}-{}", std::process::id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());

    let mut payload_json: serde_json::Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": format!("Invalid JSON payload: {}", e)}));
                std::process::exit(1);
            }
            return Err(e.into());
        }
    };
    if let Some(obj) = payload_json.as_object_mut() {
        obj.insert("request_id".to_string(), serde_json::json!(request_id));
    }

    let cursor: Option<u64> = {
        let params = serde_json::json!({});
        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    result["next_seq"].as_u64()
                } else { None }
            }
            Err(_) => None,
        }
    };

    let emit_params = serde_json::json!({
        "topic": topic,
        "payload": payload_json,
    });

    let emit_resp = match client::rpc_call(reg.socket_path(), "event.emit", emit_params).await {
        Ok(r) => r,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to emit request event: {}", e)}));
                std::process::exit(1);
            }
            return Err(e).context("Failed to emit request event");
        }
    };

    match client::unwrap_result(emit_resp) {
        Ok(result) => {
            if !json {
                println!("Request sent: {} (seq: {}, request_id: {})",
                    topic,
                    result["seq"].as_u64().unwrap_or(0),
                    request_id);
            }
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to emit request: {e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Failed to emit request: {}", e);
        }
    }

    if !json {
        println!("Waiting for reply on topic '{}' (timeout: {}s)...", reply_topic, timeout);
    }

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(interval);
    let mut poll_cursor = cursor;

    loop {
        let mut params = serde_json::json!({ "topic": reply_topic });
        if let Some(c) = poll_cursor {
            params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let event_payload = &event["payload"];
                            let matches = event_payload
                                .get("request_id")
                                .and_then(|r| r.as_str())
                                .map(|r| r == request_id)
                                .unwrap_or(true);

                            if matches {
                                if json {
                                    println!("{}", serde_json::json!({
                                        "request_id": request_id,
                                        "topic": reply_topic,
                                        "payload": event_payload,
                                    }));
                                } else {
                                    println!("Reply received:");
                                    println!("{}", serde_json::to_string_pretty(event_payload)?);
                                }
                                return Ok(());
                            }
                        }
                    }

                    if let Some(events) = result["events"].as_array()
                        && !events.is_empty()
                            && let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if start.elapsed() > timeout_dur {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Timeout waiting for reply on topic '{}' ({}s)", reply_topic, timeout)}));
                std::process::exit(1);
            }
            anyhow::bail!("Timeout waiting for reply on topic '{}' ({}s)", reply_topic, timeout);
        }

        tokio::time::sleep(poll_interval).await;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_spawn(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    wait: bool,
    wait_timeout: u64,
    shell: bool,
    backend: SpawnBackend,
    json: bool,
    command: Vec<String>,
) -> Result<()> {
    let session_name = name.clone().unwrap_or_else(|| {
        format!("spawn-{}", std::process::id())
    });

    let shell_cmd = build_spawn_shell_cmd(&session_name, &roles, &tags, shell, &command)?;

    let resolved = resolve_spawn_backend(&backend);
    let spawn_result = match resolved {
        SpawnBackend::Terminal => spawn_via_terminal(&session_name, &shell_cmd),
        SpawnBackend::Tmux => spawn_via_tmux(&session_name, &shell_cmd),
        SpawnBackend::Background => spawn_via_background(&session_name, &shell_cmd),
        SpawnBackend::Auto => unreachable!("resolve_spawn_backend always resolves Auto"),
    };
    if let Err(e) = spawn_result {
        if json {
            println!("{}", serde_json::json!({
                "ok": false,
                "session_name": session_name,
                "backend": resolved.to_string(),
                "error": format!("{e}"),
            }));
            std::process::exit(1);
        }
        return Err(e);
    }

    if !json {
        println!("Spawned session '{}' via {} backend", session_name, resolved);
    }

    if wait {
        if !json {
            println!("Waiting for session to register (timeout: {}s)...", wait_timeout);
        }
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(wait_timeout);

        loop {
            if let Ok(reg) = manager::find_session(&session_name) {
                if json {
                    println!("{}", serde_json::json!({
                        "session_name": session_name,
                        "backend": resolved.to_string(),
                        "ready": true,
                        "session_id": reg.id.as_str(),
                    }));
                } else {
                    println!("Session '{}' is ready", session_name);
                }
                return Ok(());
            }
            if start.elapsed() > timeout {
                if json {
                    println!("{}", serde_json::json!({
                        "session_name": session_name,
                        "backend": resolved.to_string(),
                        "ready": false,
                        "error": format!("Timeout waiting for session to register ({}s)", wait_timeout),
                    }));
                    std::process::exit(1);
                }
                anyhow::bail!(
                    "Timeout waiting for session '{}' to register ({}s)",
                    session_name,
                    wait_timeout
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "session_name": session_name,
            "backend": resolved.to_string(),
        }));
    }

    Ok(())
}

fn build_spawn_shell_cmd(
    session_name: &str,
    roles: &[String],
    tags: &[String],
    shell: bool,
    command: &[String],
) -> Result<String> {
    let termlink_bin = std::env::current_exe()
        .context("Failed to determine termlink binary path")?;
    let termlink_path = termlink_bin.to_string_lossy();

    let mut register_args = vec![
        "register".to_string(),
        "--name".to_string(),
        session_name.to_string(),
    ];
    if !roles.is_empty() {
        register_args.push("--roles".to_string());
        register_args.push(roles.join(","));
    }
    if !tags.is_empty() {
        register_args.push("--tags".to_string());
        register_args.push(tags.join(","));
    }
    if shell || command.is_empty() {
        register_args.push("--shell".to_string());
    }

    let shell_cmd = if command.is_empty() {
        let mut parts = vec![termlink_path.to_string()];
        parts.extend(register_args.iter().cloned());

        if let Ok(rd) = std::env::var("TERMLINK_RUNTIME_DIR") {
            format!("TERMLINK_RUNTIME_DIR={} {}", shell_escape(&rd), parts.join(" "))
        } else {
            parts.join(" ")
        }
    } else {
        let mut reg_parts = vec![termlink_path.to_string()];
        reg_parts.extend(register_args.iter().cloned());

        let user_cmd = command.iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" ");
        let env_prefix = if let Ok(rd) = std::env::var("TERMLINK_RUNTIME_DIR") {
            format!("export TERMLINK_RUNTIME_DIR={}; ", shell_escape(&rd))
        } else {
            String::new()
        };

        format!(
            "{env_prefix}{} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nkill $TL_PID 2>/dev/null\nwait $TL_PID 2>/dev/null",
            reg_parts.join(" ")
        )
    };

    Ok(shell_cmd)
}

fn resolve_spawn_backend(backend: &SpawnBackend) -> SpawnBackend {
    match backend {
        SpawnBackend::Auto => {
            #[cfg(target_os = "macos")]
            {
                if std::process::Command::new("pgrep")
                    .args(["-x", "WindowServer"])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    return SpawnBackend::Terminal;
                }
            }

            if std::process::Command::new("tmux")
                .arg("-V")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return SpawnBackend::Tmux;
            }

            SpawnBackend::Background
        }
        other => other.clone(),
    }
}

fn spawn_via_terminal(session_name: &str, shell_cmd: &str) -> Result<()> {
    let escaped_cmd = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
    let applescript = format!(
        r#"tell application "Terminal"
    activate
    do script "{escaped_cmd}"
end tell"#
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&applescript)
        .status()
        .context("Failed to run osascript — is Terminal.app available?")?;

    if !status.success() {
        anyhow::bail!("Failed to open new Terminal.app window for session '{}'", session_name);
    }
    Ok(())
}

fn spawn_via_tmux(session_name: &str, shell_cmd: &str) -> Result<()> {
    let tmux_session = format!("tl-{}", session_name);
    let status = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", &tmux_session, shell_cmd])
        .status()
        .context("Failed to run tmux — is tmux installed?")?;

    if !status.success() {
        anyhow::bail!("Failed to create tmux session '{}' for TermLink session '{}'", tmux_session, session_name);
    }
    Ok(())
}

fn spawn_via_background(session_name: &str, shell_cmd: &str) -> Result<()> {
    let child = std::process::Command::new("setsid")
        .args(["sh", "-c", shell_cmd])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn()
        .or_else(|_| {
            std::process::Command::new("sh")
                .args(["-c", shell_cmd])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .spawn()
        })
        .context("Failed to spawn background session")?;

    let _ = child;
    let _ = session_name;
    Ok(())
}
