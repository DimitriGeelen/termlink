use rmcp::ErrorData as McpError;
use rmcp::model::{
    Annotated, GetPromptRequestParams, GetPromptResult, Implementation,
    ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParams, Prompt, PromptArgument, PromptMessage, PromptMessageRole,
    RawResource, RawResourceTemplate, ReadResourceRequestParams,
    ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{tool_handler, RoleServer, ServerHandler, ServiceExt};

use crate::tools::TermLinkTools;

use termlink_session::{client, manager};

#[tool_handler]
impl ServerHandler for TermLinkTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new(
            "termlink-mcp",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async {
            let mut resources: Vec<Annotated<RawResource>> =
                vec![Annotated::new(
                    RawResource::new("termlink://sessions", "sessions")
                        .with_description("List of all active TermLink sessions")
                        .with_mime_type("application/json"),
                    None,
                )];

            if let Ok(sessions) = manager::list_sessions(false) {
                for s in &sessions {
                    resources.push(Annotated::new(
                        RawResource::new(
                            format!("termlink://sessions/{}", s.id),
                            format!("session:{}", s.display_name),
                        )
                        .with_description(format!(
                            "Status of session '{}' ({})",
                            s.display_name, s.state
                        ))
                        .with_mime_type("application/json"),
                        None,
                    ));
                }
            }

            Ok(ListResourcesResult::with_all_items(resources))
        }
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_
    {
        async {
            let templates: Vec<Annotated<RawResourceTemplate>> = vec![Annotated::new(
                RawResourceTemplate::new(
                    "termlink://sessions/{session_id}",
                    "session_detail",
                )
                .with_description("Detailed status of a TermLink session by ID or name"),
                None,
            )];

            Ok(ListResourceTemplatesResult::with_all_items(templates))
        }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = &request.uri;

            if uri == "termlink://sessions" {
                return read_sessions_list().await;
            }

            if let Some(session_id) = uri.strip_prefix("termlink://sessions/") {
                return read_session_detail(session_id).await;
            }

            Err(McpError::invalid_params(
                format!("Unknown resource URI: {uri}"),
                None,
            ))
        }
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListPromptsResult, McpError>> + Send + '_ {
        async {
            Ok(ListPromptsResult::with_all_items(vec![
                Prompt::new(
                    "debug_session",
                    Some("Diagnose why a TermLink session is not responding or behaving unexpectedly"),
                    Some(vec![
                        PromptArgument::new("session")
                            .with_description("Session ID or display name to debug")
                            .with_required(true),
                    ]),
                ),
                Prompt::new(
                    "session_overview",
                    Some("Get a comprehensive overview of all active TermLink sessions"),
                    None,
                ),
                Prompt::new(
                    "orchestrate",
                    Some("Help coordinate work across multiple TermLink sessions by role or tag"),
                    Some(vec![
                        PromptArgument::new("task")
                            .with_description("Description of the task to coordinate")
                            .with_required(true),
                        PromptArgument::new("role")
                            .with_description("Filter sessions by role (optional)"),
                        PromptArgument::new("tag")
                            .with_description("Filter sessions by tag (optional)"),
                    ]),
                ),
            ]))
        }
    }

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<GetPromptResult, McpError>> + Send + '_ {
        async move {
            let args = request.arguments.unwrap_or_default();

            match request.name.as_str() {
                "debug_session" => build_debug_session_prompt(&args).await,
                "session_overview" => build_session_overview_prompt().await,
                "orchestrate" => build_orchestrate_prompt(&args).await,
                _ => Err(McpError::invalid_params(
                    format!("Unknown prompt: {}", request.name),
                    None,
                )),
            }
        }
    }
}

async fn read_sessions_list() -> Result<ReadResourceResult, McpError> {
    let sessions = manager::list_sessions(false)
        .map_err(|e| McpError::internal_error(format!("Failed to list sessions: {e}"), None))?;

    let items: Vec<serde_json::Value> = sessions
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

    let text = serde_json::to_string_pretty(&items)
        .map_err(|e| McpError::internal_error(format!("JSON error: {e}"), None))?;

    Ok(ReadResourceResult::new(vec![
        ResourceContents::text(text, "termlink://sessions").with_mime_type("application/json"),
    ]))
}

