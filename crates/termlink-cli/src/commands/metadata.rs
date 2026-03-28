use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

use crate::cli::KvAction;
use crate::util::truncate;

pub(crate) async fn cmd_tag(
    target: &str,
    set: Vec<String>,
    add: Vec<String>,
    remove: Vec<String>,
    json: bool,
    timeout_secs: u64,
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

    let mut params = serde_json::json!({});
    if !set.is_empty() {
        params["tags"] = serde_json::json!(set);
    }
    if !add.is_empty() {
        params["add_tags"] = serde_json::json!(add);
    }
    if !remove.is_empty() {
        params["remove_tags"] = serde_json::json!(remove);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "session.update", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                    std::process::exit(1);
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Tag update timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Tag update timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let tags = result["tags"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                println!(
                    "Updated {}: tags=[{}]",
                    result["display_name"].as_str().unwrap_or(target),
                    tags,
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Tag update failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_discover(
    tags: Vec<String>,
    roles: Vec<String>,
    caps: Vec<String>,
    name: Option<String>,
    json: bool,
    count: bool,
    first: bool,
    wait: bool,
    wait_timeout: u64,
    id: bool,
    no_header: bool,
) -> Result<()> {
    let has_filters = !tags.is_empty() || !roles.is_empty() || !caps.is_empty() || name.is_some();

    let do_filter = |sessions: Vec<termlink_session::registration::Registration>| -> Vec<termlink_session::registration::Registration> {
        sessions
            .into_iter()
            .filter(|s| {
                tags.iter().all(|t| s.tags.contains(t))
                    && roles.iter().all(|r| s.roles.contains(r))
                    && caps.iter().all(|c| s.capabilities.contains(c))
                    && name.as_ref().is_none_or(|n| {
                        s.display_name.to_lowercase().contains(&n.to_lowercase())
                    })
            })
            .collect()
    };

    let filtered = if wait {
        let start = std::time::Instant::now();
        let timeout_dur = std::time::Duration::from_secs(wait_timeout);
        loop {
            let sessions = match manager::list_sessions(false) {
                Ok(s) => s,
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "error": format!("Failed to discover sessions: {}", e)}));
                        std::process::exit(1);
                    }
                    return Err(e).context("Failed to discover sessions");
                }
            };
            let result = do_filter(sessions);
            if !result.is_empty() {
                break result;
            }
            if start.elapsed() > timeout_dur {
                if json {
                    println!("{}", serde_json::json!({"ok": false, "error": format!("No matching sessions found within {}s", wait_timeout)}));
                    std::process::exit(1);
                }
                anyhow::bail!("No matching sessions found within {}s", wait_timeout);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    } else {
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => {
                if json {
                    println!("{}", serde_json::json!({"ok": false, "error": format!("Failed to discover sessions: {}", e)}));
                    std::process::exit(1);
                }
                return Err(e).context("Failed to discover sessions");
            }
        };
        do_filter(sessions)
    };

    if count {
        println!("{}", filtered.len());
        return Ok(());
    }

    if first {
        if let Some(s) = filtered.first() {
            if json {
                println!("{}", serde_json::json!({
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
            } else if id {
                println!("{}", s.id.as_str());
            } else {
                println!("{}", s.display_name);
            }
        } else {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": "No matching sessions"}));
            }
            std::process::exit(1);
        }
        return Ok(());
    }

    if json {
        let items: Vec<serde_json::Value> = filtered.iter().map(|s| {
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
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if filtered.is_empty() {
        if !no_header {
            if has_filters {
                println!("No sessions match the specified filters.");
            } else {
                println!("No sessions discovered.");
            }
        }
        return Ok(());
    }

    if !no_header {
        println!(
            "{:<14} {:<16} {:<14} {:<20} {:<16} TAGS",
            "ID", "NAME", "STATE", "CAPABILITIES", "ROLES"
        );
        println!("{}", "-".repeat(90));
    }

    for session in &filtered {
        println!(
            "{:<14} {:<16} {:<14} {:<20} {:<16} {}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            truncate(&session.capabilities.join(","), 19),
            truncate(&session.roles.join(","), 15),
            session.tags.join(","),
        );
    }

    if !no_header {
        println!();
        println!("{} session(s) discovered", filtered.len());
    }
    Ok(())
}

pub(crate) async fn cmd_kv(target: &str, action: KvAction, json: bool, raw: bool, keys: bool, timeout_secs: u64) -> Result<()> {
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
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    match action {
        KvAction::Set { key, value } => {
            let json_value: serde_json::Value = serde_json::from_str(&value)
                .unwrap_or(serde_json::Value::String(value));

            let rpc = client::rpc_call(
                reg.socket_path(),
                "kv.set",
                serde_json::json!({"key": key, "value": json_value}),
            );
            let resp = match tokio::time::timeout(timeout_dur, rpc).await {
                Ok(r) => match r {
                    Ok(v) => v,
                    Err(e) => {
                        if json {
                            println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                            std::process::exit(1);
                        }
                        return Err(e).context("Failed to connect to session");
                    }
                },
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("kv.set timed out after {}s", timeout_secs)}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.set timed out after {}s", timeout_secs);
                }
            };

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        let replaced = result["replaced"].as_bool().unwrap_or(false);
                        println!(
                            "{} {}={}",
                            if replaced { "Updated" } else { "Set" },
                            result["key"].as_str().unwrap_or("?"),
                            serde_json::to_string(&json_value)?,
                        );
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.set failed: {}", e);
                }
            }
        }
        KvAction::Get { key } => {
            let rpc = client::rpc_call(
                reg.socket_path(),
                "kv.get",
                serde_json::json!({"key": key}),
            );
            let resp = match tokio::time::timeout(timeout_dur, rpc).await {
                Ok(r) => match r {
                    Ok(v) => v,
                    Err(e) => {
                        if json {
                            println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                            std::process::exit(1);
                        }
                        return Err(e).context("Failed to connect to session");
                    }
                },
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("kv.get timed out after {}s", timeout_secs)}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.get timed out after {}s", timeout_secs);
                }
            };

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else if result["found"].as_bool().unwrap_or(false) {
                        let value = &result["value"];
                        if raw {
                            if let Some(s) = value.as_str() {
                                println!("{}", s);
                            } else {
                                println!("{}", serde_json::to_string(value)?);
                            }
                        } else {
                            println!("{}", serde_json::to_string_pretty(value)?);
                        }
                    } else {
                        eprintln!("Key '{}' not found", key);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.get failed: {}", e);
                }
            }
        }
        KvAction::List => {
            let rpc = client::rpc_call(
                reg.socket_path(),
                "kv.list",
                serde_json::json!({}),
            );
            let resp = match tokio::time::timeout(timeout_dur, rpc).await {
                Ok(r) => match r {
                    Ok(v) => v,
                    Err(e) => {
                        if json {
                            println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                            std::process::exit(1);
                        }
                        return Err(e).context("Failed to connect to session");
                    }
                },
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("kv.list timed out after {}s", timeout_secs)}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.list timed out after {}s", timeout_secs);
                }
            };

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else if keys {
                        if let Some(entries) = result["entries"].as_array() {
                            for entry in entries {
                                println!("{}", entry["key"].as_str().unwrap_or("?"));
                            }
                        }
                    } else {
                        let entries = result["entries"].as_array();
                        if let Some(entries) = entries {
                            if entries.is_empty() {
                                println!("No key-value pairs.");
                            } else {
                                for entry in entries {
                                    let key = entry["key"].as_str().unwrap_or("?");
                                    let value = &entry["value"];
                                    println!("{}={}", key, serde_json::to_string(value)?);
                                }
                                println!();
                                println!("{} pair(s)", result["count"]);
                            }
                        }
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.list failed: {}", e);
                }
            }
        }
        KvAction::Del { key } => {
            let rpc = client::rpc_call(
                reg.socket_path(),
                "kv.delete",
                serde_json::json!({"key": key}),
            );
            let resp = match tokio::time::timeout(timeout_dur, rpc).await {
                Ok(r) => match r {
                    Ok(v) => v,
                    Err(e) => {
                        if json {
                            println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                            std::process::exit(1);
                        }
                        return Err(e).context("Failed to connect to session");
                    }
                },
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("kv.delete timed out after {}s", timeout_secs)}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.delete timed out after {}s", timeout_secs);
                }
            };

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else if result["deleted"].as_bool().unwrap_or(false) {
                        println!("Deleted '{}'", key);
                    } else {
                        eprintln!("Key '{}' not found", key);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                        std::process::exit(1);
                    }
                    anyhow::bail!("kv.delete failed: {}", e);
                }
            }
        }
    }

    Ok(())
}
