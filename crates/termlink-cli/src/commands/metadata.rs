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
            // T-2663: text mode must not exit(1) silently — a bare exit is
            // indistinguishable from a crash. Mirror the JSON branch + the
            // non-`--first` empty path (below) with an actionable stderr line.
            eprintln!("No matching sessions.");
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
    // T-2735: same precedence as before (session_hint → env → name_hint), but the
    // winner's PROVENANCE is carried forward. Only an inherited env claim gets
    // cross-checked: an explicit --session/--name is the caller stating an
    // intent, not a value they inherited without knowing it.
    let (query, from_env) = match session_hint {
        Some(s) => (Some(s), false),
        None => match env_hint {
            Some(e) => (Some(e), true),
            None => (name_hint, false),
        },
    };

    if let Some(q) = query.as_deref() {
        match manager::find_session(q) {
            Ok(reg) => {
                let all = manager::list_sessions(false).unwrap_or_default();
                let shared = count_shared_identity(&reg, &all);
                let env_check = from_env.then(|| {
                    check_env_claim(
                        reg.id.as_str(),
                        &all,
                        &walk_ancestor_pids(std::process::id()),
                        procfs_available(),
                    )
                });
                print_whoami_card(&reg, json, None, shared, env_check.as_ref())?;
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
            let shared = count_shared_identity(reg, &sessions);
            // No env claim was consumed on this path — the walk IS the answer,
            // so there is nothing to cross-check against.
            print_whoami_card(reg, json, Some(*ancestor_pid), shared, None)?;
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

    // T-2691: distinguish "auto-resolution cannot work here" from "you have
    // several sessions". Reaching this point on a host without procfs is NOT an
    // ambiguity — the PID-ancestor walk is structurally unavailable, so telling the
    // operator to disambiguate is unactionable advice: no choice they make will
    // make the walk succeed next time. Only `TERMLINK_SESSION_ID` / `--session`
    // will.
    let auto_resolution_available = procfs_available();

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
        // Machine-readable so an MCP/script consumer can branch without parsing prose.
        let (auto_resolution, hint) = if auto_resolution_available {
            (
                "attempted",
                "Set TERMLINK_SESSION_ID=<id> for your session and rerun, or pass --session <id> / --name <display_name>.",
            )
        } else {
            (
                "unavailable-no-procfs",
                "This platform has no /proc, so PID-ancestor auto-resolution cannot run (it is Linux-only). \
                 Set TERMLINK_SESSION_ID=<id> for your session, or pass --session <id> / --name <display_name>.",
            )
        };
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "ambiguous": true,
            "auto_resolution": auto_resolution,
            "candidates": cards,
            "hint": hint,
        }))?);
    } else {
        if auto_resolution_available {
            println!("Multiple candidate sessions on this hub — which one are you?");
        } else {
            println!("Cannot auto-resolve which session you are: this platform has no /proc,");
            println!("so the PID-ancestor walk is unavailable (it is Linux-only).");
            println!();
            println!("Candidate sessions on this hub:");
        }
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
        if auto_resolution_available {
            println!("Hint: set TERMLINK_SESSION_ID=<id> for your session (paste the id from above)");
            println!("      and rerun `termlink whoami`. Or pass --session <id> / --name <display_name>.");
        } else {
            // Do NOT suggest "rerun whoami" here — it will report the same thing
            // forever on this platform. Name the setting that actually resolves it.
            println!("Set TERMLINK_SESSION_ID=<id> for your session (paste the id from above) so");
            println!("every later termlink command resolves it, or pass --session <id> / --name <display_name>.");
        }
    }
    Ok(())
}

