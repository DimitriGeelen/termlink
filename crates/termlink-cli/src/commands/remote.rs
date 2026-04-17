use anyhow::{Context, Result};

use termlink_session::client;

use termlink_protocol::events::{
    file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
};

use crate::cli::{ProfileAction, RemoteInboxAction};
use crate::config::{hubs_config_path, load_hubs_config, save_hubs_config, HubEntry};
use crate::util::{generate_request_id, truncate, DEFAULT_CHUNK_SIZE};

use super::ListDisplayOpts;

/// Options for remote inject command.
pub(crate) struct RemoteInjectOpts<'a> {
    pub session: &'a str,
    pub text: &'a str,
    pub enter: bool,
    pub key: Option<&'a str>,
    pub delay_ms: u64,
    pub json: bool,
    pub timeout_secs: u64,
}

/// Connection parameters for a remote hub.
pub(crate) struct RemoteConn<'a> {
    pub hub: &'a str,
    pub secret_file: Option<&'a str>,
    pub secret_hex: Option<&'a str>,
    pub scope: &'a str,
}

/// Connect to a remote hub via TOFU TLS and authenticate.
/// Returns an authenticated client ready for RPC calls.
pub(crate) async fn connect_remote_hub(
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
) -> Result<client::Client> {
    use termlink_session::auth::{self, PermissionScope};

    // --- Parse hub address ---
    let parts: Vec<&str> = hub.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid hub address '{}'. Expected format: host:port", hub);
    }
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse()
        .context(format!("Invalid port in '{}'", hub))?;

    // --- Read secret ---
    let hex = if let Some(path) = secret_file {
        std::fs::read_to_string(path)
            .context(format!("Secret file not found: {}", path))?
            .trim()
            .to_string()
    } else if let Some(h) = secret_hex {
        h.to_string()
    } else {
        anyhow::bail!("Either --secret-file or --secret is required");
    };

    // --- Parse hex to bytes ---
    if hex.len() != 64 {
        anyhow::bail!("Secret must be 64 hex characters (32 bytes), got {} characters", hex.len());
    }
    let secret_bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Secret contains invalid hex characters")?;
    let secret: auth::TokenSecret = secret_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Secret must be exactly 32 bytes"))?;

    // --- Parse scope ---
    let perm_scope = match scope {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        _ => anyhow::bail!("Invalid scope '{}'. Use: observe, interact, control, execute", scope),
    };

    // --- Generate auth token ---
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    // --- Connect via TOFU TLS ---
    let addr = termlink_protocol::TransportAddr::Tcp { host, port };
    let mut rpc_client = client::Client::connect_addr(&addr)
        .await
        .context(format!("Cannot connect to {} — is the hub running?", hub))?;

    // --- Authenticate ---
    match rpc_client.call("hub.auth", serde_json::json!("auth"), serde_json::json!({"token": token.raw})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Authentication failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Authentication error: {}", e);
        }
    }

    Ok(rpc_client)
}

/// Interactive remote session picker — connects to hub, lists sessions, prompts user.
/// Returns the selected session name/ID.
pub(crate) async fn pick_remote_session(
    conn: &RemoteConn<'_>,
) -> Result<String> {
    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() {
        anyhow::bail!("No session specified and stdin is not a terminal (cannot prompt)");
    }

    let mut rpc_client = connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await?;

    let resp = rpc_client
        .call("session.discover", serde_json::json!("discover"), serde_json::json!({}))
        .await;

    let sessions = match resp {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            r.result["sessions"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Discover failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Discover error: {}", e);
        }
    };

    if sessions.is_empty() {
        anyhow::bail!("No active sessions on {}.", conn.hub);
    }

    if sessions.len() == 1 {
        let name = sessions[0]["display_name"].as_str().unwrap_or("?");
        let id = sessions[0]["id"].as_str().unwrap_or("?");
        eprintln!("Auto-selecting: {} ({})", name, id);
        return Ok(name.to_string());
    }

    eprintln!("Sessions on {}:", conn.hub);
    eprintln!(
        "  {:<4} {:<20} {:<12} {:<10} TAGS",
        "#", "NAME", "STATE", "PID"
    );
    eprintln!("  {}", "-".repeat(60));
    for (i, s) in sessions.iter().enumerate() {
        let name = s["display_name"].as_str().unwrap_or("?");
        let state = s["state"].as_str().unwrap_or("?");
        let pid = s["pid"].as_u64().unwrap_or(0);
        let tags = s["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        eprintln!(
            "  {:<4} {:<20} {:<12} {:<10} {}",
            i + 1,
            truncate(name, 19),
            state,
            pid,
            tags
        );
    }
    eprintln!();
    eprint!("Select session [1-{}]: ", sessions.len());

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read input: {}", e))?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid selection: '{}'", input.trim()))?;

    if choice < 1 || choice > sessions.len() {
        anyhow::bail!(
            "Selection out of range: {} (expected 1-{})",
            choice,
            sessions.len()
        );
    }

    let selected = &sessions[choice - 1];
    let name = selected["display_name"].as_str().unwrap_or("?");
    let id = selected["id"].as_str().unwrap_or("?");
    eprintln!("→ {} ({})", name, id);
    Ok(name.to_string())
}

/// Resolve a remote session target: if provided, return it; if None, prompt interactively.
pub(crate) async fn resolve_remote_target(
    session: Option<String>,
    conn: &RemoteConn<'_>,
) -> Result<String> {
    if let Some(s) = session {
        return Ok(s);
    }
    pick_remote_session(conn).await
}

pub(crate) fn cmd_remote_profile(action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::Add { name, address, secret_file, secret, scope, json } => {
            if !address.contains(':') {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": "Address must be in host:port format (e.g., 192.168.10.107:9100)"}));
                }
                anyhow::bail!("Address must be in host:port format (e.g., 192.168.10.107:9100)");
            }
            let mut config = load_hubs_config();
            let is_update = config.hubs.contains_key(&name);
            config.hubs.insert(name.clone(), HubEntry {
                address: address.clone(),
                secret_file,
                secret,
                scope,
            });
            save_hubs_config(&config)?;
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "action": if is_update { "updated" } else { "added" },
                    "name": name,
                    "address": address,
                    "config": hubs_config_path().display().to_string(),
                }));
            } else {
                if is_update {
                    println!("Updated profile '{}' → {}", name, address);
                } else {
                    println!("Added profile '{}' → {}", name, address);
                }
                println!("  Config: {}", hubs_config_path().display());
            }
            Ok(())
        }
        ProfileAction::List { json, no_header } => {
            let config = load_hubs_config();
            if json {
                let profiles: Vec<serde_json::Value> = {
                    let mut names: Vec<_> = config.hubs.keys().collect();
                    names.sort();
                    names.iter().map(|name| {
                        let entry = &config.hubs[*name];
                        serde_json::json!({
                            "name": name,
                            "address": entry.address,
                            "scope": entry.scope,
                            "secret_type": if entry.secret_file.is_some() { "file" }
                                else if entry.secret.is_some() { "inline" }
                                else { "none" },
                        })
                    }).collect()
                };
                println!("{}", serde_json::json!({"ok": true, "profiles": profiles}));
                return Ok(());
            }
            if config.hubs.is_empty() {
                println!("No hub profiles configured.");
                println!("  Add one: termlink remote profile add <name> <address> --secret-file <path>");
                return Ok(());
            }
            if !no_header {
                println!("{:<12} {:<28} {:<10} SECRET", "NAME", "ADDRESS", "SCOPE");
                println!("{}", "-".repeat(64));
            }
            let mut names: Vec<_> = config.hubs.keys().collect();
            names.sort();
            for name in names {
                let entry = &config.hubs[name];
                let scope = entry.scope.as_deref().unwrap_or("-");
                let secret_info = if entry.secret_file.is_some() {
                    "file"
                } else if entry.secret.is_some() {
                    "inline"
                } else {
                    "none"
                };
                println!("{:<12} {:<28} {:<10} {}", name, entry.address, scope, secret_info);
            }
            if !no_header {
                println!();
                println!("{} profile(s) in {}", config.hubs.len(), hubs_config_path().display());
            }
            Ok(())
        }
        ProfileAction::Remove { name, json } => {
            let mut config = load_hubs_config();
            if config.hubs.remove(&name).is_some() {
                save_hubs_config(&config)?;
                if json {
                    println!("{}", serde_json::json!({"ok": true, "action": "removed", "name": name}));
                } else {
                    println!("Removed profile '{}'", name);
                }
            } else {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Profile '{}' not found", name)}));
                }
                println!("Profile '{}' not found", name);
            }
            Ok(())
        }
    }
}

pub(crate) async fn cmd_remote_ping(
    conn: &RemoteConn<'_>,
    session: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_ping_inner(conn, session, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote ping", timeout_secs);
        }
    }
}

async fn cmd_remote_ping_inner(
    conn: &RemoteConn<'_>,
    session: Option<&str>,
    json: bool,
) -> Result<()> {
    let start = std::time::Instant::now();
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };
    let auth_ms = start.elapsed().as_millis();

    match session {
        Some(target) => {
            let ping_start = std::time::Instant::now();
            let params = serde_json::json!({ "target": target });
            match rpc_client.call("termlink.ping", serde_json::json!("ping"), params).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    let total_ms = start.elapsed().as_millis();
                    let rpc_ms = ping_start.elapsed().as_millis();
                    if json {
                        println!("{}", serde_json::json!({
                            "ok": true,
                            "hub": conn.hub,
                            "session": target,
                            "id": r.result["id"],
                            "display_name": r.result["display_name"],
                            "state": r.result["state"],
                            "total_ms": total_ms as u64,
                            "auth_ms": auth_ms as u64,
                            "rpc_ms": rpc_ms as u64,
                        }));
                    } else {
                        println!(
                            "PONG from {} ({}) on {} — state: {} — {}ms (auth: {}ms, rpc: {}ms)",
                            r.result["id"].as_str().unwrap_or("?"),
                            r.result["display_name"].as_str().unwrap_or("?"),
                            conn.hub,
                            r.result["state"].as_str().unwrap_or("?"),
                            total_ms, auth_ms, rpc_ms,
                        );
                    }
                    Ok(())
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                        format!("Session '{}' not found on {}", target, conn.hub)
                    } else {
                        format!("Ping failed: {} {}", e.error.code, e.error.message)
                    };
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": target, "error": msg}));
                    }
                    anyhow::bail!("{}", msg);
                }
                Err(e) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": target, "error": format!("Ping error: {e}")}));
                    }
                    anyhow::bail!("Ping error: {}", e);
                }
            }
        }
        None => {
            let discover_start = std::time::Instant::now();
            match rpc_client.call("session.discover", serde_json::json!("discover"), serde_json::json!({})).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    let total_ms = start.elapsed().as_millis();
                    let discover_ms = discover_start.elapsed().as_millis();
                    let count = r.result["sessions"].as_array().map(|a| a.len()).unwrap_or(0);
                    if json {
                        println!("{}", serde_json::json!({
                            "ok": true,
                            "hub": conn.hub,
                            "sessions": count,
                            "total_ms": total_ms as u64,
                            "auth_ms": auth_ms as u64,
                            "discover_ms": discover_ms as u64,
                        }));
                    } else {
                        println!(
                            "PONG from hub {} — {} session(s) — {}ms (auth: {}ms, discover: {}ms)",
                            conn.hub, count, total_ms, auth_ms, discover_ms,
                        );
                    }
                    Ok(())
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = format!("Hub ping failed: {} {}", e.error.code, e.error.message);
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": msg}));
                    }
                    anyhow::bail!("{}", msg);
                }
                Err(e) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Hub ping error: {e}")}));
                    }
                    anyhow::bail!("Hub ping error: {}", e);
                }
            }
        }
    }
}

pub(crate) async fn cmd_remote_list(
    conn: &RemoteConn<'_>,
    name: Option<&str>,
    tags: Option<&str>,
    roles: Option<&str>,
    cap: Option<&str>,
    display: &ListDisplayOpts,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_list_inner(conn, name, tags, roles, cap, display)).await {
        Ok(result) => result,
        Err(_) => {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote list", timeout_secs);
        }
    }
}

