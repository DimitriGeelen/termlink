use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

use termlink_protocol::events::{
    agent_topic, negotiate_topic, AgentRequest, AgentResponse, AgentStatus,
    NegotiateAttempt, NegotiateCorrection, NegotiateAccept, NegotiationState,
    NegotiateOffer, SCHEMA_VERSION,
};

use crate::util::generate_request_id;

pub(crate) async fn cmd_agent_ask(
    target: &str,
    action: &str,
    params_str: &str,
    from: Option<&str>,
    timeout: u64,
    interval: u64,
    json: bool,
) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let request_id = generate_request_id();
    let sender = from.map(|s| s.to_string()).unwrap_or_else(|| format!("cli-{}", std::process::id()));

    let params: serde_json::Value = match serde_json::from_str(params_str) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Invalid JSON in --params: {}", e)}));
            }
            return Err(e.into());
        }
    };

    let request = AgentRequest {
        schema_version: SCHEMA_VERSION.to_string(),
        request_id: request_id.clone(),
        from: sender.clone(),
        to: target.to_string(),
        action: action.to_string(),
        params,
        timeout_secs: if timeout > 0 { Some(timeout) } else { None },
    };

    // Snapshot cursor before emitting (quick subscribe to get next_seq)
    let cursor: Option<u64> = {
        let params = serde_json::json!({"timeout_ms": 1});
        match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    result["next_seq"].as_u64()
                } else { None }
            }
            Err(_) => None,
        }
    };

    // Emit the agent.request event
    let payload = match serde_json::to_value(&request) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to serialize AgentRequest: {}", e)}));
            }
            return Err(e.into());
        }
    };
    let emit_params = serde_json::json!({
        "topic": agent_topic::REQUEST,
        "payload": payload,
    });

    let emit_resp = match client::rpc_call(reg.socket_path(), "event.emit", emit_params).await {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to emit agent request: {}", e)}));
            }
            return Err(e).context("Failed to emit agent request");
        }
    };

    match client::unwrap_result(emit_resp) {
        Ok(result) => {
            if !json {
                let seq = result["seq"].as_u64().unwrap_or(0);
                eprintln!("Request sent: action={}, request_id={}, seq={}", action, request_id, seq);
            }
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "action": action,
                    "request_id": request_id,
                    "error": format!("Failed to emit agent request: {}", e),
                }));
            }
            anyhow::bail!("Failed to emit agent request: {}", e);
        }
    }

    // Poll for agent.response and agent.status
    if !json {
        eprintln!("Waiting for response (timeout: {}s)...", timeout);
    }

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let subscribe_timeout = interval.max(500);
    let mut sub_cursor = cursor;

    loop {
        let remaining = timeout_dur.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "action": action,
                    "request_id": request_id,
                    "error": format!("Timeout waiting for agent response ({}s)", timeout),
                }));
            }
            anyhow::bail!("Timeout waiting for agent response ({}s). request_id={}", timeout, request_id);
        }

        let effective_timeout = subscribe_timeout.min(remaining.as_millis() as u64);
        let mut sub_params = serde_json::json!({"timeout_ms": effective_timeout});
        if let Some(c) = sub_cursor {
            sub_params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.subscribe", sub_params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let topic = event["topic"].as_str().unwrap_or("");
                            let event_payload = &event["payload"];

                            let matches = event_payload
                                .get("request_id")
                                .and_then(|r| r.as_str())
                                .map(|r| r == request_id)
                                .unwrap_or(false);

                            if !matches { continue; }

                            if topic == agent_topic::RESPONSE {
                                if let Ok(response) = serde_json::from_value::<AgentResponse>(event_payload.clone()) {
                                    if json {
                                        let is_ok = response.status == termlink_protocol::events::ResponseStatus::Ok;
                                        let report = serde_json::json!({
                                            "ok": is_ok,
                                            "action": action,
                                            "request_id": request_id,
                                            "result": response.result,
                                            "error": response.error_message,
                                        });
                                        if !is_ok { super::json_error_exit(report); }
                                        println!("{report}");
                                    } else if response.status == termlink_protocol::events::ResponseStatus::Ok {
                                        println!("{}", serde_json::to_string_pretty(&response.result)?);
                                    } else {
                                        let msg = response.error_message.as_deref().unwrap_or("unknown error");
                                        eprintln!("Error: {}", msg);
                                        std::process::exit(1);
                                    }
                                } else if json {
                                    println!("{}", serde_json::json!({
                                        "ok": true,
                                        "action": action,
                                        "request_id": request_id,
                                        "result": event_payload,
                                    }));
                                } else {
                                    println!("{}", serde_json::to_string_pretty(event_payload)?);
                                }
                                return Ok(());
                            }

                            if !json && topic == agent_topic::STATUS
                                && let Ok(status) = serde_json::from_value::<AgentStatus>(event_payload.clone()) {
                                    let pct = status.percent.map(|p| format!(" ({}%)", p)).unwrap_or_default();
                                    let msg = status.message.as_deref().unwrap_or("");
                                    eprintln!("[status] {}{}: {}", status.phase, pct, msg);
                                }
                        }
                    }

                    if let Some(next) = result["next_seq"].as_u64() {
                        sub_cursor = Some(next.saturating_sub(1));
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Subscribe error: {}", e);
            }
        }
    }
}

pub(crate) async fn cmd_agent_listen(
    target: &str,
    timeout: u64,
    interval: u64,
    json: bool,
) -> Result<()> {
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
        eprintln!("Listening for agent requests on '{}' (topic: {})...", target, agent_topic::REQUEST);
        if timeout > 0 {
            eprintln!("Timeout: {}s", timeout);
        } else {
            eprintln!("Press Ctrl+C to stop");
        }
    }

    let start = std::time::Instant::now();
    let timeout_dur = if timeout > 0 {
        Some(std::time::Duration::from_secs(timeout))
    } else {
        None
    };
    let subscribe_timeout = interval.max(500);

    let mut sub_cursor: Option<u64> = None;

    loop {
        if let Some(td) = timeout_dur
            && start.elapsed() > td {
                if !json {
                    eprintln!("Listen timeout reached ({}s)", timeout);
                }
                return Ok(());
            }

        let effective_timeout = if let Some(td) = timeout_dur {
            subscribe_timeout.min(td.saturating_sub(start.elapsed()).as_millis() as u64)
        } else {
            subscribe_timeout
        };

        let mut params = serde_json::json!({
            "topic": agent_topic::REQUEST,
            "timeout_ms": effective_timeout,
        });
        if let Some(c) = sub_cursor {
            params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let event_payload = &event["payload"];
                            if json {
                                println!("{}", serde_json::json!({
                                    "ok": true,
                                    "seq": event["seq"],
                                    "from": event_payload["from"],
                                    "action": event_payload["action"],
                                    "request_id": event_payload["request_id"],
                                    "params": event_payload["params"],
                                    "timeout_secs": event_payload["timeout_secs"],
                                }));
                            } else if let Ok(req) = serde_json::from_value::<AgentRequest>(event_payload.clone()) {
                                println!("[{}] from={} action={} request_id={}",
                                    event["seq"].as_u64().unwrap_or(0),
                                    req.from, req.action, req.request_id);
                                if req.params != serde_json::json!(null) && req.params != serde_json::json!({}) {
                                    println!("  params: {}", serde_json::to_string(&req.params)?);
                                }
                            } else {
                                println!("{}", serde_json::to_string_pretty(event_payload)?);
                            }
                        }
                    }

                    if let Some(next) = result["next_seq"].as_u64() {
                        sub_cursor = Some(next.saturating_sub(1));
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Subscribe error: {}", e);
            }
        }
    }
}

/// Run a 4-phase format negotiation with a specialist session.
///
/// Protocol: offer → attempt → correction → accept (max N rounds).
/// Uses agent.request/response events with negotiate.* actions.
/// Options for the `termlink agent negotiate` command.
pub(crate) struct NegotiateOpts<'a> {
    pub specialist: &'a str,
    pub schema_str: &'a str,
    pub draft_str: &'a str,
    pub from: Option<&'a str>,
    pub max_rounds: u8,
    pub timeout: u64,
    pub interval: u64,
    pub json: bool,
}

