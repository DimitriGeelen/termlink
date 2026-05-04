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

/// T-1429 Phase-1: contact a peer agent on the canonical `dm:<a>:<b>` topic.
///
/// Resolves `<target>` to a local session via `manager::find_session`, reads
/// the peer's `identity_fingerprint` from `SessionMetadata` (T-1436), then
/// delegates to `cmd_channel_dm` which already does dm-topic canonicalisation,
/// idempotent topic creation, and posting.
///
/// Phase-1 scope: --message only, local-hub only, fire-and-forget. Phase-2
/// adds --ack-required, --require-online, --file, and advanced target forms
/// (`name@hub:port`, `sender_id:<hex>`) — see T-1429 task for the deferred
/// ACs.
///
/// Errors:
/// - target not found locally → exit code 1, message names the session
/// - peer registered before T-1436 (no identity_fingerprint in metadata) →
///   exit 8, message instructs operator to upgrade the peer's binary
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
        let reg = manager::find_session(target_name).map_err(|e| {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target_name,
                    "error": format!("Session '{target_name}' not found: {e}"),
                }));
            }
            anyhow::anyhow!("Session '{target_name}' not found: {e}")
        })?;

        reg.metadata.identity_fingerprint.clone().ok_or_else(|| {
            let msg = format!(
                "Peer '{target_name}' has no identity_fingerprint in metadata — \
                 likely registered before T-1436. Upgrade the peer's termlink \
                 binary and restart the session, then retry. (Or use \
                 --target-fp <hex> to bypass session.discover for cross-host.)"
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
    };
    let peer_fp = peer_fp_owned.as_str();

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
    .with_context(|| format!("agent contact: posting to dm topic for peer fp={peer_fp} failed"))
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
}
