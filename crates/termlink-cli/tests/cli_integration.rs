//! End-to-end CLI integration tests for the `termlink` binary.
//!
//! These tests spawn the actual `termlink` binary as child processes,
//! coordinating background sessions with foreground CLI commands.
//! Each test uses an isolated temp directory via TERMLINK_RUNTIME_DIR.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;

use termlink_test_utils::{wait_for_socket, ProcessGuard, TestDir};

/// Build a Command for the `termlink` binary with isolated runtime dir.
fn termlink_cmd(runtime_dir: &std::path::Path) -> Command {
    termlink_test_utils::termlink_cmd(&cargo::cargo_bin!("termlink"), runtime_dir)
}

/// Start a `termlink register` process in the background.
fn start_register(runtime_dir: &std::path::Path, name: &str) -> ProcessGuard {
    let child = termlink_cmd(runtime_dir)
        .args(["register", "--name", name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn termlink register");
    ProcessGuard::new(child, name)
}

// ─── Registration & Lifecycle Tests ────────────────────────────────

#[test]
fn cli_register_and_list() {
    let dir = TestDir::new("reg-list");
    let _guard = start_register(&dir.path, "testbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["list"])
        .output()
        .expect("Failed to run termlink list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("testbox"), "Expected 'testbox' in list output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_ping_session() {
    let dir = TestDir::new("ping");
    let _guard = start_register(&dir.path, "pingable");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["ping", "pingable"])
        .output()
        .expect("Failed to run termlink ping");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pingable"), "Expected 'pingable' in ping output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_status_query() {
    let dir = TestDir::new("status");
    let _guard = start_register(&dir.path, "statusbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["status", "statusbox"])
        .output()
        .expect("Failed to run termlink status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("statusbox") || stdout.contains("ready"),
        "Expected session info in status output: {}", stdout);
    assert!(output.status.success());
}

// ─── Command Execution Tests ───────────────────────────────────────

#[test]
fn cli_exec_command() {
    let dir = TestDir::new("exec");
    let _guard = start_register(&dir.path, "worker");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["exec", "worker", "echo hello-from-test"])
        .output()
        .expect("Failed to run termlink exec");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello-from-test"),
        "Expected 'hello-from-test' in exec output: {}", stdout);
    assert!(output.status.success());
}

// ─── Event Tests ───────────────────────────────────────────────────

#[test]
fn cli_emit_and_events() {
    let dir = TestDir::new("emit-events");
    let _guard = start_register(&dir.path, "eventer");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Emit two events
    let output = termlink_cmd(&dir.path)
        .args(["emit", "eventer", "build.start"])
        .output()
        .expect("Failed to run termlink emit");
    assert!(output.status.success(), "emit failed: {}",
        String::from_utf8_lossy(&output.stderr));

    let output = termlink_cmd(&dir.path)
        .args(["emit", "eventer", "build.done"])
        .output()
        .expect("Failed to run termlink emit");
    assert!(output.status.success(), "emit failed: {}",
        String::from_utf8_lossy(&output.stderr));

    // Default events (no --since) shows ALL events including seq=0
    let output = termlink_cmd(&dir.path)
        .args(["events", "eventer"])
        .output()
        .expect("Failed to run termlink events");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build.start"),
        "Expected 'build.start' (seq=0) in events output: {}", stdout);
    assert!(stdout.contains("build.done"),
        "Expected 'build.done' in events output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_topics_command() {
    let dir = TestDir::new("topics");
    let _guard = start_register(&dir.path, "topicbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Emit events on different topics
    for topic in &["build.start", "test.pass", "deploy.done"] {
        let output = termlink_cmd(&dir.path)
            .args(["emit", "topicbox", topic])
            .output()
            .expect("Failed to run termlink emit");
        assert!(output.status.success());
    }

    // Query topics
    let output = termlink_cmd(&dir.path)
        .args(["topics", "topicbox"])
        .output()
        .expect("Failed to run termlink topics");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build.start"), "Missing build.start: {}", stdout);
    assert!(stdout.contains("test.pass"), "Missing test.pass: {}", stdout);
    assert!(stdout.contains("deploy.done"), "Missing deploy.done: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_wait_receives_emitted_event() {
    let dir = TestDir::new("wait-emit");
    let _guard = start_register(&dir.path, "waitable");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Start wait in background thread (it blocks)
    let dir_clone = dir.path.clone();
    let wait_handle = std::thread::spawn(move || {
        termlink_cmd(&dir_clone)
            .args(["wait", "waitable", "--topic", "hello", "--timeout", "10"])
            .output()
            .expect("Failed to run termlink wait")
    });

    // Give wait time to connect and start polling
    std::thread::sleep(Duration::from_secs(1));

    // Emit the event (will be at seq=1, visible to since=1 polling)
    let emit_output = termlink_cmd(&dir.path)
        .args(["emit", "waitable", "hello", "--payload", r#"{"msg":"world"}"#])
        .output()
        .expect("Failed to run termlink emit");
    assert!(emit_output.status.success(), "emit failed: {}",
        String::from_utf8_lossy(&emit_output.stderr));

    // Wait should complete successfully
    let wait_output = wait_handle.join().expect("Wait thread panicked");
    assert!(wait_output.status.success(),
        "wait failed: {}", String::from_utf8_lossy(&wait_output.stderr));
    let stdout = String::from_utf8_lossy(&wait_output.stdout);
    assert!(stdout.contains("world"),
        "Expected payload with 'world' in wait output: {}", stdout);
}

#[test]
fn cli_wait_timeout_exits_nonzero() {
    let dir = TestDir::new("wait-timeout");
    let _guard = start_register(&dir.path, "timeouty");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Wait with very short timeout — no event will arrive
    let output = termlink_cmd(&dir.path)
        .args(["wait", "timeouty", "--topic", "never", "--timeout", "1"])
        .output()
        .expect("Failed to run termlink wait");

    assert!(!output.status.success(), "Expected non-zero exit on timeout");
}

// ─── KV Store Tests ────────────────────────────────────────────────

#[test]
fn cli_kv_set_get_list_del() {
    let dir = TestDir::new("kv-crud");
    let _guard = start_register(&dir.path, "kvbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Set
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "set", "color", "blue"])
        .output()
        .expect("Failed to run kv set");
    assert!(output.status.success(), "kv set failed: {}",
        String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("color"), "Expected 'color' in set output: {}", stdout);

    // Get
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "get", "color"])
        .output()
        .expect("Failed to run kv get");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("blue"), "Expected 'blue' in get output: {}", stdout);

    // List
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "list"])
        .output()
        .expect("Failed to run kv list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("color"), "Expected 'color' in list output: {}", stdout);

    // Del
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "del", "color"])
        .output()
        .expect("Failed to run kv del");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deleted") || stdout.contains("color"),
        "Expected deletion confirmation: {}", stdout);

    // Get after delete — should fail
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "get", "color"])
        .output()
        .expect("Failed to run kv get after delete");
    assert!(!output.status.success(), "Expected non-zero exit for missing key");
}

