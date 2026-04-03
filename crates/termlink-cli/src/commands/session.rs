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

/// Options for session registration.
pub(crate) struct RegisterOpts {
    pub name: Option<String>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
    pub cap: Vec<String>,
    pub shell: bool,
    pub enable_token_secret: bool,
    pub allowed_commands: Vec<String>,
    pub json: bool,
    pub quiet: bool,
}

pub(crate) async fn cmd_register(opts: RegisterOpts) -> Result<()> {
    let RegisterOpts { name, roles, tags, cap, shell, enable_token_secret, allowed_commands, json, quiet } = opts;
    let verbose = !json && !quiet;
    let mut config = SessionConfig {
        display_name: name,
        roles,
        tags,
        capabilities: cap,
    };

    // Add data_plane capability when shell mode is enabled
    if shell {
        if !config.capabilities.contains(&"data_plane".to_string()) {
            config.capabilities.push("data_plane".into());
        }
        if !config.capabilities.contains(&"stream".to_string()) {
            config.capabilities.push("stream".into());
        }
    }

    let mut session = match termlink_session::Session::register(config).await {
        Ok(s) => s,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to register session: {}", e)}));
            }
            return Err(e).context("Failed to register session");
        }
    };

    // Enable token-based auth if requested
    let token_secret_hex = if enable_token_secret {
        let secret = termlink_session::auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
        session.registration.token_secret = Some(secret_hex.clone());
        if verbose {
            println!("Token auth enabled. Secret: {secret_hex}");
            println!("  Create tokens with: termlink token create {} --scope observe", session.id());
        }
        Some(secret_hex)
    } else {
        None
    };

    // Set command allowlist if specified
    if !allowed_commands.is_empty() {
        session.registration.allowed_commands = Some(allowed_commands.clone());
        if verbose {
            println!("Command allowlist: {:?}", allowed_commands);
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "id": session.id(),
            "display_name": session.display_name(),
            "socket_path": session.registration.socket_path().display().to_string(),
            "pid": std::process::id(),
            "shell": shell,
            "token_secret": token_secret_hex,
        }));
    } else if verbose {
        println!("Session registered:");
        println!("  ID:      {}", session.id());
        println!("  Name:    {}", session.display_name());
        println!("  Socket:  {}", session.registration.socket_path().display());
    }

    // Set up session context (with or without PTY)
    let pty_session = if shell {
        // Set data_socket metadata for discoverability
        let data_path = data_server::data_socket_path(session.registration.socket_path());
        session.registration.metadata.data_socket =
            Some(data_path.to_string_lossy().into_owned());

        let pty = match PtySession::spawn(None, 1024 * 1024) {
            Ok(p) => p,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to spawn PTY session: {}", e)}));
                }
                return Err(e).context("Failed to spawn PTY session");
            }
        };
        if verbose {
            println!("  PTY:     yes (shell: {})",
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()));
        }
        Some(Arc::new(pty))
    } else {
        if verbose {
            println!("  PTY:     no (use --shell for bidirectional I/O)");
        }
        None
    };

    // Persist updated registration (capabilities + metadata + auth + allowlist)
    if (shell || enable_token_secret || session.registration.allowed_commands.is_some())
        && let Err(e) = session.persist_registration()
    {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to persist updated registration: {}", e)}));
        }
        return Err(e).context("Failed to persist updated registration");
    }

    if verbose {
        println!();
        println!("Listening for connections... (Ctrl+C to stop)");
    }

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

