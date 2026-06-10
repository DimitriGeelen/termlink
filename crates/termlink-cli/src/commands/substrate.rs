//! T-2111: substrate-status — unified one-shot CLI verb composing the four
//! read-side substrate primitives into one situational-awareness view.
//!
//! CLI-tier parity for the `/substrate` slash-command skill (T-2096). Today an
//! operator at a non-claude terminal — or an MCP-invoked agent, or a cron job,
//! or a shell pipeline — has no single command answering "is my substrate
//! healthy right now?". They must run four separate verbs (`agent find-idle`,
//! `channel claims-summary --all`, `channel queue-status`, `fleet
//! governor-status`) and visually correlate. This verb closes that gap.
//!
//! ## What it composes (T-2018 §6 build manifest)
//!
//! | Sub-read | Substrate primitive | Question |
//! |----------|---------------------|----------|
//! | `agent.find_idle` (local hub) | #2 DISPATCH (T-2020/T-2045) | Who's free to take work? |
//! | `channel.claims_summary` per topic (local hub) | #1 CLAIM (T-2019/T-2042) | Any stuck claims? |
//! | `OfflineQueue::open` (local SQLite) | #5 RESILIENCE (T-2051) | Queue draining? |
//! | `hub.governor_status` per hub (fleet) | #10 BACKPRESSURE (T-2048) | Any hub pressured? |
//!
//! ## Design contract
//!
//! - **Parallel by construction.** All four reads dispatch via `tokio::join!`;
//!   total latency ≈ max-of-four not sum-of-four.
//! - **Graceful degradation.** A failed sub-read renders as a `(<SECTION>
//!   unavailable: ...)` line in human mode + `ok:false` in JSON. The other
//!   three sections still render. Local hub down kills DISPATCH+CLAIM but
//!   RESILIENCE (local SQLite) and BACKPRESSURE (fleet-wide) still work.
//! - **`--only-pressured` filter.** Mirrors the underlying sub-verb flags
//!   (`claims-summary --only-stuck`, `governor-status --only-pressured`).
//!   Filters CLAIM + BACKPRESSURE sections; DISPATCH + RESILIENCE pass
//!   through (their `--only-*` analogs don't apply).
//! - **Read-only.** No auth side-effects, no state mutation, no log writes.
//!
//! Future slices (deferred — not in T-2111): `--watch <secs>` continuous
//! monitor, `--notify <cmd>` event hook, `--log <path>` audit, `substrate
//! history` retrospective, MCP parity. Same arc shape as T-2078..T-2087
//! (find-idle) and T-2064..T-2069 (governor).

use anyhow::Result;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use termlink_protocol::control::method;
use termlink_protocol::transport::TransportAddr;

use super::remote::{connect_remote_hub, governor_hub_is_pressured};

// ────────────────────────────────────────────────────────────────────────────
// Result types
// ────────────────────────────────────────────────────────────────────────────

/// Per-section result. `Ok` carries the section's JSON value; `Err` carries
/// the failure reason (already stringified for display). Mirror of
/// `FleetGovernorResult` in `remote.rs` (T-2048).
pub(crate) type SubResult = std::result::Result<Value, String>;

// ────────────────────────────────────────────────────────────────────────────
// Top-level entry
// ────────────────────────────────────────────────────────────────────────────

