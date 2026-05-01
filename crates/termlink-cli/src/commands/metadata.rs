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

/// T-1299 / T-1297 — `termlink whoami`.
///
/// Reads the local session registry directly (no hub round-trip) so it
/// works whether or not the hub is running. Hub-side `session.whoami`
/// handler exists for cross-host callers (`termlink remote call ...`).
pub(crate) async fn cmd_whoami(
    session_hint: Option<String>,
    name_hint: Option<String>,
    json: bool,
) -> Result<()> {
    let env_hint = std::env::var("TERMLINK_SESSION_ID").ok().filter(|s| !s.is_empty());
    let query = session_hint.or(env_hint).or(name_hint);

    if let Some(q) = query.as_deref() {
        match manager::find_session(q) {
            Ok(reg) => {
                print_whoami_card(&reg, json, None)?;
                return Ok(());
            }
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false,
                        "found": false,
                        "query": q,
                        "error": format!("{e}"),
                        "hint": "Set TERMLINK_SESSION_ID to your session id (visible in `termlink list --json`), or run without --session/--name to list candidates.",
                    }));
                }
                anyhow::bail!(
                    "No session matched '{q}': {e}\n\
                     Hint: set TERMLINK_SESSION_ID=<id> for your session (see `termlink list`), \
                     or run `termlink whoami` without --session/--name to list candidates."
                );
            }
        }
    }

    // T-1303: PID-walk fallback. No flag and no env var → walk our own ancestor
    // chain on Linux and pick the closest registered session that owns one of those PIDs.
    let sessions = manager::list_sessions(false).context("Failed to list sessions")?;
    let ancestors = walk_ancestor_pids(std::process::id());
    for ancestor_pid in &ancestors {
        if let Some(reg) = sessions.iter().find(|s| s.pid == *ancestor_pid) {
            print_whoami_card(reg, json, Some(*ancestor_pid))?;
            return Ok(());
        }
    }

    // No hint — print all candidates so the caller can pick one.
    if sessions.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "ok": false,
                "ambiguous": false,
                "candidates": [],
                "hint": "No live sessions on this hub. Register one with: termlink register --name <name> --shell",
            }))?);
        } else {
            println!("No live sessions on this hub.");
            println!("Register one with: termlink register --name <name> --shell");
        }
        return Ok(());
    }

    if json {
        let cards: Vec<_> = sessions.iter().map(|s| serde_json::json!({
            "id": s.id.as_str(),
            "display_name": s.display_name,
            "state": s.state.to_string(),
            "pid": s.pid,
            "roles": s.roles,
            "tags": s.tags,
            "cwd": s.metadata.cwd,
        })).collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "ambiguous": true,
            "candidates": cards,
            "hint": "Set TERMLINK_SESSION_ID=<id> for your session and rerun, or pass --session <id> / --name <display_name>.",
        }))?);
    } else {
        println!("Multiple candidate sessions on this hub — which one are you?");
        println!();
        for s in &sessions {
            let roles = if s.roles.is_empty() { "-".to_string() } else { s.roles.join(",") };
            println!(
                "  {}  {:<24}  pid={:<7}  roles={}  cwd={}",
                s.id.as_str(),
                truncate(&s.display_name, 24),
                s.pid,
                roles,
                s.metadata.cwd.as_deref().unwrap_or("-"),
            );
        }
        println!();
        println!("Hint: set TERMLINK_SESSION_ID=<id> for your session (paste the id from above)");
        println!("      and rerun `termlink whoami`. Or pass --session <id> / --name <display_name>.");
    }
    Ok(())
}

