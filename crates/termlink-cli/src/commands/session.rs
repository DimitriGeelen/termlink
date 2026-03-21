use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;

use termlink_session::client;
use termlink_session::data_server;
use termlink_session::handler::SessionContext;
use termlink_session::manager;
use termlink_session::pty::PtySession;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

use crate::util::{parse_signal, truncate};

pub(crate) async fn cmd_register(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    shell: bool,
    enable_token_secret: bool,
    allowed_commands: Vec<String>,
) -> Result<()> {
    let mut config = SessionConfig {
        display_name: name,
        roles,
        tags,
        ..Default::default()
    };

    // Add data_plane capability when shell mode is enabled
    if shell {
        config.capabilities.push("data_plane".into());
        config.capabilities.push("stream".into());
    }

    let mut session = termlink_session::Session::register(config)
        .await
        .context("Failed to register session")?;

    // Enable token-based auth if requested
    if enable_token_secret {
        let secret = termlink_session::auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
        session.registration.token_secret = Some(secret_hex.clone());
        println!("Token auth enabled. Secret: {secret_hex}");
        println!("  Create tokens with: termlink token create {} --scope observe", session.id());
    }

    // Set command allowlist if specified
    if !allowed_commands.is_empty() {
        session.registration.allowed_commands = Some(allowed_commands.clone());
        println!("Command allowlist: {:?}", allowed_commands);
    }

    println!("Session registered:");
    println!("  ID:      {}", session.id());
    println!("  Name:    {}", session.display_name());
    println!("  Socket:  {}", session.registration.socket_path().display());

    // Set up session context (with or without PTY)
    let pty_session = if shell {
        // Set data_socket metadata for discoverability
        let data_path = data_server::data_socket_path(session.registration.socket_path());
        session.registration.metadata.data_socket =
            Some(data_path.to_string_lossy().into_owned());

        let pty = PtySession::spawn(None, 1024 * 1024)
            .context("Failed to spawn PTY session")?;
        println!("  PTY:     yes (shell: {})",
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()));
        Some(Arc::new(pty))
    } else {
        println!("  PTY:     no (use --shell for bidirectional I/O)");
        None
    };

    // Persist updated registration (capabilities + metadata + auth + allowlist)
    if shell || enable_token_secret || session.registration.allowed_commands.is_some() {
        session.persist_registration()
            .context("Failed to persist updated registration")?;
    }

    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    let session_id = session.id().clone();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );

    let (registration, listener, _) = session.into_parts();
    let ctx = if let Some(ref pty) = pty_session {
        SessionContext::with_pty(registration.clone(), pty.clone())
            .with_registration_path(json_path)
    } else {
        SessionContext::new(registration.clone())
            .with_registration_path(json_path)
    };
    let shared = Arc::new(RwLock::new(ctx));

    let reg_for_cleanup = registration;

    // Compute data socket path before moving reg
    let data_socket_path = if shell {
        Some(data_server::data_socket_path(reg_for_cleanup.socket_path()))
    } else {
        None
    };

    let shared_clone = shared.clone();

    // If PTY, create broadcast channel and run read loop with broadcasting
    let pty_handle = if let Some(ref pty) = pty_session {
        let pty_clone = pty.clone();
        if let Some(ref data_path) = data_socket_path {
            // Shell mode: broadcast PTY output to data plane clients
            let (tx, rx) = tokio::sync::broadcast::channel::<Vec<u8>>(256);
            let data_pty = pty.clone();
            let data_path = data_path.clone();
            println!("  Data:    {}", data_path.display());

            // Start data plane server
            tokio::spawn(async move {
                if let Err(e) = data_server::run(&data_path, data_pty, rx).await {
                    tracing::error!(error = %e, "Data plane server error");
                }
            });

            // PTY read loop with broadcast
            Some(tokio::spawn(async move {
                let _ = pty_clone.read_loop_with_broadcast(Some(tx)).await;
            }))
        } else {
            // No data plane — plain read loop
            Some(tokio::spawn(async move {
                let _ = pty_clone.read_loop().await;
            }))
        }
    } else {
        None
    };

    tokio::select! {
        _ = server::run_accept_loop(listener, shared_clone) => {}
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("Shutting down...");

            // Kill PTY child if running
            if let Some(ref pty) = pty_session {
                let _ = pty.signal(libc::SIGTERM);
            }
            if let Some(h) = pty_handle {
                h.abort();
            }

            // Clean up registration files
            let json_path = termlink_session::Registration::json_path(&sessions_dir, &session_id);
            let _ = std::fs::remove_file(reg_for_cleanup.socket_path());
            let _ = std::fs::remove_file(&json_path);

            // Clean up data socket if present
            if let Some(ref data_path) = data_socket_path {
                let _ = std::fs::remove_file(data_path);
            }

            println!("Session {} deregistered.", session_id);
        }
    }

    Ok(())
}

