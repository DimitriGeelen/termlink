//! T-2045 (T-2020 GO): `termlink agent find-idle` CLI verb.
//!
//! Calls the hub's `agent.find_idle` RPC (T-2045 slice 1) over the local
//! UDS socket and renders the result. Pure read — no state mutation.
//!
//! Local-hub-only by design (per T-2020 inception §5.4 "What's NOT in this
//! primitive"). Cross-hub finding is the orchestrator's job — it walks
//! `hubs.toml` and calls find-idle per hub.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use termlink_protocol::control::method;
use termlink_protocol::transport::TransportAddr;
use termlink_session::client;

pub(crate) async fn cmd_agent_find_idle(
    role: Option<&str>,
    capabilities: &[String],
    limit: Option<u32>,
    json_output: bool,
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

    let mut params = json!({});
    if let Some(r) = role {
        params["role"] = json!(r);
    }
    if !capabilities.is_empty() {
        params["capabilities"] = json!(capabilities);
    }
    if let Some(n) = limit {
        params["limit"] = json!(n);
    }

    let resp = client::rpc_call_addr(&addr, method::AGENT_FIND_IDLE, params)
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
    for entry in &idle {
        let agent_id = entry.get("agent_id").and_then(|v| v.as_str()).unwrap_or("?");
        let last_hb = entry
            .get("last_heartbeat_ms")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let age_s = ((now_ms - last_hb) / 1000).max(0);
        let role_str = entry
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let caps: Vec<String> = entry
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let caps_str = if caps.is_empty() {
            "-".to_string()
        } else {
            caps.join(",")
        };
        println!(
            "{agent_id}\tage={age_s}s\trole={role_str}\tcapabilities={caps_str}"
        );
    }
    Ok(())
}
