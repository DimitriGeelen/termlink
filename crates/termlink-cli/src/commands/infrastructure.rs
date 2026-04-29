use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::json;

/// T-1031: Resolve the hub pidfile path, checking the default runtime dir first
/// and falling back to /var/lib/termlink/ (systemd-managed hubs).
/// Returns (pidfile_path, socket_path) from whichever dir has a running hub.
pub(crate) fn resolve_hub_paths() -> (PathBuf, PathBuf) {
    let default_pidfile = termlink_hub::pidfile::hub_pidfile_path();
    let default_socket = termlink_hub::server::hub_socket_path();

    // Check default runtime dir first
    if matches!(
        termlink_hub::pidfile::check(&default_pidfile),
        termlink_hub::pidfile::PidfileStatus::Running(_) | termlink_hub::pidfile::PidfileStatus::Stale(_)
    ) {
        return (default_pidfile, default_socket);
    }

    // Fallback: check /var/lib/termlink/ (systemd-managed hubs).
    // Only do this if TERMLINK_RUNTIME_DIR is not explicitly set (tests set
    // it to isolated temp dirs and should never discover the real hub).
    if std::env::var("TERMLINK_RUNTIME_DIR").is_err() {
        let alt_dir = PathBuf::from("/var/lib/termlink");
        let alt_pidfile = alt_dir.join("hub.pid");
        if alt_pidfile.exists() {
            let alt_socket = alt_dir.join("hub.sock");
            return (alt_pidfile, alt_socket);
        }
    }

    // Nothing found — return defaults
    (default_pidfile, default_socket)
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(_) => {
                tokio::signal::ctrl_c().await.ok();
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
    }
}

pub(crate) async fn cmd_hub_start(tcp_addr: Option<&str>, json_output: bool) -> Result<()> {
    let socket_path = termlink_hub::server::hub_socket_path();
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

    if !json_output {
        println!("Starting hub server...");
        println!("  Socket:  {}", socket_path.display());
        if let Some(addr) = tcp_addr {
            println!("  TCP:     {}", addr);
        }
        println!("  Pidfile: {}", pidfile_path.display());
    }

    let handle = termlink_hub::server::run_with_tcp(&socket_path, tcp_addr)
        .await
        .context("Hub server error")?;

    // T-1026: hub.tcp is now written by the server after bind (server.rs)

    if tcp_addr.is_some() {
        let secret_path = termlink_hub::server::hub_secret_path();
        let cert_path = termlink_hub::tls::hub_cert_path();
        if json_output {
            println!("{}", json!({
                "ok": true,
                "pid": std::process::id(),
                "socket": socket_path.display().to_string(),
                "pidfile": pidfile_path.display().to_string(),
                "tcp": tcp_addr,
                "secret_file": secret_path.display().to_string(),
                "tls_cert": cert_path.display().to_string(),
            }));
        } else {
            println!("  Secret:  {}", secret_path.display());
            println!("  TLS cert: {}", cert_path.display());
            println!();
            println!("TCP connections use TLS with auto-generated self-signed certificate.");
            println!("Auth required. Clients must call 'hub.auth' with a token.");
            println!("Read the secret: cat {}", secret_path.display());
        }
    } else if json_output {
        println!("{}", json!({
            "ok": true,
            "pid": std::process::id(),
            "socket": socket_path.display().to_string(),
            "pidfile": pidfile_path.display().to_string(),
        }));
    }

    if !json_output {
        println!();
        println!("Listening for connections... (Ctrl+C or SIGTERM to stop)");
    }

    wait_for_shutdown_signal().await;

    if !json_output {
        println!();
        println!("Shutting down hub...");
    }
    handle.shutdown();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    if !json_output {
        println!("Hub stopped.");
    }

    Ok(())
}

