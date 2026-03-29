use anyhow::{Context, Result};

use termlink_session::client;

use termlink_protocol::events::{
    file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
};

use crate::cli::ProfileAction;
use crate::config::{hubs_config_path, load_hubs_config, save_hubs_config, HubEntry};
use crate::util::{generate_request_id, truncate, DEFAULT_CHUNK_SIZE};

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
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
) -> Result<String> {
    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() {
        anyhow::bail!("No session specified and stdin is not a terminal (cannot prompt)");
    }

    let mut rpc_client = connect_remote_hub(hub, secret_file, secret_hex, scope).await?;

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
        anyhow::bail!("No active sessions on {}.", hub);
    }

    if sessions.len() == 1 {
        let name = sessions[0]["display_name"].as_str().unwrap_or("?");
        let id = sessions[0]["id"].as_str().unwrap_or("?");
        eprintln!("Auto-selecting: {} ({})", name, id);
        return Ok(name.to_string());
    }

    eprintln!("Sessions on {}:", hub);
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
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
) -> Result<String> {
    if let Some(s) = session {
        return Ok(s);
    }
    pick_remote_session(hub, secret_file, secret_hex, scope).await
}

pub(crate) fn cmd_remote_profile(action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::Add { name, address, secret_file, secret, scope, json } => {
            if !address.contains(':') {
                if json {
                    println!("{}", serde_json::json!({"ok": false, "error": "Address must be in host:port format (e.g., 192.168.10.107:9100)"}));
                    std::process::exit(1);
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
                    println!("{}", serde_json::json!({"ok": false, "error": format!("Profile '{}' not found", name)}));
                    std::process::exit(1);
                }
                println!("Profile '{}' not found", name);
            }
            Ok(())
        }
    }
}

pub(crate) async fn cmd_remote_ping(
    hub: &str,
    session: Option<&str>,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_ping_inner(hub, session, secret_file, secret_hex, scope, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Timeout after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Timeout after {}s waiting for remote ping", timeout_secs);
        }
    }
}

async fn cmd_remote_ping_inner(
    hub: &str,
    session: Option<&str>,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    json: bool,
) -> Result<()> {
    let start = std::time::Instant::now();
    let mut rpc_client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
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
                            "hub": hub,
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
                            hub,
                            r.result["state"].as_str().unwrap_or("?"),
                            total_ms, auth_ms, rpc_ms,
                        );
                    }
                    Ok(())
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                        format!("Session '{}' not found on {}", target, hub)
                    } else {
                        format!("Ping failed: {} {}", e.error.code, e.error.message)
                    };
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": target, "error": msg}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("{}", msg);
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": target, "error": format!("Ping error: {e}")}));
                        std::process::exit(1);
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
                            "hub": hub,
                            "sessions": count,
                            "total_ms": total_ms as u64,
                            "auth_ms": auth_ms as u64,
                            "discover_ms": discover_ms as u64,
                        }));
                    } else {
                        println!(
                            "PONG from hub {} — {} session(s) — {}ms (auth: {}ms, discover: {}ms)",
                            hub, count, total_ms, auth_ms, discover_ms,
                        );
                    }
                    Ok(())
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = format!("Hub ping failed: {} {}", e.error.code, e.error.message);
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": msg}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("{}", msg);
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Hub ping error: {e}")}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("Hub ping error: {}", e);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_remote_list(
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    name: Option<&str>,
    tags: Option<&str>,
    roles: Option<&str>,
    cap: Option<&str>,
    count: bool,
    first: bool,
    names: bool,
    ids: bool,
    no_header: bool,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_list_inner(hub, secret_file, secret_hex, scope, name, tags, roles, cap, count, first, names, ids, no_header, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Timeout after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Timeout after {}s waiting for remote list", timeout_secs);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_remote_list_inner(
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    name: Option<&str>,
    tags: Option<&str>,
    roles: Option<&str>,
    cap: Option<&str>,
    count: bool,
    first: bool,
    names: bool,
    ids: bool,
    no_header: bool,
    json: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
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

            if first {
                if let Some(s) = sessions.first() {
                    if json {
                        println!("{}", serde_json::to_string_pretty(s)?);
                    } else {
                        println!("{}", s["display_name"].as_str().unwrap_or("?"));
                    }
                } else {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "error": "No matching sessions"}));
                    }
                    std::process::exit(1);
                }
                return Ok(());
            }

            if count {
                if json {
                    println!("{}", serde_json::json!({"ok": true, "count": sessions.len()}));
                } else {
                    println!("{}", sessions.len());
                }
                return Ok(());
            }

            if names {
                for s in sessions {
                    println!("{}", s["display_name"].as_str().unwrap_or("?"));
                }
                return Ok(());
            }

            if ids {
                for s in sessions {
                    println!("{}", s["id"].as_str().unwrap_or("?"));
                }
                return Ok(());
            }

            if json {
                println!("{}", serde_json::json!({"ok": true, "sessions": sessions}));
                return Ok(());
            }

            if sessions.is_empty() {
                if !no_header {
                    println!("No sessions on {}.", hub);
                }
                return Ok(());
            }

            if !no_header {
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

            if !no_header {
                println!();
                println!("{} session(s) on {}", sessions.len(), hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = format!("Discover failed: {} {}", e.error.code, e.error.message);
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Discover error: {e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Discover error: {}", e);
        }
    }
}

pub(crate) async fn cmd_remote_status(
    hub: &str,
    session: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    json: bool,
    short: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_status_inner(hub, session, secret_file, secret_hex, scope, json, short)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Timeout after {}s waiting for remote status", timeout_secs);
        }
    }
}

