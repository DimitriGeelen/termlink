//! Integration tests for the TermLink MCP server.
//!
//! Uses rmcp client + real TermLink sessions to verify all MCP tools
//! produce correct structured responses.

use rmcp::model::CallToolRequestParams;
use rmcp::{RoleClient, ServiceExt};
use serde_json::json;
use termlink_test_utils::{start_session, TestDir};
use tokio::sync::Mutex;

use termlink_mcp::TermLinkTools;

/// Serialize tests that set TERMLINK_RUNTIME_DIR.
static ENV_LOCK: Mutex<()> = Mutex::const_new(());

type McpClient = rmcp::service::RunningService<RoleClient, ()>;

/// Helper: create MCP client connected to TermLinkTools server in-process.
async fn mcp_client() -> McpClient {
    let (server_transport, client_transport) = tokio::io::duplex(65536);

    let server = TermLinkTools::new();
    tokio::spawn(async move {
        let svc = server.serve(server_transport).await.unwrap();
        svc.waiting().await.unwrap();
    });

    let client: McpClient = ().serve(client_transport).await.unwrap();
    client
}

/// Helper: call a tool with JSON arguments.
async fn call(client: &McpClient, name: &'static str, args: serde_json::Value) -> String {
    let params = if args.is_object() && !args.as_object().unwrap().is_empty() {
        CallToolRequestParams::new(name).with_arguments(
            args.as_object().unwrap().clone(),
        )
    } else {
        CallToolRequestParams::new(name)
    };

    let result = client.call_tool(params).await.unwrap();
    result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.clone())
        .unwrap_or_default()
}

#[tokio::test]
async fn test_list_tools() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-list-tools");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let tools = client.list_all_tools().await.unwrap();

    let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    for expected in &[
        "termlink_ping", "termlink_list_sessions", "termlink_discover",
        "termlink_exec", "termlink_output", "termlink_inject",
        "termlink_signal", "termlink_emit", "termlink_emit_to",
        "termlink_event_poll", "termlink_kv_set", "termlink_kv_get",
        "termlink_kv_list", "termlink_kv_del", "termlink_broadcast",
        "termlink_wait", "termlink_spawn", "termlink_run", "termlink_status",
    ] {
        assert!(names.iter().any(|n| n == expected), "missing tool: {expected}");
    }
    assert!(tools.len() >= 19, "expected at least 19 tools, got {}", tools.len());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_list_sessions_empty() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-list-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_list_sessions", json!({})).await;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert!(parsed.is_empty(), "expected empty list, got: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_list_sessions_with_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-list-sess");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-test-alpha", vec!["worker".into()]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_list_sessions", json!({})).await;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["display_name"], "mcp-test-alpha");
    assert!(parsed[0]["roles"].as_array().unwrap().contains(&json!("worker")));

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_ping_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-ping");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-ping-target", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_ping", json!({"target": "mcp-ping-target"})).await;

    assert!(!text.contains("Error"), "ping failed: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_ping_nonexistent() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-ping-bad");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_ping", json!({"target": "no-such-session"})).await;

    assert!(text.contains("Error"), "expected error for nonexistent session: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_status_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-status");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-status-tgt", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_status", json!({"target": "mcp-status-tgt"})).await;

    assert!(!text.contains("Error"), "status failed: {text}");
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(parsed.get("state").is_some() || parsed.get("display_name").is_some(),
        "expected session status fields: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_discover_by_role_and_name() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-discover");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h1, _r1) = start_session(&dir.sessions_dir(), "disc-alpha", vec!["coder".into()]).await;
    let (_h2, _r2) = start_session(&dir.sessions_dir(), "disc-beta", vec!["tester".into()]).await;

    let client = mcp_client().await;

    // Discover by role
    let text = call(&client, "termlink_discover", json!({"roles": ["coder"]})).await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 1, "expected 1 match, got: {text}");
    assert_eq!(parsed[0]["display_name"], "disc-alpha");

    // Discover by name substring
    let text = call(&client, "termlink_discover", json!({"name": "beta"})).await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["display_name"], "disc-beta");

    // Discover all (no filters)
    let text = call(&client, "termlink_discover", json!({})).await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 2, "expected 2 sessions: {text}");

    client.cancel().await.unwrap();
    _h1.abort();
    _h2.abort();
}