pub(crate) async fn cmd_doctor(json_output: bool, fix: bool, strict: bool) -> Result<()> {
    use termlink_session::{client, discovery, liveness, manager};

    let mut checks: Vec<serde_json::Value> = Vec::new();
    let mut pass_count = 0u32;
    let mut warn_count = 0u32;
    let mut fail_count = 0u32;

    macro_rules! check {
        ($name:expr, pass, $msg:expr) => {{
            pass_count += 1;
            checks.push(json!({"check": $name, "status": "pass", "message": $msg}));
            if !json_output { println!("  \x1b[32m✓\x1b[0m {}: {}", $name, $msg); }
        }};
        ($name:expr, warn, $msg:expr) => {{
            warn_count += 1;
            checks.push(json!({"check": $name, "status": "warn", "message": $msg}));
            if !json_output { println!("  \x1b[33m!\x1b[0m {}: {}", $name, $msg); }
        }};
        ($name:expr, fail, $msg:expr) => {{
            fail_count += 1;
            checks.push(json!({"check": $name, "status": "fail", "message": $msg}));
            if !json_output { println!("  \x1b[31m✗\x1b[0m {}: {}", $name, $msg); }
        }};
    }

    if !json_output {
        println!("TermLink Doctor");
        println!("===============\n");
    }

    // 1. Runtime directory
    let runtime_dir = discovery::runtime_dir();
    if runtime_dir.exists() {
        check!("runtime_dir", pass, format!("{}", runtime_dir.display()));
    } else {
        check!("runtime_dir", fail, format!("{} does not exist", runtime_dir.display()));
    }

    // 2. Sessions directory
    let sessions_dir = discovery::sessions_dir();
    if sessions_dir.exists() {
        check!("sessions_dir", pass, format!("{}", sessions_dir.display()));
    } else {
        check!("sessions_dir", warn, format!("{} does not exist (no sessions registered yet)", sessions_dir.display()));
    }

    // 3. Session health
    let sessions = manager::list_sessions(true).unwrap_or_default();
    let total = sessions.len();
    let mut alive = 0u32;
    let mut dead = 0u32;
    let mut stale_sockets: Vec<String> = Vec::new();

    let ping_timeout = std::time::Duration::from_secs(3);
    for s in &sessions {
        if liveness::process_exists(s.pid) {
            // Try actual ping with timeout to avoid hanging on dead sockets
            let rpc_future = client::rpc_call(s.socket_path(), "termlink.ping", json!({}));
            match tokio::time::timeout(ping_timeout, rpc_future).await {
                Ok(Ok(_)) => alive += 1,
                Ok(Err(_)) | Err(_) => {
                    dead += 1;
                    stale_sockets.push(s.display_name.clone());
                }
            }
        } else {
            dead += 1;
            stale_sockets.push(s.display_name.clone());
        }
    }

    if total == 0 {
        check!("sessions", pass, "no sessions registered");
    } else if dead == 0 {
        check!("sessions", pass, format!("{total} registered, all responding"));
    } else {
        check!("sessions", warn, format!("{total} registered, {alive} alive, {dead} dead/stale"));
        for name in &stale_sockets {
            if !json_output {
                println!("      stale: {name}");
            }
        }
        if fix {
            let cleaned = manager::clean_stale_sessions(&sessions_dir, true)
                .unwrap_or_default();
            if !json_output && !cleaned.is_empty() {
                println!("      \x1b[32mfixed:\x1b[0m removed {} stale session(s)", cleaned.len());
            }
        } else if !stale_sockets.is_empty() && !json_output {
            println!("      Run 'termlink doctor --fix' to auto-clean");
        }
    }

    // 4. Hub status — T-1030/T-1031: use resolve_hub_paths() to find the hub
    //    regardless of whether it's in default runtime_dir or /var/lib/termlink.
    let (pidfile_path, hub_socket) = resolve_hub_paths();
    let alt_dir = pidfile_path.parent() != Some(termlink_session::discovery::runtime_dir().as_path());
    let suffix = if alt_dir { format!(" (via {})", pidfile_path.parent().unwrap().display()) } else { String::new() };
    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            let hub_rpc = client::rpc_call(&hub_socket, "termlink.ping", json!({}));
            match tokio::time::timeout(ping_timeout, hub_rpc).await {
                Ok(Ok(_)) => check!("hub", pass, format!("running (PID {pid}), responding{suffix}")),
                Ok(Err(_)) | Err(_) => check!("hub", warn, format!("running (PID {pid}), but not responding on socket{suffix}")),
            }
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            if fix {
                termlink_hub::pidfile::remove(&pidfile_path);
                let _ = std::fs::remove_file(&hub_socket);
                check!("hub", warn, format!("stale pidfile (PID {pid}) — fixed: removed pidfile and socket{suffix}"));
            } else {
                check!("hub", warn, format!("stale pidfile (PID {pid} is dead). Run 'termlink doctor --fix' to clean up"));
            }
        }
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            check!("hub", pass, "not running (optional — needed for multi-session routing)");
        }
    }

    // 4b. UFW rule vs. TCP listener consistency (T-934)
    //
    // Catches the exact state that triggered T-930: a firewall rule on
    // the hub's TCP port exists but nothing is bound to it, so cross-host
    // callers get connection-refused with no actionable signal. Parse
    // `ufw status` for any rule whose comment mentions "termlink" (case-
    // insensitive) and extract the port. If no corresponding listener is
    // found via `ss -tln`, emit a warn check.
    //
    // The whole block is best-effort: ufw unavailable or unreadable is a
    // skipped check, not a warning. That keeps the signal high for
    // environments actually using ufw + termlink together.
    {
        let ufw_output = std::process::Command::new("ufw")
            .arg("status")
            .output();
        if let Ok(out) = ufw_output
            && out.status.success()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut termlink_ports: Vec<u16> = Vec::new();
            for line in stdout.lines() {
                if !line.to_lowercase().contains("termlink") {
                    continue;
                }
                // Format: "9100/tcp  ALLOW  192.168.10.0/24  # TermLink..."
                if let Some(first) = line.split_whitespace().next()
                    && let Some(port_str) = first.strip_suffix("/tcp")
                    && let Ok(port) = port_str.parse::<u16>()
                {
                    termlink_ports.push(port);
                }
            }
            if !termlink_ports.is_empty() {
                let ss_output = std::process::Command::new("ss")
                    .args(["-tln"])
                    .output()
                    .ok();
                let listening = ss_output
                    .as_ref()
                    .and_then(|o| {
                        if o.status.success() {
                            Some(String::from_utf8_lossy(&o.stdout).to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                let mut missing: Vec<u16> = Vec::new();
                for port in &termlink_ports {
                    let needle = format!(":{port} ");
                    if !listening.contains(&needle) {
                        missing.push(*port);
                    }
                }
                if missing.is_empty() {
                    check!(
                        "ufw_listener",
                        pass,
                        format!(
                            "ufw allows {} — listener present",
                            termlink_ports
                                .iter()
                                .map(|p| format!("{p}/tcp"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    );
                } else {
                    check!(
                        "ufw_listener",
                        warn,
                        format!(
                            "ufw allows {} but nothing is listening — run 'termlink hub start --tcp 0.0.0.0:{}' or start termlink-hub.service",
                            missing
                                .iter()
                                .map(|p| format!("{p}/tcp"))
                                .collect::<Vec<_>>()
                                .join(", "),
                            missing[0]
                        )
                    );
                }
            }
        }
    }

    // 5. Orphaned sockets (sockets without matching registration JSON)
    if sessions_dir.exists() {
        let mut orphan_count = 0u32;
        let mut orphan_paths: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && ext == "sock" {
                        let json_path = path.with_extension("json");
                        if !json_path.exists() {
                            orphan_count += 1;
                            orphan_paths.push(path);
                        }
                    }
            }
        }
        if orphan_count > 0 {
            if fix {
                for p in &orphan_paths {
                    let _ = std::fs::remove_file(p);
                    // Also remove .sock.data if it exists
                    let data_path = p.with_extension("sock.data");
                    let _ = std::fs::remove_file(&data_path);
                }
                check!("sockets", warn, format!("{orphan_count} orphaned socket(s) — fixed: removed"));
            } else {
                check!("sockets", warn, format!("{orphan_count} orphaned socket(s) without registration"));
            }
        } else {
            check!("sockets", pass, "no orphaned sockets");
        }
    }

    // 6. Dispatch manifest
    {
        let project_root = std::env::current_dir().unwrap_or_default();
        let manifest = crate::manifest::DispatchManifest::load(&project_root);
        match manifest {
            Ok(m) => {
                let pending = m.pending_dispatches();
                if pending.is_empty() {
                    if m.dispatches.is_empty() {
                        check!("dispatch", pass, "no dispatch manifest");
                    } else {
                        check!("dispatch", pass, format!("{} dispatch(es), none pending", m.dispatches.len()));
                    }
                } else {
                    let ids: Vec<&str> = pending.iter().map(|d| d.id.as_str()).collect();
                    check!("dispatch", warn, format!(
                        "{} pending dispatch(es): {}",
                        pending.len(),
                        ids.join(", ")
                    ));
                    if fix {
                        // Expire dispatches older than 24 hours
                        let mut m = m;
                        let cutoff_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .saturating_sub(86400);
                        let cutoff = crate::manifest::secs_to_rfc3339(cutoff_secs);
                        let mut expired_count = 0u32;
                        for d in &mut m.dispatches {
                            if d.status == crate::manifest::DispatchStatus::Pending
                                && d.created_at < cutoff
                            {
                                d.status = crate::manifest::DispatchStatus::Expired;
                                expired_count += 1;
                            }
                        }
                        if expired_count > 0 {
                            if let Err(e) = m.save(&project_root) {
                                if !json_output {
                                    println!("      \x1b[31merror:\x1b[0m failed to save manifest: {e}");
                                }
                            } else if !json_output {
                                println!("      \x1b[32mfixed:\x1b[0m expired {expired_count} dispatch(es) older than 24h");
                            }
                        } else if !json_output {
                            println!("      no dispatches old enough to auto-expire (< 24h)");
                        }
                    } else if !json_output {
                        println!("      Run 'termlink doctor --fix' to expire stale dispatches");
                        println!("      Or  'termlink dispatch-status' for details");
                    }
                }
            }
            Err(e) => {
                check!("dispatch", warn, format!("failed to read manifest: {e}"));
            }
        }
    }

    // 7. Inbox status (T-1001).
    //
    // T-1400: prefer `channel.list(prefix="inbox:")` over the legacy
    // `inbox.status` RPC. The channel-aware path uses the same data the
    // hub-side migration shim already mirrors transfers into, and avoids
    // contributing to the T-1166 retirement-gate legacy traffic. On any
    // failure (including MethodNotFound on older hubs), fall back to the
    // legacy probe so the doctor remains useful across version skew.
    if hub_socket.exists() {
        let probe_channel_list = async {
            let resp = termlink_session::client::rpc_call(
                &hub_socket,
                "channel.list",
                json!({"prefix": "inbox:"}),
            )
            .await
            .map_err(|e| format!("channel.list transport: {e}"))?;
            let result = termlink_session::client::unwrap_result(resp)?;
            let topics = result["topics"].as_array().cloned().unwrap_or_default();
            let target_count = topics.len();
            let total: u64 = topics
                .iter()
                .filter_map(|t| t["count"].as_u64())
                .sum();
            Ok::<(u64, usize), String>((total, target_count))
        };

        let probe_inbox_status = async {
            let resp = termlink_session::client::rpc_call(
                &hub_socket,
                "inbox.status",
                json!({}),
            )
            .await
            .map_err(|e| format!("inbox.status transport: {e}"))?;
            let result = termlink_session::client::unwrap_result(resp)?;
            Ok::<(u64, usize), String>((
                result["total_transfers"].as_u64().unwrap_or(0),
                result["targets"].as_array().map(|t| t.len()).unwrap_or(0),
            ))
        };

        let outcome: Result<(u64, usize), String> = match probe_channel_list.await {
            Ok(v) => Ok(v),
            Err(_) => probe_inbox_status.await,
        };

        match outcome {
            Ok((0, _)) => check!("inbox", pass, "no pending transfers"),
            Ok((total, targets)) => {
                check!("inbox", warn, format!("{total} pending transfer(s) for {targets} target(s)"))
            }
            Err(e) => check!("inbox", warn, format!("inbox query failed: {e}")),
        }
    }

    // 7b. T-1171 / G-011: Client-side secret cache audit.
    //
    //     ~/.termlink/secrets/<host>.hex is a cache of the shared hub.secret.
    //     When the hub restarts with a new secret (or the runtime_dir migrates)
    //     the cache silently diverges and the next auth fails. Two surface
    //     signals, both low-cost to compute locally:
    //       (a) perms must be 0600 — world-readable caches leaked the G-011
    //           smell where proxmox4.hex was 644.
    //       (b) if this host runs a local hub, any cache file older than the
    //           live hub.secret is a drift candidate. The operator confirms
    //           whether the cache actually points at the local hub; we only
    //           surface the age signal, we don't auto-heal.
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let secrets_dir = PathBuf::from(&home).join(".termlink").join("secrets");
        if !secrets_dir.exists() {
            check!("secret_cache", pass, "no cached secrets");
        } else {
            let local_hub_secret: Option<(PathBuf, std::time::SystemTime)> = {
                let (pidfile, _) = resolve_hub_paths();
                pidfile.parent().and_then(|dir| {
                    let p = dir.join("hub.secret");
                    std::fs::metadata(&p)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| (p, t))
                })
            };
            let issues = audit_secret_cache(
                &secrets_dir,
                local_hub_secret.as_ref().map(|(p, t)| (p.as_path(), *t)),
            );
            if issues.is_empty() {
                check!("secret_cache", pass, "all cached secrets look healthy");
            } else {
                for msg in &issues {
                    check!("secret_cache", warn, msg.clone());
                }
            }
        }

        // T-1284 / G-011: profile audit — purely lexical, runs even when
        // secrets_dir doesn't exist (profile may reference a path that
        // hasn't been populated yet). Flags self-hub profiles using the
        // IP-keyed cache as secret_file; structural fix is to point
        // secret_file at the live <runtime_dir>/hub.secret.
        let hubs_config = crate::config::load_hubs_config();
        let profile_hints = audit_hubs_for_self_hub_cache(&hubs_config, &secrets_dir);
        if profile_hints.is_empty() {
            check!(
                "secret_cache_profiles",
                pass,
                "no self-hub profiles using IP-keyed cache"
            );
        } else {
            for hint in &profile_hints {
                check!("secret_cache_profiles", warn, hint.clone());
            }
        }
    }

    // 8. Version + MCP tools
    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let mcp_tools = termlink_mcp::tool_count();
    check!("version", pass, format!("termlink {version} ({commit}), {mcp_tools} MCP tools"));

    // Summary
    if json_output {
        let result = json!({
            "ok": fail_count == 0,
            "checks": checks,
            "summary": {
                "pass": pass_count,
                "warn": warn_count,
                "fail": fail_count,
            }
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!();
        if fail_count > 0 {
            println!("\x1b[31m{fail_count} failed\x1b[0m, {warn_count} warnings, {pass_count} passed");
        } else if warn_count > 0 {
            println!("{warn_count} warnings, \x1b[32m{pass_count} passed\x1b[0m");
        } else {
            println!("\x1b[32mAll {pass_count} checks passed\x1b[0m");
        }
    }

    if fail_count > 0 || (strict && warn_count > 0) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }
    Ok(())
}

pub(crate) fn cmd_hub_stop(json: bool) -> Result<()> {
    let (pidfile_path, socket_path) = resolve_hub_paths();

    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            if json {
                println!("{}", serde_json::json!({"ok": true, "action": "none", "reason": "Hub is not running"}));
            } else {
                println!("Hub is not running.");
            }
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            termlink_hub::pidfile::remove(&pidfile_path);
            let _ = std::fs::remove_file(&socket_path);
            if json {
                println!("{}", serde_json::json!({"ok": true, "action": "cleaned", "pid": pid, "reason": "Stale pidfile removed"}));
            } else {
                println!("Hub pidfile found (PID {pid}) but process is dead. Cleaning up.");
            }
        }
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            if !json {
                println!("Stopping hub (PID {pid})...");
            }
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if !termlink_session::liveness::process_exists(pid) {
                    if json {
                        println!("{}", serde_json::json!({"ok": true, "action": "stopped", "pid": pid}));
                    } else {
                        println!("Hub stopped.");
                    }
                    return Ok(());
                }
            }
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "action": "timeout", "pid": pid, "error": format!("Hub did not stop within 2 seconds. You may need to kill -9 {pid}.")}));
            } else {
                println!("Hub did not stop within 2 seconds. You may need to kill -9 {pid}.");
            }
        }
    }
    Ok(())
}