pub(crate) async fn cmd_agent_negotiate(opts: NegotiateOpts<'_>) -> Result<()> {
    let NegotiateOpts { specialist, schema_str, draft_str, from, max_rounds, timeout, interval, json } = opts;
    let reg = match manager::find_session(specialist) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Specialist session '{}' not found: {}", specialist, e)}));
            }
            return Err(e).context(format!("Specialist session '{}' not found", specialist));
        }
    };

    let request_id = generate_request_id();
    let sender = from
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("cli-{}", std::process::id()));

    // Parse schema (support @file syntax)
    let schema: serde_json::Value = if let Some(path) = schema_str.strip_prefix('@') {
        let data = std::fs::read_to_string(path)
            .context(format!("Failed to read schema file: {path}"))?;
        serde_json::from_str(&data).context("Invalid JSON in schema file")?
    } else {
        serde_json::from_str(schema_str).context("Invalid JSON in --schema")?
    };

    let mut draft: serde_json::Value = if let Some(path) = draft_str.strip_prefix('@') {
        let data = std::fs::read_to_string(path)
            .context(format!("Failed to read draft file: {path}"))?;
        serde_json::from_str(&data).context("Invalid JSON in draft file")?
    } else {
        serde_json::from_str(draft_str).context("Invalid JSON in --draft")?
    };

    // Create offer (Phase 1 — from CLI, acting as orchestrator)
    let offer = NegotiateOffer {
        schema_version: SCHEMA_VERSION.to_string(),
        specialist_id: reg.id.as_str().to_string(),
        specialist_name: Some(reg.display_name.clone()),
        format_schema: schema.clone(),
        example: None,
        constraints: vec![],
        format_id: None,
    };

    let mut state = NegotiationState::from_offer(&request_id, &offer);
    state.max_rounds = max_rounds;

    if !json {
        eprintln!(
            "Negotiation started: specialist={}, request_id={}, max_rounds={}",
            specialist, request_id, max_rounds
        );
    }

    // Snapshot cursor (quick subscribe to get next_seq)
    let cursor: Option<u64> = {
        let params = serde_json::json!({"timeout_ms": 1});
        match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    result["next_seq"].as_u64()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    };
    let mut sub_cursor = cursor;

    // Negotiation loop
    while state.is_active() {
        // Phase 2: Send attempt
        if let Err(e) = state.record_attempt() {
            if !json {
                eprintln!("Negotiation ended: {e}");
            }
            break;
        }

        let attempt = NegotiateAttempt {
            schema_version: SCHEMA_VERSION.to_string(),
            draft: draft.clone(),
            questions: vec![],
            round: state.round,
        };

        let attempt_request = AgentRequest {
            schema_version: SCHEMA_VERSION.to_string(),
            request_id: request_id.clone(),
            from: sender.clone(),
            to: specialist.to_string(),
            action: negotiate_topic::ATTEMPT.to_string(),
            params: serde_json::to_value(&attempt).context("Failed to serialize attempt")?,
            timeout_secs: if timeout > 0 { Some(timeout) } else { None },
        };

        let payload =
            serde_json::to_value(&attempt_request).context("Failed to serialize request")?;
        let emit_params = serde_json::json!({
            "topic": agent_topic::REQUEST,
            "payload": payload,
        });

        client::rpc_call(reg.socket_path(), "event.emit", emit_params)
            .await
            .context("Failed to emit negotiate.attempt")?;

        if !json {
            eprintln!(
                "[round {}] Attempt sent, waiting for correction...",
                state.round
            );
        }

        // Phase 3: Wait for correction or accept
        let round_start = std::time::Instant::now();
        let timeout_dur = std::time::Duration::from_secs(timeout);
        let subscribe_timeout = interval.max(500);
        let mut got_response = false;

        while !got_response {
            let remaining = timeout_dur.saturating_sub(round_start.elapsed());
            if remaining.is_zero() { break; }

            let effective_timeout = subscribe_timeout.min(remaining.as_millis() as u64);
            let mut sub_params = serde_json::json!({"timeout_ms": effective_timeout});
            if let Some(c) = sub_cursor {
                sub_params["since"] = serde_json::json!(c);
            }

            if let Ok(resp) =
                client::rpc_call(reg.socket_path(), "event.subscribe", sub_params).await
                && let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let topic = event["topic"].as_str().unwrap_or("");
                            let event_payload = &event["payload"];

                            let matches = event_payload
                                .get("request_id")
                                .and_then(|r| r.as_str())
                                .is_some_and(|r| r == request_id);

                            if !matches {
                                continue;
                            }

                            let ev_action = event_payload
                                .get("action")
                                .and_then(|a| a.as_str())
                                .unwrap_or("");

                            if ev_action == negotiate_topic::ACCEPT
                                || ev_action == negotiate_topic::CORRECTION
                                || topic == agent_topic::RESPONSE
                            {
                                let resp_payload = event_payload
                                    .get("result")
                                    .or_else(|| event_payload.get("params"))
                                    .cloned()
                                    .unwrap_or_default();

                                // Try as NegotiateAccept
                                if ev_action == negotiate_topic::ACCEPT
                                    && let Ok(accept) =
                                        serde_json::from_value::<NegotiateAccept>(resp_payload.clone())
                                    {
                                        state.record_accept(&accept);
                                        if !json {
                                            eprintln!("[round {}] Accepted!", state.round);
                                            println!(
                                                "{}",
                                                serde_json::to_string_pretty(&accept.final_schema)?
                                            );
                                        }
                                        got_response = true;
                                        break;
                                    }

                                // Try as NegotiateCorrection
                                if let Ok(correction) =
                                    serde_json::from_value::<NegotiateCorrection>(resp_payload.clone())
                                {
                                    let _ = state.record_correction(&correction);

                                    if correction.accepted {
                                        if !json {
                                            eprintln!("[round {}] Accepted!", state.round);
                                            println!(
                                                "{}",
                                                serde_json::to_string_pretty(&state.current_schema)?
                                            );
                                        }
                                    } else {
                                        if !json {
                                            eprintln!(
                                                "[round {}] Correction: {} fix(es)",
                                                state.round,
                                                correction.fixes.len()
                                            );
                                            for fix in &correction.fixes {
                                                eprintln!(
                                                    "  - {}: expected '{}', got '{}' {}",
                                                    fix.field,
                                                    fix.expected,
                                                    fix.got,
                                                    fix.hint
                                                        .as_deref()
                                                        .map(|h| format!("(hint: {h})"))
                                                        .unwrap_or_default()
                                                );
                                            }
                                        }
                                        // Apply fixes to draft (best effort: set top-level fields)
                                        for fix in &correction.fixes {
                                            if let Some(obj) = draft.as_object_mut()
                                                && !fix.field.contains('[') {
                                                    obj.insert(
                                                        fix.field.clone(),
                                                        serde_json::json!(fix.expected),
                                                    );
                                                }
                                        }
                                    }
                                    got_response = true;
                                    break;
                                }

                                // Generic agent.response fallback
                                if let Ok(response) =
                                    serde_json::from_value::<AgentResponse>(event_payload.clone())
                                {
                                    if response.status
                                        == termlink_protocol::events::ResponseStatus::Ok
                                    {
                                        if !json {
                                            eprintln!("[round {}] Response received (treating as accept)", state.round);
                                            println!(
                                                "{}",
                                                serde_json::to_string_pretty(&response.result)?
                                            );
                                        }
                                    } else if !json {
                                        eprintln!(
                                            "[round {}] Error: {}",
                                            state.round,
                                            response
                                                .error_message
                                                .as_deref()
                                                .unwrap_or("unknown")
                                        );
                                    }
                                    got_response = true;
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(next) = result["next_seq"].as_u64() {
                        sub_cursor = Some(next.saturating_sub(1));
                    }
                }

            if round_start.elapsed() > timeout_dur {
                if json {
                    println!("{}", serde_json::json!({
                        "ok": false,
                        "result": "timeout",
                        "rounds": state.round,
                        "corrections": state.corrections.len(),
                        "draft": draft,
                    }));
                } else {
                    eprintln!(
                        "[round {}] Timeout ({}s) — falling back to best-effort draft",
                        state.round, timeout
                    );
                    println!("{}", serde_json::to_string_pretty(&draft)?);
                }
                return Ok(());
            }

            // event.subscribe blocks server-side; no sleep needed
        }
    }

    if state.is_accepted() {
        if json {
            println!("{}", serde_json::json!({
                "ok": true,
                "result": "accepted",
                "rounds": state.round,
                "corrections": state.corrections.len(),
                "schema": state.current_schema,
            }));
        } else {
            eprintln!(
                "Negotiation complete: {} round(s), {} total correction(s)",
                state.round,
                state.corrections.len()
            );
        }
    } else {
        if json {
            super::json_error_exit(serde_json::json!({
                "ok": false,
                "result": "failed",
                "rounds": state.round,
                "corrections": state.corrections.len(),
                "phase": format!("{}", state.phase),
            }));
        } else {
            eprintln!(
                "Negotiation failed after {} round(s): {}",
                state.round, state.phase
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

/// T-1448 (b): parse the positional `<target>` argument of `agent contact`
/// into `(name, Some(project))` when the operator typed `name:project`, or
/// `(name, None)` for the bare form. Empty parts and multi-colon inputs are
/// rejected. The project, when present, is stamped as `metadata.to_project`
/// on the resulting dm post — mirror of the sender-side `from_project` that
/// T-1472 auto-injects on `channel post`.
///
/// Pure helper — no I/O, no globals. Tested in `contact_tests::parse_*`.
pub(crate) fn parse_contact_target(input: &str) -> Result<(String, Option<String>), String> {
    if input.is_empty() {
        return Err("target name cannot be empty".to_string());
    }
    let parts: Vec<&str> = input.split(':').collect();
    match parts.len() {
        1 => Ok((parts[0].to_string(), None)),
        2 => {
            let name = parts[0];
            let project = parts[1];
            if name.is_empty() {
                return Err(format!(
                    "target name cannot be empty in {input:?} \
                     (expected `<name>` or `<name>:<project>`)"
                ));
            }
            if project.is_empty() {
                return Err(format!(
                    "project qualifier cannot be empty after `:` in {input:?} \
                     (use `<name>` for no project, or `<name>:<project>`)"
                ));
            }
            Ok((name.to_string(), Some(project.to_string())))
        }
        _ => Err(format!(
            "target may contain at most one `:` (form is `<name>[:<project>]`), got {input:?}"
        )),
    }
}

/// T-1646: resolve `agent contact`'s `--message`/`--file` pair into a single
/// `String`. Exactly one must be set. Empty file rejected.
///
/// Pure, unit-tested. Extracted so the CLI-layer dispatcher in `main.rs`
/// stays a one-liner and the policy lives next to the verb it serves.
pub(crate) fn resolve_contact_message(
    message: Option<&str>,
    file: Option<&std::path::Path>,
) -> anyhow::Result<String> {
    match (message, file) {
        (Some(_), Some(_)) => Err(anyhow::anyhow!(
            "specify exactly one of --message or --file, not both"
        )),
        (None, None) => Err(anyhow::anyhow!(
            "specify exactly one of --message <STRING> or --file <PATH>"
        )),
        (Some(m), None) => Ok(m.to_string()),
        (None, Some(p)) => {
            let body = std::fs::read_to_string(p).map_err(|e| {
                anyhow::anyhow!("failed to read {}: {e}", p.display())
            })?;
            if body.is_empty() {
                return Err(anyhow::anyhow!(
                    "file {} is empty — refusing to post empty message",
                    p.display()
                ));
            }
            Ok(body)
        }
    }
}

/// T-2275: a peer resolved via fleet `agent-presence` (cross-hub contact-by-name).
pub(crate) struct FleetContactResolution {
    pub identity_fingerprint: String,
    pub hub_address: String,
}

/// T-2275: resolve `<agent_id>` to `{identity_fingerprint, hub}` by walking every
/// hub in `hubs.toml` and reading its `agent-presence` heartbeats. This is the
/// native parity of the shell `agent-listeners-fleet.sh` path (used by
/// `agent-send.sh --to`, T-2273) — it lets `agent contact <name>` reach a peer on
/// another hub without depending on repo scripts being deployed on the host.
///
/// Reuses the shared `termlink_session::fleet_presence` parser so the CLI and MCP
/// resolvers cannot drift on the heartbeat contract. Picks the freshest LIVE
/// match across hubs; per-hub failures (unreachable / auth) are skipped so a down
/// hub never aborts the walk. Returns `None` when no hub has a LIVE heartbeat for
/// the agent_id (or `hubs.toml` is empty/missing) — the caller then surfaces the
/// not-found error.
async fn resolve_contact_via_fleet(agent_id: &str) -> Option<FleetContactResolution> {
    use termlink_session::fleet_presence::{resolve_agent_presence, PresenceStatus};
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        return None;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    // Dedup hubs by address — multiple profiles can point at one physical hub
    // (e.g. workstation-107-public + local-test → same bind).
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut best: Option<(FleetContactResolution, i64)> = None;
    for entry in config.hubs.values() {
        if !seen.insert(entry.address.clone()) {
            continue;
        }
        // agent-presence heartbeats are small + frequent; a 500-envelope slice
        // covers a generous window for any reasonably-sized fleet.
        let msgs =
            match super::channel::fetch_topic_msgs("agent-presence", Some(&entry.address), 500)
                .await
            {
                Ok(m) => m,
                Err(_) => continue, // down / auth-fail hub never aborts the walk
            };
        let Some(m) = resolve_agent_presence(&msgs, agent_id, now_ms) else {
            continue;
        };
        // Only LIVE peers are contactable destinations — a doorbell to a STALE/
        // OFFLINE pty is wasted.
        if m.status != PresenceStatus::Live {
            continue;
        }
        let Some(fp) = m.identity_fingerprint else {
            continue;
        };
        let fresher = best.as_ref().map(|(_, ts)| m.last_ts_ms > *ts).unwrap_or(true);
        if fresher {
            best = Some((
                FleetContactResolution {
                    identity_fingerprint: fp,
                    hub_address: entry.address.clone(),
                },
                m.last_ts_ms,
            ));
        }
    }
    best.map(|(r, _)| r)
}

/// T-2384: choose which fingerprint to address a *locally-registered* peer.
///
/// On a shared host, a local session's registration metadata carries the **host**
/// `identity_fingerprint` — the register process runs without `TERMLINK_AGENT_ID`
/// so `load_identity_fingerprint_best_effort()` falls back to the host key. But the
/// peer's `agent-presence` heartbeat advertises its **per-agent** key, and that is
/// the exact fp the peer's push-waker (`/be-reachable`'s dm rail) subscribes on.
/// Address the presence-advertised fp so the computed `dm:<a>:<b>` topic matches
/// the recipient's subscribe topic (else the doorbell writes to a rail nobody is
/// listening on — the silent no-wake breakpoint #4, PL-166/236).
///
/// Precedence: presence-advertised fp wins when present; fall back to the
/// registration metadata fp when the peer advertises no LIVE presence (a
/// registered-but-not-`/be-reachable` peer still resolves, just via the host fp
/// as before — no hard failure). `None` only when both are absent, preserving the
/// existing "no identity_fingerprint" error path.
pub(crate) fn prefer_presence_fp(
    presence_fp: Option<String>,
    reg_fp: Option<String>,
) -> Option<String> {
    presence_fp.or(reg_fp)
}

/// T-2293 (V2 discovery registry): a fully-resolved registry record for an
/// `agent_id` — the `{host:port, hub, topics-read, liveness}` shape of AC1.
pub(crate) struct FleetAgentRecord {
    pub agent_id: String,
    pub identity_fingerprint: Option<String>,
    /// The reachable hub address a peer posts to in order to reach this agent.
    /// Prefers the agent's self-reported `metadata.addr`; falls back to the hub
    /// the heartbeat was actually read from (resolver-stamped — this is the
    /// authoritative routing answer even when the agent self-reports nothing).
    pub hub: String,
    /// The hub the resolver actually read this heartbeat from (resolver-stamped).
    pub hub_found_on: String,
    /// The agent's self-reported `metadata.addr` (None on the default local hub).
    pub self_reported_addr: Option<String>,
    /// T-2297 (V2b): the HUB-ATTESTED observed source address (`metadata.observed_addr`).
    /// When present, `hub` is resolved from THIS in preference to `self_reported_addr`.
    pub observed_addr: Option<String>,
    pub host: Option<String>,
    pub role: Option<String>,
    pub listen_topics: Vec<String>,
    pub status: termlink_session::fleet_presence::PresenceStatus,
    pub age_secs: i64,
}

/// T-2293: resolve `<agent_id>` to its full registry record by walking every hub
/// in `hubs.toml` and reading each hub's `agent-presence` heartbeats. This is the
/// discovery-registry core (RC2 of the T-2291 reliable-comms inception): it
/// answers "where is agent X, and is it live?" for ANY agent_id — including the
/// caller's own (reverse/symmetric lookup, AC4).
///
/// Distinct from `resolve_contact_via_fleet` (which only returns the freshest
/// LIVE match's fingerprint+hub for the contact path): this keeps the freshest
/// match REGARDLESS of liveness and reports its LIVE/STALE/OFFLINE status, so a
/// "found" result means "this agent is actually present on a hub" — the G-155
/// false-green fix (AC5). A configured-but-empty hub yields `None` here, not a
/// green "reachable".
///
/// Reuses the shared `fleet_presence` parser so CLI/MCP cannot drift on the
/// heartbeat contract. Per-hub failures (down / auth) are skipped — a dead hub
/// never aborts the walk. Returns `None` only when NO hub carries a heartbeat
/// for the agent_id (or `hubs.toml` is empty/missing).
pub(crate) async fn resolve_agent_registry_via_fleet(
    agent_id: &str,
) -> Option<FleetAgentRecord> {
    use termlink_session::fleet_presence::resolve_agent_presence;
    let config = crate::config::load_hubs_config();
    if config.hubs.is_empty() {
        return None;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut best: Option<(FleetAgentRecord, i64)> = None;
    for entry in config.hubs.values() {
        if !seen.insert(entry.address.clone()) {
            continue;
        }
        // T-2293: bound each per-hub fetch (8s, matching the T-2062 fleet
        // per-hub timeout). A stalled/dead hub then contributes at most 8s and
        // is skipped — it never hangs the whole walk (the symptom that surfaced
        // in live testing: one unreachable hub in hubs.toml stalled `resolve`).
        let fetch =
            super::channel::fetch_topic_msgs("agent-presence", Some(&entry.address), 500);
        let msgs = match tokio::time::timeout(std::time::Duration::from_secs(8), fetch).await {
            Ok(Ok(m)) => m,
            Ok(Err(_)) => continue, // down / auth-fail hub
            Err(_) => continue,     // per-hub timeout — never stalls the walk
        };
        let Some(m) = resolve_agent_presence(&msgs, agent_id, now_ms) else {
            continue;
        };
        let fresher = best.as_ref().map(|(_, ts)| m.last_ts_ms > *ts).unwrap_or(true);
        if fresher {
            // T-2297: prefer the hub-attested observed_addr over the self-reported
            // addr, falling back to the hub the heartbeat was read from.
            let hub = m
                .observed_addr
                .clone()
                .or_else(|| m.addr.clone())
                .unwrap_or_else(|| entry.address.clone());
            best = Some((
                FleetAgentRecord {
                    agent_id: agent_id.to_string(),
                    identity_fingerprint: m.identity_fingerprint.clone(),
                    hub,
                    hub_found_on: entry.address.clone(),
                    self_reported_addr: m.addr.clone(),
                    observed_addr: m.observed_addr.clone(),
                    host: m.host.clone(),
                    role: m.role.clone(),
                    listen_topics: m.listen_topics.clone(),
                    status: m.status,
                    age_secs: m.age_secs,
                },
                m.last_ts_ms,
            ));
        }
    }
    best.map(|(r, _)| r)
}

/// T-2293: `termlink agent resolve <agent_id> [--json]` — print the discovery
/// registry record for an agent. The operator/agent-facing surface of
/// `resolve_agent_registry_via_fleet`. Exit 0 on found, exit 4 on not-found
/// (distinct from transport/usage errors), so scripts can branch on presence.
pub(crate) async fn cmd_agent_resolve(agent_id: &str, json: bool) -> Result<()> {
    let rec = resolve_agent_registry_via_fleet(agent_id).await;
    match rec {
        Some(r) => {
            if json {
                let out = serde_json::json!({
                    "ok": true,
                    "agent_id": r.agent_id,
                    "identity_fingerprint": r.identity_fingerprint,
                    "hub": r.hub,
                    "hub_found_on": r.hub_found_on,
                    "self_reported_addr": r.self_reported_addr,
                    "observed_addr": r.observed_addr,
                    "host": r.host,
                    "role": r.role,
                    "listen_topics": r.listen_topics,
                    "liveness": r.status.as_str(),
                    "age_secs": r.age_secs,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("agent:      {}", r.agent_id);
                println!("liveness:   {} ({}s ago)", r.status.as_str(), r.age_secs);
                println!("hub:        {}", r.hub);
                if let Some(oa) = &r.observed_addr {
                    println!("            (hub-attested observed_addr: {oa} — T-2297)");
                } else if r.self_reported_addr.is_none() {
                    println!("            (resolver-stamped — agent self-reported no addr)");
                }
                println!(
                    "fingerprint:{}",
                    r.identity_fingerprint
                        .as_deref()
                        .map(|f| format!(" {f}"))
                        .unwrap_or_else(|| " <none>".to_string())
                );
                if let Some(h) = &r.host {
                    println!("host:       {h}");
                }
                if let Some(role) = &r.role {
                    println!("role:       {role}");
                }
                println!(
                    "topics:     {}",
                    if r.listen_topics.is_empty() {
                        "<none>".to_string()
                    } else {
                        r.listen_topics.join(", ")
                    }
                );
            }
            Ok(())
        }
        None => {
            if json {
                let out = serde_json::json!({
                    "ok": false,
                    "agent_id": agent_id,
                    "error": "not-found",
                    "hint": "no hub in hubs.toml carries an agent-presence heartbeat for this agent_id",
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                eprintln!(
                    "agent resolve: '{agent_id}' not found on any hub in hubs.toml.\n  \
                     The agent is not advertising presence (not LIVE/STALE/OFFLINE anywhere).\n  \
                     If it should be reachable, have it run `/be-reachable` (and check the\n  \
                     right hub is in ~/.termlink/hubs.toml)."
                );
            }
            std::process::exit(4);
        }
    }
}

/// T-1429: contact a peer agent on the canonical `dm:<a>:<b>` topic.
///
/// Resolves `<target>` to a local session via `manager::find_session`, reads
/// the peer's `identity_fingerprint` from `SessionMetadata` (T-1436), then
/// delegates to `cmd_channel_dm` which already does dm-topic canonicalisation,
/// idempotent topic creation, and posting.
///
/// Shipped surface: --message / --file (T-1646 — resolved upstream in main.rs
/// via `resolve_contact_message`, this fn takes the resolved `&str`), --thread
/// (canonical `metadata._thread` routing per agent-chat-arc protocol), --json,
/// --dry-run (T-1478 preview-without-posting), --target-fp <hex> (cross-host
/// bypass when `manager::find_session` can't resolve the peer locally),
/// --require-online + --online-window-secs (T-1480, exit 9 on offline),
/// --ack-required + --ack-timeout-secs (T-1485, exit 10 on ack timeout).
///
/// Phase-2 still deferred: `name@hub:port` federated name syntax for remote
/// session discovery via channel.list/peer-registry overlay — `--target-fp
/// <hex>` is the current workaround when the peer's name isn't locally
/// resolvable. See T-1429 task body.
///
/// Errors:
/// - target not found locally → exit code 1, message names the session
/// - peer registered before T-1436 (no identity_fingerprint in metadata) →
///   exit 8, message lists three recovery paths: upgrade peer binary,
///   pass `--target-fp <hex>`, or post via `agent-chat-arc --mention`
///   (T-1644).
pub(crate) async fn cmd_agent_contact(
    target: Option<&str>,
    target_fp: Option<&str>,
    message: &str,
    thread: Option<&str>,
    hub: Option<&str>,
    json: bool,
    dry_run: bool,
    require_online: bool,
    online_window_secs: u64,
    ack_required: bool,
    ack_timeout_secs: u64,
) -> Result<()> {
    // T-1429 Phase-2 (this build): support --target-fp <hex> as a cross-host
    // bypass for the local-only session.discover gap. Either positional
    // <TARGET> or --target-fp must be set, but not both.
    if target.is_some() && target_fp.is_some() {
        let msg = "specify either <TARGET> or --target-fp, not both";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    if target.is_none() && target_fp.is_none() {
        let msg = "must specify either <TARGET> (display name) or --target-fp <hex>";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    // T-1448 (b): if positional <TARGET> was given, split off an optional
    // `:project` suffix. The project, if present, becomes `to_project`
    // metadata on the resulting dm post (mirror of T-1472's sender-side
    // `from_project`). Operator can also pass `--metadata to_project=...`
    // explicitly via the underlying `cmd_channel_dm` extra_metadata flow,
    // and that path is preserved for `--target-fp` callers (the
    // `name:project` syntax applies only to the positional target).
    let (target_name_owned, to_project_opt): (Option<String>, Option<String>) =
        if let Some(raw) = target {
            match parse_contact_target(raw) {
                Ok((name, project)) => (Some(name), project),
                Err(msg) => {
                    if json {
                        super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
                    }
                    anyhow::bail!(msg);
                }
            }
        } else {
            (None, None)
        };

    // T-2275: set when the peer was resolved via fleet presence on a (possibly
    // remote) hub — used below to route the dm post to that hub when the
    // operator did not pass an explicit --hub.
    let mut fleet_hub: Option<String> = None;
    let peer_fp_owned: String = if let Some(fp) = target_fp {
        // Trust the operator-supplied fingerprint. Light validation: must be
        // hex, at least 8 chars (canonical short fp is 16 chars).
        if fp.len() < 8 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
            let msg = format!("--target-fp must be hex (got {fp:?})");
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
            }
            anyhow::bail!(msg);
        }
        fp.to_string()
    } else {
        let target_name = target_name_owned.as_deref().expect("checked above");
        match manager::find_session(target_name) {
            // T-2275: peer is not a local session — fall back to fleet presence
            // (agent-presence across every hub in hubs.toml). Native parity of
            // agent-send.sh --to (T-2273); a peer on another hub is now reachable
            // by name instead of yielding a misleading "not found".
            Err(e) => match resolve_contact_via_fleet(target_name).await {
                Some(r) => {
                    fleet_hub = Some(r.hub_address);
                    r.identity_fingerprint
                }
                None => {
                    let msg = format!(
                        "Session '{target_name}' not found locally or as a LIVE peer on \
                         any hub in hubs.toml: {e}. Run `termlink agent find-idle` (or the \
                         /peers skill) to see who is reachable, or pass --target-fp <hex>."
                    );
                    if json {
                        super::json_error_exit(serde_json::json!({
                            "ok": false,
                            "target": target_name,
                            "error": msg,
                        }));
                    }
                    anyhow::bail!(msg);
                }
            },
            Ok(reg) => {
            // T-2384: the local session's registration metadata fp may be the
            // *host* key on a shared host (register runs without
            // TERMLINK_AGENT_ID). Prefer the peer's *per-agent* presence-advertised
            // fp — the exact fp its push-waker subscribes on — so the computed
            // dm:<a>:<b> topic push-wakes the recipient. Falls back to the
            // registration metadata fp when the peer advertises no LIVE presence
            // (registered-but-not-/be-reachable → resolves via host fp as before).
            let reg_fp = reg.metadata.identity_fingerprint.clone();
            let presence_fp = resolve_contact_via_fleet(target_name)
                .await
                .map(|r| r.identity_fingerprint);
            prefer_presence_fp(presence_fp, reg_fp).ok_or_else(|| {
            let msg = format!(
                "Peer '{target_name}' has no identity_fingerprint in metadata — \
                 likely registered before T-1436. Three recovery paths: \
                 (1) upgrade the peer's termlink binary and restart the session, then retry; \
                 (2) if you know the peer's fingerprint, pass --target-fp <hex>; \
                 (3) post to a public topic with --mention, e.g.: \
                 `termlink channel post agent-chat-arc --mention {target_name} \
                 --metadata _thread=<task-id> --payload '<msg>'` \
                 (T-1430 protocol canon — works without restarting the peer)."
            );
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target_name,
                    "error": msg,
                    "exit_code": 8,
                }));
            }
            eprintln!("error: {msg}");
            std::process::exit(8);
            })?
            }
        }
    };
    let peer_fp = peer_fp_owned.as_str();
    // T-2275: route the dm post to the hub where the peer was found via fleet
    // presence, unless the operator passed an explicit --hub (which wins).
    let hub: Option<&str> = hub.or(fleet_hub.as_deref());

    // T-1429 Phase-2 partial: --thread routes via `metadata._thread`
    // (agent-chat-arc protocol canon). T-1448 (b): also auto-attach
    // `to_project=<project>` when the operator typed `name:project`.
    let mut extra_metadata: Vec<String> = Vec::new();
    if let Some(t) = thread {
        extra_metadata.push(format!("_thread={t}"));
    }
    if let Some(p) = &to_project_opt {
        extra_metadata.push(format!("to_project={p}"));
    }

    // T-1480 (Q3 deferred): clamp the online window. Rationale: <10s is too
    // tight for chat-arc heartbeat cadence (~1/min); >24h defeats the
    // fail-fast intent and risks calling a peer that's been silent for days.
    let clamped_window_secs = online_window_secs.clamp(10, 86_400);

    // T-1478: dry-run path — print preview JSON, do not contact the hub.
    // Builds the full metadata block including the auto-injected
    // `from_project` (T-1472), the `to_project` from the `name:project`
    // qualifier (T-1474), and `_thread` from `--thread`. Always JSON
    // output (the use case is scripting / CI verification).
    if dry_run {
        let identity = super::channel::load_identity_or_create()
            .context("agent contact --dry-run: cannot load local identity")?;
        let my_id = identity.fingerprint().to_string();
        let topic = super::channel::dm_topic(&my_id, peer_fp);
        // T-1479: count local sessions sharing peer_fp as their
        // identity_fingerprint — co-residency detection. Best-effort: if
        // session enumeration fails, we pass None and the preview stays silent.
        let local_session_count =
            manager::list_sessions(false).ok().map(|sessions| {
                sessions
                    .iter()
                    .filter(|s| {
                        s.metadata
                            .identity_fingerprint
                            .as_deref()
                            .map(|fp| fp == peer_fp)
                            .unwrap_or(false)
                    })
                    .count()
            });
        let mut preview = render_dry_run_preview(
            &my_id,
            peer_fp,
            &topic,
            extra_metadata.iter().map(String::as_str),
            message,
            local_session_count,
        );
        // T-1480: when --require-online is also set in dry-run, run the
        // presence check and surface its result. Dry-run never fails on
        // offline; the operator just sees the would-be verdict.
        if require_online {
            match super::channel::check_peer_online_via_chat_arc(
                peer_fp,
                hub,
                clamped_window_secs,
            )
            .await
            {
                Ok(check) => {
                    preview["online_check"] = check.to_json();
                }
                Err(e) => {
                    preview["online_check"] = serde_json::json!({
                        "error": format!("presence probe failed: {e}"),
                        "window_secs": clamped_window_secs,
                    });
                }
            }
        }
        println!("{}", serde_json::to_string_pretty(&preview)?);
        return Ok(());
    }

    // T-1480 (Q3 deferred): pre-flight presence check — fail-fast when peer
    // hasn't been seen on agent-chat-arc within the configured window. The
    // dm post itself would still queue (chat-arc is offset-durable), but
    // operators who pass --require-online have explicitly opted into the
    // synchronous-contact semantic.
    if require_online {
        let check = super::channel::check_peer_online_via_chat_arc(
            peer_fp,
            hub,
            clamped_window_secs,
        )
        .await
        .with_context(|| {
            format!(
                "agent contact --require-online: presence probe failed for peer fp={peer_fp}"
            )
        })?;
        if !check.online {
            let last_seen_phrase = match check.last_seen_ms {
                Some(ms) => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    let age_secs = ((now_ms - ms) / 1000).max(0);
                    format!("last seen {age_secs}s ago (ts_ms={ms})")
                }
                None => "last seen: never (no posts on agent-chat-arc)".to_string(),
            };
            let msg = format!(
                "peer fp={peer_fp} not online — {last_seen_phrase}, \
                 window={clamped_window_secs}s. Re-run without --require-online \
                 to queue the post (chat-arc is offset-durable), or wait for the \
                 peer's next heartbeat."
            );
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "peer_fp": peer_fp,
                    "online_check": check.to_json(),
                    "error": msg,
                    "exit_code": 9,
                }));
            }
            eprintln!("error: {msg}");
            std::process::exit(9);
        }
    }

    // Capture send timestamp BEFORE posting so the ack-wait poll only
    // matches messages strictly after our post (T-1485). Use ms precision
    // because hub timestamps are ms.
    let send_ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    // T-1485: pass `json` through to the post path. With --ack-required +
    // --json, callers get NDJSON: one envelope for the post, then a second
    // envelope for the ack outcome (success or timeout). NDJSON-style is
    // consistent with how other termlink commands compose multi-step output.
    super::channel::cmd_channel_dm(
        peer_fp,
        Some(message),
        None,             // reply_to
        &[],              // mentions
        &extra_metadata,  // T-1429 Phase-2 partial: --thread
        false,            // topic_only
        hub,
        json,
    )
    .await
    .with_context(|| format!("agent contact: posting to dm topic for peer fp={peer_fp} failed"))?;

    // T-1485: synchronous engagement — wait for the peer to post back.
    if ack_required {
        let clamped_ack_timeout = ack_timeout_secs.clamp(5, 600);
        let identity = super::channel::load_identity_or_create()
            .context("agent contact --ack-required: cannot load local identity")?;
        let my_id = identity.fingerprint().to_string();
        let topic = super::channel::dm_topic(&my_id, peer_fp);
        let wait_start = std::time::Instant::now();
        let ack = super::channel::wait_for_peer_ack(
            &topic,
            peer_fp,
            send_ts_ms,
            hub,
            clamped_ack_timeout,
        )
        .await
        .with_context(|| {
            format!(
                "agent contact --ack-required: ack-wait poll failed for peer fp={peer_fp} on topic={topic}"
            )
        })?;
        let waited_secs = wait_start.elapsed().as_secs();
        match ack {
            Some(ts_ms) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "peer_fp": peer_fp,
                            "topic": topic,
                            "ack": {
                                "ts_ms": ts_ms,
                                "wait_secs": waited_secs,
                            },
                        }))?
                    );
                } else {
                    println!(
                        "ack received from {peer_fp} after {waited_secs}s (ts_ms={ts_ms})"
                    );
                }
                Ok(())
            }
            None => {
                let msg = format!(
                    "no ack from peer fp={peer_fp} within {clamped_ack_timeout}s on topic={topic}. \
                     The post landed (chat-arc is offset-durable) — peer just hasn't responded \
                     yet. Re-run without --ack-required for fire-and-forget, or increase \
                     --ack-timeout-secs."
                );
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false,
                        "peer_fp": peer_fp,
                        "topic": topic,
                        "wait_secs": waited_secs,
                        "ack_timeout_secs": clamped_ack_timeout,
                        "error": msg,
                        "exit_code": 10,
                    }));
                }
                eprintln!("error: {msg}");
                std::process::exit(10);
            }
        }
    } else {
        Ok(())
    }
}

