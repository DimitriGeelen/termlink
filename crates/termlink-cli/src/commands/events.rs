use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

/// T-1401: hub-side mirror topic for broadcasts (defined in
/// `termlink_hub::channel::BROADCAST_GLOBAL_TOPIC`). Inlined here because
/// the CLI does not depend on the hub crate. Both sides MUST use the same
/// literal — change in lockstep.
const BROADCAST_GLOBAL_TOPIC: &str = "broadcast:global";

/// T-1401: Try to send the broadcast as a signed `channel.post(broadcast:global)`
/// envelope, mirroring the hub-side `mirror_event_broadcast` shape (T-1162).
/// On any failure the caller falls back to legacy `event.broadcast` so the
/// command remains functional across version skew.
async fn try_broadcast_via_channel_post(
    hub_socket: &std::path::Path,
    topic: &str,
    payload: &serde_json::Value,
    timeout_dur: std::time::Duration,
) -> Result<i64> {
    use base64::Engine;
    use termlink_protocol::control::channel::canonical_sign_bytes;
    use termlink_protocol::control::method;

    let identity = super::channel::load_identity_or_create()?;
    let payload_bytes = serde_json::to_vec(payload)?;
    let ts_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    // Mirror T-1162 wire shape: topic="broadcast:global", msg_type=<original>,
    // payload=<JSON bytes>, no artifact_ref. Hub recomputes signed_bytes with
    // identical inputs, so signing must match exactly.
    let signed = canonical_sign_bytes(
        BROADCAST_GLOBAL_TOPIC,
        topic,
        &payload_bytes,
        None,
        ts_unix_ms,
    );
    let sig = identity.sign(&signed);
    let sig_hex: String = sig
        .to_bytes()
        .iter()
        .fold(String::with_capacity(128), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(&mut s, "{b:02x}");
            s
        });

    let mut params = serde_json::json!({
        "topic": BROADCAST_GLOBAL_TOPIC,
        "msg_type": topic,
        "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
        "ts": ts_unix_ms,
        "sender_id": identity.fingerprint(),
        "sender_pubkey_hex": identity.public_key_hex(),
        "signature_hex": sig_hex,
    });

    // Replaces the previous `params.from` injection that only worked for
    // event.broadcast — for channel.post it goes into metadata so the
    // hub-side soft-lint can attribute the caller.
    if let Ok(sid) = std::env::var("TERMLINK_SESSION_ID")
        && !sid.is_empty()
    {
        params["metadata"] = serde_json::json!({"from": sid});
    }

    let rpc = client::rpc_call(hub_socket, method::CHANNEL_POST, params);
    let resp = tokio::time::timeout(timeout_dur, rpc)
        .await
        .map_err(|_| anyhow::anyhow!("channel.post timed out"))?
        .map_err(|e| anyhow::anyhow!("channel.post connect: {e}"))?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow::anyhow!("channel.post error: {e}"))?;
    let offset = result["offset"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("channel.post response missing offset"))?;
    Ok(offset)
}

