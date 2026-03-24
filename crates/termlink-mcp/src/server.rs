use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{tool_handler, ServerHandler, ServiceExt};

use crate::tools::TermLinkTools;

#[tool_handler]
impl ServerHandler for TermLinkTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(
            Implementation::new("termlink-mcp", env!("CARGO_PKG_VERSION"))
        )
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