/// T-1704: count how many OTHER sessions on this hub share this registration's
/// identity_fingerprint. Used by `whoami` to flag the shared-host case
/// (PL-166 / G-056) and nudge operators toward T-1700 `--identity-key`.
/// Pure function — takes the full session list rather than calling
/// `manager::list_sessions` itself so it stays unit-testable. Returns 0
/// when the target has no identity_fingerprint (pre-T-1436 sessions).
fn count_shared_identity(
    target: &termlink_session::registration::Registration,
    sessions: &[termlink_session::registration::Registration],
) -> usize {
    let Some(target_fp) = target.metadata.identity_fingerprint.as_deref() else {
        return 0;
    };
    sessions
        .iter()
        .filter(|s| {
            s.id.as_str() != target.id.as_str()
                && s.metadata.identity_fingerprint.as_deref() == Some(target_fp)
        })
        .count()
}

/// T-1440: build the JSON payload for `termlink whoami --json`. Extracted
/// from `print_whoami_card` so tests can assert wire shape (notably the
/// presence/absence of identity_fingerprint per T-1436 plumbing) without
/// capturing stdout.
fn whoami_card_json(
    reg: &termlink_session::registration::Registration,
    pid_walked_match: Option<u32>,
    shared_identity_count: usize,
    env_check: Option<&EnvClaimCheck>,
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
        // T-1704: surface the shared-identity count alongside the fingerprint
        // so JSON callers can detect the PL-166 case without re-querying.
        card["session"]["identity_shared_with"] =
            serde_json::json!(shared_identity_count);
    }
    // T-1477: surface the auto-resolved from_project so operators see what
    // T-1472's CLI default-injection would stamp on a `channel post` from
    // this session. Same resolver as channel.rs (single source of truth).
    if let Some(cwd) = reg.metadata.cwd.as_deref()
        && let Some(project) = super::channel::resolve_project_name_from(std::path::Path::new(cwd))
    {
        card["posts_as"] = serde_json::json!({ "from_project": project });
    }
    if let Some(p) = pid_walked_match {
        card["resolved_via"] = serde_json::json!("pid_walk");
        card["pid_walk_match"] = serde_json::json!(p);
    }
    // T-2735: name the source when the answer came from an inherited env var,
    // and say so when the ancestor walk disagrees. `env_claim_verified` is
    // deliberately tri-state rather than a bool: "could not check" (no procfs)
    // must not read as "checked and fine", which is the exact conflation
    // T-2691 removed from the ambiguous path.
    if let Some(check) = env_check {
        card["resolved_via"] = serde_json::json!("env");
        match check {
            EnvClaimCheck::Confirmed { ancestor_pid } => {
                card["env_claim_verified"] = serde_json::json!("confirmed");
                card["pid_walk_match"] = serde_json::json!(ancestor_pid);
            }
            EnvClaimCheck::Conflict { walked_id, ancestor_pid } => {
                card["env_claim_verified"] = serde_json::json!("conflict");
                card["env_claim_conflict"] = serde_json::json!({
                    "claimed_id": reg.id.as_str(),
                    "ancestor_owned_by": walked_id,
                    "ancestor_pid": ancestor_pid,
                    "hint": "TERMLINK_SESSION_ID names a session that does not own this process. \
                             It is inherited by every descendant of a spawned shell, so it is \
                             probably stale. Unset it (or set it to the id above) to let the \
                             PID-ancestor walk answer.",
                });
            }
            EnvClaimCheck::NoWalkEvidence => {
                card["env_claim_verified"] = serde_json::json!("unconfirmed");
            }
            EnvClaimCheck::Unavailable => {
                card["env_claim_verified"] = serde_json::json!("unavailable-no-procfs");
            }
        }
    }
    card
}

