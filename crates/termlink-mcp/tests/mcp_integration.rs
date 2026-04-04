//! Integration tests for the TermLink MCP server.
//!
//! Uses rmcp client + real TermLink sessions to verify all MCP tools
//! produce correct structured responses.

use rmcp::model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams, ResourceContents};
use rmcp::{RoleClient, ServiceExt};
use serde_json::json;
use termlink_session::client;
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
        "termlink_event_poll", "termlink_event_subscribe", "termlink_kv_set", "termlink_kv_get",
        "termlink_kv_list", "termlink_kv_del", "termlink_broadcast",
        "termlink_wait", "termlink_spawn", "termlink_run", "termlink_status",
        "termlink_interact", "termlink_doctor", "termlink_clean",
        "termlink_tag", "termlink_request", "termlink_resize",
        "termlink_version", "termlink_token_create", "termlink_token_inspect",
        "termlink_file_receive",
        "termlink_dispatch_status",
        "termlink_overview",
        "termlink_send",
        "termlink_batch_exec",
    ] {
        assert!(names.iter().any(|n| n == expected), "missing tool: {expected}");
    }
    assert!(tools.len() >= 43, "expected at least 43 tools, got {}", tools.len());

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
async fn test_list_sessions_filtered_by_role() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-list-filt");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h1, _r1) = start_session(&dir.sessions_dir(), "coder-one", vec!["coder".into()]).await;
    let (_h2, _r2) = start_session(&dir.sessions_dir(), "tester-one", vec!["tester".into()]).await;

    let client = mcp_client().await;

    // Filter by role=coder — should return only coder-one
    let text = call(&client, "termlink_list_sessions", json!({"role": "coder"})).await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 1, "expected 1 coder session, got: {text}");
    assert_eq!(parsed[0]["display_name"], "coder-one");

    // Filter by name substring
    let text = call(&client, "termlink_list_sessions", json!({"name": "tester"})).await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 1, "expected 1 tester session, got: {text}");
    assert_eq!(parsed[0]["display_name"], "tester-one");

    // No filter — returns all
    let text = call(&client, "termlink_list_sessions", json!({})).await;
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 2, "expected 2 sessions, got: {text}");

    client.cancel().await.unwrap();
    _h1.abort();
    _h2.abort();
}

#[tokio::test]
async fn test_ping_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-ping");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-ping-target", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_ping", json!({"target": "mcp-ping-target"})).await;

    assert!(!text.contains("\"ok\": false"), "ping failed: {text}");

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

    assert!(text.contains("error"), "expected error for nonexistent session: {text}");

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

    assert!(!text.contains("\"ok\": false"), "status failed: {text}");
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

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["ok"], true, "run should succeed: {text}");
    assert_eq!(parsed["exit_code"], 0);
    assert!(parsed["stdout"].as_str().unwrap_or("").contains("hello-from-mcp"));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_run_command_exit_code() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-run-exit");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_run", json!({"command": "false"})).await;

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["ok"], false, "run of false should not be ok");
    assert_ne!(parsed["exit_code"], 0, "exit code should be non-zero");

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

#[tokio::test]
async fn test_interact_non_pty_returns_error() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-interact-nopty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // start_session creates a non-PTY endpoint — interact should fail gracefully
    let (_h, _reg) = start_session(&dir.sessions_dir(), "interact-nopty", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_interact",
        json!({"target": "interact-nopty", "command": "echo hello"})).await;

    assert!(text.contains("error"), "expected PTY error for non-PTY session: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_interact_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-interact-bad");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_interact",
        json!({"target": "no-such-session", "command": "echo hello"})).await;

    assert!(text.contains("error") && text.contains("not found"),
        "expected not found error: {text}");

    client.cancel().await.unwrap();
}

// === Resource tests ===

#[tokio::test]
async fn test_list_resources_empty() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-res-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = client.list_all_resources().await.unwrap();

    // Should have the sessions list resource even with no sessions
    assert!(result.iter().any(|r| r.uri == "termlink://sessions"),
        "missing sessions resource");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_list_resources_with_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-res-sess");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h1, _r1) = start_session(&dir.sessions_dir(), "res-alpha", vec![]).await;
    let (_h2, _r2) = start_session(&dir.sessions_dir(), "res-beta", vec![]).await;

    let client = mcp_client().await;
    let result = client.list_all_resources().await.unwrap();

    // 1 sessions list + 2 per-session resources = 3
    assert!(result.len() >= 3, "expected >= 3 resources, got {}", result.len());
    assert!(result.iter().any(|r| r.uri == "termlink://sessions"));
    assert!(result.iter().any(|r| r.uri.contains("res-alpha") || r.name.contains("res-alpha")));
    assert!(result.iter().any(|r| r.uri.contains("res-beta") || r.name.contains("res-beta")));

    client.cancel().await.unwrap();
    _h1.abort();
    _h2.abort();
}

