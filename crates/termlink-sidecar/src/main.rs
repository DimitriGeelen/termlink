//! Thin `termlink-sidecar` entry point. B2 only wires the C-1 estate guard:
//! the sidecar refuses to start outside a marked throwaway tree.

use termlink_sidecar::estate;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("start") => match estate::guard_from_env() {
            Ok(root) => {
                println!("estate ok: {}", root.display());
                // Later PH-1 tasks (B4+) start the store and API here.
            }
            Err(e) => {
                eprintln!("termlink-sidecar: refusing to start: {e}");
                std::process::exit(2);
            }
        },
        _ => {
            eprintln!("usage: termlink-sidecar start");
            std::process::exit(2);
        }
    }
}
