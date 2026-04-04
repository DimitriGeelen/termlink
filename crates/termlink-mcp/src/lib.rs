pub mod tools;
pub mod server;

pub use tools::TermLinkTools;

/// Returns the number of MCP tools registered by TermLinkTools.
pub fn tool_count() -> usize {
    TermLinkTools::new().tool_router.list_all().len()
}