#[tokio::test]
async fn test_kv_set_get_list_del() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-kv");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-kv-target", vec![]).await;

    let client = mcp_client().await;

    // Set
    let text = call(&client, "termlink_kv_set",
        json!({"target": "mcp-kv-target", "key": "status", "value": "active"})).await;
    assert!(text.contains("Set") || text.contains("Updated"), "kv set failed: {text}");

    // Get
    let text = call(&client, "termlink_kv_get",
        json!({"target": "mcp-kv-target", "key": "status"})).await;
    assert!(text.contains("active"), "expected 'active', got: {text}");

    // List
    let text = call(&client, "termlink_kv_list",
        json!({"target": "mcp-kv-target"})).await;
    assert!(text.contains("status"), "expected 'status' in list: {text}");

    // Delete
    let text = call(&client, "termlink_kv_del",
        json!({"target": "mcp-kv-target", "key": "status"})).await;
    assert!(text.contains("Deleted"), "kv del failed: {text}");

    // Get deleted — not found
    let text = call(&client, "termlink_kv_get",
        json!({"target": "mcp-kv-target", "key": "status"})).await;
    assert!(text.contains("not found"), "expected not found: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_emit_and_event_poll() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-events");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-events-tgt", vec![]).await;

    let client = mcp_client().await;

    // Emit
    let text = call(&client, "termlink_emit",
        json!({"target": "mcp-events-tgt", "topic": "test.hello", "payload": {"msg": "world"}})).await;
    assert!(text.contains("Emitted"), "emit failed: {text}");

    // Poll
    let text = call(&client, "termlink_event_poll",
        json!({"target": "mcp-events-tgt", "topic": "test.hello"})).await;
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert!(!events.is_empty(), "expected events, got: {text}");
    assert_eq!(events[0]["topic"], "test.hello");
    assert_eq!(events[0]["payload"]["msg"], "world");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_wait_receives_event() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-wait");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, reg) = start_session(&dir.sessions_dir(), "mcp-wait-tgt", vec![]).await;

    // Emit event via direct RPC first
    let socket = reg.socket_path().to_path_buf();
    termlink_session::client::rpc_call(
        &socket,
        "event.emit",
        json!({"topic": "test.done", "payload": {"result": "ok"}}),
    )
    .await
    .unwrap();

    let client = mcp_client().await;
    let text = call(&client, "termlink_wait",
        json!({"target": "mcp-wait-tgt", "topic": "test.done", "timeout": 5})).await;

    assert!(!text.contains("Timeout"), "wait timed out: {text}");
    assert!(text.contains("ok") || text.contains("Event received"), "unexpected: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_wait_timeout() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-wait-to");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-wait-to-tgt", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_wait",
        json!({"target": "mcp-wait-to-tgt", "topic": "never.happens", "timeout": 1})).await;

    assert!(text.contains("Timeout"), "expected timeout: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_run_command() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-run");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_run", json!({"command": "echo hello-from-mcp"})).await;

    assert!(text.contains("hello-from-mcp"), "expected output, got: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_run_command_exit_code() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-run-exit");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_run", json!({"command": "false"})).await;

    assert!(text.contains("exit_code"), "expected exit code info: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tool_schemas_have_descriptions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-schemas");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let tools = client.list_all_tools().await.unwrap();

    for tool in &tools {
        assert!(
            tool.description.as_ref().is_some_and(|d| !d.is_empty()),
            "tool '{}' has empty description",
            tool.name
        );
    }

    client.cancel().await.unwrap();
}
