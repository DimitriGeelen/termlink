//! T-2045 (T-2020 GO): `termlink agent find-idle` CLI verb.
//!
//! Calls the hub's `agent.find_idle` RPC (T-2045 slice 1) over the local
//! UDS socket and renders the result. Pure read — no state mutation.
//!
//! Local-hub-only by design (per T-2020 inception §5.4 "What's NOT in this
//! primitive"). Cross-hub finding is the orchestrator's job — it walks
//! `hubs.toml` and calls find-idle per hub.
//!
//! T-2078 added `--watch <secs>` continuous monitor (substrate primitive
//! #2 observability arc Slice 1) — periodic re-render of the idle roster
//! with diff scaffolding for future `--notify` / `--log` slices.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use termlink_protocol::control::method;
use termlink_protocol::transport::TransportAddr;
use termlink_session::client;

/// T-2078: per-agent snapshot kept across watch ticks for the diff helper.
/// Only carries what `--notify` / `--log` will need in future slices —
/// agent_id is the BTreeMap key, so it lives outside the snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdleSnapshot {
    pub last_heartbeat_ms: i64,
    pub role: Option<String>,
    pub capabilities: Vec<String>,
}

/// T-2078: change-event kinds for the find-idle diff. Idle is binary
/// (in the result or not) — there is no "transition" because the only
/// state is "currently idle". An agent that goes busy disappears from
/// the result (Removed); an agent that frees up reappears (New). An
/// agent that simply re-heartbeats while still idle produces NO event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IdleChangeKind {
    New,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdleChangeEvent {
    pub agent_id: String,
    pub kind: IdleChangeKind,
    pub snap: IdleSnapshot,
}

/// T-2078: pure helper diffing two idle-roster snapshots into a list of
/// change events. Returns `New` for agents present in `curr` but absent
/// from `prev`, `Removed` for agents present in `prev` but absent from
/// `curr`. Agents present in both produce NO event (re-heartbeat is not
/// a state change — see IdleChangeKind doc). Extracted for unit testing
/// without spinning up a hub.
pub(crate) fn diff_idle_sets(
    prev: &std::collections::BTreeMap<String, IdleSnapshot>,
    curr: &std::collections::BTreeMap<String, IdleSnapshot>,
) -> Vec<IdleChangeEvent> {
    let mut out = Vec::new();
    // New = in curr, not in prev.
    for (agent_id, snap) in curr {
        if !prev.contains_key(agent_id) {
            out.push(IdleChangeEvent {
                agent_id: agent_id.clone(),
                kind: IdleChangeKind::New,
                snap: snap.clone(),
            });
        }
    }
    // Removed = in prev, not in curr. Carry the PRIOR snapshot since the
    // agent's current state is "not idle" (no snapshot available).
    for (agent_id, snap) in prev {
        if !curr.contains_key(agent_id) {
            out.push(IdleChangeEvent {
                agent_id: agent_id.clone(),
                kind: IdleChangeKind::Removed,
                snap: snap.clone(),
            });
        }
    }
    out
}

/// T-2078: parse the JSON-RPC `agent.find_idle` result envelope into a
/// snapshot map (agent_id → IdleSnapshot). Order is lost — BTreeMap is
/// alphabetical, but `--watch` re-renders fresh each tick so the ordering
/// shows up consistently. Extracted as a pure function so the watch loop
/// and the diff path share one parser.
pub(crate) fn parse_idle_result(
    result: &Value,
) -> std::collections::BTreeMap<String, IdleSnapshot> {
    let mut out = std::collections::BTreeMap::new();
    let arr = match result.get("idle").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return out,
    };
    for entry in arr {
        let agent_id = match entry.get("agent_id").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let last_heartbeat_ms = entry
            .get("last_heartbeat_ms")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let role = entry
            .get("role")
            .and_then(|v| v.as_str())
            .map(String::from);
        let capabilities: Vec<String> = entry
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        out.insert(
            agent_id,
            IdleSnapshot {
                last_heartbeat_ms,
                role,
                capabilities,
            },
        );
    }
    out
}

