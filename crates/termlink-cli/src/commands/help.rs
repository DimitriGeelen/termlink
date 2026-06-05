// T-2002 (cycle 13 #1): `termlink help` — CLI parity with MCP `termlink_help`.
//
// Wraps `termlink_mcp::build_cli_help_json` and renders either the raw JSON
// envelope (`--json`) or a human-readable categorized listing. Same axis
// surface as the MCP HelpParams struct — adding an axis here MUST also
// grow `build_cli_help_json` (and vice versa), enforced by the parity test
// in `termlink-mcp::tools::tests::build_cli_help_json_matches_mcp_shape`.

use anyhow::Result;
use serde_json::Value;

/// Grouped invocation arguments — keeps the dispatch arm in `main.rs` readable
/// and shadows the clap variant structure so renaming the variant doesn't
/// ripple through the function signature.
pub(crate) struct HelpInvocation {
    pub target: Option<String>,
    pub json: bool,
    pub category: Option<String>,
    pub name_filter: Option<String>,
    pub list_categories: bool,
    pub tool_detail: Option<String>,
    pub summary: bool,
    pub essentials: bool,
    pub max_parameters: Option<usize>,
    pub min_parameters: Option<usize>,
    pub exclude_deprecated: bool,
    pub deprecated_only: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub fields: Vec<String>,
    pub categories: Vec<String>,
    pub exclude_categories: Vec<String>,
}

/// Outcome of routing the positional `<target>` arg (T-2004 + T-2005).
/// Priority order (top to bottom — first match wins):
/// - `Drilled` → exact tool-name match → tool_detail (T-2004)
/// - `Scoped` → exact category-name match → category filter (T-2005)
/// - `Filtered` → anything else → name_filter substring search (T-2004 fallback)
/// - `Inactive` → no positional supplied; explicit flags pass through unchanged
#[derive(Debug, PartialEq, Eq)]
enum PositionalRoute {
    Drilled(String),
    Scoped(String),
    Filtered(String),
    Inactive,
}

/// Decide what to do with the optional positional `<target>` arg.
/// Returns `Err` with a user-facing message if the positional conflicts
/// with explicit `--tool-detail`, `--name-filter`, or `--category` flags.
fn resolve_positional(
    target: Option<String>,
    explicit_tool_detail: bool,
    explicit_name_filter: bool,
    explicit_category: bool,
) -> Result<PositionalRoute, String> {
    let Some(t) = target else {
        return Ok(PositionalRoute::Inactive);
    };
    if explicit_tool_detail {
        return Err(format!(
            "positional <target>='{t}' conflicts with --tool-detail. Pick one — drop the positional or drop the flag.",
        ));
    }
    if explicit_name_filter {
        return Err(format!(
            "positional <target>='{t}' conflicts with --name-filter. Pick one — drop the positional or drop the flag.",
        ));
    }
    if explicit_category {
        return Err(format!(
            "positional <target>='{t}' conflicts with --category. Pick one — drop the positional or drop the flag.",
        ));
    }
    // Priority 1: exact tool name → drill in.
    if termlink_mcp::registry_tool_names().contains(t.as_str()) {
        return Ok(PositionalRoute::Drilled(t));
    }
    // Priority 2: exact category name → scope to that category.
    if termlink_mcp::registry_category_names().contains(t.as_str()) {
        return Ok(PositionalRoute::Scoped(t));
    }
    // Priority 3: substring search fallback.
    Ok(PositionalRoute::Filtered(t))
}