async fn cmd_remote_list_inner(
    conn: &RemoteConn<'_>,
    name: Option<&str>,
    tags: Option<&str>,
    roles: Option<&str>,
    cap: Option<&str>,
    display: &ListDisplayOpts,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let mut params = serde_json::json!({});
    if let Some(n) = name {
        params["name"] = serde_json::json!(n);
    }
    if let Some(t) = tags {
        let tag_list: Vec<&str> = t.split(',').map(|s| s.trim()).collect();
        params["tags"] = serde_json::json!(tag_list);
    }
    if let Some(r) = roles {
        let role_list: Vec<&str> = r.split(',').map(|s| s.trim()).collect();
        params["roles"] = serde_json::json!(role_list);
    }
    if let Some(c) = cap {
        let cap_list: Vec<&str> = c.split(',').map(|s| s.trim()).collect();
        params["capabilities"] = serde_json::json!(cap_list);
    }

    match rpc_client.call("session.discover", serde_json::json!("discover"), params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let sessions = r.result["sessions"].as_array();
            let sessions = sessions.map(|a| a.as_slice()).unwrap_or(&[]);

            if display.first {
                if let Some(s) = sessions.first() {
                    if display.json {
                        let mut wrapped = serde_json::json!({"ok": true});
                        if let Some(obj) = s.as_object() {
                            for (k, v) in obj {
                                wrapped[k] = v.clone();
                            }
                        }
                        println!("{}", serde_json::to_string_pretty(&wrapped)?);
                    } else {
                        println!("{}", s["display_name"].as_str().unwrap_or("?"));
                    }
                } else {
                    if display.json {
                        super::json_error_exit(serde_json::json!({"ok": false, "error": "No matching sessions"}));
                    }
                    std::process::exit(1);
                }
                return Ok(());
            }

            if display.count {
                if display.json {
                    println!("{}", serde_json::json!({"ok": true, "count": sessions.len()}));
                } else {
                    println!("{}", sessions.len());
                }
                return Ok(());
            }

            if display.names {
                for s in sessions {
                    println!("{}", s["display_name"].as_str().unwrap_or("?"));
                }
                return Ok(());
            }

            if display.ids {
                for s in sessions {
                    println!("{}", s["id"].as_str().unwrap_or("?"));
                }
                return Ok(());
            }

            if display.json {
                println!("{}", serde_json::json!({"ok": true, "sessions": sessions}));
                return Ok(());
            }

            if sessions.is_empty() {
                if !display.no_header {
                    println!("No sessions on {}.", conn.hub);
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

            for s in sessions {
                let id = s["id"].as_str().unwrap_or("?");
                let display_name = s["display_name"].as_str().unwrap_or("?");
                let state = s["state"].as_str().unwrap_or("?");
                let pid = s["pid"].as_u64().unwrap_or(0);
                let tags_arr = s["tags"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
                    .unwrap_or_default();
                println!(
                    "{:<14} {:<16} {:<14} {:<8} {}",
                    truncate(id, 13),
                    truncate(display_name, 15),
                    state,
                    pid,
                    tags_arr,
                );
            }

            if !display.no_header {
                println!();
                println!("{} session(s) on {}", sessions.len(), conn.hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = format!("Discover failed: {} {}", e.error.code, e.error.message);
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Discover error: {e}")}));
            }
            anyhow::bail!("Discover error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_status(
    conn: &RemoteConn<'_>,
    session: &str,
    json: bool,
    short: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_status_inner(conn, session, json, short)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote status", timeout_secs);
        }
    }
}

async fn cmd_remote_status_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    json: bool,
    short: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let params = serde_json::json!({
        "target": session,
    });

    match rpc_client.call("query.status", serde_json::json!("status"), params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let result = &r.result;

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
                    result["pid"].as_u64().unwrap_or(0),
                );
                return Ok(());
            }

            println!("Session: {} (on {})", result["id"].as_str().unwrap_or("?"), conn.hub);
            println!("  Name:        {}", result["display_name"].as_str().unwrap_or("?"));
            println!("  State:       {}", result["state"].as_str().unwrap_or("?"));
            println!("  PID:         {}", result["pid"]);
            println!("  Created:     {}", result["created_at"].as_str().unwrap_or("?"));
            println!("  Heartbeat:   {}", result["heartbeat_at"].as_str().unwrap_or("?"));
            if let Some(caps) = result.get("capabilities").and_then(|c| c.as_array()) {
                let cap_strs: Vec<&str> = caps.iter().filter_map(|c| c.as_str()).collect();
                if !cap_strs.is_empty() {
                    println!("  Capabilities: {}", cap_strs.join(", "));
                }
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
                let raw = mode["raw"].as_bool().unwrap_or(false);
                let canonical = mode["canonical"].as_bool().unwrap_or(false);
                let echo = mode["echo"].as_bool().unwrap_or(false);
                let alt_screen = mode["alternate_screen"].as_bool().unwrap_or(false);
                let mode_label = if raw { "raw" }
                    else if canonical && echo { "canonical+echo" }
                    else if canonical { "canonical" }
                    else { "cooked" };
                print!("  Term Mode:   {}", mode_label);
                if alt_screen { print!(" (alternate screen)"); }
                println!();
            }
            if let Some(meta) = result.get("metadata")
                && let Some(shell) = meta.get("shell").and_then(|s| s.as_str()) {
                    println!("  Shell:       {}", shell);
                }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Status query failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": format!("Status query error: {e}")}));
            }
            anyhow::bail!("Status query error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_inject(
    conn: &RemoteConn<'_>,
    opts: &RemoteInjectOpts<'_>,
) -> Result<()> {
    let RemoteInjectOpts { session, text, enter, key, delay_ms, json, timeout_secs } = *opts;
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_inject_inner(conn, session, text, enter, key, delay_ms, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote inject", timeout_secs);
        }
    }
}

async fn cmd_remote_inject_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    text: &str,
    enter: bool,
    key: Option<&str>,
    delay_ms: u64,
    json: bool,
) -> Result<()> {
    let mut client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let mut keys = Vec::new();
    if let Some(key_name) = key {
        keys.push(serde_json::json!({ "type": "key", "value": key_name }));
    } else {
        keys.push(serde_json::json!({ "type": "text", "value": text }));
    }
    if enter {
        keys.push(serde_json::json!({ "type": "key", "value": "Enter" }));
    }

    let inject_params = serde_json::json!({
        "target": session,
        "keys": keys,
        "inject_delay_ms": delay_ms,
    });

    match client.call("command.inject", serde_json::json!("inject"), inject_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            if json {
                let mut wrapped = serde_json::json!({"ok": true});
                if let Some(obj) = r.result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", serde_json::to_string_pretty(&wrapped)?);
            } else {
                let bytes = r.result["bytes_len"].as_u64().unwrap_or(0);
                println!("Injected {} bytes into '{}' on {}", bytes, session, conn.hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Inject failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": format!("Inject error: {e}")}));
            }
            anyhow::bail!("Inject error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_send_file(
    conn: &RemoteConn<'_>,
    session: &str,
    path: &str,
    chunk_size: usize,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_send_file_inner(conn, session, path, chunk_size, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
            }
            anyhow::bail!("Timeout after {}s waiting for remote file transfer", timeout_secs);
        }
    }
}

async fn cmd_remote_send_file_inner(
    conn: &RemoteConn<'_>,
    session: &str,
    path: &str,
    chunk_size: usize,
    json: bool,
) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let file_path = std::path::Path::new(path);
    let file_data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to read file: {path}: {e}")}));
            }
            return Err(e).context(format!("Failed to read file: {}", path));
        }
    };

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let size = file_data.len() as u64;
    let chunk_sz = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total_chunks = file_data.len().div_ceil(chunk_sz) as u32;
    let transfer_id = generate_request_id().replace("req-", "xfer-");

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let sha256 = format!("{:x}", hasher.finalize());

    let mut client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    eprintln!(
        "Sending '{}' ({} bytes, {} chunks) to '{}' on {}",
        filename, size, total_chunks, session, conn.hub
    );

    let init = FileInit {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        filename: filename.clone(),
        size,
        total_chunks,
        from: format!("remote-cli-{}", std::process::id()),
    };
    let init_payload = serde_json::to_value(&init)?;
    let emit_params = serde_json::json!({
        "target": session,
        "topic": file_topic::INIT,
        "payload": init_payload,
    });
    match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Failed to emit file.init: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to emit file.init: {e}")}));
            }
            anyhow::bail!("Failed to emit file.init: {}", e);
        }
    }

    let encoder = base64::engine::general_purpose::STANDARD;
    for (i, chunk_data) in file_data.chunks(chunk_sz).enumerate() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: encoder.encode(chunk_data),
        };
        let chunk_payload = serde_json::to_value(&chunk)?;
        let emit_params = serde_json::json!({
            "target": session,
            "topic": file_topic::CHUNK,
            "payload": chunk_payload,
        });
        match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
            Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
            Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                let msg = format!("Failed to emit chunk {}/{}: {} {}", i + 1, total_chunks, e.error.code, e.error.message);
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
                }
                anyhow::bail!("{}", msg);
            }
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to emit chunk {}/{}: {}", i + 1, total_chunks, e)}));
                }
                anyhow::bail!("Failed to emit chunk {}/{}: {}", i + 1, total_chunks, e);
            }
        }
        if total_chunks > 1 {
            eprint!("\r  Chunk {}/{}", i + 1, total_chunks);
        }
    }
    if total_chunks > 1 {
        eprintln!();
    }

    let complete = FileComplete {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        sha256: sha256.clone(),
    };
    let complete_payload = serde_json::to_value(&complete)?;
    let emit_params = serde_json::json!({
        "target": session,
        "topic": file_topic::COMPLETE,
        "payload": complete_payload,
    });
    match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = format!("Failed to emit file.complete: {} {}", e.error.code, e.error.message);
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to emit file.complete: {e}")}));
            }
            anyhow::bail!("Failed to emit file.complete: {}", e);
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "transfer_id": transfer_id,
            "filename": filename,
            "size": size,
            "chunks": total_chunks,
            "sha256": sha256,
            "hub": conn.hub,
            "session": session,
        }));
    } else {
        eprintln!("Transfer complete. SHA-256: {}", sha256);
        println!("Sent '{}' ({} bytes) to '{}' on {}", filename, size, session, conn.hub);
    }

    Ok(())
}

pub(crate) async fn cmd_remote_events(
    conn: &RemoteConn<'_>,
    topic_filter: Option<&str>,
    targets_csv: Option<&str>,
    interval_ms: u64,
    max_count: u64,
    json: bool,
    payload_only: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let targets: Vec<&str> = targets_csv
        .map(|t| t.split(',').map(|s| s.trim()).collect())
        .unwrap_or_default();

    eprintln!("Watching events on {}. Press Ctrl+C to stop.", conn.hub);
    if let Some(t) = topic_filter {
        eprintln!("  Topic filter: {}", t);
    }
    if !targets.is_empty() {
        eprintln!("  Targets: {}", targets.join(", "));
    }
    eprintln!();

    let subscribe_timeout_ms = interval_ms.max(500);
    let mut cursors = serde_json::json!({});
    let mut total_received: u64 = 0;

    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                eprintln!();
                eprintln!("Stopped. {} event(s) collected.", total_received);
                break;
            }
            collect_result = async {
                let mut params = serde_json::json!({
                    "timeout_ms": subscribe_timeout_ms,
                });
                if !targets.is_empty() {
                    params["targets"] = serde_json::json!(targets);
                }
                if let Some(t) = topic_filter {
                    params["topic"] = serde_json::json!(t);
                }
                if !cursors.as_object().is_none_or(|m| m.is_empty()) {
                    params["since"] = cursors.clone();
                }

                rpc_client.call("event.collect", serde_json::json!("collect"), params).await
            } => {
                match collect_result {
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                        if let Some(events) = r.result["events"].as_array() {
                            for event in events {
                                total_received += 1;

                                if payload_only {
                                    let payload = &event["payload"];
                                    if !payload.is_null() {
                                        println!("{}", serde_json::to_string(payload).unwrap_or_default());
                                    }
                                } else if json {
                                    let mut wrapped = serde_json::json!({"ok": true});
                                    if let Some(obj) = event.as_object() {
                                        for (k, v) in obj {
                                            wrapped[k] = v.clone();
                                        }
                                    }
                                    println!("{}", serde_json::to_string(&wrapped).unwrap_or_default());
                                } else {
                                    let session_name = event["session_name"].as_str().unwrap_or("?");
                                    let seq = event["seq"].as_u64().unwrap_or(0);
                                    let topic = event["topic"].as_str().unwrap_or("?");
                                    let payload = &event["payload"];
                                    let ts = event["timestamp"].as_u64().unwrap_or(0);

                                    if payload.is_null()
                                        || payload.as_object().is_some_and(|o| o.is_empty())
                                    {
                                        println!("[{session_name}#{seq}] {topic} (t={ts})");
                                    } else {
                                        println!(
                                            "[{session_name}#{seq}] {topic}: {} (t={ts})",
                                            serde_json::to_string(payload).unwrap_or_default()
                                        );
                                    }
                                }
                            }
                        }

                        if let Some(new_cursors) = r.result.get("cursors")
                            && let Some(obj) = new_cursors.as_object()
                        {
                            for (k, v) in obj {
                                cursors[k] = v.clone();
                            }
                        }

                        if max_count > 0 && total_received >= max_count {
                            eprintln!();
                            eprintln!("{} event(s) collected (limit reached).", total_received);
                            break;
                        }
                    }
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                        eprintln!("Collect error: {} {}. Retrying...", e.error.code, e.error.message);
                    }
                    Err(e) => {
                        eprintln!("Hub connection error: {}. Retrying...", e);
                    }
                }
            }
        }
    }

    Ok(())
}

pub(crate) async fn cmd_remote_exec(
    conn: &RemoteConn<'_>,
    session: &str,
    command: &str,
    timeout: u64,
    cwd: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "hub": conn.hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let mut params = serde_json::json!({
        "target": session,
        "command": command,
        "timeout": timeout,
    });
    if let Some(dir) = cwd {
        params["cwd"] = serde_json::json!(dir);
    }

    match rpc_client.call("command.execute", serde_json::json!("exec"), params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let result = &r.result;

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
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, conn.hub)
            } else {
                format!("Execution failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": msg}));
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "session": session, "hub": conn.hub, "error": format!("Execution error: {e}")}));
            }
            anyhow::bail!("Execution error: {}", e);
        }
    }
}