async fn read_session_detail(session_id: &str) -> Result<ReadResourceResult, McpError> {
    let reg = manager::find_session(session_id)
        .map_err(|e| McpError::invalid_params(format!("Session not found: {e}"), None))?;

    let uri = format!("termlink://sessions/{}", reg.id);

    match client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({})).await {
        Ok(resp) => match client::unwrap_result(resp) {
            Ok(status) => {
                let text = serde_json::to_string_pretty(&status)
                    .map_err(|e| McpError::internal_error(format!("JSON error: {e}"), None))?;

                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(text, uri).with_mime_type("application/json"),
                ]))
            }
            Err(e) => Err(McpError::internal_error(
                format!("Status query failed: {e}"),
                None,
            )),
        },
        Err(e) => {
            // Session file exists but not responding — return registration data
            let fallback = serde_json::json!({
                "id": reg.id.as_str(),
                "display_name": reg.display_name,
                "state": "unreachable",
                "pid": reg.pid,
                "tags": reg.tags,
                "roles": reg.roles,
                "error": format!("Connection failed: {e}"),
            });

            let text = serde_json::to_string_pretty(&fallback)
                .map_err(|e| McpError::internal_error(format!("JSON error: {e}"), None))?;

            Ok(ReadResourceResult::new(vec![
                ResourceContents::text(text, uri).with_mime_type("application/json"),
            ]))
        }
    }
}

// === Prompt builders ===

async fn build_debug_session_prompt(
    args: &serde_json::Map<String, serde_json::Value>,
) -> Result<GetPromptResult, McpError> {
    let session = args
        .get("session")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required argument: session", None))?;

    let reg = manager::find_session(session)
        .map_err(|e| McpError::invalid_params(format!("Session not found: {e}"), None))?;

    // Gather diagnostic data
    let mut diagnostics = format!(
        "# Debug Session: {}\n\n## Registration\n- ID: {}\n- PID: {}\n- Display name: {}\n- Tags: {:?}\n- Roles: {:?}\n- Capabilities: {:?}\n",
        session,
        reg.id,
        reg.pid,
        reg.display_name,
        reg.tags,
        reg.roles,
        reg.capabilities,
    );

    // Try live status
    match client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({})).await {
        Ok(resp) => match client::unwrap_result(resp) {
            Ok(status) => {
                diagnostics.push_str(&format!(
                    "\n## Live Status\n```json\n{}\n```\n",
                    serde_json::to_string_pretty(&status).unwrap_or_else(|_| "?".into())
                ));
            }
            Err(e) => {
                diagnostics.push_str(&format!("\n## Live Status\nFailed: {e}\n"));
            }
        },
        Err(e) => {
            diagnostics.push_str(&format!(
                "\n## Live Status\n**UNREACHABLE**: {e}\n\nThe session's socket exists but the process is not responding.\nPossible causes:\n- Process crashed but socket wasn't cleaned up\n- Process is hung/blocked\n- Socket permissions issue\n"
            ));
        }
    }

    // Try to get recent output (if PTY session)
    match client::rpc_call(
        reg.socket_path(),
        "query.output",
        serde_json::json!({"lines": 20, "strip_ansi": true}),
    )
    .await
    {
        Ok(resp) => {
            if let Ok(result) = client::unwrap_result(resp) {
                if let Some(output) = result["output"].as_str() {
                    if !output.trim().is_empty() {
                        diagnostics.push_str(&format!(
                            "\n## Recent Terminal Output (last 20 lines)\n```\n{}\n```\n",
                            output.trim()
                        ));
                    }
                }
            }
        }
        Err(_) => {} // Not a PTY session, skip
    }

    // Check if process is alive
    let pid_alive = unsafe { libc::kill(reg.pid as i32, 0) } == 0;
    diagnostics.push_str(&format!(
        "\n## Process Status\n- PID {} is {}\n",
        reg.pid,
        if pid_alive { "**alive**" } else { "**dead** (zombie registration)" }
    ));

    Ok(GetPromptResult::new(vec![
        PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I need help debugging TermLink session '{}'. Here's what I've gathered:\n\n{}",
                session, diagnostics
            ),
        ),
    ]))
}

