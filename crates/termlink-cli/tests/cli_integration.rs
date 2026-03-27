//! End-to-end CLI integration tests for the `termlink` binary.
//!
//! These tests spawn the actual `termlink` binary as child processes,
//! coordinating background sessions with foreground CLI commands.
//! Each test uses an isolated temp directory via TERMLINK_RUNTIME_DIR.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use serde_json;

use termlink_test_utils::{wait_for_socket, ProcessGuard, TestDir};

/// Build a Command for the `termlink` binary with isolated runtime dir.
fn termlink_cmd(runtime_dir: &std::path::Path) -> Command {
    termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), runtime_dir)
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

// ─── Discovery Tests ─────────────────────────────────────────────

#[test]
fn cli_discover_by_role() {
    let dir = TestDir::new("discover-role");
    let _g1 = start_register(&dir.path, "coder-1");
    let _g2 = start_register(&dir.path, "tester-1");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Wait for both sockets
    let sessions_dir = dir.sessions_dir();
    let start = Instant::now();
    loop {
        let count = std::fs::read_dir(&sessions_dir)
            .map(|entries| entries.filter(|e| {
                e.as_ref().ok().is_some_and(|e| e.path().extension().is_some_and(|x| x == "sock"))
            }).count())
            .unwrap_or(0);
        if count >= 2 { break; }
        if start.elapsed() > Duration::from_secs(10) {
            panic!("Only {} of 2 sockets appeared", count);
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Discover all — should find both
    let output = termlink_cmd(&dir.path)
        .args(["discover"])
        .output()
        .expect("Failed to run termlink discover");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("coder-1"), "Missing coder-1: {}", stdout);
    assert!(stdout.contains("tester-1"), "Missing tester-1: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_discover_by_name() {
    let dir = TestDir::new("discover-name");
    let _g1 = start_register(&dir.path, "finder-alpha");
    let _g2 = start_register(&dir.path, "finder-beta");

    let sessions_dir = dir.sessions_dir();
    let start = Instant::now();
    loop {
        let count = std::fs::read_dir(&sessions_dir)
            .map(|entries| entries.filter(|e| {
                e.as_ref().ok().is_some_and(|e| e.path().extension().is_some_and(|x| x == "sock"))
            }).count())
            .unwrap_or(0);
        if count >= 2 { break; }
        if start.elapsed() > Duration::from_secs(10) {
            panic!("Only {} of 2 sockets appeared", count);
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Discover by name pattern
    let output = termlink_cmd(&dir.path)
        .args(["discover", "--name", "alpha"])
        .output()
        .expect("Failed to run termlink discover");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("finder-alpha"), "Missing finder-alpha: {}", stdout);
    assert!(!stdout.contains("finder-beta"), "Should not contain finder-beta: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_discover_json_output() {
    let dir = TestDir::new("discover-json");
    let _guard = start_register(&dir.path, "json-disc");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["discover", "--json"])
        .output()
        .expect("Failed to run termlink discover --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .expect(&format!("Expected JSON array: {}", stdout));
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["display_name"], "json-disc");
    assert!(output.status.success());
}

// ─── Register --self Tests ───────────────────────────────────────

#[test]
fn cli_register_self_creates_endpoint() {
    let dir = TestDir::new("reg-self");

    // Start register --self in background
    let child = termlink_cmd(&dir.path)
        .args(["register", "--self", "--name", "my-endpoint"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn register --self");
    let _guard = ProcessGuard::new(child, "my-endpoint");

    // Wait for socket
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Should be listable
    let output = termlink_cmd(&dir.path)
        .args(["list"])
        .output()
        .expect("Failed to run termlink list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("my-endpoint"), "Expected 'my-endpoint' in list: {}", stdout);

    // Should be pingable
    let output = termlink_cmd(&dir.path)
        .args(["ping", "my-endpoint"])
        .output()
        .expect("Failed to run termlink ping");
    assert!(output.status.success(), "ping failed: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn cli_register_self_supports_events() {
    let dir = TestDir::new("reg-self-ev");

    let child = termlink_cmd(&dir.path)
        .args(["register", "--self", "--name", "ev-endpoint"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn register --self");
    let _guard = ProcessGuard::new(child, "ev-endpoint");

    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Emit event to endpoint
    let output = termlink_cmd(&dir.path)
        .args(["emit", "ev-endpoint", "test.ping", "--payload", r#"{"from":"test"}"#])
        .output()
        .expect("Failed to emit");
    assert!(output.status.success(), "emit failed: {}",
        String::from_utf8_lossy(&output.stderr));

    // Poll events
    let output = termlink_cmd(&dir.path)
        .args(["events", "ev-endpoint"])
        .output()
        .expect("Failed to poll events");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.ping"), "Expected test.ping event: {}", stdout);
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

// ─── Vendor Tests ─────────────────────────────────────────────────

/// Helper: create a temp dir with git init for vendor tests.
fn vendor_project(name: &str) -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix(name).tempdir().unwrap();
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to git init");
    dir
}

#[test]
fn cli_vendor_fresh_project() {
    let project = vendor_project("vendor-fresh");

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run termlink vendor");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "vendor failed: {}", stdout);
    assert!(stdout.contains("Vendored"), "Expected 'Vendored' in output: {}", stdout);

    // Binary exists
    assert!(project.path().join(".termlink/bin/termlink").exists());
    // VERSION exists
    assert!(project.path().join(".termlink/VERSION").exists());
    // .gitignore created with .termlink entry
    let gi = std::fs::read_to_string(project.path().join(".gitignore")).unwrap();
    assert!(gi.contains(".termlink"), "Expected .termlink in .gitignore: {}", gi);
    // MCP config created
    let settings = std::fs::read_to_string(project.path().join(".claude/settings.local.json")).unwrap();
    assert!(settings.contains("termlink"), "Expected termlink in MCP settings: {}", settings);
}

#[test]
fn cli_vendor_idempotent() {
    let project = vendor_project("vendor-idem");

    // First vendor
    Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .unwrap();

    // Second vendor
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to re-vendor");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Updated"), "Expected 'Updated' on re-vendor: {}", stdout);

    // .gitignore should NOT have duplicate .termlink entries
    let gi = std::fs::read_to_string(project.path().join(".gitignore")).unwrap();
    let count = gi.matches(".termlink").count();
    assert_eq!(count, 1, "Expected exactly 1 .termlink entry, got {}: {}", count, gi);
}

#[test]
fn cli_vendor_status() {
    let project = vendor_project("vendor-stat");

    // Not vendored yet
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "status", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to check vendor status");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Not vendored"), "Expected 'Not vendored': {}", stdout);

    // Vendor it
    Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .unwrap();

    // Now status should show version
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "status", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to check vendor status");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Version:"), "Expected version in status: {}", stdout);
    assert!(stdout.contains("MCP:"), "Expected MCP status: {}", stdout);
    assert!(stdout.contains("Ignore:"), "Expected gitignore status: {}", stdout);
}

#[test]
fn cli_vendor_dry_run() {
    let project = vendor_project("vendor-dry");

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--dry-run", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run vendor --dry-run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would"), "Expected 'Would' in dry-run output: {}", stdout);

    // Nothing should be created
    assert!(!project.path().join(".termlink").exists(), "Vendor dir should not exist in dry-run");
}