/// Print a whoami identity card. When `pid_walked_match` is `Some(pid)`, annotate
/// the output to show the lookup succeeded via PID-walk (T-1303). The
/// `shared_identity_count` is the number of OTHER sessions on this hub
/// sharing this registration's identity_fingerprint (T-1704); when >0 the
/// card emits a hint about `--identity-key` (T-1700) to surface the PL-166
/// shared-host case.
fn print_whoami_card(
    reg: &termlink_session::registration::Registration,
    json: bool,
    pid_walked_match: Option<u32>,
    shared_identity_count: usize,
    env_check: Option<&EnvClaimCheck>,
) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&whoami_card_json(reg, pid_walked_match, shared_identity_count, env_check))?);
    } else {
        println!("ID:           {}", reg.id.as_str());
        println!("Display name: {}", reg.display_name);
        println!("State:        {}", reg.state);
        println!("PID:          {}", reg.pid);
        // T-1440: copy-pasteable into `agent contact --target-fp <hex>`.
        if let Some(fp) = reg.metadata.identity_fingerprint.as_deref() {
            println!("Identity FP:  {fp}");
            // T-1704: PL-166 hint — flag when this FP is host-shared.
            if shared_identity_count > 0 {
                let plural = if shared_identity_count == 1 { "" } else { "s" };
                println!(
                    "              \u{21B3} shared with {shared_identity_count} other session{plural} on this hub — see `termlink register --identity-key <PATH>` for per-agent identity (T-1700)"
                );
            }
        }
        println!("Roles:        {}", if reg.roles.is_empty() { "(none)".to_string() } else { reg.roles.join(", ") });
        println!("Tags:         {}", if reg.tags.is_empty() { "(none)".to_string() } else { reg.tags.join(", ") });
        println!("Capabilities: {}", if reg.capabilities.is_empty() { "(none)".to_string() } else { reg.capabilities.join(", ") });
        if let Some(cwd) = reg.metadata.cwd.as_deref() {
            println!("Cwd:          {cwd}");
            // T-1477: from_project that `channel post` auto-injection would
            // stamp from this cwd (T-1472). Omitted when no `.framework.yaml`
            // is reachable up the cwd path.
            if let Some(project) =
                super::channel::resolve_project_name_from(std::path::Path::new(cwd))
            {
                println!("Posts as:     from_project={project}");
            }
        }
        if let Some(p) = pid_walked_match {
            println!();
            println!("(matched via PID-walk: ancestor pid={p})");
        }
        // T-2735: only the genuine disagreement is loud. Confirmed / unconfirmed
        // / no-procfs stay silent in text mode — a line printed on every whoami
        // is how a warning stops being read (PL-219). The tri-state stays
        // available to scripts through --json.
        if let Some(EnvClaimCheck::Conflict { walked_id, ancestor_pid }) = env_check {
            println!();
            println!("WARNING: this identity came from an inherited TERMLINK_SESSION_ID,");
            println!("         and the process-ancestor walk disagrees with it.");
            println!("           claimed (env):        {}", reg.id.as_str());
            println!("           owns this process:    {walked_id} (ancestor pid={ancestor_pid})");
            println!();
            println!("         TERMLINK_SESSION_ID is inherited by every descendant of a");
            println!("         spawned shell, so it is probably stale. Unset it to let the");
            println!("         PID-walk answer, or set it to the id that owns this process.");
        }
    }
    Ok(())
}

/// T-2691: is a procfs available to walk?
///
/// The PID-ancestor fallback below is Linux-only by construction — it parses
/// `/proc/<pid>/stat`. On macOS (a platform README lists as supported, and for
/// which Homebrew is the *recommended* install) there is no `/proc`, so the walk
/// cannot succeed. Before T-2691 that produced a silent wrong answer: the chain
/// collapsed to `[self]`, no session matched, and `whoami` reported "ambiguous —
/// here are all candidates", indistinguishable from a genuine multi-session
/// ambiguity. The remedy the ambiguous path implies (pick one of these) is not the
/// remedy that works (set `TERMLINK_SESSION_ID`), because auto-resolution will
/// never succeed on that platform no matter which session you pick.
///
/// This is a RUNTIME probe rather than `#[cfg(target_os = "linux")]` on purpose:
/// a cfg makes the non-Linux branch unreachable on Linux and therefore impossible
/// to test from a Linux host, which is exactly how the original defect survived.
/// Taking the root as a parameter lets both branches be proven in unit tests.
fn procfs_available_at(proc_root: &str) -> bool {
    std::path::Path::new(proc_root).join("self").join("stat").exists()
}

