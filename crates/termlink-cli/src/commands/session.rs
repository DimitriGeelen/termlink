use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;

use termlink_session::client;
use termlink_session::data_server;
use termlink_session::handler::SessionContext;
use termlink_session::manager;
use termlink_session::pty::PtySession;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

use crate::util::{parse_signal, truncate};

use termlink_protocol::format_age;

/// Filter a list of session registrations by tag, name, role, and capability.
///
/// - `tag`: retain sessions with at least one matching tag (exact match)
/// - `name`: retain sessions whose display_name contains the substring (case-insensitive)
/// - `role`: retain sessions with at least one matching role (exact match)
/// - `cap`: retain sessions with at least one matching capability (exact match)
pub(crate) fn filter_sessions(
    mut sessions: Vec<termlink_session::registration::Registration>,
    tag: Option<&str>,
    name: Option<&str>,
    role: Option<&str>,
    cap: Option<&str>,
) -> Vec<termlink_session::registration::Registration> {
    if let Some(tag) = tag {
        sessions.retain(|s| s.tags.iter().any(|t| t == tag));
    }
    if let Some(name) = name {
        let name_lower = name.to_lowercase();
        sessions.retain(|s| s.display_name.to_lowercase().contains(&name_lower));
    }
    if let Some(role) = role {
        sessions.retain(|s| s.roles.iter().any(|r| r == role));
    }
    if let Some(cap) = cap {
        sessions.retain(|s| s.capabilities.iter().any(|c| c == cap));
    }
    sessions
}

/// Parse a `major.minor.patch` version into comparable numbers.
///
/// Returns `None` for anything that does not parse — callers decide what that
/// means rather than having a silent default chosen for them here.
///
/// Numeric, deliberately: `patch` is commits-since-tag in this project, so it
/// routinely reaches four digits. A lexical compare puts `0.11.9` *after*
/// `0.11.1346`, which inverts the answer on exactly the versions that occur most.
pub(crate) fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    // Tolerate a trailing pre-release/build suffix on the patch component
    // (`0.11.5-dirty`): take the leading digits and ignore the rest.
    let patch_raw = parts.next()?;
    let digits: String = patch_raw.chars().take_while(|c| c.is_ascii_digit()).collect();
    let patch = digits.parse().ok()?;
    Some((major, minor, patch))
}

/// Was this session registered by a binary older than `reference`?
///
/// **Absent or unparseable counts as stale.** T-2359 settled this for hubs — a
/// peer too old to report its version *is* the staleness class — and the
/// alternative is worse than it looks: treating "no version" as current would
/// pass precisely the oldest sessions, which are the population the check
/// exists to find. A version we cannot read is likewise not one we can vouch
/// for. Note that sessions registered before T-2744 carry a placeholder `0.9.0`
/// rather than their binary's real version.
///
/// **Lineage assumption.** `patch` is commits-since-tag, so this comparison is
/// meaningful only within one build lineage. That holds for sessions on a single
/// host, which is the scope of this command, but it is the same caveat
/// `.context/cron/fleet-version-floors.conf` carries for hubs — a larger patch
/// number across lineages does not mean "newer".
pub(crate) fn is_stale_binary(session_version: Option<&str>, reference: &str) -> bool {
    let Some(reference) = parse_version(reference) else {
        // We cannot say anything is stale relative to a reference we cannot
        // parse. Report nothing rather than everything.
        return false;
    };
    match session_version.and_then(parse_version) {
        Some(v) => v < reference,
        None => true,
    }
}

/// Retain only sessions registered by a binary older than `reference`.
pub(crate) fn filter_stale_binary(
    mut sessions: Vec<termlink_session::registration::Registration>,
    reference: &str,
) -> Vec<termlink_session::registration::Registration> {
    sessions.retain(|s| is_stale_binary(s.metadata.termlink_version.as_deref(), reference));
    sessions
}

/// Options for listing/filtering sessions.
pub(crate) struct ListFilterOpts<'a> {
    pub include_stale: bool,
    pub tag: Option<&'a str>,
    pub name: Option<&'a str>,
    pub role: Option<&'a str>,
    pub cap: Option<&'a str>,
    pub wait: bool,
    pub wait_timeout: u64,
    /// T-2745: retain only sessions registered by a binary older than this one.
    pub stale_binary: bool,
}

/// Options for session registration.
pub(crate) struct RegisterOpts {
    pub name: Option<String>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
    pub cap: Vec<String>,
    pub shell: bool,
    pub enable_token_secret: bool,
    pub allowed_commands: Vec<String>,
    pub json: bool,
    pub quiet: bool,
    /// T-1700 / T-1693 Shape 1: per-agent identity key file. When set, the
    /// session registers and signs envelopes with the key at this path
    /// instead of the host-shared default. Wired by exporting
    /// `TERMLINK_IDENTITY_FILE` for this process before
    /// `Session::register` so registration metadata + downstream
    /// `channel.post` signing both pick it up.
    pub identity_key: Option<std::path::PathBuf>,
}

/// T-2292: adopt a per-agent identity by DEFAULT when the session declares a
/// logical agent id (`TERMLINK_AGENT_ID`) and no explicit `--identity-key`
/// already pinned `TERMLINK_IDENTITY_FILE`. Resolves + creates a stable key at
/// `~/.termlink/identities/<agent_id>.key` and pins it via
/// `TERMLINK_IDENTITY_FILE` so the SessionMetadata fingerprint and downstream
/// `channel.post` signing both resolve to the same per-agent key. Co-resident
/// agents on a shared host therefore get DISTINCT fingerprints (RC1 of the
/// T-2291 reliable-comms inception) instead of collapsing onto one shared key.
///
/// No-op when `--identity-key` already set `TERMLINK_IDENTITY_FILE`, when
/// `TERMLINK_AGENT_ID` is unset/blank (single-agent host → unchanged shared
/// default), or when `HOME` is unresolvable. Non-fatal on key-create failure:
/// the resolver still attempts the per-agent path, so registration is never
/// blocked by an identity-binding hiccup.
fn bind_per_agent_identity_default(verbose: bool) {
    if std::env::var("TERMLINK_IDENTITY_FILE").is_ok() {
        return; // explicit --identity-key (or preset) wins.
    }
    let agent_id = match std::env::var("TERMLINK_AGENT_ID") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => return,
    };
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    use termlink_session::agent_identity::{Identity, per_agent_identity_path};
    let base = std::path::PathBuf::from(home).join(".termlink");
    let key_path = per_agent_identity_path(&base, &agent_id);
    let outcome = match Identity::load_or_create_from_file(&key_path) {
        Ok(ident) => {
            // SAFETY: same single-threaded startup reasoning as the
            // --identity-key path above — no task reads TERMLINK_IDENTITY_FILE
            // until Session::register / Endpoint::start runs sequentially below.
            unsafe {
                std::env::set_var("TERMLINK_IDENTITY_FILE", &key_path);
            }
            Ok(ident.fingerprint().to_string())
        }
        Err(e) => Err(e.to_string()),
    };
    let (info, warn) = bind_identity_messages(&agent_id, outcome, &key_path.display().to_string(), verbose);
    if let Some(line) = info {
        println!("{line}");
    }
    if let Some(line) = warn {
        eprintln!("{line}");
    }
}

