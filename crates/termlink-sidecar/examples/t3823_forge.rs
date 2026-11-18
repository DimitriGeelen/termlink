//! T-3823 (T-3704 r3, the receiver half live): post ONE forged arc-011 message to a live W1 inbox and report it.
//!
//! The forge is a genuine sidecar `Sender` holding a REAL rostered agent key (`--key-file`) that claims ANOTHER agent
//! name (`--claim`). With named rosters (`agent=hex`) the receiving sidecar binds that key to its true name and must
//! refuse the post (`AgentKeyMismatch`), with the hub's key-card check off or in `log`. The sender's store is a
//! throwaway test estate; nothing on the receiving side is written by this program.
//!
//! cargo run -p termlink-sidecar --example t3823_forge -- --key-file K --claim NAME --to AGENT --to-instance INST \
//!     --hub-socket /var/lib/termlink/hub.sock --hub-id ID
//! Prints one JSON line: {conv, msg_id, claim, to}.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

fn arg(args: &[String], name: &str) -> String {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (key, claim, to, to_inst, sock, hub_id) = (
        PathBuf::from(arg(&args, "--key-file")),
        arg(&args, "--claim"),
        arg(&args, "--to"),
        arg(&args, "--to-instance"),
        PathBuf::from(arg(&args, "--hub-socket")),
        arg(&args, "--hub-id"),
    );
    assert!(key.exists(), "key file {} does not exist (never created here)", key.display());
    let identity = termlink_session::agent_identity::Identity::load_or_create_from_file(&key).expect("key");
    let e = Estate::create_without_hub().expect("scratch estate");
    let store = Arc::new(Store::open(&e.store_root(), &claim).expect("scratch store"));
    let sender = Sender::new(
        SendConfig {
            agent: claim.clone(),
            instance: format!("{claim}.t3823-forge"),
            identity,
            hub: termlink_protocol::TransportAddr::Unix { path: sock },
            declared_hub_id: hub_id,
            queue_path: e.root().join("outbox.sqlite"),
            timeout: Duration::from_secs(10),
            pump_interval: None,
        },
        store,
    )
    .expect("sender");
    let conv = "t3823-forge".to_string();
    let sent = sender
        .send(&to, &to_inst, Some(&conv), "T-3823 live check: a forged post claiming another agent name; it must be refused", None)
        .expect("send");
    // the post goes out from the outbox; give it a moment, then report
    std::thread::sleep(Duration::from_secs(2));
    println!("{}", serde_json::json!({"conv": conv, "msg_id": sent.msg_id, "claim": claim, "to": to}));
}
