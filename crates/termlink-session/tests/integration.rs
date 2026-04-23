//! End-to-end integration tests for multi-session communication.
//!
//! These tests spin up real sessions with Unix socket listeners and verify
//! the full request/response cycle across the stack.

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

use termlink_test_utils::{start_session, TestDir};

#[tokio::test]
async fn two_sessions_ping_each_other() {
    let dir = TestDir::new("ping");

    let (h_alice, reg_alice) = start_session(&dir.sessions_dir(),"alice", vec![]).await;
    let (h_bob, reg_bob) = start_session(&dir.sessions_dir(),"bob", vec![]).await;

    // Bob pings Alice
    let resp = client::rpc_call(reg_alice.socket_path(), "termlink.ping", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["display_name"], "alice");
    assert_eq!(result["state"], "ready");

    // Alice pings Bob
    let resp = client::rpc_call(reg_bob.socket_path(), "termlink.ping", json!({}))
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
    let dir = TestDir::new("exec");

    let (h_target, reg_target) = start_session(&dir.sessions_dir(),"worker", vec!["executor".into()]).await;

    // A "client" session sends a command.execute to the worker
    let resp = client::rpc_call(
        reg_target.socket_path(),
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
    let dir = TestDir::new("disc");

    let (h1, _) = start_session(&dir.sessions_dir(),"session-x", vec!["coder".into()]).await;
    let (h2, _) = start_session(&dir.sessions_dir(),"session-y", vec!["reviewer".into()]).await;
    let (h3, _) = start_session(&dir.sessions_dir(),"session-z", vec![]).await;

    let sessions = manager::list_sessions_in(&dir.sessions_dir(), false).unwrap();
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
    let dir = TestDir::new("dereg");

    let (h_keep, _) = start_session(&dir.sessions_dir(),"keeper", vec![]).await;

    // Register and immediately deregister
    let config = SessionConfig {
        display_name: Some("ephemeral".into()),
        ..Default::default()
    };
    let ephemeral = Session::register_in(config, &dir.sessions_dir()).await.unwrap();
    let eph_id = ephemeral.id().clone();

    // Verify it's visible
    let sessions = manager::list_sessions_in(&dir.sessions_dir(), false).unwrap();
    assert_eq!(sessions.len(), 2);

    // Deregister
    ephemeral.deregister().unwrap();

    // Verify it's gone
    let sessions = manager::list_sessions_in(&dir.sessions_dir(), false).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].display_name, "keeper");

    // Verify find_session_in also can't find it
    assert!(manager::find_session_in(&dir.sessions_dir(), eph_id.as_str()).is_err());

    h_keep.abort();
}