pub(crate) fn run(inv: HelpInvocation) -> Result<()> {
    // Clap's Vec<String> with value_delimiter=',' defaults to empty when the
    // flag isn't passed; the MCP wrapper distinguishes "unset" (None) from
    // "explicitly empty" ([]) — collapse both here since the operator surface
    // doesn't expose explicit-empty (you'd just omit the flag).
    let fields = empty_to_none(inv.fields);
    let categories = empty_to_none(inv.categories);
    let exclude_categories = empty_to_none(inv.exclude_categories);

    // T-2004 + T-2005: 3-tier positional <target> routing — exact tool →
    // tool_detail (drill-in), exact category → category (namespace scope),
    // else → name_filter (substring search). Conflicts with explicit
    // --tool-detail / --name-filter / --category bail with a usage hint.
    let (category, tool_detail, name_filter) = match resolve_positional(
        inv.target,
        inv.tool_detail.is_some(),
        inv.name_filter.is_some(),
        inv.category.is_some(),
    ) {
        Ok(PositionalRoute::Drilled(name)) => (inv.category, Some(name), inv.name_filter),
        Ok(PositionalRoute::Scoped(cat)) => (Some(cat), inv.tool_detail, inv.name_filter),
        Ok(PositionalRoute::Filtered(needle)) => (inv.category, inv.tool_detail, Some(needle)),
        Ok(PositionalRoute::Inactive) => (inv.category, inv.tool_detail, inv.name_filter),
        Err(msg) => {
            // T-2008 (cycle 13 #5): honor --json on positional/flag conflict so
            // scripts wrapping `termlink help --json ...` always parse a stable
            // envelope on stdout. Plain-text path stays for non-JSON callers.
            // Same shape as T-1914 / execution.rs / identity.rs: {ok:false,
            // error:<msg>}. Exit 2 (usage error, distinct from json_error_exit's
            // operational exit 1) preserves the pre-T-2008 contract.
            if inv.json {
                println!("{}", positional_conflict_json_envelope(&msg));
            } else {
                eprintln!("error: {msg}");
            }
            std::process::exit(2);
        }
    };

    let json_str = termlink_mcp::build_cli_help_json(
        category,
        name_filter,
        inv.list_categories,
        tool_detail,
        inv.summary,
        inv.essentials,
        inv.max_parameters,
        inv.min_parameters,
        inv.exclude_deprecated,
        inv.deprecated_only,
        inv.limit,
        inv.offset,
        inv.sort_by,
        fields,
        categories,
        exclude_categories,
    );

    if inv.json {
        println!("{json_str}");
        return Ok(());
    }

    // Parse-and-render path. If the envelope is malformed (shouldn't happen
    // — build_cli_help_json always produces valid JSON), fall back to raw
    // print so the operator sees something useful.
    let value: Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => {
            println!("{json_str}");
            return Ok(());
        }
    };

    render_human(&value);
    Ok(())
}

fn empty_to_none(v: Vec<String>) -> Option<Vec<String>> {
    if v.is_empty() { None } else { Some(v) }
}

/// T-2008 (cycle 13 #5): JSON-mode envelope for positional/flag conflict.
/// Shape: `{"ok": false, "error": "<msg>"}` — matches T-1914 and the broader
/// CLI --json error-path convention (execution.rs, identity.rs). Returned as
/// a `String` so callers can `println!` it directly and tests can parse it
/// without going through the exit() path.
fn positional_conflict_json_envelope(msg: &str) -> String {
    serde_json::json!({"ok": false, "error": msg}).to_string()
}

fn render_human(value: &Value) {
    // Error envelope: `{ok: false, error: "..."}` — surface and bail.
    if let Some(false) = value.get("ok").and_then(|v| v.as_bool()) {
        if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
            eprintln!("error: {err}");
            if let Some(hint) = value.get("hint").and_then(|v| v.as_str()) {
                eprintln!("hint: {hint}");
            }
            if let Some(suggestions) = value.get("did_you_mean").and_then(|v| v.as_array()) {
                if !suggestions.is_empty() {
                    eprintln!("did you mean:");
                    for s in suggestions {
                        if let Some(name) = s.as_str() {
                            eprintln!("  {name}");
                        }
                    }
                }
            }
        }
        return;
    }

    // tool_detail envelope: `{tool, name, category, full_description, parameters, ...}`.
    if value.get("tool").is_some() && value.get("full_description").is_some() {
        render_tool_detail(value);
        return;
    }

    // summary envelope: `{total_tools, total_categories, ...}` — distinct from
    // default-mode envelope by absence of category arrays + presence of stats keys.
    if value.get("total_categories").is_some() && value.get("total_tools").is_some()
        && !value.as_object().map(|o| o.keys().any(|k| {
            // any key that maps to an array of {name,description} rows is a category — default mode
            value.get(k).and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|first| first.get("description"))
                .is_some()
        })).unwrap_or(false)
    {
        render_summary(value);
        return;
    }

    // essentials envelope: `{essentials: [...], total}`.
    if let Some(rows) = value.get("essentials").and_then(|v| v.as_array()) {
        render_essentials(rows, value.get("total").and_then(|v| v.as_u64()));
        return;
    }

    // list_categories envelope: `{categories: [{name, tool_count, ...}], total_categories, total_tools}`.
    if let Some(cats) = value.get("categories").and_then(|v| v.as_array()) {
        render_list_categories(
            cats,
            value.get("total_categories").and_then(|v| v.as_u64()),
            value.get("total_tools").and_then(|v| v.as_u64()),
        );
        return;
    }

    // matches[] envelope (name_filter / bulk-flat / paginated): `{matches: [...], total_matched, ...}`.
    if let Some(rows) = value.get("matches").and_then(|v| v.as_array()) {
        render_matches(rows, value);
        return;
    }

    // Default mode: per-category dump `{cat1: [...], cat2: [...], total_tools}`.
    render_default(value);
}