/// Whether the ancestor walk can work on this host at all.
pub(crate) fn procfs_available() -> bool {
    procfs_available_at("/proc")
}

/// T-2735 — verdict of cross-checking an *inherited* identity claim against the
/// process ancestor chain.
///
/// `TERMLINK_SESSION_ID` is seeded into a spawned session's shell
/// (`session.rs:277`) and is then inherited by every descendant of that shell.
/// So the variable is not evidence that the *current* process belongs to the
/// session it names — only that some ancestor once did. `whoami` consumed it
/// ahead of the T-1303 PID-walk and returned the claimed identity with full
/// confidence, so a stale or foreign value produced a confident wrong answer to
/// the single question the command exists to answer.
///
/// This does NOT change which source wins: the env var still resolves the query
/// exactly where it did before. It makes a disagreement *legible* (Directive #2),
/// because the honest failure here is silence, not refusal — the variable is not
/// a security boundary (abusing it already requires a process inside the
/// session), so escalating to a refusal would cost working setups more than it
/// protects.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EnvClaimCheck {
    /// The claimed session owns one of our ancestors — the two sources agree.
    Confirmed { ancestor_pid: u32 },
    /// The walk found a DIFFERENT registered session owning our chain. The
    /// inherited value is stale or foreign and the answer is about to be wrong.
    Conflict { walked_id: String, ancestor_pid: u32 },
    /// The walk ran and found no registered session anywhere in the chain. This
    /// is the ordinary shape when the shell itself is not registered, so it is
    /// NOT evidence against the claim and must stay quiet (PL-219).
    NoWalkEvidence,
    /// No procfs, so the walk is structurally unavailable (T-2691). Silence here
    /// carries no information and must never be reported as confirmation.
    Unavailable,
}

/// Pure core of the T-2735 cross-check. Takes the ancestor chain and session
/// list as arguments so every branch is testable without spawning a process
/// tree or mutating the environment — the same injection discipline the
/// runtime_dir truth table uses.
pub(crate) fn check_env_claim(
    claimed_id: &str,
    sessions: &[termlink_session::registration::Registration],
    ancestors: &[u32],
    procfs: bool,
) -> EnvClaimCheck {
    if !procfs {
        return EnvClaimCheck::Unavailable;
    }
    // Nearest ancestor first: the closest registered session is the one that
    // actually owns this process, which is the same precedence the T-1303
    // fallback uses when it picks a winner.
    for pid in ancestors {
        if let Some(reg) = sessions.iter().find(|s| s.pid == *pid) {
            return if reg.id.as_str() == claimed_id {
                EnvClaimCheck::Confirmed { ancestor_pid: *pid }
            } else {
                EnvClaimCheck::Conflict {
                    walked_id: reg.id.as_str().to_string(),
                    ancestor_pid: *pid,
                }
            };
        }
    }
    EnvClaimCheck::NoWalkEvidence
}