pub(crate) fn cmd_hub_restart(json: bool) -> Result<()> {
    let (pidfile_path, _) = resolve_hub_paths();

    // Find current hub PID
    let old_pid = match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::Running(pid) => pid,
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            if json {
                println!("{}", json!({"ok": false, "error": format!("Hub PID {} is stale (dead)", pid)}));
            } else {
                println!("Hub PID {} is stale (dead). Use 'termlink hub start' instead.", pid);
            }
            return Ok(());
        }
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            if json {
                println!("{}", json!({"ok": false, "error": "Hub is not running"}));
            } else {
                println!("Hub is not running. Use 'termlink hub start' to start it.");
            }
            return Ok(());
        }
    };

    // Determine TCP address from existing hub config (use resolved dir, not default)
    let runtime_dir = pidfile_path.parent().unwrap_or(std::path::Path::new("/tmp"));
    let tcp_flag_path = runtime_dir.join("hub.tcp");
    let tcp_addr = std::fs::read_to_string(&tcp_flag_path)
        .ok()
        .map(|s| s.trim().to_string());

    if !json {
        println!("Restarting hub (PID {})...", old_pid);
    }

    // Find our own binary path
    let self_exe = std::env::current_exe().context("Cannot determine own binary path")?;

    // Build the hub start command
    let mut cmd = std::process::Command::new(&self_exe);
    cmd.arg("hub").arg("start");
    if let Some(ref addr) = tcp_addr {
        cmd.arg("--tcp").arg(addr);
    }

    // T-1031: If the old hub used a non-default runtime dir (e.g., systemd's
    // /var/lib/termlink), pass it to the new process so it writes to the same
    // location. Without this, the new hub defaults to /tmp/termlink-0/ and
    // generates a different secret, breaking auth for all remote clients.
    let default_runtime = termlink_session::discovery::runtime_dir();
    if pidfile_path.parent().is_some_and(|d| d != default_runtime.as_path()) {
        cmd.env("TERMLINK_RUNTIME_DIR", pidfile_path.parent().unwrap());
    }

    // Detach the child process so it outlives us
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    // Stop the old hub first
    if !json {
        println!("  Stopping old hub (PID {})...", old_pid);
    }
    unsafe { libc::kill(old_pid as i32, libc::SIGTERM) };

    // Wait for old hub to die (up to 3s)
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !termlink_session::liveness::process_exists(old_pid) {
            break;
        }
    }

    if termlink_session::liveness::process_exists(old_pid) {
        if json {
            println!("{}", json!({"ok": false, "error": format!("Old hub (PID {}) did not stop within 3s", old_pid)}));
        } else {
            println!("  Old hub (PID {}) did not stop. Aborting restart.", old_pid);
        }
        return Ok(());
    }

    // Start new hub
    if !json {
        if let Some(ref addr) = tcp_addr {
            println!("  Starting new hub with TCP on {}...", addr);
        } else {
            println!("  Starting new hub (Unix socket only)...");
        }
    }

    match cmd.spawn() {
        Ok(child) => {
            let new_pid = child.id();

            // Wait briefly for new hub to bind
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Verify new hub is running (check same dir as old hub)
            let running = matches!(
                termlink_hub::pidfile::check(&pidfile_path),
                termlink_hub::pidfile::PidfileStatus::Running(_)
            );

            if json {
                println!("{}", json!({
                    "ok": running,
                    "old_pid": old_pid,
                    "new_pid": new_pid,
                    "tcp": tcp_addr,
                }));
            } else if running {
                println!("  Hub restarted successfully (PID {} → {})", old_pid, new_pid);
            } else {
                println!("  Hub spawned (PID {}) but not yet responding. Check 'termlink hub status'.", new_pid);
            }
        }
        Err(e) => {
            if json {
                super::json_error_exit(json!({"ok": false, "error": format!("Failed to spawn new hub: {}", e)}));
            } else {
                println!("  Failed to start new hub: {}", e);
            }
        }
    }

    Ok(())
}

