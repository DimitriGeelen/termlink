//! Thin `termlink-sidecar` binary entry point that hands its arguments to the shared cli module.
//! Thin `termlink-sidecar` entry point; every verb lives in [`termlink_sidecar::cli`].
//! The same verbs run as `termlink sidecar ...` (PH-2 row P1, OQ-a).

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(termlink_sidecar::cli::run(&[], &args));
}