/// Decide what to print after a per-agent identity bind attempt (T-2642).
///
/// Returns `(stdout_info, stderr_warning)`.
///
/// The bind-FAILURE warning is emitted **unconditionally** — a silent fallback
/// to the shared host identity mis-attributes the operator's posts, which is a
/// no-silent-failures (Directive #2) violation; the operator must see it even
/// under `--json` / `--quiet`. The warning goes to stderr, so it never corrupts
/// `--json` stdout. The SUCCESS info line is stdout and stays `verbose`-gated
/// (printing it under `--json` would corrupt the machine-readable stdout).
fn bind_identity_messages(
    agent_id: &str,
    outcome: Result<String, String>,
    key_path_display: &str,
    verbose: bool,
) -> (Option<String>, Option<String>) {
    match outcome {
        Ok(fingerprint) => {
            let info = if verbose {
                Some(format!(
                    "Identity (T-2292 per-agent '{agent_id}'): {key_path_display} ({fingerprint})"
                ))
            } else {
                None
            };
            (info, None)
        }
        Err(e) => {
            let warn = format!(
                "warning: could not bind per-agent identity for '{agent_id}': {e}; \
                 falling back to shared host default — your posts will be attributed to the \
                 shared HOST identity, not '{agent_id}'. Fix: ensure ~/.termlink is writable, \
                 or pass --identity-key <path> to bind an explicit key."
            );
            (None, Some(warn))
        }
    }
}

