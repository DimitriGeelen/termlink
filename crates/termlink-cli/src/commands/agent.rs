use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

use termlink_protocol::events::{
    agent_topic, AgentRequest, AgentResponse, AgentStatus, SCHEMA_VERSION,
};

use crate::util::generate_request_id;

pub(crate) async fn cmd_agent_ask(
    target: &str,
    action: &str,
    params_str: &str,
    from: Option<&str>,
    timeout: u64,
    interval: u64,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let request_id = generate_request_id();
    let sender = from.map(|s| s.to_string()).unwrap_or_else(|| format!("cli-{}", std::process::id()));

    let params: serde_json::Value = serde_json::from_str(params_str)
        .context("Invalid JSON in --params")?;

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
    let payload = serde_json::to_value(&request)
        .context("Failed to serialize AgentRequest")?;
    let emit_params = serde_json::json!({
        "topic": agent_topic::REQUEST,
        "payload": payload,
    });

    let emit_resp = client::rpc_call(reg.socket_path(), "event.emit", emit_params)
        .await
        .context("Failed to emit agent request")?;

    match client::unwrap_result(emit_resp) {
        Ok(result) => {
            let seq = result["seq"].as_u64().unwrap_or(0);
            eprintln!("Request sent: action={}, request_id={}, seq={}", action, request_id, seq);
        }
        Err(e) => {
            anyhow::bail!("Failed to emit agent request: {}", e);
        }
    }

    // Poll for agent.response and agent.status
    eprintln!("Waiting for response (timeout: {}s)...", timeout);

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
                                    if response.status == termlink_protocol::events::ResponseStatus::Ok {
                                        println!("{}", serde_json::to_string_pretty(&response.result)?);
                                    } else {
                                        let msg = response.error_message.as_deref().unwrap_or("unknown error");
                                        eprintln!("Error: {}", msg);
                                        std::process::exit(1);
                                    }
                                } else {
                                    println!("{}", serde_json::to_string_pretty(event_payload)?);
                                }
                                return Ok(());
                            }

                            if topic == agent_topic::STATUS
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
            anyhow::bail!("Timeout waiting for agent response ({}s). request_id={}", timeout, request_id);
        }

        tokio::time::sleep(poll_interval).await;
    }
}

pub(crate) async fn cmd_agent_listen(
    target: &str,
    timeout: u64,
    interval: u64,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    eprintln!("Listening for agent requests on '{}' (topic: {})...", target, agent_topic::REQUEST);
    if timeout > 0 {
        eprintln!("Timeout: {}s", timeout);
    } else {
        eprintln!("Press Ctrl+C to stop");
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
                            if let Ok(req) = serde_json::from_value::<AgentRequest>(event_payload.clone()) {
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
                eprintln!("Listen timeout reached ({}s)", timeout);
                return Ok(());
            }

        tokio::time::sleep(poll_interval).await;
    }
}