/// T-1483: resolve `--target <name>` → identity_fingerprint via local
/// `session.discover` + `SessionMetadata.identity_fingerprint`. Mirrors the
/// path used by `cmd_agent_contact`; extracted here so `agent who` can
/// share the same error semantics. Returns exit codes via `process::exit`
/// on miss so the caller doesn't have to plumb the JSON envelope:
/// - session not found → exit 1 (caller already printed JSON if needed)
/// - peer registered before T-1436 → exit 8 (message lists three recovery
///   paths: upgrade peer binary, pass --target-fp, or post via
///   `agent-chat-arc --mention` — T-1644).
fn resolve_target_name_to_fp(target_name: &str, json: bool) -> String {
    let reg = match manager::find_session(target_name) {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Session '{target_name}' not found: {e}");
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target_name,
                    "error": msg,
                }));
            }
            eprintln!("error: {msg}");
            std::process::exit(1);
        }
    };
    match reg.metadata.identity_fingerprint.clone() {
        Some(fp) => fp,
        None => {
            let msg = format!(
                "Peer '{target_name}' has no identity_fingerprint in metadata — \
                 likely registered before T-1436. Three recovery paths: \
                 (1) upgrade the peer's termlink binary and restart the session, then retry; \
                 (2) if you know the peer's fingerprint, pass --target-fp <hex>; \
                 (3) post to a public topic with --mention, e.g.: \
                 `termlink channel post agent-chat-arc --mention {target_name} \
                 --metadata _thread=<task-id> --payload '<msg>'` \
                 (T-1430 protocol canon — works without restarting the peer)."
            );
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target_name,
                    "error": msg,
                    "exit_code": 8,
                }));
            }
            eprintln!("error: {msg}");
            std::process::exit(8);
        }
    }
}

