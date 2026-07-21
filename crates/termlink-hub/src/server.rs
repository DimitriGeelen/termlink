use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf,
};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::watch;

use termlink_protocol::control;
use termlink_protocol::jsonrpc::{ErrorResponse, Request, Response, RpcResponse};
use termlink_session::auth::{self, PermissionScope};
use termlink_session::auth::PeerCredentials;
use termlink_session::discovery;

use crate::pidfile;
use crate::remote_store;
use crate::router;
use crate::supervisor;
use crate::tls;

/// Return the well-known hub socket path: `runtime_dir()/hub.sock`.
pub fn hub_socket_path() -> PathBuf {
    discovery::runtime_dir().join("hub.sock")
}

/// T-1633: Emit a one-shot warning when the hub starts as root with the
/// default `runtime_dir` falling through to `/tmp/termlink-$UID`. Hits the
/// PL-021 footgun pattern: bare-respawn without `TERMLINK_RUNTIME_DIR=...`
/// loses the operator's intended persistent path, so identity and channel
/// state can vanish at next reboot (volatile /tmp via tmpfs or
/// systemd-tmpfiles).
///
/// Returns true if the warning was emitted (used by tests; production callers
/// can ignore the return).
///
/// Conditions for warning (all must hold):
///   1. `TERMLINK_RUNTIME_DIR` is unset (operator did not explicitly choose)
///   2. Effective UID is 0 (root — non-root /tmp/termlink-UID is the
///      documented default for interactive sessions and not a footgun)
///   3. The resolved default path starts with `/tmp/`
fn warn_if_volatile_default_runtime_dir() -> bool {
    let uid = unsafe { libc::getuid() };
    warn_if_volatile_default_runtime_dir_impl(uid, discovery::runtime_dir())
}

/// Pure-function core of [`warn_if_volatile_default_runtime_dir`]. `uid` and
/// `resolved` are injected so the four-way truth table (env set/unset × root/
/// non-root × /tmp/non-tmp) can be exercised without privilege or env
/// mutation. Production callers always go through the wrapper.
fn warn_if_volatile_default_runtime_dir_impl(uid: u32, resolved: PathBuf) -> bool {
    if std::env::var("TERMLINK_RUNTIME_DIR").is_ok() {
        return false;
    }

    if uid != 0 {
        return false;
    }

    let resolved_str = resolved.to_string_lossy();
    if !resolved_str.starts_with("/tmp/") {
        return false;
    }

    tracing::warn!(
        resolved = %resolved.display(),
        "Hub starting as root with TERMLINK_RUNTIME_DIR unset — falling through to volatile /tmp default. \
         If /tmp is wiped on reboot (tmpfs OR systemd-tmpfiles D /tmp, PL-021), hub.secret + TLS cert \
         + bus state will be regenerated on next boot and ALL TOFU-pinned clients will need to re-auth. \
         For production: set TERMLINK_RUNTIME_DIR=/var/lib/termlink (ensure dir exists, owned by root, 0700), \
         or install the systemd unit at .context/systemd/termlink-hub.service which carries the env."
    );
    true
}

/// Return the hub secret file path: `runtime_dir()/hub.secret`.
pub fn hub_secret_path() -> PathBuf {
    discovery::runtime_dir().join("hub.secret")
}

/// A handle to signal the hub to shut down gracefully.
#[derive(Clone)]
pub struct ShutdownHandle {
    tx: watch::Sender<bool>,
}

impl ShutdownHandle {
    /// Signal the hub to shut down. The accept loop will stop and
    /// active connections will be given time to complete.
    pub fn shutdown(&self) {
        let _ = self.tx.send(true);
    }
}

/// Load an existing hub secret from disk if present and valid. Returns
/// `Some(hex)` on valid existing secret, `None` if missing or unparseable.
fn load_existing_hub_secret() -> Option<String> {
    let path = hub_secret_path();
    let contents = std::fs::read_to_string(&path).ok()?;
    let hex = contents.trim();
    if hex.len() != 64 {
        return None;
    }
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(hex.to_string())
}

/// Return the hub secret, reusing the on-disk value when possible (T-933).
///
/// If `hub.secret` already exists and parses as a valid 64-char hex string,
/// it is reused — this preserves HMAC auth across hub restarts so
/// cross-host agents don't have to re-distribute their cached secret on
/// every bounce. Otherwise a fresh secret is generated and written to disk
/// with mode 0600.
fn generate_and_write_hub_secret() -> std::io::Result<String> {
    if let Some(existing) = load_existing_hub_secret() {
        tracing::info!(
            path = %hub_secret_path().display(),
            "Hub secret loaded from disk (persist-if-present, T-933)"
        );
        return Ok(existing);
    }

    let secret = auth::generate_secret();
    let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

    let path = hub_secret_path();
    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &secret_hex)?;

    // Set file permissions to 0600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!(path = %path.display(), "Hub secret written");
    Ok(secret_hex)
}

/// Start the hub server, binding to the given socket path.
///
/// Returns a [`ShutdownHandle`] that can be used to trigger graceful shutdown.
/// The server will:
/// 1. Stop accepting new connections
/// 2. Wait up to 5 seconds for active connections to complete
/// 3. Remove pidfile and socket file
///
/// Acquires a pidfile to prevent multiple hub instances. The pidfile is removed
/// on clean shutdown. Stale pidfiles from crashed hubs are cleaned automatically.
pub async fn run(socket_path: &Path) -> std::io::Result<ShutdownHandle> {
    run_with_tcp(socket_path, None).await
}

/// Start the hub server with optional TCP listener.
///
/// When `tcp_addr` is provided (e.g., "0.0.0.0:9100"), the hub listens on
/// both the Unix socket and the TCP address simultaneously. A hub secret is
/// generated and written to `hub.secret` for TCP auth.
pub async fn run_with_tcp(
    socket_path: &Path,
    tcp_addr: Option<&str>,
) -> std::io::Result<ShutdownHandle> {
    // T-1633: One-shot footgun check before we commit to any runtime_dir path.
    // Emitted before pidfile acquisition so the warning lands even if startup
    // fails later for an unrelated reason.
    let _ = warn_if_volatile_default_runtime_dir();

    let pidfile_path = pidfile::hub_pidfile_path();

    // Acquire pidfile (prevents double-start, cleans stale)
    pidfile::acquire(&pidfile_path).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::AddrInUse, e.to_string())
    })?;

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let unix_listener = UnixListener::bind(socket_path)?;
    tracing::info!(path = %socket_path.display(), "Hub listening on Unix");

    // Generate hub secret when TCP is enabled
    let token_secret = if tcp_addr.is_some() {
        Some(generate_and_write_hub_secret()?)
    } else {
        None
    };

    // Optional TCP listener with TLS
    let (tcp_listener, tls_acceptor) = if let Some(addr) = tcp_addr {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            std::io::Error::new(e.kind(), format!("Failed to bind TCP {}: {}", addr, e))
        })?;
        let local_addr = listener.local_addr()?;
        tracing::info!(%local_addr, "Hub listening on TCP (TLS)");

        // T-1026: Record TCP address for hub restart (server-side, covers all start paths)
        let tcp_flag = termlink_session::discovery::runtime_dir().join("hub.tcp");
        let _ = std::fs::write(&tcp_flag, local_addr.to_string());

        let acceptor = tls::load_or_generate_cert()?;
        (Some(listener), Some(acceptor))
    } else {
        // T-1026: Remove stale hub.tcp if starting without TCP
        let tcp_flag = termlink_session::discovery::runtime_dir().join("hub.tcp");
        let _ = std::fs::remove_file(&tcp_flag);
        (None, None)
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let handle = ShutdownHandle { tx: shutdown_tx };

    // Initialize the remote session store
    let remote_store = router::init_remote_store();

    // T-966: Initialize the event aggregator
    router::init_aggregator();

    // T-1160: Initialize the channel bus at <runtime_dir>/bus/.
    let bus_root = termlink_session::discovery::runtime_dir().join("bus");
    crate::channel::init_bus(bus_root);

    // T-1300: Initialize the topic↔role lint engine. Reads
    // <runtime_dir>/topic_roles.yaml if present, otherwise installs the
    // built-in defaults from T-1297 § Spike 3. Spawns a SIGHUP watcher so
    // operators can edit the rule file and reload without a hub restart.
    let runtime_dir = termlink_session::discovery::runtime_dir();
    crate::topic_lint::init(&runtime_dir);
    crate::topic_lint::spawn_sighup_watcher();

    // T-1304: Initialize the RPC audit log. Best-effort append-only telemetry
    // sink at <runtime_dir>/rpc-audit.jsonl, read by `fw metrics api-usage`
    // for T-1166 entry-gate measurement.
    crate::rpc_audit::init(&runtime_dir);

    // T-2048: Install connection-cap + per-sender rate-limit governors.
    // Reads TERMLINK_MAX_CONNECTIONS / TERMLINK_RATE_LIMIT_PER_SEC from env,
    // falls back to DEFAULT_MAX_CONNECTIONS / DEFAULT_RATE_LIMIT_PER_SEC.
    crate::governor::init();

    // T-2137: Spawn the periodic rate-bucket eviction loop. Closes the
    // T-2018 §6 #10 retention/compaction gap — without this, the
    // per-sender rate-bucket HashMap grew unbounded
    // (production observation: 258_236 buckets against a 5-agent fleet).
    crate::governor::spawn_rate_evict_loop();

    // T-2049: Install client_msg_id LRU dedupe cache. Reads
    // TERMLINK_DEDUPE_TTL_MS / TERMLINK_DEDUPE_CAPACITY from env, falls
    // back to DEFAULT_DEDUPE_TTL_MS / DEFAULT_DEDUPE_CAPACITY.
    crate::dedupe::init();

    // T-2027/T-2089 slice 1: Install cv_index for broadcast-with-replay.
    // Reads TERMLINK_CV_INDEX_CAP_PER_TOPIC from env, falls back to
    // DEFAULT_CV_INDEX_CAP_PER_TOPIC.
    crate::cv_index::init();

    // T-2333 (arc-004 webhook fan-out, Slice 2): Install the outbound webhook
    // runtime. Reads TERMLINK_WEBHOOK_CONFIG (JSON path); absent/invalid ⇒
    // disabled with no panic (opt-in, no hard dependency).
    crate::webhook::init();
    // T-2334 (Slice 3): background retry-drain loop for failed webhook deliveries
    // (exponential backoff + dead-letter). Idles cheaply when webhooks disabled.
    crate::webhook::spawn_retry_loop();

    // Start the session supervisor
    let supervisor_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        supervisor::run(supervisor::DEFAULT_INTERVAL, supervisor_rx).await;
    });

    // T-2427: opt-in periodic retention sweeper. Enabled ONLY when
    // TERMLINK_SWEEP_INTERVAL_SECS is set to a positive integer (typically in
    // the systemd unit next to TERMLINK_RUNTIME_DIR). Unset = exact T-1155
    // behavior: retention enforced only via explicit channel.sweep.
    if let Some(sweep_interval) = crate::retention_sweeper::interval_from_env() {
        let sweeper_rx = shutdown_rx.clone();
        tokio::spawn(async move {
            crate::retention_sweeper::run(sweep_interval, sweeper_rx).await;
        });
    } else {
        tracing::info!(
            "Periodic retention sweeper disabled (TERMLINK_SWEEP_INTERVAL_SECS unset) — retention enforced only via explicit channel.sweep (T-1155); set the env var in the hub unit to opt in (T-2427)"
        );
    }

    // Start the remote store reaper (expires stale remote sessions)
    let reaper_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        remote_store::run_reaper(remote_store, remote_store::REAPER_INTERVAL, reaper_rx).await;
    });

    let socket_path_owned = socket_path.to_path_buf();
    tokio::spawn(async move {
        run_accept_loop(unix_listener, tcp_listener, tls_acceptor, token_secret, shutdown_rx).await;

        // Cleanup on exit. Secret file is intentionally preserved (T-933:
        // persist-if-present) so cross-host agents don't need to
        // re-distribute credentials on every hub restart.
        let _ = std::fs::remove_file(&socket_path_owned);
        // T-1026: Remove hub.tcp so non-TCP restart doesn't inherit stale config
        let tcp_flag = termlink_session::discovery::runtime_dir().join("hub.tcp");
        let _ = std::fs::remove_file(&tcp_flag);
        // T-1028: Cert files intentionally preserved (persist-if-present, T-985)
        // so client TOFU fingerprints survive hub restarts.
        // Use tls::cleanup() only for explicit --clean shutdown.
        pidfile::remove(&pidfile_path);
        tracing::info!("Hub shut down cleanly");
    });

    Ok(handle)
}

