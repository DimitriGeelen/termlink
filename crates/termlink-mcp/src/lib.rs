pub mod tools;
pub mod server;

pub use tools::TermLinkTools;
// T-2002: re-export the CLI-facing help wrapper so `termlink help` can call
// `termlink_mcp::build_cli_help_json(...)` directly without piercing the
// `tools::` module path.
pub use tools::build_cli_help_json;

/// Returns the number of MCP tools registered by TermLinkTools.
pub fn tool_count() -> usize {
    TermLinkTools::new().tool_router.list_all().len()
}
