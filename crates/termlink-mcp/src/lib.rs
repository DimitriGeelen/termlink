pub mod tools;
pub mod server;

pub use tools::TermLinkTools;
// T-2002: re-export the CLI-facing help wrapper so `termlink help` can call
// `termlink_mcp::build_cli_help_json(...)` directly without piercing the
// `tools::` module path.
pub use tools::build_cli_help_json;
// T-2004: re-export the tool-name set so `termlink help <target>` can
// disambiguate exact-tool-name (tool_detail) vs substring (name_filter)
// at the dispatch layer without re-walking the registry every call.
pub use tools::registry_tool_names;
// T-2005: re-export the category-name set so `termlink help <target>` can
// route exact-category-name to the `category` filter (priority 2, sandwiched
// between tool_detail and name_filter substring search).
pub use tools::registry_category_names;

/// Returns the number of MCP tools registered by TermLinkTools.
pub fn tool_count() -> usize {
    TermLinkTools::new().tool_router.list_all().len()
}