pub(crate) async fn cmd_register(opts: RegisterOpts) -> Result<()> {
    let RegisterOpts { name, roles, tags, cap, shell, enable_token_secret, allowed_commands, json, quiet, identity_key } = opts;
    let verbose = !json && !quiet;

    // T-1700: bind per-agent identity BEFORE Session::register so the
    // fingerprint baked into SessionMetadata (registration.rs T-1436 path)
    // and any later channel.post signing (channel.rs::load_identity_or_create)
    // resolve to the same key file. Creates the file at chmod 600 on first
    // use; failure here is fatal because the user explicitly asked for this
    // identity.
    if let Some(ref key_path) = identity_key {
        use termlink_session::agent_identity::Identity;
        let ident = Identity::load_or_create_from_file(key_path)
            .with_context(|| format!("Failed to load/create identity at {}", key_path.display()))?;
        // SAFETY: setting env vars after `main` has started is unsound in
        // multi-threaded programs that read env concurrently. At this point
        // the tokio runtime is up but no task reads TERMLINK_IDENTITY_FILE
        // yet (registration.rs reads it inside Session::register, called
        // sequentially below). Acceptable in this single-call path.
        unsafe {
            std::env::set_var("TERMLINK_IDENTITY_FILE", key_path);
        }
        if verbose {
            println!(
                "Identity (T-1700): {} ({})",
                key_path.display(),
                ident.fingerprint()
            );
        }
    }

    // T-2292: per-agent identity by default (no-op when --identity-key above
    // already pinned TERMLINK_IDENTITY_FILE, or when TERMLINK_AGENT_ID is unset).
    bind_per_agent_identity_default(verbose);

    let mut config = SessionConfig {
        display_name: name,
        roles,
        tags,
        capabilities: cap,
    };

    // Add data_plane capability when shell mode is enabled
    if shell {
        if !config.capabilities.contains(&"data_plane".to_string()) {
            config.capabilities.push("data_plane".into());
        }
        if !config.capabilities.contains(&"stream".to_string()) {
            config.capabilities.push("stream".into());
        }
    }

    let mut session = match termlink_session::Session::register(config).await {
        Ok(s) => s,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to register session: {}", e)}));
            }
            return Err(e).context("Failed to register session");
        }
    };

    // Enable token-based auth if requested
    let token_secret_hex = if enable_token_secret {
        let secret = termlink_session::auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
        session.registration.token_secret = Some(secret_hex.clone());
        if verbose {
            println!("Token auth enabled. Secret: {secret_hex}");
            println!("  Create tokens with: termlink token create {} --scope observe", session.id());
        }
        Some(secret_hex)
    } else {
        None
    };

    // Set command allowlist if specified
    if !allowed_commands.is_empty() {
        session.registration.allowed_commands = Some(allowed_commands.clone());
        if verbose {
            println!("Command allowlist: {:?}", allowed_commands);
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "id": session.id(),
            "display_name": session.display_name(),
            "socket_path": session.registration.socket_path().display().to_string(),
            "pid": std::process::id(),
            "shell": shell,
            "token_secret": token_secret_hex,
        }));
    } else if verbose {
        println!("Session registered:");
        println!("  ID:      {}", session.id());
        println!("  Name:    {}", session.display_name());
        println!("  Socket:  {}", session.registration.socket_path().display());
    }

    // Set up session context (with or without PTY)
    let pty_session = if shell {
        // Set data_socket metadata for discoverability
        let data_path = data_server::data_socket_path(session.registration.socket_path());
        session.registration.metadata.data_socket =
            Some(data_path.to_string_lossy().into_owned());

        // T-1302: seed TERMLINK_SESSION_ID into the spawned shell so `termlink whoami`
        // (and any tool inside the shell) can auto-resolve without a flag.
        let env = vec![("TERMLINK_SESSION_ID".to_string(), session.id().as_str().to_string())];
        let pty = match PtySession::spawn_with_env(None, 1024 * 1024, &env) {
            Ok(p) => p,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to spawn PTY session: {}", e)}));
                }
                return Err(e).context("Failed to spawn PTY session");
            }
        };
        if verbose {
            println!("  PTY:     yes (shell: {})",
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()));
        }
        Some(Arc::new(pty))
    } else {
        if verbose {
            println!("  PTY:     no (use --shell for bidirectional I/O)");
        }
        None
    };

    // Persist updated registration (capabilities + metadata + auth + allowlist)
    if (shell || enable_token_secret || session.registration.allowed_commands.is_some())
        && let Err(e) = session.persist_registration()
    {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to persist updated registration: {}", e)}));
        }
        return Err(e).context("Failed to persist updated registration");
    }

    if verbose {
        println!();
        println!("Listening for connections... (Ctrl+C to stop)");
    }

    let session_id = session.id().clone();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );

    let (registration, listener, _) = session.into_parts();
    let ctx = if let Some(ref pty) = pty_session {
        SessionContext::with_pty(registration.clone(), pty.clone())
            .with_registration_path(json_path)
    } else {
        SessionContext::new(registration.clone())
            .with_registration_path(json_path)
    };
    let shared = Arc::new(RwLock::new(ctx));

    let reg_for_cleanup = registration;

    // Compute data socket path before moving reg
    let data_socket_path = if shell {
        Some(data_server::data_socket_path(reg_for_cleanup.socket_path()))
    } else {
        None
    };

    let shared_clone = shared.clone();

    // If PTY, create broadcast channel and run read loop with broadcasting
    let pty_handle = if let Some(ref pty) = pty_session {
        let pty_clone = pty.clone();
        if let Some(ref data_path) = data_socket_path {
            // Shell mode: broadcast PTY output to data plane clients
            let (tx, rx) = tokio::sync::broadcast::channel::<Vec<u8>>(256);
            let data_pty = pty.clone();
            let data_path = data_path.clone();
            println!("  Data:    {}", data_path.display());

            // Start data plane server
            tokio::spawn(async move {
                if let Err(e) = data_server::run(&data_path, data_pty, rx).await {
                    tracing::error!(error = %e, "Data plane server error");
                }
            });

            // PTY read loop with broadcast. T-2439: surface loop death —
            // a silently-exited read loop means session output stops
            // flowing with no trace (sibling data_server spawn above
            // already logs).
            Some(tokio::spawn(async move {
                if let Err(e) = pty_clone.read_loop_with_broadcast(Some(tx)).await {
                    tracing::error!(error = %e, "PTY read loop (broadcast) exited with error — session output stopped");
                }
            }))
        } else {
            // No data plane — plain read loop (T-2439: same loud-exit contract).
            Some(tokio::spawn(async move {
                if let Err(e) = pty_clone.read_loop().await {
                    tracing::error!(error = %e, "PTY read loop exited with error — session output stopped");
                }
            }))
        }
    } else {
        None
    };

    // T-2230: periodic self-heartbeat. Before this fix `termlink register` set
    // heartbeat_at once at registration and never advanced it (touch_heartbeat
    // had zero production callers), so status showed the session frozen at
    // creation time forever — the symptom ring20 reported as "frozen husks"
    // surviving a hub restart (T-2229). The interval task advances heartbeat_at
    // in BOTH the in-memory registration (read by the query.status RPC) and the
    // on-disk JSON file (read by the hub directory sweep). Interval tunable via
    // TERMLINK_HEARTBEAT_INTERVAL_SECS (default 30s).
    let shared_hb = shared.clone();
    let hb_interval_secs = std::env::var("TERMLINK_HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(30);
    let heartbeat_task = tokio::spawn(async move {
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(hb_interval_secs));
        ticker.tick().await; // consume the immediate first tick
        loop {
            ticker.tick().await;
            let mut ctx = shared_hb.write().await;
            if let Some(path) = ctx.registration_path.clone()
                && let Err(e) = ctx.registration.touch_heartbeat(&path)
            {
                tracing::warn!(error = %e, "T-2230: heartbeat touch failed");
            }
        }
    });

    tokio::select! {
        _ = server::run_accept_loop(listener, shared_clone) => {}
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("Shutting down...");

            // Kill PTY child if running
            if let Some(ref pty) = pty_session {
                let _ = pty.signal(libc::SIGTERM);
            }
            if let Some(h) = pty_handle {
                h.abort();
            }

            // Clean up registration files
            let json_path = termlink_session::Registration::json_path(&sessions_dir, &session_id);
            let _ = std::fs::remove_file(reg_for_cleanup.socket_path());
            let _ = std::fs::remove_file(&json_path);

            // Clean up data socket if present
            if let Some(ref data_path) = data_socket_path {
                let _ = std::fs::remove_file(data_path);
            }

            println!("Session {} deregistered.", session_id);
        }
    }

    heartbeat_task.abort();
    Ok(())
}

pub(crate) async fn cmd_register_self(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    cap: Vec<String>,
    json: bool,
    identity_key: Option<std::path::PathBuf>,
) -> Result<()> {
    let verbose = !json;

    // T-1701: parity with cmd_register's --identity-key handling (T-1700).
    // Bind the per-agent identity BEFORE Endpoint::start so the event-only
    // session's SessionMetadata.identity_fingerprint and any subsequent
    // channel.post signing both pick up the override.
    if let Some(ref key_path) = identity_key {
        use termlink_session::agent_identity::Identity;
        let ident = Identity::load_or_create_from_file(key_path)
            .with_context(|| format!("Failed to load/create identity at {}", key_path.display()))?;
        // SAFETY: same single-threaded reasoning as cmd_register — no task
        // is reading TERMLINK_IDENTITY_FILE yet at this point in startup.
        unsafe {
            std::env::set_var("TERMLINK_IDENTITY_FILE", key_path);
        }
        if verbose {
            println!(
                "Identity (T-1701): {} ({})",
                key_path.display(),
                ident.fingerprint()
            );
        }
    }

    // T-2292: per-agent identity by default (parity with cmd_register).
    bind_per_agent_identity_default(verbose);

    let config = SessionConfig {
        display_name: name,
        roles,
        tags,
        capabilities: cap,
    };

    let endpoint = match termlink_session::endpoint::Endpoint::start(config).await {
        Ok(e) => e,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to register endpoint: {}", e)}));
            }
            return Err(e).context("Failed to register endpoint");
        }
    };

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "id": endpoint.id(),
            "display_name": endpoint.registration().display_name,
            "socket_path": endpoint.socket_path().display().to_string(),
            "pid": std::process::id(),
            "mode": "self",
        }));
    } else {
        println!("Endpoint registered (event-only, no PTY):");
        println!("  ID:      {}", endpoint.id());
        println!("  Socket:  {}", endpoint.socket_path().display());
        println!("  Capabilities: events, kv, status");
        println!();
        println!("Listening... (Ctrl+C to stop)");
    }

    endpoint.run_until_shutdown().await;

    if !json {
        println!("Endpoint deregistered.");
    }
    Ok(())
}

