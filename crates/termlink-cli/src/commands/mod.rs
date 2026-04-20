pub(crate) mod session;
pub(crate) mod pty;
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