pub(crate) fn cmd_hub_status(json_output: bool, short: bool, check: bool) -> Result<()> {
    // T-1032: Use resolve_hub_paths() for split-brain runtime dir detection
    let (pidfile_path, socket_path) = resolve_hub_paths();

    let is_running = matches!(
        termlink_hub::pidfile::check(&pidfile_path),
        termlink_hub::pidfile::PidfileStatus::Running(_)
    );

    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            if json_output {
                println!("{}", json!({"ok": true, "status": "not_running"}));
            } else if short {
                println!("not_running");
            } else {
                println!("Hub: not running");
            }
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            if json_output {
                println!("{}", json!({"ok": true, "status": "stale", "pid": pid}));
            } else if short {
                println!("stale {pid}");
            } else {
                println!("Hub: stale (PID {pid} is dead, pidfile needs cleanup)");
                println!("  Run 'termlink hub stop' to clean up.");
            }
        }
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            let runtime_dir = pidfile_path.parent().map(|p| p.display().to_string()).unwrap_or_default();
            if json_output {
                println!("{}", json!({
                    "ok": true,
                    "status": "running",
                    "pid": pid,
                    "socket": socket_path.display().to_string(),
                    "pidfile": pidfile_path.display().to_string(),
                    "runtime_dir": runtime_dir,
                }));
            } else if short {
                println!("running {pid}");
            } else {
                println!("Hub: running (PID {pid})");
                println!("  Runtime dir: {}", runtime_dir);
                println!("  Socket: {}", socket_path.display());
                println!("  Pidfile: {}", pidfile_path.display());
            }
        }
    }

    if check && !is_running {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }
    Ok(())
}