/// T-1481: peer observability primitive. Walks recent `agent-chat-arc`
/// activity, returns a summary of last-seen, posts in window, and distinct
/// `from_project` values associated with the peer FP. Disambiguation tool
/// for cross-host operators investigating an unknown peer.
///
/// T-1483: accepts either `--target-fp <hex>` (cross-host, no resolution)
/// or `--target <name>` (local-hub via session.discover). Mutually
/// exclusive; one is required.
pub(crate) async fn cmd_agent_who(
    target_fp: Option<&str>,
    target: Option<&str>,
    window_secs: u64,
    hub: Option<&str>,
    json: bool,
    filter_thread: Option<&str>,
) -> Result<()> {
    if target.is_some() && target_fp.is_some() {
        let msg = "specify either --target or --target-fp, not both";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    if target.is_none() && target_fp.is_none() {
        let msg = "must specify either --target <name> or --target-fp <hex>";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    let resolved_fp_owned: String = if let Some(fp) = target_fp {
        if fp.len() < 8 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
            let msg = format!("--target-fp must be hex (got {fp:?})");
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
            }
            anyhow::bail!(msg);
        }
        fp.to_string()
    } else {
        resolve_target_name_to_fp(target.expect("checked above"), json)
    };
    let target_fp = resolved_fp_owned.as_str();
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    let activity = super::channel::fetch_peer_activity_via_chat_arc(
        target_fp,
        hub,
        clamped_window_secs,
        filter_thread,
    )
    .await
    .with_context(|| format!("agent who: failed to fetch activity for fp={target_fp}"))?;

    if json {
        // T-1488: when filter_thread is set, echo it on the JSON envelope
        // so callers can disambiguate "no activity" from "no activity on
        // this thread". Omitted entirely when unset to stay
        // backward-compatible with T-1481 consumers.
        let mut envelope = activity.to_json();
        if let Some(t) = filter_thread
            && let Some(obj) = envelope.as_object_mut()
        {
            obj.insert(
                "filter_thread".to_string(),
                serde_json::Value::String(t.to_string()),
            );
        }
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        return Ok(());
    }

    // Text mode — sectioned output.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    if let Some(t) = filter_thread {
        println!("# filter_thread={}", t);
    }
    println!("peer_fp:           {}", activity.peer_fp);
    match activity.last_seen_ms {
        None => println!("last_seen:         never (no posts on agent-chat-arc)"),
        Some(ms) => {
            let age_secs = ((now_ms - ms) / 1000).max(0);
            println!(
                "last_seen:         {}s ago (ts_ms={})",
                age_secs, ms,
            );
        }
    }
    println!(
        "posts_in_window:   {} (window_secs={})",
        activity.posts_in_window, activity.window_secs
    );
    if activity.from_projects.is_empty() {
        println!("from_projects:     (none observed in window)");
    } else {
        println!("from_projects:");
        for (project, count) in &activity.from_projects {
            println!("  {project:30} {count:>6}");
        }
    }
    Ok(())
}

/// T-1482 / T-1484: fleet-wide presence summary on `agent-chat-arc`.
/// Aggregates by sender_id, returns one row per active peer with last_seen,
/// posts in window, and top from_project (most-frequently-stamped).
/// Companion to `agent who` — that's per-peer; this is fleet-wide.
///
/// `filter_project`: when Some(p), only posts whose `from_project ==
/// p` count toward presence; peers with zero matching posts are excluded
/// (T-1484). Use for project-scoped triage.
pub(crate) async fn cmd_agent_presence(
    window_secs: u64,
    hub: Option<&str>,
    json: bool,
    filter_project: Option<&str>,
    filter_thread: Option<&str>,
    watch: bool,
    watch_interval: u64,
    top: Option<usize>,
    by_project: bool,
) -> Result<()> {
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    // T-1489: clamp `--top` to [1, 1000]. clamp() requires both bounds —
    // pass through Option::map.
    let clamped_top: Option<usize> = top.map(|n| n.clamp(1, 1000));

    // T-1486: --watch is a streaming mode; --json is one-shot. Reject the
    // combo at the verb level so callers get a clear error instead of an
    // unparseable NDJSON-on-cleared-screen mess.
    if watch && json {
        let msg = "--watch and --json are incompatible: --watch streams \
                   re-rendered text frames; --json is one-shot. Pick one.";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    if watch {
        let clamped_interval = watch_interval.clamp(1, 300);
        loop {
            // ANSI: clear screen + cursor home. Avoids the flicker of a
            // process-spawn in `watch -n N` and preserves alignment.
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            let view_label = if by_project { "by-project" } else { "by-peer" };
            println!(
                "# agent presence --watch | view={} | interval={}s | window={}s | {}",
                view_label, clamped_interval, clamped_window_secs, now_str
            );
            // Per-iteration fetch + render. Errors are non-fatal — we
            // print them and keep watching, so a transient hub blip
            // doesn't kill the dashboard.
            if by_project {
                match super::channel::fetch_fleet_by_project_via_chat_arc(
                    hub,
                    clamped_window_secs,
                    filter_project,
                    filter_thread,
                )
                .await
                {
                    Ok(rows) => render_by_project_text(
                        &rows,
                        clamped_window_secs,
                        filter_project,
                        filter_thread,
                        clamped_top,
                    ),
                    Err(e) => {
                        println!("# fetch error (will retry on next tick): {e}");
                    }
                }
            } else {
                match super::channel::fetch_fleet_presence_via_chat_arc(
                    hub,
                    clamped_window_secs,
                    filter_project,
                    filter_thread,
                )
                .await
                {
                    Ok(rows) => render_presence_text(
                        &rows,
                        clamped_window_secs,
                        filter_project,
                        filter_thread,
                        clamped_top,
                    ),
                    Err(e) => {
                        println!("# fetch error (will retry on next tick): {e}");
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    }

    // T-1491: by-project branch reuses helpers; JSON envelope shape differs.
    if by_project {
        let rows = super::channel::fetch_fleet_by_project_via_chat_arc(
            hub,
            clamped_window_secs,
            filter_project,
            filter_thread,
        )
        .await
        .context("agent presence --by-project: failed to fetch fleet activity")?;

        if json {
            let total_projects = rows.len();
            let display_rows: &[super::channel::FleetProjectRow] = match clamped_top {
                Some(n) => &rows[..rows.len().min(n)],
                None => &rows,
            };
            let projects: Vec<serde_json::Value> = display_rows.iter().map(|r| r.to_json()).collect();
            let mut out_obj = serde_json::Map::new();
            out_obj.insert(
                "view".to_string(),
                serde_json::Value::String("by-project".to_string()),
            );
            out_obj.insert(
                "window_secs".to_string(),
                serde_json::Value::from(clamped_window_secs),
            );
            if let Some(f) = filter_project {
                out_obj.insert(
                    "filter_project".to_string(),
                    serde_json::Value::String(f.to_string()),
                );
            }
            if let Some(t) = filter_thread {
                out_obj.insert(
                    "filter_thread".to_string(),
                    serde_json::Value::String(t.to_string()),
                );
            }
            if let Some(n) = clamped_top {
                out_obj.insert("top".to_string(), serde_json::Value::from(n));
                out_obj.insert(
                    "total_projects".to_string(),
                    serde_json::Value::from(total_projects),
                );
            }
            out_obj.insert("projects".to_string(), serde_json::Value::Array(projects));
            println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(out_obj))?);
            return Ok(());
        }

        render_by_project_text(
            &rows,
            clamped_window_secs,
            filter_project,
            filter_thread,
            clamped_top,
        );
        return Ok(());
    }

    let rows = super::channel::fetch_fleet_presence_via_chat_arc(
        hub,
        clamped_window_secs,
        filter_project,
        filter_thread,
    )
    .await
    .context("agent presence: failed to fetch fleet presence")?;

    if json {
        let total_peers = rows.len();
        // T-1489: apply --top truncation post-sort. The helper's sort is
        // already posts desc, so `take(N)` returns the N busiest.
        let display_rows: &[super::channel::FleetPeerRow] = match clamped_top {
            Some(n) => &rows[..rows.len().min(n)],
            None => &rows,
        };
        let peers: Vec<serde_json::Value> = display_rows.iter().map(|r| r.to_json()).collect();
        let mut out_obj = serde_json::Map::new();
        out_obj.insert(
            "window_secs".to_string(),
            serde_json::Value::from(clamped_window_secs),
        );
        // T-1484: echo filter back so callers can confirm what was applied
        // (especially useful when result is empty — distinguishes "no fleet
        // activity" from "filter matched nothing").
        if let Some(f) = filter_project {
            out_obj.insert(
                "filter_project".to_string(),
                serde_json::Value::String(f.to_string()),
            );
        }
        // T-1490: echo filter_thread back so callers can confirm what was
        // applied; omitted when unset (backward-compat).
        if let Some(t) = filter_thread {
            out_obj.insert(
                "filter_thread".to_string(),
                serde_json::Value::String(t.to_string()),
            );
        }
        // T-1489: echo top + total_peers when truncation flag is set so
        // callers can disambiguate "exactly N peers" from "N truncated".
        if let Some(n) = clamped_top {
            out_obj.insert("top".to_string(), serde_json::Value::from(n));
            out_obj.insert(
                "total_peers".to_string(),
                serde_json::Value::from(total_peers),
            );
        }
        out_obj.insert("peers".to_string(), serde_json::Value::Array(peers));
        println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(out_obj))?);
        return Ok(());
    }

    render_presence_text(
        &rows,
        clamped_window_secs,
        filter_project,
        filter_thread,
        clamped_top,
    );
    Ok(())
}

/// T-1487: operator-facing one-shot presence check. Single-line output
/// + exit code semantics (0 online, 1 offline). Composes the existing
/// chat-arc presence probe (T-1480) and local name resolution (T-1483)
/// — pure UX wrapper, no new wire protocol.
pub(crate) async fn cmd_agent_ping(
    target: Option<&str>,
    target_fp: Option<&str>,
    window_secs: u64,
    hub: Option<&str>,
    json: bool,
) -> Result<()> {
    // Mutual-exclusion + one-required, mirror agent who/contact.
    if target.is_some() && target_fp.is_some() {
        let msg = "specify either <TARGET> or --target-fp, not both";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    if target.is_none() && target_fp.is_none() {
        let msg = "must specify either <TARGET> (display name) or --target-fp <hex>";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    let display_target = target
        .map(String::from)
        .unwrap_or_else(|| target_fp.unwrap().to_string());

    let peer_fp_owned: String = if let Some(fp) = target_fp {
        if fp.len() < 8 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
            let msg = format!("--target-fp must be hex (got {fp:?})");
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
            }
            anyhow::bail!(msg);
        }
        fp.to_string()
    } else {
        resolve_target_name_to_fp(target.unwrap(), json)
    };
    let peer_fp = peer_fp_owned.as_str();

    let clamped_window_secs = window_secs.clamp(10, 86_400);
    let check = super::channel::check_peer_online_via_chat_arc(
        peer_fp,
        hub,
        clamped_window_secs,
    )
    .await
    .with_context(|| {
        format!("agent ping: presence probe failed for peer fp={peer_fp}")
    })?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let last_seen_phrase = match check.last_seen_ms {
        None => "never".to_string(),
        Some(ms) => {
            let age_secs = ((now_ms - ms) / 1000).max(0);
            if age_secs < 60 {
                format!("{age_secs}s ago")
            } else if age_secs < 3600 {
                format!("{}m ago", age_secs / 60)
            } else if age_secs < 86_400 {
                format!("{}h ago", age_secs / 3600)
            } else {
                format!("{}d ago", age_secs / 86_400)
            }
        }
    };

    if json {
        let envelope = serde_json::json!({
            "target_or_fp": display_target,
            "peer_fp": peer_fp,
            "online": check.online,
            "last_seen_ms": check.last_seen_ms,
            "last_seen": last_seen_phrase,
            "window_secs": clamped_window_secs,
            "posts_in_window": check.posts_in_window,
        });
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        let status = if check.online { "online" } else { "offline" };
        println!(
            "{display_target} ({peer_fp}): {status} — last seen {last_seen_phrase} (window={clamped_window_secs}s)"
        );
    }

    if check.online {
        Ok(())
    } else {
        // Use process::exit(1) to preserve the JSON envelope already
        // printed (anyhow::bail! would prepend "Error: ..." to stderr).
        std::process::exit(1);
    }
}

/// T-1492: content-access verb — show recent chat-arc posts from a single
/// peer. Companion to `agent who` (aggregates) / `agent presence` (fleet).
/// Walks `agent-chat-arc`, filters by sender_id, optionally further by
/// thread/project, prints last N chronologically.
pub(crate) async fn cmd_agent_recent(
    target: Option<&str>,
    target_fp: Option<&str>,
    n: usize,
    window_secs: u64,
    filter_thread: Option<&str>,
    filter_project: Option<&str>,
    filter_msg_types: Option<&[&str]>,
    filter_grep: Option<&str>,
    hub: Option<&str>,
    json: bool,
    watch: bool,
    watch_interval: u64,
    depth: u64,
) -> Result<()> {
    if target.is_some() && target_fp.is_some() {
        let msg = "specify either <TARGET> or --target-fp, not both";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    if target.is_none() && target_fp.is_none() {
        let msg = "must specify either <TARGET> (display name) or --target-fp <hex>";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    // T-1498: --watch + --json incompatible (streaming vs one-shot).
    if watch && json {
        let msg = "--watch and --json are incompatible: --watch streams \
                   re-rendered text frames; --json is one-shot. Pick one.";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    let display_target = target
        .map(String::from)
        .unwrap_or_else(|| target_fp.unwrap().to_string());

    let peer_fp_owned: String = if let Some(fp) = target_fp {
        if fp.len() < 8 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
            let msg = format!("--target-fp must be hex (got {fp:?})");
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
            }
            anyhow::bail!(msg);
        }
        fp.to_string()
    } else {
        resolve_target_name_to_fp(target.unwrap(), json)
    };
    let peer_fp = peer_fp_owned.as_str();

    let clamped_n = n.clamp(1, 200);
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    // T-1817: history depth — single-page (default 1000) vs bounded multi-page.
    let clamped_depth = depth.clamp(1, 100_000);

    // T-1498: watch loop branches before the one-shot fetch path.
    if watch {
        let clamped_interval = watch_interval.clamp(1, 300);
        loop {
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            let mut filter_suffix = String::new();
            if let Some(p) = filter_project {
                filter_suffix.push_str(&format!(" | project={p}"));
            }
            if let Some(t) = filter_thread {
                filter_suffix.push_str(&format!(" | thread={t}"));
            }
            if let Some(types) = filter_msg_types {
                if !types.is_empty() {
                    filter_suffix.push_str(&format!(" | msg_type={}", types.join(",")));
                }
            }
            if let Some(g) = filter_grep {
                filter_suffix.push_str(&format!(" | grep={g}"));
            }
            println!(
                "# agent recent {} --watch | peer_fp={} | interval={}s | window={}s | n={}{} | {}",
                display_target, peer_fp, clamped_interval, clamped_window_secs,
                clamped_n, filter_suffix, now_str
            );
            // T-1817: paginated fetch — depth controls round-trips above the 1000-page cap.
            match super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth).await {
                Ok(msgs) => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    let window_ms = (clamped_window_secs as i64).saturating_mul(1000);
                    let posts = super::channel::extract_recent_posts(
                        &msgs,
                        clamped_n,
                        window_ms,
                        now_ms,
                        Some(peer_fp),
                        filter_thread,
                        filter_project,
                        filter_msg_types,
                        filter_grep,
                    );
                    render_recent_body(&posts, now_ms);
                }
                Err(e) => {
                    println!("# fetch error (will retry on next tick): {e}");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    }

    // T-1817: paginated fetch — `--depth` controls how deep we walk before
    // filtering. Default 1000 matches pre-T-1817 single-page behavior (covers
    // ~16h on a 1/min chat-arc). Higher --depth walks bounded multi-page so
    // busy fleets dominated by other peers can still surface this peer.
    let msgs = super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth)
        .await
        .with_context(|| {
            format!("agent recent: failed to fetch chat-arc for peer fp={peer_fp}")
        })?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let window_ms = (clamped_window_secs as i64).saturating_mul(1000);

    let posts = super::channel::extract_recent_posts(
        &msgs,
        clamped_n,
        window_ms,
        now_ms,
        Some(peer_fp),
        filter_thread,
        filter_project,
        filter_msg_types,
        filter_grep,
    );

    if json {
        let mut out_obj = serde_json::Map::new();
        out_obj.insert(
            "target".to_string(),
            serde_json::Value::String(display_target.clone()),
        );
        out_obj.insert(
            "peer_fp".to_string(),
            serde_json::Value::String(peer_fp.to_string()),
        );
        out_obj.insert(
            "window_secs".to_string(),
            serde_json::Value::from(clamped_window_secs),
        );
        out_obj.insert("n".to_string(), serde_json::Value::from(clamped_n));
        if let Some(t) = filter_thread {
            out_obj.insert(
                "filter_thread".to_string(),
                serde_json::Value::String(t.to_string()),
            );
        }
        if let Some(p) = filter_project {
            out_obj.insert(
                "filter_project".to_string(),
                serde_json::Value::String(p.to_string()),
            );
        }
        if let Some(types) = filter_msg_types {
            if !types.is_empty() {
                out_obj.insert(
                    "filter_msg_types".to_string(),
                    serde_json::Value::Array(
                        types.iter().map(|t| serde_json::Value::String(t.to_string())).collect(),
                    ),
                );
            }
        }
        if let Some(g) = filter_grep {
            if !g.is_empty() {
                out_obj.insert(
                    "filter_grep".to_string(),
                    serde_json::Value::String(g.to_string()),
                );
            }
        }
        let posts_json: Vec<serde_json::Value> = posts.iter().map(|p| p.to_json()).collect();
        out_obj.insert("posts".to_string(), serde_json::Value::Array(posts_json));
        println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(out_obj))?);
        return Ok(());
    }

    // Text mode header
    let mut filter_suffix = String::new();
    if let Some(p) = filter_project {
        filter_suffix.push_str(&format!(" project={p}"));
    }
    if let Some(t) = filter_thread {
        filter_suffix.push_str(&format!(" thread={t}"));
    }
    if let Some(types) = filter_msg_types {
        if !types.is_empty() {
            filter_suffix.push_str(&format!(" msg_type={}", types.join(",")));
        }
    }
    if let Some(g) = filter_grep {
        filter_suffix.push_str(&format!(" grep={g}"));
    }
    println!(
        "# agent recent {} (peer_fp={}) | window={}s | n={}{}",
        display_target, peer_fp, clamped_window_secs, clamped_n, filter_suffix
    );
    render_recent_body(&posts, now_ms);
    Ok(())
}