pub(crate) async fn cmd_register_self(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    cap: Vec<String>,
    json: bool,
) -> Result<()> {
    let config = SessionConfig {
        display_name: name,
        roles,
        tags,
        capabilities: cap,
    };

    let endpoint = match termlink_session::endpoint::Endpoint::start(config).await {
        Ok(e) => e,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to register endpoint: {}", e)}));
            }
            return Err(e).context("Failed to register endpoint");
        }
    };

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "id": endpoint.id(),
            "display_name": endpoint.registration().display_name,
            "socket_path": endpoint.socket_path().display().to_string(),
            "pid": std::process::id(),
            "mode": "self",
        }));
    } else {
        println!("Endpoint registered (event-only, no PTY):");
        println!("  ID:      {}", endpoint.id());
        println!("  Socket:  {}", endpoint.socket_path().display());
        println!("  Capabilities: events, kv, status");
        println!();
        println!("Listening... (Ctrl+C to stop)");
    }

    endpoint.run_until_shutdown().await;

    if !json {
        println!("Endpoint deregistered.");
    }
    Ok(())
}

pub(crate) async fn cmd_list(include_stale: bool, display: &super::ListDisplayOpts, tag_filter: Option<&str>, name_filter: Option<&str>, role_filter: Option<&str>, cap_filter: Option<&str>, wait: bool, wait_timeout: u64) -> Result<()> {
    let do_filter = |include_stale: bool| -> Result<Vec<termlink_session::registration::Registration>> {
        let mut sessions = manager::list_sessions(include_stale)
            .context("Failed to list sessions")?;
        if let Some(tag) = tag_filter {
            sessions.retain(|s| s.tags.iter().any(|t| t == tag));
        }
        if let Some(name) = name_filter {
            let name_lower = name.to_lowercase();
            sessions.retain(|s| s.display_name.to_lowercase().contains(&name_lower));
        }
        if let Some(role) = role_filter {
            sessions.retain(|s| s.roles.iter().any(|r| r == role));
        }
        if let Some(cap) = cap_filter {
            sessions.retain(|s| s.capabilities.iter().any(|c| c == cap));
        }
        Ok(sessions)
    };

    let sessions = if wait {
        let start = std::time::Instant::now();
        let timeout_dur = std::time::Duration::from_secs(wait_timeout);
        loop {
            let result = match do_filter(include_stale) {
                Ok(r) => r,
                Err(e) => {
                    if display.json {
                        super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to list sessions: {}", e)}));
                    }
                    return Err(e);
                }
            };
            if !result.is_empty() {
                break result;
            }
            if start.elapsed() > timeout_dur {
                if display.json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("No matching sessions found within {}s", wait_timeout)}));
                }
                anyhow::bail!("No matching sessions found within {}s", wait_timeout);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    } else {
        match do_filter(include_stale) {
            Ok(r) => r,
            Err(e) => {
                if display.json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to list sessions: {}", e)}));
                }
                return Err(e);
            }
        }
    };

    if display.count {
        if display.json {
            println!("{}", serde_json::json!({"ok": true, "count": sessions.len()}));
        } else {
            println!("{}", sessions.len());
        }
        return Ok(());
    }

    if display.first {
        if let Some(s) = sessions.first() {
            if display.json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "id": s.id.as_str(),
                    "display_name": s.display_name,
                    "state": s.state.to_string(),
                    "pid": s.pid,
                    "uid": s.uid,
                    "created_at": s.created_at,
                    "heartbeat_at": s.heartbeat_at,
                    "tags": s.tags,
                    "roles": s.roles,
                    "capabilities": s.capabilities,
                    "metadata": s.metadata,
                    "socket_path": s.socket_path().display().to_string(),
                }));
            } else if display.ids {
                println!("{}", s.id.as_str());
            } else {
                println!("{}", s.display_name);
            }
        } else {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": "No matching sessions"}));
            }
            std::process::exit(1);
        }
        return Ok(());
    }

    if display.names {
        if display.json {
            let items: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
            println!("{}", serde_json::json!({"ok": true, "names": items}));
        } else {
            for s in &sessions {
                println!("{}", s.display_name);
            }
        }
        return Ok(());
    }

    if display.ids {
        if display.json {
            let items: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
            println!("{}", serde_json::json!({"ok": true, "ids": items}));
        } else {
            for s in &sessions {
                println!("{}", s.id.as_str());
            }
        }
        return Ok(());
    }

    if display.json {
        let items: Vec<serde_json::Value> = sessions.iter().map(|s| {
            serde_json::json!({
                "id": s.id.as_str(),
                "display_name": s.display_name,
                "state": s.state.to_string(),
                "pid": s.pid,
                "uid": s.uid,
                "created_at": s.created_at,
                "heartbeat_at": s.heartbeat_at,
                "tags": s.tags,
                "roles": s.roles,
                "capabilities": s.capabilities,
                "metadata": s.metadata,
                "socket_path": s.socket_path().display().to_string(),
            })
        }).collect();
        println!("{}", serde_json::json!({"ok": true, "sessions": items}));
        return Ok(());
    }

    if sessions.is_empty() {
        if !display.no_header {
            println!("No active sessions.");
        }
        return Ok(());
    }

    if !display.no_header {
        println!(
            "{:<14} {:<16} {:<14} {:<8} TAGS",
            "ID", "NAME", "STATE", "PID"
        );
        println!("{}", "-".repeat(64));
    }

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

    if !display.no_header {
        println!();
        println!("{} session(s)", sessions.len());
    }
    Ok(())
}