// === Inbox Commands (T-997) ===

pub(crate) async fn cmd_inbox_status(json_output: bool) -> Result<()> {
    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running (no socket at {})", hub_socket.display());
    }

    let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
    let cache = termlink_session::hub_capabilities::shared_cache();
    let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
    let status = termlink_session::inbox_channel::status_with_fallback(&addr, cache, &mut ctx)
        .await
        .context("Failed to query inbox status from hub")?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else if status.total_transfers == 0 {
        println!("Inbox: empty (no pending transfers)");
    } else {
        println!("Inbox: {} pending transfer(s)", status.total_transfers);
        println!();
        for t in &status.targets {
            println!("  {}: {} transfer(s)", t.target, t.pending);
        }
    }
    Ok(())
}

pub(crate) async fn cmd_inbox_clear(target: Option<&str>, all: bool, json_output: bool) -> Result<()> {
    if target.is_none() && !all {
        anyhow::bail!("Specify a target session name, or use --all to clear everything");
    }

    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running (no socket at {})", hub_socket.display());
    }

    let scope = if all {
        termlink_session::inbox_channel::ClearScope::All
    } else {
        termlink_session::inbox_channel::ClearScope::Target(target.unwrap().to_string())
    };

    let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
    let cache = termlink_session::hub_capabilities::shared_cache();
    let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
    let result = termlink_session::inbox_channel::clear_with_fallback(&addr, scope, cache, &mut ctx)
        .await
        .context("Failed to clear inbox via hub")?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if result.cleared == 0 {
        println!("No transfers to clear for '{}'", result.target);
    } else {
        println!("Cleared {} transfer(s) for '{}'", result.cleared, result.target);
    }
    Ok(())
}

