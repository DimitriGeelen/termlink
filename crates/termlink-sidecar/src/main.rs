//! Thin `termlink-sidecar` entry point.
//!
//! - `start --config <file>`: the sidecar process (refuses outside a marked estate).
//! - `hook post-tool-use|session-start|stop`: the Claude Code hook (always exits 0).
//! - `finalize --nonce <n>`: the detached hand-over finalizer a hook starts.
//! - `send`, `reply`, `status`: the agent-side client verbs (B14, see `client`).
//! - `admin held|allow|deny`: the operator's verbs on the admin socket (B15, OQ-2 E).

use termlink_sidecar::{client, daemon, hook};

fn usage() -> i32 {
    eprintln!("usage: termlink-sidecar start --config <file>");
    eprintln!("       termlink-sidecar hook post-tool-use|session-start|stop");
    eprintln!("       termlink-sidecar finalize --nonce <nonce>");
    eprintln!("       termlink-sidecar send|reply|status ... (run with no arguments for their usage)");
    2
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let code = match a.as_slice() {
        ["start", "--config", f] => daemon::main_start(std::path::Path::new(f)),
        // B2 form: the estate guard check alone.
        ["start"] => match termlink_sidecar::estate::guard_from_env() {
            Ok(root) => {
                println!("estate ok: {} (no --config: guard check only)", root.display());
                0
            }
            Err(e) => {
                eprintln!("termlink-sidecar: refusing to start: {e}");
                2
            }
        },
        ["hook", event] => hook::main_hook(event),
        ["finalize", "--nonce", n] => hook::main_finalize(n),
        [verb @ ("send" | "reply" | "status" | "admin"), rest @ ..] => client::main_client(verb, rest),
        _ => usage(),
    };
    std::process::exit(code);
}
