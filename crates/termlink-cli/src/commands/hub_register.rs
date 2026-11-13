//! T-3760: `termlink register --hub ADDR --hub-token-file FILE` — a session run by a
//! non-root user, served on TCP and registered on a (root) hub over the network with
//! a registration claim (`termlink hub token --register-principal ...`).
//!
//! The process never needs the hub secret or root: it holds an observe token whose
//! claim lets it register names under one prefix and renew or remove only its own
//! records. It keeps a lease (heartbeat), re-registers when the hub has forgotten it
//! (hub restart), and deregisters on exit.
//!
//! T-3781: the link machinery (claim, endpoint, TCP serving, lease) lives in
//! [`super::hub_link`] and is shared with a plain `register` run by a linked user.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::json;
use tokio::sync::RwLock;

use termlink_session::auth;
use termlink_session::handler::SessionContext;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

use super::hub_link::{register_name, PreparedLink};

pub(crate) struct HubRegisterOpts {
    pub hub: String,
    pub token_file: String,
    pub listen: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
    pub cap: Vec<String>,
    pub json: bool,
}

pub(crate) async fn cmd_register_on_hub(o: HubRegisterOpts) -> Result<()> {
    // Fail before registering anywhere if the token, the name or the hub refuses us.
    let prepared = PreparedLink::prepare(&o.hub, &o.token_file, o.listen.as_deref()).await?;
    let name = register_name(&prepared.claim, o.name.as_deref(), std::process::id())?;

    // The session itself, in this user's own runtime dir (same-uid callers use the
    // Unix socket); token auth is mandatory because it is also served on TCP.
    let config = SessionConfig { display_name: Some(name.clone()), roles: o.roles.clone(), tags: o.tags.clone(), capabilities: o.cap.clone() };
    let mut session = termlink_session::Session::register(config).await.context("failed to register the local session")?;
    let secret = auth::generate_secret();
    session.registration.token_secret = Some(secret.iter().map(|b| format!("{b:02x}")).collect());
    session.persist_registration().context("failed to persist the session registration")?;

    let session_id = session.id().clone();
    let json_path = termlink_session::registration::Registration::json_path(&termlink_session::discovery::sessions_dir(), &session_id);
    let (registration, unix_listener, _) = session.into_parts();
    let socket_path = registration.socket_path().to_path_buf();
    let shared = Arc::new(RwLock::new(SessionContext::new(registration).with_registration_path(json_path.clone())));
    let cleanup = |why: &str| {
        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&json_path);
        tracing::info!(why, "T-3760: local registration removed");
    };

    let mut link = match prepared.start(&name, &o.roles, &o.tags, &o.cap, shared.clone()).await {
        Ok(l) => l,
        Err(e) => {
            cleanup("hub refused");
            return Err(e);
        }
    };
    if o.json {
        println!("{}", json!({"ok": true, "hub": link.hub, "remote_id": link.remote_id, "name": name, "owner": link.owner,
            "endpoint": link.endpoint.to_string(), "session_id": session_id.as_str()}));
    } else {
        println!("Registered on hub {} as {name} (record {}, owner {}), served on {}", link.hub, link.remote_id, link.owner, link.endpoint);
    }

    let unix_loop = tokio::spawn(server::run_accept_loop_tcp_token_only(unix_listener, shared));
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;
    let outcome: Result<()> = tokio::select! {
        r = &mut link.tcp_loop => Err(anyhow::anyhow!("TCP listener stopped: {r:?}")),
        _ = tokio::signal::ctrl_c() => Ok(()),
        _ = sigterm.recv() => Ok(()),
        _ = sighup.recv() => Ok(()),
    };
    let hub = link.hub.clone();
    link.shutdown().await;
    unix_loop.abort();
    cleanup("shutdown");
    if !o.json {
        println!("Deregistered {name} from {hub}.");
    }
    outcome
}