/// Start the hub server and block until shutdown.
///
/// This is the simple API for CLI usage — starts the server and waits
/// for the shutdown handle to be triggered.
pub async fn run_blocking(socket_path: &Path, tcp_addr: Option<&str>) -> std::io::Result<()> {
    let pidfile_path = pidfile::hub_pidfile_path();

    // Acquire pidfile (prevents double-start, cleans stale)
    pidfile::acquire(&pidfile_path).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::AddrInUse, e.to_string())
    })?;

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let unix_listener = UnixListener::bind(socket_path)?;
    tracing::info!(path = %socket_path.display(), "Hub listening on Unix");

    // Generate hub secret when TCP is enabled
    let token_secret = if tcp_addr.is_some() {
        Some(generate_and_write_hub_secret()?)
    } else {
        None
    };

    let (tcp_listener, tls_acceptor) = if let Some(addr) = tcp_addr {
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            std::io::Error::new(e.kind(), format!("Failed to bind TCP {}: {}", addr, e))
        })?;
        let local_addr = listener.local_addr()?;
        tracing::info!(%local_addr, "Hub listening on TCP (TLS)");

        // T-1026: Record TCP address for hub restart
        let tcp_flag = termlink_session::discovery::runtime_dir().join("hub.tcp");
        let _ = std::fs::write(&tcp_flag, local_addr.to_string());

        let acceptor = tls::load_or_generate_cert()?;
        (Some(listener), Some(acceptor))
    } else {
        let tcp_flag = termlink_session::discovery::runtime_dir().join("hub.tcp");
        let _ = std::fs::remove_file(&tcp_flag);
        (None, None)
    };

    // T-2048: Governors (idempotent — no-op if `run_with_tcp` already
    // installed them, e.g. tests that spin up both shapes).
    crate::governor::init();
    // T-2137: Rate-bucket eviction loop. Each call spawns a fresh task;
    // in practice this path is only used by simple-API callers (and
    // tests, which exit before the first eviction tick fires), so the
    // small duplication risk vs `run_with_tcp` is acceptable.
    crate::governor::spawn_rate_evict_loop();
    // T-2049: Dedupe cache (idempotent).
    crate::dedupe::init();
    // T-2333: Webhook fan-out runtime (idempotent; opt-in via TERMLINK_WEBHOOK_CONFIG).
    crate::webhook::init();
    // T-2334: Webhook retry-drain loop (idempotent-safe; idles when disabled).
    crate::webhook::spawn_retry_loop();

    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    run_accept_loop(unix_listener, tcp_listener, tls_acceptor, token_secret, shutdown_rx).await;

    // Cleanup on exit. Secret file is intentionally preserved (T-933:
    // persist-if-present). Cert files also preserved (T-985: TOFU stability).
    pidfile::remove(&pidfile_path);
    Ok(())
}

/// Map hub RPC methods to their required permission scopes.
///
/// Hub-specific methods have their own scope mapping. Forwarded methods
/// (anything not handled directly by the hub) use `auth::method_scope()`.
fn hub_method_scope(method: &str) -> PermissionScope {
    match method {
        // Observe: read-only hub operations
        control::method::SESSION_DISCOVER
        | control::method::SESSION_WHOAMI
        | control::method::EVENT_COLLECT
        | "hub.version"
        | "hub.bus_state"
        | "hub.governor_status" => PermissionScope::Observe,

        // Observe: read-only channel/bus + agent introspection (T-2267).
        // Before this arm existed the entire channel.* surface fell through
        // to the `_ => Execute` deny-by-default below (via auth::method_scope),
        // so honest read-only cross-hub callers were forced to mint
        // execute-scope tokens just to LIST a topic — the root cause of
        // recurring "I can't reach the hub" misdiagnoses. These are pure reads
        // with no side effects and must require only Observe.
        control::method::CHANNEL_LIST
        | control::method::CHANNEL_SUBSCRIBE
        | control::method::CHANNEL_RECEIPTS
        | control::method::CHANNEL_CLAIMS
        | control::method::CHANNEL_CLAIMS_SUMMARY
        | control::method::CHANNEL_CV_KEYS
        | control::method::AGENT_FIND_IDLE => PermissionScope::Observe,

        // Interact: append / own-lease mutations + addressed/fan-out delivery.
        // channel.claim/renew/release mutate only the caller's own lease (T-2267).
        control::method::CHANNEL_POST
        | control::method::CHANNEL_CLAIM
        | control::method::CHANNEL_RENEW
        | control::method::CHANNEL_RELEASE
        | control::method::EVENT_EMIT_TO
        | control::method::EVENT_BROADCAST
        | "session.register_remote"
        | "session.heartbeat"
        | "session.deregister_remote" => PermissionScope::Interact,

        // Control: topic lifecycle, retention policy, operator claim overrides,
        // and destructive bulk operations (T-2267). These affect other
        // subscribers or override another worker's ownership.
        control::method::CHANNEL_CREATE
        | control::method::CHANNEL_SET_RETENTION
        | control::method::CHANNEL_TRANSFER_CLAIM
        | control::method::CHANNEL_FORCE_RELEASE
        | control::method::CHANNEL_TRIM
        | control::method::CHANNEL_SWEEP => PermissionScope::Control,

        // Execute: irreversible whole-topic destruction (T-2421). One notch
        // above trim/sweep (Control): those empty a topic under a policy the
        // topic keeps; delete erases the topic's existence, cursors, claims,
        // and cv_index — for every subscriber, unrecoverably.
        control::method::CHANNEL_DELETE => PermissionScope::Execute,

        // Forwarded methods: use per-method scope from the session auth model.
        // Genuinely unknown methods still deny-by-default to Execute there.
        _ => auth::method_scope(method),
    }
}

/// T-2048: Write a single `HUB_AT_CAPACITY` JSON-RPC error envelope to a
/// just-accepted Unix-socket stream and close. LOUD refuse per IW-3 —
/// the client gets a structured `{error: {code: -32019, data:
/// {retry_after_ms}}}` envelope on their first `lines.next_line().await`
/// instead of a silent EOF. The `id` is `null` because no request has
/// been parsed yet.
async fn write_capacity_refusal<S>(stream: &mut S, retry_after_ms: u64)
where
    S: AsyncWrite + Unpin,
{
    let envelope = ErrorResponse::with_data(
        serde_json::Value::Null,
        control::error_code::HUB_AT_CAPACITY,
        &format!("Hub at capacity (retry in {retry_after_ms}ms)"),
        serde_json::json!({ "retry_after_ms": retry_after_ms }),
    );
    if let Ok(mut line) = serde_json::to_vec(&RpcResponse::Error(envelope)) {
        line.push(b'\n');
        let _ = stream.write_all(&line).await;
        let _ = stream.shutdown().await;
    }
}

