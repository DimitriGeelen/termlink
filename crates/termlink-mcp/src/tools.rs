use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use termlink_protocol::format_age;
use termlink_session::{client, manager};

/// TermLink MCP server — exposes terminal orchestration as structured tools.
#[derive(Debug, Clone)]
pub struct TermLinkTools {
    pub tool_router: ToolRouter<Self>,
}

impl Default for TermLinkTools {
    fn default() -> Self {
        Self::new()
    }
}

impl TermLinkTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

/// Helper: create a JSON error response string.
fn json_err(msg: impl std::fmt::Display) -> String {
    serde_json::to_string_pretty(&serde_json::json!({"ok": false, "error": msg.to_string()}))
        .unwrap_or_else(|e| format!("{{\"ok\":false,\"error\":\"{e}\"}}" ))
}

// === Parameter types ===

#[derive(Deserialize, JsonSchema)]
pub struct ListSessionsParams {
    /// Filter by tag (sessions must have this tag)
    pub tag: Option<String>,
    /// Filter by role (sessions must have this role)
    pub role: Option<String>,
    /// Filter by display name (substring match)
    pub name: Option<String>,
}

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
pub struct DiscoverParams {
    /// Filter by tags (sessions must have ALL specified tags)
    pub tags: Option<Vec<String>>,
    /// Filter by roles (sessions must have ALL specified roles)
    pub roles: Option<Vec<String>>,
    /// Filter by capabilities (sessions must have ALL specified capabilities)
    pub cap: Option<Vec<String>>,
    /// Filter by display name (case-insensitive substring match)
    pub name: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SpawnParams {
    /// Display name for the new session
    pub name: Option<String>,
    /// Roles to assign (e.g., "worker", "specialist")
    pub roles: Option<Vec<String>>,
    /// Tags to assign
    pub tags: Option<Vec<String>>,
    /// Command to run in the session (if empty, starts a shell)
    pub command: Option<Vec<String>>,
    /// Wait for session to register before returning (default: true)
    pub wait: Option<bool>,
    /// Wait timeout in seconds (default: 10)
    pub wait_timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RunParams {
    /// Command to execute in an ephemeral session
    pub command: String,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct KvSetParams {
    /// Session ID or display name
    pub target: String,
    /// Key to set
    pub key: String,
    /// Value (any JSON value)
    pub value: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct KvGetParams {
    /// Session ID or display name
    pub target: String,
    /// Key to retrieve
    pub key: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct KvListParams {
    /// Session ID or display name
    pub target: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct KvDelParams {
    /// Session ID or display name
    pub target: String,
    /// Key to delete
    pub key: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct BroadcastParams {
    /// Event topic
    pub topic: String,
    /// JSON payload (optional)
    pub payload: Option<serde_json::Value>,
    /// Target session IDs or names (empty = all sessions)
    pub targets: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct InteractParams {
    /// Session ID or display name (must be a PTY session)
    pub target: String,
    /// Shell command to run in the PTY (e.g., "ls -la", "git status")
    pub command: String,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
    /// Poll interval in milliseconds (default: 200)
    pub poll_ms: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusParams {
    /// Session ID or display name
    pub target: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TagParams {
    /// Session ID or display name
    pub target: String,
    /// Replace all tags with this list (mutually exclusive with add/remove)
    pub set: Option<Vec<String>>,
    /// Tags to add to the session
    pub add: Option<Vec<String>>,
    /// Tags to remove from the session
    pub remove: Option<Vec<String>>,
    /// Set a new display name for the session
    pub name: Option<String>,
    /// Replace all roles with this list
    pub roles: Option<Vec<String>>,
    /// Roles to add to the session
    pub add_roles: Option<Vec<String>>,
    /// Roles to remove from the session
    pub remove_roles: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RequestParams {
    /// Session ID or display name to send the request to
    pub target: String,
    /// Event topic for the request (e.g., "task.run")
    pub topic: String,
    /// JSON payload to include in the request
    pub payload: Option<serde_json::Value>,
    /// Topic to wait for the reply on (e.g., "task.result")
    pub reply_topic: String,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ResizeParams {
    /// Session ID or display name
    pub target: String,
    /// Number of columns (width)
    pub cols: u16,
    /// Number of rows (height)
    pub rows: u16,
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

#[derive(Deserialize, JsonSchema)]
pub struct EventSubscribeParams {
    /// Session ID or display name
    pub target: String,
    /// Timeout in milliseconds (default 5000). Server blocks until events arrive or timeout.
    pub timeout_ms: Option<u64>,
    /// Filter by topic
    pub topic: Option<String>,
    /// Replay historical events with seq > since before streaming live events
    pub since: Option<u64>,
    /// Maximum events to return (default 100)
    pub max_events: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TopicsParams {
    /// Session ID or display name (if omitted, queries all sessions)
    pub target: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PtyModeParams {
    /// Session ID or display name
    pub target: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CollectParams {
    /// Target session names to collect from (if omitted, collects from all hub-known sessions)
    pub targets: Option<Vec<String>>,
    /// Filter by event topic
    pub topic: Option<String>,
    /// Timeout in milliseconds for push-based delivery (default: 5000). Hub blocks until events arrive or timeout.
    pub timeout_ms: Option<u64>,
    /// Per-session cursors for continuation (map of session_name → last_seen_seq)
    pub since: Option<serde_json::Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FileSendParams {
    /// Session ID or display name to send the file to
    pub target: String,
    /// Absolute path to the file to send
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FileReceiveParams {
    /// Session ID or display name to receive the file from
    pub target: String,
    /// Directory to write the received file into (must exist)
    pub output_dir: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TokenCreateParams {
    /// Session ID or display name (must have --token-secret enabled)
    pub target: String,
    /// Permission scope: "observe", "control", or "execute"
    pub scope: Option<String>,
    /// Time-to-live in seconds (default: 3600)
    pub ttl: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TokenInspectParams {
    /// The token string to inspect (format: payload.signature)
    pub token: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentAskParams {
    /// Session ID or display name to send the agent request to
    pub target: String,
    /// Action to request (e.g., "analyze", "build", "test")
    pub action: String,
    /// JSON parameters for the action (default: {})
    pub params: Option<serde_json::Value>,
    /// Sender identity (default: mcp-<pid>)
    pub from: Option<String>,
    /// Timeout in seconds to wait for response (default: 30)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SendParams {
    /// Session ID or display name to send the RPC call to
    pub target: String,
    /// JSON-RPC method name (e.g., "termlink.ping", "query.capabilities")
    pub method: String,
    /// JSON parameters for the method (default: {})
    pub params: Option<String>,
    /// Timeout in seconds (default: 10)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchExecParams {
    /// Shell command to run on each matching session
    pub command: String,
    /// Filter by tag (sessions must have this tag)
    pub tag: Option<String>,
    /// Filter by role (sessions must have this role)
    pub role: Option<String>,
    /// Filter by display name (substring match)
    pub name: Option<String>,
    /// Timeout per session in seconds (default: 30)
    pub timeout: Option<u64>,
    /// Maximum parallel executions (default: 10)
    pub max_parallel: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchPingParams {
    /// Filter by tag (sessions must have this tag)
    pub tag: Option<String>,
    /// Filter by role (sessions must have this role)
    pub role: Option<String>,
    /// Filter by display name (substring match)
    pub name: Option<String>,
    /// Timeout per ping in seconds (default: 5)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchTagParams {
    /// Filter: only apply to sessions with this tag
    pub filter_tag: Option<String>,
    /// Filter: only apply to sessions with this role
    pub filter_role: Option<String>,
    /// Filter: only apply to sessions matching this name (substring)
    pub filter_name: Option<String>,
    /// Tags to add to matching sessions
    pub add_tags: Option<Vec<String>>,
    /// Tags to remove from matching sessions
    pub remove_tags: Option<Vec<String>>,
    /// Roles to add to matching sessions
    pub add_roles: Option<Vec<String>>,
    /// Roles to remove from matching sessions
    pub remove_roles: Option<Vec<String>>,
}

// === Result types ===

#[derive(Serialize, JsonSchema)]
pub struct SessionInfo {
    pub id: String,
    pub display_name: String,
    pub state: String,
    pub pid: u32,
    pub uid: u32,
    pub created_at: String,
    pub heartbeat_at: String,
    /// Human-readable age (e.g., "3d", "2h", "45m", "12s")
    pub age: String,
    pub tags: Vec<String>,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
    pub metadata: serde_json::Value,
}

// === Tool implementations ===

pub(crate) fn parse_signal(name: &str) -> Option<i32> {
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
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        match client::rpc_call(reg.socket_path(), "termlink.ping", serde_json::json!({})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| "PONG".into()),
                Err(e) => json_err(format!("ping failed: {e}")),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_list_sessions",
        description = "List active TermLink sessions with optional filtering by tag, role, or name. All filters are optional — omit all for a full list."
    )]
    async fn termlink_list_sessions(&self, Parameters(p): Parameters<ListSessionsParams>) -> String {
        match manager::list_sessions(false) {
            Ok(sessions) => {
                let filtered: Vec<_> = sessions
                    .iter()
                    .filter(|s| {
                        if p.tag.as_ref().is_some_and(|tag| !s.tags.iter().any(|t| t == tag)) {
                            return false;
                        }
                        if p.role.as_ref().is_some_and(|role| !s.roles.iter().any(|r| r == role)) {
                            return false;
                        }
                        if p.name.as_ref().is_some_and(|name| !s.display_name.contains(name.as_str())) {
                            return false;
                        }
                        true
                    })
                    .collect();

                let infos: Vec<SessionInfo> = filtered
                    .iter()
                    .map(|s| SessionInfo {
                        id: s.id.to_string(),
                        display_name: s.display_name.clone(),
                        state: s.state.to_string(),
                        pid: s.pid,
                        uid: s.uid,
                        created_at: s.created_at.clone(),
                        heartbeat_at: s.heartbeat_at.clone(),
                        age: format_age(&s.created_at),
                        tags: s.tags.clone(),
                        roles: s.roles.clone(),
                        capabilities: s.capabilities.clone(),
                        metadata: serde_json::to_value(&s.metadata).unwrap_or_default(),
                    })
                    .collect();
                serde_json::to_string_pretty(&infos).unwrap_or_else(|_| "[]".into())
            }
            Err(e) => json_err(e),
        }
    }

    #[tool(
        name = "termlink_status",
        description = "Get detailed status of a TermLink session including capabilities, tags, and metadata"
    )]
    async fn termlink_status(&self, Parameters(p): Parameters<StatusParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        match client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_exec",
        description = "Execute a command on a TermLink session and return stdout/stderr/exit_code"
    )]
    async fn termlink_exec(&self, Parameters(p): Parameters<ExecParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
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

                    let response = serde_json::json!({
                        "ok": exit_code == 0,
                        "exit_code": exit_code,
                        "stdout": stdout,
                        "stderr": stderr,
                        "target": p.target,
                    });
                    serde_json::to_string_pretty(&response)
                        .unwrap_or_else(json_err)
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_output",
        description = "Read recent terminal output from a PTY-backed TermLink session"
    )]
    async fn termlink_output(&self, Parameters(p): Parameters<OutputParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let params = serde_json::json!({
            "lines": p.lines.unwrap_or(50),
        });

        match client::rpc_call(reg.socket_path(), "query.output", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => result["output"].as_str().unwrap_or("").to_string(),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_inject",
        description = "Inject text (keystrokes) into a PTY-backed TermLink session"
    )]
    async fn termlink_inject(&self, Parameters(p): Parameters<InjectParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let mut keys = vec![serde_json::json!({"type": "text", "value": p.text})];
        if p.enter.unwrap_or(false) {
            keys.push(serde_json::json!({"type": "key", "value": "Enter"}));
        }

        match client::rpc_call(reg.socket_path(), "command.inject", serde_json::json!({"keys": keys})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(_) => "Injected successfully".to_string(),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_signal",
        description = "Send a signal (TERM, INT, KILL, HUP, USR1, USR2) to a TermLink session's process"
    )]
    async fn termlink_signal(&self, Parameters(p): Parameters<SignalParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let sig_num = match parse_signal(&p.signal) {
            Some(n) => n,
            None => return json_err(format!("unknown signal '{}'. Use TERM, INT, KILL, HUP, USR1, USR2", p.signal)),
        };

        match client::rpc_call(reg.socket_path(), "command.signal", serde_json::json!({"signal": sig_num})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => format!(
                    "Signal {} sent to PID {}",
                    result["signal"].as_i64().unwrap_or(sig_num as i64),
                    result["pid"].as_u64().unwrap_or(0),
                ),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_emit",
        description = "Emit an event to a session's event bus"
    )]
    async fn termlink_emit(&self, Parameters(p): Parameters<EmitParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
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
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_emit_to",
        description = "Push an event directly to a target session's event bus via the hub (no polling needed)"
    )]
    async fn termlink_emit_to(&self, Parameters(p): Parameters<EmitToParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("hub is not running. Start it with: termlink hub");
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
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_event_poll",
        description = "Poll events from a session's event bus, optionally filtered by topic and sequence number"
    )]
    async fn termlink_event_poll(&self, Parameters(p): Parameters<EventPollParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
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
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_event_subscribe",
        description = "Subscribe to events from a session using push-based delivery. Blocks until events arrive or timeout. Lower latency than polling. Optional 'since' parameter replays historical events before streaming live ones."
    )]
    async fn termlink_event_subscribe(&self, Parameters(p): Parameters<EventSubscribeParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let mut params = serde_json::json!({});
        if let Some(timeout_ms) = p.timeout_ms {
            params["timeout_ms"] = serde_json::json!(timeout_ms);
        }
        if let Some(topic) = &p.topic {
            params["topic"] = serde_json::json!(topic);
        }
        if let Some(since) = p.since {
            params["since"] = serde_json::json!(since);
        }
        if let Some(max_events) = p.max_events {
            params["max_events"] = serde_json::json!(max_events);
        }

        match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_discover",
        description = "Find TermLink sessions by tag, role, or name. Returns matching sessions with IDs, tags, roles, and capabilities."
    )]
    async fn termlink_discover(&self, Parameters(p): Parameters<DiscoverParams>) -> String {
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => return json_err(e),
        };

        let tags = p.tags.unwrap_or_default();
        let roles = p.roles.unwrap_or_default();
        let caps = p.cap.unwrap_or_default();

        let filtered: Vec<_> = sessions
            .into_iter()
            .filter(|s| {
                tags.iter().all(|t| s.tags.contains(t))
                    && roles.iter().all(|r| s.roles.contains(r))
                    && caps.iter().all(|c| s.capabilities.contains(c))
                    && p.name.as_ref().is_none_or(|n| {
                        s.display_name.to_lowercase().contains(&n.to_lowercase())
                    })
            })
            .collect();

        let items: Vec<serde_json::Value> = filtered
            .iter()
            .map(|s| {
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
                })
            })
            .collect();

        serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".into())
    }

    #[tool(
        name = "termlink_spawn",
        description = "Spawn a new TermLink session in the background. Returns the session name. Use with --wait to block until registered."
    )]
    async fn termlink_spawn(&self, Parameters(p): Parameters<SpawnParams>) -> String {
        let session_name = p.name.unwrap_or_else(|| format!("mcp-spawn-{}", std::process::id()));
        let roles = p.roles.unwrap_or_default();
        let tags = p.tags.unwrap_or_default();
        let command = p.command.unwrap_or_default();
        let wait = p.wait.unwrap_or(true);
        let wait_timeout = p.wait_timeout.unwrap_or(10);

        let termlink_bin = match std::env::current_exe() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => return json_err(format!("cannot determine termlink binary: {e}")),
        };

        let mut register_args = vec![
            "register".to_string(),
            "--name".to_string(),
            session_name.clone(),
        ];
        if !roles.is_empty() {
            register_args.push("--roles".to_string());
            register_args.push(roles.join(","));
        }
        if !tags.is_empty() {
            register_args.push("--tags".to_string());
            register_args.push(tags.join(","));
        }
        if command.is_empty() {
            register_args.push("--shell".to_string());
        }

        let shell_cmd = if command.is_empty() {
            let mut parts = vec![termlink_bin];
            parts.extend(register_args);
            parts.join(" ")
        } else {
            let mut reg_parts = vec![termlink_bin];
            reg_parts.extend(register_args);
            let user_cmd = command.join(" ");
            format!(
                "{} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nkill $TL_PID 2>/dev/null\nwait $TL_PID 2>/dev/null",
                reg_parts.join(" ")
            )
        };

        let child = std::process::Command::new("sh")
            .args(["-c", &shell_cmd])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null())
            .spawn();

        if let Err(e) = child {
            return json_err(format!("failed to spawn: {e}"));
        }

        if wait {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(wait_timeout);
            loop {
                if manager::find_session(&session_name).is_ok() {
                    return format!("Spawned session '{}' (ready)", session_name);
                }
                if start.elapsed() > timeout {
                    return format!("Spawned session '{}' (timeout waiting for registration)", session_name);
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        }

        format!("Spawned session '{}'", session_name)
    }

    #[tool(
        name = "termlink_run",
        description = "Execute a command in an ephemeral TermLink session and return the output. The session is cleaned up after execution."
    )]
    async fn termlink_run(&self, Parameters(p): Parameters<RunParams>) -> String {
        use termlink_session::executor;

        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(30));

        match executor::execute(&p.command, None, None, Some(timeout), None).await {
            Ok(result) => {
                let response = serde_json::json!({
                    "ok": result.exit_code == 0,
                    "exit_code": result.exit_code,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "command": p.command,
                });
                serde_json::to_string_pretty(&response)
                    .unwrap_or_else(json_err)
            }
            Err(e) => json_err(e),
        }
    }

    #[tool(
        name = "termlink_kv_set",
        description = "Set a key-value pair on a TermLink session's store"
    )]
    async fn termlink_kv_set(&self, Parameters(p): Parameters<KvSetParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let params = serde_json::json!({"key": p.key, "value": p.value});
        match client::rpc_call(reg.socket_path(), "kv.set", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => {
                    let replaced = result["replaced"].as_bool().unwrap_or(false);
                    format!(
                        "{} {}={}",
                        if replaced { "Updated" } else { "Set" },
                        result["key"].as_str().unwrap_or("?"),
                        serde_json::to_string(&p.value).unwrap_or_default(),
                    )
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_kv_get",
        description = "Get a value from a TermLink session's key-value store"
    )]
    async fn termlink_kv_get(&self, Parameters(p): Parameters<KvGetParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        match client::rpc_call(reg.socket_path(), "kv.get", serde_json::json!({"key": p.key})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => {
                    if result["found"].as_bool().unwrap_or(false) {
                        serde_json::to_string_pretty(&result["value"])
                            .unwrap_or_else(|_| "null".into())
                    } else {
                        format!("Key '{}' not found", p.key)
                    }
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_kv_list",
        description = "List all key-value pairs stored on a TermLink session"
    )]
    async fn termlink_kv_list(&self, Parameters(p): Parameters<KvListParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        match client::rpc_call(reg.socket_path(), "kv.list", serde_json::json!({})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result)
                    .unwrap_or_else(json_err),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_kv_del",
        description = "Delete a key from a TermLink session's key-value store"
    )]
    async fn termlink_kv_del(&self, Parameters(p): Parameters<KvDelParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        match client::rpc_call(reg.socket_path(), "kv.delete", serde_json::json!({"key": p.key})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => {
                    if result["deleted"].as_bool().unwrap_or(false) {
                        format!("Deleted '{}'", p.key)
                    } else {
                        format!("Key '{}' not found", p.key)
                    }
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_broadcast",
        description = "Broadcast an event to multiple TermLink sessions via the hub. If no targets specified, broadcasts to all."
    )]
    async fn termlink_broadcast(&self, Parameters(p): Parameters<BroadcastParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("hub is not running. Start it with: termlink hub");
        }

        let mut params = serde_json::json!({
            "topic": p.topic,
            "payload": p.payload.unwrap_or(serde_json::json!({})),
        });
        if let Some(targets) = &p.targets
            && !targets.is_empty() {
                params["targets"] = serde_json::json!(targets);
            }

        match client::rpc_call(&hub_socket, "event.broadcast", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => {
                    let targeted = result["targeted"].as_u64().unwrap_or(0);
                    let succeeded = result["succeeded"].as_u64().unwrap_or(0);
                    let failed = result["failed"].as_u64().unwrap_or(0);
                    format!(
                        "Broadcast '{}': {}/{} succeeded{}",
                        result["topic"].as_str().unwrap_or(&p.topic),
                        succeeded,
                        targeted,
                        if failed > 0 { format!(" ({} failed)", failed) } else { String::new() },
                    )
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_interact",
        description = "Run a shell command in a PTY session and return its output. Injects the command, waits for completion via a unique marker, and returns clean output with exit code. This is the preferred tool for running commands in terminal sessions — it handles injection, waiting, and output capture atomically."
    )]
    async fn termlink_interact(&self, Parameters(p): Parameters<InteractParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let timeout_secs = p.timeout.unwrap_or(30);
        let poll_ms = p.poll_ms.unwrap_or(200);

        // Unique marker for this invocation
        let marker = format!(
            "___TERMLINK_DONE_{:x}_{:x}___",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        );

        // Snapshot scrollback before injection
        let pre_resp = match client::rpc_call(
            reg.socket_path(),
            "query.output",
            serde_json::json!({ "bytes": 131072, "strip_ansi": true }),
        ).await {
            Ok(r) => r,
            Err(e) => return json_err(format!("not a PTY session or connection failed: {e}")),
        };

        let pre_output = match client::unwrap_result(pre_resp) {
            Ok(r) => r["output"].as_str().unwrap_or("").to_string(),
            Err(e) => return json_err(format!("session has no PTY: {e}")),
        };
        let pre_len = pre_output.len();

        // Inject command + marker on one line
        let inject_line = format!("{}; echo \"{marker} exit=$?\"", p.command);
        let keys = serde_json::json!([
            { "type": "text", "value": inject_line },
            { "type": "key", "value": "Enter" }
        ]);
        if let Err(e) = client::rpc_call(
            reg.socket_path(),
            "command.inject",
            serde_json::json!({ "keys": keys }),
        ).await {
            return json_err(format!("failed to inject command: {e}"));
        }

        // Poll until marker appears
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let poll_interval = tokio::time::Duration::from_millis(poll_ms);

        loop {
            if tokio::time::Instant::now() >= deadline {
                return json_err(format!("timeout after {}s waiting for command to complete", timeout_secs));
            }

            tokio::time::sleep(poll_interval).await;

            let resp = match client::rpc_call(
                reg.socket_path(),
                "query.output",
                serde_json::json!({ "bytes": 131072, "strip_ansi": true }),
            ).await {
                Ok(r) => r,
                Err(e) => return json_err(format!("connection lost: {e}")),
            };

            let full_output = match client::unwrap_result(resp) {
                Ok(r) => r["output"].as_str().unwrap_or("").to_string(),
                Err(e) => return json_err(format!("poll failed: {e}")),
            };

            // Diff against pre-injection snapshot
            let output = if full_output.len() > pre_len {
                &full_output[pre_len..]
            } else {
                &full_output
            };

            let marker_with_exit = format!("{marker} exit=");
            let has_marker = output.contains(&marker_with_exit) && {
                output.lines().any(|line| {
                    if let Some(pos) = line.find(&marker_with_exit) {
                        let after = &line[pos + marker_with_exit.len()..];
                        after.starts_with(|c: char| c.is_ascii_digit())
                    } else {
                        false
                    }
                })
            };

            if has_marker {
                // Extract exit code
                let mut exit_code: Option<i32> = None;
                for line in output.lines() {
                    if line.contains(&marker)
                        && let Some(exit_str) = line.split("exit=").nth(1) {
                            exit_code = exit_str.trim().parse().ok();
                        }
                }

                // Clean output: skip the command echo line, stop before marker line
                let clean_output = {
                    let after_cmd_echo = output.find('\n')
                        .map(|pos| &output[pos + 1..])
                        .unwrap_or(output);

                    if let Some(pos) = after_cmd_echo.find(&marker_with_exit) {
                        let before = &after_cmd_echo[..pos];
                        before.rfind('\n')
                            .map(|nl| &after_cmd_echo[..nl])
                            .unwrap_or("")
                    } else {
                        after_cmd_echo
                    }
                };

                let trimmed = clean_output.trim();
                let exit = exit_code.unwrap_or(-1);

                let response = serde_json::json!({
                    "ok": exit == 0,
                    "exit_code": exit,
                    "output": trimmed,
                    "target": p.target,
                    "command": p.command,
                });
                return serde_json::to_string_pretty(&response)
                    .unwrap_or_else(json_err);
            }
        }
    }

    #[tool(
        name = "termlink_doctor",
        description = "Run health checks on the TermLink environment. Returns a structured JSON report with pass/warn/fail status for: runtime directory, sessions directory, session liveness, hub status, orphaned sockets, dispatch manifest, and version. Use this to diagnose connectivity or infrastructure issues before attempting operations."
    )]
    async fn termlink_doctor(&self) -> String {
        use termlink_session::{discovery, liveness};

        let mut checks: Vec<serde_json::Value> = Vec::new();
        let mut pass_count = 0u32;
        let mut warn_count = 0u32;
        let mut fail_count = 0u32;

        macro_rules! check {
            ($name:expr, pass, $msg:expr) => {{
                pass_count += 1;
                checks.push(serde_json::json!({"check": $name, "status": "pass", "message": $msg}));
            }};
            ($name:expr, warn, $msg:expr) => {{
                warn_count += 1;
                checks.push(serde_json::json!({"check": $name, "status": "warn", "message": $msg}));
            }};
            ($name:expr, fail, $msg:expr) => {{
                fail_count += 1;
                checks.push(serde_json::json!({"check": $name, "status": "fail", "message": $msg}));
            }};
        }

        // 1. Runtime directory
        let runtime_dir = discovery::runtime_dir();
        if runtime_dir.exists() {
            check!("runtime_dir", pass, format!("{}", runtime_dir.display()));
        } else {
            check!("runtime_dir", fail, format!("{} does not exist", runtime_dir.display()));
        }

        // 2. Sessions directory
        let sessions_dir = discovery::sessions_dir();
        if sessions_dir.exists() {
            check!("sessions_dir", pass, format!("{}", sessions_dir.display()));
        } else {
            check!("sessions_dir", warn, format!("{} does not exist (no sessions yet)", sessions_dir.display()));
        }

        // 3. Session health
        let sessions = manager::list_sessions(true).unwrap_or_default();
        let total = sessions.len();
        let mut alive = 0u32;
        let mut dead = 0u32;
        let mut stale_names: Vec<String> = Vec::new();

        for s in &sessions {
            if liveness::process_exists(s.pid) {
                match client::rpc_call(s.socket_path(), "termlink.ping", serde_json::json!({})).await {
                    Ok(_) => alive += 1,
                    Err(_) => {
                        dead += 1;
                        stale_names.push(s.display_name.clone());
                    }
                }
            } else {
                dead += 1;
                stale_names.push(s.display_name.clone());
            }
        }

        if total == 0 {
            check!("sessions", pass, "no sessions registered");
        } else if dead == 0 {
            check!("sessions", pass, format!("{total} registered, all responding"));
        } else {
            check!("sessions", warn, format!("{total} registered, {alive} alive, {dead} dead/stale: {}", stale_names.join(", ")));
        }

        // 4. Hub status
        let hub_socket = termlink_hub::server::hub_socket_path();
        let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
        match termlink_hub::pidfile::check(&pidfile_path) {
            termlink_hub::pidfile::PidfileStatus::Running(pid) => {
                match client::rpc_call(&hub_socket, "termlink.ping", serde_json::json!({})).await {
                    Ok(_) => check!("hub", pass, format!("running (PID {pid}), responding")),
                    Err(_) => check!("hub", warn, format!("running (PID {pid}), but not responding")),
                }
            }
            termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
                check!("hub", warn, format!("stale pidfile (PID {pid} is dead)"));
            }
            termlink_hub::pidfile::PidfileStatus::NotRunning => {
                check!("hub", pass, "not running");
            }
        }

        // 5. Orphaned sockets
        if sessions_dir.exists() {
            let mut orphan_count = 0u32;
            if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension()
                        && ext == "sock" {
                            let json_path = path.with_extension("json");
                            if !json_path.exists() {
                                orphan_count += 1;
                            }
                        }
                }
            }
            if orphan_count > 0 {
                check!("sockets", warn, format!("{orphan_count} orphaned socket(s)"));
            } else {
                check!("sockets", pass, "no orphaned sockets");
            }
        }