/// T-2111 Slice 1: one-shot `termlink substrate status` handler.
///
/// `timeout_secs` bounds each per-hub RPC in the BACKPRESSURE sweep and each
/// local-hub RPC in the DISPATCH + CLAIM reads. RESILIENCE is a local SQLite
/// read so the timeout doesn't apply.
pub(crate) async fn cmd_substrate_status(
    json_output: bool,
    only_pressured: bool,
    timeout_secs: u64,
) -> Result<()> {
    let local_sock = termlink_hub::server::hub_socket_path();
    let local_addr: Option<TransportAddr> = if local_sock.exists() {
        Some(TransportAddr::unix(local_sock))
    } else {
        None
    };

    let (dispatch_res, claim_res, resilience_res, backpressure_res) = tokio::join!(
        fetch_dispatch(local_addr.as_ref(), timeout_secs),
        fetch_claim(local_addr.as_ref(), only_pressured, timeout_secs),
        fetch_resilience(),
        fetch_backpressure(only_pressured, timeout_secs),
    );

    let any_failure = [
        &dispatch_res,
        &claim_res,
        &resilience_res,
        &backpressure_res,
    ]
    .iter()
    .any(|r| r.is_err());

    if json_output {
        let envelope = json!({
            "ok": !any_failure,
            "ts": now_rfc3339(),
            "only_pressured": only_pressured,
            "dispatch":     section_json(&dispatch_res),
            "claim":        section_json(&claim_res),
            "resilience":   section_json(&resilience_res),
            "backpressure": section_json(&backpressure_res),
        });
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        let text = render_substrate_text(
            &dispatch_res,
            &claim_res,
            &resilience_res,
            &backpressure_res,
            only_pressured,
        );
        print!("{}", text);
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Sub-fetches
// ────────────────────────────────────────────────────────────────────────────

/// DISPATCH (substrate #2): call `agent.find_idle` on the local hub.
/// Returns the RPC result as-is — same shape as `agent find-idle --json`.
async fn fetch_dispatch(local_addr: Option<&TransportAddr>, timeout_secs: u64) -> SubResult {
    let addr = match local_addr {
        Some(a) => a,
        None => {
            return Err(
                "local hub not running (no socket) — DISPATCH read needs the local hub"
                    .to_string(),
            );
        }
    };
    let timeout_dur = Duration::from_secs(timeout_secs);
    let probe = async {
        let resp =
            termlink_session::client::rpc_call_addr(addr, method::AGENT_FIND_IDLE, json!({}))
                .await
                .map_err(|e| format!("agent.find_idle RPC failed: {e}"))?;
        let result = termlink_session::client::unwrap_result(resp)
            .map_err(|e| format!("hub returned error for agent.find_idle: {e}"))?;
        Ok::<Value, String>(result)
    };
    match tokio::time::timeout(timeout_dur, probe).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!("agent.find_idle timed out after {}s", timeout_secs)),
    }
}

/// CLAIM (substrate #1): enumerate topics via `channel.list`, fan-out
/// `channel.claims_summary` per topic on the local hub. Mirror of
/// `cmd_channel_claims_summary` with `--all --json` (channel.rs T-2042/T-2076).
/// When `only_stuck=true` the topics array drops non-stuck entries but the
/// fleet-wide `topic_count` + `stuck_count` are kept truthful.
async fn fetch_claim(
    local_addr: Option<&TransportAddr>,
    only_stuck: bool,
    timeout_secs: u64,
) -> SubResult {
    let addr = match local_addr {
        Some(a) => a,
        None => {
            return Err(
                "local hub not running (no socket) — CLAIM read needs the local hub".to_string(),
            );
        }
    };
    let timeout_dur = Duration::from_secs(timeout_secs);
    let probe = async {
        // T-2042: enumerate topic names via channel.list.
        let resp = termlink_session::client::rpc_call_addr(
            addr,
            method::CHANNEL_LIST,
            json!({}),
        )
        .await
        .map_err(|e| format!("channel.list RPC failed: {e}"))?;
        let list_result = termlink_session::client::unwrap_result(resp)
            .map_err(|e| format!("hub returned error for channel.list: {e}"))?;
        let topics_raw = list_result["topics"].as_array().cloned().unwrap_or_default();
        let topic_names: Vec<String> = topics_raw
            .iter()
            .filter_map(|t| t["name"].as_str().map(|s| s.to_string()))
            .collect();

        // Per-topic claims_summary fan-out. Errors are non-fatal — they
        // surface as one entry with `ok:false` so the sweep keeps going
        // (matches `render_claims_summary_fleet_json` semantics).
        let mut entries: Vec<Value> = Vec::with_capacity(topic_names.len());
        let mut stuck_count: u64 = 0;
        for t in &topic_names {
            match termlink_session::claim_client::channel_claims_summary(addr, t).await {
                Ok(summary) => {
                    let stuck = is_potentially_stuck(&summary);
                    if stuck {
                        stuck_count += 1;
                    }
                    if only_stuck && !stuck {
                        continue;
                    }
                    entries.push(json!({
                        "ok": true,
                        "topic": summary.topic,
                        "active_count": summary.active_count,
                        "expired_count": summary.expired_count,
                        "oldest_active_at_ms": summary.oldest_active_at_ms,
                        "oldest_active_age_ms": summary.oldest_active_age_ms,
                        "next_active_expiry_ms": summary.next_active_expiry_ms,
                        "potentially_stuck": stuck,
                    }));
                }
                Err(e) => {
                    // Always retained — a fetch error could mask a stuck topic.
                    entries.push(json!({
                        "ok": false,
                        "topic": t,
                        "error": format!("{e}"),
                    }));
                }
            }
        }
        let shown = entries.len();
        Ok::<Value, String>(json!({
            "ok": true,
            "topic_count": topic_names.len(),
            "stuck_count": stuck_count,
            "shown": shown,
            "only_stuck": only_stuck,
            "topics": entries,
        }))
    };
    match tokio::time::timeout(timeout_dur, probe).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!("channel.claims_summary sweep timed out after {}s", timeout_secs)),
    }
}

