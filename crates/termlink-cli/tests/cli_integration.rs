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

/// Start a `termlink register --shell` process in the background (PTY-backed).
fn start_register_shell(runtime_dir: &std::path::Path, name: &str) -> ProcessGuard {
    let child = termlink_cmd(runtime_dir)
        .args(["register", "--name", name, "--shell"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn termlink register --shell");
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
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Expected valid JSON: {e}\nGot: {stdout}"));
    assert_eq!(parsed["ok"], true);
    let sessions = parsed["sessions"].as_array().expect("Expected sessions array");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["display_name"], "json-disc");
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

// ─── JSON Output Tests ──────────────────────────────────────────────

#[test]
fn cli_ping_json_output() {
    let dir = TestDir::new("ping-json");
    let _guard = start_register(&dir.path, "jsonping");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["ping", "jsonping", "--json"])
        .output()
        .expect("Failed to run termlink ping --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("ping --json should output valid JSON");

    assert_eq!(json["ok"], true);
    assert!(json["latency_ms"].is_number(), "Expected latency_ms number");
    assert!(json["display_name"].as_str().unwrap().contains("jsonping"));
}

#[test]
fn cli_clean_json_output() {
    let dir = TestDir::new("clean-json");

    // No sessions — clean should output JSON with count 0
    let output = termlink_cmd(&dir.path)
        .args(["clean", "--json"])
        .output()
        .expect("Failed to run termlink clean --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("clean --json should output valid JSON");

    assert_eq!(json["count"], 0);
    assert_eq!(json["dry_run"], false);
    assert!(json["sessions"].as_array().unwrap().is_empty());
}

#[test]
fn cli_tag_json_output() {
    let dir = TestDir::new("tag-json");
    let _guard = start_register(&dir.path, "tagbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["tag", "tagbox", "--add", "dev,test", "--json"])
        .output()
        .expect("Failed to run termlink tag --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("tag --json should output valid JSON");

    let tags = json["tags"].as_array().expect("Expected tags array");
    let tag_strs: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
    assert!(tag_strs.contains(&"dev"), "Expected 'dev' in tags: {:?}", tag_strs);
    assert!(tag_strs.contains(&"test"), "Expected 'test' in tags: {:?}", tag_strs);
}

#[test]
fn cli_tag_rename_session() {
    let dir = TestDir::new("tag-rename");
    let _guard = start_register(&dir.path, "old-name");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["tag", "old-name", "--name", "new-name", "--json"])
        .output()
        .expect("Failed to run termlink tag --name");

    assert!(output.status.success(), "tag --name should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("tag --name --json should output valid JSON");
    assert_eq!(json["display_name"], "new-name", "Name should be updated");
}

#[test]
fn cli_tag_set_roles() {
    let dir = TestDir::new("tag-roles");
    let _guard = start_register(&dir.path, "role-box");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["tag", "role-box", "--role", "orchestrator,worker", "--json"])
        .output()
        .expect("Failed to run termlink tag --role");

    assert!(output.status.success(), "tag --role should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("tag --role --json should output valid JSON");
    let roles = json["roles"].as_array().expect("Expected roles array");
    let role_strs: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
    assert!(role_strs.contains(&"orchestrator"), "Expected 'orchestrator' in roles: {:?}", role_strs);
    assert!(role_strs.contains(&"worker"), "Expected 'worker' in roles: {:?}", role_strs);
}

// ─── Shell Completions Regression Tests ─────────────────────────────

#[test]
fn cli_completions_bash() {
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["completions", "bash"])
        .output()
        .expect("Failed to run termlink completions bash");

    assert!(output.status.success(), "completions bash should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_termlink"), "Expected bash completion function");
}

#[test]
fn cli_completions_zsh() {
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["completions", "zsh"])
        .output()
        .expect("Failed to run termlink completions zsh");

    assert!(output.status.success(), "completions zsh should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_termlink"), "Expected zsh completion function");
}

#[test]
fn cli_completions_fish() {
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["completions", "fish"])
        .output()
        .expect("Failed to run termlink completions fish");

    assert!(output.status.success(), "completions fish should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("termlink"), "Expected fish completion for termlink");
}

// ─── More JSON Output Tests ─────────────────────────────────────────

#[test]
fn cli_exec_json_output() {
    let dir = TestDir::new("exec-json");
    let _guard = start_register(&dir.path, "execbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["exec", "execbox", "echo hello-json-test", "--json"])
        .output()
        .expect("Failed to run termlink exec --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("exec --json should output valid JSON");

    assert_eq!(json["exit_code"], 0);
    assert!(json["stdout"].as_str().unwrap().contains("hello-json-test"));
}

#[test]
fn cli_version_json_output() {
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["version", "--json"])
        .output()
        .expect("Failed to run termlink version --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("version --json should output valid JSON");

    assert!(json["version"].is_string(), "Expected version string");
    assert!(json["commit"].is_string(), "Expected commit string");
    assert!(json["target"].is_string(), "Expected target string");
}

#[test]
fn cli_hub_status_json_output() {
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["hub", "status", "--json"])
        .output()
        .expect("Failed to run termlink hub status --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("hub status --json should output valid JSON");

    assert!(json["status"].is_string(), "Expected status string");
}

// ─── Doctor Tests ───────────────────────────────────────────────────

#[test]
fn cli_doctor_text_output() {
    let dir = TestDir::new("doctor");

    let output = termlink_cmd(&dir.path)
        .args(["doctor"])
        .output()
        .expect("Failed to run termlink doctor");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TermLink Doctor"), "Expected 'TermLink Doctor' header: {}", stdout);
    assert!(stdout.contains("version"), "Expected version check: {}", stdout);
    assert!(stdout.contains("passed"), "Expected pass summary: {}", stdout);
}

#[test]
fn cli_doctor_json_output() {
    let dir = TestDir::new("doctor-json");

    let output = termlink_cmd(&dir.path)
        .args(["doctor", "--json"])
        .output()
        .expect("Failed to run termlink doctor --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("doctor --json should output valid JSON");

    assert!(json["checks"].is_array(), "Expected checks array");
    assert!(json["summary"]["pass"].is_number(), "Expected pass count");
    assert!(json["summary"]["warn"].is_number(), "Expected warn count");
    assert!(json["summary"]["fail"].is_number(), "Expected fail count");
}

// ─── Info Tests ─────────────────────────────────────────────────────

#[test]
fn cli_info_text_output() {
    let dir = TestDir::new("info-text");

    let output = termlink_cmd(&dir.path)
        .args(["info"])
        .output()
        .expect("Failed to run termlink info");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Runtime"), "Expected 'Runtime' in info output: {}", stdout);
    assert!(stdout.contains("Version"), "Expected 'Version' in info output: {}", stdout);
}

#[test]
fn cli_info_json_output() {
    let dir = TestDir::new("info-json");

    let output = termlink_cmd(&dir.path)
        .args(["info", "--json"])
        .output()
        .expect("Failed to run termlink info --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("info --json should output valid JSON");

    assert!(json["runtime_dir"].is_string(), "Expected runtime_dir string");
    assert!(json["sessions_dir"].is_string(), "Expected sessions_dir string");
    assert!(json["version"].is_string(), "Expected version string");
}

// ─── List Count Tests ───────────────────────────────────────────────

#[test]
fn cli_list_count() {
    let dir = TestDir::new("list-count");
    let _guard = start_register(&dir.path, "countbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["list", "--count"])
        .output()
        .expect("Failed to run termlink list --count");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "1", "Expected count of 1 session: {}", stdout);
}

#[test]
fn cli_list_count_empty() {
    let dir = TestDir::new("list-count-empty");

    let output = termlink_cmd(&dir.path)
        .args(["list", "--count"])
        .output()
        .expect("Failed to run termlink list --count");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "0", "Expected count of 0 sessions: {}", stdout);
}

// ─── Register --json Tests ──────────────────────────────────────────

#[test]
fn cli_register_json_output() {
    use std::io::BufRead;

    let dir = TestDir::new("reg-json");
    let mut child = termlink_cmd(&dir.path)
        .args(["register", "--name", "jsonbox", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn termlink register --json");

    // Read the first line of stdout (the JSON output)
    let stdout = child.stdout.take().unwrap();
    let mut reader = std::io::BufReader::new(stdout);
    let mut first_line = String::new();

    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            panic!("Timed out waiting for JSON output from register --json");
        }
        if reader.read_line(&mut first_line).unwrap() > 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _guard = ProcessGuard::new(child, "jsonbox");

    let parsed: serde_json::Value = serde_json::from_str(first_line.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from register --json: {e}\nGot: {first_line}"));

    assert_eq!(parsed["display_name"], "jsonbox");
    assert!(parsed["id"].is_string(), "Expected id field");
    assert!(parsed["socket_path"].is_string(), "Expected socket_path field");
    assert!(parsed["pid"].is_number(), "Expected pid field");
    assert_eq!(parsed["shell"], false);
}

// ─── Run --json Tests ───────────────────────────────────────────────

#[test]
fn cli_run_json_output() {
    let dir = TestDir::new("run-json");

    let output = termlink_cmd(&dir.path)
        .args(["run", "--json", "--", "echo", "hello world"])
        .output()
        .expect("Failed to run termlink run --json");

    assert!(output.status.success(), "run --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from run --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["exit_code"], 0);
    assert_eq!(parsed["stdout"].as_str().unwrap().trim(), "hello world");
    assert!(parsed["elapsed_ms"].is_number(), "Expected elapsed_ms field");
    assert!(parsed["session_id"].is_string(), "Expected session_id field");
    assert!(parsed["command"].is_string(), "Expected command field");
}

#[test]
fn cli_run_json_nonzero_exit() {
    let dir = TestDir::new("run-json-fail");

    let output = termlink_cmd(&dir.path)
        .args(["run", "--json", "--", "sh", "-c", "echo err >&2; exit 42"])
        .output()
        .expect("Failed to run termlink run --json");

    assert!(!output.status.success(), "Expected non-zero exit");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from run --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["exit_code"], 42);
    assert!(parsed["stderr"].as_str().unwrap().contains("err"), "Expected stderr to contain 'err'");
}

// ─── Spawn --json Tests ─────────────────────────────────────────────

#[test]
fn cli_spawn_json_output() {
    let dir = TestDir::new("spawn-json");

    let output = termlink_cmd(&dir.path)
        .args(["spawn", "--name", "spawntest", "--backend", "background", "--wait", "--wait-timeout", "5", "--json", "--", "sleep", "10"])
        .output()
        .expect("Failed to run termlink spawn --json");

    assert!(output.status.success(), "spawn --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from spawn --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["session_name"], "spawntest");
    assert_eq!(parsed["backend"], "background");
    assert_eq!(parsed["ready"], true);
    assert!(parsed["session_id"].is_string(), "Expected session_id field");
}

// ─── Kv --json Tests ────────────────────────────────────────────────

#[test]
fn cli_kv_json_set_get_list_del() {
    let dir = TestDir::new("kv-json");
    let _guard = start_register(&dir.path, "kvbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Set
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "--json", "set", "foo", "42"])
        .output()
        .expect("Failed to run kv set --json");
    assert!(output.status.success(), "kv set failed: {}", String::from_utf8_lossy(&output.stderr));
    let parsed: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .expect("Invalid JSON from kv set");
    assert_eq!(parsed["key"], "foo");

    // Get
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "--json", "get", "foo"])
        .output()
        .expect("Failed to run kv get --json");
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .expect("Invalid JSON from kv get");
    assert_eq!(parsed["found"], true);
    assert_eq!(parsed["value"], 42);

    // List
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "--json", "list"])
        .output()
        .expect("Failed to run kv list --json");
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .expect("Invalid JSON from kv list");
    assert_eq!(parsed["count"], 1);
    assert!(parsed["entries"].is_array());

    // Del
    let output = termlink_cmd(&dir.path)
        .args(["kv", "kvbox", "--json", "del", "foo"])
        .output()
        .expect("Failed to run kv del --json");
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .expect("Invalid JSON from kv del");
    assert_eq!(parsed["deleted"], true);
}

// ─── Send --json Tests ──────────────────────────────────────────────

#[test]
fn cli_send_json_output() {
    let dir = TestDir::new("send-json");
    let _guard = start_register(&dir.path, "sendbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["send", "sendbox", "termlink.ping", "--json"])
        .output()
        .expect("Failed to run termlink send --json");

    assert!(output.status.success(), "send --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from send --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["method"], "termlink.ping");
    assert!(parsed["result"].is_object(), "Expected result object");
}

// ─── Ping --timeout Tests ───────────────────────────────────────────

#[test]
fn cli_ping_with_timeout() {
    let dir = TestDir::new("ping-timeout");
    let _guard = start_register(&dir.path, "pingbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["ping", "pingbox", "--timeout", "10", "--json"])
        .output()
        .expect("Failed to run termlink ping --timeout");

    assert!(output.status.success(), "ping --timeout failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert_eq!(parsed["ok"], true);
    assert!(parsed["latency_ms"].is_number());
}

// ─── Inject --json Tests ────────────────────────────────────────────

#[test]
fn cli_inject_json_output() {
    let dir = TestDir::new("inject-json");
    let _guard = start_register_shell(&dir.path, "injectbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Give the PTY a moment to initialize
    std::thread::sleep(Duration::from_millis(200));

    let output = termlink_cmd(&dir.path)
        .args(["pty", "inject", "injectbox", "echo hello", "--enter", "--json"])
        .output()
        .expect("Failed to run termlink pty inject --json");

    assert!(output.status.success(), "inject --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from inject --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["ok"], true);
    assert!(parsed["bytes_injected"].is_number(), "Expected bytes_injected field");
    assert_eq!(parsed["target"], "injectbox");
}

// ─── Event emit --json Tests ────────────────────────────────────────

#[test]
fn cli_event_emit_json_output() {
    let dir = TestDir::new("emit-json");
    let _guard = start_register(&dir.path, "emitbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["event", "emit", "emitbox", "test.hello", "--payload", r#"{"msg":"hi"}"#, "--json"])
        .output()
        .expect("Failed to run event emit --json");

    assert!(output.status.success(), "event emit --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from event emit --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["topic"], "test.hello");
    assert!(parsed["seq"].is_number(), "Expected seq field");
}

#[test]
fn cli_event_poll_json_output() {
    let dir = TestDir::new("poll-json");
    let _guard = start_register(&dir.path, "pollbox");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Emit an event first
    let _ = termlink_cmd(&dir.path)
        .args(["event", "emit", "pollbox", "test.data", "--payload", r#"{"val":42}"#])
        .output()
        .expect("Failed to emit event");

    // Poll with --json
    let output = termlink_cmd(&dir.path)
        .args(["event", "poll", "pollbox", "--json"])
        .output()
        .expect("Failed to run event poll --json");

    assert!(output.status.success(), "event poll --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from event poll --json: {e}\nGot: {stdout}"));

    assert!(parsed["events"].is_array(), "Expected events array");
    assert!(parsed["next_seq"].is_number(), "Expected next_seq field");
}

// ─── Vendor status --json Tests ─────────────────────────────────────

#[test]
fn cli_vendor_status_json_not_vendored() {
    let dir = TestDir::new("vendor-json");

    let output = termlink_cmd(&dir.path)
        .args(["vendor", "status", "--target", &dir.path.display().to_string(), "--json"])
        .output()
        .expect("Failed to run vendor status --json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from vendor status --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["vendored"], false);
}

#[test]
fn cli_vendor_status_json_vendored() {
    let dir = TestDir::new("vendor-json-v");

    // Vendor first
    let output = termlink_cmd(&dir.path)
        .args(["vendor", "--target", &dir.path.display().to_string()])
        .output()
        .expect("Failed to run vendor");
    assert!(output.status.success(), "vendor failed: {}", String::from_utf8_lossy(&output.stderr));

    // Check status --json
    let output = termlink_cmd(&dir.path)
        .args(["vendor", "status", "--target", &dir.path.display().to_string(), "--json"])
        .output()
        .expect("Failed to run vendor status --json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON from vendor status --json: {e}\nGot: {stdout}"));

    assert_eq!(parsed["vendored"], true);
    assert!(parsed["version"].is_string(), "Expected version field");
    assert!(parsed["binary"].is_string(), "Expected binary field");
    assert!(parsed["size_bytes"].is_number(), "Expected size_bytes field");
    assert!(parsed["mcp_configured"].is_boolean(), "Expected mcp_configured field");
    assert!(parsed["gitignore_ok"].is_boolean(), "Expected gitignore_ok field");
}

// ─── Event Topics JSON Test ──────────────────────────────────────

#[test]
fn cli_event_topics_json_output() {
    let dir = TestDir::new("topics-json");
    let _guard = start_register(&dir.path, "topics-src");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Emit an event to create a topic
    let emit_out = termlink_cmd(&dir.path)
        .args(["event", "emit", "topics-src", "test.topic", "--payload", r#"{"x":1}"#])
        .output()
        .expect("Failed to emit event");
    assert!(emit_out.status.success());

    // Query topics with --json
    let output = termlink_cmd(&dir.path)
        .args(["event", "topics", "--json"])
        .output()
        .expect("Failed to run event topics --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert!(parsed["sessions"].is_array(), "Expected sessions array");
    assert!(parsed["total_topics"].is_number(), "Expected total_topics");
}

// ─── Event Wait JSON Test ────────────────────────────────────────

#[test]
fn cli_event_wait_json_output() {
    let dir = TestDir::new("wait-json");
    let _guard = start_register(&dir.path, "wait-src");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let dir_path = dir.path.clone();

    // Spawn a thread to emit the event after a delay.
    // 1500ms gives `event wait` enough time to start even under heavy parallel load.
    let emitter = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        let _ = termlink_cmd(&dir_path)
            .args(["event", "emit", "wait-src", "done.signal", "--payload", r#"{"result":"ok"}"#])
            .output();
    });

    let output = termlink_cmd(&dir.path)
        .args(["event", "wait", "wait-src", "--topic", "done.signal", "--timeout", "5", "--json"])
        .output()
        .expect("Failed to run event wait --json");

    emitter.join().unwrap();

    assert!(output.status.success(), "wait should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert_eq!(parsed["matched"], true);
    assert_eq!(parsed["topic"], "done.signal");
    assert!(parsed["seq"].is_number(), "Expected seq");
    assert!(parsed["payload"].is_object(), "Expected payload object");
}

// ─── PTY Output JSON Test ────────────────────────────────────────

#[test]
fn cli_pty_output_json() {
    let dir = TestDir::new("pty-out-json");
    let _guard = start_register_shell(&dir.path, "pty-out");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    // Give PTY time to initialize
    std::thread::sleep(Duration::from_millis(500));

    let output = termlink_cmd(&dir.path)
        .args(["pty", "output", "pty-out", "--json"])
        .output()
        .expect("Failed to run pty output --json");

    assert!(output.status.success(), "pty output --json should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert!(parsed["output"].is_string(), "Expected output string");
    assert!(parsed["bytes"].is_number(), "Expected bytes count");
    assert!(parsed["target"].is_string(), "Expected target");
}

// ─── Broadcast Tests ─────────────────────────────────────────────────

#[test]
fn cli_broadcast_no_hub_json() {
    let dir = TestDir::new("bcast-json");
    // Broadcast without a hub should fail with JSON error
    let output = termlink_cmd(&dir.path)
        .args(["event", "broadcast", "test-topic", "--json"])
        .output()
        .expect("Failed to run broadcast --json");

    assert!(!output.status.success(), "broadcast should fail without hub");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));
    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"].as_str().unwrap().contains("Hub is not running"));
}

// ─── Token Tests ─────────────────────────────────────────────────────

#[test]
fn cli_token_inspect_invalid_format() {
    let dir = TestDir::new("tok-inv");
    let output = termlink_cmd(&dir.path)
        .args(["token", "inspect", "not-a-valid-token"])
        .output()
        .expect("Failed to run token inspect");

    assert!(!output.status.success(), "token inspect should fail for invalid format");
}

#[test]
fn cli_token_inspect_invalid_json() {
    let dir = TestDir::new("tok-inv-json");
    let output = termlink_cmd(&dir.path)
        .args(["token", "inspect", "not-a-valid-token", "--json"])
        .output()
        .expect("Failed to run token inspect --json");

    assert!(!output.status.success(), "token inspect --json should fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Expected valid JSON: {e}\nGot: {stdout}"));
    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"].as_str().unwrap().contains("Invalid token format"));
}

// ─── Signal Tests ────────────────────────────────────────────────────

#[test]
fn cli_signal_not_found() {
    let dir = TestDir::new("sig-nf");
    // Signal to nonexistent session should fail
    let output = termlink_cmd(&dir.path)
        .args(["signal", "nonexistent", "TERM"])
        .output()
        .expect("Failed to run signal");

    assert!(!output.status.success(), "signal should fail for nonexistent session");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"), "Expected 'not found' in error: {stderr}");
}

// ─── List Filter Tests ──────────────────────────────────────────────

#[test]
fn cli_list_names_mode() {
    let dir = TestDir::new("list-names");
    let _guard = start_register(&dir.path, "named-box");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["list", "--names"])
        .output()
        .expect("Failed to run list --names");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "named-box", "Expected just the name: {stdout}");
}

#[test]
fn cli_list_ids_mode() {
    let dir = TestDir::new("list-ids");
    let _guard = start_register(&dir.path, "id-box");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["list", "--ids"])
        .output()
        .expect("Failed to run list --ids");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output just the session ID (UUID-like)
    assert!(!stdout.trim().is_empty(), "Expected session ID output");
    assert!(!stdout.contains("id-box"), "Expected only ID, not display name");
}

#[test]
fn cli_list_first_mode() {
    let dir = TestDir::new("list-first");
    let _guard1 = start_register(&dir.path, "first-1");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();
    let _guard2 = start_register(&dir.path, "first-2");
    // Give second session time to register
    std::thread::sleep(Duration::from_millis(500));

    let output = termlink_cmd(&dir.path)
        .args(["list", "--first"])
        .output()
        .expect("Failed to run list --first");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    // --first should return only one session
    assert_eq!(lines.len(), 1, "Expected exactly 1 line with --first, got: {lines:?}");
}

// ─── File Send Error Tests ───────────────────────────────────────────

#[test]
fn cli_file_send_nonexistent_file() {
    let dir = TestDir::new("fsend-err");
    let _guard = start_register(&dir.path, "file-target");
    wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["file", "send", "file-target", "/tmp/termlink-nonexistent-test-file.xyz", "--json"])
        .output()
        .expect("Failed to run file send --json");

    assert!(!output.status.success(), "file send should fail for non-existent file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));
    assert_eq!(parsed["ok"], false);
}

// ─── Dispatch Error Tests ────────────────────────────────────────────

#[test]
fn cli_dispatch_no_hub_json() {
    let dir = TestDir::new("disp-err");
    let output = termlink_cmd(&dir.path)
        .args(["dispatch", "--tag", "test-worker", "--", "echo", "hello", "--json"])
        .output()
        .expect("Failed to run dispatch --json");

    // Should fail because no hub is running
    assert!(!output.status.success(), "dispatch should fail without a hub");
}

// ─── Info Check Mode ─────────────────────────────────────────────────

#[test]
fn cli_info_check_mode() {
    let dir = TestDir::new("info-check");
    let output = termlink_cmd(&dir.path)
        .args(["info", "--check"])
        .output()
        .expect("Failed to run info --check");

    // --check exits 1 when hub is not running (expected in test environment)
    assert!(!output.status.success(), "info --check should fail when hub not running");
}

#[test]
fn cli_info_short_mode() {
    let dir = TestDir::new("info-short");
    let output = termlink_cmd(&dir.path)
        .args(["info", "--short"])
        .output()
        .expect("Failed to run info --short");

    assert!(output.status.success(), "info --short should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // --short should output just the runtime dir path
    assert!(!stdout.is_empty(), "info --short should produce output");
}

// ─── Hub Status Check Mode ──────────────────────────────────────────

#[test]
fn cli_hub_status_check_not_running() {
    let dir = TestDir::new("hub-chk");
    let output = termlink_cmd(&dir.path)
        .args(["hub", "status", "--check"])
        .output()
        .expect("Failed to run hub status --check");

    // --check exits 1 when hub is not running
    assert!(!output.status.success(), "hub status --check should fail when hub not running");
}

// ─── Doctor Strict Mode ─────────────────────────────────────────────

#[test]
fn cli_doctor_strict_json() {
    let dir = TestDir::new("doc-strict");
    let output = termlink_cmd(&dir.path)
        .args(["doctor", "--strict", "--json"])
        .output()
        .expect("Failed to run doctor --strict --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));
    assert!(parsed["ok"].is_boolean(), "Expected ok field");
    assert!(parsed["checks"].is_array(), "Expected checks array");
}

// ─── Vendor JSON Output ─────────────────────────────────────────────

#[test]
fn cli_vendor_json_output() {
    let project = vendor_project("vendor-json-out");

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--json", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run termlink vendor --json");

    assert!(output.status.success(), "vendor --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["action"], "vendored");
    assert!(parsed["source"].is_string(), "Expected source field");
    assert!(parsed["binary"].is_string(), "Expected binary field");
    assert!(parsed["version"].is_string(), "Expected version field");
    assert!(parsed["size_bytes"].is_number(), "Expected size_bytes field");
    assert!(parsed["previous_version"].is_null(), "First vendor should have null previous_version");
}

#[test]
fn cli_vendor_json_update() {
    let project = vendor_project("vendor-json-upd");

    // First vendor
    Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .unwrap();

    // Second vendor with --json
    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--json", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to re-vendor --json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["action"], "updated");
    assert!(parsed["previous_version"].is_string(), "Re-vendor should have previous_version");
}

// ─── Vendor Status Check Mode ───────────────────────────────────────

#[test]
fn cli_vendor_status_check_not_vendored() {
    let project = vendor_project("vendor-chk-no");

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "status", "--check", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run vendor status --check");

    // --check should exit non-zero when not vendored
    assert!(!output.status.success(), "vendor status --check should fail when not vendored");
}

#[test]
fn cli_vendor_status_check_json_not_vendored() {
    let project = vendor_project("vendor-chk-jno");

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "status", "--check", "--json", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run vendor status --check --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert_eq!(parsed["vendored"], false);
    assert_eq!(parsed["needs_update"], true);
}

// ─── Vendor Preserves Existing Content ──────────────────────────────

#[test]
fn cli_vendor_preserves_existing_gitignore() {
    let project = vendor_project("vendor-gi-exist");

    // Create .gitignore with existing content
    std::fs::write(project.path().join(".gitignore"), "node_modules/\n*.log\n").unwrap();

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run vendor");
    assert!(output.status.success());

    let gi = std::fs::read_to_string(project.path().join(".gitignore")).unwrap();
    assert!(gi.contains("node_modules/"), "Existing .gitignore content should be preserved");
    assert!(gi.contains("*.log"), "Existing .gitignore content should be preserved");
    assert!(gi.contains(".termlink"), "Should add .termlink entry");
}

#[test]
fn cli_vendor_merges_existing_mcp_settings() {
    let project = vendor_project("vendor-mcp-merge");

    // Create existing .claude/settings.local.json with another MCP server
    let claude_dir = project.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join("settings.local.json"),
        r#"{"mcpServers": {"other-tool": {"command": "other-binary", "args": ["serve"]}}}"#,
    ).unwrap();

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run vendor");
    assert!(output.status.success());

    let settings_content = std::fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&settings_content)
        .expect("Settings should be valid JSON");

    // Both MCP servers should exist
    assert!(settings["mcpServers"]["termlink"].is_object(), "termlink MCP should be added");
    assert!(settings["mcpServers"]["other-tool"].is_object(), "existing MCP server should be preserved");
    assert_eq!(
        settings["mcpServers"]["other-tool"]["command"], "other-binary",
        "existing MCP config should be untouched"
    );
}

#[test]
fn cli_token_create_and_inspect_roundtrip() {
    let dir = TestDir::new("token-roundtrip");

    // Register with --token-secret --json
    let mut reg = termlink_cmd(&dir.path)
        .args(["register", "--name", "token-test", "--token-secret", "--json", "--quiet"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn register");

    // Wait for socket to appear
    let socket = wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5))
        .expect("Socket never appeared");
    // The session name is used by token create
    let _ = socket;

    // Give it a moment for JSON output
    std::thread::sleep(Duration::from_millis(200));

    // Create a token
    let token_output = termlink_cmd(&dir.path)
        .args(["token", "create", "token-test", "--scope", "execute", "--ttl", "3600", "--json"])
        .output()
        .expect("Failed to run token create");

    let _ = reg.kill();
    let _ = reg.wait();

    if !token_output.status.success() {
        let stderr = String::from_utf8_lossy(&token_output.stderr);
        panic!("token create failed: {}", stderr);
    }

    let token_json: serde_json::Value =
        serde_json::from_slice(&token_output.stdout).expect("token create output not valid JSON");
    assert_eq!(token_json["ok"], true);
    let token_str = token_json["token"].as_str().expect("Missing token field");
    assert!(token_str.contains('.'), "Token should be base64.signature format");
    assert_eq!(token_json["scope"], "execute");
    assert_eq!(token_json["ttl"], 3600);

    // Inspect the token
    let inspect_output = termlink_cmd(&dir.path)
        .args(["token", "inspect", token_str, "--json"])
        .output()
        .expect("Failed to run token inspect");

    assert!(inspect_output.status.success());
    let inspect_json: serde_json::Value =
        serde_json::from_slice(&inspect_output.stdout).expect("token inspect output not valid JSON");
    assert_eq!(inspect_json["ok"], true);
    let payload = &inspect_json["payload"];
    assert_eq!(payload["scope"], "execute");
    assert!(payload["expires_at"].as_u64().is_some(), "Should have expires_at");
    assert_eq!(inspect_json["expired"], false);
}

#[test]
fn cli_vendor_handles_corrupt_settings() {
    let project = vendor_project("vendor-corrupt");

    // Create corrupt .claude/settings.local.json
    let claude_dir = project.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(claude_dir.join("settings.local.json"), "not json {{{").unwrap();

    let output = Command::new(cargo::cargo_bin!("termlink"))
        .args(["vendor", "--target"])
        .arg(project.path())
        .output()
        .expect("Failed to run vendor");

    // Should succeed (binary still copied) but warn about settings
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WARN"), "Should warn about corrupt settings: {}", stdout);

    // Binary should still exist
    assert!(project.path().join(".termlink/bin/termlink").exists());
    // Corrupt settings should not be overwritten
    let settings_content = std::fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
    assert_eq!(settings_content, "not json {{{", "Corrupt settings should not be modified");
}

// ─── Agent Ask Error Path ───────────────────────────────────────────

#[test]
fn cli_agent_ask_no_target() {
    let dir = TestDir::new("agent-ask-err");
    let output = termlink_cmd(&dir.path)
        .args([
            "agent", "ask", "nonexistent-session",
            "--action", "query.status",
            "--timeout", "2",
            "--json",
        ])
        .output()
        .expect("Failed to run agent ask");

    // Should fail because target session doesn't exist
    assert!(!output.status.success(), "agent ask should fail when target doesn't exist");
}

// ─── Push Nonexistent Source File ───────────────────────────────────

#[test]
fn cli_push_nonexistent_source() {
    let dir = TestDir::new("push-err");
    let output = termlink_cmd(&dir.path)
        .args([
            "push", "localhost:9999", "any-session",
            "/tmp/this-file-definitely-does-not-exist-xyz.txt",
            "--timeout", "2",
            "--json",
        ])
        .output()
        .expect("Failed to run push");

    // Should fail because file doesn't exist or hub is unreachable
    assert!(!output.status.success(), "push should fail with nonexistent file");
}

// ─── PTY Resize Error Path ─────────────────────────────────────────

#[test]
fn cli_pty_resize_no_target() {
    let dir = TestDir::new("resize-err");
    let output = termlink_cmd(&dir.path)
        .args([
            "pty", "resize", "nonexistent-session",
            "--cols", "120", "--rows", "40",
            "--json",
        ])
        .output()
        .expect("Failed to run pty resize");

    // Should fail because target session doesn't exist
    assert!(!output.status.success(), "pty resize should fail when target doesn't exist");
}

// ─── List --no-header ───────────────────────────────────────────────

#[test]
fn cli_list_no_header() {
    let dir = TestDir::new("list-nohdr");
    let mut _reg = start_register(&dir.path, "nohdr-sess");
    let socket = wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5));
    assert!(socket.is_ok(), "Session should register");

    let output = termlink_cmd(&dir.path)
        .args(["list", "--no-header"])
        .output()
        .expect("Failed to run list --no-header");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // --no-header should suppress the table header and footer
    assert!(!stdout.contains("Sessions:"), "Should not have header with --no-header");
    // But should still show the session
    assert!(stdout.contains("nohdr-sess"), "Should show session name");
}

// ─── Status --short ─────────────────────────────────────────────────

#[test]
fn cli_status_short_with_session() {
    let dir = TestDir::new("status-short");
    let mut _reg = start_register(&dir.path, "short-sess");
    let socket = wait_for_socket(&dir.sessions_dir(), Duration::from_secs(5));
    assert!(socket.is_ok(), "Session should register");
    std::thread::sleep(Duration::from_millis(200));

    let output = termlink_cmd(&dir.path)
        .args(["status", "short-sess", "--short"])
        .output()
        .expect("Failed to run status --short");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // --short should output a compact one-line status
    assert!(stdout.contains("short-sess"), "Should contain session name: {}", stdout);
}

// ─── Hub Status Short Mode ──────────────────────────────────────────

#[test]
fn cli_hub_status_short_not_running() {
    let dir = TestDir::new("hub-short");
    let output = termlink_cmd(&dir.path)
        .args(["hub", "status", "--short"])
        .output()
        .expect("Failed to run hub status --short");

    assert!(output.status.success(), "hub status --short should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "not_running", "short output should be 'not_running'");
}

// ─── Hub Status Runtime Dir Isolation ───────────────────────────────

#[test]
fn cli_hub_status_isolated_does_not_discover_system_hub() {
    // When TERMLINK_RUNTIME_DIR is explicitly set (via TestDir), resolve_hub_paths()
    // must NOT fall back to /var/lib/termlink. This prevents tests from discovering
    // the real systemd-managed hub.
    let dir = TestDir::new("hub-iso");
    let output = termlink_cmd(&dir.path)
        .args(["hub", "status", "--json"])
        .output()
        .expect("Failed to run hub status --json");

    assert!(output.status.success(), "hub status --json should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    // With isolated dir and no hub started, status must be not_running
    assert_eq!(parsed["status"], "not_running",
        "Isolated runtime dir should not discover system hub");
    // Must not contain /var/lib/termlink paths
    assert!(!stdout.contains("/var/lib/termlink"),
        "Should not reference system runtime dir when TERMLINK_RUNTIME_DIR is set");
}

// ─── Hub Status Stale Pidfile ───────────────────────────────────────

#[test]
fn cli_hub_status_stale_pidfile() {
    let dir = TestDir::new("hub-stale");
    // Write a pidfile with a PID that definitely doesn't exist
    std::fs::write(dir.path.join("hub.pid"), "4000000").unwrap();

    let output = termlink_cmd(&dir.path)
        .args(["hub", "status", "--json"])
        .output()
        .expect("Failed to run hub status --json");

    assert!(output.status.success(), "hub status --json should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));

    assert_eq!(parsed["status"], "stale", "Should detect stale pidfile");
    assert_eq!(parsed["pid"], 4000000, "Should report the stale PID");
}

// ─── Hub Status Check + Short Combined ──────────────────────────────

#[test]
fn cli_hub_status_check_short_not_running() {
    let dir = TestDir::new("hub-chk-short");
    let output = termlink_cmd(&dir.path)
        .args(["hub", "status", "--check", "--short"])
        .output()
        .expect("Failed to run hub status --check --short");

    // --check should exit 1 when hub is not running
    assert!(!output.status.success(), "should fail with --check when hub not running");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "not_running", "short output should still be 'not_running'");
}

// ─── List --wait Timeout ────────────────────────────────────────────

#[test]
fn cli_list_wait_timeout() {
    let dir = TestDir::new("list-wait-to");
    let start = Instant::now();

    let output = termlink_cmd(&dir.path)
        .args(["list", "--wait", "--wait-timeout", "1", "--tag", "nonexistent-tag-xyz"])
        .output()
        .expect("Failed to run list --wait");

    let elapsed = start.elapsed();
    // Should fail after timeout since no sessions with that tag exist
    assert!(!output.status.success(), "list --wait should timeout and fail");
    // Should have waited approximately 1 second
    assert!(elapsed >= Duration::from_millis(800), "Should wait near timeout: {:?}", elapsed);
    assert!(elapsed < Duration::from_secs(10), "Should not wait too long: {:?}", elapsed);
}

// ─── TOFU Commands ─────────────────────────────────────────────────

#[test]
fn cli_tofu_list_empty() {
    let dir = TestDir::new("tofu-empty");
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "list"])
        .output()
        .expect("Failed to run tofu list");

    assert!(output.status.success(), "tofu list should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("empty") || stdout.contains("0 entries") || stdout.contains("No trusted"),
        "Empty store should say so: {}", stdout);
}

#[test]
fn cli_tofu_list_json_empty() {
    let dir = TestDir::new("tofu-json");
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "list", "--json"])
        .output()
        .expect("Failed to run tofu list --json");

    assert!(output.status.success(), "tofu list --json should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("Invalid JSON: {e}\nGot: {stdout}"));
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["count"], 0);
    assert!(parsed["entries"].is_array());
}

#[test]
fn cli_tofu_clear_all_empty() {
    let dir = TestDir::new("tofu-clr");
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "clear", "--all"])
        .output()
        .expect("Failed to run tofu clear --all");

    assert!(output.status.success(), "tofu clear --all should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cleared 0"), "Should report 0 cleared: {}", stdout);
}

#[test]
fn cli_tofu_clear_specific_nonexistent() {
    let dir = TestDir::new("tofu-clr-ne");
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "clear", "nonexistent:9100"])
        .output()
        .expect("Failed to run tofu clear");

    assert!(output.status.success(), "tofu clear should succeed even if not found");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No TOFU entry found"), "Should say not found: {}", stdout);
}

#[test]
fn cli_tofu_roundtrip() {
    // Create a known_hubs file, list it, clear one entry, verify
    let dir = TestDir::new("tofu-rt");
    let termlink_dir = dir.path.join(".termlink");
    std::fs::create_dir_all(&termlink_dir).unwrap();
    std::fs::write(
        termlink_dir.join("known_hubs"),
        "# TermLink known hubs (TOFU)\n# host:port fingerprint first_seen last_seen\n\
         test1:9100 sha256:aaa 2026-01-01T00:00:00Z 2026-01-02T00:00:00Z\n\
         test2:9200 sha256:bbb 2026-01-01T00:00:00Z 2026-01-02T00:00:00Z\n",
    ).unwrap();

    // List should show 2 entries
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "list", "--json"])
        .output()
        .expect("Failed to run tofu list");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed["count"], 2);

    // Clear one
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "clear", "test1:9100"])
        .output()
        .expect("Failed to run tofu clear");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Removed"), "Should confirm removal: {}", stdout);

    // List again — should be 1
    let output = termlink_cmd(&dir.path)
        .env("HOME", dir.path.as_os_str())
        .args(["tofu", "list", "--json"])
        .output()
        .expect("Failed to run tofu list after clear");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed["count"], 1);
    assert_eq!(parsed["entries"][0]["host"], "test2:9200");
}
