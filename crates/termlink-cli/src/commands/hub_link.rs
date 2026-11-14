//! T-3781: the "hub link" of a non-root user — a registration-claim token (T-3760)
//! plus the hub address — and everything that follows from having one:
//!
//! - which runtime dir this process uses ([`apply_runtime_dir_rule`]),
//! - the name a session carries on the hub ([`hub_side_name`]),
//! - serving a session on TCP and holding its hub record ([`PreparedLink`] / [`Linked`]),
//!   shared by `register --hub` (T-3760) and a plain `register` run by a linked user.
//!
//! Operator ruling A (T-3759): the worker runs as the requesting user and registers on the
//! main hub over the network with its own user-bound claim. Nothing here runs as another user.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use serde_json::json;
use tokio::sync::RwLock;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_session::auth::{self, RegisterClaim};
use termlink_session::client;
use termlink_session::handler::SessionContext;
use termlink_session::server;

/// Env override for the hub address of the link.
pub(crate) const ENV_ADDR: &str = "TERMLINK_HUB_REGISTER_ADDR";
/// Env override for the claim token file of the link.
pub(crate) const ENV_TOKEN: &str = "TERMLINK_HUB_REGISTER_TOKEN_FILE";
/// Default files under `$HOME/.termlink/` (files, because `fw` spawns through an
/// `env -i` allow-list that drops unknown variables).
pub(crate) const ADDR_FILE: &str = "hub-register.addr";
pub(crate) const TOKEN_FILE: &str = "hub-register.token";

/// Seconds between lease renewals. The hub drops a record after 300 s of silence.
pub(crate) const HEARTBEAT_SECS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HubLink {
    pub hub: String,
    pub token_file: PathBuf,
}

/// Resolve the link from explicit inputs. Env wins; both env values or neither. Then the
/// two default files under `home/.termlink`; both or neither. Pure apart from reading
/// the address file — unit-tested.
pub(crate) fn resolve_link_from(env_addr: Option<String>, env_token: Option<String>, home: Option<&Path>) -> Result<Option<HubLink>> {
    let env_addr = env_addr.filter(|s| !s.trim().is_empty());
    let env_token = env_token.filter(|s| !s.trim().is_empty());
    match (env_addr, env_token) {
        (Some(a), Some(t)) => return Ok(Some(HubLink { hub: a.trim().to_string(), token_file: PathBuf::from(t) })),
        (Some(_), None) => anyhow::bail!("{ENV_ADDR} is set but {ENV_TOKEN} is not; set both or neither"),
        (None, Some(_)) => anyhow::bail!("{ENV_TOKEN} is set but {ENV_ADDR} is not; set both or neither"),
        (None, None) => {}
    }
    let Some(home) = home else { return Ok(None) };
    let dir = home.join(".termlink");
    let (addr_f, tok_f) = (dir.join(ADDR_FILE), dir.join(TOKEN_FILE));
    match (addr_f.exists(), tok_f.exists()) {
        (false, false) => Ok(None),
        (true, false) => anyhow::bail!("hub link half-provisioned: {} exists but {} is missing", addr_f.display(), tok_f.display()),
        (false, true) => anyhow::bail!("hub link half-provisioned: {} exists but {} is missing", tok_f.display(), addr_f.display()),
        (true, true) => {
            let hub = std::fs::read_to_string(&addr_f).with_context(|| format!("cannot read {}", addr_f.display()))?;
            let hub = hub.trim().to_string();
            if hub.is_empty() {
                anyhow::bail!("{} is empty; it must hold the hub address IP:PORT", addr_f.display());
            }
            Ok(Some(HubLink { hub, token_file: tok_f }))
        }
    }
}

/// The link of this process (env, then `$HOME/.termlink`).
pub(crate) fn resolve_link() -> Result<Option<HubLink>> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve_link_from(std::env::var(ENV_ADDR).ok(), std::env::var(ENV_TOKEN).ok(), home.as_deref())
}

/// The effective uid, except that the test seam `TERMLINK_TEST_TREAT_AS_NON_ROOT=1`
/// reports a non-root uid, so the linked paths can be tested by a root test runner
/// (PL-213). It only ever makes a process behave as LESS privileged.
pub(crate) fn effective_uid() -> u32 {
    if std::env::var_os("TERMLINK_TEST_TREAT_AS_NON_ROOT").is_some_and(|v| v == "1") {
        return 65534;
    }
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}