#[tokio::test]
async fn test_read_sessions_list_resource() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-res-read-list");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "res-read-tgt", vec!["agent".into()]).await;

    let client = mcp_client().await;
    let result = client
        .read_resource(ReadResourceRequestParams::new("termlink://sessions"))
        .await
        .unwrap();

    let text = match result.contents.first().expect("expected content") {
        ResourceContents::TextResourceContents { text, .. } => text.clone(),
        _ => panic!("expected text content"),
    };

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["display_name"], "res-read-tgt");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_read_session_detail_resource() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-res-detail");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, reg) = start_session(&dir.sessions_dir(), "res-detail-tgt", vec![]).await;

    let client = mcp_client().await;
    let result = client
        .read_resource(ReadResourceRequestParams::new(format!("termlink://sessions/{}", reg.id)))
        .await
        .unwrap();

    let text = match result.contents.first().expect("expected content") {
        ResourceContents::TextResourceContents { text, .. } => text.clone(),
        _ => panic!("expected text content"),
    };

    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(parsed.get("state").is_some() || parsed.get("display_name").is_some(),
        "expected session detail: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_read_resource_unknown_uri() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-res-unknown");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = client
        .read_resource(ReadResourceRequestParams::new("termlink://nonexistent"))
        .await;

    assert!(result.is_err(), "expected error for unknown URI");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_list_resource_templates() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-res-templates");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = client.list_resource_templates(None).await.unwrap();

    assert!(!result.resource_templates.is_empty(), "expected at least 1 template");
    assert!(result.resource_templates.iter().any(|t| t.uri_template.contains("{session_id}")),
        "expected session_id template");

    client.cancel().await.unwrap();
}

// === Prompt tests ===

#[tokio::test]
async fn test_list_prompts() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-prompts-list");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = client.list_all_prompts().await.unwrap();

    let names: Vec<&str> = result.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"debug_session"), "missing debug_session prompt");
    assert!(names.contains(&"session_overview"), "missing session_overview prompt");
    assert!(names.contains(&"orchestrate"), "missing orchestrate prompt");
    assert_eq!(result.len(), 3, "expected 3 prompts");

    // Check debug_session has required argument
    let debug = result.iter().find(|p| p.name == "debug_session").unwrap();
    let args = debug.arguments.as_ref().unwrap();
    assert!(args.iter().any(|a| a.name == "session" && a.required == Some(true)));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_get_prompt_session_overview_empty() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-prompt-overview-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = client
        .get_prompt(GetPromptRequestParams::new("session_overview"))
        .await
        .unwrap();

    assert!(!result.messages.is_empty(), "expected at least 1 message");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_get_prompt_session_overview_with_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-prompt-overview-sess");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h1, _r1) = start_session(&dir.sessions_dir(), "prompt-alpha", vec!["coder".into()]).await;
    let (_h2, _r2) = start_session(&dir.sessions_dir(), "prompt-beta", vec!["tester".into()]).await;

    let client = mcp_client().await;
    let result = client
        .get_prompt(GetPromptRequestParams::new("session_overview"))
        .await
        .unwrap();

    let text = format!("{:?}", result.messages);
    assert!(text.contains("prompt-alpha"), "should mention alpha: {text}");
    assert!(text.contains("prompt-beta"), "should mention beta: {text}");

    client.cancel().await.unwrap();
    _h1.abort();
    _h2.abort();
}

#[tokio::test]
async fn test_get_prompt_debug_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-prompt-debug");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "debug-target", vec![]).await;

    let client = mcp_client().await;
    let result = client
        .get_prompt(
            GetPromptRequestParams::new("debug_session")
                .with_arguments({
                    let mut m = serde_json::Map::new();
                    m.insert("session".into(), json!("debug-target"));
                    m
                })
        )
        .await
        .unwrap();

    let text = format!("{:?}", result.messages);
    assert!(text.contains("debug-target"), "should mention session name: {text}");
    assert!(text.contains("alive") || text.contains("Live Status"), "should include process status: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_get_prompt_orchestrate() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-prompt-orch");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "orch-worker", vec!["worker".into()]).await;

    let client = mcp_client().await;
    let result = client
        .get_prompt(
            GetPromptRequestParams::new("orchestrate")
                .with_arguments({
                    let mut m = serde_json::Map::new();
                    m.insert("task".into(), json!("Run tests on all workers"));
                    m.insert("role".into(), json!("worker"));
                    m
                })
        )
        .await
        .unwrap();

    let text = format!("{:?}", result.messages);
    assert!(text.contains("Run tests"), "should include task: {text}");
    assert!(text.contains("orch-worker"), "should include matching session: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_get_prompt_unknown() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-prompt-unknown");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = client
        .get_prompt(GetPromptRequestParams::new("nonexistent"))
        .await;

    assert!(result.is_err(), "expected error for unknown prompt");

    client.cancel().await.unwrap();
}

