//! T-3757 step 1: the MCP founding-verb tools declare `from`.
//!
//! The MCP server runs in-process against an in-process session; the session's
//! use log (T-3702) is read back and every `termlink_exec` / `termlink_inject`
//! row must name the caller the MCP process declared.

use rmcp::model::CallToolRequestParams;
use rmcp::{RoleClient, ServiceExt};
use serde_json::json;
use termlink_mcp::TermLinkTools;
use termlink_test_utils::{start_session, TestDir};

type McpClient = rmcp::service::RunningService<RoleClient, ()>;

async fn mcp_client() -> McpClient {
    let (server_transport, client_transport) = tokio::io::duplex(65536);
    tokio::spawn(async move {
        let svc = TermLinkTools::new().serve(server_transport).await.unwrap();
        svc.waiting().await.unwrap();
    });
    ().serve(client_transport).await.unwrap()
}

async fn call(client: &McpClient, name: &'static str, args: serde_json::Value) -> String {
    let p = CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone());
    let r = client.call_tool(p).await.unwrap();
    r.content.first().and_then(|c| c.raw.as_text()).map(|t| t.text.clone()).unwrap_or_default()
}

fn rows(dir: &TestDir) -> Vec<serde_json::Value> {
    let d = dir.sessions_dir().join("use-log");
    let mut out = Vec::new();
    for e in std::fs::read_dir(&d).unwrap_or_else(|e| panic!("no use-log {}: {e}", d.display())).flatten() {
        for l in std::fs::read_to_string(e.path()).unwrap().lines() {
            out.push(serde_json::from_str(l).unwrap());
        }
    }
    out
}

// One test only: it sets process env (TERMLINK_RUNTIME_DIR, TERMLINK_FROM).
#[tokio::test]
async fn t3757_mcp_exec_and_inject_declare_from() {
    let dir = TestDir::new("t3757-mcp");
    unsafe {
        std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path);
        std::env::remove_var("TERMLINK_OWNER_ONLY");
        std::env::remove_var("TERMLINK_USE_LOG");
        std::env::set_var("TERMLINK_FROM", "mcp-agent");
    }
    let (h, _reg) = start_session(&dir.sessions_dir(), "t", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let client = mcp_client().await;

    let r = call(&client, "termlink_exec", json!({"target": "t", "command": "echo x"})).await;
    assert!(r.contains("\"ok\": true") || r.contains("\"ok\":true"), "exec: {r}");
    // The session has no PTY, so the inject itself fails; its row is written
    // BEFORE the verb acts (T-3702), which is what this test reads.
    let _ = call(&client, "termlink_inject", json!({"target": "t", "text": "ls"})).await;

    let got: Vec<(String, String)> = rows(&dir)
        .into_iter()
        .map(|r| (r["verb"].as_str().unwrap_or("").into(), r["caller"]["from"].as_str().unwrap_or("<none>").into()))
        .collect();
    assert_eq!(
        got,
        vec![("command.execute".into(), "mcp-agent".into()), ("command.inject".into(), "mcp-agent".into())]
    );
    client.cancel().await.unwrap();
    h.abort();
}