/// RESILIENCE (substrate #5): read the local offline-queue file via
/// `OfflineQueue::open`. No hub involvement — same path as
/// `cmd_channel_queue_status` (channel.rs T-2051).
async fn fetch_resilience() -> SubResult {
    let path: PathBuf = termlink_session::offline_queue::default_queue_path();
    if !path.exists() {
        return Ok(json!({
            "queue_path": path.display().to_string(),
            "exists": false,
            "pending": 0,
        }));
    }
    let queue = termlink_session::offline_queue::OfflineQueue::open(&path)
        .map_err(|e| format!("failed to open offline queue at {}: {e}", path.display()))?;
    let size = queue
        .size()
        .map_err(|e| format!("failed to read queue size: {e}"))?;
    let head = queue
        .peek_oldest()
        .map_err(|e| format!("failed to peek queue head: {e}"))?;
    let head_json = head.as_ref().map(|(id, post)| {
        json!({
            "queue_id": id.0,
            "topic": post.topic,
            "msg_type": post.msg_type,
            "ts_unix_ms": post.ts_unix_ms,
            "sender_id": post.sender_id,
            "artifact_ref": post.artifact_ref,
        })
    });
    Ok(json!({
        "queue_path": path.display().to_string(),
        "exists": true,
        "cap": queue.cap(),
        "pending": size,
        "oldest": head_json,
    }))
}