/// Sort a session list by the given key.
fn sort_sessions(sessions: &mut [termlink_session::registration::Registration], sort_key: &str) {
    match sort_key {
        "age" => sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        "age-desc" => sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        "name" => sessions.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase())),
        "name-desc" => sessions.sort_by(|a, b| b.display_name.to_lowercase().cmp(&a.display_name.to_lowercase())),
        "state" => sessions.sort_by(|a, b| format!("{}", a.state).cmp(&format!("{}", b.state))),
        _ => {} // unknown sort key — keep original order
    }
}

pub(crate) async fn cmd_list(filter: &ListFilterOpts<'_>, display: &super::ListDisplayOpts, sort_key: Option<&str>) -> Result<()> {
    let ListFilterOpts { include_stale, tag, name, role, cap, wait, wait_timeout, stale_binary } = *filter;
    // T-2745: the version this binary was built from — the same string
    // registration stamps into a session's metadata, so a session recording
    // something lower was registered by an older build.
    let reference_version = env!("CARGO_PKG_VERSION");
    let do_filter = |include_stale: bool| -> Result<Vec<termlink_session::registration::Registration>> {
        let sessions = manager::list_sessions(include_stale)
            .context("Failed to list sessions")?;
        let sessions = filter_sessions(sessions, tag, name, role, cap);
        Ok(if stale_binary {
            filter_stale_binary(sessions, reference_version)
        } else {
            sessions
        })
    };

    let sessions = if wait {
        let start = std::time::Instant::now();
        let timeout_dur = std::time::Duration::from_secs(wait_timeout);
        loop {
            let result = match do_filter(include_stale) {
                Ok(r) => r,
                Err(e) => {
                    if display.json {
                        super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to list sessions: {}", e)}));
                    }
                    return Err(e);
                }
            };
            if !result.is_empty() {
                break result;
            }
            if start.elapsed() > timeout_dur {
                if display.json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("No matching sessions found within {}s", wait_timeout)}));
                }
                anyhow::bail!("No matching sessions found within {}s", wait_timeout);
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    } else {
        match do_filter(include_stale) {
            Ok(r) => r,
            Err(e) => {
                if display.json {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to list sessions: {}", e)}));
                }
                return Err(e);
            }
        }
    };

    // Apply sorting if requested
    let mut sessions = sessions;
    if let Some(key) = sort_key {
        sort_sessions(&mut sessions, key);
    }

    if display.count {
        if display.json {
            println!("{}", serde_json::json!({"ok": true, "count": sessions.len()}));
        } else {
            println!("{}", sessions.len());
        }
        return Ok(());
    }

    if display.first {
        if let Some(s) = sessions.first() {
            if display.json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "id": s.id.as_str(),
                    "display_name": s.display_name,
                    "state": s.state.to_string(),
                    "pid": s.pid,
                    "uid": s.uid,
                    "created_at": s.created_at,
                    "heartbeat_at": s.heartbeat_at,
                    "tags": s.tags,
                    "roles": s.roles,
                    "capabilities": s.capabilities,
                    "metadata": s.metadata,
                    "socket_path": s.socket_path().display().to_string(),
                }));
            } else if display.ids {
                println!("{}", s.id.as_str());
            } else {
                println!("{}", s.display_name);
            }
        } else {
            if display.json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": "No matching sessions"}));
            }
            // T-2667: text mode must not exit(1) silently — a bare exit is
            // indistinguishable from a crash. Mirror the JSON branch with an
            // actionable stderr line (T-2663 remediation, ported from metadata.rs).
            eprintln!("No matching sessions.");
            std::process::exit(1);
        }
        return Ok(());
    }

    if display.names {
        if display.json {
            let items: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
            println!("{}", serde_json::json!({"ok": true, "names": items}));
        } else {
            for s in &sessions {
                println!("{}", s.display_name);
            }
        }
        return Ok(());
    }

    if display.ids {
        if display.json {
            let items: Vec<&str> = sessions.iter().map(|s| s.id.as_str()).collect();
            println!("{}", serde_json::json!({"ok": true, "ids": items}));
        } else {
            for s in &sessions {
                println!("{}", s.id.as_str());
            }
        }
        return Ok(());
    }

    if display.json {
        let items: Vec<serde_json::Value> = sessions.iter().map(|s| {
            serde_json::json!({
                "id": s.id.as_str(),
                "display_name": s.display_name,
                "state": s.state.to_string(),
                "pid": s.pid,
                "uid": s.uid,
                "created_at": s.created_at,
                "heartbeat_at": s.heartbeat_at,
                "age": format_age(&s.created_at),
                "tags": s.tags,
                "roles": s.roles,
                "capabilities": s.capabilities,
                "metadata": s.metadata,
                "socket_path": s.socket_path().display().to_string(),
            })
        }).collect();
        let mut envelope = serde_json::json!({"ok": true, "sessions": items});
        if stale_binary {
            // Additive only — the existing shape is untouched, so a consumer
            // that does not know about --stale-binary still parses this.
            envelope["reference_version"] = serde_json::json!(reference_version);
            envelope["stale_binary"] = serde_json::json!(true);
        }
        println!("{envelope}");
        return Ok(());
    }

    // T-2745: --stale-binary answers a different question than the session
    // table does, so it gets its own rendering rather than a column bolted onto
    // a shared table. Every line names the version that made the session stale
    // next to the version it is being compared against — the operator should
    // not need a second command to know what to do.
    if stale_binary {
        if sessions.is_empty() {
            if !display.no_header {
                // "All current" and "there is nothing here" are different
                // answers, and only one of them is reassuring. Saying "every
                // session was registered by X or newer" when there are no
                // sessions at all is vacuously true and reads as a clean bill
                // of health for a fleet that does not exist.
                let total = manager::list_sessions(include_stale).map(|s| s.len()).unwrap_or(0);
                if total == 0 {
                    println!("No sessions registered — nothing to check.");
                } else {
                    println!(
                        "No stale-binary sessions — all {total} were registered by {reference_version} or newer."
                    );
                }
            }
            return Ok(());
        }
        if !display.no_header {
            println!(
                "Sessions registered by a binary older than {reference_version} \
                 (patch is commits-since-tag; comparable within one build lineage):"
            );
        }
        for session in &sessions {
            let recorded = session
                .metadata
                .termlink_version
                .as_deref()
                .unwrap_or("unrecorded");
            println!(
                "{:<14} {:<16} registered by {}",
                session.id.as_str(),
                truncate(&session.display_name, 15),
                recorded
            );
        }
        return Ok(());
    }

    if sessions.is_empty() {
        if !display.no_header {
            println!("No active sessions.");
        }
        return Ok(());
    }

    if !display.no_header {
        println!(
            "{:<14} {:<16} {:<6} {:<6} {:<8} TAGS",
            "ID", "NAME", "STATE", "AGE", "PID"
        );
        println!("{}", "-".repeat(70));
    }

    for session in &sessions {
        let tags = if session.tags.is_empty() {
            String::new()
        } else {
            session.tags.join(",")
        };
        let age = format_age(&session.created_at);
        println!(
            "{:<14} {:<16} {:<6} {:<6} {:<8} {}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            age,
            session.pid,
            tags,
        );
    }

    if !display.no_header {
        println!();
        println!("{} session(s)", sessions.len());
    }
    Ok(())
}