pub(crate) async fn cmd_events(target: &str, since: Option<u64>, topic: Option<&str>, json: bool, timeout_secs: u64, payload_only: bool) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let mut params = serde_json::json!({});
    if let Some(s) = since {
        params["since"] = serde_json::json!(s);
    }
    if let Some(t) = topic {
        params["topic"] = serde_json::json!(t);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc = client::rpc_call(reg.socket_path(), "event.poll", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => match r {
            Ok(v) => v,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Event poll timed out after {}s", timeout_secs)}));
            }
            anyhow::bail!("Event poll timed out after {}s", timeout_secs);
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
                println!("{}", serde_json::to_string_pretty(&wrapped)?);
                return Ok(());
            }
            let events = result["events"].as_array()
                .context("Server returned unexpected format: missing 'events' array")?;

            if payload_only {
                for event in events {
                    let payload = &event["payload"];
                    if !payload.is_null() {
                        println!("{}", serde_json::to_string(payload)?);
                    }
                }
                return Ok(());
            }

            if events.is_empty() {
                println!("No events (next_seq: {})", result["next_seq"]);
                return Ok(());
            }

            for event in events {
                let seq = event["seq"].as_u64().unwrap_or(0);
                let topic = event["topic"].as_str().unwrap_or("?");
                let payload = &event["payload"];
                let ts = event["timestamp"].as_u64().unwrap_or(0);

                if payload.is_null() || (payload.as_object().is_some_and(|o| o.is_empty())) {
                    println!("[{seq}] {topic} (t={ts})");
                } else {
                    println!("[{seq}] {topic}: {} (t={ts})", serde_json::to_string(payload)?);
                }
            }
            println!();
            println!("{} event(s), next_seq: {}", result["count"], result["next_seq"]);
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Event poll failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_emit(target: &str, topic: &str, payload_str: &str, json: bool, timeout_secs: u64) -> Result<()> {
    let payload: serde_json::Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Invalid JSON payload: {}", e)}));
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
    let rpc = client::rpc_call(
        reg.socket_path(),
        "event.emit",
        serde_json::json!({ "topic": topic, "payload": payload }),
    );
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => match r {
            Ok(v) => v,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Event emit timed out after {}s", timeout_secs)}));
            }
            anyhow::bail!("Event emit timed out after {}s", timeout_secs);
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
                println!("{}", serde_json::to_string_pretty(&wrapped)?);
            } else {
                println!(
                    "Event emitted: {} (seq: {})",
                    result["topic"].as_str().unwrap_or("?"),
                    result["seq"].as_u64().unwrap_or(0),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Event emit failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_broadcast(topic: &str, payload_str: &str, targets: Vec<String>, json: bool, timeout_secs: u64) -> Result<()> {
    let replacement = if targets.is_empty() { "channel post" } else { "event emit_to" };
    super::print_deprecation_warning("event broadcast", replacement);
    let payload: serde_json::Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Invalid JSON payload: {}", e)}));
            }
            return Err(e.into());
        }
    };

    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    if !hub_socket.exists() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    // T-1401: when no explicit --targets are given (the dominant case), prefer
    // the channel.post(broadcast:global) path. The hub already mirrors every
    // event.broadcast to the same topic (T-1162), so subscribers see identical
    // envelopes. On any failure (older hub, signing/identity setup issue) fall
    // through to the legacy event.broadcast call below.
    if targets.is_empty()
        && let Ok(offset) =
            try_broadcast_via_channel_post(&hub_socket, topic, &payload, timeout_dur).await
    {
        if json {
            let wrapped = serde_json::json!({
                "ok": true,
                "topic": topic,
                "channel_topic": BROADCAST_GLOBAL_TOPIC,
                "offset": offset,
                "targeted": 1,
                "succeeded": 1,
                "failed": 0,
            });
            println!("{}", serde_json::to_string_pretty(&wrapped)?);
        } else {
            println!(
                "Broadcast '{}': 1/1 succeeded (channel:{} offset={})",
                topic, BROADCAST_GLOBAL_TOPIC, offset
            );
        }
        return Ok(());
    }

    // T-1417: Pre-T-1166 cut migration. The legacy `event.broadcast` is
    // retiring; replace the per-target fan-out with parallel `event.emit_to`
    // calls. Result shape ({topic, targeted, succeeded, failed}) preserved
    // so downstream consumers don't need to change.
    //
    // For empty-targets we already prefer channel.post(broadcast:global)
    // above; if that block didn't return, either targets is non-empty (fan
    // out below) or channel.post failed for empty-targets. The latter is
    // surfaced as an error rather than falling through to the retiring
    // event.broadcast path.
    if targets.is_empty() {
        if json {
            super::json_error_exit(serde_json::json!({
                "ok": false, "topic": topic,
                "error": "channel.post(broadcast:global) failed and event.broadcast is retiring (T-1166); no usable broadcast path"
            }));
        }
        anyhow::bail!(
            "channel.post(broadcast:global) failed and event.broadcast is retiring (T-1166); no usable broadcast path"
        );
    }

    let (targeted, succeeded, failed, errors) =
        broadcast_via_emit_to_fanout(&hub_socket, topic, &payload, &targets, timeout_dur).await;

    if json {
        let mut wrapped = serde_json::json!({
            "ok": failed == 0,
            "topic": topic,
            "targeted": targeted,
            "succeeded": succeeded,
            "failed": failed,
        });
        if !errors.is_empty() {
            wrapped["errors"] = serde_json::json!(errors);
        }
        println!("{}", serde_json::to_string_pretty(&wrapped)?);
    } else {
        println!(
            "Broadcast '{}': {}/{} succeeded{}",
            topic,
            succeeded,
            targeted,
            if failed > 0 {
                format!(" ({} failed)", failed)
            } else {
                String::new()
            },
        );
        if failed > 0 {
            for err in &errors {
                eprintln!("  {}", err);
            }
        }
    }
    Ok(())
}