async fn cmd_remote_status_inner(
    hub: &str,
    session: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    json: bool,
    short: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
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
                println!("{}", serde_json::to_string_pretty(result)?);
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

            println!("Session: {} (on {})", result["id"].as_str().unwrap_or("?"), hub);
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
                format!("Session '{}' not found on {}", session, hub)
            } else {
                format!("Status query failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                println!("{}", serde_json::json!({"ok": false, "session": session, "hub": hub, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "session": session, "hub": hub, "error": format!("Status query error: {e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Status query error: {}", e);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_remote_inject(
    hub: &str,
    session: &str,
    text: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    enter: bool,
    key: Option<&str>,
    delay_ms: u64,
    scope: &str,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_inject_inner(hub, session, text, secret_file, secret_hex, enter, key, delay_ms, scope, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Timeout after {}s waiting for remote inject", timeout_secs);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_remote_inject_inner(
    hub: &str,
    session: &str,
    text: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    enter: bool,
    key: Option<&str>,
    delay_ms: u64,
    scope: &str,
    json: bool,
) -> Result<()> {
    let mut client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
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
                println!("{}", serde_json::to_string_pretty(&r.result)?);
            } else {
                let bytes = r.result["bytes_len"].as_u64().unwrap_or(0);
                println!("Injected {} bytes into '{}' on {}", bytes, session, hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            let msg = if e.error.message.contains("not found") || e.error.message.contains("No route") {
                format!("Session '{}' not found on {}", session, hub)
            } else {
                format!("Inject failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                println!("{}", serde_json::json!({"ok": false, "session": session, "hub": hub, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "session": session, "hub": hub, "error": format!("Inject error: {e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Inject error: {}", e);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_remote_send_file(
    hub: &str,
    session: &str,
    path: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    chunk_size: usize,
    scope: &str,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout_dur, cmd_remote_send_file_inner(hub, session, path, secret_file, secret_hex, chunk_size, scope, json)).await {
        Ok(result) => result,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Timeout after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Timeout after {}s waiting for remote file transfer", timeout_secs);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_remote_send_file_inner(
    hub: &str,
    session: &str,
    path: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    chunk_size: usize,
    scope: &str,
    json: bool,
) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let file_path = std::path::Path::new(path);
    let file_data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": format!("Failed to read file: {path}: {e}")}));
                std::process::exit(1);
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

    let mut client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    eprintln!(
        "Sending '{}' ({} bytes, {} chunks) to '{}' on {}",
        filename, size, total_chunks, session, hub
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
                format!("Session '{}' not found on {}", session, hub)
            } else {
                format!("Failed to emit file.init: {} {}", e.error.code, e.error.message)
            };
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to emit file.init: {e}")}));
                std::process::exit(1);
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
                    println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": msg}));
                    std::process::exit(1);
                }
                anyhow::bail!("{}", msg);
            }
            Err(e) => {
                if json {
                    println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to emit chunk {}/{}: {}", i + 1, total_chunks, e)}));
                    std::process::exit(1);
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
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to emit file.complete: {e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Failed to emit file.complete: {}", e);
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "transfer_id": transfer_id,
            "filename": filename,
            "size": size,
            "chunks": total_chunks,
            "sha256": sha256,
            "hub": hub,
            "session": session,
        }));
    } else {
        eprintln!("Transfer complete. SHA-256: {}", sha256);
        println!("Sent '{}' ({} bytes) to '{}' on {}", filename, size, session, hub);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_remote_events(
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    topic_filter: Option<&str>,
    targets_csv: Option<&str>,
    interval_ms: u64,
    max_count: u64,
    json: bool,
    payload_only: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
            }
            return Err(e).context("Failed to connect to hub");
        }
    };

    let targets: Vec<&str> = targets_csv
        .map(|t| t.split(',').map(|s| s.trim()).collect())
        .unwrap_or_default();

    eprintln!("Watching events on {}. Press Ctrl+C to stop.", hub);
    if let Some(t) = topic_filter {
        eprintln!("  Topic filter: {}", t);
    }
    if !targets.is_empty() {
        eprintln!("  Targets: {}", targets.join(", "));
    }
    eprintln!();

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);
    let mut cursors = serde_json::json!({});
    let mut total_received: u64 = 0;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!();
                eprintln!("Stopped. {} event(s) collected.", total_received);
                break;
            }
            _ = tokio::time::sleep(poll_interval) => {
                let mut params = serde_json::json!({});
                if !targets.is_empty() {
                    params["targets"] = serde_json::json!(targets);
                }
                if let Some(t) = topic_filter {
                    params["topic"] = serde_json::json!(t);
                }
                if !cursors.as_object().is_none_or(|m| m.is_empty()) {
                    params["since"] = cursors.clone();
                }

                match rpc_client.call("event.collect", serde_json::json!("collect"), params).await {
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
                                    println!("{}", serde_json::to_string(event).unwrap_or_default());
                                } else {
                                    let session_name = event["session_name"].as_str().unwrap_or("?");
                                    let seq = event["seq"].as_u64().unwrap_or(0);
                                    let topic = event["topic"].as_str().unwrap_or("?");
                                    let payload = &event["payload"];
                                    let ts = event["timestamp"].as_u64().unwrap_or(0);

                                    if payload.is_null()
                                        || (payload.is_object()
                                            && payload.as_object().unwrap().is_empty())
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_remote_exec(
    hub: &str,
    session: &str,
    command: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
    timeout: u64,
    cwd: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut rpc_client = match connect_remote_hub(hub, secret_file, secret_hex, scope).await {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "hub": hub, "session": session, "error": format!("Failed to connect to hub: {e}")}));
                std::process::exit(1);
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
                println!("{}", serde_json::to_string_pretty(result)?);
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
                format!("Session '{}' not found on {}", session, hub)
            } else {
                format!("Execution failed: {} {}", e.error.code, e.error.message)
            };
            if json {
                println!("{}", serde_json::json!({"ok": false, "session": session, "hub": hub, "error": msg}));
                std::process::exit(1);
            }
            anyhow::bail!("{}", msg);
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "session": session, "hub": hub, "error": format!("Execution error: {e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Execution error: {}", e);
        }
    }
}