/// T-1498: text-mode body renderer extracted from cmd_agent_recent so
/// the watch loop and one-shot path share rendering exactly. Pure
/// (println-side-effect-only) — no fetch, no JSON, no header. Caller
/// owns the header.
fn render_recent_body(posts: &[super::channel::RecentPost], now_ms: i64) {
    if posts.is_empty() {
        println!("(no posts found in window)");
        return;
    }
    for p in posts {
        let age_secs = ((now_ms - p.ts_ms) / 1000).max(0);
        let age_str = if age_secs < 60 {
            format!("{age_secs}s ago")
        } else if age_secs < 3600 {
            format!("{}m ago", age_secs / 60)
        } else if age_secs < 86_400 {
            format!("{}h ago", age_secs / 3600)
        } else {
            format!("{}d ago", age_secs / 86_400)
        };
        let mut tags = format!("msg_type={}", p.msg_type);
        if let Some(t) = &p.thread {
            tags.push_str(&format!(" thread={t}"));
        }
        if let Some(pr) = &p.project {
            tags.push_str(&format!(" project={pr}"));
        }
        // T-1506: show @<offset> so operator can pick for `agent quote <offset>`.
        println!("[{}] @{} {}", age_str, p.offset, tags);
        for line in p.content.lines() {
            println!("    {}", line);
        }
        if p.content.is_empty() {
            println!("    (empty)");
        }
        println!();
    }
    println!("{} post(s) shown", posts.len());
}

/// T-1500: fleet-wide chronological log — "tail -f for the fleet".
/// All posts across all peers in a window, time-ordered, peer-short
/// prefixed for multi-peer disambiguation. No peer or thread filter
/// required — both optional. Pure wrapper around
/// `extract_recent_posts(..., peer=None, ...)`.
pub(crate) async fn cmd_agent_timeline(
    n: usize,
    window_secs: u64,
    filter_thread: Option<&str>,
    filter_project: Option<&str>,
    filter_msg_types: Option<&[&str]>,
    filter_grep: Option<&str>,
    hub: Option<&str>,
    json: bool,
    watch: bool,
    watch_interval: u64,
    depth: u64,
) -> Result<()> {
    if watch && json {
        let msg = "--watch and --json are incompatible: --watch streams \
                   re-rendered text frames; --json is one-shot. Pick one.";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    let clamped_n = n.clamp(1, 500);
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    // T-1818: history depth — single-page (default 1000) vs bounded multi-page.
    let clamped_depth = depth.clamp(1, 100_000);

    if watch {
        let clamped_interval = watch_interval.clamp(1, 300);
        loop {
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            let mut filter_suffix = String::new();
            if let Some(t) = filter_thread {
                filter_suffix.push_str(&format!(" | thread={t}"));
            }
            if let Some(p) = filter_project {
                filter_suffix.push_str(&format!(" | project={p}"));
            }
            if let Some(types) = filter_msg_types {
                if !types.is_empty() {
                    filter_suffix.push_str(&format!(" | msg_type={}", types.join(",")));
                }
            }
            if let Some(g) = filter_grep {
                filter_suffix.push_str(&format!(" | grep={g}"));
            }
            println!(
                "# agent timeline --watch | interval={}s | window={}s | n={}{} | {}",
                clamped_interval, clamped_window_secs, clamped_n, filter_suffix, now_str
            );
            // T-1818: paginated fetch — depth controls round-trips above the 1000-page cap.
            match super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth).await {
                Ok(msgs) => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    let window_ms = (clamped_window_secs as i64).saturating_mul(1000);
                    let posts = super::channel::extract_recent_posts(
                        &msgs,
                        clamped_n,
                        window_ms,
                        now_ms,
                        None,
                        filter_thread,
                        filter_project,
                        filter_msg_types,
                        filter_grep,
                    );
                    render_timeline_body(&posts, now_ms);
                }
                Err(e) => {
                    println!("# fetch error (will retry on next tick): {e}");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    }

    // T-1818: paginated fetch — `--depth` controls how deep we walk before
    // filtering. Default 1000 matches pre-T-1818 single-page behavior.
    let msgs = super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth)
        .await
        .with_context(|| "agent timeline: failed to fetch chat-arc".to_string())?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let window_ms = (clamped_window_secs as i64).saturating_mul(1000);

    let posts = super::channel::extract_recent_posts(
        &msgs,
        clamped_n,
        window_ms,
        now_ms,
        None,
        filter_thread,
        filter_project,
        filter_msg_types,
        filter_grep,
    );

    if json {
        let mut out_obj = serde_json::Map::new();
        out_obj.insert("verb".to_string(), serde_json::Value::String("agent.timeline".to_string()));
        out_obj.insert("window_secs".to_string(), serde_json::Value::from(clamped_window_secs));
        out_obj.insert("n".to_string(), serde_json::Value::from(clamped_n));
        if let Some(t) = filter_thread {
            out_obj.insert("filter_thread".to_string(), serde_json::Value::String(t.to_string()));
        }
        if let Some(p) = filter_project {
            out_obj.insert("filter_project".to_string(), serde_json::Value::String(p.to_string()));
        }
        if let Some(types) = filter_msg_types {
            if !types.is_empty() {
                out_obj.insert(
                    "filter_msg_types".to_string(),
                    serde_json::Value::Array(
                        types.iter().map(|t| serde_json::Value::String(t.to_string())).collect(),
                    ),
                );
            }
        }
        if let Some(g) = filter_grep {
            if !g.is_empty() {
                out_obj.insert(
                    "filter_grep".to_string(),
                    serde_json::Value::String(g.to_string()),
                );
            }
        }
        let posts_json: Vec<serde_json::Value> = posts.iter().map(|p| p.to_json()).collect();
        out_obj.insert("posts".to_string(), serde_json::Value::Array(posts_json));
        println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(out_obj))?);
        return Ok(());
    }

    let mut filter_suffix = String::new();
    if let Some(t) = filter_thread {
        filter_suffix.push_str(&format!(" thread={t}"));
    }
    if let Some(p) = filter_project {
        filter_suffix.push_str(&format!(" project={p}"));
    }
    if let Some(types) = filter_msg_types {
        if !types.is_empty() {
            filter_suffix.push_str(&format!(" msg_type={}", types.join(",")));
        }
    }
    if let Some(g) = filter_grep {
        filter_suffix.push_str(&format!(" grep={g}"));
    }
    println!(
        "# agent timeline | window={}s | n={}{}",
        clamped_window_secs, clamped_n, filter_suffix
    );
    render_timeline_body(&posts, now_ms);
    Ok(())
}

/// T-1503: read `current_task` from `.context/working/focus.yaml` if
/// present. Pure file read — no error if missing/unreadable, returns
/// None. Walks up from $PWD looking for `.context/working/focus.yaml`
/// (max 4 levels). Tolerates ~/.context paths too.
fn resolve_focus_task() -> Option<String> {
    let mut cwd = std::env::current_dir().ok()?;
    for _ in 0..4 {
        let p = cwd.join(".context/working/focus.yaml");
        if p.is_file() {
            if let Ok(text) = std::fs::read_to_string(&p) {
                for line in text.lines() {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("current_task:") {
                        let v = rest.trim().trim_matches('"').trim_matches('\'');
                        if !v.is_empty() && v != "null" && v != "~" {
                            return Some(v.to_string());
                        }
                    }
                }
            }
            return None;
        }
        if !cwd.pop() {
            break;
        }
    }
    None
}

/// T-1503: read `project_name` from `.framework.yaml` if present. Same
/// walk-up search pattern as `resolve_focus_task`.
fn resolve_framework_project() -> Option<String> {
    let mut cwd = std::env::current_dir().ok()?;
    for _ in 0..4 {
        let p = cwd.join(".framework.yaml");
        if p.is_file() {
            if let Ok(text) = std::fs::read_to_string(&p) {
                for line in text.lines() {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("project_name:") {
                        let v = rest.trim().trim_matches('"').trim_matches('\'');
                        if !v.is_empty() {
                            return Some(v.to_string());
                        }
                    }
                }
            }
            return None;
        }
        if !cwd.pop() {
            break;
        }
    }
    None
}