// === Doctor & Clean tool tests ===

#[tokio::test]
async fn test_doctor_empty_env() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-doctor-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_doctor", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("doctor should return valid JSON");
    assert!(parsed["checks"].is_array(), "should have checks array");
    assert!(parsed["summary"]["pass"].is_number(), "should have pass count");
    assert!(parsed["summary"]["warn"].is_number(), "should have warn count");
    assert!(parsed["summary"]["fail"].is_number(), "should have fail count");

    // In empty env: runtime_dir fail, sessions_dir warn, sessions pass, hub pass, version pass
    let total = parsed["summary"]["pass"].as_u64().unwrap_or(0)
        + parsed["summary"]["warn"].as_u64().unwrap_or(0)
        + parsed["summary"]["fail"].as_u64().unwrap_or(0);
    assert!(total >= 4, "expected at least 4 checks, got {total}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_doctor_with_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-doctor-sessions");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "doctor-test", vec!["worker".into()]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_doctor", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    // With a live session, sessions check should show 1 registered
    let checks = parsed["checks"].as_array().unwrap();
    let session_check = checks.iter().find(|c| c["check"] == "sessions").unwrap();
    let msg = session_check["message"].as_str().unwrap();
    assert!(msg.contains("1 registered"), "should show 1 registered session, got: {msg}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_clean_empty_env() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-clean-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_clean", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("clean should return valid JSON");
    assert_eq!(parsed["total"].as_u64().unwrap_or(999), 0, "nothing to clean in empty env");
    assert!(parsed["cleaned_sessions"].as_array().unwrap().is_empty());
    assert_eq!(parsed["cleaned_sockets"].as_u64().unwrap(), 0);
    assert!(!parsed["cleaned_hub"].as_bool().unwrap());

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_clean_removes_orphaned_socket() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-clean-orphan");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Create sessions dir with an orphaned socket (no matching .json)
    let sessions_dir = dir.path.join("sessions");
    std::fs::create_dir_all(&sessions_dir).unwrap();
    std::fs::write(sessions_dir.join("orphan.sock"), b"").unwrap();

    let client = mcp_client().await;
    let result = call(&client, "termlink_clean", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["cleaned_sockets"].as_u64().unwrap(), 1, "should clean 1 orphaned socket");
    assert!(!sessions_dir.join("orphan.sock").exists(), "socket should be removed");

    client.cancel().await.unwrap();
}

// === Tag tool tests ===

#[tokio::test]
async fn test_tag_add() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-tag-add");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "tag-target", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_tag", json!({
        "target": "tag-target",
        "add": ["project:alpha", "env:staging"]
    })).await;

    assert!(result.contains("Updated"), "should confirm update: {result}");
    assert!(result.contains("project:alpha"), "should include added tag: {result}");
    assert!(result.contains("env:staging"), "should include added tag: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tag_set_and_remove() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-tag-set-rm");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "tag-setrm", vec![]).await;

    let client = mcp_client().await;

    // Set tags
    let result = call(&client, "termlink_tag", json!({
        "target": "tag-setrm",
        "set": ["a", "b", "c"]
    })).await;
    assert!(result.contains("Updated"), "set should work: {result}");

    // Remove one
    let result = call(&client, "termlink_tag", json!({
        "target": "tag-setrm",
        "remove": ["b"]
    })).await;
    assert!(result.contains("Updated"), "remove should work: {result}");
    assert!(!result.contains(", b,") && !result.contains("[b]"), "b should be removed: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tag_no_params() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-tag-nop");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "tag-nop", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_tag", json!({
        "target": "tag-nop"
    })).await;

    assert!(result.contains("error"), "should error with no operation: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tag_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-tag-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_tag", json!({
        "target": "nonexistent",
        "add": ["test"]
    })).await;

    assert!(result.contains("error"), "should error for missing session: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tag_rename_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-tag-rename");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "rename-me", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_tag", json!({
        "target": "rename-me",
        "name": "renamed-session"
    })).await;

    assert!(!result.contains("\"ok\": false"), "should succeed: {result}");
    assert!(result.contains("renamed-session"), "should contain new name: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tag_set_roles() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-tag-roles");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "role-target", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_tag", json!({
        "target": "role-target",
        "roles": ["orchestrator", "monitor"]
    })).await;

    assert!(!result.contains("\"ok\": false"), "should succeed: {result}");
    assert!(result.contains("orchestrator"), "should contain orchestrator role: {result}");

    client.cancel().await.unwrap();
}

// === Request tool tests ===

