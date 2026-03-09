//! End-to-end CLI integration tests for the `termlink` binary.
//!
//! These tests spawn the actual `termlink` binary as child processes,
//! coordinating background sessions with foreground CLI commands.
//! Each test uses an isolated temp directory via TERMLINK_RUNTIME_DIR.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use assert_cmd::cargo::cargo_bin;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// RAII guard that kills a child process on drop.
/// Guarantees cleanup even on test panic.
struct ProcessGuard {
    child: Child,
    #[allow(dead_code)]
    name: String,
}

impl ProcessGuard {
    fn new(child: Child, name: &str) -> Self {
        Self {
            child,
            name: name.to_string(),
        }
    }

}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Create an isolated test directory with sessions subdirectory.
fn test_dir(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(format!("/tmp/tl-cli-{}-{}", n, name));
    let _ = std::fs::remove_dir_all(&dir);
    let sessions = dir.join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    dir
}

/// Wait until at least one .sock file appears in the sessions directory.
fn wait_for_socket(sessions_dir: &Path, timeout: Duration) -> Result<PathBuf, String> {
    let start = Instant::now();
    loop {
        if let Ok(entries) = std::fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "sock") {
                    return Ok(path);
                }
            }
        }
        if start.elapsed() > timeout {
            return Err(format!(
                "No socket appeared in {:?} within {:?}",
                sessions_dir, timeout
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Build a Command for the `termlink` binary with isolated runtime dir.
fn termlink_cmd(runtime_dir: &Path) -> Command {
    let mut cmd = Command::new(cargo_bin!("termlink"));
    cmd.env("TERMLINK_RUNTIME_DIR", runtime_dir);
    // Suppress tracing output in tests
    cmd.env("RUST_LOG", "");
    cmd
}

/// Start a `termlink register` process in the background.
fn start_register(runtime_dir: &Path, name: &str) -> ProcessGuard {
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
    let dir = test_dir("reg-list");
    let _guard = start_register(&dir, "testbox");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir)
        .args(["list"])
        .output()
        .expect("Failed to run termlink list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("testbox"), "Expected 'testbox' in list output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_ping_session() {
    let dir = test_dir("ping");
    let _guard = start_register(&dir, "pingable");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir)
        .args(["ping", "pingable"])
        .output()
        .expect("Failed to run termlink ping");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pingable"), "Expected 'pingable' in ping output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_status_query() {
    let dir = test_dir("status");
    let _guard = start_register(&dir, "statusbox");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir)
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
    let dir = test_dir("exec");
    let _guard = start_register(&dir, "worker");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    let output = termlink_cmd(&dir)
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
    let dir = test_dir("emit-events");
    let _guard = start_register(&dir, "eventer");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Emit two events
    let output = termlink_cmd(&dir)
        .args(["emit", "eventer", "build.start"])
        .output()
        .expect("Failed to run termlink emit");
    assert!(output.status.success(), "emit failed: {}",
        String::from_utf8_lossy(&output.stderr));

    let output = termlink_cmd(&dir)
        .args(["emit", "eventer", "build.done"])
        .output()
        .expect("Failed to run termlink emit");
    assert!(output.status.success(), "emit failed: {}",
        String::from_utf8_lossy(&output.stderr));

    // Default events (no --since) shows ALL events including seq=0
    let output = termlink_cmd(&dir)
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
    let dir = test_dir("topics");
    let _guard = start_register(&dir, "topicbox");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Emit events on different topics
    for topic in &["build.start", "test.pass", "deploy.done"] {
        let output = termlink_cmd(&dir)
            .args(["emit", "topicbox", topic])
            .output()
            .expect("Failed to run termlink emit");
        assert!(output.status.success());
    }

    // Query topics
    let output = termlink_cmd(&dir)
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
    let dir = test_dir("wait-emit");
    let _guard = start_register(&dir, "waitable");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Start wait in background thread (it blocks)
    let dir_clone = dir.clone();
    let wait_handle = std::thread::spawn(move || {
        termlink_cmd(&dir_clone)
            .args(["wait", "waitable", "--topic", "hello", "--timeout", "10"])
            .output()
            .expect("Failed to run termlink wait")
    });

    // Give wait time to connect and start polling
    std::thread::sleep(Duration::from_secs(1));

    // Emit the event (will be at seq=1, visible to since=1 polling)
    let emit_output = termlink_cmd(&dir)
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
    let dir = test_dir("wait-timeout");
    let _guard = start_register(&dir, "timeouty");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Wait with very short timeout — no event will arrive
    let output = termlink_cmd(&dir)
        .args(["wait", "timeouty", "--topic", "never", "--timeout", "1"])
        .output()
        .expect("Failed to run termlink wait");

    assert!(!output.status.success(), "Expected non-zero exit on timeout");
}

// ─── KV Store Tests ────────────────────────────────────────────────

#[test]
fn cli_kv_set_get_list_del() {
    let dir = test_dir("kv-crud");
    let _guard = start_register(&dir, "kvbox");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Set
    let output = termlink_cmd(&dir)
        .args(["kv", "kvbox", "set", "color", "blue"])
        .output()
        .expect("Failed to run kv set");
    assert!(output.status.success(), "kv set failed: {}",
        String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("color"), "Expected 'color' in set output: {}", stdout);

    // Get
    let output = termlink_cmd(&dir)
        .args(["kv", "kvbox", "get", "color"])
        .output()
        .expect("Failed to run kv get");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("blue"), "Expected 'blue' in get output: {}", stdout);

    // List
    let output = termlink_cmd(&dir)
        .args(["kv", "kvbox", "list"])
        .output()
        .expect("Failed to run kv list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("color"), "Expected 'color' in list output: {}", stdout);

    // Del
    let output = termlink_cmd(&dir)
        .args(["kv", "kvbox", "del", "color"])
        .output()
        .expect("Failed to run kv del");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deleted") || stdout.contains("color"),
        "Expected deletion confirmation: {}", stdout);

    // Get after delete — should fail
    let output = termlink_cmd(&dir)
        .args(["kv", "kvbox", "get", "color"])
        .output()
        .expect("Failed to run kv get after delete");
    assert!(!output.status.success(), "Expected non-zero exit for missing key");
}

#[test]
fn cli_kv_json_value() {
    let dir = test_dir("kv-json");
    let _guard = start_register(&dir, "jsonbox");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Set a JSON value
    let output = termlink_cmd(&dir)
        .args(["kv", "jsonbox", "set", "config", r#"{"debug":true}"#])
        .output()
        .expect("Failed to run kv set with JSON");
    assert!(output.status.success());

    // Get it back
    let output = termlink_cmd(&dir)
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
    let dir = test_dir("info");

    let output = termlink_cmd(&dir)
        .args(["info"])
        .output()
        .expect("Failed to run termlink info");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Info should show the runtime directory
    assert!(stdout.contains("Runtime") || stdout.contains("runtime") || stdout.contains(&dir.to_string_lossy().to_string()),
        "Expected runtime info in output: {}", stdout);
    assert!(output.status.success());
}

#[test]
fn cli_clean_with_no_sessions() {
    let dir = test_dir("clean-empty");

    let output = termlink_cmd(&dir)
        .args(["clean", "--dry-run"])
        .output()
        .expect("Failed to run termlink clean");

    assert!(output.status.success());
}

// ─── Multi-Session Tests ───────────────────────────────────────────

#[test]
fn cli_list_multiple_sessions() {
    let dir = test_dir("multi-list");
    let _g1 = start_register(&dir, "alpha");
    let _g2 = start_register(&dir, "beta");
    let _g3 = start_register(&dir, "gamma");

    // Wait for all three sockets
    let sessions_dir = dir.join("sessions");
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

    let output = termlink_cmd(&dir)
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
    let dir = test_dir("request-reply");
    let _guard = start_register(&dir, "worker");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Emit the reply event AFTER a delay (simulating specialist responding)
    let dir_clone = dir.clone();
    let _reply_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        termlink_cmd(&dir_clone)
            .args(["emit", "worker", "task.completed", "--payload", r#"{"status":"done","result":"ok"}"#])
            .output()
            .expect("Failed to emit reply event");
    });

    // Run request — it will wait for the reply
    let output = termlink_cmd(&dir)
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
    let dir = test_dir("request-timeout");
    let _guard = start_register(&dir, "silent");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();

    // Request with short timeout — no reply will come
    let output = termlink_cmd(&dir)
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