pub(crate) async fn cmd_inbox_list(target: &str, json_output: bool) -> Result<()> {
    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running (no socket at {})", hub_socket.display());
    }

    let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
    let cache = termlink_session::hub_capabilities::shared_cache();
    let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
    let entries = termlink_session::inbox_channel::list_with_fallback(&addr, target, cache, &mut ctx)
        .await
        .context("Failed to query inbox from hub")?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&json!({ "transfers": entries }))?);
    } else if entries.is_empty() {
        println!("No pending transfers for '{target}'");
    } else {
        println!("{} pending transfer(s) for '{target}':", entries.len());
        println!();
        for tr in &entries {
            let status = if tr.complete { "complete" } else { "partial" };
            println!(
                "  {}  {} ({} bytes, {})",
                tr.transfer_id, tr.filename, tr.size, status
            );
        }
    }
    Ok(())
}

// === TOFU Commands (T-1035) ===

pub(crate) fn cmd_tofu_list(json_output: bool) -> Result<()> {
    let store = termlink_session::tofu::KnownHubStore::default_store();
    let entries = store.list_all();

    if json_output {
        let items: Vec<serde_json::Value> = entries.iter().map(|e| {
            json!({
                "host": e.host_port,
                "fingerprint": e.fingerprint,
                "first_seen": e.first_seen,
                "last_seen": e.last_seen,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&json!({
            "ok": true,
            "count": items.len(),
            "entries": items,
        }))?);
    } else if entries.is_empty() {
        println!("No trusted hubs (TOFU store is empty)");
        println!("  File: {}", termlink_session::tofu::known_hubs_path().display());
    } else {
        println!("Trusted hubs ({} entries):", entries.len());
        println!("  File: {}", termlink_session::tofu::known_hubs_path().display());
        println!();
        println!("{:<30} {:<20} {:<22} LAST SEEN", "HOST", "FINGERPRINT", "FIRST SEEN");
        println!("{}", "-".repeat(95));
        for e in &entries {
            let fp_short = if e.fingerprint.len() > 18 {
                format!("{}...", &e.fingerprint[..18])
            } else {
                e.fingerprint.clone()
            };
            println!("{:<30} {:<20} {:<22} {}", e.host_port, fp_short, e.first_seen, e.last_seen);
        }
    }
    Ok(())
}

pub(crate) fn cmd_tofu_clear(host: Option<&str>, all: bool, json_output: bool) -> Result<()> {
    let store = termlink_session::tofu::KnownHubStore::default_store();

    if all {
        let count = store.clear_all();
        if json_output {
            println!("{}", json!({"ok": true, "cleared": count}));
        } else {
            println!("Cleared {} TOFU entries", count);
        }
    } else if let Some(host_port) = host {
        let existed = store.remove(host_port);
        if json_output {
            println!("{}", json!({"ok": existed, "host": host_port, "removed": existed}));
        } else if existed {
            println!("Removed TOFU entry for {}", host_port);
            println!("  Next connection will re-trust (TOFU)");
        } else {
            println!("No TOFU entry found for '{}'", host_port);
            println!("  Known entries:");
            for e in store.list_all() {
                println!("    {}", e.host_port);
            }
        }
    } else {
        anyhow::bail!("Specify a host:port to clear, or use --all to clear everything");
    }
    Ok(())
}

/// T-1171 / G-011: Audit `~/.termlink/secrets/*.hex` for perm smells and
/// staleness relative to a local hub secret. Returns a warning message per
/// issue; empty vec means all caches look healthy.
///
/// - Skips non-regular files, entries whose name doesn't end in `.hex`, and
///   `.bak` siblings.
/// - Perm check: any mode != 0o600 (low 9 bits) is flagged.
/// - Freshness check: only runs when `local_hub` is provided. A cache with
///   `mtime < hub_mtime` is flagged — the operator decides whether the
///   cache actually points at the local hub.
pub(crate) fn audit_secret_cache(
    secrets_dir: &std::path::Path,
    local_hub: Option<(&std::path::Path, std::time::SystemTime)>,
) -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;
    let mut issues = Vec::new();
    let entries = match std::fs::read_dir(secrets_dir) {
        Ok(e) => e,
        Err(_) => return issues,
    };
    // T-1284: read the local hub.secret value once so we can compare each
    // cache by VALUE, not just by mtime. Value comparison eliminates the
    // mtime-based false-positive ("cache for remote hub X is older than
    // local hub.secret"); a cache that matches the local secret is
    // healthy regardless of mtime, and a cache that diverges + is older
    // is a real drift candidate (not a heuristic guess).
    let hub_value: Option<String> = local_hub.and_then(|(p, _)| {
        std::fs::read_to_string(p)
            .ok()
            .map(|s| s.trim().to_lowercase())
    });
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with(".hex") || name.ends_with(".bak") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_file() {
            continue;
        }
        let mode = meta.permissions().mode() & 0o777;
        if mode != 0o600 {
            issues.push(format!(
                "{} has mode {:o} (expected 600) — world/group-readable cache",
                path.display(),
                mode
            ));
        }
        if let Some((hub_path, hub_mtime)) = local_hub {
            // Try value comparison first — definitive when readable.
            let cache_value = std::fs::read_to_string(&path)
                .ok()
                .map(|s| s.trim().to_lowercase());
            match (cache_value.as_deref(), hub_value.as_deref()) {
                (Some(c), Some(h)) if c == h => {
                    // Match → cache IS the local hub's secret. Healthy
                    // regardless of mtime ordering. Skip the mtime check.
                    continue;
                }
                (Some(_), Some(_)) => {
                    // Diverges. Only flag when mtime suggests this cache
                    // PREDATES the current hub.secret — that's the real
                    // "stale cache for the local hub" signal. A diverging
                    // cache that's NEWER than hub.secret is almost
                    // certainly a cache of a remote hub, not drift.
                    if let Ok(cache_mtime) = meta.modified()
                        && cache_mtime < hub_mtime
                    {
                        issues.push(format!(
                            "{} cache value diverges from local hub.secret AND is older than {} — drift candidate; if this cache points at the local hub, refresh it",
                            path.display(),
                            hub_path.display()
                        ));
                    }
                    continue;
                }
                _ => {
                    // Value compare unavailable — fall back to mtime-only.
                }
            }
            if let Ok(cache_mtime) = meta.modified()
                && cache_mtime < hub_mtime
            {
                issues.push(format!(
                    "{} is older than local {} — may be stale if this cache points at the local hub",
                    path.display(),
                    hub_path.display()
                ));
            }
        }
    }
    issues
}