/// T-1417: Parallel `event.emit_to` fanout — replacement for the retiring
/// `event.broadcast` per-target dispatch. Each target gets its own RPC,
/// issued concurrently. Per-target failures are aggregated, not propagated:
/// a partial-success broadcast (3/5 ok) returns succeeded=3, failed=2 with
/// per-target error messages, matching the legacy event.broadcast result
/// contract.
///
/// `from` is populated from $TERMLINK_SESSION_ID when set (preserves the
/// T-1300 soft-lint role-resolution behavior the legacy path had).
async fn broadcast_via_emit_to_fanout(
    hub_socket: &std::path::Path,
    topic: &str,
    payload: &serde_json::Value,
    targets: &[String],
    timeout_dur: std::time::Duration,
) -> (u64, u64, u64, Vec<String>) {
    let from_sid = std::env::var("TERMLINK_SESSION_ID")
        .ok()
        .filter(|s| !s.is_empty());

    let mut handles = Vec::with_capacity(targets.len());
    for target in targets {
        let mut params = serde_json::json!({
            "target": target,
            "topic": topic,
            "payload": payload,
        });
        if let Some(sid) = &from_sid {
            params["from"] = serde_json::json!(sid);
        }
        let socket = hub_socket.to_path_buf();
        let target_owned = target.clone();
        let handle = tokio::spawn(async move {
            let rpc = client::rpc_call(&socket, "event.emit_to", params);
            (
                target_owned,
                tokio::time::timeout(timeout_dur, rpc).await,
            )
        });
        handles.push(handle);
    }

    let targeted = targets.len() as u64;
    let mut succeeded: u64 = 0;
    let mut failed: u64 = 0;
    let mut errors: Vec<String> = Vec::new();

    for h in handles {
        match h.await {
            Ok((target, Ok(Ok(resp)))) => match client::unwrap_result(resp) {
                Ok(_) => succeeded += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("{}: {}", target, e));
                }
            },
            Ok((target, Ok(Err(e)))) => {
                failed += 1;
                errors.push(format!("{}: connection: {}", target, e));
            }
            Ok((target, Err(_))) => {
                failed += 1;
                errors.push(format!("{}: timeout", target));
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("(join error): {}", e));
            }
        }
    }

    (targeted, succeeded, failed, errors)
}

pub(crate) async fn cmd_emit_to(
    target: &str,
    topic: &str,
    payload_str: &str,
    from: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let payload: serde_json::Value = match serde_json::from_str(payload_str) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Invalid JSON payload: {}", e)}));
            }
            return Err(e.into());
        }
    };

    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    if !hub_socket.exists() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    let mut params = serde_json::json!({
        "target": target,
        "topic": topic,
        "payload": payload,
    });
    if let Some(sender) = from {
        params["from"] = serde_json::json!(sender);
    }
    // T-1310: mirror T-1300 broadcast pattern — populate `from` from
    // $TERMLINK_SESSION_ID when caller did not pass an explicit value.
    // Enables T-1309 caller-attribution breakdown to cover event.emit_to.
    if params.get("from").is_none()
        && let Ok(sid) = std::env::var("TERMLINK_SESSION_ID")
        && !sid.is_empty()
    {
        params["from"] = serde_json::json!(sid);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc = client::rpc_call(&hub_socket, "event.emit_to", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => match r {
            Ok(v) => v,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to connect to hub: {} — is it running? Start it with: termlink hub start", e)}));
                }
                return Err(e).context("Failed to connect to hub — is it running? Start it with: termlink hub start");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("emit-to timed out after {}s", timeout_secs)}));
            }
            anyhow::bail!("emit-to timed out after {}s", timeout_secs);
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
                println!("{}", serde_json::to_string_pretty(&wrapped)?);
            } else {
                println!(
                    "Pushed to {}: {} (seq: {})",
                    result["target"].as_str().unwrap_or(target),
                    result["topic"].as_str().unwrap_or(topic),
                    result["seq"].as_u64().unwrap_or(0),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("emit-to failed: {}", e);
        }
    }
}

pub(crate) struct WatchOpts<'a> {
    pub interval_ms: u64,
    pub topic_filter: Option<&'a str>,
    pub json: bool,
    pub timeout_secs: u64,
    pub max_count: u64,
    pub payload_only: bool,
    pub since: Option<u64>,
}

/// T-2636: returns true when a multi-session `event watch` tick dispatched at
/// least one session RPC but EVERY one errored — the busy-loop condition that
/// requires a sleep-backoff before the next tick.
///
/// A dead-socket `rpc_call` returns near-instantly (no `event.subscribe`
/// long-poll to pace the loop), so without a backoff the outer loop re-dispatches
/// with zero delay and pins a CPU core at 100%. This is the exact case the sibling
/// single-hub `cmd_watch_hub` (Err-arm at ~824-829) already handles with a 500ms
/// sleep; the two loops diverged and only one got the guard.
///
/// A tick with ≥1 live session returns false: that session's subscribe long-poll
/// naturally paces the loop, so a healthy tick must NOT be artificially delayed.
/// An empty tick (no sessions dispatched) also returns false — there is nothing
/// to spin on.
fn watch_tick_all_errored(ok_count: usize, err_count: usize) -> bool {
    err_count > 0 && ok_count == 0
}