/// True when this user may create files in `dir`. A dir that cannot be stat-ed for lack of
/// permission is not writable; a dir that does not exist yet counts as writable (register
/// creates it, and failing there is the existing, loud behaviour).
fn dir_writable(dir: &Path) -> bool {
    match std::fs::metadata(dir) {
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(_) => true,
        Ok(_) => {
            let Ok(c) = std::ffi::CString::new(dir.as_os_str().as_encoded_bytes()) else { return true };
            // SAFETY: c is a valid NUL-terminated path; access only reads it.
            unsafe { libc::access(c.as_ptr(), libc::W_OK | libc::X_OK) == 0 }
        }
    }
}

/// The runtime-dir rule (pure): drop an explicit `TERMLINK_RUNTIME_DIR` only for a non-root
/// caller that has a hub link and cannot write the dir. Such a caller reaches the hub over
/// TCP; the hub's own dir (root, 0700) is useless to it, and using it made `register` fail
/// while `fw` kept exporting it (T-3759).
pub(crate) fn should_drop_runtime_dir(euid: u32, env_dir: Option<&Path>, has_link: bool, writable: impl Fn(&Path) -> bool) -> bool {
    euid != 0 && has_link && env_dir.is_some_and(|d| !writable(d))
}

static DROPPED_RUNTIME_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Apply [`should_drop_runtime_dir`] to this process, once, at startup (before any task
/// reads the environment). Returns the dir that was dropped, if any.
pub(crate) fn apply_runtime_dir_rule() -> Option<PathBuf> {
    DROPPED_RUNTIME_DIR
        .get_or_init(|| {
            let env_dir = std::env::var_os("TERMLINK_RUNTIME_DIR").map(PathBuf::from);
            let has_link = matches!(resolve_link(), Ok(Some(_)));
            if !should_drop_runtime_dir(effective_uid(), env_dir.as_deref(), has_link, dir_writable) {
                return None;
            }
            // SAFETY: called first thing in main, before any spawned task reads the env.
            unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
            env_dir
        })
        .clone()
}

/// The runtime dir dropped by [`apply_runtime_dir_rule`], for messages.
pub(crate) fn dropped_runtime_dir() -> Option<PathBuf> {
    DROPPED_RUNTIME_DIR.get().cloned().flatten()
}

/// Whether this caller should use its hub link: non-root and one is provisioned.
pub(crate) fn link_for_this_user() -> Result<Option<HubLink>> {
    if effective_uid() == 0 {
        return Ok(None);
    }
    resolve_link()
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

/// Strict name for `register --hub` (T-3760): `name` if given (it must be under the
/// claim's prefix), otherwise `<prefix><pid>`.
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

/// The name a linked session carries on the hub: its local name when the claim permits
/// it, else `<prefix><local>`. Local names come from callers that know nothing of the
/// claim (`fw` names its workers), so the prefix is added rather than refused.
pub(crate) fn hub_side_name(claim: &RegisterClaim, local: &str) -> Result<String> {
    if claim.permits_name(local) {
        return Ok(local.to_string());
    }
    let n = format!("{}{local}", claim.prefix);
    if !claim.permits_name(&n) {
        anyhow::bail!(
            "session name '{local}' cannot be registered on the hub: '{n}' must use only [A-Za-z0-9._-] and be at most 64 characters"
        );
    }
    Ok(n)
}

/// The local IP this host uses to reach `hub` (a UDP "connect" sends no packet).
/// The hub only accepts an endpoint on the caller's own address, so this is the
/// address to listen on and advertise.
pub(crate) fn local_ip_toward(hub: &str) -> Result<IpAddr> {
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

/// T-3782: a forward token for the hub, minted from this session's own secret (the one
/// its TCP side validates). Execute scope, bound to the session id, at the token TTL
/// ceiling; the lease loop sends a fresh one with every heartbeat. `None` when the
/// session has no (valid) secret, so the record stays observe-only.
pub(crate) fn mint_forward_token(secret_hex: Option<&str>, session_id: &str) -> Option<String> {
    let hex = secret_hex?;
    if hex.len() != 64 {
        return None;
    }
    let mut secret: auth::TokenSecret = [0u8; 32];
    for (i, b) in secret.iter_mut().enumerate() {
        *b = u8::from_str_radix(hex.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(auth::create_token(&secret, auth::PermissionScope::Execute, session_id, auth::DEFAULT_MAX_TOKEN_TTL_SECS).raw)
}

/// Connect to the hub with the claim token (observe scope plus the claim).
pub(crate) async fn connect(hub: &str, token_file: &str) -> Result<client::Client> {
    super::remote::connect_remote_hub_with_token(hub, None, None, Some(token_file), "observe").await
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

/// Token, claim, endpoint and a live hub connection — everything that can fail before
/// a local session exists, so a refusal leaves nothing behind.
pub(crate) struct PreparedLink {
    pub hub: String,
    token_file: String,
    pub claim: RegisterClaim,
    addr: SocketAddr,
    client: client::Client,
}

impl PreparedLink {
    pub(crate) async fn prepare(hub: &str, token_file: &str, listen: Option<&str>) -> Result<PreparedLink> {
        let token = super::remote::load_hub_token_file(token_file, "observe")?;
        let claim = claim_from_token(&token)?;
        let addr = listen_addr(listen, || local_ip_toward(hub))?;
        let client = connect(hub, token_file).await?;
        Ok(PreparedLink { hub: hub.to_string(), token_file: token_file.to_string(), claim, addr, client })
    }

    /// Serve `shared` on TCP (the session must carry a token secret: TCP is token-only),
    /// register `name` on the hub, and keep the record leased until [`Linked::shutdown`].
    pub(crate) async fn start(
        self,
        name: &str,
        roles: &[String],
        tags: &[String],
        caps: &[String],
        shared: Arc<RwLock<SessionContext>>,
    ) -> Result<Linked> {
        let PreparedLink { hub, token_file, claim, addr, mut client } = self;
        let tcp = tokio::net::TcpListener::bind(addr).await.with_context(|| format!("cannot listen on {addr}"))?;
        let endpoint = tcp.local_addr()?;
        let (secret_hex, session_id) = {
            let ctx = shared.read().await;
            (ctx.registration.token_secret.clone(), ctx.registration.id.as_str().to_string())
        };
        let minter = move || mint_forward_token(secret_hex.as_deref(), &session_id);
        let mut params = json!({
            "display_name": name, "host": endpoint.ip().to_string(), "port": endpoint.port(),
            "pid": std::process::id(), "roles": roles, "tags": tags, "capabilities": caps,
        });
        if let Some(t) = minter() {
            params["forward_token"] = json!(t);
        }
        let rid = register_on(&mut client, &params).await?;
        let tcp_loop = tokio::spawn(server::run_accept_loop_tcp(tcp, shared));
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let lease = tokio::spawn(lease_loop(hub.clone(), token_file, client, params, rid.clone(), stop_rx, Box::new(minter)));
        Ok(Linked { hub, owner: claim.principal.clone(), name: name.to_string(), remote_id: rid, endpoint, stop: Some(stop_tx), lease: Some(lease), tcp_loop })
    }
}

/// Renew the lease every [`HEARTBEAT_SECS`]; re-register when the hub forgot the record,
/// reconnect first when the connection broke; deregister when told to stop.
async fn lease_loop(
    hub: String,
    token_file: String,
    mut client: client::Client,
    params: serde_json::Value,
    mut rid: String,
    mut stop: tokio::sync::oneshot::Receiver<()>,
    minter: Box<dyn Fn() -> Option<String> + Send>,
) {
    let mut params = params;
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_SECS));
    ticker.tick().await;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // T-3782: each beat carries a freshly minted forward token.
                let mut hb = json!({"id": rid});
                if let Some(t) = minter() {
                    hb["forward_token"] = json!(t);
                    params["forward_token"] = hb["forward_token"].clone();
                }
                let r = client.call("session.heartbeat", json!("t3760-hb"), hb).await;
                match classify_heartbeat(&r) {
                    Lease::Renewed => {}
                    lease => {
                        eprintln!("hub lease {lease:?}; registering again");
                        if matches!(lease, Lease::Broken(_)) {
                            match connect(&hub, &token_file).await {
                                Ok(c) => client = c,
                                Err(e) => { eprintln!("reconnect failed (retrying in {HEARTBEAT_SECS}s): {e:#}"); continue; }
                            }
                        }
                        match register_on(&mut client, &params).await {
                            Ok(r) => { rid = r; eprintln!("re-registered as record {rid}"); }
                            Err(e) => eprintln!("re-registration failed (retrying in {HEARTBEAT_SECS}s): {e:#}"),
                        }
                    }
                }
            }
            _ = &mut stop => break,
        }
    }
    let _ = client.call("session.deregister_remote", json!("t3760-dereg"), json!({"id": rid})).await;
}

/// A session served on TCP with a leased hub record.
pub(crate) struct Linked {
    pub hub: String,
    pub owner: String,
    pub name: String,
    pub remote_id: String,
    pub endpoint: SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    lease: Option<tokio::task::JoinHandle<()>>,
    pub tcp_loop: tokio::task::JoinHandle<Result<(), std::io::Error>>,
}

impl Linked {
    /// Deregister from the hub (bounded) and stop serving TCP.
    pub(crate) async fn shutdown(mut self) {
        if let Some(s) = self.stop.take() {
            let _ = s.send(());
        }
        if let Some(l) = self.lease.take() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), l).await;
        }
        self.tcp_loop.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn link_resolution_env_first_then_both_default_files() {
        assert_eq!(resolve_link_from(None, None, None).unwrap(), None);
        let l = resolve_link_from(Some("10.0.0.1:9100".into()), Some("/t".into()), None).unwrap().unwrap();
        assert_eq!(l, HubLink { hub: "10.0.0.1:9100".into(), token_file: "/t".into() });
        let e = resolve_link_from(Some("10.0.0.1:9100".into()), None, None).unwrap_err().to_string();
        assert!(e.contains(ENV_TOKEN), "{e}");
        let e = resolve_link_from(None, Some("/t".into()), None).unwrap_err().to_string();
        assert!(e.contains(ENV_ADDR), "{e}");

        let h = tmp();
        assert_eq!(resolve_link_from(None, None, Some(h.path())).unwrap(), None, "no files: no link");
        std::fs::create_dir_all(h.path().join(".termlink")).unwrap();
        std::fs::write(h.path().join(".termlink").join(ADDR_FILE), "127.0.0.1:9100\n").unwrap();
        let e = resolve_link_from(None, None, Some(h.path())).unwrap_err().to_string();
        assert!(e.contains(TOKEN_FILE) && e.contains("missing"), "{e}");
        std::fs::write(h.path().join(".termlink").join(TOKEN_FILE), "x").unwrap();
        let l = resolve_link_from(None, None, Some(h.path())).unwrap().unwrap();
        assert_eq!(l.hub, "127.0.0.1:9100");
        assert_eq!(l.token_file, h.path().join(".termlink").join(TOKEN_FILE));
        std::fs::remove_file(h.path().join(".termlink").join(ADDR_FILE)).unwrap();
        let e = resolve_link_from(None, None, Some(h.path())).unwrap_err().to_string();
        assert!(e.contains(ADDR_FILE) && e.contains("missing"), "{e}");
        // an empty address file is an error; env beats the files
        std::fs::write(h.path().join(".termlink").join(ADDR_FILE), "  ").unwrap();
        assert!(resolve_link_from(None, None, Some(h.path())).is_err(), "empty addr file");
        let l = resolve_link_from(Some("10.9.9.9:1".into()), Some("/e".into()), Some(h.path())).unwrap().unwrap();
        assert_eq!(l.hub, "10.9.9.9:1");
    }

    #[test]
    fn runtime_dir_dropped_only_for_linked_non_root_and_unwritable() {
        let d = Path::new("/var/lib/termlink");
        let no = |_: &Path| false;
        let yes = |_: &Path| true;
        assert!(should_drop_runtime_dir(1000, Some(d), true, no));
        assert!(!should_drop_runtime_dir(0, Some(d), true, no), "root keeps it");
        assert!(!should_drop_runtime_dir(1000, Some(d), false, no), "no link keeps it");
        assert!(!should_drop_runtime_dir(1000, Some(d), true, yes), "writable keeps it");
        assert!(!should_drop_runtime_dir(1000, None, true, no), "nothing set, nothing to drop");
    }

    #[test]
    fn dir_writable_reads_the_real_permissions() {
        let h = tmp();
        assert!(dir_writable(h.path()));
        assert!(dir_writable(&h.path().join("not-yet")), "a missing dir is created later");
    }

    #[test]
    fn hub_side_name_keeps_or_prefixes_and_refuses_the_impossible() {
        let c = RegisterClaim::new("alice", "alice-").unwrap();
        assert_eq!(hub_side_name(&c, "alice-review").unwrap(), "alice-review");
        assert_eq!(hub_side_name(&c, "worker-1").unwrap(), "alice-worker-1");
        assert!(hub_side_name(&c, "bad name").is_err());
        assert!(hub_side_name(&c, &"x".repeat(64)).is_err(), "prefix pushes it past 64");
    }

    #[test]
    fn claim_is_required_and_strict_name_must_sit_under_the_prefix() {
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
    fn forward_token_is_minted_from_the_session_secret_and_validates_against_it() {
        let secret = auth::generate_secret();
        let hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
        let t = mint_forward_token(Some(&hex), "tl-abc").expect("token");
        let (payload, scope) = auth::validate_token(&secret, &t, None).expect("valid against the same secret");
        assert_eq!(scope, auth::PermissionScope::Execute);
        assert_eq!(payload.session_id, "tl-abc");
        assert!(auth::validate_token(&auth::generate_secret(), &t, None).is_err(), "another secret refuses it");
        assert_eq!(mint_forward_token(None, "x"), None, "no secret: observe only");
        assert_eq!(mint_forward_token(Some("zz"), "x"), None, "malformed secret: none");
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