/// Remote inbox operations — query/clear inbox on a remote hub via RPC (T-1009).
pub(crate) async fn cmd_remote_inbox(
    conn: &RemoteConn<'_>,
    action: RemoteInboxAction,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_inbox_inner(conn, action)).await {
        Ok(result) => result,
        Err(_) => anyhow::bail!("Timeout after {}s waiting for remote inbox RPC", timeout_secs),
    }
}

async fn cmd_remote_inbox_inner(
    conn: &RemoteConn<'_>,
    action: RemoteInboxAction,
) -> Result<()> {
    let mut rpc_client = connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope)
        .await
        .context("Failed to connect to remote hub")?;

    match action {
        RemoteInboxAction::Status { json } => {
            let resp = rpc_client
                .call("inbox.status", serde_json::json!("inbox-s"), serde_json::json!({}))
                .await
                .context("inbox.status RPC failed")?;
            match resp {
                termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&r.result)?);
                    } else {
                        let total = r.result["total_transfers"].as_u64().unwrap_or(0);
                        if total == 0 {
                            println!("Inbox on {}: empty (no pending transfers)", conn.hub);
                        } else {
                            let targets = r.result["targets"].as_array();
                            println!("Inbox on {}: {} pending transfer(s)", conn.hub, total);
                            if let Some(targets) = targets {
                                for t in targets {
                                    println!(
                                        "  {} — {} transfer(s)",
                                        t["target"].as_str().unwrap_or("?"),
                                        t["transfer_count"].as_u64().unwrap_or(0),
                                    );
                                }
                            }
                        }
                    }
                }
                termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
                    anyhow::bail!("inbox.status error: {}", e.error.message);
                }
            }
        }
        RemoteInboxAction::List { target, json } => {
            let resp = rpc_client
                .call("inbox.list", serde_json::json!("inbox-l"), serde_json::json!({"target": target}))
                .await
                .context("inbox.list RPC failed")?;
            match resp {
                termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&r.result)?);
                    } else {
                        let transfers = r.result["transfers"].as_array();
                        if let Some(transfers) = transfers {
                            if transfers.is_empty() {
                                println!("No pending transfers for '{}' on {}", target, conn.hub);
                            } else {
                                println!("Pending transfers for '{}' on {}:", target, conn.hub);
                                for t in transfers {
                                    let id = t["transfer_id"].as_str().unwrap_or("?");
                                    let filename = t["filename"].as_str().unwrap_or("?");
                                    let size = t["total_size"].as_u64().unwrap_or(0);
                                    println!("  {} — {} ({} bytes)", id, filename, size);
                                }
                            }
                        } else {
                            println!("No transfers for '{}' on {}", target, conn.hub);
                        }
                    }
                }
                termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
                    anyhow::bail!("inbox.list error: {}", e.error.message);
                }
            }
        }
        RemoteInboxAction::Clear { target, all, json } => {
            let params = if all {
                serde_json::json!({"all": true})
            } else if let Some(ref t) = target {
                serde_json::json!({"target": t})
            } else {
                anyhow::bail!("Specify a target name or use --all");
            };
            let resp = rpc_client
                .call("inbox.clear", serde_json::json!("inbox-c"), params)
                .await
                .context("inbox.clear RPC failed")?;
            match resp {
                termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&r.result)?);
                    } else {
                        let cleared = r.result["cleared"].as_u64().unwrap_or(0);
                        let tgt = r.result["target"].as_str().unwrap_or("*");
                        println!("Cleared {} transfer(s) for '{}' on {}", cleared, tgt, conn.hub);
                    }
                }
                termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
                    anyhow::bail!("inbox.clear error: {}", e.error.message);
                }
            }
        }
    }
    Ok(())
}

/// T-1102: One-screen fleet overview for human operators.
/// Shows each hub's status, session count, version, latency, and actionable fixes.
pub(crate) async fn cmd_fleet_status(
    json: bool,
    timeout_secs: u64,
    verbose: bool,
) -> Result<()> {
    use serde_json::json;

    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({
                "ok": true,
                "fleet": [],
                "summary": {"total": 0, "up": 0, "down": 0, "auth_fail": 0},
                "actions": []
            }))?);
        } else {
            eprintln!("No hubs configured. Add hubs with: termlink remote profile add <name> <host:port> --secret-file <path>");
        }
        return Ok(());
    }

    let mut hub_entries: Vec<serde_json::Value> = Vec::new();
    let mut actions: Vec<String> = Vec::new();
    let mut up_count = 0u32;
    let mut down_count = 0u32;
    let mut auth_fail_count = 0u32;

    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();

    for name in &hub_names {
        let entry = &config.hubs[*name];
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let connect_start = std::time::Instant::now();

        let result = tokio::time::timeout(
            timeout_dur,
            connect_remote_hub(
                &entry.address,
                entry.secret_file.as_deref(),
                entry.secret.as_deref(),
                entry.scope.as_deref().unwrap_or("execute"),
            ),
        ).await;

        match result {
            Ok(Ok(mut client)) => {
                let latency = connect_start.elapsed().as_millis();
                up_count += 1;

                // Query session count and optionally names
                let (session_count, session_names) = match client.call(
                    "session.discover", json!("fleet-sd"), json!({}),
                ).await {
                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                        let sessions = r.result["sessions"].as_array();
                        let count = sessions.map(|s| s.len()).unwrap_or(0);
                        let names: Vec<String> = sessions
                            .map(|s| s.iter()
                                .filter_map(|sess| sess["display_name"].as_str().map(String::from))
                                .collect())
                            .unwrap_or_default();
                        (count, names)
                    }
                    _ => (0, Vec::new()),
                };

                let mut hub_entry = json!({
                    "hub": name,
                    "address": entry.address,
                    "status": "up",
                    "latency_ms": latency,
                    "sessions": session_count,
                });
                if verbose {
                    hub_entry["session_names"] = json!(session_names);
                }
                hub_entries.push(hub_entry);

                if !json {
                    eprintln!("  \x1b[32mUP\x1b[0m    {:<20} {:<24} {:>3} sessions  ({}ms)",
                        name, entry.address, session_count, latency);
                    if verbose && !session_names.is_empty() {
                        for sname in &session_names {
                            eprintln!("         \x1b[2m- {}\x1b[0m", sname);
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                let msg = format!("{}", e);
                let is_auth = msg.contains("invalid signature")
                    || msg.contains("Token validation failed")
                    || msg.contains("TOFU VIOLATION")
                    || msg.contains("fingerprint changed");

                if is_auth {
                    auth_fail_count += 1;
                    hub_entries.push(json!({
                        "hub": name,
                        "address": entry.address,
                        "status": "auth-fail",
                        "error": &msg,
                    }));
                    if !json {
                        eprintln!("  \x1b[33mAUTH\x1b[0m  {:<20} {:<24} secret mismatch — hub was restarted with a new secret",
                            name, entry.address);
                    }
                    actions.push(format!(
                        "{}: Reauth needed — termlink fleet reauth {} --bootstrap-from ssh:<host>",
                        name, name
                    ));
                } else {
                    down_count += 1;
                    hub_entries.push(json!({
                        "hub": name,
                        "address": entry.address,
                        "status": "down",
                        "error": &msg,
                    }));
                    if !json {
                        eprintln!("  \x1b[31mDOWN\x1b[0m  {:<20} {:<24} {}",
                            name, entry.address, msg);
                    }
                    if msg.contains("Cannot connect") || msg.contains("Connection refused") {
                        actions.push(format!(
                            "{}: Hub process not running — start via: ssh root@{} systemctl start termlink-hub",
                            name, entry.address.split(':').next().unwrap_or(&entry.address)
                        ));
                    } else {
                        actions.push(format!("{}: {}", name, msg));
                    }
                }

                // Track failure for learning/concern auto-register
                let _ = maybe_record_auth_mismatch_learning(name, &entry.address, &msg);
                let _ = maybe_track_fleet_failure(name, &entry.address, auth_mismatch_class(&msg));
            }
            Err(_) => {
                down_count += 1;
                hub_entries.push(json!({
                    "hub": name,
                    "address": entry.address,
                    "status": "timeout",
                }));
                if !json {
                    eprintln!("  \x1b[31mDOWN\x1b[0m  {:<20} {:<24} timeout after {}s",
                        name, entry.address, timeout_secs);
                }
                actions.push(format!(
                    "{}: Timeout — check network connectivity to {}",
                    name, entry.address
                ));
            }
        }
    }

    let total = hub_names.len() as u32;

    if json {
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": down_count == 0 && auth_fail_count == 0,
            "fleet": hub_entries,
            "summary": {
                "total": total,
                "up": up_count,
                "down": down_count,
                "auth_fail": auth_fail_count,
            },
            "actions": actions,
        }))?);
    } else {
        eprintln!();
        if up_count == total {
            eprintln!("  FLEET: \x1b[32mall {} hubs operational\x1b[0m", total);
        } else {
            eprintln!("  FLEET: {} hub(s), \x1b[32m{} up\x1b[0m, \x1b[31m{} down\x1b[0m, \x1b[33m{} auth-fail\x1b[0m",
                total, up_count, down_count, auth_fail_count);
        }

        if !actions.is_empty() {
            eprintln!();
            eprintln!("  ACTIONS NEEDED:");
            for (i, action) in actions.iter().enumerate() {
                eprintln!("    {}. {}", i + 1, action);
            }
        }
        eprintln!();
    }

    Ok(())
}

pub(crate) async fn cmd_fleet_doctor(
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        if json {
            println!("{}", serde_json::json!({"ok": true, "hubs": [], "message": "No hubs configured in ~/.termlink/hubs.toml"}));
        } else {
            eprintln!("No hubs configured in ~/.termlink/hubs.toml");
        }
        return Ok(());
    }

    if !json {
        eprintln!("Fleet doctor: {} hub(s) configured\n", config.hubs.len());
    }

    let mut hub_results: Vec<serde_json::Value> = Vec::new();
    let mut total_pass: u32 = 0;
    let total_warn: u32 = 0;
    let mut total_fail: u32 = 0;

    // Sort hub names for deterministic output
    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();

    for name in hub_names {
        let entry = &config.hubs[name];

        if !json {
            eprintln!("--- {} ({}) ---", name, entry.address);
        }

        // Quick connectivity check via connect_remote_hub
        let connect_start = std::time::Instant::now();
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let result = tokio::time::timeout(
            timeout_dur,
            connect_remote_hub(
                &entry.address,
                entry.secret_file.as_deref(),
                entry.secret.as_deref(),
                entry.scope.as_deref().unwrap_or("execute"),
            ),
        ).await;

        // T-1034: Resolve secret source for diagnostics
        let secret_source = entry.secret_file.as_deref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| {
                if entry.secret.is_some() { "inline secret".to_string() }
                else { "none".to_string() }
            });

        match result {
            Ok(Ok(_client)) => {
                let latency = connect_start.elapsed().as_millis();
                total_pass += 1;
                hub_results.push(serde_json::json!({"hub": name, "address": entry.address, "status": "ok", "latency_ms": latency, "secret_source": &secret_source}));
                if !json {
                    eprintln!("  [PASS] connected in {}ms", latency);
                }
                // T-1053: pass resets the auth-failure streak + re-arms concern gating.
                let _ = maybe_track_fleet_failure(name, &entry.address, None);
            }
            Ok(Err(e)) => {
                total_fail += 1;
                let msg = format!("{}", e);
                let diagnostic = classify_fleet_error(&msg, &entry.address);
                hub_results.push(serde_json::json!({"hub": name, "address": entry.address, "status": "error", "error": &msg, "secret_source": &secret_source, "diagnostic": &diagnostic}));
                if !json {
                    eprintln!("  [FAIL] {}", msg);
                    eprintln!("  secret: {}", secret_source);
                    eprintln!("  hint: {}", diagnostic);
                }
                // T-1052: auto-register a learning for auth/TOFU failure classes so drift
                // is detectable by future agents (R1). Silent best-effort — never blocks.
                let _ = maybe_record_auth_mismatch_learning(name, &entry.address, &msg);
                // T-1053: track per-hub streak; register a concern after N failures >24h apart.
                let _ = maybe_track_fleet_failure(name, &entry.address, auth_mismatch_class(&msg));
            }
            Err(_) => {
                total_fail += 1;
                let diagnostic = "Check network connectivity and that hub is listening on the configured port";
                hub_results.push(serde_json::json!({"hub": name, "address": entry.address, "status": "timeout", "secret_source": &secret_source, "diagnostic": diagnostic}));
                if !json {
                    eprintln!("  [FAIL] Timeout after {}s", timeout_secs);
                    eprintln!("  hint: {}", diagnostic);
                }
                // T-1053: timeouts aren't auth-class failures → reset streak.
                let _ = maybe_track_fleet_failure(name, &entry.address, None);
            }
        }

        if !json {
            eprintln!();
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": total_fail == 0,
            "hubs": hub_results,
            "summary": {"total": hub_results.len(), "pass": total_pass, "warn": total_warn, "fail": total_fail}
        }))?);
    } else {
        eprintln!("Fleet summary: {} hub(s), {} ok, {} warn, {} fail",
            hub_results.len(), total_pass, total_warn, total_fail);
    }

    Ok(())
}

