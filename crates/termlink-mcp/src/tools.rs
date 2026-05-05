use std::sync::Arc;

use base64::Engine as _;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use termlink_protocol::format_age;
use termlink_session::{client, endpoint::EndpointHandle, manager};

/// TermLink MCP server — exposes terminal orchestration as structured tools.
#[derive(Clone)]
pub struct TermLinkTools {
    pub tool_router: ToolRouter<Self>,
    /// Background endpoints registered via `termlink_register`.
    endpoints: Arc<Mutex<Vec<EndpointHandle>>>,
}

impl std::fmt::Debug for TermLinkTools {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TermLinkTools")
            .field("tool_router", &self.tool_router)
            .field("endpoints", &"[...]")
            .finish()
    }
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
            endpoints: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Helper: create a JSON error response string.
fn json_err(msg: impl std::fmt::Display) -> String {
    serde_json::to_string_pretty(&serde_json::json!({"ok": false, "error": msg.to_string()}))
        .unwrap_or_else(|e| format!("{{\"ok\":false,\"error\":\"{e}\"}}" ))
}

/// Helper: convert days-since-Unix-epoch to UTC YYYY-MM-DD string.
/// No chrono dep — uses civil-from-days algorithm (Howard Hinnant, public domain).
fn epoch_days_to_ymd(days: i64) -> String {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", year, m, d)
}

/// Helper: connect to a remote hub via TOFU TLS and authenticate.
///
/// Returns an authenticated [`client::Client`] on success, or a pre-formatted
/// JSON error string on any validation/connection/auth failure. MCP tools can
/// early-return the error string directly.
///
/// Mirrors the validation order of `commands/remote.rs::connect_remote_hub`,
/// but returns String errors (not anyhow) so MCP tools stay crash-safe.
/// Resolve a hub profile name to (address, secret_file, secret).
/// If hub contains ':', treat as direct address. Otherwise look up in ~/.termlink/hubs.toml.
fn resolve_hub_profile(hub: &str) -> Option<(String, Option<String>, Option<String>)> {
    if hub.contains(':') {
        return None; // Direct address, no profile resolution needed
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = std::path::PathBuf::from(home).join(".termlink/hubs.toml");
    let content = std::fs::read_to_string(&path).ok()?;

    // Simple TOML parser for [hubs.NAME] sections
    let section_key = format!("[hubs.{}]", hub);
    let section_start = content.find(&section_key)?;
    let section_body = &content[section_start + section_key.len()..];
    let section_end = section_body.find("\n[").unwrap_or(section_body.len());
    let section = &section_body[..section_end];

    let mut address = None;
    let mut secret_file = None;
    let mut secret = None;

    for line in section.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            match key {
                "address" => address = Some(val.to_string()),
                "secret_file" => secret_file = Some(val.to_string()),
                "secret" => secret = Some(val.to_string()),
                _ => {}
            }
        }
    }

    address.map(|addr| (addr, secret_file, secret))
}

/// T-1039: List all hub profiles from ~/.termlink/hubs.toml.
/// Returns vec of (name, address, secret_file, secret).
fn list_all_hub_profiles() -> Vec<(String, String, Option<String>, Option<String>)> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = std::path::PathBuf::from(home).join(".termlink/hubs.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut profiles = Vec::new();
    let prefix = "[hubs.";

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(prefix) && line.ends_with(']') {
            let name = line[prefix.len()..line.len() - 1].to_string();
            if let Some((addr, sf, sec)) = resolve_hub_profile(&name) {
                profiles.push((name, addr, sf, sec));
            }
        }
    }
    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    profiles
}