/// T-1284 / G-011: Audit `hubs.toml` for profiles that use an IP-keyed
/// cache file (under `secrets_dir`) for what is structurally a local-hub
/// address. Such profiles are the giving-end of cache drift: the cache
/// is written once and silently goes stale on hub restart. Per
/// CLAUDE.md R3 the fix is to point `secret_file` directly at the live
/// `<runtime_dir>/hub.secret`.
///
/// Returns one migration hint per offending profile. Empty vec means
/// no profiles need migration. The check is purely lexical/structural —
/// it does not contact any hub. Address forms recognised as "self":
/// `127.x.x.x`, `localhost`, `::1`, `0.0.0.0`. Anything else is treated
/// as remote (where IP-keyed cache is appropriate).
pub(crate) fn audit_hubs_for_self_hub_cache(
    config: &crate::config::HubsConfig,
    secrets_dir: &std::path::Path,
) -> Vec<String> {
    let mut hints = Vec::new();
    // Sort for deterministic output across runs.
    let mut names: Vec<&String> = config.hubs.keys().collect();
    names.sort();
    for name in names {
        let entry = &config.hubs[name];
        let Some(secret_file) = entry.secret_file.as_deref() else {
            continue;
        };
        let secret_path = std::path::PathBuf::from(secret_file);
        if !secret_path.starts_with(secrets_dir) {
            continue;
        }
        if !is_self_hub_address(&entry.address) {
            continue;
        }
        hints.push(format!(
            "Profile '{name}' uses IP-keyed cache ({sf}) for the local hub at {addr}. Per G-011, point secret_file directly at <runtime_dir>/hub.secret to avoid drift.",
            name = name,
            sf = secret_file,
            addr = entry.address,
        ));
    }
    hints
}