pub(crate) fn cmd_clean(dry_run: bool, json: bool, no_header: bool, count: bool) -> Result<()> {
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let stale = match manager::clean_stale_sessions(&sessions_dir, !dry_run) {
        Ok(s) => s,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to scan for stale sessions: {}", e)}));
            }
            return Err(e).context("Failed to scan for stale sessions");
        }
    };

    if json {
        let items: Vec<serde_json::Value> = stale.iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "display_name": s.display_name,
                "pid": s.pid,
                "created_at": s.created_at,
            })
        }).collect();
        println!("{}", serde_json::json!({
            "ok": true,
            "dry_run": dry_run,
            "action": if dry_run { "would_remove" } else { "removed" },
            "count": stale.len(),
            "sessions": items,
        }));
        return Ok(());
    }

    if count {
        println!("{}", stale.len());
        return Ok(());
    }

    if stale.is_empty() {
        println!("No stale sessions found.");
        return Ok(());
    }

    let action = if dry_run { "Would remove" } else { "Removed" };

    if !no_header {
        println!(
            "{:<14} {:<16} {:<8} CREATED",
            "ID", "NAME", "PID"
        );
        println!("{}", "-".repeat(54));
    }

    for s in &stale {
        println!(
            "{:<14} {:<16} {:<8} {}",
            &s.id[..s.id.len().min(13)],
            truncate(&s.display_name, 15),
            s.pid,
            &s.created_at[..s.created_at.len().min(19)],
        );
    }

    if !no_header {
        println!();
        println!("{} {} stale session(s).", action, stale.len());
    }
    Ok(())
}