async fn connect_remote_hub_mcp(
    hub: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    scope: &str,
) -> Result<client::Client, String> {
    use termlink_session::auth::{self, PermissionScope};

    // Resolve profile if hub doesn't contain ':'
    let (resolved_hub, profile_secret_file, profile_secret) = if let Some(profile) = resolve_hub_profile(hub) {
        profile
    } else {
        (hub.to_string(), None, None)
    };

    // Parse hub address
    let parts: Vec<&str> = resolved_hub.split(':').collect();
    if parts.len() != 2 {
        return Err(json_err(format!(
            "Invalid hub address '{}'. Expected format: host:port or profile name",
            hub
        )));
    }
    let host = parts[0].to_string();
    let port: u16 = match parts[1].parse() {
        Ok(p) => p,
        Err(_) => return Err(json_err(format!("Invalid port in '{}'", resolved_hub))),
    };

    // Read secret — CLI params override profile defaults
    let hex = if let Some(path) = secret_file {
        match std::fs::read_to_string(path) {
            Ok(s) => s.trim().to_string(),
            Err(_) => {
                return Err(json_err(format!("Secret file not found: {}", path)));
            }
        }
    } else if let Some(h) = secret_hex {
        h.to_string()
    } else if let Some(ref path) = profile_secret_file {
        match std::fs::read_to_string(path) {
            Ok(s) => s.trim().to_string(),
            Err(_) => {
                return Err(json_err(format!("Profile secret file not found: {}", path)));
            }
        }
    } else if let Some(ref s) = profile_secret {
        s.clone()
    } else {
        return Err(json_err("Either secret_file or secret is required (or configure a profile with: termlink remote profile add)"));
    };

    // Parse hex
    if hex.len() != 64 {
        return Err(json_err(format!(
            "Secret must be 64 hex characters (32 bytes), got {}",
            hex.len()
        )));
    }
    let secret_bytes: Vec<u8> = match (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
    {
        Ok(b) => b,
        Err(_) => return Err(json_err("Secret contains invalid hex characters")),
    };
    let secret: auth::TokenSecret = match secret_bytes.try_into() {
        Ok(s) => s,
        Err(_) => return Err(json_err("Secret must be exactly 32 bytes")),
    };

    // Parse scope
    let perm_scope = match scope {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        other => {
            return Err(json_err(format!(
                "Invalid scope '{}'. Use: observe, interact, control, execute",
                other
            )));
        }
    };

    // Generate auth token
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    // Connect via TOFU TLS
    let addr = termlink_protocol::TransportAddr::Tcp { host, port };
    let mut rpc_client = match client::Client::connect_addr(&addr).await {
        Ok(c) => c,
        Err(e) => {
            return Err(json_err(format!(
                "Cannot connect to {} — is the hub running? ({})",
                hub, e
            )));
        }
    };

    // Authenticate
    match rpc_client
        .call(
            "hub.auth",
            serde_json::json!("auth"),
            serde_json::json!({"token": token.raw}),
        )
        .await
    {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => Ok(rpc_client),
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => Err(json_err(format!(
            "Authentication failed: {} {}",
            e.error.code, e.error.message
        ))),
        Err(e) => Err(json_err(format!("Authentication error: {}", e))),
    }
}

use termlink_protocol::shell_escape;

// === Task governance gate ===

/// Check whether task governance is enforced and a task_id is required.
///
/// When `TERMLINK_TASK_GOVERNANCE=1`, tools that mutate or interact with sessions
/// must include a `task_id` parameter. If missing, this returns an error string
/// that the tool should return directly.
///
/// When governance is not enabled (default), this always returns Ok(()).
fn check_task_governance(task_id: &Option<String>, tool_name: &str) -> Result<(), String> {
    let governance = std::env::var("TERMLINK_TASK_GOVERNANCE").unwrap_or_default();
    if governance != "1" {
        return Ok(());
    }
    match task_id {
        Some(id) if !id.trim().is_empty() => Ok(()),
        _ => Err(json_err(format!(
            "Task governance is enabled (TERMLINK_TASK_GOVERNANCE=1). \
             The '{tool_name}' tool requires a 'task_id' parameter. \
             Provide the task ID of the task you are working on \
             (e.g., \"task_id\": \"T-123\")."
        ))),
    }
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
    /// Environment variables to set (map of KEY → VALUE)
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Task ID for governance tracking (required when TERMLINK_TASK_GOVERNANCE=1)
    pub task_id: Option<String>,
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
    /// Start from this sequence number (replay history from seq onwards)
    pub since: Option<u64>,
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
    /// Capabilities to advertise (e.g., "code", "test", "review")
    pub cap: Option<Vec<String>>,
    /// Environment variables to set in the session (map of KEY → VALUE)
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Command to run in the session (if empty, starts a shell)
    pub command: Option<Vec<String>>,
    /// Wait for session to register before returning (default: true)
    pub wait: Option<bool>,
    /// Wait timeout in seconds (default: 10)
    pub wait_timeout: Option<u64>,
    /// Working directory for the spawned session (cd into before executing)
    pub cwd: Option<String>,
    /// Task ID for governance tracking (required when TERMLINK_TASK_GOVERNANCE=1)
    pub task_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RunParams {
    /// Command to execute in an ephemeral session
    pub command: String,
    /// Timeout in seconds (default: 30)
    pub timeout: Option<u64>,
    /// Working directory for the command
    pub cwd: Option<String>,
    /// Environment variables to set (map of KEY → VALUE)
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct HubStartParams {
    /// Optional TCP bind address (e.g., "0.0.0.0:9100"). When set, the hub
    /// listens on both its Unix socket and the given TCP address, generates
    /// a hub secret for HMAC auth, and writes a TLS cert for TOFU. Leave
    /// unset for local-only (Unix socket) operation.
    pub tcp_addr: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteCallParams {
    /// Remote hub address in "host:port" format (e.g., "192.168.10.107:9100")
    pub hub: String,
    /// JSON-RPC method to invoke on the remote hub (e.g., "session.discover",
    /// "command.inject", "termlink.ping", "event.broadcast")
    pub method: String,
    /// JSON params for the RPC call (tool-specific structure)
    pub params: Option<serde_json::Value>,
    /// Path to a file containing the 32-byte hex hub secret. Takes precedence
    /// over `secret` if both are set.
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret (64 hex characters). Use `secret_file`
    /// instead in production — this form is for scripting.
    pub secret: Option<String>,
    /// Permission scope for the auth token: observe, interact, control, execute.
    /// Default: "control".
    pub scope: Option<String>,
    /// Connection + RPC timeout in seconds. Default: 30.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemotePingParams {
    /// Remote hub address in "host:port" format
    pub hub: String,
    /// Optional target session name. Omit to ping the hub itself (discover).
    pub session: Option<String>,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "observe".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 10.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct InboxListParams {
    /// Target session name to query inbox for
    pub target: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct InboxClearParams {
    /// Target session name to clear inbox for (omit if using all)
    pub target: Option<String>,
    /// Clear all pending transfers for all targets
    pub all: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChannelCreateParams {
    /// Topic name (e.g. "broadcast:global", "channel:learnings")
    pub name: String,
    /// Retention policy kind: "forever" | "days" | "messages". Default: forever.
    pub retention_kind: Option<String>,
    /// Retention value for "days" or "messages" kinds. Ignored for "forever".
    pub retention_value: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChannelPostParams {
    /// Topic name
    pub topic: String,
    /// Free-form message type tag (e.g. "note", "learning", "artifact"). Default: "note".
    pub msg_type: Option<String>,
    /// Inline UTF-8 payload. Exactly one of `payload` or `payload_b64` is required.
    pub payload: Option<String>,
    /// Base64 binary payload. Exactly one of `payload` or `payload_b64` is required.
    pub payload_b64: Option<String>,
    /// Optional opaque artifact reference (e.g. "ref://...").
    pub artifact_ref: Option<String>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPostParams {
    /// Free-form message text to post on agent-chat-arc.
    pub text: String,
    /// Free-form message type tag. Default: "note".
    pub msg_type: Option<String>,
    /// Optional thread tag (typically T-XXX). Stored as `metadata._thread`.
    pub thread: Option<String>,
    /// Optional project name. Stored as `metadata._project` / `from_project`.
    pub project: Option<String>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentTypingParams {
    /// TTL in milliseconds for the typing indicator. Default: 5000ms.
    /// `metadata.expires_at_ms` is set to `now + ttl_ms`. Peers reading
    /// `agent typers` filter expired indicators out automatically.
    pub ttl_ms: Option<u64>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentReactParams {
    /// Offset of the parent chat-arc post being reacted to.
    pub offset: u64,
    /// Reaction emoji or short string (e.g. "👍", "👀", "ack").
    pub emoji: String,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentReplyParams {
    /// Offset of the parent chat-arc post being replied to.
    pub offset: u64,
    /// Reply text content.
    pub text: String,
    /// Optional thread tag. Stored as `metadata._thread`.
    pub thread: Option<String>,
    /// Optional project name. Stored as `metadata._project` / `from_project`.
    pub project: Option<String>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPinParams {
    /// Offset of the chat-arc post to pin (or unpin).
    pub offset: u64,
    /// If true, emit an unpin envelope instead of pin. Default: false.
    pub unpin: Option<bool>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentStarParams {
    /// Offset of the chat-arc post to star (or unstar).
    pub offset: u64,
    /// If true, emit an unstar envelope instead of star. Default: false.
    pub unstar: Option<bool>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentRedactParams {
    /// Offset of the chat-arc post being retracted.
    pub offset: u64,
    /// Optional reason string. Stored as `metadata.reason`.
    pub reason: Option<String>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentEditParams {
    /// Offset of the chat-arc post being edited.
    pub offset: u64,
    /// New post content. Replaces the original at render time.
    pub text: String,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentAckParams {
    /// Highest offset the caller has read on agent-chat-arc.
    pub up_to: u64,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentDescribeParams {
    /// Topic description to record on agent-chat-arc.
    pub description: String,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPollStartParams {
    /// Poll question (free-text).
    pub question: String,
    /// Two or more option labels. Pipe character is reserved.
    pub options: Vec<String>,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPollVoteParams {
    /// Offset of the poll_start envelope.
    pub poll_id: u64,
    /// Zero-indexed option choice.
    pub choice: u64,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPollEndParams {
    /// Offset of the poll_start envelope to close.
    pub poll_id: u64,
    /// Override sender_id (default: identity fingerprint).
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentRecentParams {
    /// Max envelopes to return. Default 20, capped at 1000.
    pub limit: Option<u64>,
    /// If set, filter to envelopes whose sender_id matches.
    pub peer_fp: Option<String>,
    /// If set, filter to envelopes whose msg_type matches (e.g. "note").
    pub msg_type_filter: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentSearchParams {
    /// Substring query — matched against base64-decoded payload.
    pub query: String,
    /// Max matches to return. Default 100, capped at 1000.
    pub limit: Option<u64>,
    /// If set, filter to envelopes whose sender_id matches.
    pub peer_fp: Option<String>,
    /// If set, filter to envelopes whose msg_type matches.
    pub msg_type_filter: Option<String>,
    /// If true, case-sensitive substring match. Default: false.
    pub case_sensitive: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentAncestorsParams {
    /// Offset whose ancestor chain to compute.
    pub offset: u64,
    /// Max chain depth (safety cap). Default 100.
    pub max_depth: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPinHistoryParams {
    /// Max events to return. Default 200, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentUnreadParams {
    /// sender_id whose unread count to compute. Defaults to local identity fingerprint.
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentDigestParams {
    /// Absolute since timestamp (unix ms). Takes precedence over window_hours.
    pub since_ts: Option<i64>,
    /// Period window in hours, relative to now. Default 24, capped at 720 (30 days).
    pub window_hours: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentRedactionsParams {
    /// Max redactions to return. Default 200, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentAckStatusParams {}

#[derive(Deserialize, JsonSchema)]
pub struct AgentEmojiStatsParams {
    /// Max emojis to return. Default 50, capped at 500.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentAckHistoryParams {
    /// Sender fingerprint whose ack timeline to fetch. Defaults to caller's local Identity.
    pub sender_id: Option<String>,
    /// Max receipts to return. Default 200, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentEditsOfParams {
    /// Offset of the chat-arc envelope whose edit history should be listed.
    pub offset: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentTopicStatsParams {
    /// Optional max days back to include (truncates older buckets). No default — full history if unset.
    pub window_days: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentActiveNowParams {
    /// Window size in minutes — only senders posting within this window are returned. Default 60.
    pub window_minutes: Option<u64>,
    /// Max senders to return. Default 100, capped at 500.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentHistoryParams {
    /// Sender fingerprint whose post history to fetch. Defaults to caller's local Identity.
    pub sender_id: Option<String>,
    /// Max posts to return. Default 50, capped at 500.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentFollowupsParams {
    /// Offset of the chat-arc envelope whose downstream references should be aggregated.
    pub offset: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentStateParams {}

#[derive(Deserialize, JsonSchema)]
pub struct AgentTopicMetadataHistoryParams {
    /// Max entries to return. Default 100, capped at 500.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentReactionsByParams {
    /// Sender fingerprint whose reaction history to fetch. Defaults to caller's local Identity.
    pub sender_id: Option<String>,
    /// Max reactions to return. Default 200, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPinnedByParams {
    /// Sender fingerprint whose pinning activity to reduce. Defaults to caller's local Identity.
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentStarredByParams {
    /// Sender fingerprint whose starring activity to reduce. Defaults to caller's local Identity.
    pub sender_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentInfoParams {}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPeersParams {
    /// Max peers to return. Default 200, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentOnThreadParams {
    /// Root offset on agent-chat-arc — descendants are collected via in_reply_to chain.
    pub root_offset: u64,
    /// Max envelopes to return. Default 200, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentReactionsParams {
    /// Offset of the chat-arc envelope whose reactions should be listed.
    pub offset: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentQuoteParams {
    /// Offset of the chat-arc envelope to fetch.
    pub offset: u64,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentThreadsParams {
    /// Max thread roots to return. Default 100, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentPinnedParams {
    /// Max pinned entries to return. Default 100, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AgentStarredParams {
    /// If set, only return stars set by this sender_id (per-user view).
    pub peer_fp: Option<String>,
    /// Max starred entries to return. Default 100, capped at 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChannelSubscribeParams {
    /// Topic name
    pub topic: String,
    /// Cursor to start at. Default: 0.
    pub cursor: Option<u64>,
    /// Max messages per call. Default: 100, max 1000.
    pub limit: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChannelListParams {
    /// Optional topic name prefix filter.
    pub prefix: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChannelQueueStatusParams {
    /// Optional path to the queue sqlite file. Defaults to `~/.termlink/outbound.sqlite`.
    pub queue_path: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteInboxStatusParams {
    /// Remote hub address in "host:port" format or profile name
    pub hub: String,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "execute".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 10.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteInboxListParams {
    /// Remote hub address in "host:port" format or profile name
    pub hub: String,
    /// Target session name to query inbox for
    pub target: String,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "execute".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 10.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteInboxClearParams {
    /// Remote hub address in "host:port" format or profile name
    pub hub: String,
    /// Target session name to clear inbox for (omit if using all)
    pub target: Option<String>,
    /// Clear all pending transfers for all targets
    pub all: Option<bool>,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "execute".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 10.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteListParams {
    /// Remote hub address in "host:port" format or profile name
    pub hub: String,
    /// Filter by session name (substring match)
    pub name: Option<String>,
    /// Filter by tags (comma-separated, all must match)
    pub tags: Option<String>,
    /// Filter by roles (comma-separated, all must match)
    pub roles: Option<String>,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "observe".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 10.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteExecParams {
    /// Remote hub address in "host:port" format or profile name
    pub hub: String,
    /// Target session name on the remote hub
    pub session: String,
    /// Shell command to execute
    pub command: String,
    /// Working directory for the command
    pub cwd: Option<String>,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "execute".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 30.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteDoctorParams {
    /// Remote hub address in "host:port" format or profile name
    pub hub: String,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Permission scope. Default: "execute".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 10.
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RemoteInjectParams {
    /// Remote hub address in "host:port" format
    pub hub: String,
    /// Target session name on the remote hub
    pub session: String,
    /// Text to inject into the remote session's terminal
    pub text: String,
    /// Path to file containing the 32-byte hex hub secret
    pub secret_file: Option<String>,
    /// Hex-encoded 32-byte hub secret
    pub secret: Option<String>,
    /// Append Enter keystroke after the text. Default: false.
    pub enter: Option<bool>,
    /// Permission scope. Default: "control".
    pub scope: Option<String>,
    /// Timeout in seconds. Default: 30.
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
pub struct KvWatchParams {
    /// Session ID or display name
    pub target: String,
    /// Max milliseconds to block waiting for a change (default: 5000)
    pub timeout_ms: Option<u64>,
    /// Replay historical changes with seq > since, then stream live ones
    pub since: Option<u64>,
    /// Cap on number of events returned (default: 100)
    pub max_events: Option<u64>,
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
    /// Task ID for governance tracking (required when TERMLINK_TASK_GOVERNANCE=1)
    pub task_id: Option<String>,
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
    /// Default sequence number for all sessions when no per-session cursor is provided. Use this to replay history from a specific point without knowing session IDs.
    pub since_default: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DispatchParams {
    /// Number of workers to spawn (required, >= 1)
    pub count: u32,
    /// Command for each worker to execute (required)
    pub command: Vec<String>,
    /// Collection timeout in seconds (default: 120)
    pub timeout: Option<u64>,
    /// Event topic to collect (default: "task.completed")
    pub topic: Option<String>,
    /// Worker name prefix (default: "worker")
    pub name_prefix: Option<String>,
    /// Roles to assign to workers
    pub roles: Option<Vec<String>>,
    /// Tags to assign to workers
    pub tags: Option<Vec<String>>,
    /// Capabilities to advertise on workers (e.g., "code", "test")
    pub cap: Option<Vec<String>>,
    /// Environment variables to set in workers (map of KEY → VALUE)
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Working directory for workers (each worker cd's into this before executing)
    pub workdir: Option<String>,
    /// Task ID for governance tracking (required when TERMLINK_TASK_GOVERNANCE=1)
    pub task_id: Option<String>,
    /// LLM model for workers: "opus", "sonnet", or "haiku". When specified, passed as
    /// TERMLINK_MODEL env var to workers. If unavailable, falls back through the default
    /// chain (opus → sonnet → haiku).
    pub model: Option<String>,
    /// Task type label (e.g., "build", "test", "refactor") used to track per-model
    /// success rates in the route cache. When omitted and `model` is also omitted,
    /// no model selection learning is recorded.
    pub task_type: Option<String>,
}

/// Resolve a dispatch model decision: applies T-1590 model selection logic.
///
/// - If `requested` is Some: try it through the model circuit breaker; on open
///   circuit, walk the default fallback chain.
/// - If `requested` is None and `task_type` is Some: pick the best-known model
///   from the route cache for that task type, then resolve through the breaker.
/// - If both are None: returns (None, false) — caller skips model selection.
///
/// Returns `(effective_model, fallback_used)`.
pub(crate) fn resolve_dispatch_model(
    requested: Option<&str>,
    task_type: Option<&str>,
    cache: &termlink_hub::route_cache::RouteCache,
) -> (Option<String>, bool) {
    let mcb = termlink_hub::circuit_breaker::model_global();
    let chain = termlink_hub::circuit_breaker::DEFAULT_MODEL_FALLBACK;

    let preferred: Option<String> = match requested {
        Some(m) => Some(m.to_string()),
        None => task_type
            .and_then(|tt| cache.best_model_for(tt))
            .map(|s| s.to_string()),
    };

    match preferred {
        Some(p) => {
            let resolved = mcb.resolve_model(&p, chain);
            let fallback_used = matches!(&resolved, Some(r) if r != &p);
            (resolved, fallback_used)
        }
        None => (None, false),
    }
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
    /// Working directory for the command
    pub cwd: Option<String>,
    /// Environment variables to set for the command (map of KEY → VALUE)
    pub env: Option<std::collections::HashMap<String, String>>,
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

#[derive(Deserialize, JsonSchema)]
pub struct BatchRunParams {
    /// List of shell commands to execute in parallel ephemeral sessions
    pub commands: Vec<String>,
    /// Timeout per command in seconds (default: 30)
    pub timeout: Option<u64>,
    /// Working directory for all commands
    pub cwd: Option<String>,
    /// Environment variables for all commands (map of KEY → VALUE)
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Maximum parallel executions (default: 10)
    pub max_parallel: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct HelpParams {
    /// Filter by category: session, execution, events, kv, files, hub, batch, dispatch, tokens, diagnostics. Omit to see all.
    pub category: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RegisterParams {
    /// Display name for this endpoint (e.g., "my-agent")
    pub name: Option<String>,
    /// Roles this endpoint provides (e.g., ["coder", "reviewer"])
    pub roles: Option<Vec<String>>,
    /// Tags for discovery (e.g., ["team-a", "gpu"])
    pub tags: Option<Vec<String>>,
    /// Capabilities (e.g., ["events", "kv"])
    pub cap: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeregisterParams {
    /// Session ID of the endpoint to deregister
    pub session_id: String,
}

// T-1038: TOFU management params
#[derive(Deserialize, JsonSchema)]
pub struct TofuListParams {}

// T-1039: Fleet doctor params
#[derive(Deserialize, JsonSchema)]
pub struct FleetDoctorParams {
    /// Timeout per hub in seconds (default: 10)
    pub timeout: Option<u64>,
}

// T-1102: Fleet status params
#[derive(Deserialize, JsonSchema)]
pub struct FleetStatusParams {
    /// Timeout per hub in seconds (default: 10)
    pub timeout: Option<u64>,
}

// T-1106: Net test params
#[derive(Deserialize, JsonSchema)]
pub struct NetTestParams {
    /// Hub profile name to test (None = test all hubs)
    pub profile: Option<String>,
    /// Timeout per layer in seconds (default: 5)
    pub timeout: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TofuClearParams {
    /// Host:port to clear (e.g., "192.168.10.109:9100")
    pub host: String,
}

// T-1040: Resolve hub pidfile and socket, checking default runtime dir first,
// then /var/lib/termlink (systemd-managed hubs). Mirrors CLI's resolve_hub_paths().
fn resolve_hub_paths() -> (std::path::PathBuf, std::path::PathBuf) {
    let default_pidfile = termlink_hub::pidfile::hub_pidfile_path();
    let default_socket = termlink_hub::server::hub_socket_path();

    if matches!(
        termlink_hub::pidfile::check(&default_pidfile),
        termlink_hub::pidfile::PidfileStatus::Running(_) | termlink_hub::pidfile::PidfileStatus::Stale(_)
    ) {
        return (default_pidfile, default_socket);
    }

    if std::env::var("TERMLINK_RUNTIME_DIR").is_err() {
        let alt_dir = std::path::PathBuf::from("/var/lib/termlink");
        let alt_pidfile = alt_dir.join("hub.pid");
        if alt_pidfile.exists() {
            let alt_socket = alt_dir.join("hub.sock");
            return (alt_pidfile, alt_socket);
        }
    }

    (default_pidfile, default_socket)
}

// T-1040: Hub restart params
#[derive(Deserialize, JsonSchema)]
pub struct HubRestartParams {}

// T-1040: Events params
#[derive(Deserialize, JsonSchema)]
pub struct EventsParams {
    /// Session name or ID to query events from
    pub target: String,
    /// Only return events with sequence number > since
    pub since: Option<u64>,
    /// Filter events by topic name
    pub topic: Option<String>,
    /// Timeout in seconds (default: 5)
    pub timeout: Option<u64>,
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

// T-1060: Forward-compat with rmcp-macros 1.4+
//
// In rmcp-macros 1.3.0, `#[tool_handler]` expands to `self.tool_router` (FIELD
// access — the pub field on TermLinkTools). In 1.4.0+ it expands to
// `Self::tool_router()` (FUNCTION call — the method generated by this macro).
// By default `#[tool_router]` generates a PRIVATE method, which a caller in
// another impl block (server.rs's `impl ServerHandler`) cannot reach, so
// `cargo install --git` (which ignores Cargo.lock and may resolve rmcp=1.3.0
// paired with rmcp-macros=1.4.0 — caret on the transitive dep in rmcp's own
// Cargo.toml) fails with:
//   error[E0624]: associated function `tool_router` is private
//
// Explicit `vis = "pub(crate)"` makes the generated method callable from the
// impl block in server.rs, regardless of which rmcp-macros version resolves.
// Works under rmcp-macros 1.3.x (field-access path ignores the method) AND
// 1.4.x+ (function-call path requires the method to be reachable). See
// T-1056 (the earlier pin attempt) and T-1060 (this structural fix).
#[tool_router(vis = "pub(crate)")]
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
        if let Err(e) = check_task_governance(&p.task_id, "termlink_exec") {
            return e;
        }

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
        if let Some(env) = &p.env {
            params["env"] = serde_json::json!(env);
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
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "signal": result["signal"].as_i64().unwrap_or(sig_num as i64),
                    "pid": result["pid"].as_u64().unwrap_or(0),
                }))
                .unwrap_or_else(json_err),
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
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "topic": result["topic"].as_str().unwrap_or("?"),
                    "seq": result["seq"].as_u64().unwrap_or(0),
                }))
                .unwrap_or_else(json_err),
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
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "target": result["target"].as_str().unwrap_or("?"),
                    "topic": result["topic"].as_str().unwrap_or("?"),
                    "seq": result["seq"].as_u64().unwrap_or(0),
                }))
                .unwrap_or_else(json_err),
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
        if let Err(e) = check_task_governance(&p.task_id, "termlink_spawn") {
            return e;
        }

        let session_name = p.name.unwrap_or_else(|| format!("mcp-spawn-{}", std::process::id()));
        let roles = p.roles.unwrap_or_default();
        let mut tags = p.tags.unwrap_or_default();

        // Add task_id as a tag for observability
        if let Some(ref tid) = p.task_id {
            tags.push(format!("task:{tid}"));
        }
        let cap = p.cap.unwrap_or_default();
        let env_vars = p.env.unwrap_or_default();
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
        if !cap.is_empty() {
            register_args.push("--cap".to_string());
            register_args.push(cap.join(","));
        }
        if command.is_empty() {
            register_args.push("--shell".to_string());
        }

        // Build env prefix from user-supplied env vars
        let mut env_prefix = String::new();
        for (key, val) in &env_vars {
            env_prefix.push_str(&format!("export {}={}; ", shell_escape(key), shell_escape(val)));
        }

        let cd_prefix = if let Some(ref wd) = p.cwd {
            format!("cd {} && ", shell_escape(wd))
        } else {
            String::new()
        };

        let shell_cmd = if command.is_empty() {
            let mut parts = vec![termlink_bin];
            parts.extend(register_args);
            format!("{cd_prefix}{env_prefix}{}", parts.join(" "))
        } else {
            let mut reg_parts = vec![termlink_bin];
            reg_parts.extend(register_args);
            let user_cmd = command.join(" ");
            format!(
                "{cd_prefix}{env_prefix}{} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nkill $TL_PID 2>/dev/null\nwait $TL_PID 2>/dev/null",
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
                    return serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "session_name": session_name,
                        "status": "ready",
                    }))
                    .unwrap_or_else(json_err);
                }
                if start.elapsed() > timeout {
                    return serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "session_name": session_name,
                        "status": "timeout",
                        "message": "spawned but timed out waiting for registration",
                    }))
                    .unwrap_or_else(json_err);
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "session_name": session_name,
            "status": "spawned",
        }))
        .unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_run",
        description = "Execute a command in an ephemeral TermLink session and return the output. The session is cleaned up after execution."
    )]
    async fn termlink_run(&self, Parameters(p): Parameters<RunParams>) -> String {
        use termlink_session::executor;

        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(30));
        let env = p.env.unwrap_or_default();
        let env_ref = if env.is_empty() { None } else { Some(&env) };

        match executor::execute(&p.command, p.cwd.as_deref(), env_ref, Some(timeout), None).await {
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
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "key": result["key"].as_str().unwrap_or("?"),
                        "replaced": replaced,
                    }))
                    .unwrap_or_else(json_err)
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
                    let found = result["found"].as_bool().unwrap_or(false);
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "key": p.key,
                        "found": found,
                        "value": if found { result["value"].clone() } else { serde_json::Value::Null },
                    }))
                    .unwrap_or_else(json_err)
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
                    let deleted = result["deleted"].as_bool().unwrap_or(false);
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "key": p.key,
                        "deleted": deleted,
                    }))
                    .unwrap_or_else(json_err)
                }
                Err(e) => json_err(e),
            },
            Err(e) => json_err(format!("connection failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_kv_watch",
        description = "Watch for key-value changes on a session. Blocks until a kv.set or kv.delete occurs, or timeout_ms elapses. Reuses event.subscribe with topic=kv.change. Optional 'since' replays historical changes before streaming live ones."
    )]
    async fn termlink_kv_watch(&self, Parameters(p): Parameters<KvWatchParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("session '{}' not found: {e}", p.target)),
        };

        let mut params = serde_json::json!({ "topic": "kv.change" });
        if let Some(timeout_ms) = p.timeout_ms {
            params["timeout_ms"] = serde_json::json!(timeout_ms);
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
        name = "termlink_broadcast",
        description = "Broadcast an event to multiple TermLink sessions via the hub. If no targets specified, broadcasts to all (via channel.post broadcast:global per T-1401/T-1403). With explicit targets, fans out via parallel event.emit_to per T-1417 (replacement for retiring legacy event.broadcast)."
    )]
    async fn termlink_broadcast(&self, Parameters(p): Parameters<BroadcastParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("hub is not running. Start it with: termlink hub");
        }

        let payload = p.payload.unwrap_or(serde_json::json!({}));
        let targets_empty = p.targets.as_ref().is_none_or(|t| t.is_empty());

        // T-1403: prefer channel.post(broadcast:global) when no targets specified
        // (the dominant case). Mirrors hub-side T-1162 envelope shape.
        if targets_empty {
            return match Self::try_broadcast_via_channel_post(&hub_socket, &p.topic, &payload).await {
                Ok(offset) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "topic": p.topic,
                    "channel_topic": "broadcast:global",
                    "offset": offset,
                    "targeted": 1,
                    "succeeded": 1,
                    "failed": 0,
                }))
                .unwrap_or_else(json_err),
                Err(e) => json_err(format!(
                    "channel.post(broadcast:global) failed and event.broadcast is retiring (T-1166): {e}"
                )),
            };
        }

        // T-1417: per-target fan-out via parallel event.emit_to. Replaces the
        // legacy event.broadcast --targets dispatch (retiring under T-1166).
        // Same response shape so downstream consumers don't need to change.
        let targets = p.targets.clone().unwrap_or_default();
        let (targeted, succeeded, failed, errors) =
            Self::broadcast_via_emit_to_fanout(&hub_socket, &p.topic, &payload, &targets).await;

        let mut wrapped = serde_json::json!({
            "ok": failed == 0,
            "topic": p.topic,
            "targeted": targeted,
            "succeeded": succeeded,
            "failed": failed,
        });
        if !errors.is_empty() {
            wrapped["errors"] = serde_json::json!(errors);
        }
        serde_json::to_string_pretty(&wrapped).unwrap_or_else(json_err)
    }

    /// T-1417: Parallel `event.emit_to` fanout — MCP-side mirror of the
    /// CLI helper in `crates/termlink-cli/src/commands/events.rs`. Each
    /// target gets its own RPC, issued concurrently. Per-target failures
    /// are aggregated into `errors`, not propagated as a hard error.
    async fn broadcast_via_emit_to_fanout(
        hub_socket: &std::path::Path,
        topic: &str,
        payload: &serde_json::Value,
        targets: &[String],
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
                (
                    target_owned,
                    client::rpc_call(&socket, "event.emit_to", params).await,
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
                Ok((target, Ok(resp))) => match client::unwrap_result(resp) {
                    Ok(_) => succeeded += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", target, e));
                    }
                },
                Ok((target, Err(e))) => {
                    failed += 1;
                    errors.push(format!("{}: connection: {}", target, e));
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("(join error): {}", e));
                }
            }
        }

        (targeted, succeeded, failed, errors)
    }

    /// T-1403: Sign and dispatch a `channel.post(broadcast:global)` envelope
    /// matching the hub-side T-1162 mirror shape exactly. Returns offset on
    /// success, or any error (caller falls back to legacy event.broadcast).
    async fn try_broadcast_via_channel_post(
        hub_socket: &std::path::Path,
        topic: &str,
        payload: &serde_json::Value,
    ) -> Result<i64, String> {
        const BROADCAST_GLOBAL_TOPIC: &str = "broadcast:global";

        let payload_bytes = serde_json::to_vec(payload)
            .map_err(|e| format!("payload serialize: {e}"))?;
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = termlink_session::agent_identity::Identity::load_or_create(&identity_dir)
            .map_err(|e| format!("identity load: {e}"))?;
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            BROADCAST_GLOBAL_TOPIC,
            topic,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let mut params = serde_json::json!({
            "topic": BROADCAST_GLOBAL_TOPIC,
            "msg_type": topic,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": identity.fingerprint(),
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
        });
        if let Ok(sid) = std::env::var("TERMLINK_SESSION_ID")
            && !sid.is_empty()
        {
            params["metadata"] = serde_json::json!({"from": sid});
        }
        let resp = client::rpc_call(
            hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        .map_err(|e| format!("channel.post connect: {e}"))?;
        let result = client::unwrap_result(resp)
            .map_err(|e| format!("channel.post error: {e}"))?;
        result["offset"]
            .as_i64()
            .ok_or_else(|| "channel.post response missing offset".to_string())
    }

    #[tool(
        name = "termlink_interact",
        description = "Run a shell command in a PTY session and return its output. Injects the command, waits for completion via a unique marker, and returns clean output with exit code. This is the preferred tool for running commands in terminal sessions — it handles injection, waiting, and output capture atomically."
    )]
    async fn termlink_interact(&self, Parameters(p): Parameters<InteractParams>) -> String {
        if let Err(e) = check_task_governance(&p.task_id, "termlink_interact") {
            return e;
        }

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
        let hub_secret_path = termlink_hub::server::hub_secret_path();
        let hub_has_tcp = hub_secret_path.exists();
        let transport = if hub_has_tcp { "unix+tcp" } else { "unix" };
        match termlink_hub::pidfile::check(&pidfile_path) {
            termlink_hub::pidfile::PidfileStatus::Running(pid) => {
                match client::rpc_call(&hub_socket, "termlink.ping", serde_json::json!({})).await {
                    Ok(_) => check!("hub", pass, format!("running (PID {pid}), responding, transport={transport}")),
                    Err(_) => check!("hub", warn, format!("running (PID {pid}), but not responding (transport={transport})")),
                }
            }
            termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
                check!("hub", warn, format!("stale pidfile (PID {pid} is dead)"));
            }
            termlink_hub::pidfile::PidfileStatus::NotRunning => {
                check!("hub", pass, "not running (use termlink_hub_start with tcp_addr=\"0.0.0.0:9100\" for cross-host)");
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
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "cols": result["cols"].as_u64().unwrap_or(p.cols as u64),
                    "rows": result["rows"].as_u64().unwrap_or(p.rows as u64),
                }))
                .unwrap_or_else(json_err),
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
                return json_err(format!(
                    "timeout: no reply on '{}' within {}s (request_id: {})",
                    p.reply_topic, timeout_secs, request_id
                ));
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
                                    return serde_json::to_string_pretty(&serde_json::json!({
                                        "ok": true,
                                        "request_id": request_id,
                                        "reply_topic": p.reply_topic,
                                        "response": event_payload,
                                    }))
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
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "target": result["display_name"].as_str().unwrap_or(&p.target),
                    "tags": result["tags"],
                    "roles": result["roles"],
                }))
                .unwrap_or_else(json_err),
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

        // If since is provided, start from that sequence; otherwise poll for existing events
        let mut cursor: Option<u64> = if p.since.is_some() {
            p.since
        } else {
            let params = serde_json::json!({"topic": p.topic});
            match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                Ok(resp) => {
                    if let Ok(result) = client::unwrap_result(resp) {
                        // Check if matching event already exists
                        if let Some(events) = result["events"].as_array()
                            && let Some(event) = events.first() {
                                return serde_json::to_string_pretty(&serde_json::json!({
                                    "ok": true,
                                    "topic": p.topic,
                                    "event": event,
                                }))
                                .unwrap_or_else(json_err);
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
                return json_err(format!(
                    "timeout waiting for event topic '{}' ({}s)",
                    p.topic, timeout_secs
                ));
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
                                return serde_json::to_string_pretty(&serde_json::json!({
                                    "ok": true,
                                    "topic": p.topic,
                                    "event": event,
                                }))
                                .unwrap_or_else(json_err);
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
            Err(e) => return json_err(format!("failed to get current directory: {e}")),
        };
        let manifest_path = project_root.join(".termlink").join("dispatch-manifest.json");

        if !manifest_path.exists() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "total": 0,
                "message": "No dispatch manifest (no dispatches have used --isolate yet)"
            }))
            .unwrap_or_else(json_err);
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => return json_err(format!("failed to read dispatch manifest: {e}")),
        };

        let manifest: serde_json::Value = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(e) => return json_err(format!("failed to parse dispatch manifest: {e}")),
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
        name = "termlink_dispatch",
        description = "Atomic multi-worker dispatch: spawns N background workers, tags them with a dispatch ID, and collects results via the hub. Each worker runs the specified command. Returns structured results from all workers. Requires the hub to be running. Use this for fan-out/fan-in orchestration patterns where you need multiple sessions to work in parallel and collect their results."
    )]
    async fn termlink_dispatch(&self, Parameters(p): Parameters<DispatchParams>) -> String {
        if let Err(e) = check_task_governance(&p.task_id, "termlink_dispatch") {
            return e;
        }

        // Validate inputs
        if p.count == 0 {
            return json_err("count must be at least 1");
        }
        if p.command.is_empty() {
            return json_err("command is required");
        }

        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running. Start it with: termlink hub start (dispatch requires the hub for event collection)");
        }

        let termlink_bin = match std::env::current_exe() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => return json_err(format!("cannot determine termlink binary: {e}")),
        };

        let count = p.count;
        let timeout = p.timeout.unwrap_or(120);
        let topic = p.topic.unwrap_or_else(|| "task.completed".into());
        let prefix = p.name_prefix.unwrap_or_else(|| "worker".into());
        let roles = p.roles.unwrap_or_default();
        let mut tags = p.tags.unwrap_or_default();
        let cap = p.cap.unwrap_or_default();
        let env_vars = p.env.unwrap_or_default();
        let workdir = p.workdir;
        let task_type = p.task_type.clone();

        // T-1590: resolve effective model via circuit breaker + route cache.
        let mut route_cache = termlink_hub::route_cache::RouteCache::load();
        let (effective_model, fallback_used) = resolve_dispatch_model(
            p.model.as_deref(),
            task_type.as_deref(),
            &route_cache,
        );
        let model_requested = p.model.clone();
        let model_used = effective_model.clone();

        // Add task_id as a tag for observability
        if let Some(ref tid) = p.task_id {
            tags.push(format!("task:{tid}"));
        }

        // Generate unique dispatch ID
        let dispatch_id = format!(
            "D-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        // Spawn N workers
        let mut worker_names = Vec::with_capacity(count as usize);
        let mut spawn_errors: Vec<String> = Vec::new();

        for i in 1..=count {
            let worker_name = format!("{prefix}-{i}");
            worker_names.push(worker_name.clone());

            let mut worker_tags = tags.clone();
            worker_tags.push(format!("_dispatch.id:{dispatch_id}"));
            worker_tags.push(format!("_dispatch.worker:{i}"));

            let mut register_args = vec![
                "register".to_string(),
                "--name".to_string(),
                worker_name.clone(),
                "--tags".to_string(),
                worker_tags.join(","),
            ];
            if !roles.is_empty() {
                register_args.push("--roles".to_string());
                register_args.push(roles.join(","));
            }
            if !cap.is_empty() {
                register_args.push("--cap".to_string());
                register_args.push(cap.join(","));
            }

            let raw_cmd = p.command.iter()
                .map(|arg| shell_escape(arg))
                .collect::<Vec<_>>()
                .join(" ");
            let user_cmd = if let Some(ref wd) = workdir {
                format!("cd {} && {}", shell_escape(wd), raw_cmd)
            } else {
                raw_cmd
            };

            let mut env_prefix = String::new();
            if let Ok(rd) = std::env::var("TERMLINK_RUNTIME_DIR") {
                env_prefix.push_str(&format!("export TERMLINK_RUNTIME_DIR={}; ", shell_escape(&rd)));
            }
            env_prefix.push_str(&format!("export TERMLINK_DISPATCH_ID={}; ", shell_escape(&dispatch_id)));
            env_prefix.push_str(&format!("export TERMLINK_ORCHESTRATOR={}; ", std::process::id()));
            env_prefix.push_str(&format!("export TERMLINK_WORKER_NAME={}; ", shell_escape(&worker_name)));
            // Model selection (T-1590: use effective model after fallback chain)
            if let Some(ref model) = effective_model {
                env_prefix.push_str(&format!("export TERMLINK_MODEL={}; ", shell_escape(model)));
            }
            // User-supplied env vars
            for (key, val) in &env_vars {
                env_prefix.push_str(&format!("export {}={}; ", shell_escape(key), shell_escape(val)));
            }

            let mut reg_parts = vec![termlink_bin.clone()];
            reg_parts.extend(register_args);

            let shell_cmd = format!(
                "{env_prefix}{} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nwait $TL_PID",
                reg_parts.join(" ")
            );

            match std::process::Command::new("setsid")
                .args(["sh", "-c", &shell_cmd])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("sh")
                        .args(["-c", &shell_cmd])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .stdin(std::process::Stdio::null())
                        .spawn()
                }) {
                Ok(_) => {}
                Err(e) => spawn_errors.push(format!("{worker_name}: {e}")),
            }
        }

        if spawn_errors.len() == count as usize {
            return json_err(format!("All workers failed to spawn: {}", spawn_errors.join("; ")));
        }

        // Wait for workers to register
        let register_timeout = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();
        let mut registered = vec![false; count as usize];

        loop {
            if registered.iter().all(|r| *r) {
                break;
            }
            if start.elapsed() > register_timeout {
                break;
            }
            for (i, name) in worker_names.iter().enumerate() {
                if !registered[i] && manager::find_session(name).is_ok() {
                    registered[i] = true;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }

        let registered_count = registered.iter().filter(|r| **r).count() as u64;

        // Collect events via hub
        let collect_timeout = std::time::Duration::from_secs(timeout);
        let subscribe_timeout_ms: u64 = 500;
        let collect_start = std::time::Instant::now();
        let mut cursors = serde_json::json!({});
        let mut collected_events: Vec<serde_json::Value> = Vec::new();
        let mut crashed_workers: Vec<String> = Vec::new();

        loop {
            if collected_events.len() as u64 >= registered_count {
                break;
            }
            if collect_start.elapsed() > collect_timeout {
                break;
            }

            let mut params = serde_json::json!({
                "topic": topic,
                "timeout_ms": subscribe_timeout_ms,
            });
            let target_names: Vec<&str> = worker_names
                .iter()
                .zip(registered.iter())
                .filter(|(_, r)| **r)
                .map(|(n, _)| n.as_str())
                .collect();
            if !target_names.is_empty() {
                params["targets"] = serde_json::json!(target_names);
            }
            if !cursors.as_object().unwrap_or(&serde_json::Map::new()).is_empty() {
                params["since"] = cursors.clone();
            }

            let resp = match client::rpc_call(&hub_socket, "event.collect", params).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            if let Ok(result) = client::unwrap_result(resp) {
                if let Some(events) = result["events"].as_array() {
                    for event in events {
                        let session_name = event["session_name"]
                            .as_str()
                            .unwrap_or("?")
                            .to_string();
                        collected_events.push(serde_json::json!({
                            "worker": session_name,
                            "payload": event["payload"],
                            "seq": event["seq"],
                            "timestamp": event["timestamp"],
                        }));
                    }
                }

                let has_events = result["events"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty());
                if has_events
                    && let Some(new_cursors) = result.get("cursors")
                    && let Some(obj) = new_cursors.as_object()
                {
                    for (k, v) in obj {
                        cursors[k] = v.clone();
                    }
                }
            }

            // Early crash detection
            let mut alive_remaining = 0u64;
            for (i, name) in worker_names.iter().enumerate() {
                if !registered[i] { continue; }
                let has_result = collected_events.iter().any(|e| e["worker"].as_str() == Some(name.as_str()));
                let already_dead = crashed_workers.iter().any(|d| d == name);
                if has_result || already_dead { continue; }
                if manager::find_session(name).is_err() {
                    crashed_workers.push(name.clone());
                } else {
                    alive_remaining += 1;
                }
            }
            if !crashed_workers.is_empty() && alive_remaining == 0 {
                break;
            }
        }

        // Cleanup: signal workers to exit
        for name in &worker_names {
            if let Ok(reg) = manager::find_session(name) {
                unsafe { libc::kill(reg.pid as i32, libc::SIGTERM); }
            }
        }

        // Build result
        let collected_count = collected_events.len() as u64;
        let timed_out = collected_count < registered_count;
        let total_elapsed = collect_start.elapsed().as_secs_f64();

        // T-1590: record per-worker outcomes against model + circuit breaker.
        if let (Some(m), Some(tt)) = (&model_used, &task_type) {
            let mcb = termlink_hub::circuit_breaker::model_global();
            let success_workers = collected_events.iter().filter(|e| {
                e["payload"]["ok"].as_bool().unwrap_or(true)
            }).count();
            let failure_workers = collected_events.len() - success_workers + crashed_workers.len();
            for _ in 0..success_workers {
                route_cache.record_model_success(m, tt);
                mcb.record_success(m);
            }
            for _ in 0..failure_workers {
                route_cache.record_model_failure(m, tt);
                mcb.record_failure(m);
            }
            // Persist updates; ignore I/O errors (cache is best-effort).
            let _ = route_cache.save();
        } else if let Some(ref m) = model_used {
            // No task_type, but we can still update the breaker on hard crashes.
            let mcb = termlink_hub::circuit_breaker::model_global();
            for _ in 0..crashed_workers.len() {
                mcb.record_failure(m);
            }
            for _ in 0..(collected_events.len()) {
                mcb.record_success(m);
            }
        }

        let mut result = serde_json::json!({
            "ok": !timed_out && crashed_workers.is_empty(),
            "dispatch_id": dispatch_id,
            "workers_spawned": count,
            "workers_registered": registered_count,
            "events_collected": collected_count,
            "timed_out": timed_out,
            "elapsed_secs": (total_elapsed * 10.0).round() / 10.0,
            "topic": topic,
            "results": collected_events,
        });
        // T-1590: surface model decision into the dispatch manifest/response.
        if let Some(ref m) = model_requested {
            result["model_requested"] = serde_json::json!(m);
        }
        if let Some(ref m) = model_used {
            result["model_used"] = serde_json::json!(m);
        }
        if model_requested.is_some() || model_used.is_some() {
            result["fallback_used"] = serde_json::json!(fallback_used);
        }
        if let Some(ref tt) = task_type {
            result["task_type"] = serde_json::json!(tt);
        }
        if !spawn_errors.is_empty() {
            result["spawn_errors"] = serde_json::json!(spawn_errors);
        }
        if !crashed_workers.is_empty() {
            result["crashed_workers"] = serde_json::json!(crashed_workers);
        }

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
        let registered_endpoints = self.endpoints.lock().await.len();
        let mcp_tools = crate::tool_count();

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
            "mcp_tools": mcp_tools,
            "registered_endpoints": registered_endpoints,
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
        if let Some(since_default) = p.since_default {
            params["since_default"] = serde_json::json!(since_default);
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

        // T-1249: Try the new channel.post + artifact.put path against the local
        // hub. On LegacyOnly (older hub) or no hub socket, fall through to the
        // 3-phase event-emit below directly to the target session.
        {
            use termlink_session::artifact::{
                send_artifact_via_client, ArtifactManifest, SendOutcome, SendPath,
            };
            use termlink_session::hub_capabilities::shared_cache;
            use termlink_session::inbox_channel::FallbackCtx;
            use termlink_protocol::TransportAddr;

            let hub_socket = termlink_hub::server::hub_socket_path();
            if hub_socket.exists() {
                let identity_base = std::env::var("HOME")
                    .ok()
                    .map(|h| std::path::PathBuf::from(h).join(".termlink"));
                let identity = match identity_base {
                    Some(base) => match termlink_session::agent_identity::Identity::load_or_create(&base) {
                        Ok(id) => Some(id),
                        Err(e) => {
                            tracing::warn!(error = %e, "T-1249: identity load failed — using legacy path");
                            None
                        }
                    },
                    None => None,
                };
                if let Some(identity) = identity {
                    let addr = TransportAddr::unix(&hub_socket);
                    match termlink_session::client::Client::connect_addr(&addr).await {
                        Ok(mut client) => {
                            let cache = shared_cache();
                            let mut ctx = FallbackCtx::new();
                            let manifest = ArtifactManifest {
                                filename: filename.clone(),
                                size,
                                from: format!("mcp-{}", std::process::id()),
                                transfer_id: Some(transfer_id.clone()),
                                content_type: None,
                            };
                            let host_port = format!("local:{}", hub_socket.display());
                            match send_artifact_via_client(
                                &mut client,
                                &host_port,
                                &p.target,
                                &file_data,
                                &manifest,
                                &identity,
                                cache,
                                &mut ctx,
                            )
                            .await
                            {
                                Ok(SendOutcome::Sent {
                                    channel_offset,
                                    path: used_path,
                                    ..
                                }) => {
                                    let path_label = match used_path {
                                        SendPath::Inline => "channel.inline",
                                        SendPath::Chunked => "channel.artifact",
                                    };
                                    let response = serde_json::json!({
                                        "ok": true,
                                        "target": p.target,
                                        "filename": filename,
                                        "size": size,
                                        "chunks": total_chunks,
                                        "transfer_id": transfer_id,
                                        "sha256": sha256,
                                        "via": path_label,
                                        "channel_offset": channel_offset,
                                        "artifact_sha256": sha256,
                                    });
                                    return serde_json::to_string_pretty(&response)
                                        .unwrap_or_else(json_err);
                                }
                                Ok(SendOutcome::LegacyOnly) => {
                                    tracing::debug!(
                                        target = %p.target,
                                        "T-1249: hub doesn't advertise artifact.put — using legacy events"
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        target = %p.target,
                                        error = %e,
                                        "T-1249: new-path send failed — using legacy events"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "T-1249: hub connect failed — using legacy events");
                        }
                    }
                }
            }
        }

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

        // T-1250: Try the new channel.subscribe + artifact.get path first.
        // Single-shot probe (no waiting): if the hub has a pending artifact for
        // this target, consume it. On LegacyOnly or empty, fall through to the
        // legacy event-stream reassembly below.
        {
            use termlink_session::artifact::{
                download_artifact_via_client, recv_artifacts_via_client, RecvOutcome,
            };
            use termlink_session::hub_capabilities::shared_cache;
            use termlink_session::inbox_channel::FallbackCtx;
            use termlink_protocol::TransportAddr;

            let hub_socket = termlink_hub::server::hub_socket_path();
            if hub_socket.exists() {
                let addr = TransportAddr::unix(&hub_socket);
                match termlink_session::client::Client::connect_addr(&addr).await {
                    Ok(mut client) => {
                        let cache = shared_cache();
                        let mut ctx = FallbackCtx::new();
                        let host_port = format!("local:{}", hub_socket.display());
                        match recv_artifacts_via_client(
                            &mut client, &host_port, &p.target, 0, cache, &mut ctx,
                        )
                        .await
                        {
                            Ok(RecvOutcome::Received { artifacts, .. }) => {
                                if let Some(a) = artifacts.into_iter().next() {
                                    let result: Result<(Vec<u8>, String, String, &'static str), String> =
                                        if let Some(sha) = a.artifact_ref.clone() {
                                            let manifest_filename = a
                                                .manifest
                                                .as_ref()
                                                .map(|m| m.filename.clone())
                                                .unwrap_or_else(|| {
                                                    format!(
                                                        "received-{}.bin",
                                                        &sha[..16.min(sha.len())]
                                                    )
                                                });
                                            // Idempotency: matching sha at dest? skip download.
                                            let dest = out_path.join(&manifest_filename);
                                            if dest.exists()
                                                && let Ok(existing) = std::fs::read(&dest)
                                            {
                                                let mut h = Sha256::new();
                                                h.update(&existing);
                                                if format!("{:x}", h.finalize()) == sha {
                                                    Ok((
                                                        existing,
                                                        sha.clone(),
                                                        manifest_filename,
                                                        "channel.artifact.skip-existing",
                                                    ))
                                                } else {
                                                    download_artifact_via_client(&mut client, &sha)
                                                        .await
                                                        .map(|b| (b, sha.clone(), manifest_filename, "channel.artifact"))
                                                        .map_err(|e| e.to_string())
                                                }
                                            } else {
                                                download_artifact_via_client(&mut client, &sha)
                                                    .await
                                                    .map(|b| (b, sha.clone(), manifest_filename, "channel.artifact"))
                                                    .map_err(|e| e.to_string())
                                            }
                                        } else {
                                            let mut h = Sha256::new();
                                            h.update(&a.payload);
                                            let computed = format!("{:x}", h.finalize());
                                            let filename = format!(
                                                "received-{}.bin",
                                                &computed[..16.min(computed.len())]
                                            );
                                            Ok((a.payload.clone(), computed, filename, "channel.inline"))
                                        };
                                    match result {
                                        Ok((bytes, sha256_hex, filename, via)) => {
                                            let dest = out_path.join(&filename);
                                            if let Err(e) = std::fs::write(&dest, &bytes) {
                                                return json_err(format!(
                                                    "failed to write file '{}': {e}",
                                                    dest.display()
                                                ));
                                            }
                                            let response = serde_json::json!({
                                                "ok": true,
                                                "target": p.target,
                                                "filename": filename,
                                                "path": dest.display().to_string(),
                                                "size": bytes.len(),
                                                "sha256": sha256_hex,
                                                "via": via,
                                            });
                                            return serde_json::to_string_pretty(&response)
                                                .unwrap_or_else(json_err);
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                target = %p.target,
                                                error = %e,
                                                "T-1250: new-path artifact processing failed — using legacy events"
                                            );
                                        }
                                    }
                                }
                            }
                            Ok(RecvOutcome::LegacyOnly) => {
                                tracing::debug!(
                                    target = %p.target,
                                    "T-1250: hub doesn't advertise channel.subscribe — using legacy events"
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    target = %p.target,
                                    error = %e,
                                    "T-1250: new-path receive failed — using legacy events"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "T-1250: hub connect failed — using legacy events");
                    }
                }
            }
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
        description = "Start the hub server in the background. The hub enables multi-session features like collect, broadcast, and discover. Returns immediately with hub pid, socket path, and (when tcp_addr is set) TCP bind address. No-op if hub is already running. Pass tcp_addr (e.g. \"0.0.0.0:9100\") to enable cross-host RPC via TOFU TLS — required for termlink_remote_* tools to reach this hub from another host."
    )]
    async fn termlink_hub_start(&self, Parameters(p): Parameters<HubStartParams>) -> String {
        let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
        let socket_path = termlink_hub::server::hub_socket_path();

        // Check if already running
        if let termlink_hub::pidfile::PidfileStatus::Running(pid) = termlink_hub::pidfile::check(&pidfile_path) {
            let response = serde_json::json!({
                "ok": true,
                "action": "already_running",
                "pid": pid,
                "socket": socket_path.display().to_string(),
                "note": "Hub already running — tcp_addr (if provided) was not applied. Stop and restart if you need to change transport.",
            });
            return serde_json::to_string_pretty(&response).unwrap_or_else(json_err);
        }

        let tcp_addr = p.tcp_addr.as_deref();
        match termlink_hub::server::run_with_tcp(&socket_path, tcp_addr).await {
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
                    "tcp_addr": tcp_addr,
                    "transport": if tcp_addr.is_some() { "unix+tcp" } else { "unix" },
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
                return json_err(format!(
                    "timeout: no agent response within {}s (action: {}, request_id: {})",
                    timeout_secs, p.action, request_id
                ));
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
        name = "termlink_help",
        description = "List available TermLink MCP tools organized by category. Use this to discover what operations are available. Optionally filter by category: session, execution, events, kv, files, hub, batch, dispatch, tokens, diagnostics."
    )]
    async fn termlink_help(&self, Parameters(p): Parameters<HelpParams>) -> String {
        let categories: Vec<(&str, Vec<(&str, &str)>)> = vec![
            ("session", vec![
                ("termlink_list_sessions", "List registered sessions with filtering"),
                ("termlink_ping", "Ping a session to check liveness"),
                ("termlink_status", "Get detailed session status"),
                ("termlink_discover", "Find sessions by tags/roles/capabilities"),
                ("termlink_spawn", "Spawn a new session in the background"),
                ("termlink_run", "Execute command in ephemeral session"),
                ("termlink_register", "Register as a discoverable endpoint"),
                ("termlink_deregister", "Deregister a previously registered endpoint"),
                ("termlink_clean", "Remove stale session registrations"),
                ("termlink_tag", "Update tags/roles on a session"),
                ("termlink_overview", "Aggregated system overview"),
            ]),
            ("execution", vec![
                ("termlink_exec", "Execute command on a session"),
                ("termlink_interact", "Interactive command execution with stdin"),
                ("termlink_signal", "Send signal to a session"),
            ]),
            ("events", vec![
                ("termlink_emit", "Emit event on a session"),
                ("termlink_emit_to", "Emit event to a target session"),
                ("termlink_broadcast", "Broadcast event to all sessions"),
                ("termlink_event_poll", "Poll session event bus"),
                ("termlink_event_subscribe", "Subscribe to session events (long-poll)"),
                ("termlink_wait", "Wait for specific event topic"),
                ("termlink_collect", "Collect events from multiple sessions via hub"),
                ("termlink_topics", "List event topics on a session"),
            ]),
            ("kv", vec![
                ("termlink_kv_set", "Set key-value on session store"),
                ("termlink_kv_get", "Get value from session store"),
                ("termlink_kv_list", "List all keys in session store"),
                ("termlink_kv_del", "Delete key from session store"),
                ("termlink_kv_watch", "Watch for key-value changes (long-poll)"),
            ]),
            ("files", vec![
                ("termlink_file_send", "Send file to a session"),
                ("termlink_file_receive", "Receive file from a session"),
            ]),
            ("hub", vec![
                ("termlink_hub_status", "Check hub running status"),
                ("termlink_hub_start", "Start the event hub (pass tcp_addr for cross-host)"),
                ("termlink_hub_stop", "Stop the event hub"),
            ]),
            ("remote", vec![
                ("termlink_remote_call", "Generic JSON-RPC call to a remote hub (cross-host)"),
                ("termlink_remote_ping", "Ping a remote hub or session (cross-host)"),
                ("termlink_remote_inject", "Inject text into a session on a remote hub (cross-host)"),
            ]),
            ("batch", vec![
                ("termlink_batch_exec", "Run command across multiple sessions"),
                ("termlink_batch_ping", "Ping multiple sessions"),
                ("termlink_batch_tag", "Tag/role operations across sessions"),
                ("termlink_batch_run", "Run commands in parallel ephemeral sessions"),
            ]),
            ("dispatch", vec![
                ("termlink_dispatch", "Atomic spawn+tag+collect for N workers"),
                ("termlink_dispatch_status", "Check dispatch manifest status"),
            ]),
            ("tokens", vec![
                ("termlink_token_create", "Create authentication token"),
                ("termlink_token_inspect", "Inspect token contents"),
            ]),
            ("diagnostics", vec![
                ("termlink_info", "Runtime info and paths"),
                ("termlink_doctor", "Health check"),
                ("termlink_version", "Version and build info"),
                ("termlink_pty_mode", "Query terminal mode"),
                ("termlink_output", "Read PTY output"),
                ("termlink_inject", "Inject text into PTY"),
                ("termlink_resize", "Resize PTY terminal"),
                ("termlink_request", "Request-reply pattern"),
                ("termlink_agent_ask", "Ask an agent session"),
                ("termlink_send", "Send raw JSON-RPC"),
            ]),
        ];

        let filter = p.category.as_deref();
        let mut result = serde_json::json!({});
        let mut tool_count = 0;

        for (cat_name, tools) in &categories {
            if let Some(f) = filter && *cat_name != f {
                continue;
            }
            let tools_json: Vec<serde_json::Value> = tools.iter()
                .map(|(name, desc)| serde_json::json!({"name": name, "description": desc}))
                .collect();
            tool_count += tools_json.len();
            result[cat_name] = serde_json::json!(tools_json);
        }

        if filter.is_some() && tool_count == 0 {
            return json_err(format!("Unknown category '{}'. Available: session, execution, events, kv, files, hub, remote, batch, dispatch, tokens, diagnostics", filter.unwrap()));
        }

        result["total_tools"] = serde_json::json!(tool_count);
        serde_json::to_string_pretty(&result).unwrap_or_else(json_err)
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
        let cwd = std::sync::Arc::new(p.cwd);
        let env = std::sync::Arc::new(p.env);

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
            let cwd = cwd.clone();
            let env = env.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let mut params = serde_json::json!({
                    "command": cmd,
                    "timeout": timeout,
                });
                if let Some(ref dir) = *cwd {
                    params["cwd"] = serde_json::json!(dir);
                }
                if let Some(ref env_map) = *env {
                    params["env"] = serde_json::json!(env_map);
                }
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

    #[tool(
        name = "termlink_batch_run",
        description = "Execute multiple commands in parallel ephemeral sessions. Each command runs in its own isolated session and results are collected. Useful for parallelizing independent tasks like running tests in different directories, checking multiple repos, or executing independent build steps."
    )]
    async fn termlink_batch_run(&self, Parameters(p): Parameters<BatchRunParams>) -> String {
        use termlink_session::executor;

        if p.commands.is_empty() {
            return json_err("commands list is empty");
        }

        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(30));
        let max_parallel = p.max_parallel.unwrap_or(10);
        let env = p.env.unwrap_or_default();
        let cwd = p.cwd;

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_parallel));
        let env = std::sync::Arc::new(env);
        let cwd = std::sync::Arc::new(cwd);
        let mut handles = Vec::new();

        for (i, command) in p.commands.iter().enumerate() {
            let sem = semaphore.clone();
            let cmd = command.clone();
            let task_timeout = timeout;
            let env = env.clone();
            let cwd = cwd.clone();

            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let env_ref = if env.is_empty() { None } else { Some(env.as_ref()) };
                match executor::execute(&cmd, cwd.as_deref(), env_ref, Some(task_timeout), None).await {
                    Ok(result) => serde_json::json!({
                        "index": i,
                        "command": cmd,
                        "ok": result.exit_code == 0,
                        "exit_code": result.exit_code,
                        "stdout": result.stdout,
                        "stderr": result.stderr,
                    }),
                    Err(e) => serde_json::json!({
                        "index": i,
                        "command": cmd,
                        "ok": false,
                        "error": e.to_string(),
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

        // Sort by index to maintain command order
        results.sort_by_key(|r| r["index"].as_u64().unwrap_or(0));

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
        name = "termlink_register",
        description = "Register this MCP server as a TermLink endpoint — makes it discoverable via termlink list/discover and able to receive events, KV operations, and status queries. The endpoint runs in the background for the lifetime of the MCP server. Returns the session ID immediately."
    )]
    async fn termlink_register(&self, Parameters(p): Parameters<RegisterParams>) -> String {
        use termlink_session::registration::SessionConfig;

        let config = SessionConfig {
            display_name: p.name,
            roles: p.roles.unwrap_or_default(),
            tags: p.tags.unwrap_or_default(),
            capabilities: p.cap.unwrap_or_default(),
        };

        let endpoint = match termlink_session::endpoint::Endpoint::start(config).await {
            Ok(e) => e,
            Err(e) => return json_err(format!("failed to register endpoint: {e}")),
        };

        let session_id = endpoint.id().to_string();
        let socket_path = endpoint.socket_path().display().to_string();
        let handle = endpoint.run_background();

        self.endpoints.lock().await.push(handle);

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "session_id": session_id,
            "socket_path": socket_path,
            "mode": "endpoint",
            "capabilities": ["events", "kv", "status"],
        }))
        .unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_deregister",
        description = "Deregister an endpoint previously created with termlink_register. Stops the background RPC server and removes the session from the hub."
    )]
    async fn termlink_deregister(&self, Parameters(p): Parameters<DeregisterParams>) -> String {
        let mut endpoints = self.endpoints.lock().await;
        let idx = endpoints.iter().position(|h| h.id().to_string() == p.session_id);

        match idx {
            Some(i) => {
                let handle = endpoints.remove(i);
                handle.stop();
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "session_id": p.session_id,
                    "message": "endpoint deregistered",
                }))
                .unwrap_or_else(json_err)
            }
            None => json_err(format!("no registered endpoint with id '{}'", p.session_id)),
        }
    }

    // === Remote (cross-host) tools ===

    #[tool(
        name = "termlink_remote_call",
        description = "Generic JSON-RPC call to a remote termlink hub over TCP+TOFU TLS. This is the universal cross-host escape hatch — any hub RPC method (session.discover, termlink.ping, command.inject, event.broadcast, agent.request, etc.) can be invoked remotely through this one tool. The remote hub must be started with termlink_hub_start(tcp_addr=\"...\") or CLI `termlink hub start --tcp`. Auth uses a 32-byte HMAC secret shared out-of-band (secret_file or secret). Returns the full JSON-RPC response as a JSON value."
    )]
    async fn termlink_remote_call(&self, Parameters(p): Parameters<RemoteCallParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("control");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(30));

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub,
                p.secret_file.as_deref(),
                p.secret.as_deref(),
                scope,
            )
            .await
            {
                Ok(c) => c,
                Err(e) => return e,
            };

            let params = p.params.unwrap_or(serde_json::json!({}));
            let req_id = serde_json::json!(format!("mcp-{}", std::process::id()));
            match rpc_client.call(&p.method, req_id, params).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "hub": p.hub,
                        "method": p.method,
                        "result": r.result,
                    }))
                    .unwrap_or_else(json_err)
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => json_err(format!(
                    "RPC error on {}: {} {}",
                    p.method, e.error.code, e.error.message
                )),
                Err(e) => json_err(format!("RPC transport error on {}: {}", p.method, e)),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(30))),
        }
    }

    #[tool(
        name = "termlink_remote_ping",
        description = "Ping a remote termlink hub (or a specific session on it) over TCP+TOFU TLS. Without a session argument, returns hub liveness + session count via session.discover. With a session argument, returns that session's state via termlink.ping routed through the hub. Useful for cross-host health checks before running heavier remote operations."
    )]
    async fn termlink_remote_ping(&self, Parameters(p): Parameters<RemotePingParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("observe");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(10));

        let fut = async move {
            let start = std::time::Instant::now();
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub,
                p.secret_file.as_deref(),
                p.secret.as_deref(),
                scope,
            )
            .await
            {
                Ok(c) => c,
                Err(e) => return e,
            };
            let auth_ms = start.elapsed().as_millis() as u64;

            let (method, params, kind) = match &p.session {
                Some(target) => (
                    "termlink.ping",
                    serde_json::json!({ "target": target }),
                    "session",
                ),
                None => ("session.discover", serde_json::json!({}), "hub"),
            };

            let req_id = serde_json::json!("mcp-ping");
            match rpc_client.call(method, req_id, params).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    let total_ms = start.elapsed().as_millis() as u64;
                    let rpc_ms = total_ms.saturating_sub(auth_ms);
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "hub": p.hub,
                        "kind": kind,
                        "session": p.session,
                        "result": r.result,
                        "total_ms": total_ms,
                        "auth_ms": auth_ms,
                        "rpc_ms": rpc_ms,
                    }))
                    .unwrap_or_else(json_err)
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => json_err(format!(
                    "Ping failed: {} {}",
                    e.error.code, e.error.message
                )),
                Err(e) => json_err(format!("Ping error: {}", e)),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(10))),
        }
    }

    #[tool(
        name = "termlink_remote_inject",
        description = "Inject text (with optional Enter) into a session on a remote termlink hub over TCP+TOFU TLS. This is the primary cross-host prompt-injection tool — equivalent to termlink_inject but routes through the hub's command.inject method rather than a local Unix socket. The remote hub routes the call to the target session automatically. Use for sending prompts/commands to agents running on another host."
    )]
    async fn termlink_remote_inject(&self, Parameters(p): Parameters<RemoteInjectParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("control");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(30));
        let enter = p.enter.unwrap_or(false);

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub,
                p.secret_file.as_deref(),
                p.secret.as_deref(),
                scope,
            )
            .await
            {
                Ok(c) => c,
                Err(e) => return e,
            };

            // Build keys array: text chars, optionally followed by Enter
            let mut keys: Vec<serde_json::Value> =
                p.text.chars().map(|c| serde_json::json!(c.to_string())).collect();
            if enter {
                keys.push(serde_json::json!("Enter"));
            }

            let params = serde_json::json!({
                "target": p.session,
                "keys": keys,
            });
            let req_id = serde_json::json!("mcp-inject");
            match rpc_client
                .call("command.inject", req_id, params)
                .await
            {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "hub": p.hub,
                        "session": p.session,
                        "bytes": p.text.len(),
                        "enter": enter,
                        "result": r.result,
                    }))
                    .unwrap_or_else(json_err)
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    let msg = if e.error.message.contains("not found")
                        || e.error.message.contains("No route")
                    {
                        format!("Session '{}' not found on {}", p.session, p.hub)
                    } else {
                        format!("Inject failed: {} {}", e.error.code, e.error.message)
                    };
                    json_err(msg)
                }
                Err(e) => json_err(format!("Inject error: {}", e)),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(30))),
        }
    }

    // === Inbox Tools (T-998) ===

    #[tool(
        name = "termlink_inbox_status",
        description = "Show hub inbox status — total pending file transfers queued for offline sessions. Returns target names and pending counts. Requires a running hub."
    )]
    async fn termlink_inbox_status(&self) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }

        let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
        let cache = termlink_session::hub_capabilities::shared_cache();
        let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
        match termlink_session::inbox_channel::status_with_fallback(&addr, cache, &mut ctx).await {
            Ok(status) => serde_json::to_string_pretty(&status).unwrap_or_else(json_err),
            Err(e) => json_err(format!("inbox.status error: {e}")),
        }
    }

    #[tool(
        name = "termlink_inbox_clear",
        description = "Clear pending file transfers from the hub inbox. Specify a target session name to clear its transfers, or set all=true to clear everything. Requires a running hub."
    )]
    async fn termlink_inbox_clear(&self, Parameters(p): Parameters<InboxClearParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }

        let scope = if p.all.unwrap_or(false) {
            termlink_session::inbox_channel::ClearScope::All
        } else if let Some(ref target) = p.target {
            termlink_session::inbox_channel::ClearScope::Target(target.clone())
        } else {
            return json_err("Specify 'target' or set 'all' to true");
        };

        let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
        let cache = termlink_session::hub_capabilities::shared_cache();
        let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
        match termlink_session::inbox_channel::clear_with_fallback(&addr, scope, cache, &mut ctx).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
            Err(e) => json_err(format!("inbox.clear error: {e}")),
        }
    }

    #[tool(
        name = "termlink_inbox_list",
        description = "List pending file transfers in the hub inbox for a specific target session. Returns transfer IDs, filenames, sizes, and completion status. Requires a running hub."
    )]
    async fn termlink_inbox_list(&self, Parameters(p): Parameters<InboxListParams>) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }

        let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
        let cache = termlink_session::hub_capabilities::shared_cache();
        let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
        match termlink_session::inbox_channel::list_with_fallback(&addr, &p.target, cache, &mut ctx).await {
            Ok(entries) => serde_json::to_string_pretty(&serde_json::json!({ "transfers": entries }))
                .unwrap_or_else(json_err),
            Err(e) => json_err(format!("inbox.list (channel-aware) failed: {e}")),
        }
    }

    // === Remote List/Exec Tools (T-1011) ===

    #[tool(
        name = "termlink_remote_list",
        description = "List sessions on a remote hub. Discovers all registered sessions, optionally filtered by name, tags, or roles. Returns session IDs, names, states, and metadata. Useful for cross-host agent discovery."
    )]
    async fn termlink_remote_list(&self, Parameters(p): Parameters<RemoteListParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("observe");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(10));

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub, p.secret_file.as_deref(), p.secret.as_deref(), scope,
            ).await {
                Ok(c) => c,
                Err(e) => return e,
            };

            let mut params = serde_json::json!({});
            if let Some(ref name) = p.name {
                params["name"] = serde_json::json!(name);
            }
            if let Some(ref tags) = p.tags {
                let tag_list: Vec<&str> = tags.split(',').map(|s| s.trim()).collect();
                params["tags"] = serde_json::json!(tag_list);
            }
            if let Some(ref roles) = p.roles {
                let role_list: Vec<&str> = roles.split(',').map(|s| s.trim()).collect();
                params["roles"] = serde_json::json!(role_list);
            }

            match rpc_client.call("session.discover", serde_json::json!("mcp-list"), params).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "hub": p.hub,
                        "sessions": r.result["sessions"],
                        "count": r.result["sessions"].as_array().map(|a| a.len()).unwrap_or(0),
                    })).unwrap_or_else(json_err)
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    json_err(format!("session.discover error on {}: {}", p.hub, e.error.message))
                }
                Err(e) => json_err(format!("RPC failed: {e}")),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(10))),
        }
    }

    #[tool(
        name = "termlink_remote_exec",
        description = "Execute a shell command on a remote session via hub routing. The command runs on the target session's host and returns stdout/stderr. Requires 'execute' scope. Useful for cross-host infrastructure management and agent coordination."
    )]
    async fn termlink_remote_exec(&self, Parameters(p): Parameters<RemoteExecParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("execute");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(30));

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub, p.secret_file.as_deref(), p.secret.as_deref(), scope,
            ).await {
                Ok(c) => c,
                Err(e) => return e,
            };

            let mut params = serde_json::json!({
                "target": p.session,
                "command": p.command,
            });
            if let Some(ref cwd) = p.cwd {
                params["cwd"] = serde_json::json!(cwd);
            }

            match rpc_client.call("command.exec", serde_json::json!("mcp-exec"), params).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": true,
                        "hub": p.hub,
                        "session": p.session,
                        "result": r.result,
                    })).unwrap_or_else(json_err)
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    json_err(format!("command.exec error on {}/{}: {}", p.hub, p.session, e.error.message))
                }
                Err(e) => json_err(format!("RPC failed: {e}")),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(30))),
        }
    }

    // === Remote Inbox Tools (T-1010) ===

    #[tool(
        name = "termlink_remote_inbox_status",
        description = "Show inbox status on a remote hub — total pending file transfers queued for offline sessions. Connects to the remote hub via TCP+TOFU TLS. The hub address can be host:port or a profile name from ~/.termlink/hubs.toml."
    )]
    async fn termlink_remote_inbox_status(&self, Parameters(p): Parameters<RemoteInboxStatusParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("execute");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(10));

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub, p.secret_file.as_deref(), p.secret.as_deref(), scope,
            ).await {
                Ok(c) => c,
                Err(e) => return e,
            };

            let cache = termlink_session::hub_capabilities::shared_cache();
            let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
            match termlink_session::inbox_channel::status_with_fallback_with_client(
                &mut rpc_client, &p.hub, cache, &mut ctx,
            ).await {
                Ok(status) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true, "hub": p.hub, "result": status,
                })).unwrap_or_else(json_err),
                Err(e) => json_err(format!("inbox.status error on {}: {}", p.hub, e)),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(10))),
        }
    }

    #[tool(
        name = "termlink_remote_inbox_list",
        description = "List pending file transfers on a remote hub's inbox for a specific target session. Connects via TCP+TOFU TLS."
    )]
    async fn termlink_remote_inbox_list(&self, Parameters(p): Parameters<RemoteInboxListParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("execute");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(10));

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub, p.secret_file.as_deref(), p.secret.as_deref(), scope,
            ).await {
                Ok(c) => c,
                Err(e) => return e,
            };

            let cache = termlink_session::hub_capabilities::shared_cache();
            let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
            match termlink_session::inbox_channel::list_with_fallback_with_client(
                &mut rpc_client, &p.hub, &p.target, cache, &mut ctx,
            ).await {
                Ok(entries) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "hub": p.hub,
                    "result": { "transfers": entries },
                })).unwrap_or_else(json_err),
                Err(e) => json_err(format!("inbox.list (channel-aware) error on {}: {e}", p.hub)),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(10))),
        }
    }

    #[tool(
        name = "termlink_remote_inbox_clear",
        description = "Clear pending file transfers on a remote hub's inbox. Specify a target session name or set all=true. Connects via TCP+TOFU TLS."
    )]
    async fn termlink_remote_inbox_clear(&self, Parameters(p): Parameters<RemoteInboxClearParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("execute");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(10));

        let fut = async move {
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub, p.secret_file.as_deref(), p.secret.as_deref(), scope,
            ).await {
                Ok(c) => c,
                Err(e) => return e,
            };

            let clear_scope = if p.all.unwrap_or(false) {
                termlink_session::inbox_channel::ClearScope::All
            } else if let Some(ref target) = p.target {
                termlink_session::inbox_channel::ClearScope::Target(target.clone())
            } else {
                return json_err("Specify 'target' or set 'all' to true");
            };

            let cache = termlink_session::hub_capabilities::shared_cache();
            let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
            match termlink_session::inbox_channel::clear_with_fallback_with_client(
                &mut rpc_client, &p.hub, clear_scope, cache, &mut ctx,
            ).await {
                Ok(result) => serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true, "hub": p.hub, "result": result,
                })).unwrap_or_else(json_err),
                Err(e) => json_err(format!("inbox.clear error on {}: {}", p.hub, e)),
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(10))),
        }
    }

    #[tool(
        name = "termlink_remote_doctor",
        description = "Health check a remote hub — connectivity, sessions, inbox status. Returns pass/warn/fail checks. The hub address can be host:port or a profile name from ~/.termlink/hubs.toml."
    )]
    async fn termlink_remote_doctor(&self, Parameters(p): Parameters<RemoteDoctorParams>) -> String {
        let scope = p.scope.as_deref().unwrap_or("execute");
        let timeout = std::time::Duration::from_secs(p.timeout.unwrap_or(10));

        let fut = async move {
            let mut checks: Vec<serde_json::Value> = Vec::new();
            let mut pass_count: u32 = 0;
            let mut warn_count: u32 = 0;
            let fail_count: u32 = 0;

            // 1. Connectivity
            let connect_start = std::time::Instant::now();
            let mut rpc_client = match connect_remote_hub_mcp(
                &p.hub, p.secret_file.as_deref(), p.secret.as_deref(), scope,
            ).await {
                Ok(c) => {
                    let latency = connect_start.elapsed().as_millis();
                    pass_count += 1;
                    checks.push(serde_json::json!({"check": "connectivity", "status": "pass", "message": format!("connected in {}ms", latency)}));
                    c
                }
                Err(e) => {
                    return e;
                }
            };

            // 2. Sessions (use session.discover — hub-level method, not session.list which requires target)
            match rpc_client.call("session.discover", serde_json::json!("mcp-doc-sd"), serde_json::json!({})).await {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    if let Some(sessions) = r.result["sessions"].as_array() {
                        let count = sessions.len();
                        let names: Vec<&str> = sessions.iter()
                            .filter_map(|s| s["display_name"].as_str())
                            .collect();
                        pass_count += 1;
                        checks.push(serde_json::json!({"check": "sessions", "status": "pass", "message": format!("{} session(s): {}", count, names.join(", "))}));
                    } else {
                        warn_count += 1;
                        checks.push(serde_json::json!({"check": "sessions", "status": "warn", "message": "unexpected response format"}));
                    }
                }
                Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                    warn_count += 1;
                    checks.push(serde_json::json!({"check": "sessions", "status": "warn", "message": format!("session.discover error: {}", e.error.message)}));
                }
                Err(e) => {
                    warn_count += 1;
                    checks.push(serde_json::json!({"check": "sessions", "status": "warn", "message": format!("RPC failed: {}", e)}));
                }
            }

            // 3. Inbox.
            //
            // T-1400: prefer channel.list(prefix="inbox:") over legacy
            // inbox.status; fall back to inbox.status on any error so the
            // probe stays useful across version skew. Same migration as
            // commands/infrastructure.rs::doctor step 7.
            let inbox_outcome: Result<(u64, usize), String> = match rpc_client
                .call(
                    "channel.list",
                    serde_json::json!("mcp-doc-cl"),
                    serde_json::json!({"prefix": "inbox:"}),
                )
                .await
            {
                Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                    let topics = r.result["topics"].as_array().cloned().unwrap_or_default();
                    let target_count = topics.len();
                    let total: u64 = topics
                        .iter()
                        .filter_map(|t| t["count"].as_u64())
                        .sum();
                    Ok((total, target_count))
                }
                _ => {
                    // Fallback to legacy inbox.status
                    match rpc_client
                        .call(
                            "inbox.status",
                            serde_json::json!("mcp-doc-is"),
                            serde_json::json!({}),
                        )
                        .await
                    {
                        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => Ok((
                            r.result["total_transfers"].as_u64().unwrap_or(0),
                            r.result["targets"].as_array().map(|t| t.len()).unwrap_or(0),
                        )),
                        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                            Err(format!("inbox.status error: {}", e.error.message))
                        }
                        Err(e) => Err(format!("RPC failed: {}", e)),
                    }
                }
            };

            match inbox_outcome {
                Ok((0, _)) => {
                    pass_count += 1;
                    checks.push(serde_json::json!({"check": "inbox", "status": "pass", "message": "no pending transfers"}));
                }
                Ok((total, targets)) => {
                    warn_count += 1;
                    checks.push(serde_json::json!({"check": "inbox", "status": "warn", "message": format!("{} pending transfer(s) for {} target(s)", total, targets)}));
                }
                Err(msg) => {
                    warn_count += 1;
                    checks.push(serde_json::json!({"check": "inbox", "status": "warn", "message": msg}));
                }
            }

            serde_json::to_string_pretty(&serde_json::json!({
                "ok": fail_count == 0,
                "hub": p.hub,
                "checks": checks,
                "summary": {"pass": pass_count, "warn": warn_count, "fail": fail_count}
            })).unwrap_or_else(json_err)
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(response) => response,
            Err(_) => json_err(format!("Timeout after {}s", p.timeout.unwrap_or(10))),
        }
    }

    // === TOFU management (T-1038) ===

    #[tool(
        name = "termlink_tofu_list",
        description = "List all trusted hub certificates (TOFU store). Shows host:port, fingerprint, first_seen, and last_seen for each trusted hub."
    )]
    async fn termlink_tofu_list(&self, Parameters(_p): Parameters<TofuListParams>) -> String {
        let store = termlink_session::tofu::KnownHubStore::default_store();
        let entries = store.list_all();
        let items: Vec<serde_json::Value> = entries.iter().map(|e| {
            serde_json::json!({
                "host": e.host_port,
                "fingerprint": e.fingerprint,
                "first_seen": e.first_seen,
                "last_seen": e.last_seen,
            })
        }).collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "count": items.len(),
            "entries": items,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_tofu_clear",
        description = "Clear a trusted hub certificate from the TOFU store. After clearing, the next connection to this hub will re-trust its certificate (TOFU). Use this when a hub has been restarted and generated a new TLS certificate."
    )]
    async fn termlink_tofu_clear(&self, Parameters(p): Parameters<TofuClearParams>) -> String {
        let store = termlink_session::tofu::KnownHubStore::default_store();
        let existed = store.remove(&p.host);
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": existed,
            "host": p.host,
            "removed": existed,
            "message": if existed {
                format!("Removed TOFU entry for {}. Next connection will re-trust.", p.host)
            } else {
                format!("No TOFU entry found for '{}'", p.host)
            },
        })).unwrap_or_else(json_err)
    }
    // === Fleet status (T-1102) ===

    #[tool(
        name = "termlink_fleet_status",
        description = "One-screen operational overview of all configured hubs. Shows each hub's status (up/down/auth-fail), session count, latency, and actionable fix steps for broken hubs. The operator's morning-check command."
    )]
    async fn termlink_fleet_status(&self, Parameters(p): Parameters<FleetStatusParams>) -> String {
        let profiles = list_all_hub_profiles();
        if profiles.is_empty() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "fleet": [],
                "summary": {"total": 0, "up": 0, "down": 0, "auth_fail": 0},
                "actions": [],
                "message": "No hubs configured in ~/.termlink/hubs.toml",
            })).unwrap_or_else(json_err);
        }

        let timeout_secs = p.timeout.unwrap_or(10);
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let mut fleet: Vec<serde_json::Value> = Vec::new();
        let mut actions: Vec<String> = Vec::new();
        let mut up_count: u32 = 0;
        let mut down_count: u32 = 0;
        let mut auth_fail_count: u32 = 0;

        for (name, address, secret_file, secret_hex) in &profiles {
            let connect_start = std::time::Instant::now();
            let result = tokio::time::timeout(
                timeout_dur,
                connect_remote_hub_mcp(
                    address,
                    secret_file.as_deref(),
                    secret_hex.as_deref(),
                    "execute",
                ),
            ).await;

            match result {
                Ok(Ok(mut client)) => {
                    let latency = connect_start.elapsed().as_millis();
                    up_count += 1;

                    // Query session count
                    let session_count = match client.call(
                        "session.discover",
                        serde_json::json!("mcp-fleet-sd"),
                        serde_json::json!({}),
                    ).await {
                        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
                            r.result["sessions"].as_array().map(|s| s.len()).unwrap_or(0)
                        }
                        _ => 0,
                    };

                    fleet.push(serde_json::json!({
                        "hub": name, "address": address,
                        "status": "up", "latency_ms": latency,
                        "sessions": session_count,
                    }));
                }
                Ok(Err(err_json)) => {
                    let is_auth = err_json.contains("invalid signature")
                        || err_json.contains("Token validation")
                        || err_json.contains("TOFU VIOLATION");
                    if is_auth {
                        auth_fail_count += 1;
                        fleet.push(serde_json::json!({"hub": name, "address": address, "status": "auth-fail", "error": err_json}));
                        actions.push(format!("{}: Reauth needed — termlink fleet reauth {} --bootstrap-from ssh:<host>", name, name));
                    } else {
                        down_count += 1;
                        fleet.push(serde_json::json!({"hub": name, "address": address, "status": "down", "error": err_json}));
                        actions.push(format!("{}: {}", name, err_json));
                    }
                }
                Err(_) => {
                    down_count += 1;
                    fleet.push(serde_json::json!({"hub": name, "address": address, "status": "timeout"}));
                    actions.push(format!("{}: Timeout — check network to {}", name, address));
                }
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": down_count == 0 && auth_fail_count == 0,
            "fleet": fleet,
            "summary": {"total": fleet.len(), "up": up_count, "down": down_count, "auth_fail": auth_fail_count},
            "actions": actions,
        })).unwrap_or_else(json_err)
    }

    // === Net test (T-1106) ===

    #[tool(
        name = "termlink_net_test",
        description = "Layered connectivity diagnostic for configured hubs. Tests each hub through TCP → TLS → auth → RPC ping, pinpointing exactly where a connection breaks. Use when fleet_status shows 'down' and you need to know if it's a network, cert, or secret issue."
    )]
    async fn termlink_net_test(&self, Parameters(p): Parameters<NetTestParams>) -> String {
        use serde_json::json;
        use std::time::{Duration, Instant};

        let all_profiles = list_all_hub_profiles();
        if all_profiles.is_empty() {
            return serde_json::to_string_pretty(&json!({
                "ok": true, "hubs": [],
                "summary": {"total": 0, "healthy": 0, "degraded": 0, "unreachable": 0},
                "message": "No hubs configured in ~/.termlink/hubs.toml",
            })).unwrap_or_else(json_err);
        }

        let profiles: Vec<_> = if let Some(ref filter) = p.profile {
            let matches: Vec<_> = all_profiles.into_iter()
                .filter(|(n, _, _, _)| n == filter).collect();
            if matches.is_empty() {
                return serde_json::to_string_pretty(&json!({
                    "ok": false,
                    "error": format!("Hub profile '{}' not found", filter),
                })).unwrap_or_else(json_err);
            }
            matches
        } else {
            all_profiles
        };

        let timeout_secs = p.timeout.unwrap_or(5);
        let timeout_dur = Duration::from_secs(timeout_secs);
        let mut hubs: Vec<serde_json::Value> = Vec::new();
        let mut healthy = 0u32;
        let mut degraded = 0u32;
        let mut unreachable = 0u32;

        for (name, address, secret_file, secret_hex) in &profiles {
            let parts: Vec<&str> = address.split(':').collect();
            if parts.len() != 2 {
                unreachable += 1;
                hubs.push(json!({
                    "hub": name, "address": address, "healthy": false,
                    "diagnosis": "invalid hub address", "layers": {},
                }));
                continue;
            }
            let host = parts[0].to_string();
            let port: u16 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => {
                    unreachable += 1;
                    hubs.push(json!({
                        "hub": name, "address": address, "healthy": false,
                        "diagnosis": "invalid port", "layers": {},
                    }));
                    continue;
                }
            };

            let mut layers = serde_json::Map::new();

            // L1: TCP
            let tcp_start = Instant::now();
            let tcp_result = tokio::time::timeout(
                timeout_dur,
                tokio::net::TcpStream::connect((host.as_str(), port)),
            ).await;
            let tcp_latency = tcp_start.elapsed().as_millis() as u64;

            let (tcp_ok, tcp_entry) = match tcp_result {
                Ok(Ok(_)) => (true, json!({"status": "pass", "latency_ms": tcp_latency})),
                Ok(Err(e)) => (false, json!({"status": "fail", "latency_ms": tcp_latency,
                    "error": e.to_string()})),
                Err(_) => (false, json!({"status": "timeout", "latency_ms": timeout_secs * 1000})),
            };
            layers.insert("tcp".to_string(), tcp_entry);

            if !tcp_ok {
                unreachable += 1;
                hubs.push(json!({
                    "hub": name, "address": address, "healthy": false,
                    "diagnosis": "Network-level failure — check firewall/VPN/routing and hub process is listening on the configured port",
                    "layers": layers,
                }));
                continue;
            }

            // L2+L3: TLS + auth (MCP's connect_remote_hub_mcp bundles both)
            let conn_start = Instant::now();
            let conn_result = tokio::time::timeout(
                timeout_dur,
                connect_remote_hub_mcp(
                    address,
                    secret_file.as_deref(),
                    secret_hex.as_deref(),
                    "execute",
                ),
            ).await;
            let conn_latency = conn_start.elapsed().as_millis() as u64;

            match conn_result {
                Ok(Ok(mut client)) => {
                    // TLS+auth bundled — report as pass for both layers
                    layers.insert("tls".to_string(),
                        json!({"status": "pass", "latency_ms": conn_latency}));
                    layers.insert("auth".to_string(),
                        json!({"status": "pass", "latency_ms": 0}));

                    // L4: PING via session.discover
                    let ping_start = Instant::now();
                    let ping_result = tokio::time::timeout(
                        timeout_dur,
                        client.call("session.discover", json!("mcp-net-ping"), json!({})),
                    ).await;
                    let ping_latency = ping_start.elapsed().as_millis() as u64;

                    let (ping_ok, ping_entry) = match ping_result {
                        Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_))) =>
                            (true, json!({"status": "pass", "latency_ms": ping_latency})),
                        Ok(Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e))) =>
                            (false, json!({"status": "fail", "latency_ms": ping_latency,
                                "error": format!("{} {}", e.error.code, e.error.message)})),
                        Ok(Err(e)) => (false, json!({"status": "fail", "latency_ms": ping_latency,
                            "error": e.to_string()})),
                        Err(_) => (false, json!({"status": "timeout",
                            "latency_ms": timeout_secs * 1000})),
                    };
                    layers.insert("ping".to_string(), ping_entry);

                    if ping_ok {
                        healthy += 1;
                        hubs.push(json!({
                            "hub": name, "address": address, "healthy": true, "layers": layers,
                        }));
                    } else {
                        degraded += 1;
                        hubs.push(json!({
                            "hub": name, "address": address, "healthy": false,
                            "diagnosis": "RPC ping failed after auth — hub is reachable and trusted but misbehaving",
                            "layers": layers,
                        }));
                    }
                }
                Ok(Err(err_json)) => {
                    let is_auth = err_json.contains("invalid signature")
                        || err_json.contains("Token validation")
                        || err_json.contains("Authentication");
                    let is_tofu = err_json.contains("TOFU") || err_json.contains("fingerprint");

                    if is_auth && !is_tofu {
                        layers.insert("tls".to_string(),
                            json!({"status": "pass", "latency_ms": conn_latency}));
                        layers.insert("auth".to_string(), json!({
                            "status": "fail", "latency_ms": conn_latency,
                            "error": err_json.clone(),
                        }));
                        degraded += 1;
                        hubs.push(json!({
                            "hub": name, "address": address, "healthy": false,
                            "diagnosis": "HMAC secret mismatch — run: termlink fleet reauth <profile> --bootstrap-from ssh:<host>",
                            "layers": layers,
                        }));
                    } else {
                        layers.insert("tls".to_string(), json!({
                            "status": "fail", "latency_ms": conn_latency,
                            "error": err_json.clone(),
                        }));
                        degraded += 1;
                        hubs.push(json!({
                            "hub": name, "address": address, "healthy": false,
                            "diagnosis": if is_tofu {
                                "TLS cert changed — run: termlink tofu clear <host:port> and retry"
                            } else {
                                "TLS handshake failed — hub may not be speaking TLS, or cert is invalid"
                            },
                            "layers": layers,
                        }));
                    }
                }
                Err(_) => {
                    layers.insert("tls".to_string(),
                        json!({"status": "timeout", "latency_ms": timeout_secs * 1000}));
                    degraded += 1;
                    hubs.push(json!({
                        "hub": name, "address": address, "healthy": false,
                        "diagnosis": "TLS handshake timed out — hub is slow or silently dropping TLS",
                        "layers": layers,
                    }));
                }
            }
        }

        serde_json::to_string_pretty(&json!({
            "ok": degraded == 0 && unreachable == 0,
            "hubs": hubs,
            "summary": {
                "total": profiles.len(),
                "healthy": healthy,
                "degraded": degraded,
                "unreachable": unreachable,
            },
        })).unwrap_or_else(json_err)
    }

    // === Fleet doctor (T-1039) ===

    #[tool(
        name = "termlink_fleet_doctor",
        description = "Health check all configured hubs in ~/.termlink/hubs.toml. Returns per-hub connectivity status, latency, and diagnostic hints for failures."
    )]
    async fn termlink_fleet_doctor(&self, Parameters(p): Parameters<FleetDoctorParams>) -> String {
        let profiles = list_all_hub_profiles();
        if profiles.is_empty() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "hubs": [],
                "message": "No hubs configured in ~/.termlink/hubs.toml",
            })).unwrap_or_else(json_err);
        }

        let timeout_secs = p.timeout.unwrap_or(10);
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let mut hub_results: Vec<serde_json::Value> = Vec::new();
        let mut pass_count: u32 = 0;
        let mut fail_count: u32 = 0;

        for (name, address, secret_file, secret_hex) in &profiles {
            let connect_start = std::time::Instant::now();
            let result = tokio::time::timeout(
                timeout_dur,
                connect_remote_hub_mcp(
                    address,
                    secret_file.as_deref(),
                    secret_hex.as_deref(),
                    "execute",
                ),
            ).await;

            match result {
                Ok(Ok(_client)) => {
                    let latency = connect_start.elapsed().as_millis();
                    pass_count += 1;
                    hub_results.push(serde_json::json!({"hub": name, "address": address, "status": "ok", "latency_ms": latency}));
                }
                Ok(Err(err_json)) => {
                    fail_count += 1;
                    hub_results.push(serde_json::json!({"hub": name, "address": address, "status": "error", "error": err_json}));
                }
                Err(_) => {
                    fail_count += 1;
                    hub_results.push(serde_json::json!({"hub": name, "address": address, "status": "timeout", "error": format!("Timeout after {}s", timeout_secs)}));
                }
            }
        }

        serde_json::to_string_pretty(&serde_json::json!({
            "ok": fail_count == 0,
            "hubs": hub_results,
            "summary": {"total": hub_results.len(), "pass": pass_count, "fail": fail_count},
        })).unwrap_or_else(json_err)
    }

    // === Hub restart (T-1040) ===

    #[tool(
        name = "termlink_hub_restart",
        description = "Restart the local termlink hub. Stops the running hub process and starts a new one, preserving TCP binding and runtime directory. Returns the new hub PID on success."
    )]
    async fn termlink_hub_restart(&self, Parameters(_p): Parameters<HubRestartParams>) -> String {
        use termlink_hub::pidfile;

        let (pidfile_path, _) = resolve_hub_paths();

        let old_pid = match pidfile::check(&pidfile_path) {
            pidfile::PidfileStatus::Running(pid) => pid,
            pidfile::PidfileStatus::Stale(pid) => {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "ok": false,
                    "error": format!("Hub PID {} is stale (dead). Use hub start instead.", pid),
                })).unwrap_or_else(json_err);
            }
            pidfile::PidfileStatus::NotRunning => {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "ok": false,
                    "error": "Hub is not running. Use hub start to start it.",
                })).unwrap_or_else(json_err);
            }
        };

        // Determine TCP address from existing hub config
        let runtime_dir = pidfile_path.parent().unwrap_or(std::path::Path::new("/tmp"));
        let tcp_flag_path = runtime_dir.join("hub.tcp");
        let tcp_addr = std::fs::read_to_string(&tcp_flag_path)
            .ok()
            .map(|s| s.trim().to_string());

        // Find our own binary path
        let self_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => return json_err(format!("Cannot determine own binary path: {e}")),
        };

        // Build the hub start command
        let mut cmd = std::process::Command::new(&self_exe);
        cmd.arg("hub").arg("start");
        if let Some(ref addr) = tcp_addr {
            cmd.arg("--tcp").arg(addr);
        }

        // Preserve non-default runtime dir (e.g., /var/lib/termlink)
        let default_runtime = termlink_session::discovery::runtime_dir();
        if pidfile_path.parent().is_some_and(|d| d != default_runtime.as_path()) {
            cmd.env("TERMLINK_RUNTIME_DIR", pidfile_path.parent().unwrap());
        }

        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        // Stop the old hub
        unsafe { libc::kill(old_pid as i32, libc::SIGTERM) };

        // Wait for old hub to die (up to 3s)
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if !termlink_session::liveness::process_exists(old_pid) {
                break;
            }
        }

        if termlink_session::liveness::process_exists(old_pid) {
            return serde_json::to_string_pretty(&serde_json::json!({
                "ok": false,
                "error": format!("Old hub (PID {}) did not stop within 3s", old_pid),
            })).unwrap_or_else(json_err);
        }

        // Spawn new hub
        match cmd.spawn() {
            Ok(_child) => {
                // Wait briefly for new hub to start
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Check if new hub is running
                let new_status = pidfile::check(&pidfile_path);
                match new_status {
                    pidfile::PidfileStatus::Running(new_pid) => {
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "old_pid": old_pid,
                            "new_pid": new_pid,
                            "tcp": tcp_addr,
                            "message": format!("Hub restarted: {} -> {}", old_pid, new_pid),
                        })).unwrap_or_else(json_err)
                    }
                    _ => {
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": false,
                            "old_pid": old_pid,
                            "error": "New hub process started but pidfile not yet written. Check hub logs.",
                        })).unwrap_or_else(json_err)
                    }
                }
            }
            Err(e) => json_err(format!("Failed to spawn new hub: {e}")),
        }
    }

    // === Events (T-1040) ===

    #[tool(
        name = "termlink_events",
        description = "Query event history from a local session. Returns events with sequence numbers, topics, timestamps, and payloads. Use 'since' to get events after a specific sequence number."
    )]
    async fn termlink_events(&self, Parameters(p): Parameters<EventsParams>) -> String {
        let reg = match manager::find_session(&p.target) {
            Ok(r) => r,
            Err(e) => return json_err(format!("Session '{}' not found: {e}", p.target)),
        };

        let mut params = serde_json::json!({});
        if let Some(s) = p.since {
            params["since"] = serde_json::json!(s);
        }
        if let Some(ref t) = p.topic {
            params["topic"] = serde_json::json!(t);
        }

        let timeout_secs = p.timeout.unwrap_or(5);
        let timeout_dur = std::time::Duration::from_secs(timeout_secs);
        let rpc = client::rpc_call(reg.socket_path(), "event.poll", params);

        let resp = match tokio::time::timeout(timeout_dur, rpc).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => return json_err(format!("Failed to connect to session: {e}")),
            Err(_) => return json_err(format!("Event poll timed out after {timeout_secs}s")),
        };

        match client::unwrap_result(resp) {
            Ok(result) => {
                let events = result["events"].as_array();
                let count = events.map(|e| e.len()).unwrap_or(0);
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "target": p.target,
                    "count": count,
                    "next_seq": result["next_seq"],
                    "events": result["events"],
                })).unwrap_or_else(json_err)
            }
            Err(e) => json_err(format!("Event poll failed: {e}")),
        }
    }

    // === Channel Bus Tools (T-1160, T-1155) ===

    #[tool(
        name = "termlink_channel_create",
        description = "Create a T-1155 bus topic with a retention policy on the local hub. Idempotent on name — a second call with the same policy is a no-op; a conflicting policy returns an error."
    )]
    async fn termlink_channel_create(
        &self,
        Parameters(p): Parameters<ChannelCreateParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let retention = match p.retention_kind.as_deref().unwrap_or("forever") {
            "forever" => serde_json::json!({"kind": "forever"}),
            "days" => serde_json::json!({"kind": "days", "value": p.retention_value.unwrap_or(30)}),
            "messages" => serde_json::json!({"kind": "messages", "value": p.retention_value.unwrap_or(1000)}),
            other => return json_err(format!("unknown retention_kind: {other}")),
        };
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_CREATE,
            serde_json::json!({"name": p.name, "retention": retention}),
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("channel.create error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_channel_post",
        description = "Post a signed envelope to a T-1155 bus topic on the local hub. The post is signed with the caller's ed25519 identity (auto-initialized at ~/.termlink/identity.key on first use)."
    )]
    async fn termlink_channel_post(
        &self,
        Parameters(p): Parameters<ChannelPostParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let payload_bytes = match (p.payload, p.payload_b64) {
            (Some(s), None) => s.into_bytes(),
            (None, Some(b64)) => match base64::engine::general_purpose::STANDARD.decode(&b64) {
                Ok(b) => b,
                Err(e) => return json_err(format!("payload_b64 decode: {e}")),
            },
            (Some(_), Some(_)) => return json_err("pass either 'payload' or 'payload_b64', not both"),
            (None, None) => return json_err("pass 'payload' (string) or 'payload_b64' (base64)"),
        };
        let msg_type = p.msg_type.unwrap_or_else(|| "note".to_string());
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            &p.topic,
            &msg_type,
            &payload_bytes,
            p.artifact_ref.as_deref(),
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let params = serde_json::json!({
            "topic": p.topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "artifact_ref": p.artifact_ref,
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("channel.post error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_post",
        description = "Post a note to agent-chat-arc — the canonical termlink agent-to-agent visibility topic. Defaults topic to 'agent-chat-arc' and msg_type to 'note'. Optional thread/project metadata is recorded under `metadata._thread` / `metadata._project`. Companion to the `agent post` CLI verb (T-1503): both auto-sign with the local ed25519 identity. Use this from any MCP-aware client to broadcast progress / reach the fleet without shelling out (T-1560)."
    )]
    async fn termlink_agent_post(
        &self,
        Parameters(p): Parameters<AgentPostParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = p.msg_type.unwrap_or_else(|| "note".to_string());
        let payload_bytes = p.text.into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            &msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        if let Some(t) = p.thread.as_ref() {
            metadata.insert("_thread".to_string(), serde_json::Value::String(t.clone()));
            metadata.insert("thread".to_string(), serde_json::Value::String(t.clone()));
        }
        if let Some(pr) = p.project.as_ref() {
            metadata.insert("_project".to_string(), serde_json::Value::String(pr.clone()));
            metadata.insert("from_project".to_string(), serde_json::Value::String(pr.clone()));
        }
        let mut params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
        });
        if !metadata.is_empty() {
            params["metadata"] = serde_json::Value::Object(metadata);
        }
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.post error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_typing",
        description = "Emit a typing indicator on agent-chat-arc — signals 'I'm composing' to peers reading `agent typers` (T-1551) or `agent typers --watch` (T-1557). Posts a `msg_type=typing` envelope with `metadata.expires_at_ms = now + ttl_ms` (default ttl: 5000ms). Companion to `termlink_agent_post` (typed text). MCP-side equivalent of the `agent typing` CLI verb (T-1550)."
    )]
    async fn termlink_agent_typing(
        &self,
        Parameters(p): Parameters<AgentTypingParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "typing";
        let ttl_ms = p.ttl_ms.unwrap_or(5000);
        let payload_bytes: Vec<u8> = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let expires_at_ms = ts_unix_ms + (ttl_ms as i64);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("expires_at_ms".to_string(), serde_json::Value::from(expires_at_ms));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.typing error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_react",
        description = "Emit a reaction envelope on agent-chat-arc tied to a parent post offset. Posts a `msg_type=reaction` envelope with payload=emoji and `metadata.in_reply_to=<offset>`. MCP-side equivalent of the `agent react <offset> <emoji>` CLI verb (T-1525). Completes the MCP engagement triad: post (T-1560) + typing (T-1561) + react (T-1562)."
    )]
    async fn termlink_agent_react(
        &self,
        Parameters(p): Parameters<AgentReactParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "reaction";
        let payload_bytes = p.emoji.into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("in_reply_to".to_string(), serde_json::Value::String(p.offset.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.react error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_reply",
        description = "Reply to an agent-chat-arc post with new content tied to a parent offset. Posts a `msg_type=note` envelope with `metadata.in_reply_to=<offset>` so the reply joins the thread under that root. Optional thread/project metadata still passes through. MCP-side equivalent of `agent reply <offset> <text>` (T-1507). Pairs with `termlink_agent_post` (top-level) and `termlink_agent_react` (emoji ack)."
    )]
    async fn termlink_agent_reply(
        &self,
        Parameters(p): Parameters<AgentReplyParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "note";
        let payload_bytes = p.text.into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("in_reply_to".to_string(), serde_json::Value::String(p.offset.to_string()));
        if let Some(t) = p.thread.as_ref() {
            metadata.insert("_thread".to_string(), serde_json::Value::String(t.clone()));
            metadata.insert("thread".to_string(), serde_json::Value::String(t.clone()));
        }
        if let Some(pr) = p.project.as_ref() {
            metadata.insert("_project".to_string(), serde_json::Value::String(pr.clone()));
            metadata.insert("from_project".to_string(), serde_json::Value::String(pr.clone()));
        }
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.reply error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_pin",
        description = "Pin (or unpin) a chat-arc post by offset. Posts a `msg_type=pin` envelope with empty payload and `metadata.pin_target=<offset>` + `metadata.action=pin|unpin` so the curation set rendered by `agent pinned` (T-1527) updates accordingly. MCP-side equivalent of `agent pin <offset>` / `agent pin --unpin`. First MCP curation verb — pairs with the read-side via the CLI's `agent pinned` (or future MCP wrapper)."
    )]
    async fn termlink_agent_pin(
        &self,
        Parameters(p): Parameters<AgentPinParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "pin";
        let payload_bytes: Vec<u8> = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let action = if p.unpin.unwrap_or(false) { "unpin" } else { "pin" };
        let mut metadata = serde_json::Map::new();
        metadata.insert("pin_target".to_string(), serde_json::Value::String(p.offset.to_string()));
        metadata.insert("action".to_string(), serde_json::Value::String(action.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.pin error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_star",
        description = "Star (or unstar) a chat-arc post by offset. Posts a `msg_type=star` envelope with empty payload and `metadata.star_target=<offset>` + `metadata.star=true|false` so the per-sender bookmark set rendered by `agent starred` (T-1528) updates accordingly. MCP-side equivalent of `agent star <offset>` / `agent star --unstar`. Personal bookmark companion to `termlink_agent_pin` (which is fleet-wide curation)."
    )]
    async fn termlink_agent_star(
        &self,
        Parameters(p): Parameters<AgentStarParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "star";
        let payload_bytes: Vec<u8> = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let star_value = if p.unstar.unwrap_or(false) { "false" } else { "true" };
        let mut metadata = serde_json::Map::new();
        metadata.insert("star_target".to_string(), serde_json::Value::String(p.offset.to_string()));
        metadata.insert("star".to_string(), serde_json::Value::String(star_value.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.star error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_redact",
        description = "Retract an agent-chat-arc post by offset. Posts a `msg_type=redaction` envelope (longer form, matching CLI T-1531) with empty payload and `metadata.redacts=<offset>` + optional `metadata.reason`. Append-only — the original envelope stays in the topic; reader-side aggregators (e.g. `agent redactions`) decide whether to filter or render struck-through. MCP-side equivalent of `agent redact <offset> [--reason <text>]`."
    )]
    async fn termlink_agent_redact(
        &self,
        Parameters(p): Parameters<AgentRedactParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "redaction";
        let payload_bytes: Vec<u8> = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("redacts".to_string(), serde_json::Value::String(p.offset.to_string()));
        if let Some(reason) = p.reason {
            metadata.insert("reason".to_string(), serde_json::Value::String(reason));
        }
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.redact error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_edit",
        description = "Edit an agent-chat-arc post by offset. Posts a `msg_type=edit` envelope with the new text as payload and `metadata.replaces=<offset>` (matching CLI T-1530). Append-only — the original envelope stays; reader-side decides whether to render the collapsed view. MCP-side equivalent of `agent edit <offset> <text>`. Closes the post-mutation triad with redact + edit; together with the curation pair (pin/star) gives MCP-aware agents the full chat-arc lifecycle write surface."
    )]
    async fn termlink_agent_edit(
        &self,
        Parameters(p): Parameters<AgentEditParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "edit";
        let payload_bytes = p.text.into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("replaces".to_string(), serde_json::Value::String(p.offset.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.edit error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_ack",
        description = "Emit a read-receipt envelope on agent-chat-arc declaring the caller has read up through `up_to`. Posts a `msg_type=receipt` envelope with payload `up_to=N` and `metadata.up_to=N` so the read-side aggregators (`agent ack-status`, `agent ack-history`, T-1538/T-1539) can compute per-sender frontiers. MCP-side equivalent of `agent ack --up-to N` (CLI T-1526). Note: requires explicit `up_to` — the CLI's auto-resolve via topic walk is not exposed here to keep this tool a pure thin write."
    )]
    async fn termlink_agent_ack(
        &self,
        Parameters(p): Parameters<AgentAckParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "receipt";
        let payload_str = format!("up_to={}", p.up_to);
        let payload_bytes = payload_str.into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("up_to".to_string(), serde_json::Value::String(p.up_to.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.ack error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_describe",
        description = "Set agent-chat-arc topic-level description metadata. Posts a `msg_type=topic_metadata` envelope with the description in payload + `metadata.description=<text>`. Read-side `agent info` (T-1524) extracts the most recent description via `latest_description`. MCP-side equivalent of `agent describe <text>` (CLI T-1532). Useful for self-documenting the topic when bootstrapping a new arc instance or rotating purpose."
    )]
    async fn termlink_agent_describe(
        &self,
        Parameters(p): Parameters<AgentDescribeParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "topic_metadata";
        let payload_bytes = p.description.clone().into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("description".to_string(), serde_json::Value::String(p.description));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.describe error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_poll_start",
        description = "Open a chat-arc poll. Posts a `msg_type=poll_start` envelope with the question as payload and `metadata.poll_options=opt1|opt2|...` (pipe-delimited per CLI T-1543 wire convention). Returns the offset of the new envelope — that offset is the `poll_id` used by subsequent vote/end calls. Requires at least 2 options; option labels cannot contain '|'."
    )]
    async fn termlink_agent_poll_start(
        &self,
        Parameters(p): Parameters<AgentPollStartParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        if p.options.len() < 2 {
            return json_err(format!("poll requires at least 2 options (got {})", p.options.len()));
        }
        if p.options.iter().any(|o| o.contains('|')) {
            return json_err("option labels cannot contain '|' (used as the metadata delimiter)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "poll_start";
        let payload_bytes = p.question.into_bytes();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let opts_joined = p.options.join("|");
        let mut metadata = serde_json::Map::new();
        metadata.insert("poll_options".to_string(), serde_json::Value::String(opts_joined));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.poll_start error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_poll_vote",
        description = "Cast a vote on a chat-arc poll. Posts a `msg_type=poll_vote` envelope with empty payload and `metadata.poll_id=<offset>` + `metadata.poll_choice=<index>`. Latest vote per (poll_id, sender) wins per CLI T-1544 semantics. MCP-side equivalent of `agent vote <poll_id> <choice>`."
    )]
    async fn termlink_agent_poll_vote(
        &self,
        Parameters(p): Parameters<AgentPollVoteParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "poll_vote";
        let payload_bytes: Vec<u8> = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("poll_id".to_string(), serde_json::Value::String(p.poll_id.to_string()));
        metadata.insert("poll_choice".to_string(), serde_json::Value::String(p.choice.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.poll_vote error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_poll_end",
        description = "Close a chat-arc poll. Posts a `msg_type=poll_end` envelope with empty payload and `metadata.poll_id=<offset>`. The aggregator (`agent poll-results`, T-1546) drops votes whose ts is after this envelope's ts. MCP-side equivalent of `agent poll-end <poll_id>` (CLI T-1545)."
    )]
    async fn termlink_agent_poll_end(
        &self,
        Parameters(p): Parameters<AgentPollEndParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let msg_type = "poll_end";
        let payload_bytes: Vec<u8> = Vec::new();
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return json_err("HOME not set"),
        };
        let identity_dir = std::path::PathBuf::from(home).join(".termlink");
        let identity = match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
            Ok(i) => i,
            Err(e) => return json_err(format!("identity load: {e}")),
        };
        let ts_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let signed = termlink_protocol::control::channel::canonical_sign_bytes(
            topic,
            msg_type,
            &payload_bytes,
            None,
            ts_unix_ms,
        );
        let sig = identity.sign(&signed);
        let mut sig_hex = String::with_capacity(128);
        for b in sig.to_bytes() {
            use std::fmt::Write;
            let _ = write!(&mut sig_hex, "{b:02x}");
        }
        let sender_id = p.sender_id.unwrap_or_else(|| identity.fingerprint().to_string());
        let mut metadata = serde_json::Map::new();
        metadata.insert("poll_id".to_string(), serde_json::Value::String(p.poll_id.to_string()));
        let params = serde_json::json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
            "ts": ts_unix_ms,
            "sender_id": sender_id,
            "sender_pubkey_hex": identity.public_key_hex(),
            "signature_hex": sig_hex,
            "metadata": serde_json::Value::Object(metadata),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_POST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("agent.poll_end error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_agent_recent",
        description = "Read the most recent envelopes from agent-chat-arc, newest-first. Walks the topic by looping `channel.subscribe` (mirrors CLI's `walk_topic_full`). Returns a JSON array of raw envelopes (offset, ts, sender_id, msg_type, payload_b64, metadata, signature). Optional filters: `peer_fp` (sender_id match), `msg_type_filter` (exact msg_type, e.g. \"note\" excludes reactions/typing/receipts). `limit` defaults to 20, capped at 1000. First MCP read tool for chat-arc — pairs with the 13-verb write surface (post/typing/react/reply/pin/star/redact/edit/ack/describe + poll trio)."
    )]
    async fn termlink_agent_recent(
        &self,
        Parameters(p): Parameters<AgentRecentParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(20).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        // Apply filters.
        let peer = p.peer_fp.as_deref();
        let mt_filter = p.msg_type_filter.as_deref();
        let filtered: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| {
                if let Some(want) = peer {
                    let got = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("");
                    if got != want {
                        return false;
                    }
                }
                if let Some(want) = mt_filter {
                    let got = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
                    if got != want {
                        return false;
                    }
                }
                true
            })
            .collect();
        // Sort by ts descending (newest first).
        let mut sorted = filtered;
        sorted.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| a.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| b.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            tb.cmp(&ta)
        });
        sorted.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(sorted)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_search",
        description = "Search agent-chat-arc for envelopes whose payload contains a substring. Walks the topic via channel.subscribe, base64-decodes payloads (utf8 lossy), and returns matches newest-first. Optional filters: `peer_fp`, `msg_type_filter`, `case_sensitive` (default false). `limit` defaults to 100, max 1000. MCP-side equivalent of `agent search <query>` (CLI T-1508). Builds on the topic-walk pattern established by T-1571."
    )]
    async fn termlink_agent_search(
        &self,
        Parameters(p): Parameters<AgentSearchParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(100).min(1000);
        let case_sensitive = p.case_sensitive.unwrap_or(false);
        let needle: String = if case_sensitive { p.query.clone() } else { p.query.to_lowercase() };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let peer = p.peer_fp.as_deref();
        let mt_filter = p.msg_type_filter.as_deref();
        let matches: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| {
                if let Some(want) = peer {
                    let got = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("");
                    if got != want { return false; }
                }
                if let Some(want) = mt_filter {
                    let got = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
                    if got != want { return false; }
                }
                let p_b64 = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
                let bytes = match base64::engine::general_purpose::STANDARD.decode(p_b64) {
                    Ok(b) => b,
                    Err(_) => return false,
                };
                let text = String::from_utf8_lossy(&bytes);
                let hay: String = if case_sensitive { text.into_owned() } else { text.to_lowercase() };
                hay.contains(&needle)
            })
            .collect();
        let mut sorted = matches;
        sorted.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| a.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| b.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            tb.cmp(&ta)
        });
        sorted.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(sorted)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_ancestors",
        description = "Walk up the reply chain from an offset on agent-chat-arc. Builds an offset→envelope map, then chains via `metadata.in_reply_to` until reaching a post with no parent. Returns the chain root-first → leaf-last as a JSON array of envelopes. Companion to `termlink_agent_on_thread` (which descends). MCP-side equivalent of `agent ancestors <offset>` (CLI T-1510). `max_depth` defaults to 100 (safety cap)."
    )]
    async fn termlink_agent_ancestors(
        &self,
        Parameters(p): Parameters<AgentAncestorsParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let max_depth = p.max_depth.unwrap_or(100);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut by_offset: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        for env in &all {
            let off = env.get("offset").and_then(|v| v.as_u64()).map(|u| u.to_string()).unwrap_or_default();
            if !off.is_empty() {
                by_offset.insert(off, env.clone());
            }
        }
        let mut chain: Vec<serde_json::Value> = Vec::new();
        let mut current = p.offset.to_string();
        let mut depth: u64 = 0;
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        while depth < max_depth {
            if seen.contains(&current) { break; }
            seen.insert(current.clone());
            let env = match by_offset.get(&current) {
                Some(e) => e.clone(),
                None => break,
            };
            chain.push(env.clone());
            let parent = env.get("metadata")
                .and_then(|m| m.get("in_reply_to"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if parent.is_empty() { break; }
            current = parent;
            depth += 1;
        }
        chain.reverse(); // root-first
        serde_json::to_string_pretty(&serde_json::Value::Array(chain)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_pin_history",
        description = "Full pin/unpin event log on agent-chat-arc. Walks the topic, filters `msg_type=pin` envelopes, and returns `[{pin_target, sender_id, action, ts_unix_ms}, ...]` sorted newest-first. Different from `termlink_agent_pinned` (current state) — this is the timeline of curation events including unpins. MCP-side equivalent of `agent pin-history` (CLI T-1535). `limit` defaults to 200, capped at 1000."
    )]
    async fn termlink_agent_pin_history(
        &self,
        Parameters(p): Parameters<AgentPinHistoryParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(200).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("pin"))
            .map(|env| {
                let target = env.get("metadata")
                    .and_then(|m| m.get("pin_target"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let action = env.get("metadata")
                    .and_then(|m| m.get("action"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("pin")
                    .to_string();
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                serde_json::json!({
                    "pin_target": target,
                    "sender_id": sender,
                    "action": action,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_unread",
        description = "Count of envelopes on agent-chat-arc with offset > the given sender's last ack frontier. If `sender_id` is omitted, defaults to the caller's local identity fingerprint (`~/.termlink/identity.json`). Returns `{sender_id, ack_up_to, total, unread_count}`. MCP-aware agents call this to detect 'new mail' without scanning the full topic. MCP-side equivalent of `agent unread` (CLI T-1512). Pairs with `termlink_agent_ack` (mark caught up) and `termlink_agent_ack_status` (cross-fleet view)."
    )]
    async fn termlink_agent_unread(
        &self,
        Parameters(p): Parameters<AgentUnreadParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let target_sender = match p.sender_id {
            Some(s) => s,
            None => {
                let home = match std::env::var("HOME") {
                    Ok(h) => h,
                    Err(_) => return json_err("HOME not set"),
                };
                let identity_dir = std::path::PathBuf::from(home).join(".termlink");
                match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
                    Ok(i) => i.fingerprint().to_string(),
                    Err(e) => return json_err(format!("identity load: {e}")),
                }
            }
        };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut ack_up_to: u64 = 0;
        for env in &all {
            if env.get("msg_type").and_then(|v| v.as_str()) != Some("receipt") { continue; }
            if env.get("sender_id").and_then(|v| v.as_str()) != Some(target_sender.as_str()) { continue; }
            let up_to = env.get("metadata")
                .and_then(|m| m.get("up_to"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| env.get("metadata").and_then(|m| m.get("up_to")).and_then(|v| v.as_u64()))
                .unwrap_or(0);
            if up_to > ack_up_to { ack_up_to = up_to; }
        }
        let total = all.len() as u64;
        let unread = all.iter()
            .filter(|env| env.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) > ack_up_to)
            .count() as u64;
        serde_json::to_string_pretty(&serde_json::json!({
            "sender_id": target_sender,
            "ack_up_to": ack_up_to,
            "total": total,
            "unread_count": unread,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_digest",
        description = "Period summary on agent-chat-arc. Walks the topic, filters envelopes whose `ts_unix_ms >= since_ts` (or now - window_hours*3600*1000), and returns `{since_ts, total_in_window, by_msg_type, top_senders, latest_5_offsets}`. `since_ts` takes precedence over `window_hours`. `window_hours` defaults to 24 (last day), capped at 720 (30 days). MCP-side equivalent of `agent digest` (CLI T-1511). Single-call period awareness — what happened recently."
    )]
    async fn termlink_agent_digest(
        &self,
        Parameters(p): Parameters<AgentDigestParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let now_ms: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let since_ts: i64 = match p.since_ts {
            Some(t) => t,
            None => {
                let hours = p.window_hours.unwrap_or(24).min(720) as i64;
                now_ms - hours * 3600 * 1000
            }
        };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let in_window: Vec<&serde_json::Value> = all.iter()
            .filter(|env| {
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                ts >= since_ts
            })
            .collect();
        let mut by_msg_type: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut by_sender: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for env in &in_window {
            let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
            *by_msg_type.entry(mt).or_insert(0) += 1;
            let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if !sender.is_empty() {
                *by_sender.entry(sender).or_insert(0) += 1;
            }
        }
        let mut top_senders: Vec<(String, u64)> = by_sender.into_iter().collect();
        top_senders.sort_by(|a, b| b.1.cmp(&a.1));
        top_senders.truncate(5);
        let top_senders_json: Vec<serde_json::Value> = top_senders
            .into_iter()
            .map(|(s, c)| serde_json::json!({"sender_id": s, "post_count": c}))
            .collect();
        let mut latest_offsets: Vec<u64> = in_window
            .iter()
            .filter_map(|env| env.get("offset").and_then(|v| v.as_u64()))
            .collect();
        latest_offsets.sort_by(|a, b| b.cmp(a));
        latest_offsets.truncate(5);
        let by_msg_type_json: serde_json::Map<String, serde_json::Value> = by_msg_type
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v)))
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "since_ts": since_ts,
            "now_ts": now_ms,
            "total_in_window": in_window.len(),
            "by_msg_type": by_msg_type_json,
            "top_senders": top_senders_json,
            "latest_5_offsets": latest_offsets,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_redactions",
        description = "List all redaction events on agent-chat-arc. Walks the topic, filters `msg_type=redaction` envelopes, and returns `[{redacts_offset, sender_id, reason, ts_unix_ms}, ...]` sorted newest-first. The original posts stay in the topic (append-only); this view gives MCP-aware agents the curation log — what's been retracted and why. MCP-side equivalent of `agent redactions` (CLI T-1534). Companion read tool to `termlink_agent_redact` (T-1566). `limit` defaults to 200, capped at 1000."
    )]
    async fn termlink_agent_redactions(
        &self,
        Parameters(p): Parameters<AgentRedactionsParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(200).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("redaction"))
            .map(|env| {
                let redacts = env.get("metadata")
                    .and_then(|m| m.get("redacts"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let reason = env.get("metadata")
                    .and_then(|m| m.get("reason"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                serde_json::json!({
                    "redacts_offset": redacts,
                    "sender_id": sender,
                    "reason": reason,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_ack_status",
        description = "Current receipt frontier per sender on agent-chat-arc. Walks the topic, filters `msg_type=receipt` envelopes, groups by `sender_id`, and keeps `max(up_to)` per sender. Returns `[{sender_id, ack_up_to, last_ack_ts}, ...]` sorted by ack_up_to desc. Lets MCP-aware agents see who's caught up (and who's stale) without dumping the full receipt log. MCP-side equivalent of `agent ack-status` (CLI T-1539). Companion read tool to `termlink_agent_ack` (T-1568)."
    )]
    async fn termlink_agent_ack_status(
        &self,
        Parameters(_p): Parameters<AgentAckStatusParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut frontiers: std::collections::HashMap<String, (u64, i64)> = std::collections::HashMap::new();
        for env in &all {
            if env.get("msg_type").and_then(|v| v.as_str()) != Some("receipt") {
                continue;
            }
            let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if sender.is_empty() { continue; }
            let up_to = env.get("metadata")
                .and_then(|m| m.get("up_to"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| env.get("metadata").and_then(|m| m.get("up_to")).and_then(|v| v.as_u64()))
                .unwrap_or(0);
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let entry = frontiers.entry(sender).or_insert((0, 0));
            if up_to > entry.0 {
                entry.0 = up_to;
                entry.1 = ts;
            }
        }
        let mut results: Vec<serde_json::Value> = frontiers
            .into_iter()
            .map(|(sender, (up_to, ts))| serde_json::json!({
                "sender_id": sender,
                "ack_up_to": up_to,
                "last_ack_ts": ts,
            }))
            .collect();
        results.sort_by(|a, b| {
            let ua = a.get("ack_up_to").and_then(|v| v.as_u64()).unwrap_or(0);
            let ub = b.get("ack_up_to").and_then(|v| v.as_u64()).unwrap_or(0);
            ub.cmp(&ua)
        });
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_emoji_stats",
        description = "Topic-wide reaction aggregator on agent-chat-arc. Walks the topic, filters `msg_type=reaction` envelopes, groups by emoji (the reaction payload), and counts uses + tracks `last_used_ts`. Returns `[{emoji, count, last_used_ts}, ...]` sorted by count desc. Zooms OUT from `termlink_agent_reactions` (per-offset) — answers \"what's resonating across the whole topic?\". MCP-side equivalent of `agent emoji-stats` (CLI T-1538). `limit` defaults to 50, capped at 500."
    )]
    async fn termlink_agent_emoji_stats(
        &self,
        Parameters(p): Parameters<AgentEmojiStatsParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(50).min(500);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut stats: std::collections::HashMap<String, (u64, i64)> = std::collections::HashMap::new();
        for env in &all {
            if env.get("msg_type").and_then(|v| v.as_str()) != Some("reaction") {
                continue;
            }
            let emoji = env.get("payload")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if emoji.is_empty() { continue; }
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let entry = stats.entry(emoji).or_insert((0, 0));
            entry.0 += 1;
            if ts > entry.1 { entry.1 = ts; }
        }
        let mut results: Vec<serde_json::Value> = stats
            .into_iter()
            .map(|(emoji, (count, last_ts))| serde_json::json!({
                "emoji": emoji,
                "count": count,
                "last_used_ts": last_ts,
            }))
            .collect();
        results.sort_by(|a, b| {
            let ca = a.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let cb = b.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            cb.cmp(&ca)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_ack_history",
        description = "Per-sender receipt timeline on agent-chat-arc. Walks the topic, filters `msg_type=receipt` envelopes by `sender_id` (defaults to caller's local Identity), and returns `[{up_to, ts_unix_ms}, ...]` sorted newest-first. Zooms IN from `termlink_agent_ack_status` (current frontier per sender across whole topic) — answers \"show me one sender's full ack timeline\". MCP-side equivalent of `agent ack-history` (CLI T-1539 family). `limit` defaults to 200, capped at 1000."
    )]
    async fn termlink_agent_ack_history(
        &self,
        Parameters(p): Parameters<AgentAckHistoryParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(200).min(1000);
        let sender_id = match p.sender_id {
            Some(s) => s,
            None => {
                let home = match std::env::var("HOME") {
                    Ok(h) => h,
                    Err(_) => return json_err("HOME not set"),
                };
                let identity_dir = std::path::PathBuf::from(home).join(".termlink");
                match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
                    Ok(i) => i.fingerprint().to_string(),
                    Err(e) => return json_err(format!("identity load: {e}")),
                }
            }
        };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("receipt"))
            .filter(|env| env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("") == sender_id)
            .map(|env| {
                let up_to = env.get("metadata")
                    .and_then(|m| m.get("up_to"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| env.get("metadata").and_then(|m| m.get("up_to")).and_then(|v| v.as_u64()))
                    .unwrap_or(0);
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                serde_json::json!({
                    "up_to": up_to,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::json!({
            "sender_id": sender_id,
            "history": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_edits_of",
        description = "List the full edit history of a chat-arc envelope. Walks the topic, filters `msg_type=edit` envelopes whose `metadata.replaces` matches the given offset, and returns `[{edit_offset, sender_id, payload_b64, ts_unix_ms}, ...]` sorted oldest-first (chronological revision history). Lets MCP-aware agents see every revision a post went through — useful for audit and conversation provenance. MCP-side equivalent of `agent edits-of <offset>` (CLI T-1517). Companion read tool to `termlink_agent_edit` (T-1567)."
    )]
    async fn termlink_agent_edits_of(
        &self,
        Parameters(p): Parameters<AgentEditsOfParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let target_offset = p.offset;
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("edit"))
            .filter(|env| {
                let replaces = env.get("metadata")
                    .and_then(|m| m.get("replaces"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .or_else(|| env.get("metadata").and_then(|m| m.get("replaces")).and_then(|v| v.as_u64()));
                replaces == Some(target_offset)
            })
            .map(|env| {
                let edit_offset = env.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let payload_b64 = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                serde_json::json!({
                    "edit_offset": edit_offset,
                    "sender_id": sender,
                    "payload_b64": payload_b64,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            ta.cmp(&tb)
        });
        serde_json::to_string_pretty(&serde_json::json!({
            "target_offset": target_offset,
            "edits": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_topic_stats",
        description = "Daily activity buckets on agent-chat-arc. Walks the topic, groups envelopes by date (UTC YYYY-MM-DD from ts_unix_ms), and aggregates total + by_msg_type per day. Returns `[{date, total, by_msg_type}, ...]` sorted by date ascending. Activity heatmap — answers \"when is this topic most active?\". MCP-side equivalent of `agent topic-stats` (CLI T-1531). Optional `window_days` truncates older buckets."
    )]
    async fn termlink_agent_topic_stats(
        &self,
        Parameters(p): Parameters<AgentTopicStatsParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        // YYYY-MM-DD bucket → (total, HashMap<msg_type, count>)
        let mut buckets: std::collections::BTreeMap<String, (u64, std::collections::HashMap<String, u64>)> = std::collections::BTreeMap::new();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let cutoff_ms: Option<i64> = p.window_days.map(|d| now_ms - (d as i64) * 86_400_000);
        for env in &all {
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            if ts <= 0 { continue; }
            if let Some(cutoff) = cutoff_ms { if ts < cutoff { continue; } }
            let day_secs = ts / 1000;
            // Compute UTC date from epoch seconds (no chrono dep — simple integer math)
            let days_since_epoch = day_secs / 86_400;
            let date_str = epoch_days_to_ymd(days_since_epoch);
            let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("post").to_string();
            let entry = buckets.entry(date_str).or_insert((0, std::collections::HashMap::new()));
            entry.0 += 1;
            *entry.1.entry(mt).or_insert(0) += 1;
        }
        let results: Vec<serde_json::Value> = buckets
            .into_iter()
            .map(|(date, (total, by_type))| {
                let mut by_type_json = serde_json::Map::new();
                for (k, v) in by_type {
                    by_type_json.insert(k, serde_json::Value::Number(v.into()));
                }
                serde_json::json!({
                    "date": date,
                    "total": total,
                    "by_msg_type": by_type_json,
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_active_now",
        description = "Senders active within the last N minutes on agent-chat-arc. Walks the topic, filters envelopes whose `ts_unix_ms >= now - window`, groups by `sender_id`, returns `[{sender_id, posts_in_window, last_post_ts}, ...]` sorted by `last_post_ts` desc. Companion to `termlink_agent_peers` (all-time directory) — answers \"who's around right now?\" without dumping the full participant list. Default window 60 minutes; capped at 500 senders."
    )]
    async fn termlink_agent_active_now(
        &self,
        Parameters(p): Parameters<AgentActiveNowParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let window_min = p.window_minutes.unwrap_or(60);
        let limit = p.limit.unwrap_or(100).min(500);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let cutoff = now_ms - (window_min as i64) * 60_000;
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut activity: std::collections::HashMap<String, (u64, i64)> = std::collections::HashMap::new();
        for env in &all {
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            if ts < cutoff { continue; }
            let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if sender.is_empty() { continue; }
            let entry = activity.entry(sender).or_insert((0, 0));
            entry.0 += 1;
            if ts > entry.1 { entry.1 = ts; }
        }
        let mut results: Vec<serde_json::Value> = activity
            .into_iter()
            .map(|(sender, (count, last_ts))| serde_json::json!({
                "sender_id": sender,
                "posts_in_window": count,
                "last_post_ts": last_ts,
            }))
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("last_post_ts").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("last_post_ts").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::json!({
            "window_minutes": window_min,
            "now_ts": now_ms,
            "active": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_history",
        description = "List a single sender's content posts on agent-chat-arc. Walks the topic, filters envelopes by `sender_id` (defaults to caller's local Identity), excludes meta types (reaction/edit/redaction/topic_metadata/receipt/pin/star), sorts newest-first. Returns `[{offset, payload_b64, ts_unix_ms}, ...]`. Per-sender feed — complement to `termlink_agent_peers` (all-time aggregate-only directory). Default limit 50, capped at 500."
    )]
    async fn termlink_agent_history(
        &self,
        Parameters(p): Parameters<AgentHistoryParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(50).min(500);
        let sender_id = match p.sender_id {
            Some(s) => s,
            None => {
                let home = match std::env::var("HOME") {
                    Ok(h) => h,
                    Err(_) => return json_err("HOME not set"),
                };
                let identity_dir = std::path::PathBuf::from(home).join(".termlink");
                match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
                    Ok(i) => i.fingerprint().to_string(),
                    Err(e) => return json_err(format!("identity load: {e}")),
                }
            }
        };
        let meta_types = ["reaction", "edit", "redaction", "topic_metadata", "receipt", "pin", "star"];
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("") == sender_id)
            .filter(|env| {
                let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
                !meta_types.contains(&mt)
            })
            .map(|env| {
                let offset = env.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
                let payload_b64 = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let msg_type = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("post").to_string();
                serde_json::json!({
                    "offset": offset,
                    "msg_type": msg_type,
                    "payload_b64": payload_b64,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::json!({
            "sender_id": sender_id,
            "posts": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_followups",
        description = "Reverse-link aggregator for a chat-arc envelope. Given an offset, walks the topic and finds EVERY envelope that references it: replies (msg_type=post + metadata.in_reply_to), edits (msg_type=edit + metadata.replaces), redactions (msg_type=redaction + metadata.redacts), pins (msg_type=pin + metadata.pin_target), stars (msg_type=star + metadata.star_target), reactions (msg_type=reaction + metadata.in_reply_to). Returns `{target_offset, replies: [...], edits: [...], redactions: [...], pins: [...], stars: [...], reactions: [...], total}` — single-call answer to \"what happened to this post?\". No CLI mirror — purely MCP-side composite read."
    )]
    async fn termlink_agent_followups(
        &self,
        Parameters(p): Parameters<AgentFollowupsParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let target = p.offset;
        let target_str = target.to_string();
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let metadata_matches = |env: &serde_json::Value, key: &str| -> bool {
            let md = match env.get("metadata") { Some(m) => m, None => return false };
            let v = match md.get(key) { Some(v) => v, None => return false };
            v.as_str() == Some(target_str.as_str())
                || v.as_u64() == Some(target)
                || v.as_str().and_then(|s| s.parse::<u64>().ok()) == Some(target)
        };
        let summarize = |env: &serde_json::Value| -> serde_json::Value {
            let offset = env.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
            let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let payload = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("").to_string();
            serde_json::json!({
                "offset": offset,
                "sender_id": sender,
                "ts_unix_ms": ts,
                "payload_b64": payload,
            })
        };
        let mut replies = Vec::new();
        let mut edits = Vec::new();
        let mut redactions = Vec::new();
        let mut pins = Vec::new();
        let mut stars = Vec::new();
        let mut reactions = Vec::new();
        for env in &all {
            let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
            match mt {
                "edit" if metadata_matches(env, "replaces") => edits.push(summarize(env)),
                "redaction" if metadata_matches(env, "redacts") => redactions.push(summarize(env)),
                "pin" if metadata_matches(env, "pin_target") => pins.push(summarize(env)),
                "star" if metadata_matches(env, "star_target") => stars.push(summarize(env)),
                "reaction" if metadata_matches(env, "in_reply_to") => reactions.push(summarize(env)),
                "" | "post" if metadata_matches(env, "in_reply_to") => replies.push(summarize(env)),
                _ => {}
            }
        }
        let total = replies.len() + edits.len() + redactions.len() + pins.len() + stars.len() + reactions.len();
        serde_json::to_string_pretty(&serde_json::json!({
            "target_offset": target,
            "replies": replies,
            "edits": edits,
            "redactions": redactions,
            "pins": pins,
            "stars": stars,
            "reactions": reactions,
            "total": total,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_state",
        description = "Full reduced state snapshot of agent-chat-arc — the curated state right now. Walks the topic and applies the latest-wins reduce-pattern across THREE state targets simultaneously: current pins (pin_target where most recent action=pin), current stars (star_target where most recent star=true), latest topic_metadata description. Returns `{description, pinned: [...], starred: [...], pin_count, star_count, last_update_ts}`. Single-call orientation snapshot — composite of `termlink_agent_pinned` + `termlink_agent_starred` + `termlink_agent_info`(description). For MCP-aware agents that want one read-call to know \"what's the curated state right now?\" without 3 separate calls."
    )]
    async fn termlink_agent_state(
        &self,
        Parameters(_p): Parameters<AgentStateParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        all.sort_by_key(|env| {
            env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0)
        });
        let mut pin_state: std::collections::HashMap<String, (String, i64)> = std::collections::HashMap::new();
        let mut star_state: std::collections::HashMap<String, (String, i64)> = std::collections::HashMap::new();
        let mut description: String = String::new();
        let mut description_ts: i64 = 0;
        let mut last_update_ts: i64 = 0;
        for env in &all {
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
            match mt {
                "pin" => {
                    if let Some(target) = env.get("metadata").and_then(|m| m.get("pin_target")).and_then(|v| v.as_str()) {
                        let action = env.get("metadata").and_then(|m| m.get("action")).and_then(|v| v.as_str()).unwrap_or("pin").to_string();
                        pin_state.insert(target.to_string(), (action, ts));
                        if ts > last_update_ts { last_update_ts = ts; }
                    }
                }
                "star" => {
                    if let Some(target) = env.get("metadata").and_then(|m| m.get("star_target")).and_then(|v| v.as_str()) {
                        let star_val = env.get("metadata").and_then(|m| m.get("star")).and_then(|v| v.as_str()).unwrap_or("true").to_string();
                        star_state.insert(target.to_string(), (star_val, ts));
                        if ts > last_update_ts { last_update_ts = ts; }
                    }
                }
                "topic_metadata" => {
                    use base64::Engine;
                    if let Some(p_b64) = env.get("payload_b64").and_then(|v| v.as_str()) {
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(p_b64) {
                            description = String::from_utf8_lossy(&bytes).to_string();
                            description_ts = ts;
                            if ts > last_update_ts { last_update_ts = ts; }
                        }
                    }
                }
                _ => {}
            }
        }
        let pinned: Vec<serde_json::Value> = pin_state.into_iter()
            .filter(|(_, (action, _))| action == "pin")
            .map(|(target, (_, ts))| serde_json::json!({"pin_target": target, "ts_unix_ms": ts}))
            .collect();
        let starred: Vec<serde_json::Value> = star_state.into_iter()
            .filter(|(_, (val, _))| val == "true")
            .map(|(target, (_, ts))| serde_json::json!({"star_target": target, "ts_unix_ms": ts}))
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "description": description,
            "description_ts": description_ts,
            "pinned": pinned,
            "pin_count": pinned.len(),
            "starred": starred,
            "star_count": starred.len(),
            "last_update_ts": last_update_ts,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_topic_metadata_history",
        description = "Chronological audit log of topic_metadata description changes on agent-chat-arc. Walks the topic, filters `msg_type=topic_metadata`, decodes each `payload_b64` as the description text, and returns `[{description, sender_id, ts_unix_ms}, ...]` sorted oldest-first. Companion to `termlink_agent_info` (which only returns the current description) — answers \"how has this topic's description evolved over time?\". Default limit 100, capped at 500."
    )]
    async fn termlink_agent_topic_metadata_history(
        &self,
        Parameters(p): Parameters<AgentTopicMetadataHistoryParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(100).min(500);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("topic_metadata"))
            .map(|env| {
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let description = env.get("payload_b64").and_then(|v| v.as_str())
                    .and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok())
                    .map(|b| String::from_utf8_lossy(&b).to_string())
                    .unwrap_or_default();
                serde_json::json!({
                    "description": description,
                    "sender_id": sender,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            ta.cmp(&tb)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_reactions_by",
        description = "Per-sender reaction history on agent-chat-arc. Walks the topic, filters `msg_type=reaction` by `sender_id` (defaults to caller's local Identity), and returns `[{emoji, in_reply_to, ts_unix_ms}, ...]` sorted newest-first. Triangulates with `termlink_agent_reactions` (per-offset) and `termlink_agent_emoji_stats` (topic-wide) — three orthogonal views of the same reaction data: by-target, by-emoji, by-sender. Default limit 200, capped at 1000."
    )]
    async fn termlink_agent_reactions_by(
        &self,
        Parameters(p): Parameters<AgentReactionsByParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(200).min(1000);
        let sender_id = match p.sender_id {
            Some(s) => s,
            None => {
                let home = match std::env::var("HOME") {
                    Ok(h) => h,
                    Err(_) => return json_err("HOME not set"),
                };
                let identity_dir = std::path::PathBuf::from(home).join(".termlink");
                match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
                    Ok(i) => i.fingerprint().to_string(),
                    Err(e) => return json_err(format!("identity load: {e}")),
                }
            }
        };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("reaction"))
            .filter(|env| env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("") == sender_id)
            .map(|env| {
                let emoji = env.get("payload").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let target = env.get("metadata")
                    .and_then(|m| m.get("in_reply_to"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                serde_json::json!({
                    "emoji": emoji,
                    "in_reply_to": target,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::json!({
            "sender_id": sender_id,
            "reactions": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_pinned_by",
        description = "List targets currently pinned BY a specific sender on agent-chat-arc. Walks the topic, filters `msg_type=pin` envelopes by `sender_id` (defaults to caller's local Identity), applies the latest-wins reduce per pin_target (an unpin event from the same sender supersedes their earlier pin), and returns `[{pin_target, ts_unix_ms}, ...]` sorted newest-first. Per-curator companion to `termlink_agent_pinned` (topic-wide). Useful for \"what has X curated?\" or \"what have I pinned?\" (default sender_id = me)."
    )]
    async fn termlink_agent_pinned_by(
        &self,
        Parameters(p): Parameters<AgentPinnedByParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let sender_id = match p.sender_id {
            Some(s) => s,
            None => {
                let home = match std::env::var("HOME") {
                    Ok(h) => h,
                    Err(_) => return json_err("HOME not set"),
                };
                let identity_dir = std::path::PathBuf::from(home).join(".termlink");
                match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
                    Ok(i) => i.fingerprint().to_string(),
                    Err(e) => return json_err(format!("identity load: {e}")),
                }
            }
        };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        all.sort_by_key(|env| {
            env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0)
        });
        let mut state: std::collections::HashMap<String, (String, i64)> = std::collections::HashMap::new();
        for env in &all {
            if env.get("msg_type").and_then(|v| v.as_str()) != Some("pin") { continue; }
            if env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("") != sender_id { continue; }
            let target = match env.get("metadata").and_then(|m| m.get("pin_target")).and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => continue,
            };
            let action = env.get("metadata").and_then(|m| m.get("action")).and_then(|v| v.as_str()).unwrap_or("pin").to_string();
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            state.insert(target, (action, ts));
        }
        let mut results: Vec<serde_json::Value> = state.into_iter()
            .filter(|(_, (action, _))| action == "pin")
            .map(|(target, (_, ts))| serde_json::json!({"pin_target": target, "ts_unix_ms": ts}))
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        serde_json::to_string_pretty(&serde_json::json!({
            "sender_id": sender_id,
            "pinned": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_starred_by",
        description = "List targets currently starred BY a specific sender on agent-chat-arc. Walks the topic, filters `msg_type=star` envelopes by `sender_id` (defaults to caller's local Identity), applies the latest-wins reduce per star_target (a `star=false` event from the same sender supersedes their earlier `star=true`), and returns `[{star_target, ts_unix_ms}, ...]` sorted newest-first. Per-curator companion to `termlink_agent_starred` (topic-wide). Useful for \"what has X bookmarked?\" or \"what have I starred?\" (default sender_id = me)."
    )]
    async fn termlink_agent_starred_by(
        &self,
        Parameters(p): Parameters<AgentStarredByParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let sender_id = match p.sender_id {
            Some(s) => s,
            None => {
                let home = match std::env::var("HOME") {
                    Ok(h) => h,
                    Err(_) => return json_err("HOME not set"),
                };
                let identity_dir = std::path::PathBuf::from(home).join(".termlink");
                match termlink_session::agent_identity::Identity::load_or_create(&identity_dir) {
                    Ok(i) => i.fingerprint().to_string(),
                    Err(e) => return json_err(format!("identity load: {e}")),
                }
            }
        };
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        all.sort_by_key(|env| {
            env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0)
        });
        let mut state: std::collections::HashMap<String, (String, i64)> = std::collections::HashMap::new();
        for env in &all {
            if env.get("msg_type").and_then(|v| v.as_str()) != Some("star") { continue; }
            if env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("") != sender_id { continue; }
            let target = match env.get("metadata").and_then(|m| m.get("star_target")).and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => continue,
            };
            let star_val = env.get("metadata").and_then(|m| m.get("star")).and_then(|v| v.as_str()).unwrap_or("true").to_string();
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            state.insert(target, (star_val, ts));
        }
        let mut results: Vec<serde_json::Value> = state.into_iter()
            .filter(|(_, (val, _))| val == "true")
            .map(|(target, (_, ts))| serde_json::json!({"star_target": target, "ts_unix_ms": ts}))
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        serde_json::to_string_pretty(&serde_json::json!({
            "sender_id": sender_id,
            "starred": results,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_info",
        description = "Topic snapshot for agent-chat-arc. Walks the topic and returns aggregate metadata: `{total, by_msg_type, unique_senders, last_activity_ts, description}`. The `description` field contains the latest payload from `msg_type=topic_metadata` envelopes (set via `termlink_agent_describe`). Single-call orientation primitive — what an MCP-aware agent fetches first when joining chat-arc. MCP-side equivalent of `agent info` (CLI T-1524)."
    )]
    async fn termlink_agent_info(
        &self,
        Parameters(_p): Parameters<AgentInfoParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let total = all.len();
        let mut by_msg_type: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut senders: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut last_activity_ts: i64 = 0;
        let mut latest_desc_ts: i64 = -1;
        let mut latest_desc: String = String::new();
        for env in &all {
            let mt = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
            *by_msg_type.entry(mt.clone()).or_insert(0) += 1;
            if let Some(s) = env.get("sender_id").and_then(|v| v.as_str()) {
                if !s.is_empty() { senders.insert(s.to_string()); }
            }
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            if ts > last_activity_ts { last_activity_ts = ts; }
            if mt == "topic_metadata" && ts > latest_desc_ts {
                let p_b64 = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
                if let Ok(b) = base64::engine::general_purpose::STANDARD.decode(p_b64) {
                    latest_desc = String::from_utf8_lossy(&b).into_owned();
                    latest_desc_ts = ts;
                }
            }
        }
        let by_msg_type_json: serde_json::Map<String, serde_json::Value> = by_msg_type
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v)))
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "topic": topic,
            "total": total,
            "by_msg_type": by_msg_type_json,
            "unique_senders": senders.len(),
            "last_activity_ts": last_activity_ts,
            "description": latest_desc,
        })).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_peers",
        description = "Participant directory for agent-chat-arc. Walks the topic, groups envelopes by `sender_id`, and returns `[{sender_id, post_count, last_post_ts}, ...]` sorted by last_post_ts desc. Single-call companion to `termlink_agent_info` — together they form the orientation primitives (info → peers → recent). MCP-side equivalent of `agent peers` (CLI T-1520). `limit` defaults to 200, capped at 1000."
    )]
    async fn termlink_agent_peers(
        &self,
        Parameters(p): Parameters<AgentPeersParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(200).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut peers: std::collections::HashMap<String, (u64, i64)> = std::collections::HashMap::new();
        for env in &all {
            let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if sender.is_empty() { continue; }
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let entry = peers.entry(sender).or_insert((0, 0));
            entry.0 += 1;
            if ts > entry.1 { entry.1 = ts; }
        }
        let mut results: Vec<serde_json::Value> = peers
            .into_iter()
            .map(|(sender, (count, last_ts))| serde_json::json!({
                "sender_id": sender,
                "post_count": count,
                "last_post_ts": last_ts,
            }))
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("last_post_ts").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("last_post_ts").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_on_thread",
        description = "Return the full thread tree rooted at an offset on agent-chat-arc. Walks the topic, builds a parent→children index from `metadata.in_reply_to`, BFS-collects all descendants from the root, and returns the envelopes (root + descendants) sorted chronologically (ts ascending). MCP-side equivalent of `agent on-thread <root>` (CLI T-1493). `limit` defaults to 200, capped at 1000."
    )]
    async fn termlink_agent_on_thread(
        &self,
        Parameters(p): Parameters<AgentOnThreadParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let root = p.root_offset.to_string();
        let limit = p.limit.unwrap_or(200).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        // Build parent_offset -> Vec<envelope>
        let mut by_parent: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();
        let mut by_offset: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        for env in &all {
            let off = env.get("offset").and_then(|v| v.as_u64()).map(|u| u.to_string()).unwrap_or_default();
            if !off.is_empty() {
                by_offset.insert(off.clone(), env.clone());
            }
            let parent = env.get("metadata")
                .and_then(|m| m.get("in_reply_to"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !parent.is_empty() {
                by_parent.entry(parent.to_string()).or_default().push(env.clone());
            }
        }
        // BFS from root.
        let mut collected: Vec<serde_json::Value> = Vec::new();
        if let Some(root_env) = by_offset.get(&root) {
            collected.push(root_env.clone());
        }
        let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        queue.push_back(root.clone());
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        seen.insert(root.clone());
        while let Some(parent) = queue.pop_front() {
            if let Some(children) = by_parent.get(&parent) {
                for child in children {
                    let coff = child.get("offset").and_then(|v| v.as_u64()).map(|u| u.to_string()).unwrap_or_default();
                    if coff.is_empty() || seen.contains(&coff) { continue; }
                    seen.insert(coff.clone());
                    collected.push(child.clone());
                    queue.push_back(coff);
                }
            }
        }
        // Sort chronologically.
        collected.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| a.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| b.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ta.cmp(&tb)
        });
        collected.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(collected)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_reactions",
        description = "List all reaction envelopes for a single chat-arc offset. Walks the topic, filters `msg_type=reaction` with `metadata.in_reply_to=<offset>`, base64-decodes payload (emoji) and returns `[{emoji, sender_id, ts_unix_ms}, ...]` sorted newest-first. MCP-side equivalent of `agent reactions <offset>` (CLI T-1514). Companion read tool to `termlink_agent_react` (T-1562)."
    )]
    async fn termlink_agent_reactions(
        &self,
        Parameters(p): Parameters<AgentReactionsParams>,
    ) -> String {
        use base64::Engine;
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let target = p.offset.to_string();
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        let mut results: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("reaction"))
            .filter(|env| {
                env.get("metadata")
                    .and_then(|m| m.get("in_reply_to"))
                    .and_then(|v| v.as_str()) == Some(target.as_str())
            })
            .map(|env| {
                let p_b64 = env.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("");
                let emoji = match base64::engine::general_purpose::STANDARD.decode(p_b64) {
                    Ok(b) => String::from_utf8_lossy(&b).into_owned(),
                    Err(_) => String::new(),
                };
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                serde_json::json!({
                    "emoji": emoji,
                    "sender_id": sender,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_quote",
        description = "Fetch a single agent-chat-arc envelope by its offset. Walks the topic via channel.subscribe and returns the raw envelope (offset, ts, sender_id, msg_type, payload_b64, metadata, signature). Returns `{\"error\":\"...\"}` if no envelope matches. MCP-side equivalent of `agent quote <offset>` (CLI T-1505). Useful when an agent has an offset reference (from a reaction, reply, or pin) and needs to resolve the original post."
    )]
    async fn termlink_agent_quote(
        &self,
        Parameters(p): Parameters<AgentQuoteParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let target_offset = p.offset;
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            for env in &msgs {
                let off = env.get("offset").and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
                if off == target_offset {
                    return serde_json::to_string_pretty(env).unwrap_or_else(json_err);
                }
            }
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        json_err(format!("offset {target_offset} not found on agent-chat-arc"))
    }

    #[tool(
        name = "termlink_agent_threads",
        description = "List thread roots on agent-chat-arc — i.e. offsets that have been replied to. Walks the topic, scans every envelope's `metadata.in_reply_to`, aggregates by parent offset, and returns `[{root_offset, reply_count, last_reply_ts}, ...]` sorted by last_reply_ts desc. Surfaces conversation hot-spots so MCP-aware agents can see what's being discussed without dumping the full topic. MCP-side equivalent of `agent threads` (CLI T-1533). `limit` defaults to 100, max 1000."
    )]
    async fn termlink_agent_threads(
        &self,
        Parameters(p): Parameters<AgentThreadsParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(100).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        // Aggregate parents-by-reply-count
        let mut parents: std::collections::HashMap<String, (u64, i64)> = std::collections::HashMap::new();
        for env in &all {
            let parent = env.get("metadata")
                .and_then(|m| m.get("in_reply_to"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if parent.is_empty() { continue; }
            let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let entry = parents.entry(parent.to_string()).or_insert((0, 0));
            entry.0 += 1;
            if ts > entry.1 { entry.1 = ts; }
        }
        let mut results: Vec<serde_json::Value> = parents
            .into_iter()
            .map(|(root, (count, last_ts))| serde_json::json!({
                "root_offset": root,
                "reply_count": count,
                "last_reply_ts": last_ts,
            }))
            .collect();
        results.sort_by(|a, b| {
            let ta = a.get("last_reply_ts").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("last_reply_ts").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        results.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(results)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_pinned",
        description = "List currently pinned posts on agent-chat-arc. Walks pin envelopes via channel.subscribe, groups by `metadata.pin_target`, keeps the latest by ts, and returns only those whose final `action` is `pin` (i.e. not subsequently unpinned). Returns a JSON array sorted newest-first: `[{pin_target, sender_id, ts_unix_ms}, ...]`. MCP-side equivalent of `agent pinned` (CLI T-1517). Companion read tool to `termlink_agent_pin` (T-1564) — completes the curation surface."
    )]
    async fn termlink_agent_pinned(
        &self,
        Parameters(p): Parameters<AgentPinnedParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(100).min(1000);
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        // Reduce: walk pin envelopes in ts order, latest-wins per pin_target.
        let mut pin_envelopes: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("pin"))
            .collect();
        pin_envelopes.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| a.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| b.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ta.cmp(&tb)
        });
        let mut latest: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
        for env in pin_envelopes {
            let target = env.get("metadata")
                .and_then(|m| m.get("pin_target"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if target.is_empty() { continue; }
            latest.insert(target, env);
        }
        let mut result_entries: Vec<serde_json::Value> = latest
            .into_values()
            .filter(|env| {
                let action = env.get("metadata")
                    .and_then(|m| m.get("action"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("pin");
                action == "pin"
            })
            .map(|env| {
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let target = env.get("metadata")
                    .and_then(|m| m.get("pin_target"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                serde_json::json!({
                    "pin_target": target,
                    "sender_id": sender,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        result_entries.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        result_entries.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(result_entries)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_agent_starred",
        description = "List currently starred posts on agent-chat-arc. Walks star envelopes via channel.subscribe, groups by `(sender_id, metadata.star_target)`, keeps the latest by ts, and returns entries whose final `metadata.star` is `true` (i.e. not subsequently unstarred). Personal bookmark view — optional `peer_fp` filter scopes to one user's stars (default = all peers). Returns a JSON array sorted newest-first: `[{star_target, sender_id, ts_unix_ms}, ...]`. MCP-side equivalent of `agent starred` (CLI T-1518). Companion read tool to `termlink_agent_star` (T-1565)."
    )]
    async fn termlink_agent_starred(
        &self,
        Parameters(p): Parameters<AgentStarredParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let topic = "agent-chat-arc";
        let limit = p.limit.unwrap_or(100).min(1000);
        let peer = p.peer_fp.as_deref();
        let mut all: Vec<serde_json::Value> = Vec::new();
        let mut cursor: u64 = 0;
        let page_limit: u64 = 1000;
        loop {
            let resp = match termlink_session::client::rpc_call(
                &hub_socket,
                termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
                serde_json::json!({"topic": topic, "cursor": cursor, "limit": page_limit}),
            )
            .await
            {
                Ok(r) => r,
                Err(e) => return json_err(format!("RPC call failed: {e}")),
            };
            let result = match termlink_session::client::unwrap_result(resp) {
                Ok(r) => r,
                Err(e) => return json_err(format!("Hub returned error: {e}")),
            };
            let msgs = result["messages"].as_array().cloned().unwrap_or_default();
            let n = msgs.len();
            all.extend(msgs);
            cursor = result["next_cursor"].as_u64().unwrap_or(cursor);
            if (n as u64) < page_limit {
                break;
            }
        }
        // Reduce: walk star envelopes in ts order, latest-wins per (sender_id, star_target).
        let mut star_envelopes: Vec<serde_json::Value> = all
            .into_iter()
            .filter(|env| env.get("msg_type").and_then(|v| v.as_str()) == Some("star"))
            .filter(|env| {
                if let Some(want) = peer {
                    env.get("sender_id").and_then(|v| v.as_str()) == Some(want)
                } else {
                    true
                }
            })
            .collect();
        star_envelopes.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| a.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64())
                .or_else(|| b.get("ts").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            ta.cmp(&tb)
        });
        let mut latest: std::collections::HashMap<(String, String), serde_json::Value> = std::collections::HashMap::new();
        for env in star_envelopes {
            let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let target = env.get("metadata")
                .and_then(|m| m.get("star_target"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if target.is_empty() { continue; }
            latest.insert((sender, target), env);
        }
        let mut result_entries: Vec<serde_json::Value> = latest
            .into_values()
            .filter(|env| {
                let star = env.get("metadata")
                    .and_then(|m| m.get("star"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("true");
                star == "true"
            })
            .map(|env| {
                let ts = env.get("ts_unix_ms").and_then(|v| v.as_i64())
                    .or_else(|| env.get("ts").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let sender = env.get("sender_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let target = env.get("metadata")
                    .and_then(|m| m.get("star_target"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                serde_json::json!({
                    "star_target": target,
                    "sender_id": sender,
                    "ts_unix_ms": ts,
                })
            })
            .collect();
        result_entries.sort_by(|a, b| {
            let ta = a.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            let tb = b.get("ts_unix_ms").and_then(|v| v.as_i64()).unwrap_or(0);
            tb.cmp(&ta)
        });
        result_entries.truncate(limit as usize);
        serde_json::to_string_pretty(&serde_json::Value::Array(result_entries)).unwrap_or_else(json_err)
    }

    #[tool(
        name = "termlink_channel_subscribe",
        description = "Pull messages from a T-1155 bus topic starting at an optional cursor. Returns messages plus a next_cursor for resumption. One-shot — the MCP caller loops externally if needed."
    )]
    async fn termlink_channel_subscribe(
        &self,
        Parameters(p): Parameters<ChannelSubscribeParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let params = serde_json::json!({
            "topic": p.topic,
            "cursor": p.cursor.unwrap_or(0),
            "limit": p.limit.unwrap_or(100),
        });
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_SUBSCRIBE,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("channel.subscribe error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_channel_list",
        description = "List T-1155 bus topics known to the local hub, optionally filtered by prefix. Returns each topic's name and retention policy."
    )]
    async fn termlink_channel_list(
        &self,
        Parameters(p): Parameters<ChannelListParams>,
    ) -> String {
        let hub_socket = termlink_hub::server::hub_socket_path();
        if !hub_socket.exists() {
            return json_err("Hub is not running (no socket found)");
        }
        let params = match p.prefix {
            Some(pref) => serde_json::json!({"prefix": pref}),
            None => serde_json::json!({}),
        };
        match termlink_session::client::rpc_call(
            &hub_socket,
            termlink_protocol::control::method::CHANNEL_LIST,
            params,
        )
        .await
        {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(json_err),
                Err(e) => json_err(format!("channel.list error: {e}")),
            },
            Err(e) => json_err(format!("RPC call failed: {e}")),
        }
    }

    #[tool(
        name = "termlink_channel_queue_status",
        description = "Read-only view of the local T-1161 offline-queue: pending-post count, cap, and head-of-line post metadata. Does not contact the hub."
    )]
    async fn termlink_channel_queue_status(
        &self,
        Parameters(p): Parameters<ChannelQueueStatusParams>,
    ) -> String {
        use termlink_session::offline_queue::{default_queue_path, OfflineQueue};
        let path = match p.queue_path {
            Some(s) => std::path::PathBuf::from(s),
            None => default_queue_path(),
        };
        if !path.exists() {
            return serde_json::to_string_pretty(&serde_json::json!({
                "queue_path": path.display().to_string(),
                "exists": false,
                "pending": 0,
            }))
            .unwrap_or_else(json_err);
        }
        let queue = match OfflineQueue::open(&path) {
            Ok(q) => q,
            Err(e) => return json_err(format!("Failed to open offline queue: {e}")),
        };
        let size = queue.size().unwrap_or(0);
        let head = queue.peek_oldest().ok().flatten();
        let head_json = head.map(|(id, post)| {
            serde_json::json!({
                "queue_id": id.0,
                "topic": post.topic,
                "msg_type": post.msg_type,
                "ts_unix_ms": post.ts_unix_ms,
                "sender_id": post.sender_id,
                "artifact_ref": post.artifact_ref,
            })
        });
        serde_json::to_string_pretty(&serde_json::json!({
            "queue_path": path.display().to_string(),
            "exists": true,
            "cap": queue.cap(),
            "pending": size,
            "oldest": head_json,
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
    fn run_params_with_env_and_cwd() {
        let json = serde_json::json!({
            "command": "echo hello",
            "timeout": 10,
            "cwd": "/tmp",
            "env": {"FOO": "bar", "PATH": "/usr/bin"},
        });
        let p: RunParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.command, "echo hello");
        assert_eq!(p.timeout, Some(10));
        assert_eq!(p.cwd.as_deref(), Some("/tmp"));
        let env = p.env.as_ref().unwrap();
        assert_eq!(env.get("FOO").unwrap(), "bar");
    }

    #[test]
    fn run_params_minimal() {
        let json = serde_json::json!({"command": "ls"});
        let p: RunParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.command, "ls");
        assert!(p.timeout.is_none());
        assert!(p.cwd.is_none());
        assert!(p.env.is_none());
    }

    #[test]
    fn spawn_params_defaults() {
        let json = serde_json::json!({});
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert!(p.name.is_none());
        assert!(p.roles.is_none());
        assert!(p.tags.is_none());
        assert!(p.cap.is_none());
        assert!(p.env.is_none());
        assert!(p.command.is_none());
        assert!(p.wait.is_none());
        assert!(p.wait_timeout.is_none());
        assert!(p.cwd.is_none());
    }

    #[test]
    fn spawn_params_full() {
        let json = serde_json::json!({
            "name": "builder",
            "roles": ["ci"],
            "tags": ["linux"],
            "cap": ["code", "test"],
            "env": {"API_KEY": "abc123"},
            "command": ["make", "build"],
            "wait": true,
            "wait_timeout": 30
        });
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.name.as_deref(), Some("builder"));
        assert_eq!(p.cap.as_ref().unwrap(), &["code", "test"]);
        assert_eq!(p.env.as_ref().unwrap().get("API_KEY").unwrap(), "abc123");
        assert_eq!(p.command.as_ref().unwrap(), &["make", "build"]);
        assert_eq!(p.wait, Some(true));
        assert!(p.cwd.is_none());
    }

    #[test]
    fn spawn_params_with_cwd() {
        let json = serde_json::json!({
            "name": "builder",
            "command": ["make"],
            "cwd": "/opt/project",
        });
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.cwd.as_deref(), Some("/opt/project"));
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
        assert!(p.since_default.is_none());
    }

    #[test]
    fn collect_params_since_default() {
        let json = serde_json::json!({"since_default": 42});
        let p: CollectParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.since_default, Some(42));
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

    #[test]
    fn dispatch_params_full() {
        let json = serde_json::json!({
            "count": 3,
            "command": ["echo", "hello"],
            "timeout": 60,
            "topic": "build.done",
            "name_prefix": "builder",
            "roles": ["worker"],
            "tags": ["team:infra"],
            "cap": ["code", "review"],
            "env": {"API_KEY": "secret", "DEBUG": "1"},
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.count, 3);
        assert_eq!(p.command, vec!["echo", "hello"]);
        assert_eq!(p.timeout, Some(60));
        assert_eq!(p.topic.as_deref(), Some("build.done"));
        assert_eq!(p.name_prefix.as_deref(), Some("builder"));
        assert_eq!(p.roles.as_ref().unwrap(), &["worker"]);
        assert_eq!(p.tags.as_ref().unwrap(), &["team:infra"]);
        assert_eq!(p.cap.as_ref().unwrap(), &["code", "review"]);
        let env = p.env.as_ref().unwrap();
        assert_eq!(env.get("API_KEY").unwrap(), "secret");
        assert_eq!(env.get("DEBUG").unwrap(), "1");
        assert!(p.workdir.is_none());
    }

    #[test]
    fn dispatch_params_minimal() {
        let json = serde_json::json!({
            "count": 1,
            "command": ["ls"],
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.count, 1);
        assert_eq!(p.command, vec!["ls"]);
        assert!(p.timeout.is_none());
        assert!(p.topic.is_none());
        assert!(p.name_prefix.is_none());
        assert!(p.roles.is_none());
        assert!(p.tags.is_none());
        assert!(p.cap.is_none());
        assert!(p.env.is_none());
        assert!(p.workdir.is_none());
    }

    #[test]
    fn dispatch_params_with_workdir() {
        let json = serde_json::json!({
            "count": 2,
            "command": ["make", "test"],
            "workdir": "/opt/project",
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.count, 2);
        assert_eq!(p.workdir.as_deref(), Some("/opt/project"));
    }

    #[test]
    fn wait_params_with_since() {
        let json = serde_json::json!({
            "target": "worker-1",
            "topic": "task.completed",
            "timeout": 60,
            "since": 42,
        });
        let p: WaitParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "worker-1");
        assert_eq!(p.topic, "task.completed");
        assert_eq!(p.timeout, Some(60));
        assert_eq!(p.since, Some(42));
    }

    #[test]
    fn wait_params_without_since() {
        let json = serde_json::json!({
            "target": "worker-1",
            "topic": "task.completed",
        });
        let p: WaitParams = serde_json::from_value(json).unwrap();
        assert!(p.since.is_none());
        assert!(p.timeout.is_none());
    }

    #[test]
    fn dispatch_params_missing_required() {
        // Missing command
        let json = serde_json::json!({"count": 1});
        assert!(serde_json::from_value::<DispatchParams>(json).is_err());

        // Missing count
        let json = serde_json::json!({"command": ["echo"]});
        assert!(serde_json::from_value::<DispatchParams>(json).is_err());
    }

    #[test]
    fn batch_run_params_full() {
        let json = serde_json::json!({
            "commands": ["echo a", "echo b", "echo c"],
            "timeout": 15,
            "cwd": "/tmp",
            "env": {"FOO": "bar"},
            "max_parallel": 5,
        });
        let p: BatchRunParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.commands.len(), 3);
        assert_eq!(p.timeout, Some(15));
        assert_eq!(p.cwd.as_deref(), Some("/tmp"));
        assert_eq!(p.max_parallel, Some(5));
        assert_eq!(p.env.as_ref().unwrap().get("FOO").unwrap(), "bar");
    }

    #[test]
    fn batch_run_params_minimal() {
        let json = serde_json::json!({"commands": ["ls"]});
        let p: BatchRunParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.commands, vec!["ls"]);
        assert!(p.timeout.is_none());
        assert!(p.cwd.is_none());
        assert!(p.env.is_none());
        assert!(p.max_parallel.is_none());
    }

    #[test]
    fn register_params_full() {
        let json = serde_json::json!({
            "name": "my-agent",
            "roles": ["coder", "reviewer"],
            "tags": ["team-a"],
            "cap": ["events", "kv"]
        });
        let p: RegisterParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.name.as_deref(), Some("my-agent"));
        assert_eq!(p.roles.as_deref(), Some(&["coder".to_string(), "reviewer".to_string()][..]));
        assert_eq!(p.tags.as_deref(), Some(&["team-a".to_string()][..]));
        assert_eq!(p.cap.as_deref(), Some(&["events".to_string(), "kv".to_string()][..]));
    }

    #[test]
    fn register_params_minimal() {
        let json = serde_json::json!({});
        let p: RegisterParams = serde_json::from_value(json).unwrap();
        assert!(p.name.is_none());
        assert!(p.roles.is_none());
        assert!(p.tags.is_none());
        assert!(p.cap.is_none());
    }

    #[test]
    fn deregister_params() {
        let json = serde_json::json!({"session_id": "abc-123"});
        let p: DeregisterParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.session_id, "abc-123");
    }

    #[test]
    fn deregister_params_missing_required() {
        let json = serde_json::json!({});
        assert!(serde_json::from_value::<DeregisterParams>(json).is_err());
    }

    #[test]
    fn shell_escape_safe_string() {
        assert_eq!(shell_escape("hello"), "hello");
        assert_eq!(shell_escape("path/to/file.txt"), "path/to/file.txt");
    }

    #[test]
    fn shell_escape_special_chars() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
        assert_eq!(shell_escape("a;b"), "'a;b'");
    }

    #[test]
    fn shell_escape_single_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn shell_escape_empty() {
        assert_eq!(shell_escape(""), "''");
    }

    // === Task governance tests ===
    // T-1004: Mutex to prevent env var race conditions in parallel test execution
    static GOV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn governance_disabled_allows_without_task_id() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };
        let result = check_task_governance(&None, "termlink_exec");
        assert!(result.is_ok());
    }

    #[test]
    fn governance_disabled_allows_with_task_id() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };
        let result = check_task_governance(&Some("T-123".into()), "termlink_exec");
        assert!(result.is_ok());
    }

    #[test]
    fn governance_enabled_blocks_without_task_id() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("TERMLINK_TASK_GOVERNANCE", "1") };
        let result = check_task_governance(&None, "termlink_spawn");
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("task_id"), "error should mention task_id: {err}");
        assert!(err.contains("termlink_spawn"), "error should mention tool name: {err}");
    }

    #[test]
    fn governance_enabled_allows_with_task_id() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("TERMLINK_TASK_GOVERNANCE", "1") };
        let result = check_task_governance(&Some("T-456".into()), "termlink_dispatch");
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };

        assert!(result.is_ok());
    }

    #[test]
    fn governance_enabled_blocks_empty_task_id() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("TERMLINK_TASK_GOVERNANCE", "1") };
        let result = check_task_governance(&Some("".into()), "termlink_exec");
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };

        assert!(result.is_err());
    }

    #[test]
    fn governance_enabled_blocks_whitespace_task_id() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("TERMLINK_TASK_GOVERNANCE", "1") };
        let result = check_task_governance(&Some("   ".into()), "termlink_interact");
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };

        assert!(result.is_err());
    }

    #[test]
    fn governance_other_values_treated_as_disabled() {
        let _lock = GOV_LOCK.lock().unwrap();
        // "0", "true", "yes" should NOT enable governance — only "1"
        for val in &["0", "true", "yes", "enabled"] {
            unsafe { std::env::set_var("TERMLINK_TASK_GOVERNANCE", val) };
            let result = check_task_governance(&None, "termlink_exec");
            unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };
            assert!(result.is_ok(), "governance should be disabled for value '{val}'");
        }
    }

    #[test]
    fn governance_error_is_valid_json() {
        let _lock = GOV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("TERMLINK_TASK_GOVERNANCE", "1") };
        let result = check_task_governance(&None, "termlink_exec");
        unsafe { std::env::remove_var("TERMLINK_TASK_GOVERNANCE") };

        let err = result.unwrap_err();
        let parsed: serde_json::Value = serde_json::from_str(&err).expect("error should be valid JSON");
        assert_eq!(parsed["ok"], false);
        assert!(parsed["error"].is_string());
    }

    // === Param deserialization tests for task_id ===

    #[test]
    fn exec_params_with_task_id() {
        let json = serde_json::json!({
            "target": "s1",
            "command": "echo hi",
            "task_id": "T-100"
        });
        let p: ExecParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.task_id.as_deref(), Some("T-100"));
    }

    #[test]
    fn exec_params_without_task_id() {
        let json = serde_json::json!({"target": "s1", "command": "echo hi"});
        let p: ExecParams = serde_json::from_value(json).unwrap();
        assert!(p.task_id.is_none());
    }

    #[test]
    fn spawn_params_with_task_id() {
        let json = serde_json::json!({"name": "builder", "task_id": "T-200"});
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.task_id.as_deref(), Some("T-200"));
    }

    #[test]
    fn spawn_params_without_task_id() {
        let json = serde_json::json!({});
        let p: SpawnParams = serde_json::from_value(json).unwrap();
        assert!(p.task_id.is_none());
    }

    #[test]
    fn interact_params_with_task_id() {
        let json = serde_json::json!({
            "target": "s1",
            "command": "ls",
            "task_id": "T-300"
        });
        let p: InteractParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.task_id.as_deref(), Some("T-300"));
    }

    #[test]
    fn interact_params_without_task_id() {
        let json = serde_json::json!({"target": "s1", "command": "ls"});
        let p: InteractParams = serde_json::from_value(json).unwrap();
        assert!(p.task_id.is_none());
    }

    #[test]
    fn dispatch_params_with_task_id() {
        let json = serde_json::json!({
            "count": 2,
            "command": ["echo", "hello"],
            "task_id": "T-400"
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.task_id.as_deref(), Some("T-400"));
    }

    #[test]
    fn dispatch_params_without_task_id() {
        let json = serde_json::json!({"count": 1, "command": ["ls"]});
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert!(p.task_id.is_none());
    }

    #[test]
    fn dispatch_params_with_model() {
        let json = serde_json::json!({
            "count": 2,
            "command": ["echo", "hello"],
            "model": "opus"
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.model.as_deref(), Some("opus"));
    }

    #[test]
    fn dispatch_params_without_model() {
        let json = serde_json::json!({"count": 1, "command": ["ls"]});
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert!(p.model.is_none());
    }

    #[test]
    fn dispatch_params_with_task_type() {
        let json = serde_json::json!({
            "count": 1,
            "command": ["echo"],
            "task_type": "build",
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.task_type.as_deref(), Some("build"));
    }

    #[test]
    fn dispatch_params_default_task_type_none() {
        let json = serde_json::json!({"count": 1, "command": ["echo"]});
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert!(p.task_type.is_none());
    }

    #[test]
    fn resolve_dispatch_model_passthrough_when_breaker_closed() {
        // Fresh cache, fresh breaker state — explicit "opus" should pass through.
        let cache = termlink_hub::route_cache::RouteCache::default();
        // Reset breaker first
        termlink_hub::circuit_breaker::model_global().record_success("opus");
        let (m, fb) = resolve_dispatch_model(Some("opus"), Some("build"), &cache);
        assert_eq!(m.as_deref(), Some("opus"));
        assert!(!fb);
    }

    #[test]
    fn resolve_dispatch_model_uses_best_for_task_type() {
        let mut cache = termlink_hub::route_cache::RouteCache::default();
        // sonnet: 100% success for "test"
        for _ in 0..5 {
            cache.record_model_success("sonnet", "test");
        }
        // haiku: 50/50 for "test"
        cache.record_model_success("haiku", "test");
        cache.record_model_failure("haiku", "test");
        // Reset breakers
        termlink_hub::circuit_breaker::model_global().record_success("sonnet");
        termlink_hub::circuit_breaker::model_global().record_success("haiku");

        let (m, fb) = resolve_dispatch_model(None, Some("test"), &cache);
        assert_eq!(m.as_deref(), Some("sonnet"));
        assert!(!fb);
    }

    #[test]
    fn resolve_dispatch_model_no_inputs_returns_none() {
        let cache = termlink_hub::route_cache::RouteCache::default();
        let (m, fb) = resolve_dispatch_model(None, None, &cache);
        assert!(m.is_none());
        assert!(!fb);
    }

    #[test]
    fn dispatch_params_model_sonnet() {
        let json = serde_json::json!({
            "count": 3,
            "command": ["make", "test"],
            "model": "sonnet",
            "task_id": "T-904"
        });
        let p: DispatchParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.model.as_deref(), Some("sonnet"));
        assert_eq!(p.task_id.as_deref(), Some("T-904"));
        assert_eq!(p.count, 3);
    }

    // === T-920 remote + hub-tcp param tests ===

    #[test]
    fn hub_start_params_with_tcp() {
        let json = serde_json::json!({"tcp_addr": "0.0.0.0:9100"});
        let p: HubStartParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.tcp_addr.as_deref(), Some("0.0.0.0:9100"));
    }

    #[test]
    fn hub_start_params_without_tcp() {
        let json = serde_json::json!({});
        let p: HubStartParams = serde_json::from_value(json).unwrap();
        assert!(p.tcp_addr.is_none());
    }

    #[test]
    fn remote_call_params_full() {
        let json = serde_json::json!({
            "hub": "192.168.10.107:9100",
            "method": "session.discover",
            "params": {"tags": ["master"]},
            "secret_file": "/etc/termlink/secret",
            "scope": "observe",
            "timeout": 15,
        });
        let p: RemoteCallParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.hub, "192.168.10.107:9100");
        assert_eq!(p.method, "session.discover");
        assert_eq!(p.scope.as_deref(), Some("observe"));
        assert_eq!(p.timeout, Some(15));
        assert!(p.secret_file.is_some());
        assert!(p.secret.is_none());
        assert!(p.params.is_some());
    }

    #[test]
    fn remote_call_params_minimal() {
        let json = serde_json::json!({
            "hub": "host:9100",
            "method": "termlink.ping",
            "secret": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        });
        let p: RemoteCallParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.method, "termlink.ping");
        assert!(p.params.is_none());
        assert!(p.scope.is_none());
        assert!(p.timeout.is_none());
        assert_eq!(p.secret.as_ref().unwrap().len(), 64);
    }

    #[test]
    fn remote_call_params_missing_required() {
        // Missing method
        let json = serde_json::json!({"hub": "host:9100"});
        assert!(serde_json::from_value::<RemoteCallParams>(json).is_err());
        // Missing hub
        let json = serde_json::json!({"method": "termlink.ping"});
        assert!(serde_json::from_value::<RemoteCallParams>(json).is_err());
    }

    #[test]
    fn remote_ping_params_hub_only() {
        let json = serde_json::json!({
            "hub": "host:9100",
            "secret_file": "/tmp/s",
        });
        let p: RemotePingParams = serde_json::from_value(json).unwrap();
        assert!(p.session.is_none());
    }

    #[test]
    fn remote_ping_params_with_session() {
        let json = serde_json::json!({
            "hub": "host:9100",
            "session": "worker-1",
            "secret_file": "/tmp/s",
            "scope": "interact",
            "timeout": 5,
        });
        let p: RemotePingParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.session.as_deref(), Some("worker-1"));
        assert_eq!(p.scope.as_deref(), Some("interact"));
        assert_eq!(p.timeout, Some(5));
    }

    #[test]
    fn remote_inject_params_full() {
        let json = serde_json::json!({
            "hub": "192.168.10.107:9100",
            "session": "dashboard-brain",
            "text": "hello from mcp",
            "enter": true,
            "secret_file": "/etc/termlink/secret",
            "scope": "control",
            "timeout": 30,
        });
        let p: RemoteInjectParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.session, "dashboard-brain");
        assert_eq!(p.text, "hello from mcp");
        assert_eq!(p.enter, Some(true));
        assert_eq!(p.scope.as_deref(), Some("control"));
    }

    #[test]
    fn remote_inject_params_defaults() {
        let json = serde_json::json!({
            "hub": "host:9100",
            "session": "a",
            "text": "x",
            "secret": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        });
        let p: RemoteInjectParams = serde_json::from_value(json).unwrap();
        assert!(p.enter.is_none());
        assert!(p.scope.is_none());
        assert!(p.timeout.is_none());
    }

    #[test]
    fn remote_inject_params_missing_required() {
        // Missing text
        let json = serde_json::json!({"hub": "h:1", "session": "s"});
        assert!(serde_json::from_value::<RemoteInjectParams>(json).is_err());
        // Missing session
        let json = serde_json::json!({"hub": "h:1", "text": "x"});
        assert!(serde_json::from_value::<RemoteInjectParams>(json).is_err());
    }

    // === TOFU params tests (T-1038) ===

    #[test]
    fn tofu_list_params_parses_empty() {
        let json = serde_json::json!({});
        let _p: TofuListParams = serde_json::from_value(json).unwrap();
    }

    #[test]
    fn tofu_clear_params_parses() {
        let json = serde_json::json!({"host": "192.168.10.109:9100"});
        let p: TofuClearParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.host, "192.168.10.109:9100");
    }

    #[test]
    fn tofu_clear_params_missing_host() {
        let json = serde_json::json!({});
        assert!(serde_json::from_value::<TofuClearParams>(json).is_err());
    }

    #[test]
    fn fleet_doctor_params_defaults() {
        let json = serde_json::json!({});
        let p: FleetDoctorParams = serde_json::from_value(json).unwrap();
        assert!(p.timeout.is_none());
    }

    #[test]
    fn fleet_doctor_params_with_timeout() {
        let json = serde_json::json!({"timeout": 30});
        let p: FleetDoctorParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.timeout, Some(30));
    }

    // === Hub restart params tests (T-1040) ===

    #[test]
    fn hub_restart_params_parses_empty() {
        let json = serde_json::json!({});
        let _p: HubRestartParams = serde_json::from_value(json).unwrap();
    }

    // === Events params tests (T-1040) ===

    #[test]
    fn events_params_parses_full() {
        let json = serde_json::json!({
            "target": "my-session",
            "since": 42,
            "topic": "file.transfer",
            "timeout": 10,
        });
        let p: EventsParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "my-session");
        assert_eq!(p.since, Some(42));
        assert_eq!(p.topic.as_deref(), Some("file.transfer"));
        assert_eq!(p.timeout, Some(10));
    }

    #[test]
    fn events_params_defaults() {
        let json = serde_json::json!({"target": "sess1"});
        let p: EventsParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.target, "sess1");
        assert!(p.since.is_none());
        assert!(p.topic.is_none());
        assert!(p.timeout.is_none());
    }

    #[test]
    fn events_params_missing_target() {
        let json = serde_json::json!({});
        assert!(serde_json::from_value::<EventsParams>(json).is_err());
    }

}
