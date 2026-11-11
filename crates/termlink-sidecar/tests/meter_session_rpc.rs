//! B-27 E-15 (L4 half): the sidecar's S1 read against a real TermLink session
//! with a real PTY, over its socket, with a notify-scope token (AL-2, FCW-9).
//! Client keystrokes move S1; the doorbell and inject do not; an observe token
//! is refused, so S1 reads unknown. Controls: see the asserts.

use std::path::PathBuf;
use std::sync::Arc;
use termlink_session::auth::{self, PermissionScope};
use termlink_session::handler::SessionContext;
use termlink_session::identity::SessionId;
use termlink_session::input_meter::InputSource;
use termlink_session::lifecycle::SessionState;
use termlink_session::registration::{Registration, SessionConfig};
use termlink_sidecar::meter::{S1Source, SessionRpc};

#[test]
fn s1_read_over_the_session_socket_counts_client_only() {
    let dir = std::env::current_dir().unwrap().join("target").join(format!("t3687-rpc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let sock: PathBuf = dir.join("s.sock");
    let secret = auth::generate_secret();
    let mut reg = Registration::new(
        SessionId::generate(),
        SessionConfig { display_name: Some("b27".into()), capabilities: vec![], roles: vec![], tags: vec![] },
        sock.clone(),
    );
    reg.state = SessionState::Ready;
    reg.token_secret = Some(secret.iter().map(|b| format!("{b:02x}")).collect());

    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let pty = rt.block_on(async { Arc::new(termlink_session::pty::PtySession::spawn(Some("/bin/sh"), 4096).unwrap()) });
    let ctx = SessionContext::with_pty(reg, pty.clone());
    let shared = Arc::new(tokio::sync::RwLock::new(ctx));
    let listener = rt.block_on(async { tokio::net::UnixListener::bind(&sock).unwrap() });
    rt.spawn(termlink_session::server::run_accept_loop(listener, shared));

    let notify = auth::create_token(&secret, PermissionScope::Notify, "", 3600).raw;
    let observe = auth::create_token(&secret, PermissionScope::Observe, "", 3600).raw;
    let rpc = SessionRpc::new().unwrap();
    rpc.bind("cc-sess", &sock, Some(notify.clone()));

    let r0 = rpc.read("cc-sess").expect("notify token reads the counters");
    assert_eq!(r0.at("client"), None);
    assert_eq!(r0.boot_id, termlink_session::input_meter::boot_id());

    // Doorbell and inject through the session's own RPCs: not client.
    rt.block_on(async {
        for (method, params) in [
            ("command.notify", serde_json::json!({"ids": ["m-1"]})),
            ("command.inject", serde_json::json!({"keys": [{"type": "text", "value": "true"}]})),
        ] {
            let tok = auth::create_token(&secret, PermissionScope::Control, "", 3600).raw;
            let resp = termlink_session::client::rpc_call_with_token_timeout(&sock, Some(&tok), method, params, std::time::Duration::from_secs(5)).await.unwrap();
            termlink_session::client::unwrap_result(resp).unwrap();
        }
    });
    let r1 = rpc.read("cc-sess").unwrap();
    assert!(r1.at("notify").is_some() && r1.at("inject").is_some(), "{r1:?}");
    assert_eq!(r1.at("client"), None, "notify/inject must not move S1");

    // A client keystroke (the data plane's path).
    rt.block_on(async { pty.write_from(InputSource::Client, b"x").await.unwrap() });
    let r2 = rpc.read("cc-sess").unwrap();
    assert!(r2.at("client").is_some_and(|c| c >= r1.at("inject").unwrap()), "{r2:?}");

    // FCW-9: an observe token is refused -> the read fails -> S1 unknown.
    rpc.bind("cc-sess", &sock, Some(observe));
    assert!(rpc.read("cc-sess").is_err());
    assert!(rpc.read("unbound").is_err());

    rt.block_on(async { let _ = pty.write(b"exit\n").await; });
    rt.shutdown_background();
    let _ = std::fs::remove_dir_all(&dir);
}
