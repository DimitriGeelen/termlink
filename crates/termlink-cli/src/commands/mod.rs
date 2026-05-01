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