/// Convert a hex string to a 32-byte array (for token_secret decoding).
fn hex_to_bytes(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

/// Handle a `hub.auth` request — validate the token and upgrade connection scope.
fn handle_hub_auth_token(
    req: &Request,
    token_secret: &Option<String>,
    granted_scope: &mut Option<PermissionScope>,
    id: serde_json::Value,
) -> Option<RpcResponse> {
    let secret = match token_secret {
        Some(s) => s,
        None => {
            return Some(
                ErrorResponse::new(
                    id,
                    control::error_code::AUTH_DENIED,
                    "Token authentication not configured for this hub",
                )
                .into(),
            );
        }
    };

    let secret_bytes: auth::TokenSecret = match hex_to_bytes(secret) {
        Some(b) => b,
        None => {
            tracing::error!("Invalid hub token_secret (not valid hex)");
            return Some(
                ErrorResponse::internal_error(id, "Internal auth configuration error").into(),
            );
        }
    };

    // Extract the token string from params
    let token_str = match req.params.get("token").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return Some(
                ErrorResponse::new(
                    id,
                    control::error_code::AUTH_REQUIRED,
                    "Missing 'token' parameter",
                )
                .into(),
            );
        }
    };

    // Validate the token (no session_id check — hub tokens are hub-scoped)
    match auth::validate_token(&secret_bytes, token_str, None) {
        Ok((_payload, scope)) => {
            *granted_scope = Some(scope);
            tracing::info!(
                scope = %scope,
                "Hub connection authenticated via token"
            );
            Some(
                Response::success(
                    id,
                    serde_json::json!({
                        "authenticated": true,
                        "scope": scope.to_string(),
                    }),
                )
                .into(),
            )
        }
        Err(e) => {
            tracing::warn!(error = %e, "Hub token validation failed");
            Some(
                ErrorResponse::new(
                    id,
                    control::error_code::AUTH_DENIED,
                    &format!("Token validation failed: {e}"),
                )
                .into(),
            )
        }
    }
}

/// Accept loop: spawns a task per connection.
///
/// Rejects connections from different UIDs (same security model as session server).
/// TCP connections start unauthenticated (only `hub.auth` allowed).
/// Unix connections from the same UID get full access.
/// Stops accepting when the shutdown signal is received, then waits up to 5 seconds
/// for active connections to complete.
pub async fn run_accept_loop(
    unix_listener: UnixListener,
    tcp_listener: Option<TcpListener>,
    tls_acceptor: Option<tokio_rustls::TlsAcceptor>,
    token_secret: Option<String>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let owner_uid = unsafe { libc::getuid() };
    // T-2048: `ConnGovernor` (process-global, installed by `run_with_tcp`
    // / `run_blocking` via `crate::governor::init()`) is the source of
    // truth for cap ENFORCEMENT. The per-loop `active_connections`
    // remains as the source of truth for DRAIN — it counts handlers
    // this specific accept loop spawned, immune to cross-test pollution
    // of the global governor in the same process.
    let active_connections = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let token_secret = std::sync::Arc::new(token_secret);
    let tls_acceptor = std::sync::Arc::new(tls_acceptor);

    loop {
        // Select over Unix listener, optional TCP listener, and shutdown signal
        tokio::select! {
            result = unix_listener.accept() => {
                match result {
                    Ok((mut stream, _addr)) => {
                        // Extract peer credentials and check UID
                        // T-1407: capture peer_pid so we can thread it into the
                        // audit log + legacy-method warn for Unix-socket callers.
                        let mut peer_pid: Option<u32> = None;
                        match PeerCredentials::from_tokio_stream(&stream) {
                            Ok(creds) => {
                                if !creds.is_same_user(owner_uid) {
                                    tracing::warn!(
                                        peer_uid = creds.uid,
                                        peer_pid = ?creds.pid,
                                        owner_uid = owner_uid,
                                        "Hub: rejected Unix connection from different UID"
                                    );
                                    continue;
                                }
                                peer_pid = creds.pid;
                            }
                            Err(e) => {
                                tracing::debug!(
                                    error = %e,
                                    "Hub: could not extract peer credentials, allowing connection"
                                );
                            }
                        }

                        // T-2048: connection-cap check BEFORE spawn. LOUD refuse
                        // per IW-3 — write one envelope, close socket.
                        if let Err(hint) = crate::governor::conn_governor().try_acquire() {
                            tracing::warn!(
                                peer_pid = ?peer_pid,
                                retry_after_ms = hint.retry_after_ms,
                                "Hub: rejecting Unix connection — at capacity"
                            );
                            tokio::spawn(async move {
                                write_capacity_refusal(&mut stream, hint.retry_after_ms).await;
                            });
                            continue;
                        }

                        // Unix same-UID connections get full access (no auth needed)
                        let secret = token_secret.clone();
                        let counter = active_connections.clone();
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tokio::spawn(async move {
                            // T-1409: Unix connections have no TCP address; pass None.
                            handle_connection(
                                stream,
                                Some(PermissionScope::Execute),
                                (*secret).clone(),
                                peer_pid,
                                None,
                            )
                            .await;
                            crate::governor::conn_governor().release();
                            counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Hub Unix accept failed");
                        break;
                    }
                }
            }

            result = async {
                match tcp_listener.as_ref() {
                    Some(l) => l.accept().await,
                    None => std::future::pending().await,
                }
            } => {
                match result {
                    Ok((mut tcp_stream, peer_addr)) => {
                        tracing::info!(
                            %peer_addr,
                            "Hub: TCP connection accepted (TLS handshake pending)"
                        );

                        // T-2048: connection-cap check BEFORE spawn. For TCP we
                        // can't write a clean envelope until TLS completes —
                        // simplest LOUD refuse: close the socket and skip
                        // handshake entirely. Operator sees TLS-handshake-fail
                        // on the client side; capacity_hits_total surfaces
                        // server-side via `hub.governor_status`.
                        if let Err(hint) = crate::governor::conn_governor().try_acquire() {
                            tracing::warn!(
                                %peer_addr,
                                retry_after_ms = hint.retry_after_ms,
                                "Hub: rejecting TCP connection — at capacity"
                            );
                            // Close without TLS handshake — fastest path off the wire.
                            tokio::spawn(async move {
                                let _ = tcp_stream.shutdown().await;
                            });
                            continue;
                        }

                        // TCP connections start with zero scope (unauthenticated)
                        let secret = token_secret.clone();
                        let acceptor = tls_acceptor.clone();
                        let counter = active_connections.clone();
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let peer_addr_str = peer_addr.to_string();
                        tokio::spawn(async move {
                            // T-1407: TCP/TLS connections are network-remote; no
                            // local PID exists, pass None.
                            // T-1409: pass peer_addr so audit + warn carry the
                            // network source for callers without a `from` tag.
                            if let Some(tls) = acceptor.as_ref() {
                                match tls.accept(tcp_stream).await {
                                    Ok(tls_stream) => {
                                        handle_connection(
                                            tls_stream,
                                            None,
                                            (*secret).clone(),
                                            None,
                                            Some(peer_addr_str),
                                        )
                                        .await;
                                    }
                                    Err(e) => {
                                        tracing::warn!(%peer_addr, error = %e, "Hub: TLS handshake failed");
                                    }
                                }
                            } else {
                                // No TLS configured — use raw TCP (tests only)
                                handle_connection(
                                    tcp_stream,
                                    None,
                                    (*secret).clone(),
                                    None,
                                    Some(peer_addr_str),
                                )
                                .await;
                            }
                            crate::governor::conn_governor().release();
                            counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Hub TCP accept failed");
                        // Don't break — Unix listener can still work
                    }
                }
            }

            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!("Hub: shutdown signal received, draining connections");
                    break;
                }
            }
        }
    }

    // Drain: wait up to 5 seconds for active connections to finish.
    // T-2048: drain uses the per-loop `active_connections` (counts only
    // handlers THIS accept loop spawned). The global `ConnGovernor` is
    // for ENFORCEMENT (process-wide cap); using its `current()` here
    // would block on counts polluted by other accept loops in the same
    // process (notably under cargo-test parallel harness).
    let drain_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while active_connections.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        if tokio::time::Instant::now() >= drain_deadline {
            let remaining = active_connections.load(std::sync::atomic::Ordering::Relaxed);
            tracing::warn!(remaining, "Hub: drain timeout, forcing shutdown");
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

/// Handle a single hub client connection with auth enforcement.
///
/// `granted_scope`:
/// - `Some(Execute)` for Unix same-UID connections (full access)
/// - `None` for TCP connections (unauthenticated — only `hub.auth` allowed)
///
/// The scope can be upgraded via `hub.auth` with a valid token.
///
/// T-1407: `peer_pid` is the connect-time PID from `getsockopt(SO_PEERCRED)`
/// for Unix-socket callers; threaded into the audit log + legacy-method
/// warn so non-TermLink callers (raw JSON-RPC shells, third-party tools)
/// can be identified by `ps -p <pid>` even when the JSON-RPC `from` field
/// is absent. TCP/TLS connections pass `None`.
/// T-1409: `peer_addr` is the TCP source address (`"ip:port"`) for
/// network-remote callers; the network analogue of `peer_pid`. Unix
/// connections pass `None`.
async fn handle_connection<S>(
    stream: S,
    initial_scope: Option<PermissionScope>,
    token_secret: Option<String>,
    peer_pid: Option<u32>,
    peer_addr: Option<String>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    // T-2305 (arc-004 push-transport): sniff the first byte to distinguish a
    // WebSocket upgrade — an HTTP request line begins with 'G' of "GET " — from
    // the legacy newline-delimited JSON-RPC line protocol, whose first byte is
    // '{' (a JSON object). The sniffed byte must not be lost, so we replay it to
    // whichever handler runs via PeekedStream. Both handlers share one dispatch
    // path (`process_request_message`), so auth/rate-limit/audit stay identical.
    let mut stream = stream;
    let mut first = [0u8; 1];
    let n = match stream.read(&mut first).await {
        Ok(0) => return, // client closed before sending anything
        Ok(n) => n,
        Err(e) => {
            tracing::debug!(error = %e, "Hub: read error before dispatch");
            return;
        }
    };
    let is_ws = first[..n].first() == Some(&b'G');
    let stream = PeekedStream::new(first[..n].to_vec(), stream);

    if is_ws {
        handle_ws_connection(stream, initial_scope, token_secret, peer_pid, peer_addr).await;
    } else {
        handle_line_connection(stream, initial_scope, token_secret, peer_pid, peer_addr).await;
    }
}

/// T-2305: the legacy newline-delimited JSON-RPC transport (extracted verbatim
/// from the old `handle_connection` body — behaviour unchanged for line clients).
async fn handle_line_connection<S>(
    stream: S,
    initial_scope: Option<PermissionScope>,
    token_secret: Option<String>,
    peer_pid: Option<u32>,
    peer_addr: Option<String>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();
    let mut granted_scope = initial_scope;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(mut json) =
            process_request_message(&line, &token_secret, &mut granted_scope, peer_pid, &peer_addr)
                .await
        {
            json.push('\n');
            if let Err(e) = writer.write_all(json.as_bytes()).await {
                tracing::debug!(error = %e, "Hub: client disconnected");
                break;
            }
        }
    }
}

/// T-2305 (arc-004 push-transport S1): the WebSocket transport. Completes the
/// RFC6455 handshake over the already-TLS-terminated stream, then carries the
/// same JSON-RPC dispatch as the line transport — one request per text frame,
/// one response per text frame. `hub.auth` flows through the shared dispatch so
/// HMAC scope-caching (invalid → authed calls rejected) is reused verbatim.
/// S2 will add the server→client broadcast push loop alongside this read side.
async fn handle_ws_connection<S>(
    stream: S,
    initial_scope: Option<PermissionScope>,
    token_secret: Option<String>,
    peer_pid: Option<u32>,
    peer_addr: Option<String>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let ws = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            tracing::debug!(error = %e, "Hub: WebSocket handshake failed");
            return;
        }
    };
    tracing::debug!(
        peer = ?peer_addr,
        "Hub: WebSocket connection upgraded (arc-004 push-transport)"
    );

    // T-2306 (S2): split so the hub can PUSH server-initiated frames concurrently
    // with serving the client's request frames. `sink` is the write half, `source`
    // the read half; a single `tokio::select!` task multiplexes both — no second
    // task, so `granted_scope` needs no lock.
    let (mut sink, mut source) = ws.split();
    let mut granted_scope = initial_scope;

    // T-2307 (S3): per-connection topic filter set via `hub.ws_subscribe`. Empty =
    // not subscribed → no pushes (opt-in default; the S2 firehose is now gated).
    let mut topic_filter: Vec<String> = Vec::new();

    // Subscribe to the in-process broadcast up front (before auth) so no event is
    // missed between subscribe completing and the first push. Events are drained and
    // dropped until the connection has both authed AND subscribed (gated below).
    // `None` when the aggregator is not initialized (a minimal test harness) → the
    // push arm stays dormant forever.
    let mut event_rx = router::aggregator().map(|a| a.subscribe());

    loop {
        tokio::select! {
            // ---- client → hub: request frames (same dispatch as the line path) ----
            msg = source.next() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => { tracing::debug!(error = %e, "Hub: WS read error"); break; }
                    None => break, // client closed
                };
                match msg {
                    Message::Text(txt) => {
                        // T-2307: intercept the WS-only `hub.ws_subscribe` control to set
                        // the per-connection topic filter; everything else goes to the
                        // shared dispatch (identical to the line path).
                        if let Some(reply) = maybe_handle_ws_subscribe(
                            txt.as_str(), &granted_scope, &mut topic_filter, peer_pid, &peer_addr,
                        ) {
                            if sink.send(Message::Text(reply.into())).await.is_err() { break; }
                        } else if let Some(json) = process_request_message(
                            txt.as_str(), &token_secret, &mut granted_scope, peer_pid, &peer_addr,
                        ).await {
                            if sink.send(Message::Text(json.into())).await.is_err() { break; }
                        }
                    }
                    Message::Binary(bin) => {
                        // Permissive: accept JSON-RPC in a binary frame too.
                        if let Ok(txt) = String::from_utf8(bin.to_vec()) {
                            if let Some(reply) = maybe_handle_ws_subscribe(
                                &txt, &granted_scope, &mut topic_filter, peer_pid, &peer_addr,
                            ) {
                                if sink.send(Message::Text(reply.into())).await.is_err() { break; }
                            } else if let Some(json) = process_request_message(
                                &txt, &token_secret, &mut granted_scope, peer_pid, &peer_addr,
                            ).await {
                                if sink.send(Message::Text(json.into())).await.is_err() { break; }
                            }
                        }
                    }
                    Message::Ping(p) => { let _ = sink.send(Message::Pong(p)).await; }
                    Message::Close(_) => break,
                    _ => {} // Pong / Frame — ignore
                }
            }

            // ---- hub → client: pushed broadcast events (only once authenticated) ----
            ev = recv_event(&mut event_rx) => {
                match ev {
                    Ok(event) => {
                        // T-2307: forward only if authenticated AND the event's topic
                        // matches this connection's subscription filter (empty filter =
                        // not subscribed = no push). Otherwise drain and drop.
                        if granted_scope.is_some() && ws_topic_matches(&topic_filter, &event.topic) {
                            let push = serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "hub.event",
                                "params": event,
                            });
                            if sink.send(Message::Text(push.to_string().into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "Hub: WS push loop lagged, dropping events");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Aggregator gone — stop selecting on a dead channel, keep serving requests.
                        event_rx = None;
                    }
                }
            }
        }
    }
}