pub(crate) fn cmd_clean(dry_run: bool, json: bool, no_header: bool, count: bool) -> Result<()> {
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let stale = match manager::clean_stale_sessions(&sessions_dir, !dry_run) {
        Ok(s) => s,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to scan for stale sessions: {}", e)}));
            }
            return Err(e).context("Failed to scan for stale sessions");
        }
    };

    if json {
        let items: Vec<serde_json::Value> = stale.iter().map(|s| {
            serde_json::json!({
                "id": s.id,
                "display_name": s.display_name,
                "pid": s.pid,
                "created_at": s.created_at,
            })
        }).collect();
        println!("{}", serde_json::json!({
            "ok": true,
            "dry_run": dry_run,
            "action": if dry_run { "would_remove" } else { "removed" },
            "count": stale.len(),
            "sessions": items,
        }));
        return Ok(());
    }

    if count {
        println!("{}", stale.len());
        return Ok(());
    }

    if stale.is_empty() {
        println!("No stale sessions found.");
        return Ok(());
    }

    let action = if dry_run { "Would remove" } else { "Removed" };

    if !no_header {
        println!(
            "{:<14} {:<16} {:<8} CREATED",
            "ID", "NAME", "PID"
        );
        println!("{}", "-".repeat(54));
    }

    for s in &stale {
        println!(
            "{:<14} {:<16} {:<8} {}",
            &s.id[..s.id.len().min(13)],
            truncate(&s.display_name, 15),
            s.pid,
            &s.created_at[..s.created_at.len().min(19)],
        );
    }

    if !no_header {
        println!();
        println!("{} {} stale session(s).", action, stale.len());
    }
    Ok(())
}