#[tokio::test]
async fn test_request_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-req-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_request", json!({
        "target": "nonexistent",
        "topic": "task.run",
        "reply_topic": "task.result"
    })).await;

    assert!(result.contains("error"), "should error for missing session: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_request_with_reply() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-req-reply");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, reg) = start_session(&dir.sessions_dir(), "req-target", vec![]).await;

    let client = mcp_client().await;

    // Spawn a task that emits a reply after a short delay
    let socket = reg.socket_path().to_path_buf();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        // Poll to find the request_id
        let poll_resp = client::rpc_call(&socket, "event.poll", serde_json::json!({"topic": "task.run"})).await;
        if let Ok(resp) = poll_resp
            && let Ok(result) = client::unwrap_result(resp)
            && let Some(events) = result["events"].as_array()
            && let Some(event) = events.first()
        {
            let req_id = event["payload"]["request_id"].as_str().unwrap_or("unknown");
            // Emit the reply
            let _ = client::rpc_call(
                &socket,
                "event.emit",
                serde_json::json!({
                    "topic": "task.result",
                    "payload": {"request_id": req_id, "status": "done", "value": 42}
                }),
            ).await;
        }
    });

    let result = call(&client, "termlink_request", json!({
        "target": "req-target",
        "topic": "task.run",
        "payload": {"action": "compute"},
        "reply_topic": "task.result",
        "timeout": 5
    })).await;

    assert!(result.contains("done") || result.contains("42"), "should contain reply data: {result}");
    assert!(!result.contains("Timeout"), "should not timeout: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_request_timeout() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-req-timeout");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "req-timeout", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_request", json!({
        "target": "req-timeout",
        "topic": "task.run",
        "reply_topic": "task.result",
        "timeout": 1
    })).await;

    assert!(result.contains("Timeout"), "should timeout: {result}");

    client.cancel().await.unwrap();
}

// === Resize tool tests ===

#[tokio::test]
async fn test_resize_non_pty() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-resize-nopty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // start_session creates non-PTY sessions — resize should fail gracefully
    let (_h, _reg) = start_session(&dir.sessions_dir(), "resize-nopty", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_resize", json!({
        "target": "resize-nopty",
        "cols": 120,
        "rows": 40
    })).await;

    assert!(result.contains("error"), "non-PTY resize should error: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_resize_nonexistent() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-resize-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_resize", json!({
        "target": "nonexistent",
        "cols": 80,
        "rows": 24
    })).await;

    assert!(result.contains("error"), "should error for missing session: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_event_subscribe_with_history() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-subscribe");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-sub-tgt", vec![]).await;

    let client = mcp_client().await;

    // Emit two events
    call(&client, "termlink_emit",
        json!({"target": "mcp-sub-tgt", "topic": "sub.test", "payload": {"n": 1}})).await;
    call(&client, "termlink_emit",
        json!({"target": "mcp-sub-tgt", "topic": "sub.test", "payload": {"n": 2}})).await;

    // Subscribe with since=0 to replay events with seq > 0, short timeout
    // First event is seq 0, second is seq 1. since=0 returns seq > 0, so just seq 1.
    let text = call(&client, "termlink_event_subscribe",
        json!({"target": "mcp-sub-tgt", "since": 0, "timeout_ms": 500, "topic": "sub.test"})).await;
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let events = parsed["events"].as_array().unwrap();
    assert!(!events.is_empty(), "expected at least 1 historical event with seq > 0, got: {text}");
    assert_eq!(events[0]["topic"], "sub.test");
    assert!(parsed["next_seq"].is_u64(), "next_seq should be present");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_event_subscribe_timeout_empty() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-sub-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "mcp-sub-empty-tgt", vec![]).await;

    let client = mcp_client().await;

    // Subscribe without since, short timeout, no events emitted
    let text = call(&client, "termlink_event_subscribe",
        json!({"target": "mcp-sub-empty-tgt", "timeout_ms": 200})).await;
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed["count"], 0, "expected 0 events on empty subscribe: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

// === dispatch_status tests ===

