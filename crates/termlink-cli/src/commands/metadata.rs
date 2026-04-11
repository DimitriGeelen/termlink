use anyhow::{Context, Result};

use termlink_session::manager;

use crate::cli::KvAction;
use crate::util::truncate;

/// Options for session tag/name/role updates.
pub(crate) struct TagOpts {
    pub set: Vec<String>,
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub new_name: Option<String>,
    pub role: Vec<String>,
    pub add_role: Vec<String>,
    pub remove_role: Vec<String>,
}

/// Options for session discovery filtering.
pub(crate) struct DiscoverOpts {
    pub tags: Vec<String>,
    pub roles: Vec<String>,
    pub caps: Vec<String>,
    pub name: Option<String>,
    pub wait: bool,
    pub wait_timeout: u64,
    pub id: bool,
}

pub(crate) async fn cmd_tag(
    tgt: &crate::target::TargetOpts,
    opts: TagOpts,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let TagOpts { set, add, remove, new_name, role, add_role, remove_role } = opts;
    let target = tgt.session.as_str();

    // Read-only mode: show current state when no modification flags given
    let has_tag_changes = !set.is_empty() || !add.is_empty() || !remove.is_empty();
    let has_role_changes = !role.is_empty() || !add_role.is_empty() || !remove_role.is_empty();
    let has_any_changes = has_tag_changes || has_role_changes || new_name.is_some();
    if !has_any_changes {
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let call_future = crate::target::call_session(tgt, "termlink.ping", serde_json::json!({}));
        let outcome: Result<serde_json::Value> =
            match tokio::time::timeout(timeout_dur, call_future).await {
                Ok(r) => r,
                Err(_) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Timed out after {}s", timeout_secs)}));
                    }
                    anyhow::bail!("Tag query timed out after {}s", timeout_secs);
                }
            };
        match outcome {
            Ok(result) => {
                if json {
                    println!("{}", serde_json::json!({
                        "ok": true,
                        "target": target,
                        "display_name": result["display_name"],
                        "tags": result["tags"],
                        "roles": result["roles"],
                    }));
                } else {
                    let tags = result["tags"]
                        .as_array()
                        .map(|a| a.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default();
                    let roles = result["roles"]
                        .as_array()
                        .map(|a| a.iter().filter_map(|r| r.as_str()).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default();
                    let name = result["display_name"].as_str().unwrap_or(target);
                    let mut parts = Vec::new();
                    if !tags.is_empty() {
                        parts.push(format!("tags=[{}]", tags));
                    }
                    if !roles.is_empty() {
                        parts.push(format!("roles=[{}]", roles));
                    }
                    if parts.is_empty() {
                        println!("{}: (no tags or roles)", name);
                    } else {
                        println!("{}: {}", name, parts.join(", "));
                    }
                }
                return Ok(());
            }
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                }
                anyhow::bail!("Failed to query tags: {}", e);
            }
        }
    }

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
    if let Some(ref name) = new_name {
        params["display_name"] = serde_json::json!(name);
    }
    if !role.is_empty() {
        params["roles"] = serde_json::json!(role);
    }
    if !add_role.is_empty() {
        params["add_roles"] = serde_json::json!(add_role);
    }
    if !remove_role.is_empty() {
        params["remove_roles"] = serde_json::json!(remove_role);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let call_future = crate::target::call_session(tgt, "session.update", params);
    let outcome: Result<serde_json::Value> =
        match tokio::time::timeout(timeout_dur, call_future).await {
            Ok(r) => r,
            Err(_) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Tag update timed out after {}s", timeout_secs)}));
                }
                anyhow::bail!("Tag update timed out after {}s", timeout_secs);
            }
        };

    match outcome {
        Ok(result) => {
            if json {
                let mut wrapped = serde_json::json!({"ok": true});
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", serde_json::to_string_pretty(&wrapped)?);
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
                let roles = result["roles"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|r| r.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                let name = result["display_name"].as_str().unwrap_or(target);
                let mut parts = vec![format!("tags=[{}]", tags)];
                if !roles.is_empty() {
                    parts.push(format!("roles=[{}]", roles));
                }
                println!("Updated {}: {}", name, parts.join(", "));
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Tag update failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_discover(
    opts: DiscoverOpts,
    display: &super::ListDisplayOpts,
) -> Result<()> {
    let DiscoverOpts { tags, roles, caps, name, wait, wait_timeout, id } = opts;
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
                    if display.json {
                        super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to discover sessions: {}", e)}));
                    }
                    return Err(e).context("Failed to discover sessions");
                }
            };
            let result = do_filter(sessions);
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
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => {
                if display.json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to discover sessions: {}", e)}));
                }
                return Err(e).context("Failed to discover sessions");
            }
        };
        do_filter(sessions)
    };

    if display.count {
        if display.json {
            println!("{}", serde_json::json!({"ok": true, "count": filtered.len()}));
        } else {
            println!("{}", filtered.len());
        }
        return Ok(());
    }

    if display.names {
        for s in &filtered {
            println!("{}", s.display_name);
        }
        return Ok(());
    }

    if display.ids {
        for s in &filtered {
            println!("{}", s.id.as_str());
        }
        return Ok(());
    }

    if display.first {
        if let Some(s) = filtered.first() {
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
            } else if id {
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

    if display.json {
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
        println!("{}", serde_json::json!({"ok": true, "sessions": items}));
        return Ok(());
    }

    if filtered.is_empty() {
        if !display.no_header {
            if has_filters {
                println!("No sessions match the specified filters.");
            } else {
                println!("No sessions discovered.");
            }
        }
        return Ok(());
    }

    if !display.no_header {
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

    if !display.no_header {
        println!();
        println!("{} session(s) discovered", filtered.len());
    }
    Ok(())
}

pub(crate) async fn cmd_kv(
    tgt: &crate::target::TargetOpts,
    action: KvAction,
    json: bool,
    raw: bool,
    keys: bool,
    timeout_secs: u64,
) -> Result<()> {
    let target = tgt.session.as_str();
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    // Small local helper: call_session + timeout + consistent json-error
    // exit. Keeps every branch below to a single line.
    async fn call(
        tgt: &crate::target::TargetOpts,
        method: &str,
        params: serde_json::Value,
        timeout_dur: std::time::Duration,
        json: bool,
        target: &str,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        match tokio::time::timeout(timeout_dur, crate::target::call_session(tgt, method, params)).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => {
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false, "target": target, "error": format!("{method} failed: {e}")
                    }));
                }
                Err(e.context(format!("{method} failed")))
            }
            Err(_) => {
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false, "target": target,
                        "error": format!("{method} timed out after {timeout_secs}s")
                    }));
                }
                anyhow::bail!("{method} timed out after {timeout_secs}s")
            }
        }
    }

    // Helper to wrap success result as {"ok": true, ...fields} for JSON output.
    fn wrap_ok(result: &serde_json::Value) -> serde_json::Value {
        let mut wrapped = serde_json::json!({"ok": true});
        if let Some(obj) = result.as_object() {
            for (k, v) in obj {
                wrapped[k] = v.clone();
            }
        }
        wrapped
    }

    match action {
        KvAction::Set { key, value } => {
            let json_value: serde_json::Value = serde_json::from_str(&value)
                .unwrap_or(serde_json::Value::String(value));
            let result = call(
                tgt,
                "kv.set",
                serde_json::json!({"key": key, "value": json_value}),
                timeout_dur,
                json,
                target,
                timeout_secs,
            )
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&wrap_ok(&result))?);
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
        KvAction::Get { key } => {
            let result = call(
                tgt,
                "kv.get",
                serde_json::json!({"key": key}),
                timeout_dur,
                json,
                target,
                timeout_secs,
            )
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&wrap_ok(&result))?);
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
        KvAction::List => {
            let result = call(
                tgt,
                "kv.list",
                serde_json::json!({}),
                timeout_dur,
                json,
                target,
                timeout_secs,
            )
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&wrap_ok(&result))?);
            } else if keys {
                if let Some(entries) = result["entries"].as_array() {
                    for entry in entries {
                        println!("{}", entry["key"].as_str().unwrap_or("?"));
                    }
                }
            } else if let Some(entries) = result["entries"].as_array() {
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
        KvAction::Del { key } => {
            let result = call(
                tgt,
                "kv.delete",
                serde_json::json!({"key": key}),
                timeout_dur,
                json,
                target,
                timeout_secs,
            )
            .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&wrap_ok(&result))?);
            } else if result["deleted"].as_bool().unwrap_or(false) {
                println!("Deleted '{}'", key);
            } else {
                eprintln!("Key '{}' not found", key);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