/// T-1034: Classify fleet doctor errors into actionable diagnostic hints.
fn classify_fleet_error(msg: &str, address: &str) -> String {
    if msg.contains("invalid signature") || msg.contains("Token validation failed") {
        "Secret mismatch — hub was likely restarted with a new secret. \
         Fetch the current secret from the remote hub's hub.secret file".to_string()
    } else if msg.contains("TOFU VIOLATION") || msg.contains("fingerprint changed") {
        format!("Hub certificate changed. If expected (hub restart), clear with: \
         termlink tofu clear {address}")
    } else if msg.contains("Connection refused") {
        "Hub is not listening on this port. Check if the hub process is running \
         on the remote host (systemctl status termlink-hub)".to_string()
    } else if msg.contains("Secret file not found") {
        "The configured secret_file path does not exist. \
         Check hubs.toml and verify the file is present".to_string()
    } else if msg.contains("InvalidContentType") || msg.contains("tls") || msg.contains("TLS") {
        "TLS handshake failed — the hub may not be running TLS on this port, \
         or there is a protocol version mismatch".to_string()
    } else {
        "Unexpected error — check hub logs on the remote host for details".to_string()
    }
}

/// T-1106: Run a layered connectivity probe per hub.
///
/// Probes in order: TCP connect → TLS handshake → HMAC auth → RPC ping.
/// Each layer's result (pass/fail + latency) is reported independently so
/// the operator can see exactly where a connection breaks. Stops at the
/// first failing layer — subsequent layers require the prior to succeed.
pub(crate) async fn cmd_net_test(
    profile_filter: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    use serde_json::json;
    use std::time::{Duration, Instant};

    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({
                "ok": true, "hubs": [],
                "summary": {"total": 0, "healthy": 0, "degraded": 0, "unreachable": 0},
            }))?);
        } else {
            eprintln!("No hubs configured. Add hubs with: termlink remote profile add <name> <host:port> --secret-file <path>");
        }
        return Ok(());
    }

    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();
    if let Some(filter) = profile_filter {
        hub_names.retain(|n| n.as_str() == filter);
        if hub_names.is_empty() {
            anyhow::bail!("Hub profile '{}' not found. Run: termlink remote profile list", filter);
        }
    }

    let timeout_dur = Duration::from_secs(timeout_secs);
    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut healthy = 0u32;
    let mut degraded = 0u32;
    let mut unreachable = 0u32;

    for name in &hub_names {
        let entry = &config.hubs[*name];

        let (host, port) = match parse_host_port(&entry.address) {
            Ok(hp) => hp,
            Err(e) => {
                unreachable += 1;
                results.push(json!({
                    "hub": name, "address": entry.address,
                    "healthy": false, "diagnosis": format!("invalid address: {}", e),
                    "layers": {},
                }));
                continue;
            }
        };

        let mut layers = serde_json::Map::new();
        let mut hub_healthy = true;
        let mut diagnosis: Option<&'static str> = None;

        // --- L1: TCP ---
        let tcp_start = Instant::now();
        let tcp_result = tokio::time::timeout(
            timeout_dur,
            tokio::net::TcpStream::connect((host.as_str(), port)),
        ).await;
        let tcp_latency = tcp_start.elapsed().as_millis() as u64;
        let tcp_ok = matches!(tcp_result, Ok(Ok(_)));
        layers.insert("tcp".to_string(), match &tcp_result {
            Ok(Ok(_)) => json!({"status": "pass", "latency_ms": tcp_latency}),
            Ok(Err(e)) => json!({"status": "fail", "latency_ms": tcp_latency, "error": e.to_string()}),
            Err(_) => json!({"status": "timeout", "latency_ms": timeout_secs * 1000}),
        });
        if !tcp_ok {
            hub_healthy = false;
            diagnosis = Some("Network-level failure — check firewall/VPN/routing and hub process is listening on the configured port");
        }

        // --- L2: TLS ---
        if tcp_ok {
            let addr = termlink_protocol::TransportAddr::Tcp {
                host: host.clone(),
                port,
            };
            let tls_start = Instant::now();
            let tls_result = tokio::time::timeout(
                timeout_dur,
                client::Client::connect_addr(&addr),
            ).await;
            let tls_latency = tls_start.elapsed().as_millis() as u64;

            match tls_result {
                Ok(Ok(mut rpc_client)) => {
                    layers.insert("tls".to_string(),
                        json!({"status": "pass", "latency_ms": tls_latency}));

                    // --- L3: AUTH ---
                    let auth_outcome = net_probe_auth(&mut rpc_client, entry, timeout_dur).await;
                    match auth_outcome {
                        Ok(auth_latency) => {
                            layers.insert("auth".to_string(),
                                json!({"status": "pass", "latency_ms": auth_latency}));

                            // --- L4: PING (session.discover) ---
                            let ping_start = Instant::now();
                            let ping_result = tokio::time::timeout(
                                timeout_dur,
                                rpc_client.call("session.discover", json!("net-ping"), json!({})),
                            ).await;
                            let ping_latency = ping_start.elapsed().as_millis() as u64;
                            match ping_result {
                                Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_))) => {
                                    layers.insert("ping".to_string(),
                                        json!({"status": "pass", "latency_ms": ping_latency}));
                                }
                                Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e))) => {
                                    hub_healthy = false;
                                    diagnosis = Some("RPC call rejected — hub is authenticated but refusing session.discover");
                                    layers.insert("ping".to_string(), json!({
                                        "status": "fail", "latency_ms": ping_latency,
                                        "error": format!("{} {}", e.error.code, e.error.message),
                                    }));
                                }
                                Ok(Err(e)) => {
                                    hub_healthy = false;
                                    diagnosis = Some("RPC transport error after auth — hub may have disconnected");
                                    layers.insert("ping".to_string(), json!({
                                        "status": "fail", "latency_ms": ping_latency,
                                        "error": e.to_string(),
                                    }));
                                }
                                Err(_) => {
                                    hub_healthy = false;
                                    diagnosis = Some("RPC timeout after auth — hub is slow or overloaded");
                                    layers.insert("ping".to_string(), json!({
                                        "status": "timeout", "latency_ms": timeout_secs * 1000,
                                    }));
                                }
                            }
                        }
                        Err((auth_latency, msg)) => {
                            hub_healthy = false;
                            diagnosis = Some("HMAC secret mismatch — run: termlink fleet reauth <profile> --bootstrap-from ssh:<host>");
                            layers.insert("auth".to_string(), json!({
                                "status": "fail", "latency_ms": auth_latency,
                                "error": msg,
                            }));
                        }
                    }
                }
                Ok(Err(e)) => {
                    hub_healthy = false;
                    let msg = e.to_string();
                    diagnosis = Some(if msg.contains("TOFU") || msg.contains("fingerprint") {
                        "TLS cert changed — run: termlink tofu clear <host:port> and retry"
                    } else {
                        "TLS handshake failed — hub may not be speaking TLS, or cert is invalid"
                    });
                    layers.insert("tls".to_string(), json!({
                        "status": "fail", "latency_ms": tls_latency,
                        "error": msg,
                    }));
                }
                Err(_) => {
                    hub_healthy = false;
                    diagnosis = Some("TLS handshake timed out — hub is slow or silently dropping TLS");
                    layers.insert("tls".to_string(), json!({
                        "status": "timeout", "latency_ms": timeout_secs * 1000,
                    }));
                }
            }
        }

        if hub_healthy {
            healthy += 1;
        } else if layers.get("tcp").and_then(|l| l.get("status")).and_then(|s| s.as_str()) == Some("pass") {
            degraded += 1;
        } else {
            unreachable += 1;
        }

        let mut hub_result = json!({
            "hub": name,
            "address": entry.address,
            "healthy": hub_healthy,
            "layers": layers,
        });
        if let Some(d) = diagnosis {
            hub_result["diagnosis"] = json!(d);
        }
        results.push(hub_result);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": unreachable == 0 && degraded == 0,
            "hubs": results,
            "summary": {
                "total": hub_names.len(),
                "healthy": healthy,
                "degraded": degraded,
                "unreachable": unreachable,
            },
        }))?);
    } else {
        render_net_test_text(&results, healthy, degraded, unreachable);
    }

    Ok(())
}

/// Parse "host:port" into (host, port) — shared logic with connect_remote_hub.
fn parse_host_port(addr: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("expected host:port, got '{}'", addr);
    }
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse()
        .context(format!("invalid port in '{}'", addr))?;
    Ok((host, port))
}

/// Run the AUTH layer of the net test: build a token from the hub's secret
/// and call `hub.auth`. Returns Ok(latency_ms) on success or Err((latency_ms, message)).
async fn net_probe_auth(
    rpc_client: &mut client::Client,
    entry: &HubEntry,
    timeout_dur: std::time::Duration,
) -> std::result::Result<u64, (u64, String)> {
    use std::time::Instant;
    use termlink_session::auth::{self, PermissionScope};

    let start = Instant::now();

    // Read secret (file or inline)
    let hex = match (entry.secret_file.as_deref(), entry.secret.as_deref()) {
        (Some(path), _) => match std::fs::read_to_string(path) {
            Ok(s) => s.trim().to_string(),
            Err(e) => return Err((start.elapsed().as_millis() as u64,
                format!("cannot read secret file {}: {}", path, e))),
        },
        (None, Some(h)) => h.to_string(),
        (None, None) => return Err((start.elapsed().as_millis() as u64,
            "no secret configured (neither secret_file nor inline secret)".to_string())),
    };
    if hex.len() != 64 {
        return Err((start.elapsed().as_millis() as u64,
            format!("secret must be 64 hex chars, got {}", hex.len())));
    }
    let secret_bytes: Vec<u8> = match (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<std::result::Result<Vec<u8>, _>>()
    {
        Ok(b) => b,
        Err(e) => return Err((start.elapsed().as_millis() as u64,
            format!("invalid hex in secret: {}", e))),
    };
    let secret: auth::TokenSecret = match secret_bytes.try_into() {
        Ok(s) => s,
        Err(_) => return Err((start.elapsed().as_millis() as u64,
            "secret must decode to exactly 32 bytes".to_string())),
    };

    let scope_str = entry.scope.as_deref().unwrap_or("execute");
    let perm_scope = match scope_str {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        _ => return Err((start.elapsed().as_millis() as u64,
            format!("invalid scope '{}'", scope_str))),
    };
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    let auth_result = tokio::time::timeout(
        timeout_dur,
        rpc_client.call("hub.auth", serde_json::json!("net-auth"),
            serde_json::json!({"token": token.raw})),
    ).await;

    let latency = start.elapsed().as_millis() as u64;
    match auth_result {
        Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_))) => Ok(latency),
        Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e))) => {
            Err((latency, format!("{} {}", e.error.code, e.error.message)))
        }
        Ok(Err(e)) => Err((latency, format!("RPC error: {}", e))),
        Err(_) => Err((latency, format!("auth timeout after {}s", timeout_dur.as_secs()))),
    }
}

/// Render the text output for `termlink net test`.
fn render_net_test_text(
    results: &[serde_json::Value],
    healthy: u32,
    degraded: u32,
    unreachable: u32,
) {
    eprintln!();
    for hub in results {
        let name = hub["hub"].as_str().unwrap_or("?");
        let addr = hub["address"].as_str().unwrap_or("?");
        let hub_healthy = hub["healthy"].as_bool().unwrap_or(false);

        let (badge, colour) = if hub_healthy {
            ("HEALTHY", "\x1b[32m")
        } else {
            ("FAIL", "\x1b[31m")
        };
        eprintln!("  {colour}{badge}\x1b[0m  {name}  ({addr})");

        for layer in ["tcp", "tls", "auth", "ping"] {
            let Some(entry) = hub["layers"].get(layer) else { continue };
            let status = entry["status"].as_str().unwrap_or("?");
            let latency = entry["latency_ms"].as_u64().unwrap_or(0);
            let marker = match status {
                "pass" => "\x1b[32mPASS\x1b[0m",
                "fail" => "\x1b[31mFAIL\x1b[0m",
                "timeout" => "\x1b[31mTIME\x1b[0m",
                _ => "----",
            };
            let layer_upper = layer.to_uppercase();
            eprintln!("    {marker}  {layer_upper:<4}  {latency:>4}ms");
            if status != "pass" {
                if let Some(err) = entry["error"].as_str() {
                    eprintln!("          \x1b[2m└─ {}\x1b[0m", err);
                }
            }
        }

        if let Some(diag) = hub["diagnosis"].as_str() {
            eprintln!("    \x1b[33m→\x1b[0m {}", diag);
        }
        eprintln!();
    }

    let total = results.len();
    if degraded == 0 && unreachable == 0 {
        eprintln!("  NET: \x1b[32mall {} hub(s) fully reachable\x1b[0m", total);
    } else {
        eprintln!("  NET: {} hub(s), \x1b[32m{} healthy\x1b[0m, \x1b[33m{} degraded\x1b[0m, \x1b[31m{} unreachable\x1b[0m",
            total, healthy, degraded, unreachable);
    }
    eprintln!();
}