/// T-1440: build the JSON payload for `termlink whoami --json`. Extracted
/// from `print_whoami_card` so tests can assert wire shape (notably the
/// presence/absence of identity_fingerprint per T-1436 plumbing) without
/// capturing stdout.
fn whoami_card_json(
    reg: &termlink_session::registration::Registration,
    pid_walked_match: Option<u32>,
) -> serde_json::Value {
    let mut card = serde_json::json!({
        "ok": true,
        "session": {
            "id": reg.id.as_str(),
            "display_name": reg.display_name,
            "state": reg.state.to_string(),
            "pid": reg.pid,
            "uid": reg.uid,
            "roles": reg.roles,
            "tags": reg.tags,
            "capabilities": reg.capabilities,
            "cwd": reg.metadata.cwd,
        }
    });
    // T-1440: chat-arc identity_fingerprint (sender_id for signed envelopes).
    // Only emit when present so pre-T-1436 registrations stay key-stable.
    if let Some(fp) = reg.metadata.identity_fingerprint.as_deref() {
        card["session"]["identity_fingerprint"] = serde_json::json!(fp);
    }
    if let Some(p) = pid_walked_match {
        card["resolved_via"] = serde_json::json!("pid_walk");
        card["pid_walk_match"] = serde_json::json!(p);
    }
    card
}

/// Print a whoami identity card. When `pid_walked_match` is `Some(pid)`, annotate
/// the output to show the lookup succeeded via PID-walk (T-1303).
fn print_whoami_card(
    reg: &termlink_session::registration::Registration,
    json: bool,
    pid_walked_match: Option<u32>,
) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&whoami_card_json(reg, pid_walked_match))?);
    } else {
        println!("ID:           {}", reg.id.as_str());
        println!("Display name: {}", reg.display_name);
        println!("State:        {}", reg.state);
        println!("PID:          {}", reg.pid);
        // T-1440: copy-pasteable into `agent contact --target-fp <hex>`.
        if let Some(fp) = reg.metadata.identity_fingerprint.as_deref() {
            println!("Identity FP:  {fp}");
        }
        println!("Roles:        {}", if reg.roles.is_empty() { "(none)".to_string() } else { reg.roles.join(", ") });
        println!("Tags:         {}", if reg.tags.is_empty() { "(none)".to_string() } else { reg.tags.join(", ") });
        println!("Capabilities: {}", if reg.capabilities.is_empty() { "(none)".to_string() } else { reg.capabilities.join(", ") });
        if let Some(cwd) = reg.metadata.cwd.as_deref() {
            println!("Cwd:          {cwd}");
        }
        if let Some(p) = pid_walked_match {
            println!();
            println!("(matched via PID-walk: ancestor pid={p})");
        }
    }
    Ok(())
}

/// Walk the process ancestor chain on Linux by parsing `/proc/<pid>/stat`.
/// Returns the chain starting at `start` and ending at PID 1 (or wherever
/// the walk fails — non-Linux, missing /proc, malformed stat, cycle).
///
/// Used by `cmd_whoami` (T-1303) to find a registered session whose pid is
/// one of our ancestors when the env-var disambiguator is not set.
fn walk_ancestor_pids(start: u32) -> Vec<u32> {
    let mut chain = vec![start];
    let mut current = start;
    // Hard cap to avoid pathological loops if /proc is somehow inconsistent.
    for _ in 0..1024 {
        if current <= 1 {
            break;
        }
        match read_ppid_from_proc(current) {
            Some(ppid) if ppid != current && !chain.contains(&ppid) => {
                chain.push(ppid);
                current = ppid;
            }
            _ => break,
        }
    }
    chain
}

/// Read field 4 (ppid) from `/proc/<pid>/stat`. Returns None on any failure
/// (missing /proc, read error, malformed format).
fn read_ppid_from_proc(pid: u32) -> Option<u32> {
    let raw = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_ppid_from_stat(&raw)
}

