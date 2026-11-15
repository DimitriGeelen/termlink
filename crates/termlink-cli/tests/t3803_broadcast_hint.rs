//! T-3803: the deprecation hint for `event broadcast --targets` must name a command that exists.
//! Found 2026-11-15 by the independent reviewer on T-1426: it said `termlink event emit_to`,
//! which clap rejects (`unrecognized subcommand 'emit_to'`); the subcommand is `emit-to`.
//!
//! Hermetic: a fresh empty runtime dir, so there is no hub to reach and nothing is sent.

use std::process::Command;

fn termlink(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_termlink"))
        .args(args)
        .env("TERMLINK_RUNTIME_DIR", dir)
        .env("HOME", dir)
        .env_remove("TERMLINK_NO_DEPRECATION_WARN")
        .env_remove("TERMLINK_HUB_REGISTER_ADDR")
        .env_remove("TERMLINK_HUB_REGISTER_TOKEN_FILE")
        .output()
        .expect("run termlink")
}

#[test]
fn broadcast_with_targets_hint_names_a_real_subcommand() {
    let dir = std::env::temp_dir().join(format!("t3803-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let out = termlink(&dir, &["event", "broadcast", "t3803-topic", "--targets", "nobody", "--timeout", "1"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let line = stderr
        .lines()
        .find(|l| l.contains("[DEPRECATED] termlink event broadcast"))
        .unwrap_or_else(|| panic!("no deprecation line in stderr: {stderr}"));
    assert!(line.contains("'termlink event emit-to'"), "hint must name `event emit-to`: {line}");
    assert!(!line.contains("emit_to"), "hint names a subcommand that does not exist: {line}");

    // The named replacement really is a subcommand.
    let help = termlink(&dir, &["event", "emit-to", "--help"]);
    assert!(help.status.success(), "`termlink event emit-to --help` failed: {}", String::from_utf8_lossy(&help.stderr));

    let _ = std::fs::remove_dir_all(&dir);
}