/// T-1052: classify an error message into the auth/cert drift classes we care about.
/// Returns `None` for unrelated errors (connection refused, etc.) so we stay quiet.
fn auth_mismatch_class(msg: &str) -> Option<&'static str> {
    if msg.contains("invalid signature") || msg.contains("Token validation failed") {
        Some("auth-mismatch")
    } else if msg.contains("TOFU VIOLATION") || msg.contains("fingerprint changed") {
        Some("tofu-violation")
    } else {
        None
    }
}

/// T-1052: compute UTC ISO-8601 timestamp. Same algorithm as `termlink_session::tofu::now_utc`
/// but inlined here to avoid exporting a new public symbol purely for this helper.
fn utc_iso8601_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let ydays = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < ydays { break; }
        remaining -= ydays;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 0usize;
    for (i, &md) in mdays.iter().enumerate() {
        if remaining < md as i64 { mo = i; break; }
        remaining -= md as i64;
    }
    format!("{y:04}-{:02}-{:02}T{h:02}:{m:02}:{s:02}Z", mo + 1, remaining + 1)
}

/// T-1052: return `true` if a dedupe marker exists, is younger than 24h, and its
/// recorded fingerprint matches the current one (→ skip recording a duplicate).
fn marker_deduped(marker: &std::path::Path, fingerprint: &str) -> bool {
    let Ok(meta) = std::fs::metadata(marker) else { return false; };
    let Ok(modified) = meta.modified() else { return false; };
    let Ok(age) = modified.elapsed() else { return false; };
    if age.as_secs() >= 86400 { return false; }
    let Ok(content) = std::fs::read_to_string(marker) else { return false; };
    content.trim() == fingerprint
}

/// T-1052 / R1 compliance: when fleet-doctor sees an auth-mismatch or TOFU violation,
/// append a learning to `.context/project/learnings.yaml` carrying the hub address,
/// the current pinned fingerprint (or "unknown"), and an ISO-8601 UTC timestamp.
///
/// Future agents can compare the recorded `hub_fingerprint=` against the currently
/// pinned fingerprint to detect memory drift (the learning was written under a
/// previous rotation of the cert).
///
/// Deduped via `.context/working/.fleet-learning-<hub>`: skip if a marker younger
/// than 24h exists AND the fingerprint hasn't changed since.
///
/// Best-effort only: silently no-ops when run outside a framework-managed project
/// (no `.context/project/` dir present). Never fails the caller.
pub(crate) fn maybe_record_auth_mismatch_learning(
    hub_name: &str,
    address: &str,
    error_msg: &str,
) -> Result<()> {
    let class = match auth_mismatch_class(error_msg) {
        Some(c) => c,
        None => return Ok(()),
    };

    // Locate framework project root from CWD. No-op outside framework projects.
    let cwd = std::env::current_dir()?;
    let learnings_path = cwd.join(".context/project/learnings.yaml");
    if !learnings_path.exists() {
        return Ok(());
    }
    let working_dir = cwd.join(".context/working");
    if let Err(e) = std::fs::create_dir_all(&working_dir) {
        return Err(anyhow::anyhow!("failed to create .context/working: {e}"));
    }

    // Look up the currently pinned fingerprint (may be absent if TOFU entry not yet recorded).
    let fingerprint = termlink_session::tofu::KnownHubStore::default_store()
        .get(address)
        .unwrap_or_else(|| "unknown".to_string());

    // Dedupe: marker file content = fingerprint at last recording. Same fingerprint
    // within 24h → skip. Changed fingerprint or older marker → record again.
    let safe_name: String = hub_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let marker = working_dir.join(format!(".fleet-learning-{safe_name}"));
    if marker_deduped(&marker, &fingerprint) {
        return Ok(());
    }

    // Build the learning entry. Fingerprint + timestamp are embedded in the learning
    // text as `key=value` pairs so future drift-detection can parse them without
    // extending the framework's `add-learning` schema.
    let now_iso = utc_iso8601_now();
    let date_only = now_iso.split('T').next().unwrap_or("");
    let learning_text = format!(
        "Fleet doctor observed {class} on hub '{hub_name}' ({address}). \
         date_observed={now_iso} hub_fingerprint={fingerprint}. \
         T-1051 Option D auto-registration — if a later agent sees a different pinned \
         fingerprint for this hub, this learning is stale and should not be acted on.",
    );

    // Determine next PL-XXX id by scanning existing entries.
    let existing = std::fs::read_to_string(&learnings_path).unwrap_or_default();
    let max_id = existing
        .lines()
        .filter_map(|l| l.trim().strip_prefix("- id: PL-"))
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let next_id = max_id + 1;
    let id = format!("PL-{:03}", next_id);

    let entry = format!(
        "- id: {id}\n  learning: \"{text}\"\n  source: T-1052\n  task: T-1051\n  date: {date}\n  context: fleet-doctor auto-registered\n  application: \"Drift-detection: compare hub_fingerprint in this learning against current KnownHubStore.get(address)\"\n",
        text = learning_text,
        date = date_only,
    );

    let mut new_content = existing;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&entry);
    std::fs::write(&learnings_path, new_content)
        .with_context(|| format!("failed to write {}", learnings_path.display()))?;

    // Refresh the dedupe marker.
    let _ = std::fs::write(&marker, &fingerprint);

    Ok(())
}

// =========================================================================
// T-1053: fleet-doctor concern auto-registration (G-019 compliance)
//
// One-off observations are recorded as learnings by T-1052. A sustained
// pattern — ≥3 consecutive fleet-doctor failures for the same hub, spanning
// >24h — is promoted to a concern in .context/project/concerns.yaml so it
// surfaces in Watchtower and audit passes.
//
// Per-hub state is persisted in .context/working/.fleet-failure-state.json
// as { "hubs": { "<hub>": { "consecutive_failures": N, "first_failure_at":
// "...", "last_failure_at": "...", "last_class": "...", "concern_registered":
// bool } } }. Passing runs reset the counter and re-arm concern_registered.
// =========================================================================

const FLEET_CONCERN_THRESHOLD: u32 = 3;
const FLEET_CONCERN_AGE_SECS: u64 = 86_400;

/// State file path. Returns None when outside a framework project.
fn fleet_state_path() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let working = cwd.join(".context/working");
    if !working.exists() {
        return None;
    }
    Some(working.join(".fleet-failure-state.json"))
}

fn fleet_concerns_path() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let concerns = cwd.join(".context/project/concerns.yaml");
    if !concerns.exists() {
        return None;
    }
    Some(concerns)
}

/// Parse the `YYYY-MM-DDTHH:MM:SSZ` format produced by `utc_iso8601_now`
/// into seconds since the epoch. Returns `None` on parse failure.
///
/// Deliberately permissive — the input comes from our own writer, so we
/// accept whatever we emit and silently fail on anything else rather than
/// panicking inside a best-effort pathway.
fn parse_iso8601_utc(s: &str) -> Option<u64> {
    // Format: YYYY-MM-DDTHH:MM:SSZ  (20 chars)
    if s.len() < 19 || !s.ends_with('Z') {
        return None;
    }
    let y: i64 = s.get(0..4)?.parse().ok()?;
    let mo: u32 = s.get(5..7)?.parse().ok()?;
    let d: u32 = s.get(8..10)?.parse().ok()?;
    let h: u32 = s.get(11..13)?.parse().ok()?;
    let mi: u32 = s.get(14..16)?.parse().ok()?;
    let se: u32 = s.get(17..19)?.parse().ok()?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 60 {
        return None;
    }

    // Days from epoch (1970-01-01) to Y-M-D.
    let mut days: i64 = 0;
    for yr in 1970..y {
        days += if yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0) { 366 } else { 365 };
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, &md) in mdays.iter().enumerate() {
        if (i as u32) + 1 == mo {
            break;
        }
        days += md as i64;
    }
    days += (d as i64) - 1;

    let secs = (days * 86_400) + (h as i64) * 3600 + (mi as i64) * 60 + (se as i64);
    if secs < 0 { None } else { Some(secs as u64) }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Load state file, returning an empty map if the file is absent or malformed.
/// Best-effort: we never fail the caller just because state is unreadable.
fn load_fleet_state(path: &std::path::Path) -> serde_json::Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({"hubs": {}}))
}

fn save_fleet_state(path: &std::path::Path, state: &serde_json::Value) {
    if let Ok(s) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, s);
    }
}

/// Update per-hub failure tracking and, on threshold breach, append a concern.
///
/// Classification rules:
/// - `class = None` (no auth-class error) → treated as a reset signal for the
///   hub's auth-failure streak. The hub may still be failing for other reasons
///   (connection refused, TLS handshake, etc.), but those aren't the
///   auth-rotation pattern T-1051 targets.
/// - `class = Some("auth-mismatch" | "tofu-violation")` → increment the streak.
///
/// Best-effort: silently no-ops outside framework projects (no `.context/`).
/// Never fails the caller.
pub(crate) fn maybe_track_fleet_failure(
    hub_name: &str,
    address: &str,
    class: Option<&str>,
) -> Result<()> {
    let state_path = match fleet_state_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    let mut state = load_fleet_state(&state_path);
    let now_iso = utc_iso8601_now();
    let now_secs = now_unix_secs();

    // Ensure hubs object exists.
    if !state.get("hubs").map(|v| v.is_object()).unwrap_or(false) {
        state["hubs"] = serde_json::json!({});
    }

    // Take (or init) the per-hub entry. We mutate via take-replace to avoid
    // borrow-checker fights with serde_json's indexing API.
    let mut hub_state = state["hubs"]
        .get(hub_name)
        .cloned()
        .unwrap_or_else(|| serde_json::json!({
            "consecutive_failures": 0,
            "first_failure_at": serde_json::Value::Null,
            "last_failure_at": serde_json::Value::Null,
            "last_class": serde_json::Value::Null,
            "concern_registered": false,
        }));

    match class {
        None => {
            // Reset — pass or non-auth failure.
            hub_state["consecutive_failures"] = serde_json::json!(0);
            hub_state["first_failure_at"] = serde_json::Value::Null;
            hub_state["last_failure_at"] = serde_json::Value::Null;
            hub_state["last_class"] = serde_json::Value::Null;
            hub_state["concern_registered"] = serde_json::json!(false);
        }
        Some(c) => {
            let prior = hub_state["consecutive_failures"].as_u64().unwrap_or(0);
            let new_count = prior + 1;
            hub_state["consecutive_failures"] = serde_json::json!(new_count);
            if prior == 0 {
                hub_state["first_failure_at"] = serde_json::json!(now_iso.clone());
            }
            hub_state["last_failure_at"] = serde_json::json!(now_iso.clone());
            hub_state["last_class"] = serde_json::json!(c);

            // Threshold check.
            let already_registered = hub_state["concern_registered"].as_bool().unwrap_or(false);
            let first_at = hub_state["first_failure_at"].as_str().unwrap_or("");
            let first_secs = parse_iso8601_utc(first_at).unwrap_or(now_secs);
            let age = now_secs.saturating_sub(first_secs);

            if !already_registered
                && (new_count as u32) >= FLEET_CONCERN_THRESHOLD
                && age > FLEET_CONCERN_AGE_SECS
                && append_fleet_concern(hub_name, address, c, new_count as u32, first_at, &now_iso).is_ok()
            {
                hub_state["concern_registered"] = serde_json::json!(true);
            }
        }
    }

    state["hubs"][hub_name] = hub_state;
    save_fleet_state(&state_path, &state);
    Ok(())
}

