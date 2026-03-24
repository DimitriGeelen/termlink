use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use termlink_session::{client, manager};

/// TermLink MCP server — exposes terminal orchestration as structured tools.
#[derive(Debug, Clone)]
pub struct TermLinkTools {
    pub tool_router: ToolRouter<Self>,
}

impl TermLinkTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

// === Parameter types ===

#[derive(Deserialize, JsonSchema)]
pub struct PingParams {
    /// Session ID or display name
    pub target: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ExecParams {
    /// Session ID or display name
    pub target: String,
    /// Command to execute
    pub command: String,
    /// Working directory (optional)
    pub cwd: Option<String>,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct OutputParams {
    /// Session ID or display name
    pub target: String,
    /// Number of lines to return (default: 50)
    pub lines: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct InjectParams {
    /// Session ID or display name
    pub target: String,
    /// Text to inject into the terminal
    pub text: String,
    /// Press Enter after injection (default: false)
    pub enter: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SignalParams {
    /// Session ID or display name
    pub target: String,
    /// Signal name: TERM, INT, KILL, HUP, USR1, USR2
    pub signal: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct EmitParams {
    /// Session ID or display name
    pub target: String,
    /// Event topic (e.g., "build.complete", "task.result")
    pub topic: String,
    /// JSON payload (optional)
    pub payload: Option<serde_json::Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EmitToParams {
    /// Target session ID or display name (receives the event)
    pub target: String,
    /// Event topic
    pub topic: String,
    /// JSON payload (optional)
    pub payload: Option<serde_json::Value>,
    /// Sender session ID (for traceability)
    pub from: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct WaitParams {
    /// Session ID or display name
    pub target: String,
    /// Event topic to wait for
    pub topic: String,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusParams {
    /// Session ID or display name
    pub target: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct EventPollParams {
    /// Session ID or display name
    pub target: String,
    /// Only return events after this sequence number
    pub since: Option<u64>,
    /// Filter by topic
    pub topic: Option<String>,
}

// === Result types ===

#[derive(Serialize, JsonSchema)]
pub struct SessionInfo {
    pub id: String,
    pub display_name: String,
    pub state: String,
    pub pid: u32,
    pub tags: Vec<String>,
    pub roles: Vec<String>,
}

// === Tool implementations ===

fn parse_signal(name: &str) -> Option<i32> {
    match name.to_uppercase().as_str() {
        "TERM" | "SIGTERM" => Some(libc::SIGTERM),
        "INT" | "SIGINT" => Some(libc::SIGINT),
        "KILL" | "SIGKILL" => Some(libc::SIGKILL),
        "HUP" | "SIGHUP" => Some(libc::SIGHUP),
        "USR1" | "SIGUSR1" => Some(libc::SIGUSR1),
        "USR2" | "SIGUSR2" => Some(libc::SIGUSR2),
        _ => name.parse().ok(),
    }
}

#[tool_router]
impl TermLinkTools {
    #[tool(
        name = "termlink_ping",
        description = "Check if a TermLink session is alive and responding"
    )]
    async fn termlink_ping(&self, Parameters(p): Parameters<PingParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        match client::rpc_call(reg.socket_path(), "termlink.ping", serde_json::json!({})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| "PONG".into()),
                Err(e) => format!("Error: ping failed: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_list_sessions",
        description = "List all active TermLink sessions with their IDs, names, states, and tags"
    )]
    async fn termlink_list_sessions(&self) -> String {
        match manager::list_sessions(false) {
            Ok(sessions) => {
                let infos: Vec<SessionInfo> = sessions
                    .iter()
                    .map(|s| SessionInfo {
                        id: s.id.to_string(),
                        display_name: s.display_name.clone(),
                        state: s.state.to_string(),
                        pid: s.pid,
                        tags: s.tags.clone(),
                        roles: s.roles.clone(),
                    })
                    .collect();
                serde_json::to_string_pretty(&infos).unwrap_or_else(|_| "[]".into())
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "termlink_status",
        description = "Get detailed status of a TermLink session including capabilities, tags, and metadata"
    )]
    async fn termlink_status(&self, Parameters(p): Parameters<StatusParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        match client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("Error: {e}")),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_exec",
        description = "Execute a command on a TermLink session and return stdout/stderr/exit_code"
    )]
    async fn termlink_exec(&self, Parameters(p): Parameters<ExecParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let mut params = serde_json::json!({
            "command": p.command,
            "timeout": p.timeout.unwrap_or(30),
        });
        if let Some(cwd) = &p.cwd {
            params["cwd"] = serde_json::json!(cwd);
        }

        match client::rpc_call(reg.socket_path(), "command.execute", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => {
                    let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
                    let stdout = result["stdout"].as_str().unwrap_or("");
                    let stderr = result["stderr"].as_str().unwrap_or("");

                    let mut output = String::new();
                    if !stdout.is_empty() {
                        output.push_str(stdout);
                    }
                    if !stderr.is_empty() {
                        if !output.is_empty() {
                            output.push('\n');
                        }
                        output.push_str(&format!("[stderr] {stderr}"));
                    }
                    if exit_code != 0 {
                        output.push_str(&format!("\n[exit_code: {exit_code}]"));
                    }
                    if output.is_empty() {
                        format!("[exit_code: {exit_code}]")
                    } else {
                        output
                    }
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_output",
        description = "Read recent terminal output from a PTY-backed TermLink session"
    )]
    async fn termlink_output(&self, Parameters(p): Parameters<OutputParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let params = serde_json::json!({
            "lines": p.lines.unwrap_or(50),
        });

        match client::rpc_call(reg.socket_path(), "query.output", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => result["output"].as_str().unwrap_or("").to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_inject",
        description = "Inject text (keystrokes) into a PTY-backed TermLink session"
    )]
    async fn termlink_inject(&self, Parameters(p): Parameters<InjectParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let mut keys = vec![serde_json::json!({"type": "text", "value": p.text})];
        if p.enter.unwrap_or(false) {
            keys.push(serde_json::json!({"type": "key", "value": "Enter"}));
        }

        match client::rpc_call(reg.socket_path(), "command.inject", serde_json::json!({"keys": keys})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(_) => "Injected successfully".to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_signal",
        description = "Send a signal (TERM, INT, KILL, HUP, USR1, USR2) to a TermLink session's process"
    )]
    async fn termlink_signal(&self, Parameters(p): Parameters<SignalParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let sig_num = match parse_signal(&p.signal) {
            Some(n) => n,
            None => return format!("Error: unknown signal '{}'. Use TERM, INT, KILL, HUP, USR1, USR2", p.signal),
        };

        match client::rpc_call(reg.socket_path(), "command.signal", serde_json::json!({"signal": sig_num})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => format!(
                    "Signal {} sent to PID {}",
                    result["signal"].as_i64().unwrap_or(sig_num as i64),
                    result["pid"].as_u64().unwrap_or(0),
                ),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_emit",
        description = "Emit an event to a session's event bus"
    )]
    async fn termlink_emit(&self, Parameters(p): Parameters<EmitParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let params = serde_json::json!({
            "topic": p.topic,
            "payload": p.payload.unwrap_or(serde_json::json!({})),
        });

        match client::rpc_call(reg.socket_path(), "event.emit", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => format!(
                    "Emitted: {} (seq: {})",
                    result["topic"].as_str().unwrap_or("?"),
                    result["seq"].as_u64().unwrap_or(0),
                ),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_emit_to",
        description = "Push an event directly to a target session's event bus via the hub (no polling needed)"
    )]
    async fn termlink_emit_to(&self, Parameters(p): Parameters<EmitToParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return "Error: hub is not running. Start it with: termlink hub".into();
        }

        let mut params = serde_json::json!({
            "target": p.target,
            "topic": p.topic,
            "payload": p.payload.unwrap_or(serde_json::json!({})),
        });
        if let Some(from) = &p.from {
            params["from"] = serde_json::json!(from);
        }

        match client::rpc_call(&hub_socket, "event.emit_to", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => format!(
                    "Pushed to {}: {} (seq: {})",
                    result["target"].as_str().unwrap_or("?"),
                    result["topic"].as_str().unwrap_or("?"),
                    result["seq"].as_u64().unwrap_or(0),
                ),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_event_poll",
        description = "Poll events from a session's event bus, optionally filtered by topic and sequence number"
    )]
    async fn termlink_event_poll(&self, Parameters(p): Parameters<EventPollParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let mut params = serde_json::json!({});
        if let Some(since) = p.since {
            params["since"] = serde_json::json!(since);
        }
        if let Some(topic) = &p.topic {
            params["topic"] = serde_json::json!(topic);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("Error: {e}")),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }
}