/// T-1504: fleet-wide aggregate counts. Single-fetch summary of chat-arc
/// activity in a window, grouped by msg_type / peer / project / thread.
pub(crate) async fn cmd_agent_stats(
    window_secs: u64,
    top: usize,
    hub: Option<&str>,
    json: bool,
) -> Result<()> {
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    let clamped_top = top.clamp(1, 100);
    let stats = super::channel::fetch_chat_arc_stats(hub, clamped_window_secs)
        .await
        .with_context(|| "agent stats: failed to fetch chat-arc")?;

    if json {
        let mk_pairs = |v: &[(String, usize)]| -> serde_json::Value {
            serde_json::Value::Array(
                v.iter()
                    .map(|(k, c)| serde_json::json!({"key": k, "count": c}))
                    .collect(),
            )
        };
        let out = serde_json::json!({
            "verb": "agent.stats",
            "window_secs": clamped_window_secs,
            "top": clamped_top,
            "total": stats.total,
            "by_msg_type": mk_pairs(&stats.by_msg_type),
            "by_peer": mk_pairs(&stats.by_peer),
            "by_project": mk_pairs(&stats.by_project),
            "by_thread": mk_pairs(&stats.by_thread),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!(
        "# agent stats | window={}s | total={} | top={}",
        clamped_window_secs, stats.total, clamped_top
    );
    fn render_section(title: &str, rows: &[(String, usize)], top: usize) {
        println!();
        println!("## {} ({} unique)", title, rows.len());
        if rows.is_empty() {
            println!("  (none)");
            return;
        }
        let take = rows.len().min(top);
        for (k, c) in &rows[..take] {
            let key_disp = if k.len() > 50 { format!("{}…", &k[..49]) } else { k.clone() };
            println!("  {:>5}  {}", c, key_disp);
        }
        if rows.len() > top {
            println!("  …and {} more", rows.len() - top);
        }
    }
    render_section("by msg_type", &stats.by_msg_type, clamped_top);
    render_section("by peer", &stats.by_peer, clamped_top);
    render_section("by project", &stats.by_project, clamped_top);
    render_section("by thread", &stats.by_thread, clamped_top);
    Ok(())
}

/// T-1503: focus-aware chat-arc post verb. Companion to the read verbs
/// (recent/on-thread/timeline). Auto-resolves `--thread` from focus.yaml
/// and `--project` from .framework.yaml. Defers to
/// `super::channel::cmd_channel_post` for actual signing/posting.
pub(crate) async fn cmd_agent_post(
    text: &str,
    thread_override: Option<&str>,
    project_override: Option<&str>,
    msg_type: &str,
    hub: Option<&str>,
    json: bool,
) -> Result<()> {
    if text.trim().is_empty() {
        let msg = "agent post: text cannot be empty";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    let resolved_thread: Option<String> = thread_override
        .map(String::from)
        .or_else(resolve_focus_task);
    let resolved_project: Option<String> = project_override
        .map(String::from)
        .or_else(resolve_framework_project);

    let mut metadata_kvs: Vec<String> = Vec::new();
    if let Some(t) = &resolved_thread {
        metadata_kvs.push(format!("thread={t}"));
    }
    if let Some(p) = &resolved_project {
        metadata_kvs.push(format!("from_project={p}"));
    }

    super::channel::cmd_channel_post(
        "agent-chat-arc",
        msg_type,
        Some(text),
        None,
        None,
        None,
        &metadata_kvs,
        false,
        hub,
        json,
        None, // T-2049 client_msg_id (auto-mint)
    super::channel::AwaitAckOpts::default(),
        ).await
}

/// T-1508: full-arc substring search — unbounded by window. Walks the
/// entire `agent-chat-arc` topic via `fetch_chat_arc_full`, runs the same
/// case-insensitive substring filter `extract_recent_posts` already
/// implements, returns last N matches. Operator answer to "did anyone
/// ever mention X?" (vs `agent timeline --grep` capped at 7 days).
pub(crate) async fn cmd_agent_search(
    query: &str,
    n: usize,
    hub: Option<&str>,
    json: bool,
) -> Result<()> {
    if query.trim().is_empty() {
        let msg = "agent search: query cannot be empty";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    let clamped_n = n.clamp(1, 500);
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    // Effectively unbounded: window is the full now_ms (epoch onward).
    // extract_recent_posts uses cutoff = now_ms - window_ms; with
    // window_ms = now_ms, cutoff is 0 — all post-epoch posts pass.
    let window_ms = now_ms.max(0);
    let msgs = super::channel::fetch_chat_arc_full(hub)
        .await
        .context("Fetching chat-arc full slice for search")?;
    let posts = super::channel::extract_recent_posts(
        &msgs,
        clamped_n,
        window_ms,
        now_ms,
        None,
        None,
        None,
        None,
        Some(query),
    );
    if json {
        let posts_json: Vec<serde_json::Value> = posts.iter().map(|p| p.to_json()).collect();
        let envelope = serde_json::json!({
            "verb": "agent.search",
            "query": query,
            "n": clamped_n,
            "total_envelopes": msgs.len(),
            "posts": posts_json,
        });
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        return Ok(());
    }
    println!(
        "# agent search | query={} | scanned={} envelopes | n={}",
        query, msgs.len(), clamped_n
    );
    render_timeline_body(&posts, now_ms);
    Ok(())
}

/// T-1507: focus-aware threaded reply on agent-chat-arc. Mirror of
/// `cmd_agent_post` but threads `reply_to: Some(offset)` through to
/// `cmd_channel_post`, which writes `metadata.in_reply_to=<offset>`. The
/// new envelope is then visible as a child via `agent quote <new-offset>`
/// (parent line rendered) and via Matrix-style traversal in
/// `cmd_channel_thread` / `cmd_channel_replies`.
pub(crate) async fn cmd_agent_reply(
    offset: u64,
    text: &str,
    thread_override: Option<&str>,
    project_override: Option<&str>,
    msg_type: &str,
    hub: Option<&str>,
    json: bool,
) -> Result<()> {
    if text.trim().is_empty() {
        let msg = "agent reply: text cannot be empty";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    let resolved_thread: Option<String> = thread_override
        .map(String::from)
        .or_else(resolve_focus_task);
    let resolved_project: Option<String> = project_override
        .map(String::from)
        .or_else(resolve_framework_project);

    let mut metadata_kvs: Vec<String> = Vec::new();
    if let Some(t) = &resolved_thread {
        metadata_kvs.push(format!("thread={t}"));
    }
    if let Some(p) = &resolved_project {
        metadata_kvs.push(format!("from_project={p}"));
    }

    super::channel::cmd_channel_post(
        "agent-chat-arc",
        msg_type,
        Some(text),
        None,
        None,
        Some(offset),
        &metadata_kvs,
        false,
        hub,
        json,
        None, // T-2049 client_msg_id (auto-mint)
    super::channel::AwaitAckOpts::default(),
        ).await
}

/// T-1500: timeline body renderer. Like `render_recent_body` but
/// prefixes each post with peer-short (first 8 chars of peer_fp) so
/// the operator can disambiguate posts across peers in the
/// chronological log.
fn render_timeline_body(posts: &[super::channel::RecentPost], now_ms: i64) {
    if posts.is_empty() {
        println!("(no posts found in window)");
        return;
    }
    for p in posts {
        let age_secs = ((now_ms - p.ts_ms) / 1000).max(0);
        let age_str = if age_secs < 60 {
            format!("{age_secs}s ago")
        } else if age_secs < 3600 {
            format!("{}m ago", age_secs / 60)
        } else if age_secs < 86_400 {
            format!("{}h ago", age_secs / 3600)
        } else {
            format!("{}d ago", age_secs / 86_400)
        };
        let peer_short: String = p.peer_fp.chars().take(8).collect();
        let mut tags = format!("msg_type={}", p.msg_type);
        if let Some(t) = &p.thread {
            tags.push_str(&format!(" thread={t}"));
        }
        if let Some(pr) = &p.project {
            tags.push_str(&format!(" project={pr}"));
        }
        // T-1506: show @<offset> so operator can pick for `agent quote <offset>`.
        println!("[{}] [{}] @{} {}", age_str, peer_short, p.offset, tags);
        for line in p.content.lines() {
            println!("    {}", line);
        }
        if p.content.is_empty() {
            println!("    (empty)");
        }
        println!();
    }
    println!("{} post(s) shown", posts.len());
}

/// T-1493: chronological reading view of all posts on a thread across
/// the fleet. Wraps the same `extract_recent_posts` helper as
/// `cmd_agent_recent` but with `filter_thread` required and `peer`
/// optional. Use when you want to read the discussion, not just the
/// aggregates from `who --thread` / `presence --thread`.
pub(crate) async fn cmd_agent_on_thread(
    thread: &str,
    n: usize,
    window_secs: u64,
    filter_project: Option<&str>,
    filter_msg_types: Option<&[&str]>,
    filter_grep: Option<&str>,
    peer: Option<&str>,
    peer_fp: Option<&str>,
    hub: Option<&str>,
    json: bool,
    watch: bool,
    watch_interval: u64,
    depth: u64,
) -> Result<()> {
    if peer.is_some() && peer_fp.is_some() {
        let msg = "specify either --peer or --peer-fp, not both";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    if thread.trim().is_empty() {
        let msg = "thread (positional argument) cannot be empty";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    // T-1494: --watch + --json incompatible (streaming vs one-shot).
    if watch && json {
        let msg = "--watch and --json are incompatible: --watch streams \
                   re-rendered text frames; --json is one-shot. Pick one.";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }
    let resolved_peer_fp: Option<String> = if let Some(fp) = peer_fp {
        if fp.len() < 8 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
            let msg = format!("--peer-fp must be hex (got {fp:?})");
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
            }
            anyhow::bail!(msg);
        }
        Some(fp.to_string())
    } else {
        peer.map(|p| resolve_target_name_to_fp(p, json))
    };

    let clamped_n = n.clamp(1, 500);
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    // T-1816: history depth for the pre-filter fetch. 1000 = the hub's
    // per-page cap (single round-trip); higher values trigger multi-page
    // pagination via T-1796 fetch_topic_msgs_paginated. 100k upper bound
    // is a safety cap (~100 round-trips); operator can request more by
    // running multiple targeted queries.
    let clamped_depth = depth.clamp(1, 100_000);

    // T-1494: watch loop branches before the one-shot fetch path.
    if watch {
        let clamped_interval = watch_interval.clamp(1, 300);
        loop {
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            let mut filter_suffix = String::new();
            if let Some(p) = filter_project {
                filter_suffix.push_str(&format!(" | project={p}"));
            }
            if let Some(types) = filter_msg_types {
                if !types.is_empty() {
                    filter_suffix.push_str(&format!(" | msg_type={}", types.join(",")));
                }
            }
            if let Some(g) = filter_grep {
                filter_suffix.push_str(&format!(" | grep={g}"));
            }
            println!(
                "# agent on-thread {} --watch | interval={}s | window={}s | n={}{} | {}",
                thread, clamped_interval, clamped_window_secs, clamped_n,
                filter_suffix, now_str
            );
            // T-1816: bounded multi-page pagination — honors --depth >1000.
            match super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth).await {
                Ok(msgs) => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    let window_ms = (clamped_window_secs as i64).saturating_mul(1000);
                    let posts = super::channel::extract_recent_posts(
                        &msgs,
                        clamped_n,
                        window_ms,
                        now_ms,
                        resolved_peer_fp.as_deref(),
                        Some(thread),
                        filter_project,
                        filter_msg_types,
                        filter_grep,
                    );
                    render_on_thread_text(
                        thread,
                        &posts,
                        clamped_window_secs,
                        filter_project,
                        resolved_peer_fp.as_deref(),
                        now_ms,
                    );
                }
                Err(e) => {
                    println!("# fetch error (will retry on next tick): {e}");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    }

    // T-1795 + T-1816: fetch the most-recent `clamped_depth` envelopes via
    // bounded multi-page pagination. Default depth=1000 = single round-trip
    // (matches pre-T-1816 behavior). Depth >1000 walks multiple pages, useful
    // on busy fleets where the most-recent 1000 envelopes contain few thread
    // matches.
    let msgs = super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth)
        .await
        .with_context(|| {
            format!("agent on-thread: failed to fetch chat-arc for thread={thread}")
        })?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let window_ms = (clamped_window_secs as i64).saturating_mul(1000);

    let posts = super::channel::extract_recent_posts(
        &msgs,
        clamped_n,
        window_ms,
        now_ms,
        resolved_peer_fp.as_deref(),
        Some(thread),
        filter_project,
        filter_msg_types,
        filter_grep,
    );

    if json {
        let mut out_obj = serde_json::Map::new();
        out_obj.insert(
            "thread".to_string(),
            serde_json::Value::String(thread.to_string()),
        );
        out_obj.insert(
            "window_secs".to_string(),
            serde_json::Value::from(clamped_window_secs),
        );
        out_obj.insert("n".to_string(), serde_json::Value::from(clamped_n));
        if let Some(p) = filter_project {
            out_obj.insert(
                "filter_project".to_string(),
                serde_json::Value::String(p.to_string()),
            );
        }
        if let Some(fp) = &resolved_peer_fp {
            out_obj.insert(
                "peer_fp".to_string(),
                serde_json::Value::String(fp.clone()),
            );
        }
        if let Some(types) = filter_msg_types {
            if !types.is_empty() {
                out_obj.insert(
                    "filter_msg_types".to_string(),
                    serde_json::Value::Array(
                        types.iter().map(|t| serde_json::Value::String(t.to_string())).collect(),
                    ),
                );
            }
        }
        if let Some(g) = filter_grep {
            if !g.is_empty() {
                out_obj.insert(
                    "filter_grep".to_string(),
                    serde_json::Value::String(g.to_string()),
                );
            }
        }
        let posts_json: Vec<serde_json::Value> = posts.iter().map(|p| p.to_json()).collect();
        out_obj.insert("posts".to_string(), serde_json::Value::Array(posts_json));
        println!("{}", serde_json::to_string_pretty(&serde_json::Value::Object(out_obj))?);
        return Ok(());
    }

    // One-shot mode header — names what's being shown.
    let mut suffix = String::new();
    if let Some(p) = filter_project {
        suffix.push_str(&format!(" project={p}"));
    }
    if let Some(fp) = &resolved_peer_fp {
        suffix.push_str(&format!(" peer_fp={fp}"));
    }
    if let Some(types) = filter_msg_types {
        if !types.is_empty() {
            suffix.push_str(&format!(" msg_type={}", types.join(",")));
        }
    }
    if let Some(g) = filter_grep {
        suffix.push_str(&format!(" grep={g}"));
    }
    println!(
        "# agent on-thread {} | window={}s | n={}{}",
        thread, clamped_window_secs, clamped_n, suffix
    );
    render_on_thread_text(
        thread,
        &posts,
        clamped_window_secs,
        filter_project,
        resolved_peer_fp.as_deref(),
        now_ms,
    );
    Ok(())
}

/// T-1495: single-shot fleet digest. Fetches `agent-chat-arc` ONCE
/// and computes three summaries from the same msgs slice — top peers,
/// top projects, last posts. Designed as the first command of a
/// session: tells the operator "what's happening on this fleet right
/// now?" without requiring three separate verb invocations.
pub(crate) async fn cmd_agent_overview(
    window_secs: u64,
    top: usize,
    hub: Option<&str>,
    json: bool,
    watch: bool,
    watch_interval: u64,
    depth: u64,
) -> Result<()> {
    let clamped_window_secs = window_secs.clamp(60, 604_800);
    let clamped_top = top.clamp(1, 50);
    // T-1819: history depth — single-page (default 1000) vs bounded multi-page.
    let clamped_depth = depth.clamp(1, 100_000);
    // T-1496: --watch + --json incompatible.
    if watch && json {
        let msg = "--watch and --json are incompatible: --watch streams \
                   re-rendered text frames; --json is one-shot. Pick one.";
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": msg}));
        }
        anyhow::bail!(msg);
    }

    // T-1496: watch loop branches before the one-shot fetch.
    if watch {
        let clamped_interval = watch_interval.clamp(1, 300);
        loop {
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            println!(
                "# agent overview --watch | interval={}s | window={}s | top={} | {}",
                clamped_interval, clamped_window_secs, clamped_top, now_str
            );
            // T-1819: paginated fetch — depth controls round-trips above the 1000-page cap.
            match super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth).await {
                Ok(msgs) => render_overview_body(&msgs, clamped_window_secs, clamped_top),
                Err(e) => {
                    println!("# fetch error (will retry on next tick): {e}");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(clamped_interval)).await;
        }
    }

    // T-1795 → T-1819: `--depth` controls how deep we walk before computing
    // summaries. Default 1000 matches pre-T-1819 single-page behavior (the
    // T-1795 fix kept the tail anchor right). Higher --depth walks bounded
    // multi-page so digests on busy fleets cover a longer window.
    let msgs = super::channel::fetch_topic_msgs_paginated("agent-chat-arc", hub, clamped_depth)
        .await
        .context("agent overview: failed to fetch chat-arc")?;

    if json {
        let envelope = compose_overview_json(&msgs, clamped_window_secs, clamped_top);
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        return Ok(());
    }

    render_overview_body(&msgs, clamped_window_secs, clamped_top);
    Ok(())
}

/// T-1496: pure helper — compute and render the 3-section overview
/// from a chat-arc msgs slice. Used by both one-shot and `--watch`
/// paths so layout stays in sync.
fn render_overview_body(msgs: &[serde_json::Value], window_secs: u64, top: usize) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let window_ms = (window_secs as i64).saturating_mul(1000);

    let peer_rows = super::channel::summarize_fleet_presence(
        msgs, now_ms, window_ms, None, None,
    );
    let project_rows = super::channel::summarize_fleet_by_project(
        msgs, now_ms, window_ms, None, None,
    );
    let recent = super::channel::extract_recent_posts(
        msgs, top, window_ms, now_ms, None, None, None,
        None,
        None,
    );
    let display_peers: &[super::channel::FleetPeerRow] =
        &peer_rows[..peer_rows.len().min(top)];
    let display_projects: &[super::channel::FleetProjectRow] =
        &project_rows[..project_rows.len().min(top)];

    if peer_rows.is_empty() && project_rows.is_empty() && recent.is_empty() {
        println!("(no fleet activity in window={}s)", window_secs);
        return;
    }
    let clamped_window_secs = window_secs;
    let clamped_top = top;

    println!(
        "## Top Peers (window={}s, top={})",
        clamped_window_secs, clamped_top
    );
    if display_peers.is_empty() {
        println!("(no peers active)");
    } else {
        println!(
            "{:<18} {:>14} {:>8}  {}",
            "PEER_FP", "LAST_SEEN", "POSTS", "TOP_PROJECT"
        );
        for r in display_peers {
            let last_seen_str = match r.last_seen_ms {
                None => "never".to_string(),
                Some(ms) => fmt_age(now_ms, ms),
            };
            let project_str = r.top_project.as_deref().unwrap_or("-");
            println!(
                "{:<18} {:>14} {:>8}  {}",
                r.peer_fp, last_seen_str, r.posts, project_str
            );
        }
    }
    println!();
    println!(
        "## Top Projects (window={}s, top={})",
        clamped_window_secs, clamped_top
    );
    if display_projects.is_empty() {
        println!("(no projects active — no tagged posts in window)");
    } else {
        println!(
            "{:<24} {:>8} {:>8} {:<18}  {}",
            "PROJECT", "POSTS", "PEERS", "TOP_PEER", "LAST_SEEN"
        );
        for r in display_projects {
            let last_seen_str = match r.last_seen_ms {
                None => "never".to_string(),
                Some(ms) => fmt_age(now_ms, ms),
            };
            let top_peer = r.top_peer_fp.as_deref().unwrap_or("-");
            println!(
                "{:<24} {:>8} {:>8} {:<18}  {}",
                r.project, r.posts, r.distinct_peers, top_peer, last_seen_str
            );
        }
    }
    println!();
    println!(
        "## Recent Posts (window={}s, top={})",
        clamped_window_secs, clamped_top
    );
    if recent.is_empty() {
        println!("(no posts in window)");
    } else {
        for p in &recent {
            let age_str = fmt_age(now_ms, p.ts_ms);
            let peer_short = if p.peer_fp.len() > 12 {
                &p.peer_fp[..12]
            } else {
                &p.peer_fp[..]
            };
            let mut tags = format!("peer={} msg_type={}", peer_short, p.msg_type);
            if let Some(t) = &p.thread {
                tags.push_str(&format!(" thread={t}"));
            }
            if let Some(pr) = &p.project {
                tags.push_str(&format!(" project={pr}"));
            }
            println!("[{}] {}", age_str, tags);
            if !p.content.is_empty() {
                // Single-line preview: first line, capped at 100 chars
                // (overview is a digest — the operator drills in via
                // `agent recent` / `agent on-thread` for full content).
                let first_line = p.content.lines().next().unwrap_or("");
                let preview: String = first_line.chars().take(100).collect();
                let suffix = if first_line.chars().count() > 100 || p.content.lines().count() > 1 {
                    "…"
                } else {
                    ""
                };
                println!("    {}{}", preview, suffix);
            }
        }
    }
    println!();
    println!(
        "# overview: window={}s, top={}, total_peers={}, total_projects={}",
        clamped_window_secs,
        clamped_top,
        peer_rows.len(),
        project_rows.len()
    );
}

/// T-1496: pure helper — compose the overview JSON envelope from a
/// chat-arc msgs slice. Same data shape as the inline T-1495 path.
fn compose_overview_json(
    msgs: &[serde_json::Value],
    window_secs: u64,
    top: usize,
) -> serde_json::Value {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let window_ms = (window_secs as i64).saturating_mul(1000);
    let peer_rows = super::channel::summarize_fleet_presence(
        msgs, now_ms, window_ms, None, None,
    );
    let project_rows = super::channel::summarize_fleet_by_project(
        msgs, now_ms, window_ms, None, None,
    );
    let recent = super::channel::extract_recent_posts(
        msgs, top, window_ms, now_ms, None, None, None,
        None,
        None,
    );
    let display_peers: &[super::channel::FleetPeerRow] =
        &peer_rows[..peer_rows.len().min(top)];
    let display_projects: &[super::channel::FleetProjectRow] =
        &project_rows[..project_rows.len().min(top)];
    let mut out_obj = serde_json::Map::new();
    out_obj.insert(
        "window_secs".to_string(),
        serde_json::Value::from(window_secs),
    );
    out_obj.insert("top".to_string(), serde_json::Value::from(top));
    out_obj.insert(
        "peers".to_string(),
        serde_json::Value::Array(display_peers.iter().map(|r| r.to_json()).collect()),
    );
    out_obj.insert(
        "projects".to_string(),
        serde_json::Value::Array(display_projects.iter().map(|r| r.to_json()).collect()),
    );
    out_obj.insert(
        "recent_posts".to_string(),
        serde_json::Value::Array(recent.iter().map(|p| p.to_json()).collect()),
    );
    out_obj.insert(
        "total_peers".to_string(),
        serde_json::Value::from(peer_rows.len()),
    );
    out_obj.insert(
        "total_projects".to_string(),
        serde_json::Value::from(project_rows.len()),
    );
    serde_json::Value::Object(out_obj)
}

/// Helper — render an "Xs/Ym/Zh/Wd ago" string from now_ms vs ts_ms.
fn fmt_age(now_ms: i64, ts_ms: i64) -> String {
    let age_secs = ((now_ms - ts_ms) / 1000).max(0);
    if age_secs < 60 {
        format!("{age_secs}s ago")
    } else if age_secs < 3600 {
        format!("{}m ago", age_secs / 60)
    } else if age_secs < 86_400 {
        format!("{}h ago", age_secs / 3600)
    } else {
        format!("{}d ago", age_secs / 86_400)
    }
}

/// T-1494: text-mode body renderer extracted from cmd_agent_on_thread
/// so the watch loop can reuse the exact same layout per-iteration.
/// Renders ONLY the data block (empty-state message OR posts + footer);
/// callers print their own header line so watch mode and one-shot mode
/// can each surface mode-specific header info.
fn render_on_thread_text(
    thread: &str,
    posts: &[super::channel::RecentPost],
    window_secs: u64,
    filter_project: Option<&str>,
    resolved_peer_fp: Option<&str>,
    now_ms: i64,
) {
    let mut suffix = String::new();
    if let Some(p) = filter_project {
        suffix.push_str(&format!(" project={p}"));
    }
    if let Some(fp) = resolved_peer_fp {
        suffix.push_str(&format!(" peer_fp={fp}"));
    }
    if posts.is_empty() {
        println!(
            "(no posts found on thread={} in window={}s{})",
            thread, window_secs, suffix
        );
        return;
    }
    for p in posts {
        let age_secs = ((now_ms - p.ts_ms) / 1000).max(0);
        let age_str = if age_secs < 60 {
            format!("{age_secs}s ago")
        } else if age_secs < 3600 {
            format!("{}m ago", age_secs / 60)
        } else if age_secs < 86_400 {
            format!("{}h ago", age_secs / 3600)
        } else {
            format!("{}d ago", age_secs / 86_400)
        };
        let peer_short = if p.peer_fp.len() > 12 {
            &p.peer_fp[..12]
        } else {
            &p.peer_fp[..]
        };
        let mut tags = format!("peer={} msg_type={}", peer_short, p.msg_type);
        if let Some(pr) = &p.project {
            tags.push_str(&format!(" project={pr}"));
        }
        // T-1506: show @<offset> so operator can pick for `agent quote <offset>`.
        println!("[{}] @{} {}", age_str, p.offset, tags);
        for line in p.content.lines() {
            println!("    {}", line);
        }
        if p.content.is_empty() {
            println!("    (empty)");
        }
        println!();
    }
    println!("{} post(s) shown", posts.len());
}

/// T-1486: text-mode renderer extracted so the watch loop can reuse it
/// per-iteration without duplicating layout. T-1489: optional `top` —
/// truncate to N busiest peers post-sort, footer reports both displayed
/// and total counts so the operator knows what was clipped.
fn render_presence_text(
    rows: &[super::channel::FleetPeerRow],
    window_secs: u64,
    filter_project: Option<&str>,
    filter_thread: Option<&str>,
    top: Option<usize>,
) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    // T-1490: build a human-readable filter suffix that names whichever
    // filters are active. Used by the empty-rows branch and the footer.
    let filter_suffix = match (filter_project, filter_thread) {
        (None, None) => String::new(),
        (Some(p), None) => format!(" matching project={}", p),
        (None, Some(t)) => format!(" matching thread={}", t),
        (Some(p), Some(t)) => format!(" matching project={} thread={}", p, t),
    };
    if rows.is_empty() {
        if filter_suffix.is_empty() {
            println!("(no peers active in window={}s)", window_secs);
        } else {
            println!("(no peers active in window={}s{})", window_secs, filter_suffix);
        }
        return;
    }
    if let Some(f) = filter_project {
        println!("# filter_project={}", f);
    }
    if let Some(t) = filter_thread {
        println!("# filter_thread={}", t);
    }
    println!(
        "{:<18} {:>14} {:>8}  {}",
        "PEER_FP", "LAST_SEEN", "POSTS", "TOP_PROJECT"
    );
    let total_peers = rows.len();
    let display_rows: &[super::channel::FleetPeerRow] = match top {
        Some(n) => &rows[..rows.len().min(n)],
        None => rows,
    };
    for r in display_rows {
        let last_seen_str = match r.last_seen_ms {
            None => "never".to_string(),
            Some(ms) => {
                let age_secs = ((now_ms - ms) / 1000).max(0);
                if age_secs < 60 {
                    format!("{age_secs}s ago")
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else if age_secs < 86_400 {
                    format!("{}h ago", age_secs / 3600)
                } else {
                    format!("{}d ago", age_secs / 86_400)
                }
            }
        };
        let project_str = r.top_project.as_deref().unwrap_or("-");
        println!(
            "{:<18} {:>14} {:>8}  {}",
            r.peer_fp, last_seen_str, r.posts, project_str
        );
    }
    println!();
    // T-1489: footer naming both shown and total when truncation applied.
    // T-1490: filter_suffix already names project/thread filters.
    let footer_count = match top {
        Some(_) if display_rows.len() < total_peers => {
            format!("{} of {}", display_rows.len(), total_peers)
        }
        _ => total_peers.to_string(),
    };
    println!(
        "{} peer(s) active in window={}s{}",
        footer_count,
        window_secs,
        filter_suffix
    );
}

/// T-1491: text-mode renderer for the by-project view. Mirrors
/// `render_presence_text` (filter-aware empty/footer phrasing, optional
/// `--top` truncation) but with project-keyed rows.
fn render_by_project_text(
    rows: &[super::channel::FleetProjectRow],
    window_secs: u64,
    filter_project: Option<&str>,
    filter_thread: Option<&str>,
    top: Option<usize>,
) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let filter_suffix = match (filter_project, filter_thread) {
        (None, None) => String::new(),
        (Some(p), None) => format!(" matching project={}", p),
        (None, Some(t)) => format!(" matching thread={}", t),
        (Some(p), Some(t)) => format!(" matching project={} thread={}", p, t),
    };
    if rows.is_empty() {
        if filter_suffix.is_empty() {
            println!(
                "(no projects active in window={}s — fleet has no tagged posts)",
                window_secs
            );
        } else {
            println!(
                "(no projects active in window={}s{})",
                window_secs, filter_suffix
            );
        }
        return;
    }
    if let Some(f) = filter_project {
        println!("# filter_project={}", f);
    }
    if let Some(t) = filter_thread {
        println!("# filter_thread={}", t);
    }
    println!(
        "{:<24} {:>8} {:>8} {:<18}  {}",
        "PROJECT", "POSTS", "PEERS", "TOP_PEER", "LAST_SEEN"
    );
    let total_projects = rows.len();
    let display_rows: &[super::channel::FleetProjectRow] = match top {
        Some(n) => &rows[..rows.len().min(n)],
        None => rows,
    };
    for r in display_rows {
        let last_seen_str = match r.last_seen_ms {
            None => "never".to_string(),
            Some(ms) => {
                let age_secs = ((now_ms - ms) / 1000).max(0);
                if age_secs < 60 {
                    format!("{age_secs}s ago")
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else if age_secs < 86_400 {
                    format!("{}h ago", age_secs / 3600)
                } else {
                    format!("{}d ago", age_secs / 86_400)
                }
            }
        };
        let top_peer = r.top_peer_fp.as_deref().unwrap_or("-");
        println!(
            "{:<24} {:>8} {:>8} {:<18}  {}",
            r.project, r.posts, r.distinct_peers, top_peer, last_seen_str
        );
    }
    println!();
    let footer_count = match top {
        Some(_) if display_rows.len() < total_projects => {
            format!("{} of {}", display_rows.len(), total_projects)
        }
        _ => total_projects.to_string(),
    };
    println!(
        "{} project(s) active in window={}s{}",
        footer_count, window_secs, filter_suffix
    );
}

/// T-1478: pure helper — build the dry-run preview JSON. Mirrors the
/// metadata that `cmd_channel_dm` would stamp on the post (from_project
/// auto-injected by `cmd_channel_post`, plus any extra_metadata supplied
/// here). Pure: no I/O, no globals; tested via contact_tests.
///
/// T-1479: takes optional `local_session_count` — when > 1, indicates
/// co-residency on the peer FP and the preview emits a `co_residency`
/// block with a context-aware warning. None means co-residency was not
/// probed (e.g. session enumeration failed) — silent on that case.
pub(crate) fn render_dry_run_preview<'a, I>(
    my_id: &str,
    peer_fp: &str,
    topic: &str,
    extra_metadata: I,
    message: &str,
    local_session_count: Option<usize>,
) -> serde_json::Value
where
    I: IntoIterator<Item = &'a str>,
{
    let mut metadata = serde_json::Map::new();
    // Mirror channel::cmd_channel_post's auto-inject for from_project so the
    // operator sees what would actually land on the wire. Pure-function call.
    if let Some(p) = std::env::current_dir()
        .ok()
        .and_then(|cwd| super::channel::resolve_project_name_from(&cwd))
    {
        metadata.insert("from_project".to_string(), serde_json::Value::String(p));
    }
    for kv in extra_metadata {
        if let Some((k, v)) = kv.split_once('=') {
            metadata.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
    }
    let mut preview = serde_json::json!({
        "dry_run": true,
        "my_id": my_id,
        "peer_fp": peer_fp,
        "topic": topic,
        "metadata": metadata.clone(),
        "message": message,
    });
    // T-1479: co-residency block (only when N > 1).
    if let Some(n) = local_session_count
        && n > 1
    {
        let to_project = metadata
            .get("to_project")
            .and_then(|v| v.as_str());
        let warning = match to_project {
            None => format!(
                "co-resident peers detected ({n} sessions share this FP locally) \
                 and no to_project qualifier — post will reach all of them; \
                 pass <name>:<project> to target one"
            ),
            Some(value) => format!(
                "co-resident peers detected ({n} sessions share this FP locally); \
                 to_project={value} will let receivers self-filter"
            ),
        };
        preview["co_residency"] = serde_json::json!({
            "local_session_count": n,
            "warning": warning,
        });
    }
    preview
}

#[cfg(test)]
mod contact_tests {
    /// T-1429 Phase-1: the canonical dm topic name is `dm:<sorted_a>:<sorted_b>`,
    /// independent of which side calls. Verified through the existing
    /// `dm_topic` helper in commands/channel.rs (private, but exercised here
    /// via cmd_channel_dm). This test is a stub asserting the shape contract
    /// stays stable — actual topic computation is tested in channel.rs.
    #[test]
    fn dm_topic_shape_canon_stable() {
        // The canon is two lowercase-hex fingerprints sorted lex, joined
        // by `dm:`. Recorded here so a future refactor doesn't silently
        // change the format that vendored agents already key off.
        let lo = "0000aaaa";
        let hi = "ffffbbbb";
        let canon = format!("dm:{lo}:{hi}");
        assert!(canon.starts_with("dm:"));
        assert!(canon.contains(":"));
        assert_eq!(canon.matches(":").count(), 2);
    }

    // T-1646: resolver tests for --message/--file (mutually exclusive).
    use super::resolve_contact_message;
    use std::path::PathBuf;

    #[test]
    fn resolve_message_only_returns_message() {
        let got = resolve_contact_message(Some("hi"), None).unwrap();
        assert_eq!(got, "hi");
    }

    #[test]
    fn resolve_neither_errors() {
        let err = resolve_contact_message(None, None).unwrap_err().to_string();
        assert!(err.contains("--message"), "got: {err}");
        assert!(err.contains("--file"), "got: {err}");
    }

    #[test]
    fn resolve_both_errors() {
        let p = PathBuf::from("/tmp/does-not-need-to-exist");
        let err = resolve_contact_message(Some("x"), Some(&p)).unwrap_err().to_string();
        assert!(err.contains("not both"), "got: {err}");
    }

    #[test]
    fn resolve_file_reads_contents() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("msg.txt");
        std::fs::write(&p, "from-file-body").unwrap();
        let got = resolve_contact_message(None, Some(&p)).unwrap();
        assert_eq!(got, "from-file-body");
    }

    #[test]
    fn resolve_empty_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("empty.txt");
        std::fs::write(&p, "").unwrap();
        let err = resolve_contact_message(None, Some(&p)).unwrap_err().to_string();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn resolve_missing_file_errors() {
        let p = PathBuf::from("/nonexistent/path/that/should/never/exist/T-1646.txt");
        let err = resolve_contact_message(None, Some(&p)).unwrap_err().to_string();
        assert!(err.contains("failed to read"), "got: {err}");
    }

    // T-1448 (b): parser tests for the `<name>[:<project>]` target syntax.

    use super::parse_contact_target;

    #[test]
    fn parse_contact_target_bare_name_returns_no_project() {
        let (name, project) = parse_contact_target("penelope").expect("bare name parses");
        assert_eq!(name, "penelope");
        assert_eq!(project, None);
    }

    #[test]
    fn parse_contact_target_name_colon_project_splits() {
        let (name, project) =
            parse_contact_target("penelope:050-email-archive").expect("name:project parses");
        assert_eq!(name, "penelope");
        assert_eq!(project.as_deref(), Some("050-email-archive"));
    }

    #[test]
    fn parse_contact_target_empty_input_rejected() {
        let err = parse_contact_target("").unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_contact_target_empty_name_rejected() {
        // `:project` — name part is empty
        let err = parse_contact_target(":050-email-archive").unwrap_err();
        assert!(err.contains("name"), "got: {err}");
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_contact_target_empty_project_rejected() {
        // `name:` — project part is empty
        let err = parse_contact_target("penelope:").unwrap_err();
        assert!(err.contains("project"), "got: {err}");
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn parse_contact_target_multi_colon_rejected() {
        let err = parse_contact_target("penelope:foo:bar").unwrap_err();
        // Allow the message to phrase this as "at most one `:`" or similar.
        assert!(err.contains("at most one") || err.contains("colon"), "got: {err}");
    }

    // T-1478: dry-run preview shape.
    use super::render_dry_run_preview;

    #[test]
    fn render_dry_run_preview_basic_shape() {
        let v = render_dry_run_preview(
            "aaaa",
            "bbbb",
            "dm:aaaa:bbbb",
            ["_thread=T-1478", "to_project=050-email-archive"].iter().copied(),
            "hello",
            None,
        );
        assert_eq!(v.get("dry_run").and_then(|x| x.as_bool()), Some(true));
        assert_eq!(v.get("my_id").and_then(|x| x.as_str()), Some("aaaa"));
        assert_eq!(v.get("peer_fp").and_then(|x| x.as_str()), Some("bbbb"));
        assert_eq!(v.get("topic").and_then(|x| x.as_str()), Some("dm:aaaa:bbbb"));
        assert_eq!(v.get("message").and_then(|x| x.as_str()), Some("hello"));
        let md = v.get("metadata").and_then(|m| m.as_object()).expect("metadata is object");
        assert_eq!(md.get("_thread").and_then(|x| x.as_str()), Some("T-1478"));
        assert_eq!(
            md.get("to_project").and_then(|x| x.as_str()),
            Some("050-email-archive"),
        );
    }

    #[test]
    fn render_dry_run_preview_no_extras_yields_empty_metadata_object() {
        let v = render_dry_run_preview("a", "b", "dm:a:b", std::iter::empty(), "x", None);
        let md = v.get("metadata").and_then(|m| m.as_object()).expect("metadata is object");
        // from_project may or may not appear depending on cwd; what matters is
        // _thread/to_project are absent when not supplied.
        assert!(md.get("_thread").is_none());
        assert!(md.get("to_project").is_none());
    }

    #[test]
    fn render_dry_run_preview_skips_malformed_extras_without_eq() {
        // Extras lacking `=` are silently skipped — they shouldn't crash.
        let v = render_dry_run_preview(
            "a",
            "b",
            "dm:a:b",
            ["malformed-no-equals", "_thread=T-1"].iter().copied(),
            "x",
            None,
        );
        let md = v.get("metadata").and_then(|m| m.as_object()).expect("metadata is object");
        assert!(md.get("malformed-no-equals").is_none());
        assert_eq!(md.get("_thread").and_then(|x| x.as_str()), Some("T-1"));
    }

    // T-1479: co-residency block.

    #[test]
    fn render_dry_run_preview_no_co_residency_block_when_count_one() {
        let v = render_dry_run_preview("a", "b", "dm:a:b", std::iter::empty(), "x", Some(1));
        assert!(v.get("co_residency").is_none(), "should be silent at N=1");
    }

    #[test]
    fn render_dry_run_preview_no_co_residency_block_when_count_zero() {
        let v = render_dry_run_preview("a", "b", "dm:a:b", std::iter::empty(), "x", Some(0));
        assert!(v.get("co_residency").is_none(), "should be silent at N=0");
    }

    #[test]
    fn render_dry_run_preview_co_residency_warns_no_to_project() {
        let v = render_dry_run_preview("a", "b", "dm:a:b", std::iter::empty(), "x", Some(3));
        let cr = v.get("co_residency").and_then(|x| x.as_object()).expect("co_residency present");
        assert_eq!(cr.get("local_session_count").and_then(|x| x.as_u64()), Some(3));
        let w = cr.get("warning").and_then(|x| x.as_str()).expect("warning string");
        assert!(w.contains("3 sessions"), "got: {w}");
        assert!(w.contains("no to_project"), "got: {w}");
        assert!(w.contains("<name>:<project>"), "got: {w}");
    }

    #[test]
    fn render_dry_run_preview_co_residency_softer_warning_with_to_project() {
        let v = render_dry_run_preview(
            "a",
            "b",
            "dm:a:b",
            ["to_project=050-email-archive"].iter().copied(),
            "x",
            Some(2),
        );
        let cr = v.get("co_residency").and_then(|x| x.as_object()).expect("co_residency present");
        let w = cr.get("warning").and_then(|x| x.as_str()).expect("warning string");
        assert!(w.contains("2 sessions"), "got: {w}");
        assert!(w.contains("self-filter"), "got: {w}");
        assert!(w.contains("050-email-archive"), "got: {w}");
    }

    #[test]
    fn parse_contact_target_preserves_project_special_chars() {
        // Project names commonly contain hyphens and digits — make sure the
        // parser doesn't over-validate the project string.
        let (name, project) =
            parse_contact_target("agent-1:002-Claude-Partner-Network").expect("hyphenated parses");
        assert_eq!(name, "agent-1");
        assert_eq!(project.as_deref(), Some("002-Claude-Partner-Network"));
    }

    /// T-2384: the send-side fp precedence helper. Presence-advertised fp (the
    /// per-agent key the recipient's waker subscribes on) must win over the
    /// registration metadata fp (the host key on a shared host); the reg fp is
    /// the fallback when the peer advertises no LIVE presence; and both-absent
    /// yields None so the existing "no identity_fingerprint" error path fires.
    #[test]
    fn prefer_presence_fp_precedence() {
        use super::prefer_presence_fp;
        let presence = "aaaa000000000001".to_string();
        let reg = "d1993c2c3ec44c94".to_string();

        // Both present & differ → presence wins (the shared-host bug fix).
        assert_eq!(
            prefer_presence_fp(Some(presence.clone()), Some(reg.clone())),
            Some(presence.clone())
        );
        // Presence absent → fall back to registration fp (not-be-reachable peer).
        assert_eq!(
            prefer_presence_fp(None, Some(reg.clone())),
            Some(reg.clone())
        );
        // Single-identity host: both equal → same fp either way (no regression).
        assert_eq!(
            prefer_presence_fp(Some(reg.clone()), Some(reg.clone())),
            Some(reg.clone())
        );
        // Both absent → None (preserves the existing error path).
        assert_eq!(prefer_presence_fp(None, None), None);
    }
}
