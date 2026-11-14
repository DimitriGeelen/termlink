//! T-3780: every RPC method name the MCP tools send must be one some handler serves.
//! Found 2026-11-13: termlink_batch_exec and termlink_remote_exec sent `command.exec`, which
//! nothing serves (sessions serve `command.execute`), so both tools could only fail with
//! method-not-found. This is a source scan, so it also catches a future typo in any tool.
//!
//! "Served" = a `pub const` method name in termlink-protocol's control.rs, or a quoted match
//! arm (`"x.y" =>`) in the session handler or the hub router/server.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// Every string literal that is the first or second argument after `marker` (calls may
/// wrap across lines) and looks like a method name (`a.b`, lowercase, dots/underscores).
/// Calls that pass a `control::method` constant are skipped: a constant cannot be a typo.
fn methods_after(src: &str, marker: &str) -> Vec<(usize, String)> {
    let mut out = vec![];
    let mut from = 0;
    while let Some(i) = src[from..].find(marker) {
        let at = from + i + marker.len();
        from = at;
        let seg: String = src[at..].chars().take(300).collect();
        let Some(q) = seg.find('"') else { continue };
        let head = &seg[..q];
        if head.matches(',').count() > 1 || head.contains(')') || head.contains("method::") {
            continue;
        }
        let tail = &seg[q + 1..];
        let Some(e) = tail.find('"') else { continue };
        let lit = &tail[..e];
        let looks = lit.contains('.')
            && !lit.starts_with('.')
            && !lit.ends_with('.')
            && lit.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_');
        if looks {
            let line = src[..at].matches('\n').count() + 1;
            out.push((line, lit.to_string()));
        }
    }
    out
}

fn served() -> BTreeSet<String> {
    let mut s = BTreeSet::new();
    for line in read("termlink-protocol/src/control.rs").lines() {
        let t = line.trim();
        if t.starts_with("pub const ")
            && t.contains(": &str = \"")
            && let Some(v) = t.split('"').nth(1)
        {
            s.insert(v.to_string());
        }
    }
    for f in ["termlink-session/src/handler.rs", "termlink-hub/src/router.rs", "termlink-hub/src/server.rs"] {
        for line in read(f).lines() {
            let t = line.trim_start().trim_start_matches('|').trim_start();
            if let Some(rest) = t.strip_prefix('"')
                && let Some(e) = rest.find('"')
                && rest[e + 1..].trim_start().starts_with(['=', '|', 'i'])
            {
                s.insert(rest[..e].to_string());
            }
        }
    }
    s
}

#[test]
fn every_mcp_rpc_method_is_served_by_some_handler() {
    let tools = read("termlink-mcp/src/tools.rs");
    let mut used: Vec<(usize, String)> = vec![];
    for m in ["rpc_call(", "rpc_call_addr(", "rpc_call_addr_with_timeout(", ".call("] {
        used.extend(methods_after(&tools, m));
    }
    assert!(used.len() >= 15, "the scan found almost nothing ({}), so it is not looking where the calls are", used.len());
    let served = served();
    assert!(served.contains("command.execute") && served.contains("session.discover"), "served set incomplete");
    let missing: Vec<String> = used
        .iter()
        .filter(|(_, m)| !served.contains(m))
        .map(|(ln, m)| format!("tools.rs:{ln} sends '{m}'"))
        .collect();
    assert!(missing.is_empty(), "MCP tools send methods no handler serves:\n{}", missing.join("\n"));
}

/// The batch-exec tool against a real session (in-process, temp runtime dir): the command
/// runs and its output comes back. Before T-3780 the tool sent `command.exec` and the
/// session answered method-not-found.
#[tokio::test]
async fn batch_exec_runs_a_command_on_a_real_session() {
    use rmcp::model::CallToolRequestParams;
    use rmcp::{RoleClient, ServiceExt};
    use termlink_test_utils::{start_session, TestDir};

    let dir = TestDir::new("t3780-batch");
    // SAFETY: this test binary runs this one test; nothing else reads the env concurrently.
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (h, _reg) = start_session(&dir.sessions_dir(), "t3780-target", vec![]).await;

    let (server_transport, client_transport) = tokio::io::duplex(65536);
    tokio::spawn(async move {
        let svc = termlink_mcp::TermLinkTools::new().serve(server_transport).await.unwrap();
        svc.waiting().await.unwrap();
    });
    let client: rmcp::service::RunningService<RoleClient, ()> = ().serve(client_transport).await.unwrap();
    let args = serde_json::json!({"command": "echo t3780-batch-ok", "name": "t3780-target", "timeout": 10});
    let res = client
        .call_tool(CallToolRequestParams::new("termlink_batch_exec").with_arguments(args.as_object().unwrap().clone()))
        .await
        .unwrap();
    let text = res.content.first().and_then(|c| c.raw.as_text()).map(|t| t.text.clone()).unwrap_or_default();
    assert!(text.contains("t3780-batch-ok"), "batch exec output: {text}");
    assert!(!text.contains("not found") || text.contains("t3780-batch-ok"), "{text}");
    client.cancel().await.unwrap();
    h.abort();
}
