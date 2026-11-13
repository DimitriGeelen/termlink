//! T-3760: `termlink register --hub ADDR --hub-token-file FILE` — a session run by a
//! non-root user, served on TCP and registered on a (root) hub over the network with
//! a registration claim (`termlink hub token --register-principal ...`).
//!
//! The process never needs the hub secret or root: it holds an observe token whose
//! claim lets it register names under one prefix and renew or remove only its own
//! records. It keeps a lease (heartbeat), re-registers when the hub has forgotten it
//! (hub restart), and deregisters on exit.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::json;
use tokio::sync::RwLock;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_session::auth::{self, RegisterClaim};
use termlink_session::client;
use termlink_session::handler::SessionContext;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

/// Seconds between lease renewals. The hub drops a record after 300 s of silence.
pub(crate) const HEARTBEAT_SECS: u64 = 60;

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

/// The registration claim inside `token`. A token without one is refused here,
/// before anything registers (the hub would refuse it anyway).
pub(crate) fn claim_from_token(token: &str) -> Result<RegisterClaim> {
    let payload = auth::peek_token_payload(token).context("the hub token file does not hold a hub token")?;
    let claim = payload.reg.ok_or_else(|| {
        anyhow::anyhow!(
            "this hub token carries no registration claim; on the hub host mint one with \
             `termlink hub token --register-principal <user> --register-prefix <prefix> --out <file>`"
        )
    })?;
    claim.check().map_err(anyhow::Error::new)?;
    Ok(claim)
}

/// The name to register: `name` if given (it must be under the claim's prefix),
/// otherwise `<prefix><pid>`.
pub(crate) fn register_name(claim: &RegisterClaim, name: Option<&str>, pid: u32) -> Result<String> {
    let n = name.map(str::to_string).unwrap_or_else(|| format!("{}{pid}", claim.prefix));
    if !claim.permits_name(&n) {
        anyhow::bail!(
            "name '{n}' is not allowed by this token: it must start with '{}' and use only [A-Za-z0-9._-] (max 64)",
            claim.prefix
        );
    }
    Ok(n)
}

/// The local IP this host uses to reach `hub` (a UDP "connect" sends no packet).
/// The hub only accepts an endpoint on the caller's own address, so this is the
/// address to listen on and advertise.
fn local_ip_toward(hub: &str) -> Result<IpAddr> {
    let target: SocketAddr = hub.parse().with_context(|| format!("hub address '{hub}' must be IP:PORT"))?;
    let bind: SocketAddr = if target.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" }.parse().expect("literal");
    let s = std::net::UdpSocket::bind(bind)?;
    s.connect(target)?;
    Ok(s.local_addr()?.ip())
}

/// Where to listen: `--listen` if given (an unspecified IP is replaced by the
/// address toward the hub, since that is what the hub sees), else that address on
/// an ephemeral port.
pub(crate) fn listen_addr(listen: Option<&str>, toward_hub: impl FnOnce() -> Result<IpAddr>) -> Result<SocketAddr> {
    match listen {
        Some(l) => {
            let mut a: SocketAddr = l.parse().with_context(|| format!("--listen '{l}' must be IP:PORT"))?;
            if a.ip().is_unspecified() {
                a.set_ip(toward_hub()?);
            }
            Ok(a)
        }
        None => Ok(SocketAddr::new(toward_hub()?, 0)),
    }
}

/// What a heartbeat answer means for the lease.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Lease {
    Renewed,
    /// The hub no longer knows the record (restart or reaped): register again.
    Forgotten,
    /// Transport failure or other refusal: reconnect, then register again.
    Broken(String),
}

pub(crate) fn classify_heartbeat<E: std::fmt::Display>(resp: &Result<RpcResponse, E>) -> Lease {
    match resp {
        Ok(RpcResponse::Success(_)) => Lease::Renewed,
        Ok(RpcResponse::Error(e)) if e.error.code == control::error_code::SESSION_NOT_FOUND => Lease::Forgotten,
        Ok(RpcResponse::Error(e)) => Lease::Broken(format!("{} {}", e.error.code, e.error.message)),
        Err(e) => Lease::Broken(e.to_string()),
    }
}

