//! Tests that the sidecar runs as the `termlink sidecar ...` subcommand.
//! PH-2 row P1 (T-3365, OQ-a ruled A): the sidecar runs as `termlink sidecar ...`.
use std::process::Command;

fn termlink() -> Command {
    Command::new(env!("CARGO_BIN_EXE_termlink"))
}

#[test]
fn sidecar_without_a_verb_prints_its_usage_and_exits_2() {
    let out = termlink().arg("sidecar").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("usage: termlink sidecar start --config <file>"), "{err}");
    assert!(err.contains("termlink sidecar hook post-tool-use|session-start|stop"), "{err}");
}

#[test]
fn sidecar_start_outside_any_marked_root_is_refused_with_exit_2() {
    let d = std::env::temp_dir().join(format!("tl-p1-cli-{}", std::process::id()));
    std::fs::create_dir_all(d.join("store")).unwrap();
    let out = termlink()
        .args(["sidecar", "start"])
        .env("HOME", &d)
        .env("TERMLINK_RUNTIME_DIR", &d)
        .env("TERMLINK_SIDECAR_STORE", d.join("store"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&d);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("refusing to start"), "{err}");
    assert!(err.contains(".termlink-sidecar-root"), "{err}");
}

#[test]
fn sidecar_passes_flags_through_to_the_verb() {
    // `--nonce` must reach the sidecar, not be parsed by termlink's own CLI.
    let out = termlink()
        .args(["sidecar", "finalize", "--nonce", "abc"])
        .env_remove("TERMLINK_SIDECAR_SOCKET")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("unexpected argument"), "{err}");
}