fn render_default(value: &Value) {
    let Some(obj) = value.as_object() else { return };
    let mut categories: Vec<(&String, &Value)> = obj.iter()
        .filter(|(_, v)| v.is_array())
        .collect();
    categories.sort_by(|a, b| a.0.cmp(b.0));

    let total = value.get("total_tools").and_then(|v| v.as_u64()).unwrap_or(0);
    println!("TermLink MCP tool registry — {total} tools across {} categories", categories.len());
    println!();

    for (cat_name, rows) in &categories {
        let Some(arr) = rows.as_array() else { continue };
        println!("[{cat_name}] ({} tools)", arr.len());
        for row in arr {
            let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let desc = row.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let dep = row.get("deprecated").and_then(|v| v.as_bool()).unwrap_or(false);
            let arity = row.get("parameter_count").and_then(|v| v.as_u64());
            let req_arity = row.get("parameter_required_count").and_then(|v| v.as_u64());
            let dep_tag = if dep { " [DEPRECATED]" } else { "" };
            let arity_tag = match (arity, req_arity) {
                (Some(a), Some(r)) => format!(" ({a}/{r})"),
                (Some(a), None) => format!(" ({a})"),
                _ => String::new(),
            };
            println!("  {name}{arity_tag}{dep_tag}");
            if !desc.is_empty() {
                println!("    {desc}");
            }
        }
        println!();
    }

    println!("Tip: termlink help --essentials             # 27-tool starter set");
    println!("     termlink help --list-categories         # category index only");
    println!("     termlink help --tool-detail <name>      # full details for one tool");
    println!("     termlink help --name-filter <substring> # search across names + descriptions");
    println!("     termlink help --json ...                # raw envelope for jq/scripting");
}

fn render_matches(rows: &[Value], envelope: &Value) {
    let total = envelope.get("total_matched").and_then(|v| v.as_u64()).unwrap_or(rows.len() as u64);
    let limit_applied = envelope.get("limit_applied").and_then(|v| v.as_bool()).unwrap_or(false);
    let next_offset = envelope.get("next_offset").and_then(|v| v.as_u64());
    let sort_by_applied = envelope.get("sort_by_applied").and_then(|v| v.as_str());

    print!("{} match(es)", rows.len());
    if total as usize != rows.len() {
        print!(" of {total}");
    }
    if let Some(axis) = sort_by_applied {
        print!(", sorted by {axis}");
    }
    println!();
    println!();

    for row in rows {
        let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let cat = row.get("category").and_then(|v| v.as_str()).unwrap_or("");
        let desc = row.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let dep = row.get("deprecated").and_then(|v| v.as_bool()).unwrap_or(false);
        let arity = row.get("parameter_count").and_then(|v| v.as_u64());
        let req_arity = row.get("parameter_required_count").and_then(|v| v.as_u64());
        let dep_tag = if dep { " [DEPRECATED]" } else { "" };
        let cat_tag = if cat.is_empty() { String::new() } else { format!(" ({cat})") };
        let arity_tag = match (arity, req_arity) {
            (Some(a), Some(r)) => format!(" arity={a}/{r}"),
            (Some(a), None) => format!(" arity={a}"),
            _ => String::new(),
        };
        println!("  {name}{cat_tag}{arity_tag}{dep_tag}");
        if !desc.is_empty() {
            println!("    {desc}");
        }
    }

    println!();
    if limit_applied {
        match next_offset {
            Some(off) => println!("More results: termlink help ... --offset {off}"),
            None => println!("(all results shown within current limit)"),
        }
    }
    if let Some(hint) = envelope.get("hint").and_then(|v| v.as_str()) {
        println!("hint: {hint}");
    }
    // T-2006: name_filter 0-match did-you-mean. Surface tool/category
    // suggestions inline so a typo'd `termlink help chanel` recovers in one
    // round-trip. Comma-joined single line matches the MCP envelope's
    // `did_you_mean` array semantics; mirrors T-1954's tool_detail path
    // but uses inline format (one line) per name_filter's flatter shape.
    if let Some(suggestions) = envelope.get("did_you_mean").and_then(|v| v.as_array()) {
        let names: Vec<&str> = suggestions
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        if !names.is_empty() {
            println!("Did you mean: {}", names.join(", "));
        }
    }
    surface_validation_echoes(envelope);
}