#[tokio::test]
async fn test_dispatch_status_no_manifest() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-dispatch-status-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_dispatch_status", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should return valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["total"], 0);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_dispatch_status_with_manifest() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-dispatch-status-manifest");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Create a dispatch manifest in current dir
    let manifest_dir = std::env::current_dir().unwrap().join(".termlink");
    let _ = std::fs::create_dir_all(&manifest_dir);
    let manifest_path = manifest_dir.join("dispatch-manifest.json");
    let manifest_content = json!({
        "dispatches": [
            {
                "id": "D-test-1",
                "created_at": "2026-04-01T00:00:00Z",
                "status": "merged",
                "worker_count": 2,
                "topic": "task.completed",
                "prefix": "worker",
                "branches": []
            },
            {
                "id": "D-test-2",
                "created_at": "2026-04-01T01:00:00Z",
                "status": "pending",
                "worker_count": 3,
                "topic": "task.completed",
                "prefix": "worker",
                "branches": [
                    {"worker_name": "w-1", "branch_name": "tl-dispatch/D-test-2/w-1", "has_commits": true}
                ]
            }
        ]
    });
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest_content).unwrap()).unwrap();

    let client = mcp_client().await;
    let result = call(&client, "termlink_dispatch_status", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should return valid JSON");
    assert_eq!(parsed["ok"], false, "pending dispatches should report ok=false");
    assert_eq!(parsed["total"], 2);
    assert_eq!(parsed["pending"], 1);
    assert_eq!(parsed["merged"], 1);
    assert_eq!(parsed["pending_dispatches"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["pending_dispatches"][0]["id"], "D-test-2");

    // Cleanup
    let _ = std::fs::remove_file(&manifest_path);
    client.cancel().await.unwrap();
}

// === info tests ===

#[tokio::test]
async fn test_info_empty_env() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-info-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_info", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("info should return valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["version"].is_string(), "should have version");
    assert!(parsed["runtime_dir"].is_string(), "should have runtime_dir");
    assert!(parsed["hub_running"].is_boolean(), "should have hub_running");
    assert_eq!(parsed["sessions"]["live"], 0);
    assert_eq!(parsed["sessions"]["stale"], 0);
    assert_eq!(parsed["sessions"]["total"], 0);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_info_with_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-info-sessions");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "info-test", vec!["worker".into()]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_info", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["sessions"]["live"], 1);
    assert_eq!(parsed["sessions"]["total"], 1);

    client.cancel().await.unwrap();
}

// === topics tests ===

#[tokio::test]
async fn test_topics_empty_env() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-topics-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_topics", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("topics should return valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["total_topics"], 0);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_topics_with_events() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-topics-events");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, reg) = start_session(&dir.sessions_dir(), "topics-test", vec![]).await;

    // Emit an event to create a topic
    let _ = client::rpc_call(reg.socket_path(), "event.emit", json!({
        "topic": "build.complete",
        "payload": {"status": "ok"}
    })).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_topics", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["total_topics"].as_u64().unwrap() >= 1, "should have at least 1 topic");
    let sessions = parsed["sessions"].as_object().unwrap();
    assert!(sessions.contains_key("topics-test"), "should include topics-test session");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_topics_specific_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-topics-target");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, reg) = start_session(&dir.sessions_dir(), "topics-target", vec![]).await;

    let _ = client::rpc_call(reg.socket_path(), "event.emit", json!({
        "topic": "task.result",
        "payload": {}
    })).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_topics", json!({"target": "topics-target"})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["sessions"].as_object().unwrap().contains_key("topics-target"));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_topics_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-topics-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_topics", json!({"target": "nonexistent"})).await;
    assert!(result.contains("error"), "should return error for nonexistent session");

    client.cancel().await.unwrap();
}

// === collect tests ===

#[tokio::test]
async fn test_collect_no_hub() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-collect-no-hub");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_collect", json!({})).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("collect should return valid JSON");
    assert_eq!(parsed["ok"], false, "should fail without hub");
    assert!(parsed["error"].as_str().unwrap().contains("Hub is not running"),
        "should mention hub not running: {result}");

    client.cancel().await.unwrap();
}

// === pty_mode tests ===

#[tokio::test]
async fn test_pty_mode_non_pty_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-pty-mode-nopty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // start_session creates a non-PTY session (no --shell)
    let (_h, _reg) = start_session(&dir.sessions_dir(), "pty-mode-test", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_pty_mode", json!({"target": "pty-mode-test"})).await;

    // Non-PTY session should return an error about no PTY
    assert!(result.contains("error") || result.contains("PTY") || result.contains("pty"),
        "should indicate no PTY available: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_pty_mode_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-pty-mode-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_pty_mode", json!({"target": "nonexistent"})).await;
    assert!(result.contains("error"), "should return error for nonexistent session: {result}");

    client.cancel().await.unwrap();
}

// ─── Overview Tests ───────────────────────────────────────────────

#[tokio::test]
async fn test_overview_empty_workspace() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-overview-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_overview", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {result}"));

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["session_count"], 0);
    assert!(parsed["sessions"].as_array().unwrap().is_empty());
    assert!(parsed["runtime_dir"].is_string());
    assert!(parsed["version"].is_string());
    assert!(parsed["mcp_tools"].as_u64().unwrap() >= 43);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_overview_with_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-overview-sessions");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h1, _r1) = start_session(&dir.sessions_dir(), "overview-s1", vec!["worker".into()]).await;
    let (_h2, _r2) = start_session(&dir.sessions_dir(), "overview-s2", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_overview", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {result}"));

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["session_count"], 2);
    let sessions = parsed["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    // Check that session details include expected fields
    assert!(sessions[0]["id"].is_string());
    assert!(sessions[0]["name"].is_string());
    assert!(sessions[0]["state"].is_string());
    assert!(sessions[0]["alive"].is_boolean());

    client.cancel().await.unwrap();
    _h1.abort();
    _h2.abort();
}