pub(crate) fn cmd_list(include_stale: bool, json: bool) -> Result<()> {
    let sessions = manager::list_sessions(include_stale)
        .context("Failed to list sessions")?;

    if json {
        let items: Vec<serde_json::Value> = sessions.iter().map(|s| {
            serde_json::json!({
                "id": s.id.as_str(),
                "display_name": s.display_name,
                "state": s.state.to_string(),
                "pid": s.pid,
                "tags": s.tags,
                "roles": s.roles,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if sessions.is_empty() {
        println!("No active sessions.");
        return Ok(());
    }

    println!(
        "{:<14} {:<16} {:<14} {:<8} TAGS",
        "ID", "NAME", "STATE", "PID"
    );
    println!("{}", "-".repeat(64));

    for session in &sessions {
        let tags = if session.tags.is_empty() {
            String::new()
        } else {
            session.tags.join(",")
        };
        println!(
            "{:<14} {:<16} {:<14} {:<8} {}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            session.pid,
            tags,
        );
    }

    println!();
    println!("{} session(s)", sessions.len());
    Ok(())
}

pub(crate) fn cmd_clean(dry_run: bool) -> Result<()> {
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let stale = manager::clean_stale_sessions(&sessions_dir, !dry_run)
        .context("Failed to scan for stale sessions")?;

    if stale.is_empty() {
        println!("No stale sessions found.");
        return Ok(());
    }

    let action = if dry_run { "Would remove" } else { "Removed" };

    println!(
        "{:<14} {:<16} {:<8} CREATED",
        "ID", "NAME", "PID"
    );
    println!("{}", "-".repeat(54));

    for s in &stale {
        println!(
            "{:<14} {:<16} {:<8} {}",
            &s.id[..s.id.len().min(13)],
            truncate(&s.display_name, 15),
            s.pid,
            &s.created_at[..s.created_at.len().min(19)],
        );
    }

    println!();
    println!("{} {} stale session(s).", action, stale.len());
    Ok(())
}

pub(crate) async fn cmd_ping(target: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(reg.socket_path(), "termlink.ping", serde_json::json!({}))
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            println!(
                "PONG from {} ({}) — state: {}",
                result["id"].as_str().unwrap_or("?"),
                result["display_name"].as_str().unwrap_or("?"),
                result["state"].as_str().unwrap_or("?"),
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Ping failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_status(target: &str, json: bool) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({}))
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }
            println!("Session: {}", result["id"].as_str().unwrap_or("?"));
            println!("  Name:        {}", result["display_name"].as_str().unwrap_or("?"));
            println!("  State:       {}", result["state"].as_str().unwrap_or("?"));
            println!("  PID:         {}", result["pid"]);
            println!("  Created:     {}", result["created_at"].as_str().unwrap_or("?"));
            println!("  Heartbeat:   {}", result["heartbeat_at"].as_str().unwrap_or("?"));
            if let Some(caps) = result.get("capabilities").and_then(|c| c.as_array()) {
                let cap_strs: Vec<&str> = caps.iter().filter_map(|c| c.as_str()).collect();
                println!("  Capabilities: {}", cap_strs.join(", "));
            }
            if let Some(tags) = result.get("tags").and_then(|t| t.as_array())
                && !tags.is_empty() {
                    let tag_strs: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                    println!("  Tags:        {}", tag_strs.join(", "));
                }
            if let Some(roles) = result.get("roles").and_then(|r| r.as_array())
                && !roles.is_empty() {
                    let role_strs: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
                    println!("  Roles:       {}", role_strs.join(", "));
                }
            if let Some(mode) = result.get("terminal_mode") {
                let canonical = mode["canonical"].as_bool().unwrap_or(false);
                let echo = mode["echo"].as_bool().unwrap_or(false);
                let raw = mode["raw"].as_bool().unwrap_or(false);
                let alt_screen = mode["alternate_screen"].as_bool().unwrap_or(false);
                let mode_label = if raw {
                    "raw"
                } else if canonical && echo {
                    "canonical+echo"
                } else if canonical {
                    "canonical"
                } else {
                    "cooked"
                };
                print!("  Term Mode:   {}", mode_label);
                if alt_screen {
                    print!(" (alternate screen)");
                }
                println!();
            }
            if let Some(meta) = result.get("metadata") {
                if let Some(shell) = meta.get("shell").and_then(|s| s.as_str()) {
                    println!("  Shell:       {}", shell);
                }
                if let Some(term) = meta.get("term").and_then(|s| s.as_str()) {
                    println!("  Terminal:    {}", term);
                }
                if let Some(cwd) = meta.get("cwd").and_then(|s| s.as_str()) {
                    println!("  CWD:         {}", cwd);
                }
                if let Some(ds) = meta.get("data_socket").and_then(|s| s.as_str()) {
                    println!("  Data plane:  {}", ds);
                }
            }
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Status query failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_exec(target: &str, command: &str, cwd: Option<&str>, timeout: u64) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = serde_json::json!({
        "command": command,
        "timeout": timeout,
    });
    if let Some(dir) = cwd {
        params["cwd"] = serde_json::json!(dir);
    }

    let resp = client::rpc_call(reg.socket_path(), "command.execute", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
            let stdout = result["stdout"].as_str().unwrap_or("");
            let stderr = result["stderr"].as_str().unwrap_or("");

            if !stdout.is_empty() {
                print!("{stdout}");
            }
            if !stderr.is_empty() {
                eprint!("{stderr}");
            }

            if exit_code != 0 {
                std::process::exit(exit_code as i32);
            }
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Execution failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_send(target: &str, method: &str, params_str: &str) -> Result<()> {
    let params: serde_json::Value =
        serde_json::from_str(params_str).context("Invalid JSON params")?;

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(reg.socket_path(), method, params)
        .await
        .context("Failed to connect to session")?;

    match resp {
        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
            println!("{}", serde_json::to_string_pretty(&r.result)?);
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
            eprintln!("Error {}: {}", e.error.code, e.error.message);
            if let Some(data) = &e.error.data {
                eprintln!("{}", serde_json::to_string_pretty(data)?);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

pub(crate) async fn cmd_signal(target: &str, signal: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let sig_num = parse_signal(signal)
        .context(format!("Unknown signal: '{}'. Use TERM, INT, KILL, HUP, USR1, USR2, or a number.", signal))?;

    let resp = client::rpc_call(
        reg.socket_path(),
        "command.signal",
        serde_json::json!({ "signal": sig_num }),
    )
    .await
    .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            println!(
                "Signal {} sent to PID {}",
                result["signal"].as_i64().unwrap_or(sig_num as i64),
                result["pid"].as_u64().unwrap_or(0),
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Signal failed: {}", e);
        }
    }
}

pub(crate) fn cmd_info(json: bool) -> Result<()> {
    let runtime_dir = termlink_session::discovery::runtime_dir();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let hub_socket = termlink_hub::server::hub_socket_path();
    let hub_running = hub_socket.exists();
    let live = manager::list_sessions(false)
        .map(|s| s.len())
        .unwrap_or(0);
    let all = manager::list_sessions(true)
        .map(|s| s.len())
        .unwrap_or(0);
    let stale = all - live;

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "runtime_dir": runtime_dir.to_string_lossy(),
            "sessions_dir": sessions_dir.to_string_lossy(),
            "hub_socket": hub_socket.to_string_lossy(),
            "hub_running": hub_running,
            "sessions": {
                "live": live,
                "stale": stale,
                "total": all,
            },
        }))?);
        return Ok(());
    }

    println!("TermLink Runtime");
    println!("{}", "-".repeat(40));
    println!("  Runtime dir:  {}", runtime_dir.display());
    println!("  Sessions dir: {}", sessions_dir.display());
    println!("  Hub socket:   {}", hub_socket.display());
    println!(
        "  Hub:          {}",
        if hub_running { "running" } else { "stopped" }
    );

    println!();
    println!("Sessions");
    println!("{}", "-".repeat(40));
    println!("  Live:   {}", live);
    println!("  Stale:  {}", stale);
    println!("  Total:  {}", all);

    if stale > 0 {
        println!();
        println!("  Tip: run 'termlink clean' to remove stale sessions");
    }

    Ok(())
}