/// BACKPRESSURE (substrate #10): walk every hub in `~/.termlink/hubs.toml`
/// and call `hub.governor_status` per hub. Mirror of
/// `cmd_fleet_governor_status` (remote.rs T-2048/T-2070).
async fn fetch_backpressure(only_pressured: bool, timeout_secs: u64) -> SubResult {
    use termlink_protocol::jsonrpc::RpcResponse;

    let config = crate::config::load_hubs_config();
    let mut hub_names: Vec<&String> = config.hubs.keys().collect();
    hub_names.sort();

    if hub_names.is_empty() {
        return Ok(json!({
            "ok": true,
            "total": 0,
            "reachable": 0,
            "hubs": [],
            "summary": {
                "hubs_at_capacity": 0,
                "hubs_rate_limited": 0,
                "shown": 0,
                "only_pressured": only_pressured,
            }
        }));
    }

    let mut results: Vec<(String, std::result::Result<Value, String>)> =
        Vec::with_capacity(hub_names.len());
    for name in &hub_names {
        let entry = &config.hubs[*name];
        let timeout_dur = Duration::from_secs(timeout_secs);
        let probe = async {
            let mut client = connect_remote_hub(
                &entry.address,
                entry.secret_file.as_deref(),
                entry.secret.as_deref(),
                entry.scope.as_deref().unwrap_or("execute"),
            )
            .await?;
            let resp = client
                .call("hub.governor_status", json!("substrate-status"), json!({}))
                .await?;
            match resp {
                RpcResponse::Success(r) => Ok::<Value, anyhow::Error>(r.result),
                RpcResponse::Error(e) => anyhow::bail!("RPC error {}: {}", e.error.code, e.error.message),
            }
        };
        let r = match tokio::time::timeout(timeout_dur, probe).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err(format!("timed out after {}s", timeout_secs)),
        };
        results.push(((*name).clone(), r));
    }

    let shown: Vec<&(String, std::result::Result<Value, String>)> = if only_pressured {
        results
            .iter()
            .filter(|(_, r)| governor_hub_is_pressured(r))
            .collect()
    } else {
        results.iter().collect()
    };

    let mut hubs_json: Vec<Value> = Vec::with_capacity(shown.len());
    for (name, r) in &shown {
        match r {
            Ok(v) => hubs_json.push(json!({
                "hub": name,
                "ok": true,
                "governor": v,
            })),
            Err(e) => hubs_json.push(json!({
                "hub": name,
                "ok": false,
                "error": e,
            })),
        }
    }

    let at_capacity: usize = results
        .iter()
        .filter_map(|(_, r)| r.as_ref().ok())
        .filter(|v| {
            let active = v.get("connections_active").and_then(|x| x.as_i64()).unwrap_or(0);
            let max = v.get("connections_max").and_then(|x| x.as_i64()).unwrap_or(i64::MAX);
            active >= max
        })
        .count();
    let rate_limited: usize = results
        .iter()
        .filter_map(|(_, r)| r.as_ref().ok())
        .filter(|v| v.get("rate_hits_total").and_then(|x| x.as_i64()).unwrap_or(0) > 0)
        .count();
    let reachable = results.iter().filter(|(_, r)| r.is_ok()).count();

    Ok(json!({
        "ok": true,
        "total": results.len(),
        "reachable": reachable,
        "hubs": hubs_json,
        "summary": {
            "hubs_at_capacity": at_capacity,
            "hubs_rate_limited": rate_limited,
            "shown": shown.len(),
            "only_pressured": only_pressured,
        }
    }))
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// T-2042 mirror: a topic is "potentially stuck" if it has any expired
/// claims OR the oldest active claim is older than 60s. Copied to avoid
/// cross-module dependency (the helper in channel.rs is module-private).
fn is_potentially_stuck(summary: &termlink_session::claim_client::ClaimsAggregate) -> bool {
    summary.expired_count > 0
        || summary
            .oldest_active_age_ms
            .map(|age| age > 60_000)
            .unwrap_or(false)
}

/// JSON envelope sub-section shape: wraps each SubResult so failed sections
/// carry `{ok:false, error}` and successful sections pass through with
/// `{ok:true, data}`.
fn section_json(r: &SubResult) -> Value {
    match r {
        Ok(v) => json!({"ok": true, "data": v}),
        Err(e) => json!({"ok": false, "error": e}),
    }
}

fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    crate::manifest::secs_to_rfc3339(secs)
}

// ────────────────────────────────────────────────────────────────────────────
// Text renderer (pure — easy to unit-test)
// ────────────────────────────────────────────────────────────────────────────