// ─── Exec Tests ───────────────────────────────────────────────────

#[tokio::test]
async fn test_exec_echo_command() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-exec-echo");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "exec-target", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_exec", json!({
        "target": "exec-target",
        "command": "echo hello-mcp-exec"
    })).await;

    let parsed: serde_json::Value = serde_json::from_str(&result)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {result}"));
    assert_eq!(parsed["ok"], true, "exec should succeed: {result}");
    assert_eq!(parsed["exit_code"], 0, "exit code should be 0: {result}");
    assert!(
        parsed["stdout"].as_str().unwrap_or("").contains("hello-mcp-exec"),
        "stdout should contain command output: {result}"
    );
    assert_eq!(parsed["target"], "exec-target");

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_exec_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-exec-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_exec", json!({
        "target": "nonexistent",
        "command": "echo test"
    })).await;
    assert!(result.contains("error"), "should error for nonexistent session: {result}");
    assert!(result.contains("not found"), "should mention not found: {result}");

    client.cancel().await.unwrap();
}

// ─── Spawn Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_spawn_background() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-spawn-bg");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_spawn", json!({
        "name": "spawn-test-worker",
        "command": ["echo", "spawned"],
        "wait": false
    })).await;

    // Spawn may succeed or fail depending on terminal availability,
    // but it should return a response (not crash)
    assert!(!result.is_empty(), "spawn should return a response");

    client.cancel().await.unwrap();
}

// ─── Hub Status Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_hub_status_not_running() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-hub-status-off");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_hub_status", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["status"], "not_running");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_hub_status_running() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-hub-status-on");
    let sessions_dir = dir.path.join("sessions");
    std::fs::create_dir_all(&sessions_dir).unwrap();
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Start a hub
    let socket_path = dir.path.join("hub.sock");
    let hub_handle = tokio::spawn({
        let socket = socket_path.clone();
        async move {
            let _ = termlink_hub::server::run(&socket).await;
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_hub_status", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["status"], "running");
    assert!(parsed["pid"].is_u64(), "should include pid");
    assert!(parsed["socket"].is_string(), "should include socket path");

    client.cancel().await.unwrap();
    hub_handle.abort();
}

// ─── File Send Tests ───────────────────────────────────────────────

#[tokio::test]
async fn test_file_send_to_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-file-send");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "file-recv", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Create a temp file to send
    let test_file = dir.path.join("test-payload.txt");
    std::fs::write(&test_file, "hello from MCP file_send test").unwrap();

    let client = mcp_client().await;
    let result = call(&client, "termlink_file_send", json!({
        "target": "file-recv",
        "path": test_file.to_str().unwrap()
    })).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["filename"], "test-payload.txt");
    assert_eq!(parsed["size"], 29); // "hello from MCP file_send test" = 29 bytes
    assert_eq!(parsed["chunks"], 1);
    assert!(parsed["transfer_id"].is_string());
    assert!(parsed["sha256"].is_string());

    client.cancel().await.unwrap();
    _h.abort();
}

#[tokio::test]
async fn test_file_send_nonexistent_target() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-file-send-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Create a temp file
    let test_file = dir.path.join("payload.txt");
    std::fs::write(&test_file, "data").unwrap();

    let client = mcp_client().await;
    let result = call(&client, "termlink_file_send", json!({
        "target": "nonexistent",
        "path": test_file.to_str().unwrap()
    })).await;
    assert!(result.contains("error"), "should error for nonexistent target: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_file_send_nonexistent_file() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-file-send-nofile");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "file-recv2", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_file_send", json!({
        "target": "file-recv2",
        "path": "/nonexistent/path/file.txt"
    })).await;
    assert!(result.contains("error"), "should error for nonexistent file: {result}");

    client.cancel().await.unwrap();
    _h.abort();
}

// ─── File Receive Tests ───────────────────────────────────────────

#[tokio::test]
async fn test_file_receive_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-file-recv-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_file_receive", json!({
        "target": "nonexistent",
        "output_dir": "/tmp"
    })).await;
    assert!(result.contains("error"), "should error for nonexistent session: {result}");
    assert!(result.contains("not found"), "should mention not found: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_file_receive_no_transfer() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-file-recv-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "empty-session", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_file_receive", json!({
        "target": "empty-session",
        "output_dir": "/tmp"
    })).await;
    assert!(result.contains("error"), "should error when no file transfer: {result}");
    assert!(result.contains("no file transfer"), "should mention no transfer: {result}");

    client.cancel().await.unwrap();
    _h.abort();
}