pub(crate) async fn cmd_watch(
    targets: Vec<String>,
    opts: WatchOpts<'_>,
) -> Result<()> {
    use std::collections::HashMap;
    let WatchOpts { interval_ms, topic_filter, json, timeout_secs, max_count, payload_only, since } = opts;

    // Resolve targets: if empty, watch all live sessions
    let registrations = if targets.is_empty() {
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to list sessions: {e}")}));
                }
                return Err(e).context("Failed to list sessions");
            }
        };
        if sessions.is_empty() {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": "No active sessions to watch."}));
            }
            anyhow::bail!("No active sessions to watch.");
        }
        sessions
            .iter()
            .filter_map(|s| manager::find_session(s.id.as_str()).ok())
            .collect::<Vec<_>>()
    } else {
        let mut regs = Vec::new();
        for t in &targets {
            match manager::find_session(t) {
                Ok(r) => regs.push(r),
                Err(e) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "target": t, "error": format!("Session '{}' not found: {}", t, e)}));
                    }
                    return Err(e).context(format!("Session '{}' not found", t));
                }
            }
        }
        regs
    };

    if registrations.is_empty() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "No reachable sessions to watch."}));
        }
        anyhow::bail!("No reachable sessions to watch.");
    }

    let session_names: HashMap<String, String> = registrations
        .iter()
        .map(|r| (r.id.as_str().to_string(), r.display_name.clone()))
        .collect();

    if !json {
        eprintln!(
            "Watching {} session(s): {}. Press Ctrl+C to stop.",
            registrations.len(),
            registrations
                .iter()
                .map(|r| r.display_name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
        if timeout_secs > 0 {
            eprintln!("  Timeout: {}s", timeout_secs);
        }
        eprintln!();
    }

    // Initialize cursors — use --since value if provided, otherwise start from live
    let mut cursors: HashMap<String, Option<u64>> = registrations
        .iter()
        .map(|r| (r.id.as_str().to_string(), since))
        .collect();

    // Use event.subscribe for push-based delivery. The server blocks until
    // events arrive (near-zero latency) instead of client-side sleep+poll.
    // Subscribe calls are dispatched concurrently across sessions.
    let subscribe_timeout = interval_ms.max(100); // min 100ms to avoid busy-loop

    let deadline = if timeout_secs > 0 {
        Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };
    let mut total_received: u64 = 0;

    loop {
        if let Some(dl) = deadline
            && std::time::Instant::now() >= dl
        {
            if !json {
                eprintln!();
                eprintln!("Stopped watching (timeout after {}s).", timeout_secs);
            }
            break;
        }

        // Dispatch subscribe calls concurrently across all sessions
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                if !json {
                    eprintln!();
                    eprintln!("Stopped watching.");
                }
                break;
            }
            results = async {
                let mut join_set = tokio::task::JoinSet::new();
                for reg in &registrations {
                    let sid = reg.id.as_str().to_string();
                    let addr = reg.socket_path().to_path_buf();
                    let cursor = cursors.get(&sid).and_then(|c| *c);
                    let topic = topic_filter.map(String::from);
                    let timeout_ms = subscribe_timeout;

                    join_set.spawn(async move {
                        let mut params = serde_json::json!({
                            "timeout_ms": timeout_ms,
                        });
                        if let Some(c) = cursor {
                            params["since"] = serde_json::json!(c);
                        }
                        if let Some(t) = &topic {
                            params["topic"] = serde_json::json!(t);
                        }

                        let resp = client::rpc_call(&addr, "event.subscribe", params).await;
                        (sid, resp)
                    });
                }

                let mut all_results = Vec::new();
                while let Some(result) = join_set.join_next().await {
                    if let Ok(r) = result {
                        all_results.push(r);
                    }
                }
                all_results
            } => {
                let mut ok_count = 0usize;
                let mut err_count = 0usize;
                for (sid, resp) in results {
                    let name = session_names.get(&sid).map(|s| s.as_str()).unwrap_or(&sid);

                    let resp = match resp {
                        Ok(r) => { ok_count += 1; r }
                        Err(_) => { err_count += 1; continue }
                    };

                    if let Ok(result) = client::unwrap_result(resp) {
                        if let Some(events) = result["events"].as_array() {
                            for event in events {
                                let seq = event["seq"].as_u64().unwrap_or(0);
                                let topic = event["topic"].as_str().unwrap_or("?");
                                let payload = &event["payload"];
                                let ts = event["timestamp"].as_u64().unwrap_or(0);

                                if payload_only {
                                    if !payload.is_null() {
                                        println!("{}", serde_json::to_string(payload).unwrap_or_default());
                                    }
                                } else if json {
                                    println!("{}", serde_json::json!({
                                        "ok": true,
                                        "session": name,
                                        "session_id": &sid,
                                        "seq": seq,
                                        "topic": topic,
                                        "payload": payload,
                                        "timestamp": ts,
                                    }));
                                } else if payload.is_null()
                                    || payload.as_object().is_some_and(|o| o.is_empty())
                                {
                                    println!("[{name}#{seq}] {topic} (t={ts})");
                                } else {
                                    println!(
                                        "[{name}#{seq}] {topic}: {} (t={ts})",
                                        serde_json::to_string(payload).unwrap_or_default()
                                    );
                                }

                                cursors.insert(sid.clone(), Some(seq));
                                total_received += 1;
                            }
                        }
                        // Update cursor from next_seq if no events were returned
                        if let Some(next) = result["next_seq"].as_u64()
                            && cursors.get(&sid).and_then(|c| *c).is_none() && next > 0 {
                                cursors.insert(sid.clone(), Some(next.saturating_sub(1)));
                            }
                    }
                }

                // T-2636: when every watched session socket is down, each
                // rpc_call errored near-instantly (no long-poll pacing) and the
                // outer loop would re-dispatch with zero delay — a 100% CPU
                // busy-loop. Mirror the sibling cmd_watch_hub's 500ms Err-arm
                // sleep. A tick with ≥1 live session is paced by its subscribe
                // long-poll, so it is NOT delayed here.
                if watch_tick_all_errored(ok_count, err_count) {
                    if !json {
                        eprintln!("All watched sessions unreachable. Retrying...");
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }

        if max_count > 0 && total_received >= max_count {
            if !json {
                eprintln!();
                eprintln!("{} event(s) received (limit reached).", total_received);
            }
            break;
        }
    }

    Ok(())
}

/// T-1645: Subscribe to the hub-level event aggregator (no `target` param).
///
/// Hub router routes `event.subscribe` with no `target` to
/// `handle_hub_subscribe` → `aggregator.collect()`. This surfaces events
/// emitted via `aggregator().inject(...)` with `session_id: "hub"` — notably
/// `inbox.queued` from `channel.post inbox:<id>` (T-1636/T-1637 emit-site).
/// The aggregator is a tokio broadcast channel: real-time only, no `since`
/// cursor. `opts.since` is ignored under hub mode (warned at JSON-mode).
pub(crate) async fn cmd_watch_hub(opts: WatchOpts<'_>) -> Result<()> {
    let WatchOpts { interval_ms, topic_filter, json, timeout_secs, max_count, payload_only, since } = opts;

    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    if !hub_socket.exists() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    if since.is_some() && !json {
        eprintln!("Note: --since is ignored under --hub (aggregator is real-time broadcast, no cursor).");
    }

    if !json {
        eprintln!("Watching hub-level event aggregator. Press Ctrl+C to stop.");
        if let Some(t) = topic_filter {
            eprintln!("  Topic filter: {}", t);
        }
        if timeout_secs > 0 {
            eprintln!("  Timeout: {}s", timeout_secs);
        }
        eprintln!();
    }

    let subscribe_timeout_ms = interval_ms.max(100);
    let deadline = if timeout_secs > 0 {
        Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };
    let mut total_received: u64 = 0;

    loop {
        if let Some(dl) = deadline
            && std::time::Instant::now() >= dl
        {
            if !json {
                eprintln!();
                eprintln!("Stopped (timeout after {}s). {} event(s) received.", timeout_secs, total_received);
            }
            break;
        }

        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                if !json {
                    eprintln!();
                    eprintln!("Stopped. {} event(s) received.", total_received);
                }
                break;
            }
            subscribe_result = async {
                let mut params = serde_json::json!({
                    "timeout_ms": subscribe_timeout_ms,
                });
                if let Some(t) = topic_filter {
                    params["topic"] = serde_json::json!(t);
                }
                // No `target` field → routes to handle_hub_subscribe (router.rs:128 + 610)
                client::rpc_call(&hub_socket, "event.subscribe", params).await
            } => {
                let resp = match subscribe_result {
                    Ok(r) => r,
                    Err(e) => {
                        if !json {
                            eprintln!("Hub connection error: {}. Retrying...", e);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        continue;
                    }
                };
                if let Ok(result) = client::unwrap_result(resp)
                    && let Some(events) = result["events"].as_array()
                {
                    for event in events {
                        let session = event["session"].as_str().unwrap_or("hub");
                        let session_name = event["session_name"].as_str().unwrap_or(session);
                        let seq = event["seq"].as_u64().unwrap_or(0);
                        let topic = event["topic"].as_str().unwrap_or("?");
                        let payload = &event["payload"];
                        let ts = event["timestamp"].as_u64().unwrap_or(0);

                        if payload_only {
                            if !payload.is_null() {
                                println!("{}", serde_json::to_string(payload).unwrap_or_default());
                            }
                        } else if json {
                            println!("{}", serde_json::json!({
                                "ok": true,
                                "source": "hub-aggregator",
                                "session": session,
                                "session_name": session_name,
                                "seq": seq,
                                "topic": topic,
                                "payload": payload,
                                "timestamp": ts,
                            }));
                        } else if payload.is_null()
                            || payload.as_object().is_some_and(|o| o.is_empty())
                        {
                            println!("[hub:{session_name}#{seq}] {topic} (t={ts})");
                        } else {
                            println!(
                                "[hub:{session_name}#{seq}] {topic}: {} (t={ts})",
                                serde_json::to_string(payload).unwrap_or_default()
                            );
                        }
                        total_received += 1;
                        if max_count > 0 && total_received >= max_count {
                            break;
                        }
                    }
                }
            }
        }

        if max_count > 0 && total_received >= max_count {
            if !json {
                eprintln!();
                eprintln!("{} event(s) received (limit reached).", total_received);
            }
            break;
        }
    }

    Ok(())
}

pub(crate) async fn cmd_wait(target: &str, topic: &str, timeout_secs: u64, interval_ms: u64, json: bool, since: Option<u64>) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    if !json {
        eprintln!("Waiting for event topic '{}' from {}...", topic, reg.display_name);
    }

    // Use event.subscribe for push-based delivery — server blocks until
    // matching event arrives, eliminating polling latency.
    let subscribe_timeout = interval_ms.max(500); // at least 500ms per subscribe call
    let deadline = if timeout_secs > 0 {
        Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };

    let mut cursor: Option<u64> = since;

    loop {
        if let Some(dl) = deadline
            && tokio::time::Instant::now() >= dl {
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false,
                        "matched": false,
                        "topic": topic,
                        "target": target,
                        "reason": "timeout",
                        "timeout_secs": timeout_secs,
                    }));
                }
                anyhow::bail!("Timeout waiting for event topic '{}'", topic);
            }

        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false,
                        "matched": false,
                        "topic": topic,
                        "target": target,
                        "reason": "interrupted",
                    }));
                }
                anyhow::bail!("Interrupted");
            }
            rpc_result = async {
                let mut params = serde_json::json!({
                    "topic": topic,
                    "timeout_ms": subscribe_timeout,
                    "max_events": 1,
                });
                if let Some(c) = cursor {
                    params["since"] = serde_json::json!(c);
                }
                client::rpc_call(reg.socket_path(), "event.subscribe", params).await
            } => {
                match rpc_result {
                    Err(_) => {
                        if json {
                            super::json_error_exit(serde_json::json!({
                                "ok": false,
                                "matched": false,
                                "topic": topic,
                                "target": target,
                                "reason": "disconnected",
                            }));
                        }
                        anyhow::bail!("Session '{}' disconnected while waiting", target);
                    }
                    Ok(resp) => {
                        if let Ok(result) = client::unwrap_result(resp) {
                            if let Some(events) = result["events"].as_array()
                                && let Some(event) = events.first() {
                                    if json {
                                        println!("{}", serde_json::json!({
                                            "ok": true,
                                            "matched": true,
                                            "topic": event["topic"],
                                            "seq": event["seq"],
                                            "timestamp": event["timestamp"],
                                            "payload": event["payload"],
                                            "target": target,
                                        }));
                                    } else {
                                        let payload = &event["payload"];
                                        if payload.is_null()
                                            || payload.as_object().is_some_and(|o| o.is_empty())
                                        {
                                            println!("{}", topic);
                                        } else {
                                            println!("{}", serde_json::to_string(payload)?);
                                        }
                                    }
                                    return Ok(());
                                }
                            if let Some(next) = result["next_seq"].as_u64() {
                                cursor = if next > 0 { Some(next - 1) } else { None };
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Outcome of probing one session's `event.topics` RPC, reduced to the pieces
/// the aggregator needs. Keeps the async/transport concern out of the pure
/// aggregation so the skip-counting path is unit-testable (T-2624).
enum TopicsProbe {
    /// Transport error or timeout — session unreachable.
    Unreachable,
    /// RPC returned an error result, or the response lacked a `topics` array.
    BadResult,
    /// Session answered with a (possibly empty) topic list.
    Topics(Vec<String>),
}

/// Result of folding per-session probes: the topic inventory plus skip tallies.
struct TopicsAggregate {
    session_topics: std::collections::BTreeMap<String, Vec<String>>,
    /// Sessions dropped for timeout/transport error.
    unreachable: usize,
    /// Sessions that answered but with an error result or no `topics` array.
    bad_result: usize,
}

/// Fold per-session probe outcomes into the topic inventory plus skip tallies
/// (T-2624). A session that answers with zero topics is reachable-but-empty
/// (not skipped, preserving the pre-fix inventory semantics) — only genuine
/// failures increment the skip counters, so the caller can surface
/// "N of M session(s) unreachable — inventory may be incomplete" instead of
/// silently under-reporting.
fn aggregate_topics_probes(probes: Vec<(String, TopicsProbe)>) -> TopicsAggregate {
    let mut session_topics = std::collections::BTreeMap::new();
    let mut unreachable = 0usize;
    let mut bad_result = 0usize;
    for (name, probe) in probes {
        match probe {
            TopicsProbe::Unreachable => unreachable += 1,
            TopicsProbe::BadResult => bad_result += 1,
            TopicsProbe::Topics(list) => {
                if !list.is_empty() {
                    session_topics.insert(name, list);
                }
            }
        }
    }
    TopicsAggregate { session_topics, unreachable, bad_result }
}

pub(crate) async fn cmd_topics(target: Option<&str>, json: bool, timeout_secs: u64, no_header: bool) -> Result<()> {
    let registrations = if let Some(t) = target {
        vec![match manager::find_session(t) {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": t, "error": format!("Session '{}' not found: {}", t, e)}));
                }
                return Err(e).context(format!("Session '{}' not found", t));
            }
        }]
    } else {
        match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to list sessions: {e}")}));
                }
                return Err(e).context("Failed to list sessions");
            }
        }
    };

    if registrations.is_empty() {
        if json {
            println!("{}", serde_json::json!({"ok": true, "sessions": [], "total_topics": 0}));
        } else {
            println!("No active sessions.");
        }
        return Ok(());
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let total_probed = registrations.len();
    let mut probes: Vec<(String, TopicsProbe)> = Vec::with_capacity(total_probed);

    for reg in &registrations {
        let rpc_future = client::rpc_call(reg.socket_path(), "event.topics", serde_json::json!({}));
        // T-2624: classify each session's outcome explicitly so failures are
        // counted rather than silently `continue`d — the aggregation lives in
        // the pure `aggregate_topics_probes` helper for unit-testability.
        let probe = match tokio::time::timeout(timeout_dur, rpc_future).await {
            Ok(Ok(resp)) => match client::unwrap_result(resp) {
                Ok(result) => match result["topics"].as_array() {
                    Some(topics) => TopicsProbe::Topics(
                        topics
                            .iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect(),
                    ),
                    None => TopicsProbe::BadResult,
                },
                Err(_) => TopicsProbe::BadResult,
            },
            Ok(Err(_)) | Err(_) => TopicsProbe::Unreachable,
        };
        probes.push((reg.display_name.clone(), probe));
    }

    let TopicsAggregate { session_topics, unreachable, bad_result } =
        aggregate_topics_probes(probes);
    let skipped = unreachable + bad_result;

    let total: usize = session_topics.values().map(|v| v.len()).sum();

    if json {
        let sessions: Vec<serde_json::Value> = session_topics
            .iter()
            .map(|(name, topics)| serde_json::json!({"session": name, "topics": topics}))
            .collect();
        println!("{}", serde_json::json!({
            "ok": true,
            "sessions": sessions,
            "total_topics": total,
            "total_sessions": session_topics.len(),
            // T-2624: partial-inventory signal — a consumer can now tell the
            // topic set excludes sessions that timed out or errored.
            "sessions_unreachable": unreachable,
            "sessions_bad_result": bad_result,
            "sessions_skipped": skipped,
            "sessions_probed": total_probed,
        }));
        return Ok(());
    }

    if session_topics.is_empty() {
        if skipped > 0 {
            println!(
                "No event topics found ({} of {} session(s) unreachable/errored — inventory may be incomplete).",
                skipped, total_probed
            );
        } else {
            println!("No event topics found.");
        }
        return Ok(());
    }

    for (name, topics) in &session_topics {
        if !no_header {
            println!("{}:", name);
        }
        for topic in topics {
            if no_header {
                println!("{}", topic);
            } else {
                println!("  {}", topic);
            }
        }
    }

    if !no_header {
        println!();
        println!(
            "{} topic(s) across {} session(s)",
            total,
            session_topics.len()
        );
        // T-2624: never let a partial inventory read as complete.
        if skipped > 0 {
            println!(
                "note: {} of {} session(s) unreachable/errored — inventory may be incomplete ({} unreachable, {} bad-result)",
                skipped, total_probed, unreachable, bad_result
            );
        }
    }
    Ok(())
}

pub(crate) struct CollectOpts<'a> {
    pub topic_filter: Option<&'a str>,
    pub interval_ms: u64,
    pub max_count: u64,
    pub json: bool,
    pub timeout_secs: u64,
    pub payload_only: bool,
    pub since: Option<u64>,
}

