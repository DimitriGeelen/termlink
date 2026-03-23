//! Interactive TTY integration tests for `termlink attach` and `termlink stream`.
//!
//! These tests use `rexpect` to spawn processes in pseudo-terminals,
//! enabling testing of interactive commands that require raw mode and PTY I/O.
//!
//! Tests are marked #[ignore] by default since they require PTY allocation
//! and can be sensitive to system load. Run with:
//!   cargo test -p termlink --test interactive_integration -- --ignored

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use assert_cmd::cargo;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(100); // offset from cli_integration

fn test_dir(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(format!("/tmp/tl-tty-{}-{}", n, name));
    let _ = std::fs::remove_dir_all(&dir);
    let sessions = dir.join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    dir
}

fn termlink_path() -> String {
    let p = cargo::cargo_bin!("termlink").to_string_lossy().to_string();
    assert!(std::path::Path::new(&p).exists(), "Binary not found: {}", p);
    p
}

/// Spawn a termlink command in a PTY with proper env vars.
fn spawn_termlink(dir: &std::path::Path, args: &str, timeout_ms: u64) -> rexpect::session::PtySession {
    let bin = termlink_path();
    let mut command = std::process::Command::new(&bin);
    command.env("TERMLINK_RUNTIME_DIR", dir);
    command.env("RUST_LOG", "");
    for arg in args.split_whitespace() {
        command.arg(arg);
    }
    rexpect::session::spawn_command(command, Some(timeout_ms))
        .unwrap_or_else(|e| panic!("Failed to spawn: {} {} — error: {}", bin, args, e))
}

/// Wait until at least one .sock file appears in the sessions directory.
fn wait_for_socket(sessions_dir: &std::path::Path, timeout: Duration) -> Result<PathBuf, String> {
    let start = std::time::Instant::now();
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
            return Err(format!("No socket in {:?} within {:?}", sessions_dir, timeout));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Wait for the data plane socket (.sock.data) to appear.
fn wait_for_data_socket(sessions_dir: &std::path::Path, timeout: Duration) -> Result<PathBuf, String> {
    let start = std::time::Instant::now();
    loop {
        if let Ok(entries) = std::fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().and_then(|n| n.to_str()).is_some_and(|name| name.ends_with(".sock.data")) {
                    return Ok(path);
                }
            }
        }
        if start.elapsed() > timeout {
            return Err(format!("No data socket in {:?} within {:?}", sessions_dir, timeout));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

// ─── Attach Tests ──────────────────────────────────────────────────

#[test]
#[ignore] // requires PTY allocation
fn attach_shows_output_and_detaches() {
    let dir = test_dir("attach");

    // Start register --shell in a PTY via rexpect
    let mut register = spawn_termlink(&dir, "register --name pty-test --shell", 10_000);

    // Wait for it to be ready
    register.exp_string("Listening for connections").expect("Register didn't start");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();
    std::thread::sleep(Duration::from_millis(500));

    // Start attach in a PTY
    let mut attach = spawn_termlink(&dir, "attach pty-test", 10_000);

    // Should see the attach banner
    attach.exp_string("Attached to").expect("Didn't see attach banner");

    // Type a command through attach to produce known output
    attach.send_line("echo ATTACH_MARKER_12345").expect("Failed to send command");

    // Should see the output
    attach.exp_string("ATTACH_MARKER_12345").expect("Didn't see marker in attach output");

    // Detach with Ctrl+]
    attach.send_control(']').expect("Failed to send Ctrl+]");

    // Should see detach message
    attach.exp_string("Detached").expect("Didn't see detach message");

    drop(attach);
    drop(register);
}

#[test]
#[ignore] // requires PTY allocation
fn attach_inject_and_see_output() {
    let dir = test_dir("attach-io");

    let mut register = spawn_termlink(&dir, "register --name io-test --shell", 15_000);
    register.exp_string("Listening for connections").expect("Register didn't start");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let mut attach = spawn_termlink(&dir, "attach io-test", 15_000);
    attach.exp_string("Attached to").expect("Didn't see attach banner");

    // Type a command through attach
    attach.send_line("echo INTERACTIVE_IO_TEST_789").expect("Failed to send command");
    attach.exp_string("INTERACTIVE_IO_TEST_789").expect("Didn't see command output");

    // Detach
    attach.send_control(']').expect("Failed to send Ctrl+]");
    attach.exp_string("Detached").expect("Didn't see detach message");

    drop(attach);
    drop(register);
}

// ─── Stream Tests ──────────────────────────────────────────────────

#[test]
#[ignore] // requires PTY allocation
fn stream_shows_output_and_detaches() {
    let dir = test_dir("stream");

    let mut register = spawn_termlink(&dir, "register --name stream-test --shell", 10_000);
    register.exp_string("Listening for connections").expect("Register didn't start");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();
    wait_for_data_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let mut stream = spawn_termlink(&dir, "stream stream-test", 10_000);
    stream.exp_string("Streaming").expect("Didn't see stream banner");

    // Type a command through the data plane stream
    stream.send_line("echo STREAM_MARKER_67890").expect("Failed to send command");
    stream.exp_string("STREAM_MARKER_67890").expect("Didn't see marker in stream output");

    stream.send_control(']').expect("Failed to send Ctrl+]");
    stream.exp_string("Detached").expect("Didn't see detach message");

    drop(stream);
    drop(register);
}

#[test]
#[ignore] // requires PTY allocation
fn stream_bidirectional_io() {
    let dir = test_dir("stream-io");

    let mut register = spawn_termlink(&dir, "register --name bidir-test --shell", 15_000);
    register.exp_string("Listening for connections").expect("Register didn't start");
    wait_for_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();
    wait_for_data_socket(&dir.join("sessions"), Duration::from_secs(5)).unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let mut stream = spawn_termlink(&dir, "stream bidir-test", 15_000);
    stream.exp_string("Streaming").expect("Didn't see stream banner");

    // Send a command through the data plane
    stream.send_line("echo BIDIR_STREAM_TEST_456").expect("Failed to send command");
    stream.exp_string("BIDIR_STREAM_TEST_456").expect("Didn't see command output");

    stream.send_control(']').expect("Failed to send Ctrl+]");
    stream.exp_string("Detached").expect("Didn't see detach message");

    drop(stream);
    drop(register);
}