// ─── Agent Ask Tests ───────────────────────────────────────────────

#[tokio::test]
async fn test_agent_ask_nonexistent_target() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-agent-ask-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_agent_ask", json!({
        "target": "nonexistent",
        "action": "test"
    })).await;
    assert!(result.contains("error"), "should error for nonexistent target: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_agent_ask_timeout() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-agent-ask-timeout");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "agent-target", vec![]).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = mcp_client().await;
    // Short timeout (1s) — no responder, so this should timeout
    let result = call(&client, "termlink_agent_ask", json!({
        "target": "agent-target",
        "action": "analyze",
        "params": {"code": "hello"},
        "timeout": 1
    })).await;
    assert!(result.contains("Timeout"), "should timeout with no responder: {result}");

    client.cancel().await.unwrap();
    _h.abort();
}

// ─── Hub Start/Stop Tests ──────────────────────────────────────────

#[tokio::test]
async fn test_hub_start_and_status() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-hub-start");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;

    // Start the hub
    let result = call(&client, "termlink_hub_start", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["action"], "started");

    // Verify status shows running
    let status = call(&client, "termlink_hub_status", json!({})).await;
    let status_parsed: serde_json::Value = serde_json::from_str(&status).expect("valid JSON");
    assert_eq!(status_parsed["ok"], true);
    assert_eq!(status_parsed["status"], "running");

    // Start again — should be already_running
    let result2 = call(&client, "termlink_hub_start", json!({})).await;
    let parsed2: serde_json::Value = serde_json::from_str(&result2).expect("valid JSON");
    assert_eq!(parsed2["action"], "already_running");

    // NOTE: Not testing hub_stop here because the hub runs in-process;
    // SIGTERM would kill the test process itself. hub_stop is tested
    // via test_hub_stop_not_running (no-op case) and test_hub_stop_stale (cleanup).

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_hub_stop_stale() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-hub-stop-stale");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Create a fake stale pidfile pointing to a dead PID
    let pidfile = dir.path.join("hub.pid");
    std::fs::write(&pidfile, "999999").unwrap(); // PID that doesn't exist

    let client = mcp_client().await;
    let result = call(&client, "termlink_hub_stop", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["action"], "cleaned");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_hub_stop_not_running() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-hub-stop-none");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_hub_stop", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["action"], "none");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_version() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-version");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let result = call(&client, "termlink_version", json!({})).await;
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    assert_eq!(parsed["ok"], true);
    assert!(parsed["version"].is_string(), "version should be a string");
    assert!(parsed["commit"].is_string(), "commit should be a string");
    assert!(parsed["target"].is_string(), "target should be a string");
    assert!(parsed["mcp_tools"].is_number(), "mcp_tools should be a number");
    assert!(parsed["mcp_tools"].as_u64().unwrap() >= 42, "expected at least 41 tools");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_token_create_no_token_secret() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-token-no-secret");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "token-test", vec![]).await;

    let client = mcp_client().await;
    let result = call(&client, "termlink_token_create", json!({"target": "token-test"})).await;

    assert!(result.contains("error"), "should return error for non-token session: {result}");
    assert!(result.contains("token auth enabled"), "should mention token auth: {result}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_token_create_with_secret() {
    use termlink_session::registration::Registration;

    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-token-create");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, mut reg) = start_session(&dir.sessions_dir(), "token-sess", vec![]).await;

    // Set a token secret on the registration file
    let secret_hex = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
    reg.token_secret = Some(secret_hex.to_string());
    let json_path = Registration::json_path(&dir.sessions_dir(), &reg.id);
    reg.write_atomic(&json_path).unwrap();

    let client = mcp_client().await;
    let result = call(&client, "termlink_token_create", json!({
        "target": "token-sess",
        "scope": "execute",
        "ttl": 600
    })).await;

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["token"].is_string(), "should return token string");
    assert_eq!(parsed["scope"], "execute");
    assert_eq!(parsed["ttl"], 600);
    assert!(parsed["session"].is_string(), "should include session ID");

    client.cancel().await.unwrap();
}

// --- signal tests ---

#[tokio::test]
async fn test_signal_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-signal-bad");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_signal", json!({
        "target": "no-such-session",
        "signal": "TERM"
    })).await;

    assert!(text.contains("error"), "expected error for nonexistent session: {text}");

    client.cancel().await.unwrap();
}

// --- output tests ---