/// T-2306 (arc-004 push-transport S2): await the next broadcast event, or never
/// resolve when there is no receiver — so the WS push `select!` arm stays dormant
/// (instead of busy-looping) when the aggregator isn't initialized or has closed.
async fn recv_event(
    rx: &mut Option<tokio::sync::broadcast::Receiver<crate::aggregator::AggregatedEvent>>,
) -> Result<crate::aggregator::AggregatedEvent, tokio::sync::broadcast::error::RecvError> {
    match rx {
        Some(r) => r.recv().await,
        None => std::future::pending().await,
    }
}

/// T-2307 (arc-004 push-transport S3): does `topic` match any entry of this
/// connection's subscription filter? An entry matches exactly, or as a prefix
/// when written `stem*` (e.g. `dm:*` matches every `dm:` topic). An empty filter
/// matches nothing — the opt-in default (no `hub.ws_subscribe` → no pushes).
fn ws_topic_matches(filter: &[String], topic: &str) -> bool {
    filter.iter().any(|pat| match pat.strip_suffix('*') {
        Some(stem) => topic.starts_with(stem),
        None => topic == pat,
    })
}

/// T-2307 (arc-004 push-transport S3): if `text` is a WS-only `hub.ws_subscribe`
/// control message, apply it to the per-connection `topic_filter` and return the
/// serialized ack/error to send back; otherwise return `None` so the caller falls
/// through to the shared JSON-RPC dispatch. Auth-gated: an unauthenticated or
/// under-scoped connection is refused (never silently subscribed).
fn maybe_handle_ws_subscribe(
    text: &str,
    granted_scope: &Option<PermissionScope>,
    topic_filter: &mut Vec<String>,
    peer_pid: Option<u32>,
    peer_addr: &Option<String>,
) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("method").and_then(|m| m.as_str()) != Some("hub.ws_subscribe") {
        return None;
    }
    let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);

    // T-2372: ws_subscribe is intercepted before process_request_message, so it
    // must charge the same per-sender rate governor and record the same rpc_audit
    // entry as every other authed RPC — otherwise a client can spam ws_subscribe
    // frames un-throttled and with no audit trail (arc-004 review finding #3).
    // T-2432: sender-key derivation is shared with process_request_message via
    // governor::derive_sender_key so the two intercept paths can never drift.
    let from = v
        .get("params")
        .and_then(|p| p.get("from"))
        .and_then(|f| f.as_str());
    let sender_id = v
        .get("params")
        .and_then(|p| p.get("sender_id"))
        .and_then(|f| f.as_str());
    let peer_addr_ref = peer_addr.as_deref();
    let sender_key = crate::governor::derive_sender_key(from, sender_id, peer_addr_ref, peer_pid);
    if let Err(hint) = crate::governor::rate_governor()
        .try_acquire(&sender_key, crate::governor::now_ms())
    {
        tracing::warn!(
            method = "hub.ws_subscribe",
            sender = %sender_key,
            retry_after_ms = hint.retry_after_ms,
            "Hub: rate-limited ws_subscribe"
        );
        return Some(
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": control::error_code::RATE_LIMITED,
                    "message": format!(
                        "Rate limit exceeded for sender '{sender_key}' (retry in {}ms)",
                        hint.retry_after_ms
                    ),
                    "data": { "retry_after_ms": hint.retry_after_ms, "sender": sender_key }
                }
            })
            .to_string(),
        );
    }
    crate::rpc_audit::record("hub.ws_subscribe", from, peer_pid, peer_addr_ref, None);

    // Require an authenticated connection with at least Observe scope.
    match *granted_scope {
        None => Some(
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": control::error_code::AUTH_REQUIRED,
                    "message": "hub.ws_subscribe requires authentication. Call 'hub.auth' first."
                }
            })
            .to_string(),
        ),
        Some(scope) if !scope.satisfies(PermissionScope::Observe) => Some(
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": control::error_code::AUTH_DENIED,
                    "message": "hub.ws_subscribe requires 'observe' scope."
                }
            })
            .to_string(),
        ),
        Some(_) => {
            let topics: Vec<String> = v
                .get("params")
                .and_then(|p| p.get("topics"))
                .and_then(|t| t.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            *topic_filter = topics.clone(); // a second call replaces the filter
            Some(
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "subscribed": topics, "count": topic_filter.len() }
                })
                .to_string(),
            )
        }
    }
}

