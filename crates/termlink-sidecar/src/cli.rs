//! One entry point for every sidecar verb (PH-2 row P1, OQ-a ruled A).
//!
//! Called by the thin `termlink-sidecar` binary with an empty prefix and by
//! `termlink sidecar ...` with the prefix `["sidecar"]`. The prefix is kept so
//! that a process this one starts again (the hook's detached finalizer) runs
//! the same verb through the same binary.
//!
//! - `start --config <file>`: the sidecar process (estate or production root).
//! - `hook post-tool-use|session-start|stop`: the Claude Code hook (always exits 0).
//! - `finalize --nonce <n>`: the detached hand-over finalizer a hook starts.
//! - `send`, `reply`, `status`: the agent-side client verbs (B14, see `client`).
//! - `admin held|allow|deny`: the operator's verbs on the admin socket (B15, OQ-2 E).

use std::sync::OnceLock;

use crate::{client, daemon, hook};

static PREFIX: OnceLock<Vec<String>> = OnceLock::new();

/// Arguments that come before a verb when this binary runs itself again:
/// empty for `termlink-sidecar`, `["sidecar"]` for `termlink`.
pub fn reexec_prefix() -> &'static [String] {
    PREFIX.get().map(Vec::as_slice).unwrap_or(&[])
}

fn usage(cmd: &str) -> i32 {
    eprintln!("usage: {cmd} start --config <file>");
    eprintln!("       {cmd} hook post-tool-use|session-start|stop");
    eprintln!("       {cmd} finalize --nonce <nonce>");
    eprintln!("       {cmd} send|reply|status ... (run with no arguments for their usage)");
    eprintln!("       {cmd} admin held|allow|deny ...");
    2
}

/// Run one verb; returns the process exit code. `prefix` is what precedes the
/// verb on this binary's command line (see the module comment).
pub fn run(prefix: &[&str], args: &[String]) -> i32 {
    let _ = PREFIX.set(prefix.iter().map(|s| s.to_string()).collect());
    let cmd = match prefix {
        [] => "termlink-sidecar".to_string(),
        p => format!("termlink {}", p.join(" ")),
    };
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["start", "--config", f] => daemon::main_start(std::path::Path::new(f)),
        // B2 form: the guard check alone.
        ["start"] => match crate::estate::guard_any_from_env() {
            Ok(mode) => {
                println!("{} ok: {} (no --config: guard check only)", mode.as_str(), mode.root().display());
                0
            }
            Err(e) => {
                eprintln!("{cmd}: refusing to start: {e}");
                2
            }
        },
        ["hook", event] => hook::main_hook(event),
        ["finalize", "--nonce", n] => hook::main_finalize(n),
        [verb @ ("send" | "reply" | "status" | "admin"), rest @ ..] => client::main_client(verb, rest),
        _ => usage(&cmd),
    }
}
