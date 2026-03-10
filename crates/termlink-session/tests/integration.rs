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
use termlink_session::codec::{FrameReader, FrameWriter};
use termlink_session::data_server;
use termlink_session::handler::SessionContext;
use termlink_session::manager::{self, Session};
use termlink_session::pty::PtySession;
use termlink_session::registration::{Registration, SessionConfig};
use termlink_session::server;

use termlink_protocol::data::{FrameFlags, FrameType};

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

    let (registration, listener, _sessions_dir) = session.into_parts();
    let reg = registration.clone();
    let ctx = SessionContext::new(registration);
    let shared = Arc::new(RwLock::new(ctx));

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

// ─── Data Plane Integration Tests ───────────────────────────────────

/// Start a PTY-backed session with data plane server.
/// Returns handles for cleanup, the registration, and the data socket path.
async fn start_pty_session(
    sessions_dir: &std::path::Path,
    name: &str,
) -> (
    Vec<tokio::task::JoinHandle<()>>,
    Registration,
    std::path::PathBuf,
    Arc<PtySession>,
) {
    let config = SessionConfig {
        display_name: Some(name.into()),
        capabilities: vec![
            "inject".into(),
            "command".into(),
            "query".into(),
            "data_plane".into(),
            "stream".into(),
        ],
        roles: vec![],
        tags: vec![],
    };
    let session = Session::register_in(config, sessions_dir)
        .await
        .unwrap();

    let pty = Arc::new(PtySession::spawn(Some("/bin/sh"), 1024 * 64).unwrap());
    let (registration, listener, _sessions_dir) = session.into_parts();
    let reg = registration.clone();
    let data_socket = data_server::data_socket_path(&reg.socket);

    // Start control plane
    let ctx = SessionContext::with_pty(registration, pty.clone());
    let shared = Arc::new(RwLock::new(ctx));
    let ctrl_handle = tokio::spawn(async move {
        server::run_accept_loop(listener, shared).await;
    });

    // Start data plane
    let (tx, rx) = tokio::sync::broadcast::channel::<Vec<u8>>(256);
    let data_pty = pty.clone();
    let data_path = data_socket.clone();
    let data_handle = tokio::spawn(async move {
        let _ = data_server::run(&data_path, data_pty, rx).await;
    });

    // Start PTY read loop with broadcast
    let read_pty = pty.clone();
    let read_handle = tokio::spawn(async move {
        let _ = read_pty.read_loop_with_broadcast(Some(tx)).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    (
        vec![ctrl_handle, data_handle, read_handle],
        reg,
        data_socket,
        pty,
    )
}

#[tokio::test]
async fn data_plane_stream_output() {
    let dir = unique_dir("dp-out");
    let (handles, reg, data_socket, pty) = start_pty_session(&dir, "streamer").await;

    // Connect to data plane
    let stream = tokio::net::UnixStream::connect(&data_socket).await.unwrap();
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let _writer = FrameWriter::new(write_half);

    // Give handler time to start select loop
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    // Inject a command via control plane
    let resp = client::rpc_call(
        &reg.socket,
        "command.inject",
        json!({ "keys": [{ "type": "text", "value": "echo DATA_PLANE_TEST\n" }] }),
    )
    .await
    .unwrap();
    assert!(client::unwrap_result(resp).is_ok());

    // Read frames from data plane — should see the output
    let mut saw_marker = false;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    while tokio::time::Instant::now() < deadline {
        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            reader.read_frame(),
        )
        .await;

        match frame {
            Ok(Ok(Some(f))) if f.header.frame_type == FrameType::Output => {
                let text = String::from_utf8_lossy(&f.payload);
                if text.contains("DATA_PLANE_TEST") {
                    saw_marker = true;
                    break;
                }
            }
            _ => continue,
        }
    }
    assert!(saw_marker, "Expected DATA_PLANE_TEST in data plane output");

    for h in handles {
        h.abort();
    }
    let _ = pty.signal(libc::SIGTERM);
    let _ = std::fs::remove_file(&data_socket);
}

#[tokio::test]
async fn data_plane_input_and_output() {
    let dir = unique_dir("dp-io");
    let (handles, _reg, data_socket, pty) = start_pty_session(&dir, "bidir").await;

    // Connect to data plane
    let stream = tokio::net::UnixStream::connect(&data_socket).await.unwrap();
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    // Send input via data plane (Input frame)
    writer
        .write_frame(
            FrameType::Input,
            FrameFlags::empty(),
            0,
            b"echo BIDIR_MARKER\n",
        )
        .await
        .unwrap();

    // Read frames — should see BIDIR_MARKER in output
    let mut saw_marker = false;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    while tokio::time::Instant::now() < deadline {
        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            reader.read_frame(),
        )
        .await;

        match frame {
            Ok(Ok(Some(f))) if f.header.frame_type == FrameType::Output => {
                let text = String::from_utf8_lossy(&f.payload);
                if text.contains("BIDIR_MARKER") {
                    saw_marker = true;
                    break;
                }
            }
            _ => continue,
        }
    }
    assert!(saw_marker, "Expected BIDIR_MARKER in data plane output");

    // Send Close frame
    writer
        .write_frame(FrameType::Close, FrameFlags::empty(), 0, &[])
        .await
        .unwrap();

    for h in handles {
        h.abort();
    }
    let _ = pty.signal(libc::SIGTERM);
    let _ = std::fs::remove_file(&data_socket);
}