        // 6. Dispatch manifest
        {
            let project_root = std::env::current_dir().unwrap_or_default();
            let manifest_path = project_root.join(".termlink").join("dispatch-manifest.json");
            if manifest_path.exists() {
                match std::fs::read_to_string(&manifest_path) {
                    Ok(content) => {
                        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                            let pending = manifest["dispatches"]
                                .as_array()
                                .map(|arr| {
                                    arr.iter()
                                        .filter(|d| d["status"].as_str() == Some("pending"))
                                        .count()
                                })
                                .unwrap_or(0);
                            let total = manifest["dispatches"]
                                .as_array()
                                .map(|a| a.len())
                                .unwrap_or(0);
                            if pending > 0 {
                                check!("dispatch", warn, format!("{pending} pending dispatch(es) of {total} total"));
                            } else {
                                check!("dispatch", pass, format!("{total} dispatch(es), none pending"));
                            }
                        } else {
                            check!("dispatch", warn, "dispatch manifest exists but failed to parse");
                        }
                    }
                    Err(e) => {
                        check!("dispatch", warn, format!("failed to read dispatch manifest: {e}"));
                    }
                }
            } else {
                check!("dispatch", pass, "no dispatch manifest");
            }
        }

        // 7. Version
        let version = env!("CARGO_PKG_VERSION");
        check!("version", pass, format!("termlink-mcp {version}"));

        let result = serde_json::json!({
            "checks": checks,
            "summary": {
                "pass": pass_count,
                "warn": warn_count,
                "fail": fail_count,
            }
        });
        serde_json::to_string_pretty(&result).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_overview",
        description = "Get a single-call overview of the TermLink workspace: active sessions, hub status, runtime directory, and version. Use this as a first call to understand the current environment before performing operations."
    )]
    async fn termlink_overview(&self) -> String {
        use termlink_session::{discovery, liveness};

        let runtime_dir = discovery::runtime_dir();
        let sessions_dir = discovery::sessions_dir();

        // Enumerate sessions
        let sessions: Vec<serde_json::Value> = manager::list_sessions(false)
            .unwrap_or_default()
            .into_iter()
            .map(|reg| {
                let alive = liveness::process_exists(reg.pid);
                let age = format_age(&reg.created_at);
                serde_json::json!({
                    "id": reg.id.as_str(),
                    "name": reg.display_name,
                    "state": reg.state.to_string(),
                    "alive": alive,
                    "pid": reg.pid,
                    "age": age,
                    "tags": reg.tags,
                    "roles": reg.roles,
                })
            })
            .collect();

        let session_count = sessions.len();

        // Hub status
        let hub_socket = termlink_hub::server::hub_socket_path();
        let pidfile = termlink_hub::pidfile::hub_pidfile_path();
        let hub_running = matches!(
            termlink_hub::pidfile::check(&pidfile),
            termlink_hub::pidfile::PidfileStatus::Running(_)
        );

        let version = env!("CARGO_PKG_VERSION");
        let mcp_tools = crate::tool_count();

        let response = serde_json::json!({
            "ok": true,
            "session_count": session_count,
            "sessions": sessions,
            "hub_running": hub_running,
            "hub_socket": hub_socket.display().to_string(),
            "runtime_dir": runtime_dir.display().to_string(),
            "sessions_dir": sessions_dir.display().to_string(),
            "version": version,
            "mcp_tools": mcp_tools,
        });
        serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_clean",
        description = "Remove stale TermLink sessions (dead processes) and orphaned sockets. Returns a report of what was cleaned. Use this to recover from crashed sessions or fix issues found by termlink_doctor."
    )]
    async fn termlink_clean(&self) -> String {
        use termlink_session::discovery;

        let sessions_dir = discovery::sessions_dir();
        let mut cleaned_sessions: Vec<String> = Vec::new();
        let mut cleaned_sockets = 0u32;
        let mut cleaned_hub = false;

        // 1. Clean stale sessions
        match manager::clean_stale_sessions(&sessions_dir, true) {
            Ok(stale) => {
                for s in &stale {
                    cleaned_sessions.push(s.display_name.clone());
                }
            }
            Err(e) => {
                return json_err(format!("scanning sessions: {e}"));
            }
        }

        // 2. Clean orphaned sockets
        if sessions_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&sessions_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension()
                        && ext == "sock" {
                            let json_path = path.with_extension("json");
                            if !json_path.exists() {
                                let _ = std::fs::remove_file(&path);
                                let data_path = path.with_extension("sock.data");
                                let _ = std::fs::remove_file(&data_path);
                                cleaned_sockets += 1;
                            }
                        }
                }
            }

        // 3. Clean stale hub pidfile
        let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
        if let termlink_hub::pidfile::PidfileStatus::Stale(_pid) = termlink_hub::pidfile::check(&pidfile_path) {
            termlink_hub::pidfile::remove(&pidfile_path);
            let hub_socket = termlink_hub::server::hub_socket_path();
            let _ = std::fs::remove_file(&hub_socket);
            cleaned_hub = true;
        }

        let result = serde_json::json!({
            "cleaned_sessions": cleaned_sessions,
            "cleaned_sockets": cleaned_sockets,
            "cleaned_hub": cleaned_hub,
            "total": cleaned_sessions.len() as u32 + cleaned_sockets + if cleaned_hub { 1 } else { 0 },
        });
        serde_json::to_string_pretty(&result).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_resize",
        description = "Resize a PTY-backed TermLink session's terminal dimensions. Useful when you need specific column width for parsing command output or formatting."
    )]
    async fn termlink_resize(&self, Parameters(p): Parameters<ResizeParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        match client::rpc_call(
            reg.socket_path(),
            "command.resize",
            serde_json::json!({ "cols": p.cols, "rows": p.rows }),
        ).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => format!(
                    "Resized to {}x{}",
                    result["cols"].as_u64().unwrap_or(p.cols as u64),
                    result["rows"].as_u64().unwrap_or(p.rows as u64),
                ),
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_request",
        description = "Send a request event to a TermLink session and wait for a reply. Emits an event with a unique request_id on the specified topic, then polls for a reply event on reply_topic with matching request_id. Use this for request-reply coordination between sessions (e.g., send 'task.run', wait for 'task.result')."
    )]
    async fn termlink_request(&self, Parameters(p): Parameters<RequestParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let timeout_secs = p.timeout.unwrap_or(30);

        // Generate unique request ID
        let request_id = format!(
            "req-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        // Build payload with request_id
        let mut payload = p.payload.unwrap_or(serde_json::json!({}));
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("request_id".to_string(), serde_json::json!(&request_id));
        }

        // Snapshot cursor before emitting (quick subscribe for next_seq)
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

        // Emit the request event
        let emit_params = serde_json::json!({
            "topic": p.topic,
            "payload": payload,
        });

        match client::rpc_call(reg.socket_path(), "event.emit", emit_params).await {
            Ok(resp) => {
                if let Err(e) = client::unwrap_result(resp) {
                    return json_err(format!("failed to emit request: {e}"));
                }
            }
            Err(e) => return json_err(format!("connection failed: {e}")),
        }

        // Subscribe for reply (server-side blocking, no sleep needed)
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let subscribe_timeout: u64 = 5000; // 5s per subscribe call
        let mut sub_cursor = cursor;

        loop {
            let remaining = deadline.duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return format!(
                    "Timeout: no reply on '{}' within {}s (request_id: {})",
                    p.reply_topic, timeout_secs, request_id
                );
            }

            let effective_timeout = subscribe_timeout.min(remaining.as_millis() as u64);
            let mut params = serde_json::json!({
                "topic": p.reply_topic,
                "timeout_ms": effective_timeout,
            });
            if let Some(c) = sub_cursor {
                params["since"] = serde_json::json!(c);
            }

            match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp)
                        && let Some(events) = result["events"].as_array() {
                            for event in events {
                                let event_payload = &event["payload"];
                                let matches = event_payload
                                    .get("request_id")
                                    .and_then(|r| r.as_str())
                                    .map(|r| r == request_id)
                                    .unwrap_or(true);

                                if matches {
                                    return serde_json::to_string_pretty(event_payload)
                                        .unwrap_or_else(|_| "Reply received".into());
                                }
                            }

                            if let Some(next) = result["next_seq"].as_u64() {
                                sub_cursor = Some(next.saturating_sub(1));
                            }
                        }
                }
                Err(e) => return json_err(format!("connection lost: {e}")),
            }
        }
    }

    #[tool(
        name = "termlink_tag",
        description = "Update tags, name, or roles on a TermLink session. Use 'add'/'remove' for tags, 'name' to rename, 'roles'/'add_roles'/'remove_roles' for roles. Returns the updated state. Tags and roles enable discovery-based orchestration."
    )]
    async fn termlink_tag(&self, Parameters(p): Parameters<TagParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let mut params = serde_json::json!({});
        if let Some(set) = &p.set {
            params["tags"] = serde_json::json!(set);
        }
        if let Some(add) = &p.add {
            params["add_tags"] = serde_json::json!(add);
        }
        if let Some(remove) = &p.remove {
            params["remove_tags"] = serde_json::json!(remove);
        }
        if let Some(name) = &p.name {
            params["display_name"] = serde_json::json!(name);
        }
        if let Some(roles) = &p.roles {
            params["roles"] = serde_json::json!(roles);
        }
        if let Some(add_roles) = &p.add_roles {
            params["add_roles"] = serde_json::json!(add_roles);
        }
        if let Some(remove_roles) = &p.remove_roles {
            params["remove_roles"] = serde_json::json!(remove_roles);
        }

        if params.as_object().is_some_and(|o| o.is_empty()) {
            return json_err("specify at least one of: set, add, remove, name, roles, add_roles, or remove_roles");
        }

        match client::rpc_call(reg.socket_path(), "session.update", params).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => {
                    let tags = result["tags"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|t| t.as_str())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let roles = result["roles"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|r| r.as_str())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let name = result["display_name"].as_str().unwrap_or(&p.target);
                    let mut parts = vec![format!("tags=[{}]", tags.join(", "))];
                    if !roles.is_empty() {
                        parts.push(format!("roles=[{}]", roles.join(", ")));
                    }
                    format!("Updated {}: {}", name, parts.join(", "))
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_wait",
        description = "Wait for a specific event topic to appear on a session's event bus. Blocks until the event arrives or timeout."
    )]
    async fn termlink_wait(&self, Parameters(p): Parameters<WaitParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let timeout_secs = p.timeout.unwrap_or(30);
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let subscribe_timeout: u64 = 5000; // 5s per subscribe call

        // Check for already-existing events via poll (catches seq 0+), then subscribe for live
        let mut cursor: Option<u64> = {
            let params = serde_json::json!({"topic": p.topic});
            match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        // Check if matching event already exists
                        if let Some(events) = result["events"].as_array()
                            && let Some(event) = events.first() {
                                let payload = &event["payload"];
                                let text = if payload.is_null()
                                    || payload.as_object().is_some_and(|o| o.is_empty())
                                {
                                    format!("Event received: {}", p.topic)
                                } else {
                                    serde_json::to_string_pretty(payload)
                                        .unwrap_or_else(|_| format!("Event received: {}", p.topic))
                                };
                                return text;
                            }
                        result["next_seq"].as_u64().map(|n| n.saturating_sub(1))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        };

        loop {
            let remaining = deadline.duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return format!("Timeout waiting for event topic '{}' ({}s)", p.topic, timeout_secs);
            }

            let effective_timeout = subscribe_timeout.min(remaining.as_millis() as u64);
            let mut params = serde_json::json!({
                "topic": p.topic,
                "timeout_ms": effective_timeout,
            });
            if let Some(c) = cursor {
                params["since"] = serde_json::json!(c);
            }

            match client::rpc_call(reg.socket_path(), "event.subscribe", params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        if let Some(events) = result["events"].as_array()
                            && let Some(event) = events.first() {
                                let payload = &event["payload"];
                                return if payload.is_null()
                                    || payload.as_object().is_some_and(|o| o.is_empty())
                                {
                                    format!("Event received: {}", p.topic)
                                } else {
                                    serde_json::to_string_pretty(payload)
                                        .unwrap_or_else(|_| format!("Event received: {}", p.topic))
                                };
                            }
                        if let Some(next) = result["next_seq"].as_u64() {
                            cursor = Some(next.saturating_sub(1));
                        }
                    }
                }
                Err(e) => return json_err(format!("connection lost: {e}")),
            }
        }
    }

    #[tool(
        name = "termlink_dispatch_status",
        description = "Read the dispatch manifest and report branch lifecycle status. Returns counts of pending/merged/conflict/deferred/expired dispatches and details of any pending dispatches with their branches. Use this to check if dispatched workers have been merged or if there are conflicts to resolve."
    )]
    async fn termlink_dispatch_status(&self) -> String {
        let project_root = match std::env::current_dir() {
            Ok(d) => d,
            Err(e) => return format!("{{\"ok\":false,\"error\":\"Failed to get current directory: {e}\"}}"),
        };
        let manifest_path = project_root.join(".termlink").join("dispatch-manifest.json");

        if !manifest_path.exists() {
            return serde_json::json!({
                "ok": true,
                "total": 0,
                "message": "No dispatch manifest (no dispatches have used --isolate yet)"
            }).to_string();
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => return format!("{{\"ok\":false,\"error\":\"Failed to read dispatch manifest: {e}\"}}"),
        };

        let manifest: serde_json::Value = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => return format!("{{\"ok\":false,\"error\":\"Failed to parse dispatch manifest: {e}\"}}"),
        };

        let dispatches = manifest["dispatches"].as_array();
        let total = dispatches.map(|a| a.len()).unwrap_or(0);

        let count_status = |status: &str| -> usize {
            dispatches
                .map(|arr| arr.iter().filter(|d| d["status"].as_str() == Some(status)).count())
                .unwrap_or(0)
        };

        let pending = count_status("pending");
        let merged = count_status("merged");
        let conflict = count_status("conflict");
        let deferred = count_status("deferred");
        let expired = count_status("expired");

        let pending_details: Vec<serde_json::Value> = dispatches
            .map(|arr| {
                arr.iter()
                    .filter(|d| d["status"].as_str() == Some("pending"))
                    .map(|d| {
                        let branches_with_commits: Vec<&str> = d["branches"]
                            .as_array()
                            .map(|b| {
                                b.iter()
                                    .filter(|br| br["has_commits"].as_bool() == Some(true))
                                    .filter_map(|br| br["branch_name"].as_str())
                                    .collect()
                            })
                            .unwrap_or_default();
                        serde_json::json!({
                            "id": d["id"],
                            "created_at": d["created_at"],
                            "worker_count": d["worker_count"],
                            "branches_with_commits": branches_with_commits,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let result = serde_json::json!({
            "ok": pending == 0,
            "total": total,
            "pending": pending,
            "merged": merged,
            "conflict": conflict,
            "deferred": deferred,
            "expired": expired,
            "pending_dispatches": pending_details,
        });
        serde_json::to_string_pretty(&result).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_info",
        description = "Get TermLink runtime information: version, commit hash, build target, runtime directory paths, hub status, and session counts (live/stale/total). Use this for diagnostics and to understand the current TermLink environment state."
    )]
    async fn termlink_info(&self) -> String {
        let runtime_dir = termlink_session::discovery::runtime_dir();
        let sessions_dir = termlink_session::discovery::sessions_dir();
        let hub_socket = termlink_hub::server::hub_socket_path();
        let hub_running = hub_socket.exists();
        let live = manager::list_sessions(false)
            .map(|s| s.len())
            .unwrap_or(0);
        let all = manager::list_sessions(true)
            .map(|s| s.len())
            .unwrap_or(0);
        let stale = all - live;

        let version = env!("CARGO_PKG_VERSION");
        let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
        let target = option_env!("BUILD_TARGET").unwrap_or("unknown");

        let result = serde_json::json!({
            "ok": true,
            "version": version,
            "commit": commit,
            "target": target,
            "runtime_dir": runtime_dir.to_string_lossy(),
            "sessions_dir": sessions_dir.to_string_lossy(),
            "hub_socket": hub_socket.to_string_lossy(),
            "hub_running": hub_running,
            "sessions": {
                "live": live,
                "stale": stale,
                "total": all,
            },
        });
        serde_json::to_string_pretty(&result).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_topics",
        description = "List event topics across all sessions (or a specific session). Returns a map of session names to their active event topics, plus a total count. Use this to discover what events sessions are emitting before subscribing or polling."
    )]
    async fn termlink_topics(&self, Parameters(p): Parameters<TopicsParams>) -> String {
        let registrations = if let Some(ref target) = p.target {
            match manager::find_session(target) {
                Ok(r) => vec![r],
                Err(e) => return json_err(format!("session '{}' not found: {e}", target)),
            }
        } else {
            manager::list_sessions(false).unwrap_or_default()
        };

        if registrations.is_empty() {
            return serde_json::json!({
                "ok": true,
                "sessions": {},
                "total_topics": 0,
            }).to_string();
        }

        let timeout = std::time::Duration::from_secs(5);
        let mut session_topics: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();

        for reg in &registrations {
            let rpc_future = client::rpc_call(reg.socket_path(), "event.topics", serde_json::json!({}));
            if let Ok(Ok(resp)) = tokio::time::timeout(timeout, rpc_future).await
                && let Ok(result) = client::unwrap_result(resp)
                && let Some(topics) = result["topics"].as_array()
            {
                let topic_list: Vec<String> = topics
                    .iter()
                    .filter_map(|t| t.as_str().map(String::from))
                    .collect();
                if !topic_list.is_empty() {
                    session_topics.insert(reg.display_name.clone(), topic_list);
                }
            }
        }

        let total: usize = session_topics.values().map(|v| v.len()).sum();

        let result = serde_json::json!({
            "ok": true,
            "sessions": session_topics,
            "total_topics": total,
        });
        serde_json::to_string_pretty(&result).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_collect",
        description = "Collect events from multiple sessions via the hub (fan-in). Requires the hub to be running. Returns events from targeted sessions with cursors for continuation polling. Use this to gather results from dispatched workers or monitor events across a fleet of sessions."
    )]
    async fn termlink_collect(&self, Parameters(p): Parameters<CollectParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return serde_json::json!({
                "ok": false,
                "error": "Hub is not running. Start it with: termlink hub start"
            }).to_string();
        }

        let timeout_ms = p.timeout_ms.unwrap_or(5000);
        let mut params = serde_json::json!({
            "timeout_ms": timeout_ms,
        });
        if let Some(ref targets) = p.targets {
            params["targets"] = serde_json::json!(targets);
        }
        if let Some(ref topic) = p.topic {
            params["topic"] = serde_json::json!(topic);
        }
        if let Some(ref since) = p.since {
            params["since"] = since.clone();
        }

        let rpc_timeout = std::time::Duration::from_millis(timeout_ms + 5000);
        match tokio::time::timeout(rpc_timeout, client::rpc_call(&hub_socket, "event.collect", params)).await {
            Ok(Ok(resp)) => {
                match client::unwrap_result(resp) {
                    Ok(result) => {
                        let events = result["events"].as_array().map(|arr| {
                            arr.iter().map(|e| {
                                serde_json::json!({
                                    "session_name": e["session_name"],
                                    "topic": e["topic"],
                                    "payload": e["payload"],
                                    "seq": e["seq"],
                                    "timestamp": e["timestamp"],
                                })
                            }).collect::<Vec<_>>()
                        }).unwrap_or_default();

                        let response = serde_json::json!({
                            "ok": true,
                            "events": events,
                            "count": events.len(),
                            "cursors": result.get("cursors").cloned().unwrap_or(serde_json::json!({})),
                        });
                        serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
                    }
                    Err(e) => serde_json::json!({
                        "ok": false,
                        "error": format!("Hub returned error: {e}"),
                    }).to_string(),
                }
            }
            Ok(Err(e)) => serde_json::json!({
                "ok": false,
                "error": format!("Failed to connect to hub: {e}"),
            }).to_string(),
            Err(_) => serde_json::json!({
                "ok": false,
                "error": format!("Timeout after {}ms", timeout_ms + 5000),
            }).to_string(),
        }
    }

    #[tool(
        name = "termlink_pty_mode",
        description = "Query the terminal mode of a PTY session. Returns whether the terminal is in canonical, echo, raw, or alternate screen mode. Use this to determine how to interact with a session — e.g., raw mode means an interactive program is running, alternate screen suggests a TUI app like vim or less."
    )]
    async fn termlink_pty_mode(&self, Parameters(p): Parameters<PtyModeParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let timeout = std::time::Duration::from_secs(5);
        match tokio::time::timeout(timeout, client::rpc_call(reg.socket_path(), "pty.mode", serde_json::json!({}))).await {
            Ok(Ok(resp)) => {
                match client::unwrap_result(resp) {
                    Ok(result) => {
                        let response = serde_json::json!({
                            "ok": true,
                            "session": p.target,
                            "canonical": result["canonical"],
                            "echo": result["echo"],
                            "raw": result["raw"],
                            "alternate_screen": result["alternate_screen"],
                        });
                        serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
                    }
                    Err(e) => json_err(e),
                }
            }
            Ok(Err(e)) => json_err(format!("failed to connect to session '{}': {e}", p.target)),
            Err(_) => json_err(format!("timeout querying pty mode for '{}'", p.target)),
        }
    }

    #[tool(
        name = "termlink_hub_status",
        description = "Check the hub lifecycle state (running, not_running, stale). Use this before calling hub-dependent tools like collect or broadcast to verify the hub is available."
    )]
    async fn termlink_hub_status(&self) -> String {
        let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
        let socket_path = termlink_hub::server::hub_socket_path();

        let response = match termlink_hub::pidfile::check(&pidfile_path) {
            termlink_hub::pidfile::PidfileStatus::NotRunning => {
                serde_json::json!({
                    "ok": true,
                    "status": "not_running",
                })
            }
            termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
                serde_json::json!({
                    "ok": true,
                    "status": "stale",
                    "pid": pid,
                    "pidfile": pidfile_path.display().to_string(),
                })
            }
            termlink_hub::pidfile::PidfileStatus::Running(pid) => {
                serde_json::json!({
                    "ok": true,
                    "status": "running",
                    "pid": pid,
                    "socket": socket_path.display().to_string(),
                    "pidfile": pidfile_path.display().to_string(),
                })
            }
        };

        serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_file_send",
        description = "Send a file to a target session via the chunked file transfer protocol (file.init + file.chunk + file.complete events). The receiving session must be listening for file events. Returns transfer_id, SHA256, bytes sent, and chunk count."
    )]
    async fn termlink_file_send(&self, Parameters(p): Parameters<FileSendParams>) -> String {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        use termlink_protocol::events::{file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION};

        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let file_path = std::path::Path::new(&p.path);
        let file_data = match std::fs::read(file_path) {
            Ok(d) => d,
            Err(e) => return json_err(format!("failed to read file '{}': {e}", p.path)),
        };

        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        let size = file_data.len() as u64;
        let chunk_size: usize = 49152; // 48KB chunks
        let total_chunks = file_data.len().div_ceil(chunk_size) as u32;

        let transfer_id = format!("xfer-mcp-{}", std::process::id());

        let mut hasher = Sha256::new();
        hasher.update(&file_data);
        let sha256 = format!("{:x}", hasher.finalize());

        let timeout = std::time::Duration::from_secs(30);

        // Phase 1: file.init
        let init = FileInit {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            filename: filename.clone(),
            size,
            total_chunks,
            from: format!("mcp-{}", std::process::id()),
        };
        let init_payload = match serde_json::to_value(&init) {
            Ok(v) => v,
            Err(e) => return json_err(format!("failed to serialize file.init: {e}")),
        };
        let emit = serde_json::json!({"topic": file_topic::INIT, "payload": init_payload});
        if let Err(e) = tokio::time::timeout(timeout, client::rpc_call(reg.socket_path(), "event.emit", emit)).await
            .map_err(|_| "timeout".to_string())
            .and_then(|r| r.map_err(|e| e.to_string()))
        {
            return json_err(format!("file.init failed: {e}"));
        }

        // Phase 2: file.chunk(s)
        let encoder = base64::engine::general_purpose::STANDARD;
        for (i, chunk_data) in file_data.chunks(chunk_size).enumerate() {
            let chunk = FileChunk {
                schema_version: SCHEMA_VERSION.to_string(),
                transfer_id: transfer_id.clone(),
                index: i as u32,
                data: encoder.encode(chunk_data),
            };
            let chunk_payload = match serde_json::to_value(&chunk) {
                Ok(v) => v,
                Err(e) => return json_err(format!("failed to serialize chunk {i}: {e}")),
            };
            let emit = serde_json::json!({"topic": file_topic::CHUNK, "payload": chunk_payload});
            if let Err(e) = tokio::time::timeout(timeout, client::rpc_call(reg.socket_path(), "event.emit", emit)).await
                .map_err(|_| "timeout".to_string())
                .and_then(|r| r.map_err(|e| e.to_string()))
            {
                return json_err(format!("chunk {}/{total_chunks} failed: {e}", i + 1));
            }
        }

        // Phase 3: file.complete
        let complete = FileComplete {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            sha256: sha256.clone(),
        };
        let complete_payload = match serde_json::to_value(&complete) {
            Ok(v) => v,
            Err(e) => return json_err(format!("failed to serialize file.complete: {e}")),
        };
        let emit = serde_json::json!({"topic": file_topic::COMPLETE, "payload": complete_payload});
        if let Err(e) = tokio::time::timeout(timeout, client::rpc_call(reg.socket_path(), "event.emit", emit)).await
            .map_err(|_| "timeout".to_string())
            .and_then(|r| r.map_err(|e| e.to_string()))
        {
            return json_err(format!("file.complete failed: {e}"));
        }

        let response = serde_json::json!({
            "ok": true,
            "target": p.target,
            "filename": filename,
            "size": size,
            "chunks": total_chunks,
            "transfer_id": transfer_id,
            "sha256": sha256,
        });
        serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_file_receive",
        description = "Receive the most recent file from a target session's event stream. Polls the session's events for a completed file transfer (file.init + file.chunk + file.complete), reassembles the chunks, verifies SHA-256 integrity, and writes the file to the specified output directory."
    )]
    async fn termlink_file_receive(&self, Parameters(p): Parameters<FileReceiveParams>) -> String {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        use termlink_protocol::events::{file_topic, FileInit, FileChunk, FileComplete};

        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let out_path = std::path::Path::new(&p.output_dir);
        if !out_path.is_dir() {
            return json_err(format!("output directory '{}' does not exist or is not a directory", p.output_dir));
        }

        // Poll all events from the session
        let timeout = std::time::Duration::from_secs(10);
        let poll_result = match tokio::time::timeout(
            timeout,
            client::rpc_call(reg.socket_path(), "event.poll", serde_json::json!({})),
        ).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return json_err(format!("failed to poll events: {e}")),
            Err(_) => return json_err("event poll timed out after 10s"),
        };

        let result = match client::unwrap_result(poll_result) {
            Ok(r) => r,
            Err(e) => return json_err(format!("event poll failed: {e}")),
        };

        let events = match result["events"].as_array() {
            Some(arr) => arr,
            None => return json_err("no events array in poll response"),
        };

        // Find the last complete file transfer: scan for the last file.init
        let mut last_init: Option<FileInit> = None;
        for event in events.iter() {
            let topic = event["topic"].as_str().unwrap_or("");
            if topic == file_topic::INIT
                && let Ok(init) = serde_json::from_value::<FileInit>(event["payload"].clone())
            {
                last_init = Some(init);
            }
        }

        let init = match last_init {
            Some(i) => i,
            None => return json_err("no file transfer found in session events"),
        };

        // Collect chunks matching this transfer_id
        let decoder = base64::engine::general_purpose::STANDARD;
        let mut chunks: std::collections::BTreeMap<u32, Vec<u8>> = std::collections::BTreeMap::new();

        for event in events.iter() {
            let topic = event["topic"].as_str().unwrap_or("");
            if topic == file_topic::CHUNK
                && let Ok(chunk) = serde_json::from_value::<FileChunk>(event["payload"].clone())
                && chunk.transfer_id == init.transfer_id
            {
                match decoder.decode(&chunk.data) {
                    Ok(data) => { chunks.insert(chunk.index, data); }
                    Err(e) => return json_err(format!("invalid base64 in chunk {}: {e}", chunk.index)),
                }
            }
        }

        if chunks.len() as u32 != init.total_chunks {
            return json_err(format!(
                "incomplete transfer — got {}/{} chunks for transfer {}",
                chunks.len(), init.total_chunks, init.transfer_id
            ));
        }

        // Find the file.complete event for SHA-256 verification
        let mut expected_sha256: Option<String> = None;
        for event in events.iter() {
            let topic = event["topic"].as_str().unwrap_or("");
            if topic == file_topic::COMPLETE
                && let Ok(complete) = serde_json::from_value::<FileComplete>(event["payload"].clone())
                && complete.transfer_id == init.transfer_id
            {
                expected_sha256 = Some(complete.sha256);
            }
        }

        let expected_sha256 = match expected_sha256 {
            Some(s) => s,
            None => return json_err(format!("no file.complete event for transfer {}", init.transfer_id)),
        };

        // Reassemble file data
        let mut file_data = Vec::new();
        for i in 0..init.total_chunks {
            match chunks.get(&i) {
                Some(data) => file_data.extend_from_slice(data),
                None => return json_err(format!("missing chunk {}/{}", i, init.total_chunks)),
            }
        }

        // Verify SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&file_data);
        let actual_sha256 = format!("{:x}", hasher.finalize());

        if actual_sha256 != expected_sha256 {
            return json_err(format!(
                "SHA-256 mismatch — expected {expected_sha256}, got {actual_sha256}"
            ));
        }

        // Write file
        let dest = out_path.join(&init.filename);
        if let Err(e) = std::fs::write(&dest, &file_data) {
            return json_err(format!("failed to write file '{}': {e}", dest.display()));
        }

        let response = serde_json::json!({
            "ok": true,
            "target": p.target,
            "filename": init.filename,
            "path": dest.display().to_string(),
            "size": file_data.len(),
            "sha256": actual_sha256,
            "transfer_id": init.transfer_id,
        });
        serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_hub_start",
        description = "Start the hub server in the background. The hub enables multi-session features like collect, broadcast, and discover. Returns immediately with hub pid and socket path. No-op if hub is already running."
    )]
    async fn termlink_hub_start(&self) -> String {
        let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
        let socket_path = termlink_hub::server::hub_socket_path();

        // Check if already running
        if let termlink_hub::pidfile::PidfileStatus::Running(pid) = termlink_hub::pidfile::check(&pidfile_path) {
            let response = serde_json::json!({
                "ok": true,
                "action": "already_running",
                "pid": pid,
                "socket": socket_path.display().to_string(),
            });
            return serde_json::to_string_pretty(&response).unwrap_or_else(json_err);
        }

        match termlink_hub::server::run(&socket_path).await {
            Ok(_handle) => {
                // Leak the handle so the hub stays alive for the MCP server lifetime
                std::mem::forget(_handle);
                // Give the hub a moment to write pidfile
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let pid = match termlink_hub::pidfile::check(&pidfile_path) {
                    termlink_hub::pidfile::PidfileStatus::Running(p) => Some(p),
                    _ => None,
                };
                let response = serde_json::json!({
                    "ok": true,
                    "action": "started",
                    "pid": pid,
                    "socket": socket_path.display().to_string(),
                });
                serde_json::to_string_pretty(&response).unwrap_or_else(json_err)
            }
            Err(e) => json_err(format!("failed to start hub: {e}")),
        }
    }

    #[tool(
        name = "termlink_hub_stop",
        description = "Stop the running hub server. Sends SIGTERM and waits up to 2 seconds for clean shutdown. Cleans up stale pidfiles if the hub process is already dead."
    )]
    async fn termlink_hub_stop(&self) -> String {
        let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

        match termlink_hub::pidfile::check(&pidfile_path) {
            termlink_hub::pidfile::PidfileStatus::NotRunning => {
                serde_json::json!({"ok": true, "action": "none", "reason": "Hub is not running"}).to_string()
            }
            termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
                termlink_hub::pidfile::remove(&pidfile_path);
                let socket_path = termlink_hub::server::hub_socket_path();
                let _ = std::fs::remove_file(&socket_path);
                serde_json::json!({"ok": true, "action": "cleaned", "pid": pid, "reason": "Stale pidfile removed"}).to_string()
            }
            termlink_hub::pidfile::PidfileStatus::Running(pid) => {
                unsafe { libc::kill(pid as i32, libc::SIGTERM) };
                // Wait up to 2 seconds for shutdown
                for _ in 0..20 {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    if !termlink_session::liveness::process_exists(pid) {
                        return serde_json::json!({"ok": true, "action": "stopped", "pid": pid}).to_string();
                    }
                }
                json_err(format!("hub (PID {pid}) did not stop within 2 seconds"))
            }
        }
    }

    #[tool(
        name = "termlink_agent_ask",
        description = "Send a typed agent request to a target session and wait for its response. Uses the agent protocol (agent.request → agent.response events). The target session must have an agent.listen handler. Returns the response result or error."
    )]
    async fn termlink_agent_ask(&self, Parameters(p): Parameters<AgentAskParams>) -> String {
        use termlink_protocol::events::{agent_topic, AgentRequest, SCHEMA_VERSION};

        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let timeout_secs = p.timeout.unwrap_or(30);
        let sender = p.from.unwrap_or_else(|| format!("mcp-{}", std::process::id()));
        let params = p.params.unwrap_or(serde_json::json!({}));

        let request_id = format!(
            "req-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let request = AgentRequest {
            schema_version: SCHEMA_VERSION.to_string(),
            request_id: request_id.clone(),
            from: sender,
            to: p.target.clone(),
            action: p.action.clone(),
            params,
            timeout_secs: Some(timeout_secs),
        };

        // Snapshot cursor before emitting
        let cursor: Option<u64> = {
            let sub_params = serde_json::json!({"timeout_ms": 1});
            match client::rpc_call(reg.socket_path(), "event.subscribe", sub_params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        result["next_seq"].as_u64()
                    } else { None }
                }
                Err(_) => None,
            }
        };

        // Emit agent.request
        let payload = match serde_json::to_value(&request) {
            Ok(v) => v,
            Err(e) => return json_err(format!("failed to serialize agent request: {e}")),
        };
        let emit_params = serde_json::json!({
            "topic": agent_topic::REQUEST,
            "payload": payload,
        });

        match client::rpc_call(reg.socket_path(), "event.emit", emit_params).await {
            Ok(resp) => {
                if let Err(e) = client::unwrap_result(resp) {
                    return json_err(format!("failed to emit agent request: {e}"));
                }
            }
            Err(e) => return json_err(format!("connection failed: {e}")),
        }

        // Subscribe for agent.response with matching request_id
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let subscribe_timeout: u64 = 5000;
        let mut sub_cursor = cursor;

        loop {
            let remaining = deadline.duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return format!(
                    "Timeout: no agent response within {}s (action: {}, request_id: {})",
                    timeout_secs, p.action, request_id
                );
            }

            let effective_timeout = subscribe_timeout.min(remaining.as_millis() as u64);
            let mut sub_params = serde_json::json!({
                "topic": agent_topic::RESPONSE,
                "timeout_ms": effective_timeout,
            });
            if let Some(c) = sub_cursor {
                sub_params["since"] = serde_json::json!(c);
            }

            match client::rpc_call(reg.socket_path(), "event.subscribe", sub_params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp)
                        && let Some(events) = result["events"].as_array()
                    {
                        for event in events {
                            let event_payload = &event["payload"];
                            let matches = event_payload
                                .get("request_id")
                                .and_then(|r| r.as_str())
                                .map(|r| r == request_id)
                                .unwrap_or(false);

                            if matches {
                                let status = event_payload.get("status")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("unknown");
                                let is_ok = status == "ok";
                                let response = serde_json::json!({
                                    "ok": is_ok,
                                    "action": p.action,
                                    "request_id": request_id,
                                    "status": status,
                                    "result": event_payload.get("result"),
                                    "error": event_payload.get("error_message"),
                                });
                                return serde_json::to_string_pretty(&response)
                                    .unwrap_or_else(json_err);
                            }
                        }

                        if let Some(next) = result["next_seq"].as_u64() {
                            sub_cursor = Some(next.saturating_sub(1));
                        }
                    }
                }
                Err(e) => return json_err(format!("connection lost: {e}")),
            }
        }
    }

    #[tool(
        name = "termlink_version",
        description = "Get the TermLink version, build commit, and target platform. No parameters needed."
    )]
    async fn termlink_version(&self) -> String {
        let version = env!("CARGO_PKG_VERSION");
        let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
        let target = option_env!("BUILD_TARGET").unwrap_or("unknown");
        let tool_count = self.tool_router.list_all().len();

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "version": version,
            "commit": commit,
            "target": target,
            "mcp_tools": tool_count,
        }))
        .unwrap_or_else(|_| format!("termlink {version} ({commit}) [{target}]"))
    }

    #[tool(
        name = "termlink_token_create",
        description = "Create a capability token for a session that has --token-secret enabled. Returns a signed token with the specified scope (observe, control, or execute) and TTL."
    )]
    async fn termlink_token_create(&self, Parameters(p): Parameters<TokenCreateParams>) -> String {
        use termlink_session::auth;

        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let secret_hex = match reg.token_secret.as_ref() {
            Some(s) => s,
            None => return json_err(format!(
                "session '{}' does not have token auth enabled. Register with --token-secret.",
                p.target
            )),
        };

        if secret_hex.len() != 64 {
            return json_err("invalid token_secret in registration (expected 64 hex chars)");
        }

        let mut secret_bytes = [0u8; 32];
        for i in 0..32 {
            match u8::from_str_radix(&secret_hex[i * 2..i * 2 + 2], 16) {
                Ok(v) => secret_bytes[i] = v,
                Err(e) => return json_err(format!("invalid hex in token_secret: {e}")),
            }
        }

        let scope_str = p.scope.as_deref().unwrap_or("execute");
        let scope = match auth::parse_scope(scope_str) {
            Ok(s) => s,
            Err(e) => return json_err(format!("invalid scope '{}': {e}", scope_str)),
        };

        let ttl = p.ttl.unwrap_or(3600);
        let token = auth::create_token(&secret_bytes, scope, reg.id.as_str(), ttl);
        let fallback = token.raw.clone();

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "token": token.raw,
            "scope": scope_str,
            "ttl": ttl,
            "session": reg.id.as_str(),
        }))
        .unwrap_or(fallback)
    }

    #[tool(
        name = "termlink_token_inspect",
        description = "Decode and inspect a TermLink capability token. Returns the token payload (session, scope, expiry) and whether it has expired. Does not verify the signature."
    )]
    async fn termlink_token_inspect(&self, Parameters(p): Parameters<TokenInspectParams>) -> String {
        use base64::Engine;

        let parts: Vec<&str> = p.token.splitn(2, '.').collect();
        if parts.len() != 2 {
            return json_err("invalid token format (expected payload.signature)");
        }

        let payload_bytes = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0]) {
            Ok(v) => v,
            Err(e) => return json_err(format!("invalid base64 in token payload: {e}")),
        };

        let payload: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
            Ok(v) => v,
            Err(e) => return json_err(format!("invalid JSON in token payload: {e}")),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expired = payload["expires_at"].as_u64().map(|e| now > e).unwrap_or(false);

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "payload": payload,
            "expired": expired,
        }))
        .unwrap_or_else(|_| format!("{payload}"))
    }

    #[tool(
        name = "termlink_send",
        description = "Send a generic JSON-RPC method call to any TermLink session. This is the lowest-level building block — lets you call any RPC method (e.g., termlink.ping, query.capabilities, pty.write) on any session."
    )]
    async fn termlink_send(&self, Parameters(p): Parameters<SendParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "ok": false,
                    "error": format!("session '{}' not found: {e}", p.target),
                }))
                .unwrap_or_else(json_err);
            }
        };

        let params: serde_json::Value = match &p.params {
            Some(s) => match serde_json::from_str(s) {
                Ok(v) => v,
                Err(e) => {
                    return serde_json::to_string_pretty(&serde_json::json!({
                        "ok": false,
                        "error": format!("invalid JSON params: {e}"),
                    }))
                    .unwrap_or_else(json_err);
                }
            },
            None => serde_json::json!({}),
        };

        let timeout_secs = p.timeout.unwrap_or(10);
        let call_fut = client::rpc_call(reg.socket_path(), &p.method, params);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            call_fut,
        )
        .await;

        match result {
            Ok(Ok(resp)) => match client::unwrap_result(resp) {
                Ok(val) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "target": p.target,
                    "method": p.method,
                    "result": val,
                }))
                .unwrap_or_else(json_err),
                Err(e) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": false,
                    "target": p.target,
                    "method": p.method,
                    "error": e,
                }))
                .unwrap_or_else(json_err),
            },
            Ok(Err(e)) => serde_json::to_string_pretty(&serde_json::json!({
                "ok": false,
                "target": p.target,
                "method": p.method,
                "error": format!("RPC call failed: {e}"),
            }))
            .unwrap_or_else(json_err),
            Err(_) => serde_json::to_string_pretty(&serde_json::json!({
                "ok": false,
                "target": p.target,
                "method": p.method,
                "error": format!("timeout after {timeout_secs}s"),
            }))
            .unwrap_or_else(json_err),
        }
    }

    #[tool(
        name = "termlink_batch_exec",
        description = "Run a shell command across multiple sessions matching a filter (tag, role, name). Executes concurrently and returns per-session results with stdout/stderr/exit_code. Useful for fleet-wide operations like 'git status' across all workers or 'echo ready' to check liveness."
    )]
    async fn termlink_batch_exec(&self, Parameters(p): Parameters<BatchExecParams>) -> String {
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => return json_err(format!("failed to list sessions: {e}")),
        };
        let filtered: Vec<_> = sessions
            .iter()
            .filter(|s| {
                if p.tag.as_ref().is_some_and(|tag| !s.tags.iter().any(|t| t == tag)) {
                    return false;
                }
                if p.role.as_ref().is_some_and(|role| !s.roles.iter().any(|r| r == role)) {
                    return false;
                }
                if p.name.as_ref().is_some_and(|name| !s.display_name.contains(name.as_str())) {
                    return false;
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "results": [],
                "total": 0,
                "succeeded": 0,
                "failed": 0,
                "message": "No sessions matched the filter"
            }))
            .unwrap_or_else(json_err);
        }

        let timeout_secs = p.timeout.unwrap_or(30);
        let max_parallel = p.max_parallel.unwrap_or(10);
        let command = p.command.clone();

        // Execute concurrently with a semaphore for max parallelism
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_parallel));
        let mut handles = Vec::new();

        for reg in &filtered {
            let sem = semaphore.clone();
            let socket = reg.socket_path().to_path_buf();
            let session_id = reg.id.as_str().to_string();
            let display_name = reg.display_name.clone();
            let cmd = command.clone();
            let timeout = timeout_secs;

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let params = serde_json::json!({
                    "command": cmd,
                    "timeout": timeout,
                });
                let rpc_timeout = std::time::Duration::from_secs(timeout + 5);
                match tokio::time::timeout(
                    rpc_timeout,
                    client::rpc_call(&socket, "command.exec", params),
                )
                .await
                {
                    Ok(Ok(resp)) => match client::unwrap_result(resp) {
                        Ok(val) => serde_json::json!({
                            "session": session_id,
                            "display_name": display_name,
                            "ok": true,
                            "stdout": val.get("stdout").and_then(|v| v.as_str()).unwrap_or(""),
                            "stderr": val.get("stderr").and_then(|v| v.as_str()).unwrap_or(""),
                            "exit_code": val.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(-1),
                        }),
                        Err(e) => serde_json::json!({
                            "session": session_id,
                            "display_name": display_name,
                            "ok": false,
                            "error": e,
                        }),
                    },
                    Ok(Err(e)) => serde_json::json!({
                        "session": session_id,
                        "display_name": display_name,
                        "ok": false,
                        "error": format!("connection failed: {e}"),
                    }),
                    Err(_) => serde_json::json!({
                        "session": session_id,
                        "display_name": display_name,
                        "ok": false,
                        "error": format!("timeout after {timeout}s"),
                    }),
                }
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(serde_json::json!({"ok": false, "error": format!("task panic: {e}")})),
            }
        }

        let total = results.len();
        let succeeded = results.iter().filter(|r| r["ok"] == true).count();
        let failed = total - succeeded;

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": failed == 0,
            "results": results,
            "total": total,
            "succeeded": succeeded,
            "failed": failed,
        }))
        .unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_batch_ping",
        description = "Ping multiple sessions matching a filter and return health status. Lightweight fleet health check — returns per-session alive/dead status with latency and age. Faster than batch_exec for liveness checks."
    )]
    async fn termlink_batch_ping(&self, Parameters(p): Parameters<BatchPingParams>) -> String {
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => return json_err(format!("failed to list sessions: {e}")),
        };
        let filtered: Vec<_> = sessions
            .iter()
            .filter(|s| {
                if p.tag.as_ref().is_some_and(|tag| !s.tags.iter().any(|t| t == tag)) {
                    return false;
                }
                if p.role.as_ref().is_some_and(|role| !s.roles.iter().any(|r| r == role)) {
                    return false;
                }
                if p.name.as_ref().is_some_and(|name| !s.display_name.contains(name.as_str())) {
                    return false;
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "results": [],
                "total": 0,
                "alive": 0,
                "dead": 0,
                "message": "No sessions matched the filter"
            }))
            .unwrap_or_else(json_err);
        }

        let timeout_secs = p.timeout.unwrap_or(5);
        let mut handles = Vec::new();

        for reg in &filtered {
            let socket = reg.socket_path().to_path_buf();
            let session_id = reg.id.as_str().to_string();
            let display_name = reg.display_name.clone();
            let age = format_age(&reg.created_at);
            let timeout = timeout_secs;

            handles.push(tokio::spawn(async move {
                let start = std::time::Instant::now();
                let rpc_timeout = std::time::Duration::from_secs(timeout);
                let alive = match tokio::time::timeout(
                    rpc_timeout,
                    client::rpc_call(&socket, "termlink.ping", serde_json::json!({})),
                )
                .await
                {
                    Ok(Ok(resp)) => matches!(resp, termlink_protocol::jsonrpc::RpcResponse::Success(_)),
                    _ => false,
                };
                let latency_ms = start.elapsed().as_millis() as u64;

                serde_json::json!({
                    "session": session_id,
                    "display_name": display_name,
                    "alive": alive,
                    "latency_ms": latency_ms,
                    "age": age,
                })
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(serde_json::json!({"alive": false, "error": format!("task panic: {e}")})),
            }
        }

        let total = results.len();
        let alive_count = results.iter().filter(|r| r["alive"] == true).count();
        let dead_count = total - alive_count;

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": dead_count == 0,
            "results": results,
            "total": total,
            "alive": alive_count,
            "dead": dead_count,
        }))
        .unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_batch_tag",
        description = "Apply tag or role changes to multiple sessions matching a filter. Useful for fleet-wide labeling — e.g., add a 'deprecated' tag to all sessions matching a name pattern, or assign a role to all workers with a specific tag."
    )]
    async fn termlink_batch_tag(&self, Parameters(p): Parameters<BatchTagParams>) -> String {
        // Validate at least one update operation
        let has_updates = p.add_tags.is_some() || p.remove_tags.is_some()
            || p.add_roles.is_some() || p.remove_roles.is_some();
        if !has_updates {
            return json_err("specify at least one of: add_tags, remove_tags, add_roles, remove_roles");
        }

        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => return json_err(format!("failed to list sessions: {e}")),
        };
        let filtered: Vec<_> = sessions
            .iter()
            .filter(|s| {
                if p.filter_tag.as_ref().is_some_and(|tag| !s.tags.iter().any(|t| t == tag)) {
                    return false;
                }
                if p.filter_role.as_ref().is_some_and(|role| !s.roles.iter().any(|r| r == role)) {
                    return false;
                }
                if p.filter_name.as_ref().is_some_and(|name| !s.display_name.contains(name.as_str())) {
                    return false;
                }
                true
            })
            .collect();

        if filtered.is_empty() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "results": [],
                "total": 0,
                "succeeded": 0,
                "failed": 0,
                "message": "No sessions matched the filter"
            }))
            .unwrap_or_else(json_err);
        }

        let mut handles = Vec::new();
        for reg in &filtered {
            let socket = reg.socket_path().to_path_buf();
            let session_id = reg.id.as_str().to_string();
            let display_name = reg.display_name.clone();
            let add_tags = p.add_tags.clone();
            let remove_tags = p.remove_tags.clone();
            let add_roles = p.add_roles.clone();
            let remove_roles = p.remove_roles.clone();

            handles.push(tokio::spawn(async move {
                let mut params = serde_json::json!({});
                if let Some(tags) = &add_tags {
                    params["add_tags"] = serde_json::json!(tags);
                }
                if let Some(tags) = &remove_tags {
                    params["remove_tags"] = serde_json::json!(tags);
                }
                if let Some(roles) = &add_roles {
                    params["add_roles"] = serde_json::json!(roles);
                }
                if let Some(roles) = &remove_roles {
                    params["remove_roles"] = serde_json::json!(roles);
                }

                let rpc_timeout = std::time::Duration::from_secs(10);
                match tokio::time::timeout(
                    rpc_timeout,
                    client::rpc_call(&socket, "session.update", params),
                )
                .await
                {
                    Ok(Ok(resp)) => match client::unwrap_result(resp) {
                        Ok(result) => serde_json::json!({
                            "session": session_id,
                            "display_name": result["display_name"].as_str().unwrap_or(&display_name),
                            "ok": true,
                            "tags": result["tags"],
                            "roles": result["roles"],
                        }),
                        Err(e) => serde_json::json!({
                            "session": session_id,
                            "display_name": display_name,
                            "ok": false,
                            "error": e,
                        }),
                    },
                    Ok(Err(e)) => serde_json::json!({
                        "session": session_id,
                        "display_name": display_name,
                        "ok": false,
                        "error": format!("connection failed: {e}"),
                    }),
                    Err(_) => serde_json::json!({
                        "session": session_id,
                        "display_name": display_name,
                        "ok": false,
                        "error": "timeout after 10s",
                    }),
                }
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(serde_json::json!({"ok": false, "error": format!("task panic: {e}")})),
            }
        }

        let total = results.len();
        let succeeded = results.iter().filter(|r| r["ok"] == true).count();
        let failed = total - succeeded;

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": failed == 0,
            "results": results,
            "total": total,
            "succeeded": succeeded,
            "failed": failed,
        }))
        .unwrap_or_else(json_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === parse_signal tests ===

    #[test]
    fn parse_signal_named_signals() {
        assert_eq!(parse_signal("TERM"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("INT"), Some(libc::SIGINT));
        assert_eq!(parse_signal("KILL"), Some(libc::SIGKILL));
        assert_eq!(parse_signal("HUP"), Some(libc::SIGHUP));
        assert_eq!(parse_signal("USR1"), Some(libc::SIGUSR1));
        assert_eq!(parse_signal("USR2"), Some(libc::SIGUSR2));
    }

    #[test]
    fn parse_signal_sig_prefixed() {
        assert_eq!(parse_signal("SIGTERM"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("SIGINT"), Some(libc::SIGINT));
        assert_eq!(parse_signal("SIGKILL"), Some(libc::SIGKILL));
        assert_eq!(parse_signal("SIGHUP"), Some(libc::SIGHUP));
        assert_eq!(parse_signal("SIGUSR1"), Some(libc::SIGUSR1));
        assert_eq!(parse_signal("SIGUSR2"), Some(libc::SIGUSR2));
    }

    #[test]
    fn parse_signal_case_insensitive() {
        assert_eq!(parse_signal("term"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("Term"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("sigint"), Some(libc::SIGINT));
        assert_eq!(parse_signal("SigKill"), Some(libc::SIGKILL));
    }

    #[test]
    fn parse_signal_numeric() {
        assert_eq!(parse_signal("9"), Some(9));
        assert_eq!(parse_signal("15"), Some(15));
        assert_eq!(parse_signal("1"), Some(1));
    }

    #[test]
    fn parse_signal_invalid() {
        assert_eq!(parse_signal("BOGUS"), None);
        assert_eq!(parse_signal(""), None);
        assert_eq!(parse_signal("SIGFOO"), None);
        assert_eq!(parse_signal("abc"), None);
    }

    // === Parameter struct deserialization tests ===

    #[test]
    fn ping_params_required_fields() {
        let json = serde_json::json!({"target": "my-session"});
        let p: PingParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "my-session");
    }

    #[test]
    fn ping_params_missing_target() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<PingParams>(json);
        assert!(result.is_err());
    }

    #[test]
    fn exec_params_all_fields() {
        let json = serde_json::json!({
            "target": "worker-1",
            "command": "ls -la",
            "cwd": "/tmp",
            "timeout": 60
        });
        let p: ExecParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "worker-1");
        assert_eq!(p.command, "ls -la");
        assert_eq!(p.cwd.as_deref(), Some("/tmp"));
        assert_eq!(p.timeout, Some(60));
    }

    #[test]
    fn exec_params_optional_fields_omitted() {
        let json = serde_json::json!({"target": "s1", "command": "echo hi"});
        let p: ExecParams = serde_json::from_value(json).unwrap();
        assert!(p.cwd.is_none());
        assert!(p.timeout.is_none());
    }

    #[test]
    fn discover_params_all_optional() {
        let json = serde_json::json!({});
        let p: DiscoverParams = serde_json::from_value(json).unwrap();
        assert!(p.tags.is_none());
        assert!(p.roles.is_none());
        assert!(p.cap.is_none());
        assert!(p.name.is_none());
    }

    #[test]
    fn discover_params_with_filters() {
        let json = serde_json::json!({
            "tags": ["prod", "gpu"],
            "roles": ["worker"],
            "cap": ["execute"],
            "name": "agent"
        });
        let p: DiscoverParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.tags.as_ref().unwrap().len(), 2);
        assert_eq!(p.roles.as_ref().unwrap()[0], "worker");
        assert_eq!(p.name.as_deref(), Some("agent"));
    }

    #[test]
    fn spawn_params_defaults() {
        let json = serde_json::json!({});
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert!(p.name.is_none());
        assert!(p.roles.is_none());
        assert!(p.tags.is_none());
        assert!(p.command.is_none());
        assert!(p.wait.is_none());
        assert!(p.wait_timeout.is_none());
    }

    #[test]
    fn spawn_params_full() {
        let json = serde_json::json!({
            "name": "builder",
            "roles": ["ci"],
            "tags": ["linux"],
            "command": ["make", "build"],
            "wait": true,
            "wait_timeout": 30
        });
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.name.as_deref(), Some("builder"));
        assert_eq!(p.command.as_ref().unwrap(), &["make", "build"]);
        assert_eq!(p.wait, Some(true));
    }

    #[test]
    fn tag_params_set_mode() {
        let json = serde_json::json!({"target": "s1", "set": ["a", "b"]});
        let p: TagParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.set.as_ref().unwrap().len(), 2);
        assert!(p.add.is_none());
        assert!(p.remove.is_none());
    }

    #[test]
    fn tag_params_add_remove_mode() {
        let json = serde_json::json!({"target": "s1", "add": ["x"], "remove": ["y"]});
        let p: TagParams = serde_json::from_value(json).unwrap();
        assert!(p.set.is_none());
        assert_eq!(p.add.as_ref().unwrap()[0], "x");
        assert_eq!(p.remove.as_ref().unwrap()[0], "y");
    }

    #[test]
    fn tag_params_name_and_roles() {
        let json = serde_json::json!({
            "target": "s1",
            "name": "new-name",
            "roles": ["orchestrator", "monitor"],
            "add": ["tag1"]
        });
        let p: TagParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.name.as_deref(), Some("new-name"));
        assert_eq!(p.roles.as_ref().unwrap(), &["orchestrator", "monitor"]);
        assert_eq!(p.add.as_ref().unwrap(), &["tag1"]);
        assert!(p.add_roles.is_none());
        assert!(p.remove_roles.is_none());
    }

    #[test]
    fn tag_params_add_remove_roles() {
        let json = serde_json::json!({
            "target": "s1",
            "add_roles": ["worker"],
            "remove_roles": ["idle"]
        });
        let p: TagParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.add_roles.as_ref().unwrap(), &["worker"]);
        assert_eq!(p.remove_roles.as_ref().unwrap(), &["idle"]);
        assert!(p.name.is_none());
        assert!(p.roles.is_none());
    }

    #[test]
    fn batch_exec_params_full() {
        let json = serde_json::json!({
            "command": "echo hello",
            "tag": "worker",
            "role": "builder",
            "name": "wk-",
            "timeout": 60,
            "max_parallel": 5
        });
        let p: BatchExecParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.command, "echo hello");
        assert_eq!(p.tag.as_deref(), Some("worker"));
        assert_eq!(p.role.as_deref(), Some("builder"));
        assert_eq!(p.name.as_deref(), Some("wk-"));
        assert_eq!(p.timeout, Some(60));
        assert_eq!(p.max_parallel, Some(5));
    }

    #[test]
    fn batch_exec_params_minimal() {
        let json = serde_json::json!({"command": "date"});
        let p: BatchExecParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.command, "date");
        assert!(p.tag.is_none());
        assert!(p.role.is_none());
        assert!(p.name.is_none());
        assert!(p.timeout.is_none());
        assert!(p.max_parallel.is_none());
    }

    #[test]
    fn batch_ping_params_full() {
        let json = serde_json::json!({
            "tag": "worker",
            "role": "compute",
            "name": "wk-",
            "timeout": 10
        });
        let p: BatchPingParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.tag.as_deref(), Some("worker"));
        assert_eq!(p.role.as_deref(), Some("compute"));
        assert_eq!(p.name.as_deref(), Some("wk-"));
        assert_eq!(p.timeout, Some(10));
    }

    #[test]
    fn batch_ping_params_empty() {
        let json = serde_json::json!({});
        let p: BatchPingParams = serde_json::from_value(json).unwrap();
        assert!(p.tag.is_none());
        assert!(p.role.is_none());
        assert!(p.name.is_none());
        assert!(p.timeout.is_none());
    }

    #[test]
    fn batch_tag_params_full() {
        let json = serde_json::json!({
            "filter_tag": "worker",
            "filter_name": "wk-",
            "add_tags": ["active"],
            "remove_tags": ["idle"],
            "add_roles": ["compute"],
            "remove_roles": ["standby"]
        });
        let p: BatchTagParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.filter_tag.as_deref(), Some("worker"));
        assert_eq!(p.add_tags.as_ref().unwrap(), &["active"]);
        assert_eq!(p.remove_tags.as_ref().unwrap(), &["idle"]);
        assert_eq!(p.add_roles.as_ref().unwrap(), &["compute"]);
        assert_eq!(p.remove_roles.as_ref().unwrap(), &["standby"]);
    }

    #[test]
    fn batch_tag_params_minimal() {
        let json = serde_json::json!({"add_tags": ["test"]});
        let p: BatchTagParams = serde_json::from_value(json).unwrap();
        assert!(p.filter_tag.is_none());
        assert!(p.filter_role.is_none());
        assert!(p.filter_name.is_none());
        assert_eq!(p.add_tags.as_ref().unwrap(), &["test"]);
        assert!(p.remove_tags.is_none());
    }

    #[test]
    fn resize_params_required() {
        let json = serde_json::json!({"target": "s1", "cols": 120, "rows": 40});
        let p: ResizeParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.cols, 120);
        assert_eq!(p.rows, 40);
    }

    #[test]
    fn resize_params_missing_field() {
        let json = serde_json::json!({"target": "s1", "cols": 80});
        let result = serde_json::from_value::<ResizeParams>(json);
        assert!(result.is_err());
    }

    #[test]
    fn event_subscribe_params_defaults() {
        let json = serde_json::json!({"target": "s1"});
        let p: EventSubscribeParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "s1");
        assert!(p.timeout_ms.is_none());
        assert!(p.topic.is_none());
        assert!(p.since.is_none());
        assert!(p.max_events.is_none());
    }

    #[test]
    fn collect_params_all_optional() {
        let json = serde_json::json!({});
        let p: CollectParams = serde_json::from_value(json).unwrap();
        assert!(p.targets.is_none());
        assert!(p.topic.is_none());
        assert!(p.timeout_ms.is_none());
        assert!(p.since.is_none());
    }

    #[test]
    fn agent_ask_params_full() {
        let json = serde_json::json!({
            "target": "specialist",
            "action": "analyze",
            "params": {"file": "main.rs"},
            "from": "orchestrator",
            "timeout": 120
        });
        let p: AgentAskParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "specialist");
        assert_eq!(p.action, "analyze");
        assert_eq!(p.params.unwrap()["file"], "main.rs");
        assert_eq!(p.from.as_deref(), Some("orchestrator"));
        assert_eq!(p.timeout, Some(120));
    }

    #[test]
    fn file_send_params_required() {
        let json = serde_json::json!({"target": "remote-1", "path": "/tmp/data.tar.gz"});
        let p: FileSendParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "remote-1");
        assert_eq!(p.path, "/tmp/data.tar.gz");
    }

    #[test]
    fn file_receive_params_required() {
        let json = serde_json::json!({"target": "worker-1", "output_dir": "/tmp/received"});
        let p: FileReceiveParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "worker-1");
        assert_eq!(p.output_dir, "/tmp/received");
    }

    #[test]
    fn list_sessions_params_all_filters() {
        let json = serde_json::json!({"tag": "prod", "role": "coder", "name": "worker"});
        let p: ListSessionsParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.tag.unwrap(), "prod");
        assert_eq!(p.role.unwrap(), "coder");
        assert_eq!(p.name.unwrap(), "worker");
    }

    #[test]
    fn list_sessions_params_empty() {
        let json = serde_json::json!({});
        let p: ListSessionsParams = serde_json::from_value(json).unwrap();
        assert!(p.tag.is_none());
        assert!(p.role.is_none());
        assert!(p.name.is_none());
    }

    #[test]
    fn send_params_all_fields() {
        let json = serde_json::json!({
            "target": "worker-1",
            "method": "termlink.ping",
            "params": "{\"foo\":1}",
            "timeout": 30
        });
        let p: SendParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "worker-1");
        assert_eq!(p.method, "termlink.ping");
        assert_eq!(p.params.unwrap(), "{\"foo\":1}");
        assert_eq!(p.timeout.unwrap(), 30);
    }

    #[test]
    fn send_params_minimal() {
        let json = serde_json::json!({"target": "session-1", "method": "query.capabilities"});
        let p: SendParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "session-1");
        assert_eq!(p.method, "query.capabilities");
        assert!(p.params.is_none());
        assert!(p.timeout.is_none());
    }

    #[test]
    fn session_info_serializes() {
        let info = SessionInfo {
            id: "tl-abc123".into(),
            display_name: "worker-1".into(),
            state: "ready".into(),
            pid: 12345,
            uid: 1000,
            created_at: "2026-01-01T00:00:00Z".into(),
            heartbeat_at: "2026-01-01T00:01:00Z".into(),
            age: "5d".into(),
            tags: vec!["prod".into()],
            roles: vec!["compute".into()],
            capabilities: vec!["execute".into()],
            metadata: serde_json::json!({"custom": "value"}),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["id"], "tl-abc123");
        assert_eq!(json["display_name"], "worker-1");
        assert_eq!(json["tags"][0], "prod");
        assert_eq!(json["metadata"]["custom"], "value");
    }

    #[test]
    fn token_create_params_required_target() {
        let json = serde_json::json!({"target": "my-session"});
        let p: TokenCreateParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "my-session");
        assert!(p.scope.is_none());
        assert!(p.ttl.is_none());
    }

    #[test]
    fn token_create_params_full() {
        let json = serde_json::json!({"target": "worker-1", "scope": "execute", "ttl": 7200});
        let p: TokenCreateParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "worker-1");
        assert_eq!(p.scope.as_deref(), Some("execute"));
        assert_eq!(p.ttl, Some(7200));
    }

    #[test]
    fn token_create_params_missing_target() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<TokenCreateParams>(json);
        assert!(result.is_err());
    }

    #[test]
    fn token_inspect_params_required_token() {
        let json = serde_json::json!({"token": "payload.sig"});
        let p: TokenInspectParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.token, "payload.sig");
    }

    #[test]
    fn token_inspect_params_missing_token() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<TokenInspectParams>(json);
        assert!(result.is_err());
    }
}