fn render_list_categories(cats: &[Value], total_cats: Option<u64>, total_tools: Option<u64>) {
    println!(
        "{} categories, {} total tools",
        total_cats.unwrap_or(cats.len() as u64),
        total_tools.unwrap_or(0),
    );
    println!();
    for cat in cats {
        let name = cat.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let count = cat.get("tool_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let live = cat.get("live_tool_count").and_then(|v| v.as_u64());
        let dep = cat.get("deprecated_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let desc = cat.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let live_tag = match live {
            Some(l) if dep > 0 => format!(" ({l} live / {dep} deprecated)"),
            _ => format!(" ({count} tools)"),
        };
        println!("  {name}{live_tag}");
        if !desc.is_empty() {
            println!("    {desc}");
        }
    }
    println!();
    println!("Drill in: termlink help --category <name>");
}

fn render_tool_detail(value: &Value) {
    let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let cat = value.get("category").and_then(|v| v.as_str()).unwrap_or("");
    let dep = value.get("deprecated").and_then(|v| v.as_bool()).unwrap_or(false);
    let short = value.get("short_description").and_then(|v| v.as_str()).unwrap_or("");
    let full = value.get("full_description").and_then(|v| v.as_str()).unwrap_or("");

    print!("{name}");
    if !cat.is_empty() {
        print!("  [{cat}]");
    }
    if dep {
        print!("  [DEPRECATED]");
        if let Some(rep) = value.get("replacement_hint").and_then(|v| v.as_str()) {
            print!(" — use {rep} instead");
        }
    }
    println!();

    if !short.is_empty() {
        println!("\n{short}");
    }
    if !full.is_empty() && full != short {
        println!("\n{full}");
    }

    if let Some(params) = value.get("parameters").and_then(|v| v.as_array()) {
        if !params.is_empty() {
            println!("\nParameters:");
            for p in params {
                let pname = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let ptype = p.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let optional = p.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
                let pdoc = p.get("doc").and_then(|v| v.as_str()).unwrap_or("");
                let opt_tag = if optional { "?" } else { "" };
                println!("  {pname}{opt_tag}: {ptype}");
                if !pdoc.is_empty() {
                    println!("    {pdoc}");
                }
            }
        }
    }

    if let Some(rel) = value.get("related_tools").and_then(|v| v.as_array()) {
        if !rel.is_empty() {
            println!("\nRelated tools:");
            for r in rel {
                if let Some(n) = r.as_str() {
                    println!("  {n}");
                }
            }
        }
    }
}

fn render_summary(value: &Value) {
    let total = value.get("total_tools").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_cats = value.get("total_categories").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_dep = value.get("total_deprecated").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_live = value.get("total_live_tools").and_then(|v| v.as_u64());
    let total_params = value.get("total_parameters").and_then(|v| v.as_u64());
    let zero_arity = value.get("zero_arity_tools").and_then(|v| v.as_u64());

    println!("Registry summary");
    println!("  {total} tools across {total_cats} categories ({total_dep} deprecated)");
    if let Some(l) = total_live {
        println!("  {l} live tools");
    }
    if let Some(p) = total_params {
        println!("  {p} total parameters");
    }
    if let Some(z) = zero_arity {
        println!("  {z} zero-arg tools");
    }

    if let Some(large) = value.get("largest_categories").and_then(|v| v.as_array()) {
        if !large.is_empty() {
            println!("\nLargest categories:");
            for c in large {
                let n = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let t = c.get("tool_count").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("  {n}  ({t})");
            }
        }
    }

    if let Some(high) = value.get("highest_arity_tools").and_then(|v| v.as_array()) {
        if !high.is_empty() {
            println!("\nHighest-arity tools:");
            for t in high {
                let n = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let a = t.get("parameter_count").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("  {n}  ({a} params)");
            }
        }
    }
}

fn render_essentials(rows: &[Value], total: Option<u64>) {
    println!("Essential tools ({}): one canonical entry-point per category",
             total.unwrap_or(rows.len() as u64));
    println!();
    for row in rows {
        let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let cat = row.get("category").and_then(|v| v.as_str()).unwrap_or("");
        let cat_desc = row.get("category_description").and_then(|v| v.as_str()).unwrap_or("");
        let desc = row.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let arity = row.get("parameter_count").and_then(|v| v.as_u64());
        let arity_tag = arity.map(|a| format!(" ({a} params)")).unwrap_or_default();
        println!("  {name}  [{cat}]{arity_tag}");
        if !cat_desc.is_empty() {
            println!("    category: {cat_desc}");
        }
        if !desc.is_empty() {
            println!("    {desc}");
        }
    }
}

fn surface_validation_echoes(envelope: &Value) {
    // Envelope-level `*_unknown` arrays mean the operator passed values the
    // server didn't recognize — silently dropped from the filter but echoed
    // here so input mistakes don't masquerade as quiet misreads.
    for axis in &["sort_by", "fields", "categories", "exclude_categories"] {
        let unknown_key = format!("{axis}_unknown");
        if let Some(unknown) = envelope.get(&unknown_key) {
            match unknown {
                Value::String(s) => eprintln!("note: --{axis}={s} not recognized (ignored)"),
                Value::Array(arr) if !arr.is_empty() => {
                    let names: Vec<String> = arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    eprintln!("note: unknown --{axis} value(s) ignored: {}", names.join(","));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invocation_default() -> HelpInvocation {
        HelpInvocation {
            target: None,
            json: true,
            category: None,
            name_filter: None,
            list_categories: false,
            tool_detail: None,
            summary: false,
            essentials: false,
            max_parameters: None,
            min_parameters: None,
            exclude_deprecated: false,
            deprecated_only: false,
            limit: None,
            offset: None,
            sort_by: None,
            fields: Vec::new(),
            categories: Vec::new(),
            exclude_categories: Vec::new(),
        }
    }

    /// T-2004: positional `<target>` that exactly matches a registered tool
    /// name routes to `tool_detail` (drill-in) — same behavior as the explicit
    /// `--tool-detail <name>` flag. Priority 1: tool match wins.
    #[test]
    fn positional_exact_tool_routes_to_tool_detail() {
        let route = resolve_positional(
            Some("termlink_channel_post".to_string()),
            false, false, false,
        );
        assert_eq!(route, Ok(PositionalRoute::Drilled("termlink_channel_post".to_string())));
    }

    /// T-2005: positional `<target>` that exactly matches a registered category
    /// name routes to `category` (namespace scope). Priority 2 — wins over
    /// substring fallback but loses to exact tool match.
    #[test]
    fn positional_exact_category_routes_to_scoped() {
        for cat in &["channel", "agent_chat", "fleet", "session"] {
            let route = resolve_positional(Some(cat.to_string()), false, false, false);
            assert_eq!(
                route,
                Ok(PositionalRoute::Scoped(cat.to_string())),
                "category '{cat}' should route to Scoped",
            );
        }
    }

    /// T-2004 + T-2005: positional that's neither a tool nor a category name
    /// falls through to name_filter substring search (priority 3).
    #[test]
    fn positional_unknown_routes_to_name_filter() {
        // Strings that resemble tool names but aren't registered.
        let route = resolve_positional(Some("zzzzz".to_string()), false, false, false);
        assert_eq!(route, Ok(PositionalRoute::Filtered("zzzzz".to_string())));
        // Multi-word substring queries (would never be tool or category names).
        let route = resolve_positional(Some("channel post".to_string()), false, false, false);
        assert_eq!(route, Ok(PositionalRoute::Filtered("channel post".to_string())));
    }

    /// T-2004: explicit `--tool-detail` AND a positional `<target>` conflict —
    /// caller intent is ambiguous, refuse with a hint.
    #[test]
    fn positional_with_explicit_tool_detail_errors() {
        let err = resolve_positional(Some("channel".to_string()), true, false, false)
            .unwrap_err();
        assert!(err.contains("--tool-detail"), "err mentions conflicting flag");
        assert!(err.contains("'channel'"), "err echoes the positional");
    }

    /// T-2004: explicit `--name-filter` AND a positional `<target>` conflict —
    /// same as the tool_detail case.
    #[test]
    fn positional_with_explicit_name_filter_errors() {
        let err = resolve_positional(Some("channel".to_string()), false, true, false)
            .unwrap_err();
        assert!(err.contains("--name-filter"), "err mentions conflicting flag");
    }

    /// T-2005: explicit `--category` AND a positional `<target>` conflict —
    /// since the positional may itself be a category, ambiguity is total.
    #[test]
    fn positional_with_explicit_category_errors() {
        let err = resolve_positional(Some("channel".to_string()), false, false, true)
            .unwrap_err();
        assert!(err.contains("--category"), "err mentions conflicting flag");
        assert!(err.contains("'channel'"), "err echoes the positional");
    }

    /// T-2004: no positional → inactive route → all upstream flags pass through.
    #[test]
    fn no_positional_is_inactive() {
        let route = resolve_positional(None, false, false, false);
        assert_eq!(route, Ok(PositionalRoute::Inactive));
        // Explicit flags without positional are NOT a conflict.
        let route = resolve_positional(None, true, true, true);
        assert_eq!(route, Ok(PositionalRoute::Inactive));
    }

    #[test]
    fn empty_vec_collapses_to_none() {
        assert_eq!(empty_to_none(Vec::<String>::new()), None);
        assert_eq!(empty_to_none(vec!["a".to_string()]), Some(vec!["a".to_string()]));
    }

    #[test]
    fn invocation_default_is_inert() {
        // A bare `termlink help --json` returns the default envelope: a JSON
        // object with `total_tools` and per-category arrays.
        let inv = invocation_default();
        let fields = empty_to_none(inv.fields);
        let categories = empty_to_none(inv.categories);
        let exclude_categories = empty_to_none(inv.exclude_categories);
        let out = termlink_mcp::build_cli_help_json(
            inv.category, inv.name_filter, inv.list_categories, inv.tool_detail,
            inv.summary, inv.essentials, inv.max_parameters, inv.min_parameters,
            inv.exclude_deprecated, inv.deprecated_only, inv.limit, inv.offset,
            inv.sort_by, fields, categories, exclude_categories,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(v.get("total_tools").and_then(|x| x.as_u64()).unwrap_or(0) > 0);
    }

    #[test]
    fn name_filter_returns_matches_envelope() {
        let mut inv = invocation_default();
        inv.name_filter = Some("channel".to_string());
        inv.limit = Some(5);
        let fields = empty_to_none(inv.fields);
        let categories = empty_to_none(inv.categories);
        let exclude_categories = empty_to_none(inv.exclude_categories);
        let out = termlink_mcp::build_cli_help_json(
            inv.category, inv.name_filter, inv.list_categories, inv.tool_detail,
            inv.summary, inv.essentials, inv.max_parameters, inv.min_parameters,
            inv.exclude_deprecated, inv.deprecated_only, inv.limit, inv.offset,
            inv.sort_by, fields, categories, exclude_categories,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let matches = v.get("matches").and_then(|x| x.as_array()).expect("matches array");
        assert!(matches.len() <= 5, "limit honored: got {} rows", matches.len());
        assert!(v.get("total_matched").is_some(), "total_matched present");
    }

    #[test]
    fn sort_by_required_arity_limit_combines() {
        // The canonical PL-202 cold-start call surface: sort by call cost ASC,
        // cap to 10, project to two fields. This is the call agents will run
        // first via the MCP tool — verify the CLI path produces the same shape.
        let mut inv = invocation_default();
        inv.limit = Some(10);
        inv.sort_by = Some("required_arity".to_string());
        inv.exclude_deprecated = true;
        inv.fields = vec!["name".to_string(), "parameter_required_count".to_string()];
        let fields = empty_to_none(inv.fields);
        let categories = empty_to_none(inv.categories);
        let exclude_categories = empty_to_none(inv.exclude_categories);
        let out = termlink_mcp::build_cli_help_json(
            inv.category, inv.name_filter, inv.list_categories, inv.tool_detail,
            inv.summary, inv.essentials, inv.max_parameters, inv.min_parameters,
            inv.exclude_deprecated, inv.deprecated_only, inv.limit, inv.offset,
            inv.sort_by, fields, categories, exclude_categories,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let matches = v.get("matches").and_then(|x| x.as_array()).expect("matches array");
        assert!(matches.len() <= 10);
        // Field projection — every row has exactly the two keys.
        for row in matches {
            let obj = row.as_object().expect("matches row is object");
            assert!(obj.contains_key("name"), "name field retained");
            assert!(obj.contains_key("parameter_required_count"), "required_arity retained");
            assert!(!obj.contains_key("description"), "description projected out");
        }
        // Envelope advertises the applied axes.
        assert_eq!(
            v.get("sort_by_applied").and_then(|x| x.as_str()),
            Some("required_arity"),
        );
    }

    /// T-2006: CLI wrapper carries `did_you_mean` on a name_filter 0-match
    /// real-needle envelope. `render_matches` reads exactly this field and
    /// surfaces the "Did you mean: ..." inline line; testing the envelope
    /// here is the load-bearing contract — proves the CLI surface inherits
    /// the MCP block via the shape-parity wrapper. Mirrors the MCP-side
    /// `name_filter_zero_match_emits_did_you_mean_for_typo` test.
    #[test]
    fn name_filter_zero_match_envelope_carries_did_you_mean() {
        let mut inv = invocation_default();
        inv.name_filter = Some("chanel".to_string());
        let fields = empty_to_none(inv.fields);
        let categories = empty_to_none(inv.categories);
        let exclude_categories = empty_to_none(inv.exclude_categories);
        let out = termlink_mcp::build_cli_help_json(
            inv.category, inv.name_filter, inv.list_categories, inv.tool_detail,
            inv.summary, inv.essentials, inv.max_parameters, inv.min_parameters,
            inv.exclude_deprecated, inv.deprecated_only, inv.limit, inv.offset,
            inv.sort_by, fields, categories, exclude_categories,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(
            v.get("total_matches").and_then(|x| x.as_u64()),
            Some(0),
            "expected 0 matches for typo 'chanel'",
        );
        let suggestions = v
            .get("did_you_mean")
            .and_then(|x| x.as_array())
            .expect("did_you_mean must be present on real-needle 0-match");
        assert!(!suggestions.is_empty(), "did_you_mean must be non-empty");
        let has_channel = suggestions.iter().any(|s| s.as_str() == Some("channel"));
        assert!(
            has_channel,
            "expected 'channel' in did_you_mean suggestions, got {:?}",
            suggestions,
        );
    }

    /// T-2008: positional `<target>` conflicting with explicit `--tool-detail`
    /// produces a JSON envelope on stdout (when `--json` is in effect) instead
    /// of a stderr-only plain line. Envelope shape matches T-1914 convention:
    /// `{ok: false, error: <msg>}`. The error string MUST name the conflicting
    /// flag so scripts can route the failure mode programmatically.
    #[test]
    fn positional_conflict_json_envelope_tool_detail() {
        let msg = resolve_positional(Some("bogus".into()), true, false, false)
            .expect_err("tool_detail conflict must Err");
        let env_str = positional_conflict_json_envelope(&msg);
        let v: serde_json::Value =
            serde_json::from_str(&env_str).expect("envelope is valid JSON");
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
        let err = v["error"].as_str().expect("error is a string");
        assert!(err.contains("--tool-detail"), "envelope cites the conflicting flag");
        assert!(err.contains("bogus"), "envelope echoes the offending positional");
    }

    /// T-2008: same as above for `--name-filter`.
    #[test]
    fn positional_conflict_json_envelope_name_filter() {
        let msg = resolve_positional(Some("bogus".into()), false, true, false)
            .expect_err("name_filter conflict must Err");
        let env_str = positional_conflict_json_envelope(&msg);
        let v: serde_json::Value =
            serde_json::from_str(&env_str).expect("envelope is valid JSON");
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
        let err = v["error"].as_str().expect("error is a string");
        assert!(err.contains("--name-filter"), "envelope cites the conflicting flag");
    }

    /// T-2008: same as above for `--category`.
    #[test]
    fn positional_conflict_json_envelope_category() {
        let msg = resolve_positional(Some("bogus".into()), false, false, true)
            .expect_err("category conflict must Err");
        let env_str = positional_conflict_json_envelope(&msg);
        let v: serde_json::Value =
            serde_json::from_str(&env_str).expect("envelope is valid JSON");
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
        let err = v["error"].as_str().expect("error is a string");
        assert!(err.contains("--category"), "envelope cites the conflicting flag");
    }
}
