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
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

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
        Ok(result) => result.context("Failed to connect to session")?,
        Err(_) => {
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
            anyhow::bail!("Tag update failed: {}", e);
        }
    }
}

pub(crate) fn cmd_discover(
    tags: Vec<String>,
    roles: Vec<String>,
    caps: Vec<String>,
    name: Option<String>,
    json: bool,
) -> Result<()> {
    let sessions = manager::list_sessions(false)
        .context("Failed to discover sessions")?;

    let has_filters = !tags.is_empty() || !roles.is_empty() || !caps.is_empty() || name.is_some();

    let filtered: Vec<_> = sessions
        .into_iter()
        .filter(|s| {
            tags.iter().all(|t| s.tags.contains(t))
                && roles.iter().all(|r| s.roles.contains(r))
                && caps.iter().all(|c| s.capabilities.contains(c))
                && name.as_ref().is_none_or(|n| {
                    s.display_name.to_lowercase().contains(&n.to_lowercase())
                })
        })
        .collect();

    if json {
        let items: Vec<serde_json::Value> = filtered.iter().map(|s| {
            serde_json::json!({
                "id": s.id.as_str(),
                "display_name": s.display_name,
                "state": s.state.to_string(),
                "pid": s.pid,
                "tags": s.tags,
                "roles": s.roles,
                "capabilities": s.capabilities,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if filtered.is_empty() {
        if has_filters {
            println!("No sessions match the specified filters.");
        } else {
            println!("No sessions discovered.");
        }
        return Ok(());
    }

    println!(
        "{:<14} {:<16} {:<14} {:<20} {:<16} TAGS",
        "ID", "NAME", "STATE", "CAPABILITIES", "ROLES"
    );
    println!("{}", "-".repeat(90));

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

    println!();
    println!("{} session(s) discovered", filtered.len());
    Ok(())
}

pub(crate) async fn cmd_kv(target: &str, action: KvAction, json: bool) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    match action {
        KvAction::Set { key, value } => {
            let json_value: serde_json::Value = serde_json::from_str(&value)
                .unwrap_or(serde_json::Value::String(value));

            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.set",
                serde_json::json!({"key": key, "value": json_value}),
            )
            .await
            .context("Failed to connect to session")?;

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
                Err(e) => anyhow::bail!("kv.set failed: {}", e),
            }
        }
        KvAction::Get { key } => {
            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.get",
                serde_json::json!({"key": key}),
            )
            .await
            .context("Failed to connect to session")?;

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else if result["found"].as_bool().unwrap_or(false) {
                        println!("{}", serde_json::to_string_pretty(&result["value"])?);
                    } else {
                        eprintln!("Key '{}' not found", key);
                        std::process::exit(1);
                    }
                }
                Err(e) => anyhow::bail!("kv.get failed: {}", e),
            }
        }
        KvAction::List => {
            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.list",
                serde_json::json!({}),
            )
            .await
            .context("Failed to connect to session")?;

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
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
                Err(e) => anyhow::bail!("kv.list failed: {}", e),
            }
        }
        KvAction::Del { key } => {
            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.delete",
                serde_json::json!({"key": key}),
            )
            .await
            .context("Failed to connect to session")?;

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
                Err(e) => anyhow::bail!("kv.delete failed: {}", e),
            }
        }
    }

    Ok(())
}