/// Parse `/proc/<pid>/stat` content into ppid. The comm field (field 2) is
/// wrapped in parens and may itself contain spaces or parens, so we split on
/// the LAST `)` and resume from there. ppid is field 4 overall (field 2 in
/// the post-`)` slice after state).
fn parse_ppid_from_stat(raw: &str) -> Option<u32> {
    let close = raw.rfind(')')?;
    let after = &raw[close + 1..];
    let parts: Vec<&str> = after.split_whitespace().collect();
    // After `)` the fields are: state, ppid, pgrp, ...
    parts.get(1)?.parse::<u32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ppid_from_stat_simple() {
        // /proc/<pid>/stat layout: pid (comm) state ppid pgrp ...
        let raw = "1234 (bash) S 5678 1234 1234 34816 1234 4194304 ...";
        assert_eq!(parse_ppid_from_stat(raw), Some(5678));
    }

    #[test]
    fn parse_ppid_from_stat_comm_has_paren() {
        // comm field can contain ')'. The right-most ')' is the closing one.
        let raw = "42 (foo) bar) S 99 42 42 0 ...";
        assert_eq!(parse_ppid_from_stat(raw), Some(99));
    }

    #[test]
    fn parse_ppid_from_stat_malformed_returns_none() {
        assert_eq!(parse_ppid_from_stat(""), None);
        assert_eq!(parse_ppid_from_stat("no parens here"), None);
        assert_eq!(parse_ppid_from_stat("1234 (bash) S NOT_A_NUMBER 1234"), None);
    }

    #[test]
    fn walk_ancestor_pids_self_terminates_at_pid1() {
        let chain = walk_ancestor_pids(std::process::id());
        // On Linux this should produce a chain ending at PID 1 (or the
        // outermost reachable ancestor in this namespace).
        assert!(!chain.is_empty(), "chain should always include self");
        assert_eq!(chain[0], std::process::id(), "first entry is self");
        // Non-fatal: in some sandbox environments /proc may not be readable
        // for all ancestors, so we just assert no infinite loop and no dups.
        let mut seen = std::collections::HashSet::new();
        for p in &chain {
            assert!(seen.insert(*p), "no duplicate pids in chain");
        }
    }

    #[test]
    fn walk_ancestor_pids_unknown_pid_returns_just_start() {
        // PID very unlikely to exist
        let chain = walk_ancestor_pids(999_999_999);
        assert_eq!(chain, vec![999_999_999]);
    }

    // T-1440: whoami_card_json surfaces identity_fingerprint when populated
    // (post-T-1436 registrations) and stays key-stable when absent (legacy /
    // pre-T-1436 fleet hosts). Build the Registration via JSON deserialize
    // so we don't have to track every private field — the wire shape is the
    // stable contract.
    fn make_reg(identity_fp: Option<&str>) -> termlink_session::registration::Registration {
        let id_field = identity_fp
            .map(|fp| format!(r#","identity_fingerprint":"{fp}""#))
            .unwrap_or_default();
        let json = format!(
            r#"{{
                "version": 1,
                "id": "tl-test1234",
                "display_name": "test-session",
                "pid": 12345,
                "uid": 0,
                "addr": {{ "type": "unix", "path": "/tmp/test.sock" }},
                "created_at": "2026-05-01T17:00:00Z",
                "heartbeat_at": "2026-05-01T17:00:00Z",
                "state": "ready",
                "capabilities": [],
                "roles": [],
                "tags": [],
                "metadata": {{ "cwd": "/tmp"{id_field} }}
            }}"#
        );
        serde_json::from_str(&json).expect("Registration JSON shape valid in test")
    }

    #[test]
    fn whoami_card_json_with_identity_fp_emits_field() {
        let fp = "d1993c2c3ec44c94";
        let reg = make_reg(Some(fp));
        let card = whoami_card_json(&reg, None);
        let session = card.get("session").and_then(|v| v.as_object()).expect("session present");
        assert_eq!(
            session.get("identity_fingerprint").and_then(|v| v.as_str()),
            Some(fp),
            "identity_fingerprint must appear in JSON when registration has it"
        );
    }

    #[test]
    fn whoami_card_json_without_identity_fp_omits_key() {
        let reg = make_reg(None);
        let card = whoami_card_json(&reg, None);
        let session = card.get("session").and_then(|v| v.as_object()).expect("session present");
        assert!(
            !session.contains_key("identity_fingerprint"),
            "identity_fingerprint key must be omitted on legacy registrations (pre-T-1436)"
        );
    }
}