#[test]
fn cli_kv_json_value() {
    let dir = TestDir::new("kv-json");
    let _guard = start_register(&dir.path, "jsonbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Set a JSON value
    let output = termlink_cmd(&dir.path)
        .args(["kv", "jsonbox", "set", "config", r#"{"debug":true}"#])
        .output()
        .expect("Failed to run kv set with JSON");
    assert!(output.status.success());

    // Get it back
    let output = termlink_cmd(&dir.path)
        .args(["kv", "jsonbox", "get", "config"])
        .output()
        .expect("Failed to run kv get");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("debug"), "Expected JSON value in output: {}", stdout);
}

// ─── Info & Clean Tests ────────────────────────────────────────────

#[test]
fn cli_info_shows_runtime() {
    let dir = TestDir::new("info");

    let output = termlink_cmd(&dir.path)
        .args(["info"])
        .output()
        .expect("Failed to run termlink info");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Info should show the runtime directory
    assert!(stdout.contains("Runtime") || stdout.contains("runtime") || stdout.contains(&dir.path.to_string_lossy().to_string()),
        "Expected runtime info in output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_clean_with_no_sessions() {
    let dir = TestDir::new("clean-empty");

    let output = termlink_cmd(&dir.path)
        .args(["clean", "--dry-run"])
        .output()
        .expect("Failed to run termlink clean");

    assert!(output.status.success());
}

// ─── Multi-Session Tests ───────────────────────────────────────────

#[test]
fn cli_list_multiple_sessions() {
    let dir = TestDir::new("multi-list");
    let _g1 = start_register(&dir.path, "alpha");
    let _g2 = start_register(&dir.path, "beta");
    let _g3 = start_register(&dir.path, "gamma");

    // Wait for all three sockets
    let sessions_dir = dir.sessions_dir();
    let start = Instant::now();
    loop {
        let count = std::fs::read_dir(&sessions_dir)
            .map(|entries| entries.filter(|e| {
                e.as_ref().ok().is_some_and(|e| e.path().extension().is_some_and(|x| x == "sock"))
            }).count())
            .unwrap_or(0);
        if count >= 3 { break; }
        if start.elapsed() > Duration::from_secs(10) {
            panic!("Only {} of 3 sockets appeared", count);
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let output = termlink_cmd(&dir.path)
        .args(["list"])
        .output()
        .expect("Failed to run termlink list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alpha"), "Missing alpha: {}", stdout);
    assert!(stdout.contains("beta"), "Missing beta: {}", stdout);
    assert!(stdout.contains("gamma"), "Missing gamma: {}", stdout);
    assert!(output.status.success());
}

// ─── Request-Reply Tests ──────────────────────────────────────────

#[test]
fn cli_request_reply_flow() {
    let dir = TestDir::new("request-reply");
    let _guard = start_register(&dir.path, "worker");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Emit the reply event AFTER a delay (simulating specialist responding)
    let dir_clone = dir.path.clone();
    let _reply_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        termlink_cmd(&dir_clone)
            .args(["emit", "worker", "task.completed", "--payload", r#"{"status":"done","result":"ok"}"#])
            .output()
            .expect("Failed to emit reply event");
    });

    // Run request — it will wait for the reply
    let output = termlink_cmd(&dir.path)
        .args([
            "request", "worker",
            "--topic", "task.delegate",
            "--payload", r#"{"action":"test"}"#,
            "--reply-topic", "task.completed",
            "--timeout", "10",
        ])
        .output()
        .expect("Failed to run termlink request");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Request sent"), "Expected 'Request sent' in output: {}", stdout);
    assert!(stdout.contains("Reply received"),
        "Expected 'Reply received' in output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_request_timeout() {
    let dir = TestDir::new("request-timeout");
    let _guard = start_register(&dir.path, "silent");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Request with short timeout — no reply will come
    let output = termlink_cmd(&dir.path)
        .args([
            "request", "silent",
            "--topic", "task.delegate",
            "--reply-topic", "task.completed",
            "--timeout", "1",
        ])
        .output()
        .expect("Failed to run termlink request");

    assert!(!output.status.success(), "Expected non-zero exit on timeout");
}
