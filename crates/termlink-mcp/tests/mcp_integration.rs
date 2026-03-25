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
        "termlink_event_poll", "termlink_kv_set", "termlink_kv_get",
        "termlink_kv_list", "termlink_kv_del", "termlink_broadcast",
        "termlink_wait", "termlink_spawn", "termlink_run", "termlink_status",
        "termlink_interact", "termlink_doctor", "termlink_clean",
        "termlink_tag", "termlink_request",
    ] {
        assert!(names.iter().any(|n| n == expected), "missing tool: {expected}");
    }
    assert!(tools.len() >= 24, "expected at least 24 tools, got {}", tools.len());

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

    assert!(text.contains("Error"), "expected PTY error for non-PTY session: {text}");

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

    assert!(text.contains("Error") && text.contains("not found"),
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
    assert_eq!(parsed["cleaned_hub"].as_bool().unwrap(), false);

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

    assert!(result.contains("Error"), "should error with no operation: {result}");

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

    assert!(result.contains("Error"), "should error for missing session: {result}");

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

    assert!(result.contains("Error"), "should error for missing session: {result}");

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
        if let Ok(resp) = poll_resp {
            if let Ok(result) = client::unwrap_result(resp) {
                if let Some(events) = result["events"].as_array() {
                    if let Some(event) = events.first() {
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
                }
            }
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