/// Append a gap-type concern to `.context/project/concerns.yaml`.
fn append_fleet_concern(
    hub_name: &str,
    address: &str,
    class: &str,
    consecutive: u32,
    first_at: &str,
    now_iso: &str,
) -> Result<()> {
    let path = match fleet_concerns_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let max_id = existing
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("- id: G-").or_else(|| t.strip_prefix("id: G-"))
        })
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let id = format!("G-{:03}", max_id + 1);
    let date_only = now_iso.split('T').next().unwrap_or("");

    let title = format!(
        "TermLink hub '{hub_name}' ({address}) has been failing fleet-doctor with {class} for {consecutive}+ consecutive runs over >24h"
    );
    let description = format!(
        "Auto-registered by T-1053 under G-019 (framework-blind-to-persistent-failure guard). Hub has failed fleet-doctor with error class '{class}' for {consecutive} consecutive runs since {first_at} (first observed). This indicates either a genuinely broken hub auth state OR stale client credentials that were not refreshed after a hub rotation. Per T-1051 Option D, the heal path is: 1) confirm the hub is intended to be up (termlink remote ping {address} from a trusted anchor), 2) if yes, refresh the client secret (termlink fleet reauth {hub_name} when T-1054 lands, or manually copy /var/lib/termlink/hub.secret via an out-of-band channel for now)."
    );

    let entry = format!(
        "\n- id: {id}\n  type: gap\n  title: \"{title}\"\n  description: \"{description}\"\n  spec_reference: \"T-1051 inception, T-1053 implementation, .context/working/.fleet-failure-state.json\"\n  severity: high\n  trigger_fired: true\n  trigger_event: \"{now_iso}: {consecutive} consecutive fleet-doctor failures on {hub_name} ({address}) with class {class}, first observed {first_at}\"\n  detection_lag_days: \"1\"\n  what_works_now: \"Fleet doctor correctly classifies the error and emits a hint. T-1052 has already recorded an isolated learning. This concern escalates the sustained pattern to Watchtower visibility.\"\n  what_remains: \"Operator must refresh the client's cached secret for this hub. Long-term fix: T-1054 (termlink fleet reauth) lands a one-command heal.\"\n  mitigation_candidate: \"Ship T-1054 (fleet reauth Tier-1) and T-1055 (fleet reauth --bootstrap-from, Tier-2).\"\n  status: watching\n  created: {date_only}\n  last_reviewed: {date_only}\n  related_tasks: [T-1051, T-1052, T-1053, T-1054, T-1055]\n",
        id = id,
        title = title,
        description = description,
        now_iso = now_iso,
        consecutive = consecutive,
        hub_name = hub_name,
        address = address,
        class = class,
        first_at = first_at,
        date_only = date_only,
    );

    let mut new_content = existing;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(&entry);
    std::fs::write(&path, new_content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

// =========================================================================
// T-1054: fleet reauth — print the copy-pasteable heal incantation for a hub
// profile. Tier-1 only (no automation / no SSH). `--bootstrap-from` lands in
// T-1055. The trust anchor MUST be out-of-band — we explicitly label it.
// =========================================================================

/// Render the heal plan for a single hub profile. Pure — no IO, no stdout —
/// so it can be unit-tested without a filesystem or a shell.
fn render_fleet_reauth_plan(profile: &str, entry: &crate::config::HubEntry) -> String {
    let mut out = String::new();
    out.push_str(&format!("# TermLink fleet reauth — {profile}\n"));
    out.push_str(&format!("Hub profile:      {profile}\n"));
    out.push_str(&format!("Hub address:      {}\n", entry.address));
    match (&entry.secret_file, &entry.secret) {
        (Some(path), _) => {
            out.push_str(&format!("Secret source:    file → {path}\n"));
        }
        (None, Some(_)) => {
            out.push_str("Secret source:    inline in hubs.toml (WARNING — hard to rotate; consider switching to secret_file)\n");
        }
        (None, None) => {
            out.push_str("Secret source:    NONE configured (hubs.toml entry is missing both secret_file and secret)\n");
        }
    }

    out.push('\n');
    out.push_str("Trust anchor:     OUT-OF-BAND required (T-1054 is Tier-1 — no automation).\n");
    out.push_str("                  The channel that delivers the new secret MUST NOT itself depend\n");
    out.push_str("                  on termlink auth (chicken-and-egg). Use SSH, git-pull with signed\n");
    out.push_str("                  commits, physical USB, or another channel whose trust survives a\n");
    out.push_str("                  termlink cert/secret rotation. T-1055 will add `--bootstrap-from`\n");
    out.push_str("                  so the anchor becomes an explicit command argument.\n\n");

    out.push_str("# Heal steps (copy-paste, adjust host as needed):\n\n");
    let host = entry.address.split(':').next().unwrap_or(&entry.address);
    out.push_str(&format!("  # 1. Read the current secret on the hub host ({host}) via an out-of-band channel.\n"));
    out.push_str("  #    Example (requires working SSH to the hub host):\n");
    out.push_str(&format!("  ssh {host} -- sudo cat /var/lib/termlink/hub.secret\n\n"));

    match &entry.secret_file {
        Some(path) => {
            out.push_str("  # 2. Write the hex value from step 1 to the local secret file.\n");
            out.push_str(&format!("  echo \"<paste-the-hex-from-step-1>\" > {path}\n"));
            out.push_str(&format!("  chmod 600 {path}\n\n"));
        }
        None => {
            out.push_str("  # 2. Update the inline secret in ~/.termlink/hubs.toml:\n");
            out.push_str(&format!("  #    [hubs.{profile}]\n"));
            out.push_str("  #    secret = \"<paste-the-hex-from-step-1>\"\n");
            out.push_str("  #    (Consider switching to secret_file = \"/root/.termlink/secrets/<host>.hex\" for cleaner rotation.)\n\n");
        }
    }

    out.push_str("# 3. Verify the heal:\n");
    out.push_str("  termlink fleet doctor\n\n");
    out.push_str(&format!("# Expected: the [PASS] line for {profile} ({}) appears and fleet-doctor reports 0 fail.\n",
        entry.address));
    out.push_str("# If still failing: confirm the hub's hub.secret file is actually the one the hub is\n");
    out.push_str("# serving (hub may be using a different runtime_dir — T-1031 handoff, see `termlink doctor`).\n");

    out
}

/// `termlink fleet reauth <profile> [--bootstrap-from SOURCE]`.
///
/// When `bootstrap_from` is `None`: prints the Tier-1 heal incantation
/// (T-1054 behavior preserved).
///
/// When `bootstrap_from` is `Some("file:PATH" | "ssh:HOST")`: fetches the
/// new secret via the named out-of-band channel, validates it, backs up the
/// existing secret file, and writes the new one at chmod 600 (T-1055, R2).
pub(crate) fn cmd_fleet_reauth(profile: &str, bootstrap_from: Option<&str>) -> Result<()> {
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        anyhow::bail!(
            "No hubs configured in {}. Add one with: termlink profile add {} <host:port> --secret-file <path>",
            crate::config::hubs_config_path().display(),
            profile,
        );
    }
    let entry = match config.hubs.get(profile) {
        Some(e) => e,
        None => {
            let mut known: Vec<&String> = config.hubs.keys().collect();
            known.sort();
            let known_list: Vec<String> = known.iter().map(|s| (*s).clone()).collect();
            anyhow::bail!(
                "Unknown hub profile '{profile}'. Configured profiles: {}. \
                 Add one with: termlink profile add {profile} <host:port> --secret-file <path>",
                if known_list.is_empty() { "<none>".to_string() } else { known_list.join(", ") },
            );
        }
    };

    match bootstrap_from {
        None => {
            // Tier-1 behavior — print the heal incantation.
            print!("{}", render_fleet_reauth_plan(profile, entry));
            Ok(())
        }
        Some(source) => cmd_fleet_reauth_bootstrap(profile, entry, source),
    }
}

/// T-1055 Tier-2 heal: fetch the new secret via the named out-of-band source,
/// validate it, back up the existing file, and write the new one.
fn cmd_fleet_reauth_bootstrap(
    profile: &str,
    entry: &crate::config::HubEntry,
    source: &str,
) -> Result<()> {
    let secret_file = match &entry.secret_file {
        Some(p) => p.clone(),
        None => anyhow::bail!(
            "Profile '{profile}' uses an inline secret (no secret_file). \
             The --bootstrap-from heal path writes to secret_file only. \
             Migrate first: in ~/.termlink/hubs.toml change [hubs.{profile}] to use \
             secret_file = \"/root/.termlink/secrets/<host>.hex\" instead of secret = ..., then retry."
        ),
    };

    // Resolve the bootstrap source to the hex value of the new secret.
    let raw = fetch_bootstrap_secret(source)
        .with_context(|| format!("failed to fetch new secret via {source}"))?;
    let hex = normalize_and_validate_secret_hex(&raw)
        .with_context(|| format!("new secret from {source} is not valid 64-char hex"))?;

    // Persist: back up existing, then atomically write the new file.
    let target = std::path::PathBuf::from(&secret_file);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create parent dir for secret file: {}", parent.display())
        })?;
    }
    if target.exists() {
        let backup = target.with_extension("hex.bak");
        std::fs::copy(&target, &backup).with_context(|| {
            format!("failed to back up existing secret to {}", backup.display())
        })?;
    }
    write_secret_file(&target, &hex)?;

    // Success — echo the short fingerprint (first 12 chars) so the operator can
    // confirm without leaking the full secret to terminal history.
    let preview: String = hex.chars().take(12).collect();
    eprintln!("[OK] heal complete");
    eprintln!("     profile:      {profile}");
    eprintln!("     address:      {}", entry.address);
    eprintln!("     secret file:  {secret_file}");
    eprintln!("     bootstrap:    {source}");
    eprintln!("     new secret:   {preview}… (first 12 of 64 hex chars)");
    eprintln!();
    eprintln!("Verify with: termlink fleet doctor");
    Ok(())
}

/// Read the hex secret from the named bootstrap source.
/// Scheme → behavior:
///   file:<path>    → read file contents (UTF-8)
///   ssh:<host>     → spawn `ssh <host> -- sudo cat /var/lib/termlink/hub.secret`
/// Any other prefix → error listing the accepted forms.
fn fetch_bootstrap_secret(source: &str) -> Result<String> {
    if let Some(path) = source.strip_prefix("file:") {
        if path.is_empty() {
            anyhow::bail!("file: source requires a path (e.g. file:/tmp/new-secret.hex)");
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read bootstrap file: {path}"))?;
        return Ok(content);
    }
    if let Some(host) = source.strip_prefix("ssh:") {
        if host.is_empty() {
            anyhow::bail!("ssh: source requires a host (e.g. ssh:hub.example.com)");
        }
        // Deliberately fixed remote path — matches the hub's default
        // runtime_dir for systemd-run deployments. Bespoke paths are an
        // explicit non-goal (see task scope).
        let output = std::process::Command::new("ssh")
            .args([host, "--", "sudo", "cat", "/var/lib/termlink/hub.secret"])
            .output()
            .with_context(|| format!("failed to invoke ssh for host '{host}'"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "ssh {host} -- sudo cat /var/lib/termlink/hub.secret exited with status {:?}: {}",
                output.status.code(),
                stderr.trim(),
            );
        }
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    anyhow::bail!(
        "Unknown bootstrap source '{source}'. Accepted forms:\n  file:<path>\n  ssh:<host>\n\
         Unsupported by design: command:<cmd> (arbitrary shell — reserved for a later task)."
    )
}

/// Trim and validate that `raw` is 64 hex chars (a 32-byte HMAC secret).
fn normalize_and_validate_secret_hex(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.len() != 64 {
        anyhow::bail!(
            "expected 64 hex characters, got {} characters (trimmed)",
            trimmed.len()
        );
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("bootstrap value contains non-hex characters");
    }
    Ok(trimmed.to_string())
}

/// Write a secret file at chmod 600. Creates the file if missing; overwrites
/// existing content atomically via a `<path>.tmp` → rename dance.
fn write_secret_file(path: &std::path::Path, hex: &str) -> Result<()> {
    let tmp = path.with_extension("hex.tmp");
    // Write content.
    std::fs::write(&tmp, hex)
        .with_context(|| format!("failed to write temp secret file: {}", tmp.display()))?;
    // Tighten perms on the temp file BEFORE the rename so there is no
    // window in which the final path exists with loose perms.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perm = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&tmp, perm).with_context(|| {
            format!("failed to chmod 600 on temp secret file: {}", tmp.display())
        })?;
    }
    std::fs::rename(&tmp, path).with_context(|| {
        format!("failed to promote {} → {}", tmp.display(), path.display())
    })?;
    Ok(())
}

pub(crate) async fn cmd_remote_doctor(
    conn: &RemoteConn<'_>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_doctor_inner(conn, json)).await {
        Ok(result) => result,
        Err(_) => anyhow::bail!("Timeout after {}s waiting for remote doctor RPC", timeout_secs),
    }
}