/// Walk the process ancestor chain on Linux by parsing `/proc/<pid>/stat`.
/// Returns the chain starting at `start` and ending at PID 1 (or wherever
/// the walk fails — non-Linux, missing /proc, malformed stat, cycle).
///
/// Used by `cmd_whoami` (T-1303) to find a registered session whose pid is
/// one of our ancestors when the env-var disambiguator is not set. Callers that
/// report failure to a human must first consult [`procfs_available`] so a
/// platform limitation is not reported as an ambiguity (T-2691).
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

    // === T-2691: procfs availability probe (Directive #4 portability) ===
    //
    // Both branches must be provable from a Linux host. That is precisely why the
    // probe takes a root path instead of being `#[cfg(target_os = "linux")]`: a cfg
    // would make the macOS branch unreachable here and unprovable, which is how the
    // original silent degradation survived unnoticed.

    // Linux-gated ON PURPOSE. This asserts a platform FACT ("/proc exists here"),
    // not behaviour, so it is the one place a cfg is correct — and it must not run
    // on the macOS job added in T-2692, where the assertion is false by design.
    // Caught by check-platform-lock.sh (T-2693) on its first run against this file:
    // the check flagged this very test as a Linux-only dependency, which is exactly
    // the class it exists to surface.
    #[test]
    #[cfg(target_os = "linux")]
    fn procfs_probe_detects_a_real_procfs() {
        assert!(
            procfs_available_at("/proc"),
            "/proc/self/stat should exist on a Linux host"
        );
    }

    // The complement: on a host WITHOUT procfs the probe must say so rather than
    // claim availability. Together these two pin the probe against reality on both
    // platform families; the negative tests below pin it against arbitrary roots.
    #[test]
    #[cfg(not(target_os = "linux"))]
    fn procfs_probe_reports_unavailable_on_non_linux() {
        assert!(
            !procfs_available_at("/proc"),
            "a non-Linux host must probe unavailable so whoami names the limitation"
        );
    }

    #[test]
    fn procfs_probe_reports_unavailable_when_absent() {
        // Simulates macOS: a root with no self/stat underneath it.
        let dir = std::env::temp_dir().join("termlink-t2691-no-procfs");
        let _ = std::fs::create_dir_all(&dir);
        assert!(
            !procfs_available_at(dir.to_str().unwrap()),
            "a directory without self/stat must probe as unavailable"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn procfs_probe_unavailable_for_nonexistent_root() {
        assert!(
            !procfs_available_at("/definitely/not/a/procfs/root"),
            "a missing root must probe as unavailable, not panic"
        );
    }

    #[test]
    fn procfs_probe_requires_self_stat_not_just_the_directory() {
        // A bare existing directory (e.g. someone created /proc on macOS) must not
        // read as a usable procfs — the walk needs <root>/<pid>/stat to parse.
        let dir = std::env::temp_dir().join("termlink-t2691-bare-dir");
        let _ = std::fs::create_dir_all(dir.join("self"));
        assert!(
            !procfs_available_at(dir.to_str().unwrap()),
            "self/ without stat must still probe unavailable"
        );
        let _ = std::fs::remove_dir_all(&dir);
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

    // ── T-2735: inherited TERMLINK_SESSION_ID cross-check ──────────────────
    //
    // The defect these pin: `whoami` consumed the env var ahead of the T-1303
    // PID-walk and reported the claimed identity with full confidence. Because
    // the variable is inherited by every descendant of a spawned shell, a stale
    // value produced a confident WRONG answer to "who am I" — the one question
    // the command exists to answer.

    fn make_reg_with(id: &str, pid: u32) -> termlink_session::registration::Registration {
        let json = format!(
            r#"{{
                "version": 1,
                "id": "{id}",
                "display_name": "s-{id}",
                "pid": {pid},
                "uid": 0,
                "addr": {{ "type": "unix", "path": "/tmp/test.sock" }},
                "created_at": "2026-05-01T17:00:00Z",
                "heartbeat_at": "2026-05-01T17:00:00Z",
                "state": "ready",
                "capabilities": [],
                "roles": [],
                "tags": [],
                "metadata": {{ "cwd": "/tmp" }}
            }}"#
        );
        serde_json::from_str(&json).expect("Registration JSON shape valid in test")
    }

    #[test]
    fn env_claim_conflicting_with_ancestor_walk_is_reported() {
        // The actual defect: the env var names tl-aaaa, but the process we are
        // running in descends from tl-bbbb's shell. Before T-2735 whoami printed
        // tl-aaaa and said nothing.
        let sessions = vec![make_reg_with("tl-aaaa", 1111), make_reg_with("tl-bbbb", 2222)];
        let verdict = check_env_claim("tl-aaaa", &sessions, &[9999, 2222, 1], true);
        assert_eq!(
            verdict,
            EnvClaimCheck::Conflict { walked_id: "tl-bbbb".to_string(), ancestor_pid: 2222 },
            "a claim contradicted by the ancestor chain must be reported, not trusted"
        );
    }

    #[test]
    fn env_claim_owning_the_ancestor_chain_stays_quiet() {
        // PL-219: the overwhelmingly common case is a correct env var. If this
        // path warned, the warning would be ignored by the time it mattered.
        let sessions = vec![make_reg_with("tl-aaaa", 1111)];
        let verdict = check_env_claim("tl-aaaa", &sessions, &[9999, 1111, 1], true);
        assert_eq!(
            verdict,
            EnvClaimCheck::Confirmed { ancestor_pid: 1111 },
            "an env claim that owns the chain is confirmed, not merely un-warned"
        );
    }

    #[test]
    fn no_registered_ancestor_is_not_evidence_against_the_claim() {
        // A shell that is not itself registered is ordinary, not suspicious.
        let sessions = vec![make_reg_with("tl-aaaa", 1111)];
        let verdict = check_env_claim("tl-aaaa", &sessions, &[9999, 8888, 1], true);
        assert_eq!(
            verdict,
            EnvClaimCheck::NoWalkEvidence,
            "absence of a registered ancestor must not be reported as a conflict"
        );
    }

    #[test]
    fn without_procfs_the_check_reports_unavailable_not_confirmed() {
        // T-2691's lesson, applied here: "could not check" must never render as
        // "checked and fine". On macOS the walk cannot run at all, so a silent
        // pass would be a fabricated confirmation.
        let sessions = vec![make_reg_with("tl-aaaa", 1111)];
        let verdict = check_env_claim("tl-aaaa", &sessions, &[9999, 1111, 1], false);
        assert_eq!(
            verdict,
            EnvClaimCheck::Unavailable,
            "no procfs means no verdict — never an implied confirmation"
        );
    }

    #[test]
    fn nearest_registered_ancestor_wins_the_comparison() {
        // Matches the T-1303 fallback's own precedence: the closest registered
        // ancestor is the session that actually owns this process. If the walk
        // preferred a more distant one, a nested session would be misreported.
        let sessions = vec![make_reg_with("tl-outer", 1111), make_reg_with("tl-inner", 2222)];
        let verdict = check_env_claim("tl-outer", &sessions, &[9999, 2222, 1111, 1], true);
        assert_eq!(
            verdict,
            EnvClaimCheck::Conflict { walked_id: "tl-inner".to_string(), ancestor_pid: 2222 },
            "the nearest registered ancestor decides, not the first one registered"
        );
    }

    #[test]
    fn conflict_card_names_both_identities_and_stays_actionable() {
        let reg = make_reg_with("tl-aaaa", 1111);
        let check =
            EnvClaimCheck::Conflict { walked_id: "tl-bbbb".to_string(), ancestor_pid: 2222 };
        let card = whoami_card_json(&reg, None, 0, Some(&check));
        assert_eq!(card["resolved_via"].as_str(), Some("env"));
        assert_eq!(card["env_claim_verified"].as_str(), Some("conflict"));
        let conflict = &card["env_claim_conflict"];
        assert_eq!(conflict["claimed_id"].as_str(), Some("tl-aaaa"));
        assert_eq!(conflict["ancestor_owned_by"].as_str(), Some("tl-bbbb"));
        assert_eq!(conflict["ancestor_pid"].as_u64(), Some(2222));
        assert!(
            conflict["hint"].as_str().is_some_and(|h| h.contains("inherited")),
            "the hint must explain WHY the value is probably stale, not just that it is"
        );
    }

    #[test]
    fn card_omits_env_fields_entirely_when_no_env_claim_was_consumed() {
        // Key stability: an explicit --session or a PID-walk answer must not
        // grow env-verification keys it has no opinion about.
        let reg = make_reg_with("tl-aaaa", 1111);
        let card = whoami_card_json(&reg, None, 0, None);
        assert!(card.get("env_claim_verified").is_none());
        assert!(card.get("env_claim_conflict").is_none());
        assert!(card.get("resolved_via").is_none());
    }

    #[test]
    fn unavailable_verdict_is_distinguishable_from_confirmed_in_json() {
        let reg = make_reg_with("tl-aaaa", 1111);
        let unavailable = whoami_card_json(&reg, None, 0, Some(&EnvClaimCheck::Unavailable));
        let confirmed =
            whoami_card_json(&reg, None, 0, Some(&EnvClaimCheck::Confirmed { ancestor_pid: 1111 }));
        assert_eq!(
            unavailable["env_claim_verified"].as_str(),
            Some("unavailable-no-procfs")
        );
        assert_eq!(confirmed["env_claim_verified"].as_str(), Some("confirmed"));
        assert_ne!(
            unavailable["env_claim_verified"], confirmed["env_claim_verified"],
            "a script must be able to tell 'not checked' from 'checked and fine'"
        );
    }

    #[test]
    fn whoami_card_json_with_identity_fp_emits_field() {
        let fp = "d1993c2c3ec44c94";
        let reg = make_reg(Some(fp));
        let card = whoami_card_json(&reg, None, 0, None);
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
        let card = whoami_card_json(&reg, None, 0, None);
        let session = card.get("session").and_then(|v| v.as_object()).expect("session present");
        assert!(
            !session.contains_key("identity_fingerprint"),
            "identity_fingerprint key must be omitted on legacy registrations (pre-T-1436)"
        );
        assert!(
            !session.contains_key("identity_shared_with"),
            "identity_shared_with key must be omitted when identity_fingerprint is absent"
        );
    }

    // T-1704: build a Registration whose id can be controlled, so
    // count_shared_identity tests can construct a small fleet without
    // bumping into duplicate ids (which would mask the same-FP filter).
    fn make_reg_id(id: &str, identity_fp: Option<&str>) -> termlink_session::registration::Registration {
        let id_field = identity_fp
            .map(|fp| format!(r#","identity_fingerprint":"{fp}""#))
            .unwrap_or_default();
        let json = format!(
            r#"{{
                "version": 1,
                "id": "{id}",
                "display_name": "test-session-{id}",
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
    fn count_shared_identity_zero_when_unique() {
        let fp = "aaaaaaaaaaaaaaaa";
        let target = make_reg_id("tl-alpha0001", Some(fp));
        let other = make_reg_id("tl-beta00002", Some("bbbbbbbbbbbbbbbb"));
        let sessions = vec![target.clone(), other];
        assert_eq!(
            count_shared_identity(&target, &sessions),
            0,
            "unique identity_fingerprint must not count itself or unrelated peers"
        );
    }

    #[test]
    fn count_shared_identity_two_for_host_shared_triple() {
        let host_fp = "d1993c2c3ec44c94";
        let target = make_reg_id("tl-self00001", Some(host_fp));
        let peer_a = make_reg_id("tl-peer00002", Some(host_fp));
        let peer_b = make_reg_id("tl-peer00003", Some(host_fp));
        let unrelated = make_reg_id("tl-other0004", Some("ffffffffffffffff"));
        let sessions = vec![target.clone(), peer_a, peer_b, unrelated];
        assert_eq!(
            count_shared_identity(&target, &sessions),
            2,
            "three sessions share the host FP — count for any one of them should be 2 (others)"
        );
    }

    #[test]
    fn count_shared_identity_zero_when_target_fp_absent() {
        let target = make_reg_id("tl-self00001", None);
        let peer = make_reg_id("tl-peer00002", Some("d1993c2c3ec44c94"));
        let sessions = vec![target.clone(), peer];
        assert_eq!(
            count_shared_identity(&target, &sessions),
            0,
            "pre-T-1436 sessions with no identity_fingerprint must report 0 (no hint emitted)"
        );
    }

    #[test]
    fn whoami_card_json_emits_identity_shared_with_when_fp_present() {
        let fp = "d1993c2c3ec44c94";
        let reg = make_reg(Some(fp));
        let card = whoami_card_json(&reg, None, 4, None);
        let session = card.get("session").and_then(|v| v.as_object()).expect("session present");
        assert_eq!(
            session.get("identity_shared_with").and_then(|v| v.as_u64()),
            Some(4),
            "identity_shared_with must surface the live count for downstream JSON callers"
        );
    }
}