pub(crate) async fn cmd_ping(target: &str, json: bool, timeout_secs: u64) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "termlink.ping", serde_json::json!({}));
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            let latency_ms = start.elapsed().as_millis();
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Ping timed out after {}s", timeout_secs),
                    "timeout_ms": timeout_secs * 1000,
                    "latency_ms": latency_ms,
                }));
            }
            anyhow::bail!("Ping timed out after {}s", timeout_secs);
        }
    };
    let latency_ms = start.elapsed().as_millis();

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "target": target,
                    "id": result["id"],
                    "display_name": result["display_name"],
                    "state": result["state"],
                    "latency_ms": latency_ms,
                }));
            } else {
                println!(
                    "PONG from {} ({}) — state: {}, latency: {}ms",
                    result["id"].as_str().unwrap_or("?"),
                    result["display_name"].as_str().unwrap_or("?"),
                    result["state"].as_str().unwrap_or("?"),
                    latency_ms,
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
            }
            anyhow::bail!("Ping failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_status(target: &str, json: bool, short: bool, timeout_secs: u64) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({}));
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Status query timed out after {}s", timeout_secs)}));
            }
            anyhow::bail!("Status query timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                let mut wrapped = serde_json::json!({"ok": true});
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", wrapped);
                return Ok(());
            }
            if short {
                println!("{} {} {}",
                    result["display_name"].as_str().unwrap_or("?"),
                    result["state"].as_str().unwrap_or("?"),
                    result["pid"],
                );
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
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Status query failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_exec(target: &str, command: &str, cwd: Option<&str>, timeout: u64, json: bool) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let mut params = serde_json::json!({
        "command": command,
        "timeout": timeout,
    });
    if let Some(dir) = cwd {
        params["cwd"] = serde_json::json!(dir);
    }

    // RPC timeout = command timeout + 5s buffer for connection/response overhead
    let rpc_timeout = std::time::Duration::from_secs(timeout + 5);
    let rpc_future = client::rpc_call(reg.socket_path(), "command.execute", params);
    let resp = match tokio::time::timeout(rpc_timeout, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Exec RPC timed out after {}s", timeout + 5)}));
            }
            anyhow::bail!("Exec RPC timed out after {}s (command timeout: {}s)", timeout + 5, timeout);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                let exit_code = result["exit_code"].as_i64().unwrap_or(0);
                let mut wrapped = serde_json::json!({"ok": exit_code == 0});
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", wrapped);
                if exit_code != 0 {
                    std::process::exit(exit_code as i32);
                }
                return Ok(());
            }

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
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Execution failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_send(target: &str, method: &str, params_str: &str, json: bool, timeout_secs: u64) -> Result<()> {
    let params: serde_json::Value = match serde_json::from_str(params_str) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Invalid JSON params: {}", e)}));
            }
            return Err(e.into());
        }
    };

    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), method, params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "method": method,
                    "error": {"code": -1, "message": format!("Timed out after {}s", timeout_secs)},
                }));
            }
            anyhow::bail!("RPC call timed out after {}s", timeout_secs);
        }
    };

    match resp {
        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "method": method,
                    "result": r.result,
                }));
            } else {
                println!("{}", serde_json::to_string_pretty(&r.result)?);
            }
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "method": method,
                    "error": {
                        "code": e.error.code,
                        "message": e.error.message,
                        "data": e.error.data,
                    },
                }));
            } else {
                eprintln!("Error {}: {}", e.error.code, e.error.message);
                if let Some(data) = &e.error.data {
                    eprintln!("{}", serde_json::to_string_pretty(data)?);
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

pub(crate) async fn cmd_signal(target: &str, signal: &str, json: bool, timeout_secs: u64) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let sig_num = match parse_signal(signal) {
        Some(n) => n,
        None => {
            let msg = format!("Unknown signal: '{}'. Use TERM, INT, KILL, HUP, USR1, USR2, or a number.", signal);
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
    };

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(
        reg.socket_path(),
        "command.signal",
        serde_json::json!({ "signal": sig_num }),
    );
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Signal timed out after {}s", timeout_secs)}));
            }
            anyhow::bail!("Signal timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "target": target,
                    "signal": result["signal"],
                    "pid": result["pid"],
                }));
            } else {
                println!(
                    "Signal {} sent to PID {}",
                    result["signal"].as_i64().unwrap_or(sig_num as i64),
                    result["pid"].as_u64().unwrap_or(0),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Signal failed: {}", e);
        }
    }
}

pub(crate) fn cmd_info(json: bool, short: bool, check: bool) -> Result<()> {
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

    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let target = option_env!("BUILD_TARGET").unwrap_or("unknown");

    if short {
        let hub_status = if hub_running { "running" } else { "stopped" };
        println!("termlink {version} sessions:{live}/{all} hub:{hub_status}");
        if check && (!hub_running || stale > 0) {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(1);
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "version": version,
            "commit": commit,
            "target": target,
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
        if check && (!hub_running || stale > 0) {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("TermLink Runtime");
    println!("{}", "-".repeat(40));
    println!("  Version:      {version} ({commit}) [{target}]");
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

    if check && (!hub_running || stale > 0) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }

    Ok(())
}