pub(crate) async fn cmd_ping(
    opts: &crate::target::TargetOpts,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let target = opts.session.as_str();
    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    let call_future = crate::target::call_session(opts, "termlink.ping", serde_json::json!({}));
    let result: Result<serde_json::Value> = match tokio::time::timeout(timeout_dur, call_future).await
    {
        Ok(r) => r,
        Err(_) => {
            let latency_ms = start.elapsed().as_millis();
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Ping timed out after {}s", timeout_secs),
                    "timeout_ms": timeout_secs * 1000,
                    "latency_ms": latency_ms,
                }));
            }
            anyhow::bail!("Ping timed out after {}s", timeout_secs);
        }
    };
    let latency_ms = start.elapsed().as_millis();

    match result {
        Ok(result) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "target": target,
                        "id": result["id"],
                        "display_name": result["display_name"],
                        "state": result["state"],
                        "latency_ms": latency_ms,
                    })
                );
            } else {
                println!(
                    "PONG from {} ({}) — state: {}, latency: {}ms",
                    result["id"].as_str().unwrap_or("?"),
                    result["display_name"].as_str().unwrap_or("?"),
                    result["state"].as_str().unwrap_or("?"),
                    latency_ms,
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
            }
            anyhow::bail!("Ping failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_status(
    opts: &crate::target::TargetOpts,
    json: bool,
    short: bool,
    timeout_secs: u64,
) -> Result<()> {
    let target = opts.session.as_str();
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    let call_future = crate::target::call_session(opts, "query.status", serde_json::json!({}));
    let outcome: Result<serde_json::Value> = match tokio::time::timeout(timeout_dur, call_future).await {
        Ok(r) => r,
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Status query timed out after {}s", timeout_secs),
                }));
            }
            anyhow::bail!("Status query timed out after {}s", timeout_secs);
        }
    };

    match outcome {
        Ok(result) => {
            if json {
                let mut wrapped = serde_json::json!({"ok": true});
                if let Some(obj) = result.as_object() {
                    for (k, v) in obj {
                        wrapped[k] = v.clone();
                    }
                }
                println!("{}", wrapped);
                return Ok(());
            }
            if short {
                println!("{} {} {}",
                    result["display_name"].as_str().unwrap_or("?"),
                    result["state"].as_str().unwrap_or("?"),
                    result["pid"],
                );
                return Ok(());
            }
            println!("Session: {}", result["id"].as_str().unwrap_or("?"));
            println!("  Name:        {}", result["display_name"].as_str().unwrap_or("?"));
            println!("  State:       {}", result["state"].as_str().unwrap_or("?"));
            println!("  PID:         {}", result["pid"]);
            println!("  Created:     {}", result["created_at"].as_str().unwrap_or("?"));
            println!("  Heartbeat:   {}", result["heartbeat_at"].as_str().unwrap_or("?"));
            if let Some(caps) = result.get("capabilities").and_then(|c| c.as_array()) {
                let cap_strs: Vec<&str> = caps.iter().filter_map(|c| c.as_str()).collect();
                println!("  Capabilities: {}", cap_strs.join(", "));
            }
            if let Some(tags) = result.get("tags").and_then(|t| t.as_array())
                && !tags.is_empty() {
                    let tag_strs: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                    println!("  Tags:        {}", tag_strs.join(", "));
                }
            if let Some(roles) = result.get("roles").and_then(|r| r.as_array())
                && !roles.is_empty() {
                    let role_strs: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
                    println!("  Roles:       {}", role_strs.join(", "));
                }
            if let Some(mode) = result.get("terminal_mode") {
                let canonical = mode["canonical"].as_bool().unwrap_or(false);
                let echo = mode["echo"].as_bool().unwrap_or(false);
                let raw = mode["raw"].as_bool().unwrap_or(false);
                let alt_screen = mode["alternate_screen"].as_bool().unwrap_or(false);
                let mode_label = if raw {
                    "raw"
                } else if canonical && echo {
                    "canonical+echo"
                } else if canonical {
                    "canonical"
                } else {
                    "cooked"
                };
                print!("  Term Mode:   {}", mode_label);
                if alt_screen {
                    print!(" (alternate screen)");
                }
                println!();
            }
            if let Some(meta) = result.get("metadata") {
                if let Some(shell) = meta.get("shell").and_then(|s| s.as_str()) {
                    println!("  Shell:       {}", shell);
                }
                if let Some(term) = meta.get("term").and_then(|s| s.as_str()) {
                    println!("  Terminal:    {}", term);
                }
                if let Some(cwd) = meta.get("cwd").and_then(|s| s.as_str()) {
                    println!("  CWD:         {}", cwd);
                }
                if let Some(ds) = meta.get("data_socket").and_then(|s| s.as_str()) {
                    println!("  Data plane:  {}", ds);
                }
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Status query failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_exec(target: &str, command: &str, cwd: Option<&str>, timeout: u64, json: bool) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {} — run 'termlink list-sessions' to see registered sessions", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found — run 'termlink list-sessions' to see registered sessions", target));
        }
    };

    let mut params = serde_json::json!({
        "command": command,
        "timeout": timeout,
    });
    if let Some(dir) = cwd {
        params["cwd"] = serde_json::json!(dir);
    }

    // RPC timeout = command timeout + 5s buffer for connection/response overhead
    let rpc_timeout = std::time::Duration::from_secs(timeout + 5);
    let rpc_future = client::rpc_call(reg.socket_path(), "command.execute", params);
    let resp = match tokio::time::timeout(rpc_timeout, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {} — the session may have exited; run 'termlink clean' to remove stale registrations, or 'termlink list-sessions' to check", e)}));
                }
                return Err(e).context("Failed to connect to session — the session may have exited; run 'termlink clean' to remove stale registrations, or 'termlink list-sessions' to check");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Exec RPC timed out after {}s", timeout + 5)}));
            }
            anyhow::bail!("Exec RPC timed out after {}s (command timeout: {}s)", timeout + 5, timeout);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                // T-2491: fail CLOSED on a missing exit_code (shared helper),
                // matching the text branch below and push.rs::exec_rpc.
                let (wrapped, exit_code) = super::exec_json_envelope(&result);
                println!("{}", wrapped);
                if exit_code != 0 {
                    std::process::exit(exit_code as i32);
                }
                return Ok(());
            }

            let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
            let stdout = result["stdout"].as_str().unwrap_or("");
            let stderr = result["stderr"].as_str().unwrap_or("");

            if !stdout.is_empty() {
                print!("{stdout}");
            }
            if !stderr.is_empty() {
                eprint!("{stderr}");
            }

            if exit_code != 0 {
                std::process::exit(exit_code as i32);
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Execution failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_send(target: &str, method: &str, params_str: &str, json: bool, timeout_secs: u64) -> Result<()> {
    let params: serde_json::Value = match serde_json::from_str(params_str) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Invalid JSON params: {}", e)}));
            }
            return Err(e.into());
        }
    };

    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {} — run 'termlink list-sessions' to see registered sessions", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found — run 'termlink list-sessions' to see registered sessions", target));
        }
    };

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), method, params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {} — the session may have exited; run 'termlink clean' to remove stale registrations, or 'termlink list-sessions' to check", e)}));
                }
                return Err(e).context("Failed to connect to session — the session may have exited; run 'termlink clean' to remove stale registrations, or 'termlink list-sessions' to check");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "method": method,
                    "error": {"code": -1, "message": format!("Timed out after {}s", timeout_secs)},
                }));
            }
            anyhow::bail!("RPC call timed out after {}s", timeout_secs);
        }
    };

    match resp {
        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "method": method,
                    "result": r.result,
                }));
            } else {
                println!("{}", serde_json::to_string_pretty(&r.result)?);
            }
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "method": method,
                    "error": {
                        "code": e.error.code,
                        "message": e.error.message,
                        "data": e.error.data,
                    },
                }));
            } else {
                eprintln!("Error {}: {}", e.error.code, e.error.message);
                if let Some(data) = &e.error.data {
                    eprintln!("{}", serde_json::to_string_pretty(data)?);
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

pub(crate) async fn cmd_signal(
    opts: &crate::target::TargetOpts,
    signal: &str,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let target = opts.session.as_str();

    let sig_num = match parse_signal(signal) {
        Some(n) => n,
        None => {
            let msg = format!(
                "Unknown signal: '{}'. Use TERM, INT, KILL, HUP, USR1, USR2, or a number.",
                signal
            );
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false, "target": target, "error": msg
                }));
            }
            anyhow::bail!("{}", msg);
        }
    };

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let call_future = crate::target::call_session(
        opts,
        "command.signal",
        serde_json::json!({ "signal": sig_num }),
    );
    let outcome: Result<serde_json::Value> =
        match tokio::time::timeout(timeout_dur, call_future).await {
            Ok(r) => r,
            Err(_) => {
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false,
                        "target": target,
                        "error": format!("Signal timed out after {}s", timeout_secs),
                    }));
                }
                anyhow::bail!("Signal timed out after {}s", timeout_secs);
            }
        };

    match outcome {
        Ok(result) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "target": target,
                    "signal": result["signal"],
                    "pid": result["pid"],
                }));
            } else {
                println!(
                    "Signal {} sent to PID {}",
                    result["signal"].as_i64().unwrap_or(sig_num as i64),
                    result["pid"].as_u64().unwrap_or(0),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
            }
            anyhow::bail!("Signal failed: {}", e);
        }
    }
}