/// T-2078: render the idle roster as a human-readable table. Same shape
/// as the original snapshot output; extracted so both one-shot and watch
/// paths use one renderer.
fn render_idle_table(snap: &std::collections::BTreeMap<String, IdleSnapshot>, now_ms: i64) {
    if snap.is_empty() {
        println!("(no idle agents matching filter)");
        return;
    }
    for (agent_id, s) in snap {
        let age_s = ((now_ms - s.last_heartbeat_ms) / 1000).max(0);
        let role_str = s.role.as_deref().unwrap_or("-");
        let caps_str = if s.capabilities.is_empty() {
            "-".to_string()
        } else {
            s.capabilities.join(",")
        };
        println!(
            "{agent_id}\tage={age_s}s\trole={role_str}\tcapabilities={caps_str}"
        );
    }
}

pub(crate) async fn cmd_agent_find_idle(
    role: Option<&str>,
    capabilities: &[String],
    limit: Option<u32>,
    json_output: bool,
    watch: Option<u64>,
) -> Result<()> {
    let sock_path = termlink_hub::server::hub_socket_path();
    if !sock_path.exists() {
        if json_output {
            println!("{}", json!({"ok": false, "error": "hub not running"}));
            std::process::exit(1);
        }
        return Err(anyhow!(
            "Hub is not running (no socket at {})",
            sock_path.display()
        ));
    }
    let addr = TransportAddr::unix(sock_path);

    // Build the param object once — same shape for one-shot and watch.
    let mut params_template = json!({});
    if let Some(r) = role {
        params_template["role"] = json!(r);
    }
    if !capabilities.is_empty() {
        params_template["capabilities"] = json!(capabilities);
    }
    if let Some(n) = limit {
        params_template["limit"] = json!(n);
    }

    // T-2078: --watch + --json are guarded at the clap layer via
    // conflicts_with — clap rejects before we get here. Belt-and-braces
    // assertion in case the variant is ever constructed by something
    // other than clap parsing.
    if watch.is_some() && json_output {
        anyhow::bail!(
            "--watch and --json are incompatible: --watch streams re-rendered \
             text frames; --json is one-shot. Pick one."
        );
    }

    if let Some(interval_raw) = watch {
        // T-2078: 5..=3600 clamp mirrors claims-summary --watch (T-2041)
        // — the idle roster updates at heartbeat cadence (~30s) so
        // sub-5s polling is pure noise.
        let interval = interval_raw.clamp(5, 3600);
        // T-2078: diff scaffolding for Slice 2 --notify. Prior tick state
        // kept across iterations; None on the first tick = baseline.
        let mut prior_state: Option<std::collections::BTreeMap<String, IdleSnapshot>> = None;
        loop {
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_ms = now_secs as i64 * 1000;
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            println!(
                "# agent find-idle --watch | interval={}s | {}",
                interval, now_str
            );
            let current_state: Option<std::collections::BTreeMap<String, IdleSnapshot>>;
            match client::rpc_call_addr(
                &addr,
                method::AGENT_FIND_IDLE,
                params_template.clone(),
            )
            .await
            {
                Ok(resp) => match client::unwrap_result(resp) {
                    Ok(result) => {
                        let snap = parse_idle_result(&result);
                        render_idle_table(&snap, now_ms);
                        current_state = Some(snap);
                    }
                    Err(e) => {
                        println!("# fetch error (will retry on next tick): {e}");
                        current_state = None;
                    }
                },
                Err(e) => {
                    println!("# fetch error (will retry on next tick): {e}");
                    current_state = None;
                }
            }
            // T-2078: Slice 2 hook anchor — diff against prior_state.
            // No --notify / --log wired yet; the events are computed for
            // future use. Only diff when BOTH prior_state and
            // current_state are Some (skip baseline + skip fetch-fail
            // ticks).
            if let (Some(prev), Some(curr)) = (prior_state.as_ref(), current_state.as_ref()) {
                let _events = diff_idle_sets(prev, curr);
                // Slice 2 (--notify) and Slice 3 (--log) will iterate
                // _events here. Kept silent for now so Slice 1 ships
                // without changing the visible UX beyond the watch
                // re-render.
            }
            if current_state.is_some() {
                prior_state = current_state;
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    }

    let resp = client::rpc_call_addr(&addr, method::AGENT_FIND_IDLE, params_template.clone())
        .await
        .context("agent.find_idle RPC failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for agent.find_idle: {e}"))?;

    let idle: Vec<Value> = result["idle"].as_array().cloned().unwrap_or_default();

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if idle.is_empty() {
        println!("(no idle agents matching filter)");
        return Ok(());
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let snap = parse_idle_result(&result);
    render_idle_table(&snap, now_ms);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(hb_ms: i64, role: Option<&str>, caps: &[&str]) -> IdleSnapshot {
        IdleSnapshot {
            last_heartbeat_ms: hb_ms,
            role: role.map(String::from),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn map(entries: &[(&str, IdleSnapshot)]) -> std::collections::BTreeMap<String, IdleSnapshot> {
        let mut m = std::collections::BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        m
    }

    #[test]
    fn find_idle_watch_diff_detects_new_agents() {
        // alpha was idle, beta is now also idle → 1 New event for beta.
        let prev = map(&[("alpha", snap(1_000, Some("claude-code"), &["rust"]))]);
        let curr = map(&[
            ("alpha", snap(1_500, Some("claude-code"), &["rust"])),
            ("beta", snap(1_400, Some("claude-code"), &["docs"])),
        ]);
        let events = diff_idle_sets(&prev, &curr);
        assert_eq!(events.len(), 1, "only beta is new");
        assert_eq!(events[0].agent_id, "beta");
        assert_eq!(events[0].kind, IdleChangeKind::New);
        assert_eq!(events[0].snap.role.as_deref(), Some("claude-code"));
    }

    #[test]
    fn find_idle_watch_diff_detects_removed_agents() {
        // alpha and beta were idle, now only alpha → 1 Removed event for beta.
        let prev = map(&[
            ("alpha", snap(1_000, Some("claude-code"), &[])),
            ("beta", snap(1_000, Some("claude-code"), &["docs"])),
        ]);
        let curr = map(&[("alpha", snap(1_500, Some("claude-code"), &[]))]);
        let events = diff_idle_sets(&prev, &curr);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].agent_id, "beta");
        assert_eq!(events[0].kind, IdleChangeKind::Removed);
        // Removed event carries the LAST-known prior snapshot so
        // downstream --notify scripts have something to render.
        assert_eq!(events[0].snap.role.as_deref(), Some("claude-code"));
    }

    #[test]
    fn find_idle_watch_diff_re_heartbeat_is_not_an_event() {
        // Same set of agents in both ticks, only heartbeat advances → 0
        // events. Idle is binary: still idle = no state change.
        let prev = map(&[("alpha", snap(1_000, Some("claude-code"), &["rust"]))]);
        let curr = map(&[("alpha", snap(1_500, Some("claude-code"), &["rust"]))]);
        let events = diff_idle_sets(&prev, &curr);
        assert!(events.is_empty(), "re-heartbeat is not an event");
    }

    #[test]
    fn find_idle_watch_diff_handles_both_directions() {
        // Mixed: beta arrived idle, gamma went busy. Expect 1 New + 1 Removed.
        let prev = map(&[
            ("alpha", snap(1_000, None, &[])),
            ("gamma", snap(1_000, Some("claude-code"), &["test"])),
        ]);
        let curr = map(&[
            ("alpha", snap(1_500, None, &[])),
            ("beta", snap(1_500, Some("claude-code"), &[])),
        ]);
        let events = diff_idle_sets(&prev, &curr);
        assert_eq!(events.len(), 2);
        let new_count = events.iter().filter(|e| e.kind == IdleChangeKind::New).count();
        let rm_count = events.iter().filter(|e| e.kind == IdleChangeKind::Removed).count();
        assert_eq!(new_count, 1);
        assert_eq!(rm_count, 1);
    }

    #[test]
    fn find_idle_watch_parses_empty_and_populated_results() {
        // Empty: missing idle array.
        let r = serde_json::json!({"ok": true});
        let s = parse_idle_result(&r);
        assert!(s.is_empty());

        // Populated: full shape.
        let r = serde_json::json!({
            "ok": true,
            "idle": [
                {"agent_id": "a", "last_heartbeat_ms": 100, "role": "claude-code", "capabilities": ["rust", "docs"]},
                {"agent_id": "b", "last_heartbeat_ms": 200}
            ]
        });
        let s = parse_idle_result(&r);
        assert_eq!(s.len(), 2);
        let a = s.get("a").expect("a present");
        assert_eq!(a.role.as_deref(), Some("claude-code"));
        assert_eq!(a.capabilities, vec!["rust".to_string(), "docs".to_string()]);
        let b = s.get("b").expect("b present");
        assert_eq!(b.last_heartbeat_ms, 200);
        assert!(b.role.is_none());
        assert!(b.capabilities.is_empty());
    }
}