/// True if `address` (as found in hubs.toml — typically `host:port`)
/// names a loopback / wildcard interface. Strips the port if present;
/// any parse failure yields `false` (treat as remote, fail-safe).
fn is_self_hub_address(address: &str) -> bool {
    let host = match address.rsplit_once(':') {
        Some((h, _)) => h,
        None => address,
    }
    .trim_start_matches('[')
    .trim_end_matches(']');
    if host == "localhost" || host == "::1" || host == "0.0.0.0" {
        return true;
    }
    if let Some(rest) = host.strip_prefix("127.") {
        // Any 127.x.y.z is loopback. Coarse check is sufficient — even
        // garbage-after-127. would only false-positive a profile the
        // operator clearly meant as local.
        return rest.split('.').count() == 3;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::audit_secret_cache;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn tmpdir(label: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!(
            "termlink-audit-test-{}-{}",
            label,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn write_hex(dir: &std::path::Path, name: &str, mode: u32) -> std::path::PathBuf {
        let p = dir.join(name);
        fs::write(&p, b"deadbeef").unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(mode)).unwrap();
        p
    }

    #[test]
    fn missing_dir_is_empty() {
        let missing = std::env::temp_dir().join("termlink-audit-test-nonexistent-xyz");
        let _ = fs::remove_dir_all(&missing);
        assert!(audit_secret_cache(&missing, None).is_empty());
    }

    #[test]
    fn good_perms_no_local_hub_is_empty() {
        let d = tmpdir("good");
        write_hex(&d, "ring20.hex", 0o600);
        assert!(audit_secret_cache(&d, None).is_empty());
    }

    #[test]
    fn bad_perms_reported() {
        let d = tmpdir("bad-perms");
        write_hex(&d, "proxmox4.hex", 0o644);
        let issues = audit_secret_cache(&d, None);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("mode 644"));
        assert!(issues[0].contains("proxmox4.hex"));
    }

    #[test]
    fn bak_siblings_skipped() {
        let d = tmpdir("bak");
        write_hex(&d, "ring20.hex.bak", 0o644); // deliberately bad perms
        assert!(
            audit_secret_cache(&d, None).is_empty(),
            ".bak siblings must not be flagged"
        );
    }

    #[test]
    fn stale_cache_reported_against_local_hub() {
        let d = tmpdir("stale");
        let cache = write_hex(&d, "ring20.hex", 0o600);
        // Backdate cache mtime by 1h using std's File::set_modified.
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(&cache)
            .unwrap()
            .set_modified(past)
            .unwrap();
        let hub_secret = d.join("hub.secret");
        fs::write(&hub_secret, b"ff").unwrap();
        let issues = audit_secret_cache(
            &d,
            Some((hub_secret.as_path(), std::time::SystemTime::now())),
        );
        assert_eq!(issues.len(), 1);
        // T-1284: with value-comparison enabled, the message now leads
        // with "diverges from local hub.secret" but still mentions
        // "older than" since both signals fire here.
        assert!(issues[0].contains("older than"), "got: {}", issues[0]);
    }

    #[test]
    fn cache_value_matches_hub_skips_mtime_warning() {
        // T-1284: a cache whose hex value MATCHES the local hub.secret
        // is healthy regardless of mtime ordering. Pre-T-1284 mtime-only
        // logic would have falsely flagged this as "older than local".
        let d = tmpdir("value-match");
        let cache = write_hex(&d, "ring20.hex", 0o600);
        // Backdate cache mtime to 1h ago.
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(&cache)
            .unwrap()
            .set_modified(past)
            .unwrap();
        // hub.secret has the same value as the cache (write_hex writes "deadbeef").
        let hub_secret = d.join("hub.secret");
        fs::write(&hub_secret, b"deadbeef").unwrap();
        let issues = audit_secret_cache(
            &d,
            Some((hub_secret.as_path(), std::time::SystemTime::now())),
        );
        assert!(
            issues.is_empty(),
            "matching value should not flag drift; got: {:?}",
            issues
        );
    }

    #[test]
    fn cache_value_diverges_and_older_uses_diverges_wording() {
        // T-1284: when cache value differs AND mtime predates hub.secret,
        // the message must include "diverges from local hub.secret" so
        // the operator distinguishes real drift from the legacy mtime
        // false-positive.
        let d = tmpdir("value-diverge");
        let cache = write_hex(&d, "stalehub.hex", 0o600);
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(&cache)
            .unwrap()
            .set_modified(past)
            .unwrap();
        // hub.secret value differs from cache ("deadbeef" vs "cafebabe").
        let hub_secret = d.join("hub.secret");
        fs::write(&hub_secret, b"cafebabe").unwrap();
        let issues = audit_secret_cache(
            &d,
            Some((hub_secret.as_path(), std::time::SystemTime::now())),
        );
        assert_eq!(issues.len(), 1, "got: {:?}", issues);
        assert!(
            issues[0].contains("diverges from local hub.secret"),
            "expected new wording; got: {}",
            issues[0]
        );
    }

    #[test]
    fn cache_diverging_but_newer_is_not_flagged() {
        // T-1284: a cache whose value differs from local hub.secret BUT
        // is newer is almost certainly a remote-hub cache, not drift.
        // Don't flag.
        let d = tmpdir("value-newer-remote");
        let _ = write_hex(&d, "remotehub.hex", 0o600);
        // hub.secret backdated → cache (touched at write_hex time) is newer.
        let hub_secret = d.join("hub.secret");
        fs::write(&hub_secret, b"different").unwrap();
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        fs::File::options()
            .write(true)
            .open(&hub_secret)
            .unwrap()
            .set_modified(past)
            .unwrap();
        let issues = audit_secret_cache(
            &d,
            Some((hub_secret.as_path(), past)),
        );
        assert!(
            issues.is_empty(),
            "newer-than-hub diverging cache should not flag; got: {:?}",
            issues
        );
    }

    #[test]
    fn audit_hubs_flags_loopback_profile_using_ip_cache() {
        use super::audit_hubs_for_self_hub_cache;
        use crate::config::{HubEntry, HubsConfig};
        let secrets_dir = tmpdir("hubs-loopback");
        let mut config = HubsConfig::default();
        config.hubs.insert(
            "local".to_string(),
            HubEntry {
                address: "127.0.0.1:9100".to_string(),
                secret_file: Some(
                    secrets_dir
                        .join("127.0.0.1.hex")
                        .to_string_lossy()
                        .to_string(),
                ),
                secret: None,
                scope: None,
                bootstrap_from: None,
            },
        );
        let hints = audit_hubs_for_self_hub_cache(&config, &secrets_dir);
        assert_eq!(hints.len(), 1);
        assert!(hints[0].contains("Profile 'local'"), "got: {}", hints[0]);
        assert!(
            hints[0].contains("<runtime_dir>/hub.secret"),
            "migration hint must point at the live secret; got: {}",
            hints[0]
        );
    }

    #[test]
    fn audit_hubs_does_not_flag_remote_profile_using_ip_cache() {
        // T-1284: remote-hub profiles legitimately use IP-keyed caches
        // (the alternative would be SSH-on-every-call). Don't flag.
        use super::audit_hubs_for_self_hub_cache;
        use crate::config::{HubEntry, HubsConfig};
        let secrets_dir = tmpdir("hubs-remote");
        let mut config = HubsConfig::default();
        config.hubs.insert(
            "ring20".to_string(),
            HubEntry {
                address: "192.168.10.121:9100".to_string(),
                secret_file: Some(
                    secrets_dir
                        .join("192.168.10.121.hex")
                        .to_string_lossy()
                        .to_string(),
                ),
                secret: None,
                scope: None,
                bootstrap_from: None,
            },
        );
        let hints = audit_hubs_for_self_hub_cache(&config, &secrets_dir);
        assert!(hints.is_empty(), "remote-hub profile must not flag; got: {:?}", hints);
    }

    #[test]
    fn audit_hubs_handles_localhost_and_ipv6_loopback() {
        use super::audit_hubs_for_self_hub_cache;
        use crate::config::{HubEntry, HubsConfig};
        let secrets_dir = tmpdir("hubs-named-loop");
        let mut config = HubsConfig::default();
        // localhost
        config.hubs.insert(
            "localhost-profile".to_string(),
            HubEntry {
                address: "localhost:9100".to_string(),
                secret_file: Some(secrets_dir.join("local.hex").to_string_lossy().to_string()),
                secret: None,
                scope: None,
                bootstrap_from: None,
            },
        );
        // IPv6 loopback (bracketed)
        config.hubs.insert(
            "v6-profile".to_string(),
            HubEntry {
                address: "[::1]:9100".to_string(),
                secret_file: Some(secrets_dir.join("v6.hex").to_string_lossy().to_string()),
                secret: None,
                scope: None,
                bootstrap_from: None,
            },
        );
        let hints = audit_hubs_for_self_hub_cache(&config, &secrets_dir);
        assert_eq!(hints.len(), 2, "expected both loopback forms to flag; got: {:?}", hints);
    }
}
