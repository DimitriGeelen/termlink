//! End-to-end integration tests for multi-session communication.
//!
//! These tests spin up real sessions with Unix socket listeners and verify
//! the full request/response cycle across the stack.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;

use termlink_session::client::{self, Client};
use termlink_session::manager::{self, Session};
use termlink_session::registration::{Registration, SessionConfig};
use termlink_session::server;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_dir(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(format!("/tmp/tl-int-{}-{}", n, name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Register a session and start its accept loop, returning the handle and registration.
async fn start_session(
    sessions_dir: &std::path::Path,
    name: &str,
    roles: Vec<String>,
) -> (tokio::task::JoinHandle<()>, Registration) {
    let config = SessionConfig {
        display_name: Some(name.into()),
        roles,
        ..Default::default()
    };
    let session = Session::register_in(config, sessions_dir)
        .await
        .unwrap();

    let reg = session.registration.clone();
    let shared = Arc::new(RwLock::new(session.registration));
    let listener = session.listener;

    let handle = tokio::spawn(async move {
        server::run_accept_loop(listener, shared).await;
    });

    // Give the accept loop a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    (handle, reg)
}

#[tokio::test]
async fn two_sessions_ping_each_other() {
    let dir = unique_dir("ping");

    let (h_alice, reg_alice) = start_session(&dir, "alice", vec![]).await;
    let (h_bob, reg_bob) = start_session(&dir, "bob", vec![]).await;

    // Bob pings Alice
    let resp = client::rpc_call(&reg_alice.socket, "termlink.ping", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["display_name"], "alice");
    assert_eq!(result["state"], "ready");

    // Alice pings Bob
    let resp = client::rpc_call(&reg_bob.socket, "termlink.ping", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["display_name"], "bob");
    assert_eq!(result["state"], "ready");

    h_alice.abort();
    h_bob.abort();
}

#[tokio::test]
async fn session_executes_command_on_another() {
    let dir = unique_dir("exec");

    let (h_target, reg_target) = start_session(&dir, "worker", vec!["executor".into()]).await;

    // A "client" session sends a command.execute to the worker
    let resp = client::rpc_call(
        &reg_target.socket,
        "command.execute",
        json!({ "command": "echo hello-from-remote" }),
    )
    .await
    .unwrap();

    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["exit_code"], 0);
    assert!(result["stdout"]
        .as_str()
        .unwrap()
        .contains("hello-from-remote"));

    h_target.abort();
}

#[tokio::test]
async fn discovery_lists_all_sessions() {
    let dir = unique_dir("disc");

    let (h1, _) = start_session(&dir, "session-x", vec!["coder".into()]).await;
    let (h2, _) = start_session(&dir, "session-y", vec!["reviewer".into()]).await;
    let (h3, _) = start_session(&dir, "session-z", vec![]).await;

    let sessions = manager::list_sessions_in(&dir, false).unwrap();
    assert_eq!(sessions.len(), 3);

    let names: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
    assert!(names.contains(&"session-x"));
    assert!(names.contains(&"session-y"));
    assert!(names.contains(&"session-z"));

    // Verify roles are discoverable
    let coder = sessions.iter().find(|s| s.display_name == "session-x").unwrap();
    assert!(coder.roles.contains(&"coder".to_string()));

    h1.abort();
    h2.abort();
    h3.abort();
}

#[tokio::test]
async fn deregister_removes_session_from_discovery() {
    let dir = unique_dir("dereg");

    let (h_keep, _) = start_session(&dir, "keeper", vec![]).await;

    // Register and immediately deregister
    let config = SessionConfig {
        display_name: Some("ephemeral".into()),
        ..Default::default()
    };
    let ephemeral = Session::register_in(config, &dir).await.unwrap();
    let eph_id = ephemeral.id().clone();

    // Verify it's visible
    let sessions = manager::list_sessions_in(&dir, false).unwrap();
    assert_eq!(sessions.len(), 2);

    // Deregister
    ephemeral.deregister().unwrap();

    // Verify it's gone
    let sessions = manager::list_sessions_in(&dir, false).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].display_name, "keeper");

    // Verify find_session_in also can't find it
    assert!(manager::find_session_in(&dir, eph_id.as_str()).is_err());

    h_keep.abort();
}

#[tokio::test]
async fn multi_request_conversation() {
    let dir = unique_dir("conv");

    let (handle, reg) = start_session(&dir, "conversant", vec!["query".into()]).await;

    // Use a persistent client for multiple requests
    let mut client = Client::connect(&reg.socket).await.unwrap();

    // 1. Ping
    let resp = client
        .call("termlink.ping", json!(1), json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["display_name"], "conversant");

    // 2. Query status
    let resp = client
        .call("query.status", json!(2), json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert!(result["pid"].as_u64().unwrap() > 0);
    assert!(result["created_at"].is_string());

    // 3. Query capabilities
    let resp = client
        .call("query.capabilities", json!(3), json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert!(result["capabilities"].is_array());
    assert!(result["roles"].as_array().unwrap().contains(&json!("query")));

    // 4. Execute a command
    let resp = client
        .call(
            "command.execute",
            json!(4),
            json!({ "command": "printf 'hello world'" }),
        )
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["exit_code"], 0);
    assert_eq!(result["stdout"], "hello world");

    // 5. Heartbeat
    let resp = client
        .call("session.heartbeat", json!(5), json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert!(result["heartbeat_at"].is_string());

    // 6. Unknown method → error
    let resp = client
        .call("nonexistent.method", json!(6), json!({}))
        .await
        .unwrap();
    if let termlink_protocol::jsonrpc::RpcResponse::Error(err) = resp {
        assert_eq!(err.error.code, -32601); // method not found
    } else {
        panic!("Expected error for unknown method");
    }

    handle.abort();
}

#[tokio::test]
async fn cross_session_exec_with_env_and_cwd() {
    let dir = unique_dir("xexec");

    let (handle, reg) = start_session(&dir, "env-worker", vec![]).await;

    let resp = client::rpc_call(
        &reg.socket,
        "command.execute",
        json!({
            "command": "echo $TL_TEST_VAR",
            "env": { "TL_TEST_VAR": "integration-pass" },
            "cwd": "/tmp"
        }),
    )
    .await
    .unwrap();

    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["exit_code"], 0);
    assert!(result["stdout"]
        .as_str()
        .unwrap()
        .contains("integration-pass"));

    handle.abort();
}

#[tokio::test]
async fn find_session_by_name_across_directory() {
    let dir = unique_dir("findname");

    let (h1, _) = start_session(&dir, "alpha", vec![]).await;
    let (h2, _) = start_session(&dir, "beta", vec![]).await;

    // Find by name
    let found = manager::find_session_in(&dir, "beta").unwrap();
    assert_eq!(found.display_name, "beta");

    // Find by ID
    let found_by_id = manager::find_session_in(&dir, found.id.as_str()).unwrap();
    assert_eq!(found_by_id.display_name, "beta");

    // Not found
    assert!(manager::find_session_in(&dir, "gamma").is_err());

    h1.abort();
    h2.abort();
}