pub(crate) fn cmd_info(json: bool, short: bool, check: bool) -> Result<()> {
    let runtime_dir = termlink_session::discovery::runtime_dir();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
    let hub_running = hub_socket.exists();
    let live = manager::list_sessions(false)
        .map(|s| s.len())
        .unwrap_or(0);
    let all = manager::list_sessions(true)
        .map(|s| s.len())
        .unwrap_or(0);
    let stale = all - live;

    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let target = option_env!("BUILD_TARGET").unwrap_or("unknown");

    if short {
        let hub_status = if hub_running { "running" } else { "stopped" };
        println!("termlink {version} sessions:{live}/{all} hub:{hub_status}");
        if check && (!hub_running || stale > 0) {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(1);
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "version": version,
            "commit": commit,
            "target": target,
            "runtime_dir": runtime_dir.to_string_lossy(),
            "sessions_dir": sessions_dir.to_string_lossy(),
            "hub_socket": hub_socket.to_string_lossy(),
            "hub_running": hub_running,
            "sessions": {
                "live": live,
                "stale": stale,
                "total": all,
            },
        }))?);
        if check && (!hub_running || stale > 0) {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("TermLink Runtime");
    println!("{}", "-".repeat(40));
    println!("  Version:      {version} ({commit}) [{target}]");
    println!("  Runtime dir:  {}", runtime_dir.display());
    println!("  Sessions dir: {}", sessions_dir.display());
    println!("  Hub socket:   {}", hub_socket.display());
    println!(
        "  Hub:          {}",
        if hub_running { "running" } else { "stopped" }
    );

    println!();
    println!("Sessions");
    println!("{}", "-".repeat(40));
    println!("  Live:   {}", live);
    println!("  Stale:  {}", stale);
    println!("  Total:  {}", all);

    if stale > 0 {
        println!();
        println!("  Tip: run 'termlink clean' to remove stale sessions");
    }

    if check && (!hub_running || stale > 0) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use termlink_protocol::TransportAddr;
    use termlink_session::identity::SessionId;
    use termlink_session::lifecycle::SessionState;
    use termlink_session::registration::{Registration, RegistrationAddr, SessionMetadata};

    // T-2642: the bind-failure warning must fire regardless of verbosity —
    // silencing it under --json/--quiet mis-attributes the operator's posts.
    #[test]
    fn bind_identity_failure_warns_even_when_not_verbose() {
        // The load-bearing case: verbose=false (i.e. --json or --quiet).
        let (info, warn) =
            bind_identity_messages("agent-x", Err::<String, String>("perm denied".to_string()), "/p/key", false);
        assert!(info.is_none(), "no stdout info on failure (would corrupt --json)");
        let w = warn.expect("failure warning must be Some even when not verbose");
        assert!(w.contains("agent-x"), "warning names the agent");
        assert!(w.contains("perm denied"), "warning carries the underlying error");
        // Actionable remediation tokens (Directive #3).
        assert!(w.contains("--identity-key"), "warning names the explicit-key remedy");
        assert!(
            w.contains("shared HOST identity"),
            "warning names the mis-attribution consequence"
        );
    }

    #[test]
    fn bind_identity_failure_warns_when_verbose_too() {
        let (info, warn) =
            bind_identity_messages("agent-y", Err::<String, String>("io".to_string()), "/p/key", true);
        assert!(info.is_none());
        assert!(warn.is_some(), "failure warning fires under verbose as well");
    }

    #[test]
    fn bind_identity_success_info_is_verbose_gated() {
        // Success info goes to stdout — must be suppressed under --json/--quiet.
        let (info_quiet, warn_quiet) =
            bind_identity_messages("agent-z", Ok("fp123".to_string()), "/p/key", false);
        assert!(info_quiet.is_none(), "success info suppressed when not verbose");
        assert!(warn_quiet.is_none(), "no warning on success");

        let (info_v, warn_v) =
            bind_identity_messages("agent-z", Ok("fp123".to_string()), "/p/key", true);
        let i = info_v.expect("success info present when verbose");
        assert!(i.contains("agent-z") && i.contains("fp123"));
        assert!(warn_v.is_none());
    }

    fn test_reg(name: &str, tags: Vec<&str>, roles: Vec<&str>, caps: Vec<&str>) -> Registration {
        Registration {
            version: 1,
            id: SessionId::generate(),
            display_name: name.to_string(),
            pid: 1000,
            uid: 1000,
            addr: RegistrationAddr(TransportAddr::Unix { path: "/tmp/test.sock".into() }),
            created_at: "2026-01-01T00:00:00Z".into(),
            heartbeat_at: "2026-01-01T00:00:00Z".into(),
            state: SessionState::Ready,
            capabilities: caps.into_iter().map(String::from).collect(),
            roles: roles.into_iter().map(String::from).collect(),
            tags: tags.into_iter().map(String::from).collect(),
            metadata: SessionMetadata::default(),
            token_secret: None,
            allowed_commands: None,
        }
    }

    fn sample_sessions() -> Vec<Registration> {
        vec![
            test_reg("worker-1", vec!["prod", "gpu"], vec!["compute"], vec!["execute"]),
            test_reg("worker-2", vec!["prod"], vec!["compute", "storage"], vec!["execute", "read"]),
            test_reg("Agent Alpha", vec!["staging"], vec!["agent"], vec!["execute"]),
            test_reg("monitor", vec!["prod"], vec!["observer"], vec!["read"]),
        ]
    }

    #[test]
    fn filter_no_filters_returns_all() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, None, None, None, None);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn filter_by_tag_exact_match() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, Some("gpu"), None, None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].display_name, "worker-1");
    }

    #[test]
    fn filter_by_tag_multiple_matches() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, Some("prod"), None, None, None);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn filter_by_name_case_insensitive() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, None, Some("agent"), None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].display_name, "Agent Alpha");
    }

    #[test]
    fn filter_by_name_substring() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, None, Some("worker"), None, None);
        assert_eq!(result.len(), 2);
    }

    // ── T-2745: stale-binary detection ──────────────────────────────────────

    #[test]
    fn version_parses_into_comparable_numbers() {
        assert_eq!(parse_version("0.11.1346"), Some((0, 11, 1346)));
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        // A pre-release/build suffix on patch is tolerated.
        assert_eq!(parse_version("0.11.5-dirty"), Some((0, 11, 5)));
        // Not versions.
        assert_eq!(parse_version("0.11"), None);
        assert_eq!(parse_version("banana"), None);
        assert_eq!(parse_version(""), None);
    }

    // The trap this helper exists to avoid. `patch` is commits-since-tag here,
    // so four-digit values are routine and a lexical compare inverts the answer
    // on exactly the versions that occur most.
    #[test]
    fn comparison_is_numeric_not_lexical() {
        assert!(
            parse_version("0.11.9").unwrap() < parse_version("0.11.1346").unwrap(),
            "0.11.9 must order before 0.11.1346"
        );
        // Confirms the two orderings genuinely disagree, so the test above is
        // testing something.
        assert!("0.11.9" > "0.11.1346", "string compare is expected to disagree");

        assert!(is_stale_binary(Some("0.11.9"), "0.11.1346"));
        assert!(!is_stale_binary(Some("0.11.1346"), "0.11.9"));
    }

    #[test]
    fn missing_or_unreadable_version_counts_as_stale() {
        assert!(is_stale_binary(None, "0.11.1346"));
        assert!(is_stale_binary(Some(""), "0.11.1346"));
        assert!(is_stale_binary(Some("banana"), "0.11.1346"));
        // Pre-T-2744 sessions carry this placeholder.
        assert!(is_stale_binary(Some("0.9.0"), "0.11.1346"));
    }

    // PL-219: the filter has to be able to *not* fire, or it is not a filter.
    #[test]
    fn current_and_newer_sessions_are_not_stale() {
        assert!(!is_stale_binary(Some("0.11.1346"), "0.11.1346"), "equal is not stale");
        assert!(!is_stale_binary(Some("0.11.1400"), "0.11.1346"), "newer is not stale");
        assert!(!is_stale_binary(Some("1.0.0"), "0.11.1346"), "major bump is not stale");
    }

    // If we cannot read our own version we cannot call anything stale relative
    // to it. Reporting nothing beats reporting everything.
    #[test]
    fn unparseable_reference_reports_nothing_rather_than_everything() {
        assert!(!is_stale_binary(Some("0.1.0"), "not-a-version"));
        assert!(!is_stale_binary(None, "not-a-version"));
    }

    #[test]
    fn filter_stale_binary_retains_only_older_sessions() {
        let mut old = test_reg("old", vec![], vec![], vec![]);
        old.metadata.termlink_version = Some("0.11.9".into());
        let mut current = test_reg("current", vec![], vec![], vec![]);
        current.metadata.termlink_version = Some("0.11.1346".into());
        let mut unrecorded = test_reg("unrecorded", vec![], vec![], vec![]);
        unrecorded.metadata.termlink_version = None;

        let kept = filter_stale_binary(vec![old, current, unrecorded], "0.11.1346");
        let names: Vec<&str> = kept.iter().map(|s| s.display_name.as_str()).collect();
        assert_eq!(names, vec!["old", "unrecorded"]);
    }

    #[test]
    fn filter_stale_binary_returns_empty_when_every_session_is_current() {
        let mut a = test_reg("a", vec![], vec![], vec![]);
        a.metadata.termlink_version = Some("0.11.1346".into());
        let mut b = test_reg("b", vec![], vec![], vec![]);
        b.metadata.termlink_version = Some("0.11.2000".into());

        assert!(filter_stale_binary(vec![a, b], "0.11.1346").is_empty());
    }

    #[test]
    fn filter_by_role() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, None, None, Some("compute"), None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_capability() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, None, None, None, Some("read"));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_combined_tag_and_role() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, Some("prod"), None, Some("observer"), None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].display_name, "monitor");
    }

    #[test]
    fn filter_combined_no_match() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, Some("staging"), None, Some("compute"), None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn filter_empty_input() {
        let result = filter_sessions(vec![], Some("prod"), Some("test"), Some("agent"), Some("execute"));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn filter_nonexistent_tag() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, Some("nonexistent"), None, None, None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn filter_name_empty_string_matches_all() {
        let sessions = sample_sessions();
        let result = filter_sessions(sessions, None, Some(""), None, None);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn sort_sessions_by_name() {
        let mut sessions = sample_sessions();
        sort_sessions(&mut sessions, "name");
        let names: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
        assert_eq!(names, vec!["Agent Alpha", "monitor", "worker-1", "worker-2"]);
    }

    #[test]
    fn sort_sessions_by_name_desc() {
        let mut sessions = sample_sessions();
        sort_sessions(&mut sessions, "name-desc");
        let names: Vec<&str> = sessions.iter().map(|s| s.display_name.as_str()).collect();
        assert_eq!(names, vec!["worker-2", "worker-1", "monitor", "Agent Alpha"]);
    }

    #[test]
    fn sort_sessions_unknown_key() {
        let mut sessions = sample_sessions();
        let original_names: Vec<String> = sessions.iter().map(|s| s.display_name.clone()).collect();
        sort_sessions(&mut sessions, "nonexistent");
        let after_names: Vec<String> = sessions.iter().map(|s| s.display_name.clone()).collect();
        assert_eq!(original_names, after_names, "unknown sort key should preserve order");
    }

    // format_age tests moved to termlink-protocol crate (T-874)
}
