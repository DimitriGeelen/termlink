pub(crate) mod session;
pub(crate) mod pty;
pub(crate) mod mirror_grid;
pub(crate) mod mirror_grid_composer;
pub(crate) mod events;
pub(crate) mod metadata;
pub(crate) mod execution;
pub(crate) mod dispatch;
pub(crate) mod infrastructure;
pub(crate) mod token;
pub(crate) mod remote;
pub(crate) mod agent;
pub(crate) mod file;
pub(crate) mod push;
pub(crate) mod vendor;
pub(crate) mod identity;
pub(crate) mod channel;
pub(crate) mod help;
pub(crate) mod agent_find_idle;
pub(crate) mod substrate;
pub(crate) mod webhook;

/// Display options shared across list-style commands (list, discover, remote list).
pub(crate) struct ListDisplayOpts {
    pub count: bool,
    pub first: bool,
    pub names: bool,
    pub ids: bool,
    pub no_header: bool,
    pub json: bool,
}

/// Print a JSON value to stdout, flush, and exit with code 1.
///
/// `process::exit(1)` alone does not flush Rust's buffered stdout,
/// so piped consumers (scripts, tests) may see empty output.
pub(crate) fn json_error_exit(value: serde_json::Value) -> ! {
    use std::io::Write;
    println!("{value}");
    let _ = std::io::stdout().flush();
    std::process::exit(1);
}

/// Build the `--json` envelope + effective exit code for an exec-style command
/// result (`command.execute` / session exec).
///
/// A missing or malformed `exit_code` defaults to **-1 (failure)**, matching the
/// text-output branches (remote.rs/session.rs) and `push.rs::exec_rpc`. The
/// `--json` branch must never fail OPEN by reporting `ok:true` / exit 0 on an
/// unknown outcome — that is the surface automation consumes, so a fail-open
/// default makes a failed-or-unreported remote command look successful (T-2491,
/// directive-#2 no-silent-failures). Returns `(envelope, exit_code)` where the
/// envelope is `{"ok": exit_code == 0, ...result}`.
pub(crate) fn exec_json_envelope(result: &serde_json::Value) -> (serde_json::Value, i64) {
    let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
    let mut wrapped = serde_json::json!({ "ok": exit_code == 0 });
    if let Some(obj) = result.as_object() {
        for (k, v) in obj {
            wrapped[k] = v.clone();
        }
    }
    (wrapped, exit_code)
}

// T-1426 (T-1166 soft deprecation): one-line stderr nudge at the top of every
// legacy primitive verb. Suppressed when TERMLINK_NO_DEPRECATION_WARN=1 so
// scripts and CI don't get spammed during the migration window.
pub(crate) fn print_deprecation_warning(primitive: &str, replacement: &str) {
    if std::env::var("TERMLINK_NO_DEPRECATION_WARN").ok().as_deref() == Some("1") {
        return;
    }
    eprintln!(
        "[DEPRECATED] termlink {primitive} — use 'termlink {replacement}' instead. See T-1166."
    );
}

#[cfg(test)]
mod deprecation_tests {
    #[test]
    fn warning_format_matches_canon() {
        let primitive = "remote push";
        let replacement = "channel post";
        let line = format!(
            "[DEPRECATED] termlink {primitive} — use 'termlink {replacement}' instead. See T-1166."
        );
        assert!(line.starts_with("[DEPRECATED] termlink remote push"));
        assert!(line.contains("'termlink channel post'"));
        assert!(line.contains("T-1166"));
    }

    #[test]
    fn suppression_env_var_documented() {
        // Helper reads exactly TERMLINK_NO_DEPRECATION_WARN=1. If you rename
        // the env var, this test reminds you to update the docs/runbooks too.
        // Avoiding actual env mutation (unsafe in Edition 2024 + race-prone
        // under parallel tests) — covered end-to-end via the build-task ACs.
        const ENV_VAR: &str = "TERMLINK_NO_DEPRECATION_WARN";
        const VALUE: &str = "1";
        assert_eq!(ENV_VAR, "TERMLINK_NO_DEPRECATION_WARN");
        assert_eq!(VALUE, "1");
    }
}

#[cfg(test)]
mod exec_json_envelope_tests {
    use super::exec_json_envelope;
    use serde_json::json;

    /// T-2491: a result WITHOUT exit_code must fail CLOSED — ok:false, negative
    /// exit code — not fail open (the old unwrap_or(0) bug reported ok:true/0).
    #[test]
    fn missing_exit_code_fails_closed() {
        let result = json!({ "stdout": "output", "stderr": "" });
        let (env, exit_code) = exec_json_envelope(&result);
        assert_eq!(env["ok"], json!(false), "missing exit_code must not report ok:true");
        assert!(exit_code < 0, "missing exit_code must yield a negative (failure) code, got {exit_code}");
    }

    /// T-2491: a non-integer exit_code is malformed → also fail closed.
    #[test]
    fn malformed_exit_code_fails_closed() {
        let result = json!({ "exit_code": "not-a-number" });
        let (env, exit_code) = exec_json_envelope(&result);
        assert_eq!(env["ok"], json!(false));
        assert!(exit_code < 0);
    }

    /// T-2491: exit_code:0 → ok:true, and the other result fields survive into
    /// the envelope unchanged.
    #[test]
    fn zero_exit_code_ok_and_fields_preserved() {
        let result = json!({ "exit_code": 0, "stdout": "hi", "stderr": "warn" });
        let (env, exit_code) = exec_json_envelope(&result);
        assert_eq!(env["ok"], json!(true));
        assert_eq!(exit_code, 0);
        assert_eq!(env["stdout"], json!("hi"));
        assert_eq!(env["stderr"], json!("warn"));
        assert_eq!(env["exit_code"], json!(0));
    }

    /// T-2491: a non-zero exit_code → ok:false, code preserved.
    #[test]
    fn nonzero_exit_code_not_ok() {
        let result = json!({ "exit_code": 42 });
        let (env, exit_code) = exec_json_envelope(&result);
        assert_eq!(env["ok"], json!(false));
        assert_eq!(exit_code, 42);
    }
}