/// T-2305: process one JSON-RPC request message (one line for the legacy
/// transport, or the text of one WS frame for arc-004 push-transport) and return
/// the serialized response WITHOUT a trailing newline, or `None` for a
/// notification / empty input. Both transports call this, so auth (`hub.auth` →
/// `granted_scope`), rate-limiting, audit, and permission checks are identical —
/// there is no second dispatch path to drift out of sync.
async fn process_request_message(
    line: &str,
    token_secret: &Option<String>,
    granted_scope: &mut Option<PermissionScope>,
    peer_pid: Option<u32>,
    peer_addr: &Option<String>,
) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let response = match serde_json::from_str::<Request>(line) {
            Ok(req) => {
                // T-1304: Record every parseable RPC dispatch (auth attempts,
                // notifications, and authenticated calls). Auth rejections still
                // get recorded — that's the point: the audit log is "what was
                // asked of the hub", not "what succeeded". Best-effort, swallows
                // errors; never blocks dispatch.
                // T-1309: thread optional caller display_name from params.from
                // so legacy-caller breakdown is possible in `fw metrics api-usage`.
                let from = req.params.get("from").and_then(|v| v.as_str());
                let peer_addr_ref = peer_addr.as_deref();

                // T-2048: per-sender rate-limit check BEFORE any dispatch.
                // T-2432 (PL-218): key priority now lives in
                // governor::derive_sender_key — `params.from` → verified
                // `params.sender_id` (every channel.post carries it, T-1427)
                // → `peer_addr` → `peer_pid` → "anonymous". Preferring stable
                // identity over pid means one-shot CLI invocations accumulate
                // into ONE bucket instead of minting a fresh bucket per
                // process (PL-209 bloat mechanism).
                let sender_id = req.params.get("sender_id").and_then(|v| v.as_str());
                let sender_key = crate::governor::derive_sender_key(
                    from,
                    sender_id,
                    peer_addr_ref,
                    peer_pid,
                );
                if let Err(hint) = crate::governor::rate_governor()
                    .try_acquire(&sender_key, crate::governor::now_ms())
                {
                    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
                    tracing::warn!(
                        method = %req.method,
                        sender = %sender_key,
                        retry_after_ms = hint.retry_after_ms,
                        "Hub: rate-limited request"
                    );
                    Some(
                        ErrorResponse::with_data(
                            id,
                            control::error_code::RATE_LIMITED,
                            &format!(
                                "Rate limit exceeded for sender '{sender_key}' (retry in {}ms)",
                                hint.retry_after_ms
                            ),
                            serde_json::json!({
                                "retry_after_ms": hint.retry_after_ms,
                                "sender": sender_key,
                            }),
                        )
                        .into(),
                    )
                } else {
                // T-1622: thread `topic` from params so the legacy event.broadcast
                // residue can be sliced by destination channel — closes the last
                // T-1166 visibility gap (operator can ID *which* channels the
                // residue is going to without SSH+jq on the hub).
                let topic = req.params.get("topic").and_then(|v| v.as_str());
                crate::rpc_audit::record(&req.method, from, peer_pid, peer_addr_ref, topic);
                // T-1311: real-time warn-log when a legacy primitive is dispatched.
                // Rate-limited to one log per (method, from) per 5 minutes inside
                // warn_if_legacy. Operator tailing logs sees deprecated usage
                // immediately, not days later in the audit tally.
                // T-1407: include peer_pid so the warn line carries the originating
                // PID for Unix-socket callers — closes the anonymous-poller blind
                // spot diagnosed during the T-1166 bake.
                // T-1409: include peer_addr so the warn line carries the TCP
                // source address for network callers without a `from` tag.
                crate::rpc_audit::warn_if_legacy(&req.method, from, peer_pid, peer_addr_ref);
                if req.method == control::method::HUB_AUTH {
                    // hub.auth is always allowed (it's the authentication mechanism)
                    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
                    handle_hub_auth_token(&req, token_secret, granted_scope, id)
                } else if req.is_notification() {
                    // Notifications don't get responses
                    router::route(&req, peer_addr_ref).await
                } else {
                    match *granted_scope {
                        None => {
                            // Unauthenticated — only hub.auth is allowed
                            let id = req.id.clone().unwrap_or(serde_json::Value::Null);
                            tracing::warn!(
                                method = %req.method,
                                "Hub: rejected unauthenticated request"
                            );
                            Some(
                                ErrorResponse::new(
                                    id,
                                    control::error_code::AUTH_REQUIRED,
                                    "Authentication required. Call 'hub.auth' with a valid token first.",
                                )
                                .into(),
                            )
                        }
                        Some(scope) => {
                            // Check permission scope
                            let required = hub_method_scope(&req.method);
                            if !scope.satisfies(required) {
                                let id = req.id.clone().unwrap_or(serde_json::Value::Null);
                                tracing::warn!(
                                    method = %req.method,
                                    required = %required,
                                    granted = %scope,
                                    "Hub: permission denied"
                                );
                                Some(
                                    ErrorResponse::new(
                                        id,
                                        control::error_code::AUTH_DENIED,
                                        &format!(
                                            "Permission denied: '{}' requires '{}' scope, connection has '{}'. \
                                             Re-authenticate with a token minted at '{}' scope or higher \
                                             (e.g. `termlink token create --scope {}`), or pass `--scope {}` \
                                             on the remote call. This is a SCOPE mismatch, not a bad secret.",
                                            req.method, required, scope, required, required, required
                                        ),
                                    )
                                    .into(),
                                )
                            } else {
                                router::route(&req, peer_addr_ref).await
                            }
                        }
                    }
                }
                } // close T-2048 else-branch
            }
            Err(e) => {
                tracing::warn!(error = %e, "Hub: failed to parse JSON-RPC request");
                Some(ErrorResponse::parse_error().into())
            }
        };

    response.map(|resp| {
        serde_json::to_string(&resp).unwrap_or_else(|e| {
            tracing::error!(error = %e, "Hub: failed to serialize response");
            let err: RpcResponse =
                ErrorResponse::internal_error(serde_json::Value::Null, "serialization error")
                    .into();
            serde_json::to_string(&err).unwrap_or_else(|_| {
                r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"serialization error"},"id":null}"#.to_string()
            })
        })
    })
}

/// T-2305 (arc-004 push-transport): replays a small already-read prefix (the
/// sniffed first byte) before delegating reads to the inner stream, so the
/// WS-vs-line first-byte sniff doesn't consume the byte the WS handshake or the
/// line parser still needs. Writes pass straight through. Generic over the same
/// `AsyncRead + AsyncWrite` bound as `handle_connection`, so it wraps a
/// `TlsStream`, a raw `TcpStream`, or a Unix stream uniformly.
struct PeekedStream<S> {
    prefix: Vec<u8>,
    pos: usize,
    inner: S,
}