async fn cmd_remote_doctor_inner(
    conn: &RemoteConn<'_>,
    json: bool,
) -> Result<()> {
    use serde_json::json;

    let mut checks: Vec<serde_json::Value> = Vec::new();
    let mut pass_count: u32 = 0;
    let mut warn_count: u32 = 0;
    let mut fail_count: u32 = 0;

    macro_rules! check {
        ($name:expr, pass, $msg:expr) => {{
            pass_count += 1;
            checks.push(json!({"check": $name, "status": "pass", "message": $msg}));
            if !json { eprintln!("  [PASS] {}: {}", $name, $msg); }
        }};
        ($name:expr, warn, $msg:expr) => {{
            warn_count += 1;
            checks.push(json!({"check": $name, "status": "warn", "message": $msg}));
            if !json { eprintln!("  [WARN] {}: {}", $name, $msg); }
        }};
        ($name:expr, fail, $msg:expr) => {{
            fail_count += 1;
            checks.push(json!({"check": $name, "status": "fail", "message": $msg}));
            if !json { eprintln!("  [FAIL] {}: {}", $name, $msg); }
        }};
    }

    if !json {
        eprintln!("Remote doctor: {}", conn.hub);
    }

    // 1. Connectivity — connect + auth
    let connect_start = std::time::Instant::now();
    let mut rpc_client = match connect_remote_hub(conn.hub, conn.secret_file, conn.secret_hex, conn.scope).await {
        Ok(c) => {
            let latency = connect_start.elapsed().as_millis();
            check!("connectivity", pass, format!("connected in {}ms", latency));
            c
        }
        Err(e) => {
            check!("connectivity", fail, format!("cannot connect: {}", e));
            if json {
                println!("{}", json!({
                    "ok": false,
                    "hub": conn.hub,
                    "checks": checks,
                    "summary": {"pass": pass_count, "warn": warn_count, "fail": fail_count}
                }));
            }
            return Ok(());
        }
    };

    // 2. Session count via discover (session.list is a per-session method, not hub-level)
    match rpc_client.call("session.discover", json!("doc-sd"), json!({})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            if let Some(sessions) = r.result["sessions"].as_array() {
                let count = sessions.len();
                let names: Vec<&str> = sessions.iter()
                    .filter_map(|s| s["display_name"].as_str())
                    .collect();
                if count == 0 {
                    check!("sessions", warn, "no sessions registered");
                } else {
                    check!("sessions", pass, format!("{} session(s): {}", count, names.join(", ")));
                }
            } else {
                check!("sessions", warn, "unexpected response format");
            }
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            check!("sessions", warn, format!("session.discover error: {}", e.error.message));
        }
        Err(e) => {
            check!("sessions", warn, format!("session.discover RPC failed: {}", e));
        }
    }

    // 3. Inbox status
    match rpc_client.call("inbox.status", json!("doc-is"), json!({})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            let total = r.result["total_transfers"].as_u64().unwrap_or(0);
            if total == 0 {
                check!("inbox", pass, "no pending transfers");
            } else {
                let targets = r.result["targets"].as_array().map(|t| t.len()).unwrap_or(0);
                check!("inbox", warn, format!("{} pending transfer(s) for {} target(s)", total, targets));
            }
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            check!("inbox", warn, format!("inbox.status error: {}", e.error.message));
        }
        Err(e) => {
            check!("inbox", warn, format!("inbox RPC failed: {}", e));
        }
    }

    // Output
    if json {
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": fail_count == 0,
            "hub": conn.hub,
            "checks": checks,
            "summary": {"pass": pass_count, "warn": warn_count, "fail": fail_count}
        }))?);
    } else {
        eprintln!("\n  Summary: {} pass, {} warn, {} fail", pass_count, warn_count, fail_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SECRET_HEX: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[tokio::test]
    async fn connect_rejects_hub_without_colon() {
        let err = connect_remote_hub("myhost", None, Some(VALID_SECRET_HEX), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("host:port"),
            "expected host:port hint, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_hub_with_extra_colons() {
        let err = connect_remote_hub("a:b:c", None, Some(VALID_SECRET_HEX), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("host:port"),
            "expected host:port hint, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_non_numeric_port() {
        let err = connect_remote_hub("host:abc", None, Some(VALID_SECRET_HEX), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            format!("{err:#}").contains("Invalid port"),
            "expected Invalid port, got: {err:#}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_missing_secret() {
        let err = connect_remote_hub("host:9100", None, None, "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("--secret-file or --secret"),
            "expected secret-required message, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_short_secret() {
        let err = connect_remote_hub("host:9100", None, Some("abcd"), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("64 hex characters"),
            "expected 64-hex-char message, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_non_hex_secret() {
        let bad = "z".repeat(64);
        let err = connect_remote_hub("host:9100", None, Some(&bad), "control")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            format!("{err:#}").contains("invalid hex"),
            "expected invalid-hex message, got: {err:#}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_unknown_scope() {
        let err = connect_remote_hub("host:9100", None, Some(VALID_SECRET_HEX), "superuser")
            .await
            .err()
            .expect("expected validation error");
        assert!(
            err.to_string().contains("Invalid scope"),
            "expected Invalid scope message, got: {err}"
        );
    }

    #[tokio::test]
    async fn connect_rejects_missing_secret_file() {
        let err = connect_remote_hub(
            "host:9100",
            Some("/nonexistent/path/that/should/not/exist"),
            None,
            "control",
        )
        .await
        .err()
        .expect("expected validation error");
        assert!(
            format!("{err:#}").contains("Secret file not found"),
            "expected secret-file-not-found message, got: {err:#}"
        );
    }

    #[tokio::test]
    async fn connect_accepts_all_four_permission_scopes() {
        for scope in ["observe", "interact", "control", "execute"] {
            let err = connect_remote_hub("127.0.0.1:1", None, Some(VALID_SECRET_HEX), scope)
                .await
                .err()
            .expect("expected validation error");
            let msg = format!("{err:#}");
            assert!(
                !msg.contains("Invalid scope"),
                "scope {scope} was rejected: {msg}"
            );
            assert!(
                !msg.contains("64 hex characters"),
                "scope {scope} failed at secret length: {msg}"
            );
        }
    }

    // -------------------------------------------------------------------
    // T-1052: fleet-doctor auto-register learning on auth-mismatch
    // -------------------------------------------------------------------

    #[test]
    fn fleet_learning_classifies_auth_errors() {
        // Known auth-mismatch patterns
        assert_eq!(auth_mismatch_class("Token validation failed: invalid signature"), Some("auth-mismatch"));
        assert_eq!(auth_mismatch_class("rpc error: invalid signature"), Some("auth-mismatch"));
        // Known TOFU patterns
        assert_eq!(auth_mismatch_class("TOFU VIOLATION: fingerprint changed"), Some("tofu-violation"));
        assert_eq!(auth_mismatch_class("fingerprint changed unexpectedly"), Some("tofu-violation"));
        // Unrelated errors must be None (don't spam learnings)
        assert_eq!(auth_mismatch_class("Connection refused"), None);
        assert_eq!(auth_mismatch_class("Secret file not found"), None);
        assert_eq!(auth_mismatch_class("TLS handshake failed"), None);
    }

    /// Reuse the crate-wide test env lock. Any test in this binary that
    /// mutates CWD or HOME must lock through this to avoid racing with
    /// sibling tests (e.g. `config::tests::save_and_load_hubs_config`).
    use crate::test_env_lock::ENV_LOCK as CWD_LOCK;

    /// Create an isolated tempdir that looks like a framework project, cd into it,
    /// run the closure, then restore CWD. Returns whatever the closure returned.
    ///
    /// Robust against a pre-set broken CWD: some unrelated tests in this crate
    /// (e.g. dispatch::isolate_rejects_non_git_dir) `set_current_dir` into a
    /// `tempfile::tempdir` that auto-deletes at function exit, leaving CWD
    /// pointing at a removed directory. If we called `current_dir()` after
    /// that, it would ENOENT. So we always anchor back to "/" after each run,
    /// and never try to preserve the caller's prior CWD.
    fn with_framework_cwd<R>(f: impl FnOnce(&std::path::Path) -> R) -> R {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Anchor CWD to a known-good path before doing anything that would
        // observe the current dir.
        std::env::set_current_dir("/").expect("cd to /");

        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1052-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".context/project")).expect("create .context/project");
        std::fs::create_dir_all(tmp.join(".context/working")).expect("create .context/working");
        std::fs::write(
            tmp.join(".context/project/learnings.yaml"),
            "# Project Learnings\nlearnings:\n",
        )
        .expect("seed learnings.yaml");
        std::fs::write(
            tmp.join(".context/project/concerns.yaml"),
            "# Concerns Register\nconcerns:\n",
        )
        .expect("seed concerns.yaml");

        std::env::set_current_dir(&tmp).expect("cd into tmp");

        // Also isolate HOME so KnownHubStore doesn't touch the real ~/.termlink.
        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&tmp)));

        // Restore CWD to a known-good anchor BEFORE removing tmp — otherwise
        // the remove would leave CWD dangling.
        std::env::set_current_dir("/").expect("restore cwd to /");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        match result {
            Ok(v) => v,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    #[test]
    fn fleet_learning_writes_entry_on_auth_mismatch() {
        with_framework_cwd(|tmp| {
            let err_msg = "rpc error: Token validation failed: invalid signature";
            maybe_record_auth_mismatch_learning("ring20-management", "10.0.0.1:9100", err_msg)
                .expect("record learning");

            let learnings = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            assert!(learnings.contains("PL-001"), "new PL-XXX id not allocated: {learnings}");
            assert!(learnings.contains("ring20-management"), "hub name missing from learning");
            assert!(learnings.contains("10.0.0.1:9100"), "hub address missing from learning");
            assert!(learnings.contains("auth-mismatch"), "class missing from learning");
            assert!(learnings.contains("hub_fingerprint="), "fingerprint key missing from learning");
            assert!(learnings.contains("date_observed="), "date_observed key missing from learning");
            assert!(learnings.contains("source: T-1052"), "source T-1052 missing from entry");
            assert!(learnings.contains("task: T-1051"), "task T-1051 missing from entry");

            // Dedupe marker written.
            assert!(
                tmp.join(".context/working/.fleet-learning-ring20-management").exists(),
                "dedupe marker not created",
            );
        });
    }

    #[test]
    fn fleet_learning_skips_unrelated_errors() {
        with_framework_cwd(|tmp| {
            maybe_record_auth_mismatch_learning("somehub", "127.0.0.1:9100", "Connection refused")
                .expect("call succeeds");
            let learnings = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            // No PL-XXX entry should have been written.
            assert!(!learnings.contains("PL-001"), "connection-refused must not create a learning: {learnings}");
            assert!(!tmp.join(".context/working/.fleet-learning-somehub").exists(),
                "no marker should exist for unrelated errors");
        });
    }

    #[test]
    fn fleet_learning_dedupes_within_24h_same_fingerprint() {
        with_framework_cwd(|tmp| {
            let err_msg = "Token validation failed: invalid signature";
            // First record
            maybe_record_auth_mismatch_learning("ring20-management", "10.0.0.1:9100", err_msg)
                .expect("first record");
            let after_first = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            let count_first = after_first.matches("- id: PL-").count();

            // Second call with same inputs — fingerprint unchanged (both "unknown"),
            // marker <24h old → should dedupe.
            maybe_record_auth_mismatch_learning("ring20-management", "10.0.0.1:9100", err_msg)
                .expect("second record");
            let after_second = std::fs::read_to_string(tmp.join(".context/project/learnings.yaml"))
                .expect("read learnings");
            let count_second = after_second.matches("- id: PL-").count();
            assert_eq!(
                count_first, count_second,
                "duplicate call within 24h must not add a second entry: first={count_first} second={count_second}",
            );
        });
    }

    // -------------------------------------------------------------------
    // T-1053: fleet-doctor concern auto-registration
    // -------------------------------------------------------------------

    #[test]
    fn parse_iso8601_utc_roundtrips_now() {
        let s = utc_iso8601_now();
        let secs = parse_iso8601_utc(&s).expect("parse our own output");
        // now_unix_secs() vs parsed should be within a few seconds.
        let now = now_unix_secs();
        let delta = secs.abs_diff(now);
        assert!(delta < 3, "parse roundtrip off by {delta}s: got {secs} vs now {now} (input {s})");
    }

    #[test]
    fn parse_iso8601_utc_rejects_malformed() {
        assert!(parse_iso8601_utc("").is_none());
        assert!(parse_iso8601_utc("2026-04-14").is_none(), "date-only must fail");
        assert!(parse_iso8601_utc("2026-13-01T00:00:00Z").is_none(), "month > 12 must fail");
        assert!(parse_iso8601_utc("2026-04-14T25:00:00Z").is_none(), "hour > 23 must fail");
        assert!(parse_iso8601_utc("not-a-date").is_none());
    }

    #[test]
    fn fleet_concern_failure_increments_counter() {
        with_framework_cwd(|tmp| {
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("first failure");
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("second failure");

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(tmp.join(".context/working/.fleet-failure-state.json"))
                    .expect("state exists"),
            ).expect("state parses");

            let hub = &state["hubs"]["ring20-management"];
            assert_eq!(hub["consecutive_failures"].as_u64(), Some(2));
            assert!(hub["first_failure_at"].is_string(), "first_failure_at should be set");
            assert_eq!(hub["last_class"].as_str(), Some("auth-mismatch"));
            assert_eq!(hub["concern_registered"].as_bool(), Some(false),
                "2 failures, no age → no concern yet");
        });
    }

    #[test]
    fn fleet_concern_success_resets_counter() {
        with_framework_cwd(|tmp| {
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("failure");
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("failure");
            // Pass → reset
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", None).expect("pass");

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(tmp.join(".context/working/.fleet-failure-state.json"))
                    .expect("state exists"),
            ).expect("state parses");
            let hub = &state["hubs"]["ring20-management"];
            assert_eq!(hub["consecutive_failures"].as_u64(), Some(0));
            assert!(hub["first_failure_at"].is_null(), "first_failure_at cleared on success");
            assert_eq!(hub["concern_registered"].as_bool(), Some(false));
        });
    }

    #[test]
    fn fleet_concern_fresh_failures_do_not_register() {
        with_framework_cwd(|tmp| {
            // 3 failures in quick succession — threshold count met but age <24h.
            for _ in 0..5 {
                maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                    .expect("failure");
            }
            let concerns = std::fs::read_to_string(tmp.join(".context/project/concerns.yaml"))
                .expect("read concerns");
            assert!(
                !concerns.contains("ring20-management"),
                "must not register concern for hub failing <24h",
            );

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(tmp.join(".context/working/.fleet-failure-state.json"))
                    .expect("state exists"),
            ).expect("state parses");
            let hub = &state["hubs"]["ring20-management"];
            assert_eq!(hub["consecutive_failures"].as_u64(), Some(5));
            assert_eq!(hub["concern_registered"].as_bool(), Some(false));
        });
    }

    #[test]
    fn fleet_concern_registers_when_aged_past_threshold() {
        with_framework_cwd(|tmp| {
            // Manually seed state with first_failure_at > 24h ago.
            let long_ago = now_unix_secs().saturating_sub(86_400 * 2); // 2 days ago
            // Build an ISO-8601 by round-tripping via our formatter indirectly —
            // since we control the format, we can manually construct a valid one.
            // Easiest: fabricate a known date-string that we know parses.
            let long_ago_iso = {
                // Simple: reuse now() then rewrite the year back — but that's fragile.
                // Use a fixed known-old string instead.
                "2026-01-01T00:00:00Z"
            };
            let seed = serde_json::json!({
                "hubs": {
                    "ring20-management": {
                        "consecutive_failures": 2,
                        "first_failure_at": long_ago_iso,
                        "last_failure_at": long_ago_iso,
                        "last_class": "auth-mismatch",
                        "concern_registered": false,
                    }
                }
            });
            let state_path = tmp.join(".context/working/.fleet-failure-state.json");
            std::fs::write(&state_path, serde_json::to_string_pretty(&seed).unwrap())
                .expect("seed state");
            let _ = long_ago; // suppress unused warning when asserts below don't need it

            // One more failure should push count to 3 and age past 24h → concern.
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("threshold-breaking failure");

            let concerns = std::fs::read_to_string(tmp.join(".context/project/concerns.yaml"))
                .expect("read concerns");
            assert!(
                concerns.contains("ring20-management"),
                "concern must be registered: {concerns}",
            );
            assert!(concerns.contains("type: gap"), "concern must be gap-typed");
            assert!(concerns.contains("severity: high"), "concern must be high severity");
            assert!(concerns.contains("status: watching"), "concern must start in watching");
            assert!(concerns.contains("T-1053"), "spec_reference must mention T-1053");

            let state: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&state_path).expect("state exists"),
            ).expect("state parses");
            assert_eq!(
                state["hubs"]["ring20-management"]["concern_registered"].as_bool(),
                Some(true),
                "state must flag concern_registered after write",
            );

            // Subsequent failure must NOT write a second concern.
            let before_second = concerns.matches("ring20-management").count();
            maybe_track_fleet_failure("ring20-management", "10.0.0.1:9100", Some("auth-mismatch"))
                .expect("subsequent failure");
            let after_second = std::fs::read_to_string(tmp.join(".context/project/concerns.yaml"))
                .expect("read concerns").matches("ring20-management").count();
            assert_eq!(before_second, after_second, "dedupe: must not add a second concern");
        });
    }

    // -------------------------------------------------------------------
    // T-1054: fleet reauth — heal incantation renderer
    // -------------------------------------------------------------------

    #[test]
    fn fleet_reauth_render_with_secret_file_includes_expected_sections() {
        let entry = crate::config::HubEntry {
            address: "192.168.10.109:9100".to_string(),
            secret_file: Some("/root/.termlink/secrets/192.168.10.109.hex".to_string()),
            secret: None,
            scope: Some("execute".to_string()),
        };
        let out = render_fleet_reauth_plan("ring20-management", &entry);

        // Header carries profile name
        assert!(out.contains("ring20-management"), "profile name missing: {out}");
        // Address visible
        assert!(out.contains("192.168.10.109:9100"), "address missing: {out}");
        // Secret source line
        assert!(out.contains("file → /root/.termlink/secrets/192.168.10.109.hex"),
            "secret file path missing: {out}");
        // R2 compliance: trust anchor must be explicitly out-of-band
        assert!(out.contains("OUT-OF-BAND"), "trust anchor warning missing: {out}");
        assert!(out.contains("T-1055"), "forward pointer to bootstrap variant missing: {out}");
        // SSH read uses just the hostname, not the full host:port
        assert!(out.contains("ssh 192.168.10.109 -- sudo cat /var/lib/termlink/hub.secret"),
            "ssh read command missing or malformed: {out}");
        // Local write uses the full secret_file path
        assert!(out.contains("echo \"<paste-the-hex-from-step-1>\" > /root/.termlink/secrets/192.168.10.109.hex"),
            "local write step missing: {out}");
        // chmod 600 appears
        assert!(out.contains("chmod 600"), "chmod 600 missing: {out}");
        // Verify step points to fleet doctor
        assert!(out.contains("termlink fleet doctor"), "verify command missing: {out}");
    }

    #[test]
    fn fleet_reauth_render_with_inline_secret_warns() {
        let entry = crate::config::HubEntry {
            address: "10.0.0.5:9100".to_string(),
            secret_file: None,
            secret: Some("aa".repeat(32)),
            scope: None,
        };
        let out = render_fleet_reauth_plan("inline-hub", &entry);
        assert!(out.contains("inline in hubs.toml"), "inline-source label missing: {out}");
        assert!(out.contains("WARNING"), "inline-secret warning missing: {out}");
        assert!(out.contains("[hubs.inline-hub]"), "toml edit example missing: {out}");
    }

    #[test]
    fn fleet_reauth_render_with_no_secret_flags_missing() {
        let entry = crate::config::HubEntry {
            address: "10.0.0.9:9100".to_string(),
            secret_file: None,
            secret: None,
            scope: None,
        };
        let out = render_fleet_reauth_plan("broken-hub", &entry);
        assert!(out.contains("NONE configured"), "missing-secret warning missing: {out}");
    }

    #[test]
    fn fleet_reauth_errors_on_unknown_profile() {
        // Isolate HOME to a tempdir with a hubs.toml containing one unrelated profile.
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1054-unknown-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".termlink")).expect("create .termlink");
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            r#"
[hubs.other]
address = "10.0.0.1:9100"
secret_file = "/tmp/other.hex"
"#,
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("does-not-exist", None);

        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("must error on unknown profile");
        let msg = format!("{err:#}");
        assert!(msg.contains("Unknown hub profile"), "error message shape: {msg}");
        assert!(msg.contains("does-not-exist"), "error must name the bad profile: {msg}");
        assert!(msg.contains("other"), "error must list known profiles: {msg}");
    }

    // -------------------------------------------------------------------
    // T-1055: fleet reauth --bootstrap-from <SOURCE>
    // -------------------------------------------------------------------

    #[test]
    fn fleet_reauth_hex_validator_accepts_valid_secret() {
        let hex = "0".repeat(64);
        assert_eq!(normalize_and_validate_secret_hex(&hex).unwrap(), hex);

        // Trimming whitespace is part of the contract (files often end with \n).
        let with_ws = format!("  {hex}\n");
        assert_eq!(normalize_and_validate_secret_hex(&with_ws).unwrap(), hex);

        // Uppercase is fine.
        let upper = "ABCDEF0123456789".repeat(4);
        assert_eq!(normalize_and_validate_secret_hex(&upper).unwrap(), upper);
    }

    #[test]
    fn fleet_reauth_hex_validator_rejects_wrong_length() {
        let short = "abcd";
        let err = normalize_and_validate_secret_hex(short).expect_err("short must error");
        assert!(format!("{err}").contains("expected 64 hex characters"), "{err}");

        let long = "a".repeat(100);
        let err = normalize_and_validate_secret_hex(&long).expect_err("long must error");
        assert!(format!("{err}").contains("expected 64 hex characters"), "{err}");
    }

    #[test]
    fn fleet_reauth_hex_validator_rejects_non_hex() {
        let bad = "z".repeat(64);
        let err = normalize_and_validate_secret_hex(&bad).expect_err("non-hex must error");
        assert!(format!("{err}").contains("non-hex characters"), "{err}");
    }

    #[test]
    fn fleet_reauth_bootstrap_unknown_prefix_errors() {
        let err = fetch_bootstrap_secret("random-junk").expect_err("unknown prefix must error");
        let msg = format!("{err:#}");
        assert!(msg.contains("Unknown bootstrap source"), "{msg}");
        assert!(msg.contains("file:"), "help text must mention file: form");
        assert!(msg.contains("ssh:"), "help text must mention ssh: form");
    }

    #[test]
    fn fleet_reauth_bootstrap_empty_prefixes_error() {
        let err = fetch_bootstrap_secret("file:").expect_err("file: alone must error");
        assert!(format!("{err:#}").contains("file: source requires a path"));
        let err = fetch_bootstrap_secret("ssh:").expect_err("ssh: alone must error");
        assert!(format!("{err:#}").contains("ssh: source requires a host"));
    }

    #[test]
    fn fleet_reauth_bootstrap_file_source_happy_path() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1055-file-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");
        std::fs::create_dir_all(tmp.join(".termlink/secrets")).expect("secrets dir");

        // Seed a bootstrap file + an existing stale secret_file.
        let new_secret = "ab".repeat(32);
        let bootstrap_path = tmp.join("new-secret.hex");
        std::fs::write(&bootstrap_path, format!("{new_secret}\n")).expect("seed bootstrap file");

        let secret_path = tmp.join(".termlink/secrets/192.168.10.109.hex");
        std::fs::write(&secret_path, "cd".repeat(32)).expect("seed stale secret");

        // Seed hubs.toml referencing the stale secret.
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            format!(
                "[hubs.ring20]\naddress = \"192.168.10.109:9100\"\nsecret_file = \"{}\"\n",
                secret_path.display(),
            ),
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let source = format!("file:{}", bootstrap_path.display());
        let result = cmd_fleet_reauth("ring20", Some(&source));

        // Capture state before restoring env.
        let written = std::fs::read_to_string(&secret_path).ok();
        let backup = std::fs::read_to_string(secret_path.with_extension("hex.bak")).ok();

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        result.expect("heal must succeed");
        assert_eq!(written.as_deref(), Some(new_secret.as_str()),
            "secret_file must contain the new secret");
        assert_eq!(backup.as_deref(), Some("cd".repeat(32).as_str()),
            ".bak must contain the prior secret");
    }

    #[test]
    fn fleet_reauth_bootstrap_rejects_invalid_hex_from_file() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1055-badhex-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");

        let bootstrap_path = tmp.join("bad.hex");
        std::fs::write(&bootstrap_path, "not a hex secret").expect("seed");

        let secret_path = tmp.join("target.hex");
        std::fs::write(&secret_path, "00".repeat(32)).expect("seed target");

        std::fs::create_dir_all(tmp.join(".termlink")).expect(".termlink");
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            format!(
                "[hubs.bad]\naddress = \"h:1\"\nsecret_file = \"{}\"\n",
                secret_path.display(),
            ),
        ).expect("seed hubs.toml");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let source = format!("file:{}", bootstrap_path.display());
        let result = cmd_fleet_reauth("bad", Some(&source));

        // Capture pre-restore state.
        let target_content_after = std::fs::read_to_string(&secret_path).ok();

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("invalid hex must error");
        let msg = format!("{err:#}");
        assert!(msg.contains("not valid 64-char hex"), "err shape: {msg}");
        // The existing file must NOT have been overwritten.
        assert_eq!(target_content_after.as_deref(), Some("00".repeat(32).as_str()),
            "target file must be untouched when bootstrap source is invalid");
    }

    #[test]
    fn fleet_reauth_bootstrap_refuses_inline_secret_profile() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1055-inline-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(tmp.join(".termlink")).expect("create .termlink");
        std::fs::write(
            tmp.join(".termlink/hubs.toml"),
            "[hubs.inline]\naddress = \"h:1\"\nsecret = \"aa\"\n",
        ).expect("seed");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: guarded by CWD_LOCK.
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("inline", Some("file:/dev/null"));

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("inline-secret profile must refuse --bootstrap-from");
        let msg = format!("{err:#}");
        assert!(msg.contains("inline secret"), "{msg}");
        assert!(msg.contains("Migrate first"), "must give actionable migration hint: {msg}");
    }

    #[test]
    fn fleet_reauth_errors_on_empty_hubs_config() {
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1054-empty-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");

        let prev_home = std::env::var_os("HOME");
        // SAFETY: single-threaded test region (guarded by CWD_LOCK).
        unsafe { std::env::set_var("HOME", &tmp) };

        let result = cmd_fleet_reauth("anything", None);

        unsafe {
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);

        let err = result.expect_err("must error on empty hubs config");
        let msg = format!("{err:#}");
        assert!(msg.contains("No hubs configured"), "error shape: {msg}");
        assert!(msg.contains("termlink profile add"), "error must suggest profile add: {msg}");
    }

    #[test]
    fn fleet_learning_no_op_outside_framework_project() {
        // Run in a fresh tempdir with NO .context/ present — must silently succeed.
        let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // Anchor CWD against a leaked `tempfile::tempdir` from an unrelated test.
        std::env::set_current_dir("/").expect("cd to /");

        let tmp = std::env::temp_dir().join(format!(
            "termlink-t1052-noframework-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&tmp).expect("create tmp");
        std::env::set_current_dir(&tmp).expect("cd into tmp");

        let result = maybe_record_auth_mismatch_learning(
            "somehub",
            "127.0.0.1:9100",
            "Token validation failed: invalid signature",
        );

        std::env::set_current_dir("/").expect("restore cwd to /");
        let _ = std::fs::remove_dir_all(&tmp);

        assert!(result.is_ok(), "must be best-effort outside framework projects: {result:?}");
    }
}