#[tokio::test]
async fn test_output_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-output-bad");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_output", json!({
        "target": "no-such-session"
    })).await;

    assert!(text.contains("error"), "expected error for nonexistent session: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_output_non_pty_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-output-nopty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "output-nopty", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_output", json!({
        "target": "output-nopty"
    })).await;

    // Non-PTY sessions return an error from query.output
    assert!(text.contains("error") || text.contains("error"), "expected error for non-PTY session: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

// --- broadcast tests ---

#[tokio::test]
async fn test_broadcast_no_hub() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-broadcast-nohub");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_broadcast", json!({
        "topic": "test.event"
    })).await;

    assert!(text.contains("hub is not running") || text.contains("error"),
        "expected hub-not-running error: {text}");

    client.cancel().await.unwrap();
}

// --- emit_to tests ---

#[tokio::test]
async fn test_emit_to_no_hub() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-emit-to-nohub");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_emit_to", json!({
        "target": "some-session",
        "topic": "test.event"
    })).await;

    assert!(text.contains("hub is not running") || text.contains("error"),
        "expected hub-not-running error: {text}");

    client.cancel().await.unwrap();
}

// --- inject tests ---

#[tokio::test]
async fn test_inject_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-inject-bad");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_inject", json!({
        "target": "no-such-session",
        "text": "hello"
    })).await;

    assert!(text.contains("error"), "expected error for nonexistent session: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_inject_with_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-inject-sess");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "inject-target", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_inject", json!({
        "target": "inject-target",
        "text": "echo hello"
    })).await;

    // Inject succeeds on any session that handles command.inject
    assert!(text.contains("Injected") || text.contains("ok"),
        "expected success for inject: {text}");

    client.cancel().await.unwrap();
    _h.abort();
}

// --- token_inspect tests ---

#[tokio::test]
async fn test_token_inspect_valid() {
    use base64::Engine;

    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-token-inspect");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let payload = serde_json::json!({
        "session": "test-session",
        "scope": "execute",
        "expires_at": 9999999999u64,
    });
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&payload).unwrap(),
    );
    let token = format!("{payload_b64}.fakesig");

    let client = mcp_client().await;
    let text = call(&client, "termlink_token_inspect", json!({"token": token})).await;

    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["payload"]["session"], "test-session");
    assert_eq!(parsed["payload"]["scope"], "execute");
    assert_eq!(parsed["expired"], false);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_token_inspect_invalid_format() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-token-inspect-bad");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_token_inspect", json!({"token": "no-dot"})).await;

    assert!(text.contains("error"), "expected error for invalid format: {text}");
    assert!(text.contains("payload.signature"), "should mention expected format: {text}");

    client.cancel().await.unwrap();
}

// ─── Send (generic RPC) Tests ───────────────────────────────────────

#[tokio::test]
async fn test_send_nonexistent_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-send-noexist");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_send", json!({
        "target": "ghost-session",
        "method": "termlink.ping"
    })).await;

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"].as_str().unwrap().contains("not found"));

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_send_invalid_params_json() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-send-badjson");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Need a live session so we get past the find_session check
    let (_h, _reg) = start_session(&dir.sessions_dir(), "send-badjson", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_send", json!({
        "target": "send-badjson",
        "method": "termlink.ping",
        "params": "not valid json {"
    })).await;

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"].as_str().unwrap().contains("invalid JSON params"),
        "expected invalid JSON params error, got: {}", parsed["error"]);

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_send_ping_live_session() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-send-ping");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let (_h, _reg) = start_session(&dir.sessions_dir(), "send-target", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_send", json!({
        "target": "send-target",
        "method": "termlink.ping"
    })).await;

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["target"], "send-target");
    assert_eq!(parsed["method"], "termlink.ping");
    assert!(parsed["result"].is_object(), "expected result object");

    client.cancel().await.unwrap();
}

// === batch_exec tests ===

#[tokio::test]
async fn test_batch_exec_no_matches() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-batch-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let text = call(&client, "termlink_batch_exec", json!({
        "command": "echo hello",
        "tag": "nonexistent-tag"
    })).await;

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["total"], 0);
    assert_eq!(parsed["succeeded"], 0);
    assert!(parsed["message"].as_str().unwrap().contains("No sessions"), "should say no matches: {text}");

    client.cancel().await.unwrap();
}

#[tokio::test]
async fn test_batch_exec_filter_by_name() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("mcp-batch-filter");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Create two sessions — only one should match
    let (_h1, _r1) = start_session(&dir.sessions_dir(), "batch-alpha", vec![]).await;
    let (_h2, _r2) = start_session(&dir.sessions_dir(), "batch-beta", vec![]).await;

    let client = mcp_client().await;
    let text = call(&client, "termlink_batch_exec", json!({
        "command": "echo hello",
        "name": "batch-alpha"
    })).await;

    // The exec will fail (no shell on test session) but we can verify filtering worked
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {text}"));
    assert_eq!(parsed["total"], 1, "should match exactly one session: {text}");

    client.cancel().await.unwrap();
}