#[tokio::test]
async fn multi_request_conversation() {
    let dir = TestDir::new("conv");

    let (handle, reg) = start_session(&dir.sessions_dir(),"conversant", vec!["query".into()]).await;

    // Use a persistent client for multiple requests
    let mut client = Client::connect(reg.socket_path()).await.unwrap();

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
    let dir = TestDir::new("xexec");

    let (handle, reg) = start_session(&dir.sessions_dir(),"env-worker", vec![]).await;

    let resp = client::rpc_call(
        reg.socket_path(),
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
    let dir = TestDir::new("findname");

    let (h1, _) = start_session(&dir.sessions_dir(),"alpha", vec![]).await;
    let (h2, _) = start_session(&dir.sessions_dir(),"beta", vec![]).await;

    // Find by name
    let found = manager::find_session_in(&dir.sessions_dir(), "beta").unwrap();
    assert_eq!(found.display_name, "beta");

    // Find by ID
    let found_by_id = manager::find_session_in(&dir.sessions_dir(), found.id.as_str()).unwrap();
    assert_eq!(found_by_id.display_name, "beta");

    // Not found
    assert!(manager::find_session_in(&dir.sessions_dir(), "gamma").is_err());

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

    // Retry PTY spawn with back-off — openpty can fail transiently under parallel load
    let pty = Arc::new({
        let mut session = None;
        for attempt in 0..5 {
            match PtySession::spawn(Some("/bin/sh"), 1024 * 64) {
                Ok(s) => { session = Some(s); break; }
                Err(e) => {
                    if attempt == 4 {
                        panic!("PtySession::spawn failed after 5 retries: {e}");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200 * (attempt + 1)));
                }
            }
        }
        session.unwrap()
    });
    let (registration, listener, _sessions_dir) = session.into_parts();
    let reg = registration.clone();
    let data_socket = data_server::data_socket_path(reg.socket_path());

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
    let dir = TestDir::new("dp-out");
    let (handles, reg, data_socket, pty) = start_pty_session(&dir.sessions_dir(),"streamer").await;

    // Connect to data plane
    let stream = tokio::net::UnixStream::connect(&data_socket).await.unwrap();
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let _writer = FrameWriter::new(write_half);

    // Give handler time to start select loop
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    // Inject a command via control plane
    let resp = client::rpc_call(
        reg.socket_path(),
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
    let dir = TestDir::new("dp-io");
    let (handles, _reg, data_socket, pty) = start_pty_session(&dir.sessions_dir(),"bidir").await;

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
    let dir = TestDir::new("dp-ping");
    let (handles, _reg, data_socket, pty) = start_pty_session(&dir.sessions_dir(),"pinger").await;

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
    let dir = TestDir::new("dp-caps");
    let (handles, reg, data_socket, pty) = start_pty_session(&dir.sessions_dir(),"capable").await;

    // Query status via control plane
    let resp = client::rpc_call(reg.socket_path(), "query.status", json!({}))
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
    let dir = TestDir::new("evt-ep");

    let (handle, reg) = start_session(&dir.sessions_dir(),"emitter", vec![]).await;

    // Emit an event
    let resp = client::rpc_call(
        reg.socket_path(),
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
        reg.socket_path(),
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
    let dir = TestDir::new("evt-topics");

    let (handle, reg) = start_session(&dir.sessions_dir(),"topicker", vec![]).await;

    // Emit events on different topics
    for topic in &["build.start", "build.done", "test.pass", "build.start"] {
        client::rpc_call(
            reg.socket_path(),
            "event.emit",
            json!({ "topic": topic, "payload": {} }),
        )
        .await
        .unwrap();
    }

    // Query topics
    let resp = client::rpc_call(reg.socket_path(), "event.topics", json!({}))
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
    let dir = TestDir::new("evt-filter");

    let (handle, reg) = start_session(&dir.sessions_dir(),"filterer", vec![]).await;

    // Emit mixed topics
    client::rpc_call(
        reg.socket_path(),
        "event.emit",
        json!({ "topic": "build.done", "payload": { "n": 1 } }),
    )
    .await
    .unwrap();
    client::rpc_call(
        reg.socket_path(),
        "event.emit",
        json!({ "topic": "test.fail", "payload": { "n": 2 } }),
    )
    .await
    .unwrap();
    client::rpc_call(
        reg.socket_path(),
        "event.emit",
        json!({ "topic": "build.done", "payload": { "n": 3 } }),
    )
    .await
    .unwrap();

    // Poll with topic filter
    let resp = client::rpc_call(
        reg.socket_path(),
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
    let dir = TestDir::new("kv-crud");

    let (handle, reg) = start_session(&dir.sessions_dir(),"kvstore", vec![]).await;

    // Set a key
    let resp = client::rpc_call(
        reg.socket_path(),
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
        reg.socket_path(),
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
        reg.socket_path(),
        "kv.set",
        json!({ "key": "config", "value": { "debug": true, "level": 3 } }),
    )
    .await
    .unwrap();

    // List all
    let resp = client::rpc_call(reg.socket_path(), "kv.list", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["count"], 2);
    let entries = result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);

    // Replace a key
    let resp = client::rpc_call(
        reg.socket_path(),
        "kv.set",
        json!({ "key": "color", "value": "red" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["replaced"], true);

    // Verify replacement
    let resp = client::rpc_call(
        reg.socket_path(),
        "kv.get",
        json!({ "key": "color" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["value"], "red");

    // Delete
    let resp = client::rpc_call(
        reg.socket_path(),
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
        reg.socket_path(),
        "kv.get",
        json!({ "key": "color" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["found"], false);

    // Delete non-existent
    let resp = client::rpc_call(
        reg.socket_path(),
        "kv.delete",
        json!({ "key": "nonexistent" }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["deleted"], false);

    // Final list — should have only "config"
    let resp = client::rpc_call(reg.socket_path(), "kv.list", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["count"], 1);

    handle.abort();
}

#[tokio::test]
async fn kv_get_nonexistent_returns_not_found() {
    let dir = TestDir::new("kv-notfound");

    let (handle, reg) = start_session(&dir.sessions_dir(),"kvempty", vec![]).await;

    let resp = client::rpc_call(
        reg.socket_path(),
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

#[tokio::test]
async fn kv_set_emits_kv_change_event() {
    let dir = TestDir::new("kv-watch-set");
    let (handle, reg) = start_session(&dir.sessions_dir(), "kvwatch", vec![]).await;

    // Prime one kv.set so the next event will have seq >= 1.
    client::rpc_call(
        reg.socket_path(),
        "kv.set",
        json!({ "key": "prime", "value": 0 }),
    )
    .await
    .unwrap();

    client::rpc_call(
        reg.socket_path(),
        "kv.set",
        json!({ "key": "theme", "value": "dark" }),
    )
    .await
    .unwrap();

    // since=0 replays events with seq > 0 (i.e., the second kv.set onward)
    let resp = client::rpc_call(
        reg.socket_path(),
        "event.subscribe",
        json!({ "topic": "kv.change", "since": 0, "timeout_ms": 500 }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["topic"], "kv.change");
    assert_eq!(events[0]["payload"]["key"], "theme");
    assert_eq!(events[0]["payload"]["value"], "dark");
    assert_eq!(events[0]["payload"]["op"], "set");
    assert_eq!(events[0]["payload"]["replaced"], false);

    handle.abort();
}

#[tokio::test]
async fn kv_delete_emits_kv_change_event() {
    let dir = TestDir::new("kv-watch-del");
    let (handle, reg) = start_session(&dir.sessions_dir(), "kvwatch", vec![]).await;

    client::rpc_call(
        reg.socket_path(),
        "kv.set",
        json!({ "key": "theme", "value": "dark" }),
    )
    .await
    .unwrap();
    client::rpc_call(
        reg.socket_path(),
        "kv.delete",
        json!({ "key": "theme" }),
    )
    .await
    .unwrap();

    // since=0 skips seq=0 (the kv.set), captures seq=1 (the kv.delete)
    let resp = client::rpc_call(
        reg.socket_path(),
        "event.subscribe",
        json!({ "topic": "kv.change", "since": 0, "timeout_ms": 500 }),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["payload"]["op"], "delete");
    assert_eq!(events[0]["payload"]["key"], "theme");
    assert_eq!(events[0]["payload"]["deleted"], true);
    assert!(events[0]["payload"]["value"].is_null());

    handle.abort();
}

// ─── Agent Message Protocol Integration Test ──────────────────────────

#[tokio::test]
async fn agent_request_response_via_events() {
    use termlink_protocol::events::{agent_topic, AgentRequest, AgentResponse, AgentStatus};

    let dir = TestDir::new("agent-msg");

    // Two sessions: orchestrator and worker
    let (h_orch, reg_orch) = start_session(&dir.sessions_dir(), "orchestrator", vec![]).await;
    let (h_work, reg_work) = start_session(&dir.sessions_dir(), "worker-1", vec![]).await;

    // 1. Orchestrator emits a request on the worker's event bus
    let request = AgentRequest {
        schema_version: "1.0".to_string(),
        request_id: "req-001".to_string(),
        from: "orchestrator".to_string(),
        to: "worker-1".to_string(),
        action: "task.run".to_string(),
        params: json!({"command": "cargo test"}),
        timeout_secs: Some(60),
    };
    let resp = client::rpc_call(
        reg_work.socket_path(),
        "event.emit",
        json!({"topic": agent_topic::REQUEST, "payload": request}),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["status"], "emitted");

    // 2. Worker emits a status update on the orchestrator's event bus
    let status = AgentStatus {
        schema_version: "1.0".to_string(),
        request_id: "req-001".to_string(),
        from: "worker-1".to_string(),
        phase: "running".to_string(),
        message: Some("Building crate...".to_string()),
        percent: Some(50),
    };
    let resp = client::rpc_call(
        reg_orch.socket_path(),
        "event.emit",
        json!({"topic": agent_topic::STATUS, "payload": status}),
    )
    .await
    .unwrap();
    client::unwrap_result(resp).unwrap();

    // 3. Worker emits a response on the orchestrator's event bus
    let response = AgentResponse {
        schema_version: "1.0".to_string(),
        request_id: "req-001".to_string(),
        from: "worker-1".to_string(),
        status: termlink_protocol::events::ResponseStatus::Ok,
        result: json!({"exit_code": 0, "tests_passed": 42}),
        error_message: None,
    };
    let resp = client::rpc_call(
        reg_orch.socket_path(),
        "event.emit",
        json!({"topic": agent_topic::RESPONSE, "payload": response}),
    )
    .await
    .unwrap();
    client::unwrap_result(resp).unwrap();

    // 4. Orchestrator polls its event bus — should see status + response
    let resp = client::rpc_call(reg_orch.socket_path(), "event.poll", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["count"], 2);
    let events = result["events"].as_array().unwrap();

    // First event: status update
    assert_eq!(events[0]["topic"], "agent.status");
    let status_payload: AgentStatus =
        serde_json::from_value(events[0]["payload"].clone()).unwrap();
    assert_eq!(status_payload.request_id, "req-001");
    assert_eq!(status_payload.phase, "running");
    assert_eq!(status_payload.percent, Some(50));

    // Second event: response
    assert_eq!(events[1]["topic"], "agent.response");
    let resp_payload: AgentResponse =
        serde_json::from_value(events[1]["payload"].clone()).unwrap();
    assert_eq!(resp_payload.request_id, "req-001");
    assert_eq!(resp_payload.status, termlink_protocol::events::ResponseStatus::Ok);
    assert_eq!(resp_payload.result["tests_passed"], 42);

    // 5. Worker polls its event bus — should see the request
    let resp = client::rpc_call(
        reg_work.socket_path(),
        "event.poll",
        json!({"topic": "agent.request"}),
    )
    .await
    .unwrap();
    let result = client::unwrap_result(resp).unwrap();
    assert_eq!(result["count"], 1);
    let events = result["events"].as_array().unwrap();
    let req_payload: AgentRequest =
        serde_json::from_value(events[0]["payload"].clone()).unwrap();
    assert_eq!(req_payload.request_id, "req-001");
    assert_eq!(req_payload.action, "task.run");
    assert_eq!(req_payload.params["command"], "cargo test");

    h_orch.abort();
    h_work.abort();
}

#[tokio::test]
async fn file_transfer_via_chunked_events() {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    use termlink_protocol::events::{
        file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
    };

    let dir = TestDir::new("file_xfer");

    let (h_sender, _reg_sender) = start_session(&dir.sessions_dir(), "sender", vec![]).await;
    let (h_receiver, reg_receiver) = start_session(&dir.sessions_dir(), "receiver", vec![]).await;

    // Test data: 150 bytes across 2 chunks of 100 bytes
    let test_data: Vec<u8> = (0..150u8).collect();
    let chunk_size = 100;
    let total_chunks = 2u32;
    let transfer_id = "xfer-test-001".to_string();

    let mut hasher = Sha256::new();
    hasher.update(&test_data);
    let expected_sha256 = format!("{:x}", hasher.finalize());

    // 1. Emit file.init on receiver's bus
    let init = FileInit {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        filename: "test.bin".to_string(),
        size: test_data.len() as u64,
        total_chunks,
        from: "sender".to_string(),
    };
    let resp = client::rpc_call(
        reg_receiver.socket_path(),
        "event.emit",
        json!({"topic": file_topic::INIT, "payload": init}),
    ).await.unwrap();
    client::unwrap_result(resp).unwrap();

    // 2. Emit chunks
    let encoder = base64::engine::general_purpose::STANDARD;
    for (i, chunk_data) in test_data.chunks(chunk_size).enumerate() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: encoder.encode(chunk_data),
        };
        let resp = client::rpc_call(
            reg_receiver.socket_path(),
            "event.emit",
            json!({"topic": file_topic::CHUNK, "payload": chunk}),
        ).await.unwrap();
        client::unwrap_result(resp).unwrap();
    }

    // 3. Emit file.complete
    let complete = FileComplete {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        sha256: expected_sha256.clone(),
    };
    let resp = client::rpc_call(
        reg_receiver.socket_path(),
        "event.emit",
        json!({"topic": file_topic::COMPLETE, "payload": complete}),
    ).await.unwrap();
    client::unwrap_result(resp).unwrap();

    // 4. Poll receiver's event bus — should see all file events
    let resp = client::rpc_call(reg_receiver.socket_path(), "event.poll", json!({}))
        .await
        .unwrap();
    let result = client::unwrap_result(resp).unwrap();

    let events = result["events"].as_array().unwrap();
    assert_eq!(events.len(), 4); // init + 2 chunks + complete

    // Verify event topics
    assert_eq!(events[0]["topic"], file_topic::INIT);
    assert_eq!(events[1]["topic"], file_topic::CHUNK);
    assert_eq!(events[2]["topic"], file_topic::CHUNK);
    assert_eq!(events[3]["topic"], file_topic::COMPLETE);

    // Verify init payload
    let init_payload: FileInit = serde_json::from_value(events[0]["payload"].clone()).unwrap();
    assert_eq!(init_payload.transfer_id, transfer_id);
    assert_eq!(init_payload.filename, "test.bin");
    assert_eq!(init_payload.total_chunks, 2);

    // Verify reassembly: decode chunks and compare
    let decoder = base64::engine::general_purpose::STANDARD;
    let mut reassembled = Vec::new();
    for i in 0..total_chunks {
        let chunk_payload: FileChunk =
            serde_json::from_value(events[(i + 1) as usize]["payload"].clone()).unwrap();
        assert_eq!(chunk_payload.index, i);
        let decoded = decoder.decode(&chunk_payload.data).unwrap();
        reassembled.extend_from_slice(&decoded);
    }
    assert_eq!(reassembled, test_data);

    // Verify SHA-256
    let complete_payload: FileComplete =
        serde_json::from_value(events[3]["payload"].clone()).unwrap();
    assert_eq!(complete_payload.sha256, expected_sha256);

    let mut verify_hasher = Sha256::new();
    verify_hasher.update(&reassembled);
    let actual_sha256 = format!("{:x}", verify_hasher.finalize());
    assert_eq!(actual_sha256, expected_sha256);

    h_sender.abort();
    h_receiver.abort();
}