async fn connect(o: &HubRegisterOpts) -> Result<client::Client> {
    super::remote::connect_remote_hub_with_token(&o.hub, None, None, Some(&o.token_file), "observe").await
}

async fn register_on(c: &mut client::Client, params: &serde_json::Value) -> Result<String> {
    match c.call("session.register_remote", json!("t3760-reg"), params.clone()).await {
        Ok(RpcResponse::Success(r)) => r.result["id"]
            .as_str()
            .map(str::to_string)
            .context("hub answered register_remote without an id"),
        Ok(RpcResponse::Error(e)) => anyhow::bail!("hub refused registration: {} {}", e.error.code, e.error.message),
        Err(e) => anyhow::bail!("registration call failed: {e}"),
    }
}

pub(crate) async fn cmd_register_on_hub(o: HubRegisterOpts) -> Result<()> {
    let token = super::remote::load_hub_token_file(&o.token_file, "observe")?;
    let claim = claim_from_token(&token)?;
    let name = register_name(&claim, o.name.as_deref(), std::process::id())?;
    let addr = listen_addr(o.listen.as_deref(), || local_ip_toward(&o.hub))?;

    // Fail before registering anywhere if the hub refuses us.
    let mut hub = connect(&o).await?;

    // The session itself, in this user's own runtime dir (same-uid callers use the
    // Unix socket); token auth is mandatory because it is also served on TCP.
    let config = SessionConfig { display_name: Some(name.clone()), roles: o.roles.clone(), tags: o.tags.clone(), capabilities: o.cap.clone() };
    let mut session = termlink_session::Session::register(config).await.context("failed to register the local session")?;
    let secret = auth::generate_secret();
    session.registration.token_secret = Some(secret.iter().map(|b| format!("{b:02x}")).collect());
    session.persist_registration().context("failed to persist the session registration")?;
    let tcp = tokio::net::TcpListener::bind(addr).await.with_context(|| format!("cannot listen on {addr}"))?;
    let local = tcp.local_addr()?;

    let session_id = session.id().clone();
    let json_path = termlink_session::registration::Registration::json_path(&termlink_session::discovery::sessions_dir(), &session_id);
    let (registration, unix_listener, _) = session.into_parts();
    let socket_path = registration.socket_path().to_path_buf();
    let shared = Arc::new(RwLock::new(SessionContext::new(registration).with_registration_path(json_path.clone())));

    let params = json!({
        "display_name": name, "host": local.ip().to_string(), "port": local.port(),
        "pid": std::process::id(), "roles": o.roles, "tags": o.tags, "capabilities": o.cap,
    });
    let cleanup = |why: &str| {
        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&json_path);
        tracing::info!(why, "T-3760: local registration removed");
    };
    let mut rid = match register_on(&mut hub, &params).await {
        Ok(r) => r,
        Err(e) => {
            cleanup("hub refused");
            return Err(e);
        }
    };
    if o.json {
        println!("{}", json!({"ok": true, "hub": o.hub, "remote_id": rid, "name": name, "owner": claim.principal,
            "endpoint": local.to_string(), "session_id": session_id.as_str()}));
    } else {
        println!("Registered on hub {} as {name} (record {rid}, owner {}), served on {local}", o.hub, claim.principal);
    }

    let unix_loop = tokio::spawn(server::run_accept_loop(unix_listener, shared.clone()));
    let mut tcp_loop = tokio::spawn(server::run_accept_loop_tcp(tcp, shared));
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_SECS));
    ticker.tick().await;
    let outcome: Result<()> = loop {
        tokio::select! {
            _ = ticker.tick() => {
                let r = hub.call("session.heartbeat", json!("t3760-hb"), json!({"id": rid})).await;
                match classify_heartbeat(&r) {
                    Lease::Renewed => {}
                    lease => {
                        eprintln!("hub lease {lease:?}; registering again");
                        if matches!(lease, Lease::Broken(_)) {
                            match connect(&o).await {
                                Ok(c) => hub = c,
                                Err(e) => { eprintln!("reconnect failed (retrying in {HEARTBEAT_SECS}s): {e:#}"); continue; }
                            }
                        }
                        match register_on(&mut hub, &params).await {
                            Ok(r) => { rid = r; eprintln!("re-registered as record {rid}"); }
                            Err(e) => eprintln!("re-registration failed (retrying in {HEARTBEAT_SECS}s): {e:#}"),
                        }
                    }
                }
            }
            r = &mut tcp_loop => break Err(anyhow::anyhow!("TCP listener stopped: {r:?}")),
            _ = tokio::signal::ctrl_c() => break Ok(()),
            _ = sigterm.recv() => break Ok(()),
            _ = sighup.recv() => break Ok(()),
        }
    };
    let _ = hub.call("session.deregister_remote", json!("t3760-dereg"), json!({"id": rid})).await;
    unix_loop.abort();
    tcp_loop.abort();
    cleanup("shutdown");
    if !o.json {
        println!("Deregistered {name} from {}.", o.hub);
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_is_required_and_name_must_sit_under_the_prefix() {
        let secret = auth::generate_secret();
        let plain = auth::create_token(&secret, auth::PermissionScope::Observe, "", 60);
        let err = claim_from_token(&plain.raw).unwrap_err().to_string();
        assert!(err.contains("no registration claim"), "{err}");

        let tok = auth::create_register_token(&secret, RegisterClaim::new("alice", "alice-").unwrap(), 60).unwrap();
        let c = claim_from_token(&tok.raw).unwrap();
        assert_eq!(register_name(&c, None, 42).unwrap(), "alice-42");
        assert_eq!(register_name(&c, Some("alice-review"), 1).unwrap(), "alice-review");
        assert!(register_name(&c, Some("bob-review"), 1).is_err());
    }

    #[test]
    fn listen_address_defaults_to_the_address_toward_the_hub() {
        let ip: IpAddr = "10.0.0.5".parse().unwrap();
        assert_eq!(listen_addr(None, || Ok(ip)).unwrap(), "10.0.0.5:0".parse().unwrap());
        assert_eq!(listen_addr(Some("0.0.0.0:4100"), || Ok(ip)).unwrap(), "10.0.0.5:4100".parse().unwrap());
        assert_eq!(listen_addr(Some("127.0.0.1:4100"), || Ok(ip)).unwrap(), "127.0.0.1:4100".parse().unwrap());
        assert!(listen_addr(Some("nonsense"), || Ok(ip)).is_err());
        assert_eq!(local_ip_toward("127.0.0.1:9100").unwrap(), "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn heartbeat_answers_map_to_lease_actions() {
        use termlink_protocol::jsonrpc::{ErrorResponse, Response};
        let ok: Result<RpcResponse, String> = Ok(Response::success(json!(1), json!({"ok": true})).into());
        assert_eq!(classify_heartbeat(&ok), Lease::Renewed);
        let gone: Result<RpcResponse, String> =
            Ok(ErrorResponse::new(json!(1), control::error_code::SESSION_NOT_FOUND, "x").into());
        assert_eq!(classify_heartbeat(&gone), Lease::Forgotten);
        let denied: Result<RpcResponse, String> = Ok(ErrorResponse::new(json!(1), control::error_code::AUTH_DENIED, "no").into());
        assert!(matches!(classify_heartbeat(&denied), Lease::Broken(_)));
        let io: Result<RpcResponse, String> = Err("reset".into());
        assert!(matches!(classify_heartbeat(&io), Lease::Broken(_)));
    }
}