async fn build_session_overview_prompt() -> Result<GetPromptResult, McpError> {
    let sessions = manager::list_sessions(false)
        .map_err(|e| McpError::internal_error(format!("Failed to list sessions: {e}"), None))?;

    if sessions.is_empty() {
        return Ok(GetPromptResult::new(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "There are no active TermLink sessions. Help me set up my terminal mesh.",
        )]));
    }

    let mut overview = format!("# TermLink Session Overview\n\n**{} active sessions:**\n\n", sessions.len());

    for s in &sessions {
        overview.push_str(&format!(
            "- **{}** ({})\n  - State: {}\n  - PID: {}\n  - Roles: {}\n  - Tags: {}\n  - Capabilities: {}\n\n",
            s.display_name,
            s.id,
            s.state,
            s.pid,
            if s.roles.is_empty() { "none".to_string() } else { s.roles.join(", ") },
            if s.tags.is_empty() { "none".to_string() } else { s.tags.join(", ") },
            if s.capabilities.is_empty() { "none".to_string() } else { s.capabilities.join(", ") },
        ));
    }

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        format!(
            "Here's my current TermLink session mesh:\n\n{}\nHelp me understand the current state and suggest what I can do with these sessions.",
            overview
        ),
    )]))
}

async fn build_orchestrate_prompt(
    args: &serde_json::Map<String, serde_json::Value>,
) -> Result<GetPromptResult, McpError> {
    let task = args
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required argument: task", None))?;

    let sessions = manager::list_sessions(false)
        .map_err(|e| McpError::internal_error(format!("Failed to list sessions: {e}"), None))?;

    let role_filter = args.get("role").and_then(|v| v.as_str());
    let tag_filter = args.get("tag").and_then(|v| v.as_str());

    let filtered: Vec<_> = sessions
        .iter()
        .filter(|s| {
            role_filter.is_none_or(|r| s.roles.iter().any(|sr| sr == r))
                && tag_filter.is_none_or(|t| s.tags.iter().any(|st| st == t))
        })
        .collect();

    let mut context = format!(
        "# Orchestration Task\n\n**Task:** {}\n\n**Available sessions ({}):**\n\n",
        task,
        filtered.len()
    );

    for s in &filtered {
        context.push_str(&format!(
            "- **{}** — roles: [{}], tags: [{}], caps: [{}]\n",
            s.display_name,
            s.roles.join(", "),
            s.tags.join(", "),
            s.capabilities.join(", "),
        ));
    }

    if filtered.is_empty() {
        context.push_str("*No sessions match the filter criteria.*\n");
    }

    context.push_str(&format!(
        "\n## Available TermLink Tools\n\
         - `termlink_interact` — run a command in a PTY session and get output\n\
         - `termlink_exec` — execute a command on a session (non-PTY)\n\
         - `termlink_emit` / `termlink_emit_to` — send events between sessions\n\
         - `termlink_broadcast` — send events to multiple sessions\n\
         - `termlink_wait` — wait for an event on a session\n\
         - `termlink_spawn` — create new sessions\n\
         - `termlink_kv_set/get` — store metadata on sessions\n"
    ));

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        format!(
            "I need to coordinate work across my TermLink sessions.\n\n{}\n\
             Help me plan and execute this task using the available sessions and tools.",
            context
        ),
    )]))
}

/// Run the MCP server on stdio (stdin/stdout).
pub async fn run_stdio() -> anyhow::Result<()> {
    let server = TermLinkTools::new();
    let (stdin, stdout) = rmcp::transport::io::stdio();
    let service = server.serve((stdin, stdout)).await?;
    service.waiting().await?;
    Ok(())
}