#[tokio::test]
async fn data_plane_ping_pong_integration() {
    let dir = unique_dir("dp-ping");
    let (handles, _reg, data_socket, pty) = start_pty_session(&dir, "pinger").await;

    let stream = tokio::net::UnixStream::connect(&data_socket).await.unwrap();
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    // Send Ping
    writer
        .write_frame(FrameType::Ping, FrameFlags::empty(), 0, b"test-ping")
        .await
        .unwrap();

    // Should get Pong back (may need to skip Output frames from shell startup)
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    let mut got_pong = false;
    while tokio::time::Instant::now() < deadline {
        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            reader.read_frame(),
        )
        .await;

        match frame {
            Ok(Ok(Some(f))) if f.header.frame_type == FrameType::Pong => {
                assert_eq!(f.payload, b"test-ping");
                got_pong = true;
                break;
            }
            Ok(Ok(Some(_))) => continue, // skip Output frames
            _ => break,
        }
    }
    assert!(got_pong, "Expected Pong response");

    for h in handles {
        h.abort();
    }
    let _ = pty.signal(libc::SIGTERM);
    let _ = std::fs::remove_file(&data_socket);
}

#[tokio::test]
async fn data_plane_capabilities_in_status() {
    let dir = unique_dir("dp-caps");
    let (handles, reg, data_socket, pty) = start_pty_session(&dir, "capable").await;

    // Query status via control plane
    let resp = client::rpc_call(&reg.socket, "query.status", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();

    // Should have data_plane and stream capabilities
    let caps = result["capabilities"].as_array().unwrap();
    let cap_strs: Vec<&str> = caps.iter().filter_map(|c| c.as_str()).collect();
    assert!(cap_strs.contains(&"data_plane"), "Expected data_plane capability, got: {:?}", cap_strs);
    assert!(cap_strs.contains(&"stream"), "Expected stream capability, got: {:?}", cap_strs);

    // Should have has_pty
    assert_eq!(result["has_pty"], true);

    for h in handles {
        h.abort();
    }
    let _ = pty.signal(libc::SIGTERM);
    let _ = std::fs::remove_file(&data_socket);
}

// ─── Event Bus Integration Tests ────────────────────────────────────

#[tokio::test]
async fn event_emit_and_poll() {
    let dir = unique_dir("evt-ep");

    let (handle, reg) = start_session(&dir, "emitter", vec![]).await;

    // Emit an event
    let resp = client::rpc_call(
        &reg.socket,
        "event.emit",
        json!({ "topic": "build.done", "payload": { "status": "ok" } }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["topic"], "build.done");
    assert!(result["seq"].as_u64().is_some());

    // Poll for events (cursor=0 gets all)
    let resp = client::rpc_call(
        &reg.socket,
        "event.poll",
        json!({ "cursor": 0 }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    let events = result["events"].as_array().unwrap();
    assert!(!events.is_empty());
    assert_eq!(events[0]["topic"], "build.done");
    assert_eq!(events[0]["payload"]["status"], "ok");

    handle.abort();
}

#[tokio::test]
async fn event_topics_lists_distinct_topics() {
    let dir = unique_dir("evt-topics");

    let (handle, reg) = start_session(&dir, "topicker", vec![]).await;

    // Emit events on different topics
    for topic in &["build.start", "build.done", "test.pass", "build.start"] {
        client::rpc_call(
            &reg.socket,
            "event.emit",
            json!({ "topic": topic, "payload": {} }),
        )
        .await
        .unwrap();
    }

    // Query topics
    let resp = client::rpc_call(&reg.socket, "event.topics", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    let topics = result["topics"].as_array().unwrap();
    let topic_strs: Vec<&str> = topics.iter().filter_map(|t| t.as_str()).collect();
    assert!(topic_strs.contains(&"build.start"));
    assert!(topic_strs.contains(&"build.done"));
    assert!(topic_strs.contains(&"test.pass"));
    assert_eq!(topics.len(), 3); // distinct

    handle.abort();
}

#[tokio::test]
async fn event_poll_with_topic_filter() {
    let dir = unique_dir("evt-filter");

    let (handle, reg) = start_session(&dir, "filterer", vec![]).await;

    // Emit mixed topics
    client::rpc_call(
        &reg.socket,
        "event.emit",
        json!({ "topic": "build.done", "payload": { "n": 1 } }),
    )
    .await
    .unwrap();
    client::rpc_call(
        &reg.socket,
        "event.emit",
        json!({ "topic": "test.fail", "payload": { "n": 2 } }),
    )
    .await
    .unwrap();
    client::rpc_call(
        &reg.socket,
        "event.emit",
        json!({ "topic": "build.done", "payload": { "n": 3 } }),
    )
    .await
    .unwrap();

    // Poll with topic filter
    let resp = client::rpc_call(
        &reg.socket,
        "event.poll",
        json!({ "cursor": 0, "topic": "build.done" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["payload"]["n"], 1);
    assert_eq!(events[1]["payload"]["n"], 3);

    handle.abort();
}

// ─── KV Store Integration Tests ─────────────────────────────────────

#[tokio::test]
async fn kv_set_get_list_delete_cycle() {
    let dir = unique_dir("kv-crud");

    let (handle, reg) = start_session(&dir, "kvstore", vec![]).await;

    // Set a key
    let resp = client::rpc_call(
        &reg.socket,
        "kv.set",
        json!({ "key": "color", "value": "blue" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["key"], "color");
    assert_eq!(result["replaced"], false);

    // Get the key
    let resp = client::rpc_call(
        &reg.socket,
        "kv.get",
        json!({ "key": "color" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["key"], "color");
    assert_eq!(result["value"], "blue");
    assert_eq!(result["found"], true);

    // Set another key with JSON value
    client::rpc_call(
        &reg.socket,
        "kv.set",
        json!({ "key": "config", "value": { "debug": true, "level": 3 } }),
    )
    .await
    .unwrap();

    // List all
    let resp = client::rpc_call(&reg.socket, "kv.list", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["count"], 2);
    let entries = result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);

    // Replace a key
    let resp = client::rpc_call(
        &reg.socket,
        "kv.set",
        json!({ "key": "color", "value": "red" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["replaced"], true);

    // Verify replacement
    let resp = client::rpc_call(
        &reg.socket,
        "kv.get",
        json!({ "key": "color" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["value"], "red");

    // Delete
    let resp = client::rpc_call(
        &reg.socket,
        "kv.delete",
        json!({ "key": "color" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["key"], "color");
    assert_eq!(result["deleted"], true);

    // Get deleted key
    let resp = client::rpc_call(
        &reg.socket,
        "kv.get",
        json!({ "key": "color" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["found"], false);

    // Delete non-existent
    let resp = client::rpc_call(
        &reg.socket,
        "kv.delete",
        json!({ "key": "nonexistent" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["deleted"], false);

    // Final list — should have only "config"
    let resp = client::rpc_call(&reg.socket, "kv.list", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["count"], 1);

    handle.abort();
}

#[tokio::test]
async fn kv_get_nonexistent_returns_not_found() {
    let dir = unique_dir("kv-notfound");

    let (handle, reg) = start_session(&dir, "kvempty", vec![]).await;

    let resp = client::rpc_call(
        &reg.socket,
        "kv.get",
        json!({ "key": "missing" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["found"], false);
    assert!(result["value"].is_null());

    handle.abort();
}