/// T-2111 Slice 1: human-format four-section renderer. Pure: takes the four
/// pre-computed SubResults + the `only_pressured` flag and returns the full
/// stdout text. The handler just prints what this returns — no I/O inside
/// the renderer.
pub(crate) fn render_substrate_text(
    dispatch: &SubResult,
    claim: &SubResult,
    resilience: &SubResult,
    backpressure: &SubResult,
    only_pressured: bool,
) -> String {
    let mut out = String::new();
    out.push_str("═══ substrate status ═══\n\n");

    // DISPATCH section
    out.push_str("DISPATCH (substrate #2 — who's free to take work?):\n");
    match dispatch {
        Ok(v) => {
            let idle = v.get("idle").and_then(|x| x.as_array()).cloned().unwrap_or_default();
            if idle.is_empty() {
                out.push_str("  (no idle agents — see `agent find-idle` for diagnostic)\n");
            } else {
                for entry in &idle {
                    let id = entry.get("agent_id").and_then(|x| x.as_str()).unwrap_or("?");
                    let role = entry.get("role").and_then(|x| x.as_str()).unwrap_or("-");
                    let hb = entry
                        .get("last_heartbeat_ms")
                        .and_then(|x| x.as_i64())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    let caps = entry
                        .get("capabilities")
                        .and_then(|x| x.as_str())
                        .unwrap_or("");
                    out.push_str(&format!(
                        "  {}  role={}  last_heartbeat_ms={}  capabilities={}\n",
                        id, role, hb, caps
                    ));
                }
            }
        }
        Err(e) => out.push_str(&format!("  (DISPATCH unavailable: {})\n", e)),
    }
    out.push('\n');

    // CLAIM section
    out.push_str("CLAIM (substrate #1 — any stuck claims?):\n");
    match claim {
        Ok(v) => {
            let topic_count = v.get("topic_count").and_then(|x| x.as_u64()).unwrap_or(0);
            let stuck_count = v.get("stuck_count").and_then(|x| x.as_u64()).unwrap_or(0);
            let topics = v.get("topics").and_then(|x| x.as_array()).cloned().unwrap_or_default();
            if only_pressured && stuck_count == 0 && topic_count > 0 {
                out.push_str(&format!(
                    "  All topics healthy (0/{} stuck)\n",
                    topic_count
                ));
            } else if topics.is_empty() && topic_count == 0 {
                out.push_str("  (no topics on hub)\n");
            } else {
                for t in &topics {
                    let topic = t.get("topic").and_then(|x| x.as_str()).unwrap_or("?");
                    if t.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
                        let active = t.get("active_count").and_then(|x| x.as_u64()).unwrap_or(0);
                        let expired = t.get("expired_count").and_then(|x| x.as_u64()).unwrap_or(0);
                        let age = t
                            .get("oldest_active_age_ms")
                            .and_then(|x| x.as_i64())
                            .map(|a| format!("{}ms", a))
                            .unwrap_or_else(|| "-".to_string());
                        let stuck = t
                            .get("potentially_stuck")
                            .and_then(|x| x.as_bool())
                            .unwrap_or(false);
                        let annotation = if stuck { "  [POTENTIALLY STUCK]" } else { "" };
                        out.push_str(&format!(
                            "  {}  active={} expired={} oldest_age={}{}\n",
                            topic, active, expired, age, annotation
                        ));
                    } else {
                        let err = t.get("error").and_then(|x| x.as_str()).unwrap_or("(unknown)");
                        out.push_str(&format!("  {}  (fetch error: {})\n", topic, err));
                    }
                }
                out.push_str(&format!(
                    "  ({} topic(s), {} with potentially stuck claims)\n",
                    topic_count, stuck_count
                ));
            }
        }
        Err(e) => out.push_str(&format!("  (CLAIM unavailable: {})\n", e)),
    }
    out.push('\n');

    // RESILIENCE section
    out.push_str("RESILIENCE (substrate #5 — is my queue draining?):\n");
    match resilience {
        Ok(v) => {
            let exists = v.get("exists").and_then(|x| x.as_bool()).unwrap_or(false);
            let pending = v.get("pending").and_then(|x| x.as_u64()).unwrap_or(0);
            let queue_path = v.get("queue_path").and_then(|x| x.as_str()).unwrap_or("?");
            if !exists {
                out.push_str(&format!(
                    "  pending=0 (queue file not created yet: {})\n",
                    queue_path
                ));
            } else if pending == 0 {
                out.push_str(&format!("  pending=0 (steady-state)  queue={}\n", queue_path));
            } else {
                let oldest = v.get("oldest");
                let age_hint = oldest
                    .and_then(|o| o.get("ts_unix_ms"))
                    .and_then(|x| x.as_i64())
                    .map(|ms| format!("oldest_ts_ms={}", ms))
                    .unwrap_or_else(|| "oldest=-".to_string());
                out.push_str(&format!(
                    "  pending={}  {}  queue={}\n",
                    pending, age_hint, queue_path
                ));
            }
        }
        Err(e) => out.push_str(&format!("  (RESILIENCE unavailable: {})\n", e)),
    }
    out.push('\n');

    // BACKPRESSURE section
    out.push_str("BACKPRESSURE (substrate #10 — any hub pressured?):\n");
    match backpressure {
        Ok(v) => {
            let total = v.get("total").and_then(|x| x.as_u64()).unwrap_or(0);
            let summary = v.get("summary").cloned().unwrap_or_else(|| json!({}));
            let shown = summary.get("shown").and_then(|x| x.as_u64()).unwrap_or(0);
            let at_capacity = summary
                .get("hubs_at_capacity")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);
            let rate_limited = summary
                .get("hubs_rate_limited")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);

            if total == 0 {
                out.push_str(
                    "  (no hubs configured — declare profiles in ~/.termlink/hubs.toml)\n",
                );
            } else if only_pressured && shown == 0 {
                out.push_str(&format!("  All hubs healthy (0/{} pressured)\n", total));
            } else {
                let hubs = v.get("hubs").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                for h in &hubs {
                    let name = h.get("hub").and_then(|x| x.as_str()).unwrap_or("?");
                    if h.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
                        let gov = h.get("governor").cloned().unwrap_or_else(|| json!({}));
                        let active =
                            gov.get("connections_active").and_then(|x| x.as_i64()).unwrap_or(0);
                        let max =
                            gov.get("connections_max").and_then(|x| x.as_i64()).unwrap_or(0);
                        let cap_hits = gov
                            .get("capacity_hits_total")
                            .and_then(|x| x.as_i64())
                            .unwrap_or(0);
                        let rate_hits =
                            gov.get("rate_hits_total").and_then(|x| x.as_i64()).unwrap_or(0);
                        out.push_str(&format!(
                            "  {}  conn={}/{}  cap_hits={} rate_hits={}\n",
                            name, active, max, cap_hits, rate_hits
                        ));
                    } else {
                        let err = h.get("error").and_then(|x| x.as_str()).unwrap_or("(unknown)");
                        out.push_str(&format!("  {}  UNREACHABLE: {}\n", name, err));
                    }
                }
                out.push_str(&format!(
                    "  ({} hub(s), {} at capacity, {} rate-limited)\n",
                    total, at_capacity, rate_limited
                ));
            }
        }
        Err(e) => out.push_str(&format!("  (BACKPRESSURE unavailable: {})\n", e)),
    }
    out
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_dispatch_empty() -> SubResult {
        Ok(json!({ "idle": [] }))
    }
    fn ok_dispatch_with_idle() -> SubResult {
        Ok(json!({
            "idle": [
                { "agent_id": "alice", "role": "claude-code", "last_heartbeat_ms": 1234567890i64, "capabilities": "deploy,review" },
            ]
        }))
    }
    fn ok_claim_clean() -> SubResult {
        Ok(json!({
            "topic_count": 3, "stuck_count": 0, "shown": 0,
            "only_stuck": true, "topics": [],
        }))
    }
    fn ok_claim_with_stuck() -> SubResult {
        Ok(json!({
            "topic_count": 2, "stuck_count": 1, "shown": 1,
            "only_stuck": true,
            "topics": [
                { "ok": true, "topic": "work-queue",
                  "active_count": 1, "expired_count": 1,
                  "oldest_active_age_ms": 95000i64, "potentially_stuck": true,
                  "oldest_active_at_ms": null, "next_active_expiry_ms": null },
            ],
        }))
    }
    fn ok_resilience_drained() -> SubResult {
        Ok(json!({
            "queue_path": "/home/u/.termlink/outbound.sqlite",
            "exists": true, "cap": 1000, "pending": 0, "oldest": null
        }))
    }
    fn ok_resilience_pending() -> SubResult {
        Ok(json!({
            "queue_path": "/home/u/.termlink/outbound.sqlite",
            "exists": true, "cap": 1000, "pending": 7,
            "oldest": {
                "queue_id": 42, "topic": "agent-chat-arc",
                "msg_type": "say", "ts_unix_ms": 1700000000000i64,
                "sender_id": "alice", "artifact_ref": null
            }
        }))
    }
    fn ok_backpressure_healthy() -> SubResult {
        Ok(json!({
            "ok": true, "total": 2, "reachable": 2,
            "hubs": [
                { "hub": "ring20-management", "ok": true,
                  "governor": { "connections_active": 1, "connections_max": 256,
                                "capacity_hits_total": 0, "rate_hits_total": 0 } },
                { "hub": "ring20-dashboard", "ok": true,
                  "governor": { "connections_active": 0, "connections_max": 256,
                                "capacity_hits_total": 0, "rate_hits_total": 0 } },
            ],
            "summary": { "hubs_at_capacity": 0, "hubs_rate_limited": 0,
                         "shown": 2, "only_pressured": false }
        }))
    }
    fn ok_backpressure_empty_filtered() -> SubResult {
        Ok(json!({
            "ok": true, "total": 2, "reachable": 2,
            "hubs": [],
            "summary": { "hubs_at_capacity": 0, "hubs_rate_limited": 0,
                         "shown": 0, "only_pressured": true }
        }))
    }

    #[test]
    fn all_healthy_zero_state_renders_affirmative_sections() {
        // T-2111 AC: zero state for each section renders an affirmative
        // line, never a silent empty section.
        let out = render_substrate_text(
            &ok_dispatch_empty(),
            &ok_claim_clean(),
            &ok_resilience_drained(),
            &ok_backpressure_empty_filtered(),
            true, // only_pressured
        );
        assert!(out.contains("═══ substrate status ═══"));
        assert!(out.contains("DISPATCH"));
        assert!(out.contains("CLAIM"));
        assert!(out.contains("RESILIENCE"));
        assert!(out.contains("BACKPRESSURE"));
        // Affirmative-zero lines:
        assert!(
            out.contains("(no idle agents"),
            "expected DISPATCH zero hint, got:\n{}",
            out
        );
        assert!(
            out.contains("All topics healthy (0/3 stuck)"),
            "expected CLAIM healthy hint, got:\n{}",
            out
        );
        assert!(
            out.contains("pending=0 (steady-state)"),
            "expected RESILIENCE steady hint, got:\n{}",
            out
        );
        assert!(
            out.contains("All hubs healthy (0/2 pressured)"),
            "expected BACKPRESSURE healthy hint, got:\n{}",
            out
        );
    }

    #[test]
    fn rendered_content_includes_real_data_when_present() {
        // T-2111 AC: populated state renders the actual data per section.
        let out = render_substrate_text(
            &ok_dispatch_with_idle(),
            &ok_claim_with_stuck(),
            &ok_resilience_pending(),
            &ok_backpressure_healthy(),
            false,
        );
        assert!(out.contains("alice"), "expected DISPATCH agent row: {}", out);
        assert!(
            out.contains("work-queue") && out.contains("[POTENTIALLY STUCK]"),
            "expected CLAIM stuck topic row: {}",
            out
        );
        assert!(
            out.contains("pending=7"),
            "expected RESILIENCE pending row: {}",
            out
        );
        assert!(
            out.contains("ring20-management") && out.contains("conn=1/256"),
            "expected BACKPRESSURE hub row: {}",
            out
        );
    }

    #[test]
    fn json_envelope_shape_with_all_sub_sections() {
        // T-2111 AC: --json envelope has {ok, ts, dispatch, claim, resilience,
        // backpressure} keys, each section wrapped in {ok, data} or {ok:false, error}.
        let env = json!({
            "ok": true,
            "ts": "2026-06-10T07:30:00Z",
            "only_pressured": false,
            "dispatch":     section_json(&ok_dispatch_with_idle()),
            "claim":        section_json(&ok_claim_clean()),
            "resilience":   section_json(&ok_resilience_drained()),
            "backpressure": section_json(&ok_backpressure_healthy()),
        });
        for key in ["ok", "ts", "dispatch", "claim", "resilience", "backpressure"] {
            assert!(env.get(key).is_some(), "envelope missing key: {}", key);
        }
        // Each sub-section has {ok, data} on success.
        for sec in ["dispatch", "claim", "resilience", "backpressure"] {
            let s = env.get(sec).unwrap();
            assert_eq!(
                s.get("ok").and_then(|x| x.as_bool()),
                Some(true),
                "section {} ok flag missing/false",
                sec
            );
            assert!(s.get("data").is_some(), "section {} data missing", sec);
        }
    }

    #[test]
    fn partial_failure_still_renders_other_sections() {
        // T-2111 AC: a sub-verb returning Err still allows the other three
        // sections to render; the failing section shows its error line.
        let err_dispatch: SubResult =
            Err("local hub not running (no socket) — DISPATCH read needs the local hub".into());
        let out = render_substrate_text(
            &err_dispatch,
            &ok_claim_clean(),
            &ok_resilience_drained(),
            &ok_backpressure_healthy(),
            false,
        );
        assert!(
            out.contains("(DISPATCH unavailable: local hub not running"),
            "expected DISPATCH error line: {}",
            out
        );
        // Other sections still rendered:
        assert!(out.contains("CLAIM"));
        assert!(out.contains("RESILIENCE"));
        assert!(out.contains("BACKPRESSURE"));
        assert!(
            out.contains("pending=0"),
            "expected RESILIENCE still renders: {}",
            out
        );
    }

    #[test]
    fn json_envelope_marks_failed_section_explicitly() {
        // T-2111 AC: in --json mode, a failed section carries ok:false + error;
        // the top-level ok is false iff any sub-section failed.
        let dispatch_err: SubResult = Err("boom".into());
        let dispatch_sec = section_json(&dispatch_err);
        assert_eq!(
            dispatch_sec.get("ok").and_then(|x| x.as_bool()),
            Some(false)
        );
        assert_eq!(
            dispatch_sec.get("error").and_then(|x| x.as_str()),
            Some("boom")
        );
        let claim_ok = section_json(&ok_claim_clean());
        assert_eq!(claim_ok.get("ok").and_then(|x| x.as_bool()), Some(true));
        assert!(claim_ok.get("data").is_some());
    }

    #[test]
    fn is_potentially_stuck_predicates() {
        use termlink_session::claim_client::ClaimsAggregate;
        // Healthy: 1 active claim 5s old → not stuck.
        let healthy = ClaimsAggregate {
            topic: "t".into(),
            active_count: 1,
            expired_count: 0,
            oldest_active_at_ms: Some(123),
            oldest_active_age_ms: Some(5_000),
            next_active_expiry_ms: Some(456),
        };
        assert!(!is_potentially_stuck(&healthy));
        // Expired count > 0 → stuck.
        let mut expired_stuck = healthy.clone();
        expired_stuck.expired_count = 1;
        assert!(is_potentially_stuck(&expired_stuck));
        // Oldest age > 60s → stuck.
        let mut age_stuck = healthy.clone();
        age_stuck.oldest_active_age_ms = Some(95_000);
        assert!(is_potentially_stuck(&age_stuck));
    }
}