pub(crate) async fn cmd_collect(
    targets: Vec<String>,
    opts: CollectOpts<'_>,
) -> Result<()> {
    let CollectOpts { topic_filter, interval_ms, max_count, json, timeout_secs, payload_only, since } = opts;
    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    if !hub_socket.exists() {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    if !json {
        eprintln!("Collecting events via hub. Press Ctrl+C to stop.");
        if let Some(t) = topic_filter {
            eprintln!("  Topic filter: {}", t);
        }
        if !targets.is_empty() {
            eprintln!("  Targets: {}", targets.join(", "));
        }
        if timeout_secs > 0 {
            eprintln!("  Timeout: {}s", timeout_secs);
        }
        eprintln!();
    }

    let subscribe_timeout_ms = interval_ms.max(500);
    let deadline = if timeout_secs > 0 {
        Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };
    let mut cursors = serde_json::json!({});
    let mut total_received: u64 = 0;

    loop {
        if let Some(dl) = deadline
            && std::time::Instant::now() >= dl
        {
            if !json {
                eprintln!();
                eprintln!("{} event(s) collected (timeout after {}s).", total_received, timeout_secs);
            }
            break;
        }

        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                if !json {
                    eprintln!();
                    eprintln!("Stopped. {} event(s) collected.", total_received);
                }
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
                if !cursors.as_object().unwrap_or(&serde_json::Map::new()).is_empty() {
                    params["since"] = cursors.clone();
                } else if let Some(s) = since {
                    // First iteration: use global --since as default for all sessions
                    params["since_default"] = serde_json::json!(s);
                }

                client::rpc_call(&hub_socket, "event.collect", params).await
            } => {
                let resp = match collect_result {
                    Ok(r) => r,
                    Err(e) => {
                        if !json {
                            eprintln!("Hub connection error: {}. Retrying...", e);
                        }
                        continue;
                    }
                };

                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let session_name = event["session_name"].as_str().unwrap_or("?");
                            let seq = event["seq"].as_u64().unwrap_or(0);
                            let topic = event["topic"].as_str().unwrap_or("?");
                            let payload = &event["payload"];
                            let ts = event["timestamp"].as_u64().unwrap_or(0);

                            if payload_only {
                                if !payload.is_null() {
                                    println!("{}", serde_json::to_string(payload).unwrap_or_default());
                                }
                            } else if json {
                                println!("{}", serde_json::json!({
                                    "ok": true,
                                    "session": session_name,
                                    "seq": seq,
                                    "topic": topic,
                                    "payload": payload,
                                    "timestamp": ts,
                                }));
                            } else if payload.is_null()
                                || payload.as_object().is_some_and(|o| o.is_empty())
                            {
                                println!("[{session_name}#{seq}] {topic} (t={ts})");
                            } else {
                                println!(
                                    "[{session_name}#{seq}] {topic}: {} (t={ts})",
                                    serde_json::to_string(payload).unwrap_or_default()
                                );
                            }

                            total_received += 1;
                        }
                    }

                    if let Some(new_cursors) = result.get("cursors")
                        && let Some(obj) = new_cursors.as_object() {
                            for (k, v) in obj {
                                cursors[k] = v.clone();
                            }
                        }

                    if max_count > 0 && total_received >= max_count {
                        if !json {
                            eprintln!();
                            eprintln!("{} event(s) collected (limit reached).", total_received);
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-2624: the topic-inventory aggregator must COUNT sessions that timed out
    // / errored rather than silently drop them. A fixture with 2 unreachable + 1
    // bad-result sessions must report those skips while total_topics covers only
    // the reachable sessions. Reverting the counters to a bare `continue` (no
    // tally) makes `unreachable`/`bad_result` read 0 and fails this (load-bearing).
    #[test]
    fn aggregate_topics_probes_counts_skipped_and_covers_reachable() {
        let probes = vec![
            ("s1".to_string(), TopicsProbe::Topics(vec!["a".into(), "b".into()])),
            ("s2".to_string(), TopicsProbe::Unreachable),
            ("s3".to_string(), TopicsProbe::Topics(vec!["c".into()])),
            ("s4".to_string(), TopicsProbe::Unreachable),
            ("s5".to_string(), TopicsProbe::BadResult),
        ];
        let agg = aggregate_topics_probes(probes);

        assert_eq!(agg.unreachable, 2, "two timed-out/errored sessions must be counted");
        assert_eq!(agg.bad_result, 1, "one bad-result session must be counted");

        let total: usize = agg.session_topics.values().map(|v| v.len()).sum();
        assert_eq!(total, 3, "total_topics must cover only the 3 reachable topics");
        assert_eq!(agg.session_topics.len(), 2, "only the 2 sessions with topics are inventoried");
    }

    // A session that answers with an empty topic list is reachable-but-empty:
    // not inventoried, but ALSO not counted as a skip (no false alarm).
    #[test]
    fn aggregate_topics_probes_empty_list_is_not_a_skip() {
        let probes = vec![
            ("s1".to_string(), TopicsProbe::Topics(vec![])),
            ("s2".to_string(), TopicsProbe::Topics(vec!["x".into()])),
        ];
        let agg = aggregate_topics_probes(probes);
        assert_eq!(agg.unreachable, 0);
        assert_eq!(agg.bad_result, 0);
        assert_eq!(agg.session_topics.len(), 1, "empty session is reachable-but-empty, not inventoried");
    }

    // T-2636: the multi-session watch loop must back off (sleep) only on a tick
    // where EVERY dispatched session RPC errored — the dead-socket busy-loop
    // condition. A healthy or mixed tick (≥1 ok) is paced by the subscribe
    // long-poll and must NOT be delayed. An empty tick has nothing to spin on.
    #[test]
    fn watch_tick_all_errored_fires_only_when_every_session_errored() {
        // all sockets down → back off (the busy-loop case this guard prevents)
        assert!(watch_tick_all_errored(0, 3), "all-errored tick must back off");
        assert!(watch_tick_all_errored(0, 1), "single errored session, none ok → back off");
    }

    #[test]
    fn watch_tick_all_errored_does_not_fire_on_healthy_or_mixed_ticks() {
        // ≥1 live session paces the loop via its long-poll — never delay it.
        // Reverting the guard to `err_count > 0` makes the mixed case fire and
        // fails this test (load-bearing).
        assert!(!watch_tick_all_errored(3, 0), "all healthy → no backoff");
        assert!(!watch_tick_all_errored(2, 1), "mixed (≥1 ok) → no backoff");
        assert!(!watch_tick_all_errored(1, 0), "one ok → no backoff");
        // empty tick: nothing dispatched, nothing to spin on. Reverting the
        // guard to `err_count == 0 || ok_count == 0` (dropping the err>0 arm)
        // makes this fire and fails the test (load-bearing).
        assert!(!watch_tick_all_errored(0, 0), "empty tick → no backoff");
    }
}
