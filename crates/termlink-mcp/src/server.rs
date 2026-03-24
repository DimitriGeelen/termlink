use rmcp::ErrorData as McpError;
use rmcp::model::{
    Annotated, Implementation, ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParams, RawResource, RawResourceTemplate, ReadResourceRequestParams,
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

/// Run the MCP server on stdio (stdin/stdout).
pub async fn run_stdio() -> anyhow::Result<()> {
    let server = TermLinkTools::new();
    let (stdin, stdout) = rmcp::transport::io::stdio();
    let service = server.serve((stdin, stdout)).await?;
    service.waiting().await?;
    Ok(())
}