impl<S> PeekedStream<S> {
    fn new(prefix: Vec<u8>, inner: S) -> Self {
        Self {
            prefix,
            pos: 0,
            inner,
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PeekedStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.pos < self.prefix.len() {
            let remaining = &self.prefix[self.pos..];
            let n = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..n]);
            self.pos += n;
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PeekedStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::sync::RwLock;

    use termlink_session::handler::SessionContext;
    use termlink_session::manager::Session;
    use termlink_session::registration::SessionConfig;
    use termlink_session::server as session_server;

    use crate::test_util::ENV_LOCK;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    /// T-2267 regression guard. The full known `channel.*` surface (plus
    /// `agent.find_idle` and `event.emit_to`) must resolve to an EXPLICIT
    /// scope in `hub_method_scope` — never the deny-by-default `Execute`
    /// catch-all. A newly-added channel method that falls through to Execute
    /// fails this test, closing the silent-drift gap that forced read-only
    /// cross-hub callers to mint execute-scope tokens just to list a topic
    /// (the root cause of recurring cross-hub comms failures).
    #[test]
    fn channel_surface_has_explicit_scopes() {
        use control::method as m;
        use PermissionScope::{Control, Execute, Interact, Observe};

        let expected: &[(&str, PermissionScope)] = &[
            // Observe — pure reads, no side effects
            (m::CHANNEL_LIST, Observe),
            (m::CHANNEL_SUBSCRIBE, Observe),
            (m::CHANNEL_RECEIPTS, Observe),
            (m::CHANNEL_CLAIMS, Observe),
            (m::CHANNEL_CLAIMS_SUMMARY, Observe),
            (m::CHANNEL_CV_KEYS, Observe),
            (m::AGENT_FIND_IDLE, Observe),
            // Interact — append / own-lease mutation / addressed delivery
            (m::CHANNEL_POST, Interact),
            (m::CHANNEL_CLAIM, Interact),
            (m::CHANNEL_RENEW, Interact),
            (m::CHANNEL_RELEASE, Interact),
            (m::EVENT_EMIT_TO, Interact),
            // Control — lifecycle / policy / operator override / destructive
            (m::CHANNEL_CREATE, Control),
            (m::CHANNEL_SET_RETENTION, Control),
            (m::CHANNEL_TRANSFER_CLAIM, Control),
            (m::CHANNEL_FORCE_RELEASE, Control),
            (m::CHANNEL_TRIM, Control),
            (m::CHANNEL_SWEEP, Control),
            // Execute — irreversible whole-topic destruction (T-2421)
            (m::CHANNEL_DELETE, Execute),
        ];

        for (method, want) in expected {
            let got = hub_method_scope(method);
            assert_eq!(
                got, *want,
                "scope for '{}' should be {:?}, got {:?}",
                method, want, got
            );
            // Execute is normally evidence the deny-by-default catch-all
            // leaked back in — EXCEPT for methods that are deliberately
            // Execute-scoped (T-2421: channel.delete, irreversible
            // whole-topic destruction, one notch above trim/sweep).
            if *want != PermissionScope::Execute {
                assert_ne!(
                    got,
                    PermissionScope::Execute,
                    "'{}' resolved to Execute (deny-by-default catch-all leaked back in) — \
                     add an explicit scope arm in hub_method_scope",
                    method
                );
            }
        }
    }

    fn test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-hubsrv-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hub_sock(dir: &Path) -> PathBuf {
        dir.join("hub.sock")
    }

    /// Start a session in the given directory, return its handle and registration.
    async fn start_session(
        sessions_dir: &Path,
        name: &str,
    ) -> (
        tokio::task::JoinHandle<()>,
        termlink_session::Registration,
    ) {
        let config = SessionConfig {
            display_name: Some(name.into()),
            ..Default::default()
        };
        let session = Session::register_in(config, sessions_dir).await.unwrap();
        let (registration, listener, _) = session.into_parts();
        let reg = registration.clone();
        let ctx = SessionContext::new(registration);
        let shared = Arc::new(RwLock::new(ctx));

        let handle = tokio::spawn(async move {
            session_server::run_accept_loop(listener, shared).await;
        });

        // Give it a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        (handle, reg)
    }

    /// Start the hub server on the given socket with a shutdown handle (Unix only, no auth).
    fn start_hub_with_shutdown(socket: PathBuf) -> (tokio::task::JoinHandle<()>, watch::Sender<bool>) {
        let (tx, rx) = watch::channel(false);
        let handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket);
            let listener = UnixListener::bind(&socket).unwrap();
            run_accept_loop(listener, None, None, None, rx).await;
        });
        (handle, tx)
    }

    /// Start the hub server on the given socket, return its handle.
    fn start_hub(socket: PathBuf) -> tokio::task::JoinHandle<()> {
        let (handle, _tx) = start_hub_with_shutdown(socket);
        handle
    }

    /// T-933: two calls to generate_and_write_hub_secret() against the same
    /// runtime dir must return the same secret (persist-if-present).
    #[tokio::test]
    async fn hub_secret_persists_across_calls() {
        let _lock = ENV_LOCK.lock().await;
        let dir = test_dir();
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let first = generate_and_write_hub_secret().expect("first generate");
        assert_eq!(first.len(), 64, "secret should be 64 hex chars");
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));

        let second = generate_and_write_hub_secret().expect("second generate");
        assert_eq!(
            first, second,
            "second call must reuse the on-disk secret, not regenerate"
        );

        // Corrupt the file and verify we regenerate on invalid contents.
        std::fs::write(dir.join("hub.secret"), "not-hex").unwrap();
        let third = generate_and_write_hub_secret().expect("regen after corrupt");
        assert_ne!(third, "not-hex");
        assert_eq!(third.len(), 64);
        assert!(third.chars().all(|c| c.is_ascii_hexdigit()));

        unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };
    }

    /// Tests discover + forward in a single test to avoid env var races.
    /// Both require TERMLINK_RUNTIME_DIR to point to the test directory.
    #[tokio::test]
    async fn hub_discover_and_forward() {
        let _lock = ENV_LOCK.lock().await;
        // Clear remote store to avoid leakage from other tests
        if let Some(s) = crate::router::remote_store() { s.clear(); }

        let dir = test_dir();
        let sessions_dir = dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        // Override sessions dir so router::route → manager finds sessions
        unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir) };

        let (h1, _) = start_session(&sessions_dir, "hub-test-a").await;
        let (h2, reg_b) = start_session(&sessions_dir, "hub-test-b").await;

        let hub_socket = hub_sock(&dir);
        let hub_handle = start_hub(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // 1. Discover — should list both sessions
        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "d-1",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], "d-1");
        let sessions = resp["result"]["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 2);

        let names: Vec<&str> = sessions
            .iter()
            .filter_map(|s| s["display_name"].as_str())
            .collect();
        assert!(names.contains(&"hub-test-a"));
        assert!(names.contains(&"hub-test-b"));

        // 2. Forward — ping session-b via the hub
        let req = json!({
            "jsonrpc": "2.0",
            "method": "termlink.ping",
            "id": "fwd-1",
            "params": { "target": reg_b.id.as_str() }
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "fwd-1");
        assert_eq!(resp["result"]["display_name"], "hub-test-b");
        assert_eq!(resp["result"]["state"], "ready");

        hub_handle.abort();
        h1.abort();
        h2.abort();
    }

    #[tokio::test]
    async fn hub_malformed_json_returns_parse_error() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hub_handle = start_hub(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        writer.write_all(b"not valid json\n").await.unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["error"]["code"], -32700); // Parse error

        hub_handle.abort();
    }

    #[tokio::test]
    async fn hub_missing_target_returns_error() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hub_handle = start_hub(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let req = json!({
            "jsonrpc": "2.0",
            "method": "query.status",
            "id": "no-target",
            "params": {}
        });
        writer
            .write_all(format!("{}\n", req).as_bytes())
            .await
            .unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();

        assert_eq!(resp["id"], "no-target");
        assert!(resp["error"]["code"].as_i64().unwrap() < 0);
        assert!(resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Missing"));

        hub_handle.abort();
    }

    #[tokio::test]
    async fn graceful_shutdown_stops_accept_loop() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (hub_handle, shutdown_tx) = start_hub_with_shutdown(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Verify hub is accepting connections
        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        drop(stream);

        // Signal shutdown
        shutdown_tx.send(true).unwrap();

        // Hub should stop within a reasonable time
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            hub_handle,
        ).await;

        assert!(result.is_ok(), "Hub did not shut down within 3 seconds");
    }

    #[tokio::test]
    async fn graceful_shutdown_drains_active_connection() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let (hub_handle, shutdown_tx) = start_hub_with_shutdown(hub_socket.clone());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Connect a client that stays open
        let stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (_reader, _writer) = stream.into_split();

        // Signal shutdown while connection is active
        shutdown_tx.send(true).unwrap();

        // Hub should still shut down (drain timeout or client disconnect)
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(7),
            hub_handle,
        ).await;

        assert!(result.is_ok(), "Hub did not shut down during drain");
    }

    #[tokio::test]
    async fn hub_dual_listen_unix_and_tcp() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Generate a secret for this test
        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        // Start hub with both Unix and TCP listeners
        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            // Bind TCP on ephemeral port
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            // Write port to file so test can read it
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        // 1. Connect via Unix and send a request (should work — full access)
        let unix_stream = tokio::net::UnixStream::connect(&hub_socket).await.unwrap();
        let (reader, mut writer) = unix_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "unix-1",
            "params": {}
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["id"], "unix-1");
        assert!(resp["result"].is_object(), "Unix connection should get valid response");

        // 2. Connect via TCP — unauthenticated requests should be rejected
        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "tcp-noauth",
            "params": {}
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["id"], "tcp-noauth");
        assert_eq!(
            resp["error"]["code"], -32009,
            "TCP without auth should get AUTH_REQUIRED"
        );

        // 3. Authenticate via hub.auth with a valid token
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let req = json!({
            "jsonrpc": "2.0",
            "method": "hub.auth",
            "id": "auth-1",
            "params": { "token": token.raw }
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["result"]["authenticated"], true);
        assert_eq!(resp["result"]["scope"], "execute");

        // 4. After auth, discover should work
        let req = json!({
            "jsonrpc": "2.0",
            "method": "session.discover",
            "id": "tcp-authed",
            "params": {}
        });
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp_line = lines.next_line().await.unwrap().unwrap();
        let resp: serde_json::Value = serde_json::from_str(&resp_line).unwrap();
        assert_eq!(resp["id"], "tcp-authed");
        assert!(resp["result"].is_object(), "Authenticated TCP should get valid response");

        // Cleanup
        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    /// T-2305: round-trip one JSON-RPC request over a client WebSocket stream and
    /// return the parsed response (skips any non-text control frames).
    async fn ws_roundtrip<S>(
        ws: &mut tokio_tungstenite::WebSocketStream<S>,
        req: serde_json::Value,
    ) -> serde_json::Value
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;
        ws.send(Message::Text(req.to_string().into())).await.unwrap();
        loop {
            match ws.next().await.unwrap().unwrap() {
                Message::Text(t) => return serde_json::from_str(t.as_str()).unwrap(),
                _ => continue,
            }
        }
    }

    /// T-2305 (arc-004 push-transport S1): a client that opens a WebSocket to the
    /// hub completes the RFC6455 upgrade, and JSON-RPC over WS reuses the exact
    /// same HMAC auth path as the line transport — unauthenticated calls are
    /// rejected, an invalid token does not authenticate, a valid token upgrades
    /// the connection scope, and the upgraded connection stays open for the next
    /// authed call.
    #[tokio::test]
    async fn ws_upgrade_auth_and_reuse() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        // AC2: the WS upgrade handshake completes on the hub's normal accept path.
        let tcp = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (mut ws, _http_resp) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}/", tcp_port), tcp)
                .await
                .expect("WebSocket upgrade handshake should succeed");

        // AC4: an unauthenticated call over WS is rejected (auth reuse, not a
        // new scheme) — same AUTH_REQUIRED (-32009) as the line path.
        let resp = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"session.discover","id":"ws-noauth","params":{}}),
        )
        .await;
        assert_eq!(
            resp["error"]["code"], -32009,
            "WS without auth should get AUTH_REQUIRED"
        );

        // An invalid token must NOT authenticate, and must not drop the socket.
        let resp = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.auth","id":"ws-bad","params":{"token":"not-a-valid-token"}}),
        )
        .await;
        assert_ne!(
            resp["result"]["authenticated"],
            serde_json::Value::Bool(true),
            "invalid token must not authenticate over WS"
        );

        // AC4a: a valid token authenticates over the SAME still-open WS connection.
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let resp = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.auth","id":"ws-auth","params":{"token":token.raw}}),
        )
        .await;
        assert_eq!(resp["result"]["authenticated"], true);
        assert_eq!(resp["result"]["scope"], "execute");

        // Connection held open: an authed call on the same WS returns a result.
        let resp = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"session.discover","id":"ws-authed","params":{}}),
        )
        .await;
        assert_eq!(resp["id"], "ws-authed");
        assert!(
            resp["result"].is_object(),
            "authenticated WS call should get a result"
        );

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    /// T-2306 (arc-004 push-transport S2): once a WS connection authenticates, the
    /// hub PUSHES aggregator broadcast events to it as `hub.event` notification
    /// frames with no client poll; an unauthenticated connection receives none.
    #[tokio::test]
    async fn ws_push_broadcast_gated_on_auth() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message;

        // Pushes require the aggregator; run_accept_loop alone does not init it.
        crate::router::init_aggregator();

        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        // Unique topic marker so a parallel test's injected event can't be
        // mistaken for ours (the aggregator is a process-global singleton).
        let marker = format!("s2-push-{}", TEST_COUNTER.fetch_add(1, Ordering::SeqCst));

        // --- authed client is pushed the event ---
        let tcp = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (mut ws, _) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}/", tcp_port), tcp)
                .await
                .unwrap();
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let resp = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.auth","id":"a","params":{"token":token.raw}}),
        )
        .await;
        assert_eq!(resp["result"]["authenticated"], true);

        // T-2307: push is opt-in — subscribe to the marker topic first.
        let sub = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.ws_subscribe","id":"sub","params":{"topics":[marker.clone()]}}),
        )
        .await;
        assert_eq!(sub["result"]["count"], 1);

        crate::router::aggregator()
            .unwrap()
            .inject(crate::aggregator::AggregatedEvent {
                session_id: "sess-x".into(),
                session_name: "test".into(),
                seq: 7,
                topic: marker.clone(),
                payload: json!({"hello":"push"}),
                timestamp: 123,
            });

        let mut got_push = false;
        for _ in 0..30 {
            let f = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
                .await
                .expect("WS should yield a frame")
                .unwrap()
                .unwrap();
            if let Message::Text(t) = f {
                let v: serde_json::Value = serde_json::from_str(t.as_str()).unwrap();
                if v["method"] == "hub.event" && v["params"]["topic"] == marker {
                    assert_eq!(v["params"]["payload"]["hello"], "push");
                    assert_eq!(v["params"]["seq"], 7);
                    got_push = true;
                    break;
                }
            }
        }
        assert!(
            got_push,
            "authenticated WS client should receive the pushed hub.event"
        );

        // --- unauthenticated client is NOT pushed events ---
        let tcp2 = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (mut ws2, _) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}/", tcp_port), tcp2)
                .await
                .unwrap();
        let marker2 = format!("s2-noauth-{}", TEST_COUNTER.fetch_add(1, Ordering::SeqCst));
        crate::router::aggregator()
            .unwrap()
            .inject(crate::aggregator::AggregatedEvent {
                session_id: "sess-y".into(),
                session_name: "test".into(),
                seq: 1,
                topic: marker2.clone(),
                payload: json!({}),
                timestamp: 1,
            });
        let saw_push = tokio::time::timeout(std::time::Duration::from_millis(600), async {
            loop {
                match ws2.next().await {
                    Some(Ok(Message::Text(t))) => {
                        let v: serde_json::Value = serde_json::from_str(t.as_str()).unwrap();
                        if v["method"] == "hub.event" {
                            return true;
                        }
                    }
                    Some(Ok(_)) => continue,
                    _ => return false,
                }
            }
        })
        .await;
        assert!(
            saw_push.is_err() || saw_push == Ok(false),
            "unauthenticated WS client must NOT receive pushes"
        );

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    /// T-2307: pure filter semantics — exact match, `stem*` prefix, empty = none.
    #[test]
    fn ws_topic_matches_exact_and_prefix() {
        assert!(ws_topic_matches(&["dm:a".to_string()], "dm:a"));
        assert!(!ws_topic_matches(&["dm:a".to_string()], "dm:b"));
        assert!(ws_topic_matches(&["dm:*".to_string()], "dm:anything"));
        assert!(!ws_topic_matches(&["dm:*".to_string()], "other-topic"));
        assert!(
            !ws_topic_matches(&[], "dm:a"),
            "empty filter must match nothing (opt-in default)"
        );
    }

    /// T-2307 (arc-004 push-transport S3): a subscribed WS client is pushed ONLY
    /// events matching its topic filter; a non-matching event is dropped, and an
    /// authenticated-but-unsubscribed connection is pushed nothing.
    #[tokio::test]
    async fn ws_subscribe_topic_filter() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message;

        crate::router::init_aggregator();

        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let want = format!("dm:want-{}", n);
        let other = format!("dm:other-{}", n);
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);

        // --- subscribed client: only the matching topic is pushed ---
        let tcp = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (mut ws, _) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}/", tcp_port), tcp)
                .await
                .unwrap();
        let a = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.auth","id":"a","params":{"token":token.raw}}),
        )
        .await;
        assert_eq!(a["result"]["authenticated"], true);
        let sub = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.ws_subscribe","id":"s","params":{"topics":[want.clone()]}}),
        )
        .await;
        assert_eq!(sub["result"]["count"], 1);

        let agg = crate::router::aggregator().unwrap();
        // non-matching first (must be filtered out), then matching.
        agg.inject(crate::aggregator::AggregatedEvent {
            session_id: "s".into(),
            session_name: "t".into(),
            seq: 1,
            topic: other.clone(),
            payload: json!({}),
            timestamp: 1,
        });
        agg.inject(crate::aggregator::AggregatedEvent {
            session_id: "s".into(),
            session_name: "t".into(),
            seq: 2,
            topic: want.clone(),
            payload: json!({"ok": true}),
            timestamp: 2,
        });

        let mut seen: Vec<String> = Vec::new();
        for _ in 0..30 {
            let f = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
                .await
                .expect("WS should yield a frame")
                .unwrap()
                .unwrap();
            if let Message::Text(t) = f {
                let v: serde_json::Value = serde_json::from_str(t.as_str()).unwrap();
                if v["method"] == "hub.event" {
                    let topic = v["params"]["topic"].as_str().unwrap().to_string();
                    seen.push(topic.clone());
                    if topic == want {
                        break;
                    }
                }
            }
        }
        assert!(seen.contains(&want), "matching topic should be pushed");
        assert!(
            !seen.contains(&other),
            "non-matching topic must be filtered out, saw: {seen:?}"
        );

        // --- authed but unsubscribed client: nothing is pushed ---
        let tcp2 = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (mut ws2, _) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}/", tcp_port), tcp2)
                .await
                .unwrap();
        let token2 = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let a2 = ws_roundtrip(
            &mut ws2,
            json!({"jsonrpc":"2.0","method":"hub.auth","id":"a2","params":{"token":token2.raw}}),
        )
        .await;
        assert_eq!(a2["result"]["authenticated"], true);
        agg.inject(crate::aggregator::AggregatedEvent {
            session_id: "s".into(),
            session_name: "t".into(),
            seq: 3,
            topic: want.clone(),
            payload: json!({}),
            timestamp: 3,
        });
        let saw = tokio::time::timeout(std::time::Duration::from_millis(600), async {
            loop {
                match ws2.next().await {
                    Some(Ok(Message::Text(t))) => {
                        let v: serde_json::Value = serde_json::from_str(t.as_str()).unwrap();
                        if v["method"] == "hub.event" {
                            return true;
                        }
                    }
                    Some(Ok(_)) => continue,
                    _ => return false,
                }
            }
        })
        .await;
        assert!(
            saw.is_err() || saw == Ok(false),
            "authed-but-unsubscribed client must receive no pushes"
        );

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    /// T-2308 (arc-004 push-transport S4): build a signed `channel.post` fixture
    /// for the real forward path (`handle_channel_post_with`). Mirrors channel.rs's
    /// private `post_params` so the S4 test can ride production code without
    /// reaching into another module's test helpers.
    fn s4_post_params(
        key: &ed25519_dalek::SigningKey,
        topic: &str,
        msg_type: &str,
        payload: &[u8],
        ts: i64,
    ) -> serde_json::Value {
        use base64::Engine;
        use ed25519_dalek::Signer;
        use termlink_protocol::control::channel::canonical_sign_bytes;
        use termlink_session::agent_identity::fingerprint_of;

        let signed = canonical_sign_bytes(topic, msg_type, payload, None, ts);
        let sig = key.sign(&signed);
        let sender_id = fingerprint_of(&key.verifying_key());
        let hexit = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "ts": ts,
            "sender_id": sender_id,
            "sender_pubkey_hex": hexit(key.verifying_key().as_bytes()),
            "signature_hex": hexit(&sig.to_bytes()),
        })
    }

    /// T-2308 (arc-004 push-transport S4): the durable delivery pointer travels the
    /// WS push path INTACT and resolves to an ackable durable position — proving the
    /// arc invariant that receipts/journal remain the unchanged, authoritative path
    /// underneath the live transport.
    ///
    /// End-to-end through real production code:
    ///  1. A signed `channel.post` to an `inbox:*` topic (`handle_channel_post_with`,
    ///     the T-1637 forward path) returns a durable `offset` AND injects an
    ///     `inbox.queued` event carrying `message_offset`/`channel` into the aggregator.
    ///  2. An authed WS client subscribed to the `inbox.queued` topic receives that
    ///     offset over the push stream — asserting the pointer is not truncated or
    ///     re-sequenced by the WS transport.
    ///  3. Reading the durable topic at the pushed offset (`Bus::envelope_at`) yields
    ///     the originally posted body — the exact position the recipient would ack.
    /// No receipt/journal code runs on the WS path itself.
    #[tokio::test]
    async fn ws_delivery_offset_through_push() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message;

        // Pushes require the process-global aggregator; the forward-path post
        // injects into that same singleton the WS push loop subscribes to.
        crate::router::init_aggregator();

        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        // Durable bus the forward-path post writes to. A unique inbox target keeps
        // this test's doorbell distinct from any parallel test's injected events.
        let bus = termlink_bus::Bus::open(&dir.join("bus")).unwrap();
        let target = format!("s4-{}", TEST_COUNTER.fetch_add(1, Ordering::SeqCst));
        let inbox_topic = format!("inbox:{target}");
        bus.create_topic(&inbox_topic, termlink_bus::Retention::Forever)
            .unwrap();

        let queued_topic = termlink_protocol::events::inbox_topic::QUEUED; // "inbox.queued"

        // --- authed WS client subscribed to the durable-delivery doorbell topic ---
        let tcp = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (mut ws, _) =
            tokio_tungstenite::client_async(format!("ws://127.0.0.1:{}/", tcp_port), tcp)
                .await
                .unwrap();
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let resp = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.auth","id":"a","params":{"token":token.raw}}),
        )
        .await;
        assert_eq!(resp["result"]["authenticated"], true);
        let sub = ws_roundtrip(
            &mut ws,
            json!({"jsonrpc":"2.0","method":"hub.ws_subscribe","id":"s","params":{"topics":[queued_topic]}}),
        )
        .await;
        assert_eq!(sub["result"]["count"], 1);

        // --- real signed forward-path post: durable offset + inbox.queued inject ---
        let key = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let body = b"s4-delivery-body";
        let ts = 1_700_000_000i64;
        let params = s4_post_params(&key, &inbox_topic, "note", body, ts);
        let post_resp = crate::channel::handle_channel_post_with(&bus, json!("p"), &params).await;
        let post_val = match post_resp {
            termlink_protocol::jsonrpc::RpcResponse::Success(s) => s.result,
            termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
                panic!("forward-path post failed: {:?}", e.error)
            }
        };
        let durable_offset = post_val["offset"]
            .as_u64()
            .expect("channel.post returns a durable offset");

        // AC2: the offset travels the WS push intact, on inbox.queued, for our channel.
        let mut pushed_offset = None;
        for _ in 0..40 {
            let f = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
                .await
                .expect("WS should yield a frame")
                .unwrap()
                .unwrap();
            if let Message::Text(t) = f {
                let v: serde_json::Value = serde_json::from_str(t.as_str()).unwrap();
                if v["method"] == "hub.event"
                    && v["params"]["topic"] == queued_topic
                    && v["params"]["payload"]["channel"] == inbox_topic
                {
                    pushed_offset = v["params"]["payload"]["message_offset"].as_u64();
                    break;
                }
            }
        }
        let pushed_offset =
            pushed_offset.expect("inbox.queued doorbell for our channel must be pushed over WS");
        assert_eq!(
            pushed_offset, durable_offset,
            "WS-pushed message_offset must equal the durable post offset (pointer intact)"
        );

        // AC3: the pushed pointer resolves to the exact durable body via the
        // unchanged read path (Bus::envelope_at) — the position a recipient acks.
        let env = bus
            .envelope_at(&inbox_topic, pushed_offset)
            .expect("durable read at pushed offset must succeed")
            .expect("a durable record must exist at the pushed offset");
        assert_eq!(
            env.payload,
            body.to_vec(),
            "durable record at the pushed offset must be the originally posted body"
        );

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn tcp_rejected_without_auth() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Try discover without auth
        let req = json!({"jsonrpc": "2.0", "method": "session.discover", "id": "r1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Should get AUTH_REQUIRED");
        assert!(resp["error"]["message"].as_str().unwrap().contains("Authentication required"));

        // Try a forwarded method without auth
        let req = json!({"jsonrpc": "2.0", "method": "termlink.ping", "id": "r2", "params": {"target": "foo"}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Forwarded methods also require auth");

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn tcp_works_after_auth() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Authenticate
        let token = auth::create_token(&secret, PermissionScope::Execute, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "hub.auth", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["result"]["authenticated"], true);

        // After auth, discover should succeed
        let req = json!({"jsonrpc": "2.0", "method": "session.discover", "id": "d1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"]["sessions"].is_array(), "Discover should work after auth");

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn tcp_wrong_token_rejected() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Try auth with wrong secret
        let wrong_secret = auth::generate_secret();
        let token = auth::create_token(&wrong_secret, PermissionScope::Execute, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "hub.auth", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010, "Wrong token should be rejected");

        // Connection should still be unauthenticated
        let req = json!({"jsonrpc": "2.0", "method": "session.discover", "id": "d1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32009, "Still unauthenticated after bad token");

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn tcp_scope_enforcement() {
        let dir = test_dir();
        let hub_socket = hub_sock(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let secret = auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();

        let (tx, rx) = watch::channel(false);
        let socket_clone = hub_socket.clone();
        let secret_clone = secret_hex.clone();
        let hub_handle = tokio::spawn(async move {
            let _ = std::fs::remove_file(&socket_clone);
            let unix_listener = UnixListener::bind(&socket_clone).unwrap();
            let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let tcp_port = tcp_listener.local_addr().unwrap().port();
            std::fs::write(socket_clone.with_extension("tcp_port"), tcp_port.to_string()).unwrap();
            run_accept_loop(unix_listener, Some(tcp_listener), None, Some(secret_clone), rx).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let tcp_port: u16 = std::fs::read_to_string(hub_socket.with_extension("tcp_port"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", tcp_port))
            .await
            .unwrap();
        let (reader, mut writer) = tcp_stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Auth with Observe-only scope
        let token = auth::create_token(&secret, PermissionScope::Observe, "", 3600);
        let req = json!({"jsonrpc": "2.0", "method": "hub.auth", "id": "a1", "params": {"token": token.raw}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["result"]["scope"], "observe");

        // Discover should work (Observe)
        let req = json!({"jsonrpc": "2.0", "method": "session.discover", "id": "d1", "params": {}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert!(resp["result"]["sessions"].is_array(), "Discover should work with Observe");

        // Broadcast should be denied (requires Interact)
        let req = json!({"jsonrpc": "2.0", "method": "event.broadcast", "id": "b1", "params": {"topic": "test", "payload": {}}});
        writer.write_all(format!("{}\n", req).as_bytes()).await.unwrap();
        let resp: serde_json::Value =
            serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
        assert_eq!(resp["error"]["code"], -32010, "Broadcast should be denied with Observe scope");

        tx.send(true).unwrap();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), hub_handle).await;
    }

    #[tokio::test]
    async fn hub_method_scope_mapping() {
        // Observe tier
        assert_eq!(hub_method_scope("session.discover"), PermissionScope::Observe);
        assert_eq!(hub_method_scope("event.collect"), PermissionScope::Observe);

        // Interact tier
        assert_eq!(hub_method_scope("event.broadcast"), PermissionScope::Interact);
        assert_eq!(hub_method_scope("session.register_remote"), PermissionScope::Interact);
        assert_eq!(hub_method_scope("session.heartbeat"), PermissionScope::Interact);
        assert_eq!(hub_method_scope("session.deregister_remote"), PermissionScope::Interact);

        // Forwarded methods use session auth model
        assert_eq!(hub_method_scope("termlink.ping"), PermissionScope::Observe);
        assert_eq!(hub_method_scope("command.execute"), PermissionScope::Execute);
        assert_eq!(hub_method_scope("command.inject"), PermissionScope::Control);
        assert_eq!(hub_method_scope("event.emit"), PermissionScope::Interact);

        // Unknown defaults to Execute
        assert_eq!(hub_method_scope("unknown.method"), PermissionScope::Execute);
    }

    // T-1633: volatile-runtime_dir startup warning truth table.
    // The impl function is pure (uid + path injected), so all four branches
    // are exercised without privilege escalation or env mutation race.
    // ENV_LOCK serializes the TERMLINK_RUNTIME_DIR read so concurrent tests
    // cannot race the env-var probe.
    mod runtime_dir_warn {
        use super::*;

        #[tokio::test]
        async fn warns_when_root_and_tmp_and_env_unset() {
            let _lock = ENV_LOCK.lock().await;
            let prev = std::env::var("TERMLINK_RUNTIME_DIR").ok();
            unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

            let fired = super::super::warn_if_volatile_default_runtime_dir_impl(
                0,
                PathBuf::from("/tmp/termlink-0"),
            );
            assert!(fired, "must warn: root + /tmp + no env");

            if let Some(v) = prev {
                unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", v) };
            }
        }

        #[tokio::test]
        async fn silent_when_non_root() {
            let _lock = ENV_LOCK.lock().await;
            let prev = std::env::var("TERMLINK_RUNTIME_DIR").ok();
            unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

            let fired = super::super::warn_if_volatile_default_runtime_dir_impl(
                1000,
                PathBuf::from("/tmp/termlink-1000"),
            );
            assert!(!fired, "non-root /tmp is the documented default, not a footgun");

            if let Some(v) = prev {
                unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", v) };
            }
        }

        #[tokio::test]
        async fn silent_when_env_set() {
            let _lock = ENV_LOCK.lock().await;
            let prev = std::env::var("TERMLINK_RUNTIME_DIR").ok();
            // Even when the env value itself points at /tmp, the operator
            // made a conscious choice — don't second-guess.
            unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", "/tmp/explicit") };

            let fired = super::super::warn_if_volatile_default_runtime_dir_impl(
                0,
                PathBuf::from("/tmp/explicit"),
            );
            assert!(!fired, "explicit env opt-out, no second-guessing");

            match prev {
                Some(v) => unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", v) },
                None => unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") },
            }
        }

        #[tokio::test]
        async fn silent_when_root_but_path_not_tmp() {
            let _lock = ENV_LOCK.lock().await;
            let prev = std::env::var("TERMLINK_RUNTIME_DIR").ok();
            unsafe { std::env::remove_var("TERMLINK_RUNTIME_DIR") };

            // Hypothetical: root + XDG_RUNTIME_DIR set, so resolved path is
            // outside /tmp. Not the footgun pattern.
            let fired = super::super::warn_if_volatile_default_runtime_dir_impl(
                0,
                PathBuf::from("/run/user/0/termlink"),
            );
            assert!(!fired, "non-/tmp resolution is not the PL-021 footgun");

            if let Some(v) = prev {
                unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", v) };
            }
        }
    }
}
