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
pub struct DiscoverParams {
    /// Filter by tags (sessions must have ALL specified tags)
    pub tags: Option<Vec<String>>,
    /// Filter by roles (sessions must have ALL specified roles)
    pub roles: Option<Vec<String>>,
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

    #[tool(
        name = "termlink_discover",
        description = "Find TermLink sessions by tag, role, or name. Returns matching sessions with IDs, tags, roles, and capabilities."
    )]
    async fn termlink_discover(&self, Parameters(p): Parameters<DiscoverParams>) -> String {
        let sessions = match manager::list_sessions(false) {
            Ok(s) => s,
            Err(e) => return format!("Error: {e}"),
        };

        let tags = p.tags.unwrap_or_default();
        let roles = p.roles.unwrap_or_default();

        let filtered: Vec<_> = sessions
            .into_iter()
            .filter(|s| {
                tags.iter().all(|t| s.tags.contains(t))
                    && roles.iter().all(|r| s.roles.contains(r))
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
                    "tags": s.tags,
                    "roles": s.roles,
                    "capabilities": s.capabilities,
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
            Err(e) => return format!("Error: cannot determine termlink binary: {e}"),
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
            return format!("Error: failed to spawn: {e}");
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
                let mut output = String::new();
                if !result.stdout.is_empty() {
                    output.push_str(&result.stdout);
                }
                if !result.stderr.is_empty() {
                    if !output.is_empty() {
                        output.push('\n');
                    }
                    output.push_str(&format!("[stderr] {}", result.stderr));
                }
                if result.exit_code != 0 {
                    output.push_str(&format!("\n[exit_code: {}]", result.exit_code));
                }
                if output.is_empty() {
                    format!("[exit_code: {}]", result.exit_code)
                } else {
                    output
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        name = "termlink_kv_set",
        description = "Set a key-value pair on a TermLink session's store"
    )]
    async fn termlink_kv_set(&self, Parameters(p): Parameters<KvSetParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_kv_get",
        description = "Get a value from a TermLink session's key-value store"
    )]
    async fn termlink_kv_get(&self, Parameters(p): Parameters<KvGetParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_kv_list",
        description = "List all key-value pairs stored on a TermLink session"
    )]
    async fn termlink_kv_list(&self, Parameters(p): Parameters<KvListParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        match client::rpc_call(reg.socket_path(), "kv.list", serde_json::json!({})).await {
            Ok(resp) => match client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|e| format!("Error: {e}")),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_kv_del",
        description = "Delete a key from a TermLink session's key-value store"
    )]
    async fn termlink_kv_del(&self, Parameters(p): Parameters<KvDelParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_broadcast",
        description = "Broadcast an event to multiple TermLink sessions via the hub. If no targets specified, broadcasts to all."
    )]
    async fn termlink_broadcast(&self, Parameters(p): Parameters<BroadcastParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return "Error: hub is not running. Start it with: termlink hub".into();
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
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_interact",
        description = "Run a shell command in a PTY session and return its output. Injects the command, waits for completion via a unique marker, and returns clean output with exit code. This is the preferred tool for running commands in terminal sessions — it handles injection, waiting, and output capture atomically."
    )]
    async fn termlink_interact(&self, Parameters(p): Parameters<InteractParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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
            Err(e) => return format!("Error: not a PTY session or connection failed: {e}"),
        };

        let pre_output = match client::unwrap_result(pre_resp) {
            Ok(r) => r["output"].as_str().unwrap_or("").to_string(),
            Err(e) => return format!("Error: session has no PTY: {e}"),
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
            return format!("Error: failed to inject command: {e}");
        }

        // Poll until marker appears
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let poll_interval = tokio::time::Duration::from_millis(poll_ms);

        loop {
            if tokio::time::Instant::now() >= deadline {
                return format!("Error: timeout after {}s waiting for command to complete", timeout_secs);
            }

            tokio::time::sleep(poll_interval).await;

            let resp = match client::rpc_call(
                reg.socket_path(),
                "query.output",
                serde_json::json!({ "bytes": 131072, "strip_ansi": true }),
            ).await {
                Ok(r) => r,
                Err(e) => return format!("Error: connection lost: {e}"),
            };

            let full_output = match client::unwrap_result(resp) {
                Ok(r) => r["output"].as_str().unwrap_or("").to_string(),
                Err(e) => return format!("Error: poll failed: {e}"),
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

                return if exit != 0 {
                    if trimmed.is_empty() {
                        format!("[exit_code: {}]", exit)
                    } else {
                        format!("{}\n[exit_code: {}]", trimmed, exit)
                    }
                } else {
                    if trimmed.is_empty() {
                        "[ok]".to_string()
                    } else {
                        trimmed.to_string()
                    }
                };
            }
        }
    }

    #[tool(
        name = "termlink_doctor",
        description = "Run health checks on the TermLink environment. Returns a structured JSON report with pass/warn/fail status for: runtime directory, sessions directory, session liveness, hub status, orphaned sockets, and version. Use this to diagnose connectivity or infrastructure issues before attempting operations."
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

        // 6. Version
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
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("Error: {e}"))
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
                return format!("Error scanning sessions: {e}");
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
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("Error: {e}"))
    }

    #[tool(
        name = "termlink_resize",
        description = "Resize a PTY-backed TermLink session's terminal dimensions. Useful when you need specific column width for parsing command output or formatting."
    )]
    async fn termlink_resize(&self, Parameters(p): Parameters<ResizeParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_request",
        description = "Send a request event to a TermLink session and wait for a reply. Emits an event with a unique request_id on the specified topic, then polls for a reply event on reply_topic with matching request_id. Use this for request-reply coordination between sessions (e.g., send 'task.run', wait for 'task.result')."
    )]
    async fn termlink_request(&self, Parameters(p): Parameters<RequestParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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

        // Snapshot cursor before emitting
        let cursor: Option<u64> = {
            match client::rpc_call(reg.socket_path(), "event.poll", serde_json::json!({})).await {
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
                    return format!("Error: failed to emit request: {e}");
                }
            }
            Err(e) => return format!("Error: connection failed: {e}"),
        }

        // Poll for reply
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let poll_interval = tokio::time::Duration::from_millis(500);
        let mut poll_cursor = cursor;

        loop {
            if tokio::time::Instant::now() >= deadline {
                return format!(
                    "Timeout: no reply on '{}' within {}s (request_id: {})",
                    p.reply_topic, timeout_secs, request_id
                );
            }

            tokio::time::sleep(poll_interval).await;

            let mut params = serde_json::json!({ "topic": p.reply_topic });
            if let Some(c) = poll_cursor {
                params["since"] = serde_json::json!(c);
            }

            match client::rpc_call(reg.socket_path(), "event.poll", params).await {
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

                            if !events.is_empty()
                                && let Some(next) = result["next_seq"].as_u64() {
                                    poll_cursor = Some(next);
                                }
                        }
                }
                Err(e) => return format!("Error: connection lost during poll: {e}"),
            }
        }
    }

    #[tool(
        name = "termlink_tag",
        description = "Manage tags on a TermLink session. Use 'add' to append tags, 'remove' to delete tags, or 'set' to replace all tags. Returns the updated tag list. Tags enable discovery-based orchestration — tag sessions by role, project, or task, then use termlink_discover to find them."
    )]
    async fn termlink_tag(&self, Parameters(p): Parameters<TagParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
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

        if params.as_object().unwrap().is_empty() {
            return "Error: specify at least one of: set, add, or remove".into();
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
                    format!(
                        "Updated {}: tags=[{}]",
                        result["display_name"].as_str().unwrap_or(&p.target),
                        tags.join(", "),
                    )
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: connection failed: {e}"),
        }
    }

    #[tool(
        name = "termlink_wait",
        description = "Wait for a specific event topic to appear on a session's event bus. Blocks until the event arrives or timeout."
    )]
    async fn termlink_wait(&self, Parameters(p): Parameters<WaitParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return format!("Error: session '{}' not found: {e}", p.target),
        };

        let timeout_secs = p.timeout.unwrap_or(30);
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);
        let poll_interval = tokio::time::Duration::from_millis(500);
        let mut cursor: Option<u64> = None;

        loop {
            if tokio::time::Instant::now() >= deadline {
                return format!("Timeout waiting for event topic '{}' ({}s)", p.topic, timeout_secs);
            }

            let mut params = serde_json::json!({"topic": p.topic});
            if let Some(c) = cursor {
                params["since"] = serde_json::json!(c);
            }

            match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        if let Some(events) = result["events"].as_array()
                            && let Some(event) = events.first() {
                                let payload = &event["payload"];
                                return if payload.is_null()
                                    || (payload.is_object()
                                        && payload.as_object().unwrap().is_empty())
                                {
                                    format!("Event received: {}", p.topic)
                                } else {
                                    serde_json::to_string_pretty(payload)
                                        .unwrap_or_else(|_| format!("Event received: {}", p.topic))
                                };
                            }
                        if let Some(next) = result["next_seq"].as_u64() {
                            cursor = if next > 0 { Some(next - 1) } else { None };
                        }
                    }
                }
                Err(e) => return format!("Error: connection lost: {e}"),
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}
