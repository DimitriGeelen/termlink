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

    // Snapshot cursor before emitting
    let cursor: Option<u64> = {
        let poll_params = serde_json::json!({});
        match client::rpc_call(reg.socket_path(), "event.poll", poll_params).await {
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
    let poll_interval = std::time::Duration::from_millis(interval);
    let mut poll_cursor = cursor;

    loop {
        let mut poll_params = serde_json::json!({});
        if let Some(c) = poll_cursor {
            poll_params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", poll_params).await {
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

                    if let Some(events) = result["events"].as_array()
                        && !events.is_empty()
                            && let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if start.elapsed() > timeout_dur {
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

        tokio::time::sleep(poll_interval).await;
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
    let poll_interval = std::time::Duration::from_millis(interval);

    let mut poll_cursor: Option<u64> = None;

    loop {
        let mut params = serde_json::json!({ "topic": agent_topic::REQUEST });
        if let Some(c) = poll_cursor {
            params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
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

                    if let Some(events) = result["events"].as_array()
                        && !events.is_empty()
                            && let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if let Some(td) = timeout_dur
            && start.elapsed() > td {
                if !json {
                    eprintln!("Listen timeout reached ({}s)", timeout);
                }
                return Ok(());
            }

        tokio::time::sleep(poll_interval).await;
    }
}

/// Run a 4-phase format negotiation with a specialist session.
///
/// Protocol: offer → attempt → correction → accept (max N rounds).
/// Uses agent.request/response events with negotiate.* actions.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_agent_negotiate(
    specialist: &str,
    schema_str: &str,
    draft_str: &str,
    from: Option<&str>,
    max_rounds: u8,
    timeout: u64,
    interval: u64,
    json: bool,
) -> Result<()> {
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

    // Snapshot cursor
    let cursor: Option<u64> = {
        let poll_params = serde_json::json!({});
        match client::rpc_call(reg.socket_path(), "event.poll", poll_params).await {
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
    let mut poll_cursor = cursor;

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
        let poll_interval_dur = std::time::Duration::from_millis(interval);
        let mut got_response = false;

        while !got_response {
            let mut poll_params = serde_json::json!({});
            if let Some(c) = poll_cursor {
                poll_params["since"] = serde_json::json!(c);
            }

            if let Ok(resp) =
                client::rpc_call(reg.socket_path(), "event.poll", poll_params).await
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

                    if let Some(events) = result["events"].as_array()
                        && !events.is_empty()
                            && let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
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

            if !got_response {
                tokio::time::sleep(poll_interval_dur).await;
            }
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
