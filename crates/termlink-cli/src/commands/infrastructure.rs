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

    // 7. Inbox status (T-1001 / T-1400 / T-1415).
    //
    // Probes inbox state via `channel.list(prefix="inbox:")`. The legacy
    // `inbox.status` RPC was retired in T-1166 / T-1415 — its hub-side
    // handler no longer exists, so the prior dual-probe fallback would
    // always fail on the inbox.status leg. channel.list is the only
    // load-bearing path now.
    if hub_socket.exists() {
        let outcome: Result<(u64, usize), String> = async {
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
        }
        .await;

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
                fix,
            );
            if issues.is_empty() {
                check!("secret_cache", pass, "all cached secrets look healthy");
            } else {
                for msg in &issues {
                    // T-1654: messages prefixed with "fixed:" represent
                    // successful auto-remediation (--fix chmod 600). Render
                    // them as pass-class so the operator sees green for the
                    // closed-loop fix, not yellow for an outstanding warning.
                    if msg.starts_with("fixed:") {
                        check!("secret_cache", pass, msg.clone());
                    } else {
                        check!("secret_cache", warn, msg.clone());
                    }
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

    // 7b. Identity attribution — T-1705 (G-056 follow-up).
    //
    // Sibling to T-1704's `whoami` hint. Groups live sessions by
    // identity_fingerprint and surfaces the shared-host case (PL-166)
    // from the diagnostic path. Operators who never run whoami but do
    // run doctor when something feels wrong land on the same hint.
    // Pre-T-1436 sessions (no identity_fingerprint) are excluded from
    // grouping rather than bundled into a phantom "no-FP" group.
    {
        let groups = group_sessions_by_identity(&sessions);
        let shared: Vec<_> = groups.iter().filter(|(_, names)| names.len() >= 2).collect();
        if shared.is_empty() {
            let with_fp: usize = groups.iter().map(|(_, n)| n.len()).sum();
            check!("identity", pass, format!("no shared identities ({with_fp} session(s) with FP)"));
        } else {
            let total_shared: usize = shared.iter().map(|(_, n)| n.len()).sum();
            let groups_desc: Vec<_> = shared.iter().map(|(fp, names)| {
                let short_fp = &fp[..8.min(fp.len())];
                format!("{}×{}", short_fp, names.len())
            }).collect();
            check!("identity", warn, format!(
                "{total_shared} sessions share {} identity FP [{}] — pass --identity-key at register for per-agent identity (T-1700)",
                shared.len(),
                groups_desc.join(", ")
            ));
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

/// T-2060 / T-2028 Track C: render the `hub.governor_status` RPC response
/// as the human-mode `Governor:` section.
///
/// Pure helper — takes the parsed JSON value and produces the multi-line
/// string. Kept pure so a unit test can pin the format without spinning up
/// a hub.
pub(crate) fn render_governor_section(v: &serde_json::Value) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let g = |k: &str| -> String {
        v.get(k)
            .and_then(|x| x.as_i64())
            .map(|n| n.to_string())
            .unwrap_or_else(|| "n/a".to_string())
    };
    let _ = writeln!(out, "Governor:");
    let _ = writeln!(
        out,
        "  Connections: {}/{} (capacity_hits_total={})",
        g("connections_active"),
        g("connections_max"),
        g("capacity_hits_total"),
    );
    let _ = writeln!(
        out,
        "  Rate buckets: {} active (rate_hits_total={}, max_rate_per_sec={})",
        g("rate_buckets_active"),
        g("rate_hits_total"),
        g("max_rate_per_sec"),
    );
    let _ = writeln!(
        out,
        "  Dedupe: {} entries (hits_total={}, ttl_ms={})",
        g("dedupe_entries_active"),
        g("dedupe_hits_total"),
        g("dedupe_ttl_ms"),
    );
    // T-2110: cv_index telemetry — substrate primitive #9 health surfaced
    // alongside dedupe so operators see broadcast-with-replay saturation
    // (overflow_total > 0 means a topic has hit its per-topic cap and new
    // cv_keys are being silently un-indexed).
    let _ = writeln!(
        out,
        "  cv_index: {} entries across {} topic(s) (overflow_total={}, cap_per_topic={})",
        g("cv_index_entries_active"),
        g("cv_index_topics_active"),
        g("cv_index_overflow_total"),
        g("cv_index_cap_per_topic"),
    );
    out
}

pub(crate) async fn cmd_hub_status(
    json_output: bool,
    short: bool,
    check: bool,
    governor: bool,
) -> Result<()> {
    // T-1032: Use resolve_hub_paths() for split-brain runtime dir detection
    let (pidfile_path, socket_path) = resolve_hub_paths();

    let is_running = matches!(
        termlink_hub::pidfile::check(&pidfile_path),
        termlink_hub::pidfile::PidfileStatus::Running(_)
    );

    // T-2060: probe governor only when running AND --governor was passed.
    // Bounded 2s timeout matches the doctor pattern so a wedged hub can't
    // hang the status verb. Failure is rendered, not silenced.
    use termlink_session::client;
    let governor_result: Option<std::result::Result<serde_json::Value, String>> =
        if governor && is_running {
            let rpc = client::rpc_call(&socket_path, "hub.governor_status", json!({}));
            match tokio::time::timeout(std::time::Duration::from_secs(2), rpc).await {
                Ok(Ok(resp)) => match client::unwrap_result(resp) {
                    Ok(result) => Some(Ok(result)),
                    Err(e) => Some(Err::<serde_json::Value, String>(e.to_string())),
                },
                Ok(Err(e)) => Some(Err::<serde_json::Value, String>(e.to_string())),
                Err(_) => Some(Err::<serde_json::Value, String>("timed out after 2s".to_string())),
            }
        } else {
            None
        };

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
                let mut env = json!({
                    "ok": true,
                    "status": "running",
                    "pid": pid,
                    "socket": socket_path.display().to_string(),
                    "pidfile": pidfile_path.display().to_string(),
                    "runtime_dir": runtime_dir,
                });
                if let Some(g) = &governor_result {
                    let obj = env.as_object_mut().expect("status envelope is object");
                    match g {
                        Ok(value) => { obj.insert("governor".into(), value.clone()); }
                        Err(e) => { obj.insert("governor".into(), json!({"error": e})); }
                    }
                }
                println!("{}", env);
            } else if short {
                println!("running {pid}");
            } else {
                println!("Hub: running (PID {pid})");
                println!("  Runtime dir: {}", runtime_dir);
                println!("  Socket: {}", socket_path.display());
                println!("  Pidfile: {}", pidfile_path.display());
                if let Some(g) = &governor_result {
                    match g {
                        Ok(value) => print!("{}", render_governor_section(value)),
                        Err(e) => println!("Governor: (unavailable: {})", e),
                    }
                }
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

/// T-1656 / G-011 R3 facet 2: export the LIVE hub HMAC secret.
///
/// Always reads `termlink_hub::server::hub_secret_path()` (== `<runtime_dir>/hub.secret`).
/// Never reads `~/.termlink/secrets/<host>.hex` — that's an IP-keyed convenience cache
/// which is NOT invalidated when the hub regenerates, so peers handed a cached value
/// see auth-mismatch symptoms while the giving end appears clean.
///
/// Stdout default for piping; `--out` writes atomically with chmod 600 (mirrors
/// `cmd_fleet_reauth_bootstrap`'s safe-write path).
pub(crate) fn cmd_hub_export_secret(out: Option<&str>, json_output: bool) -> Result<()> {
    use std::io::Write;

    let live_path = termlink_hub::server::hub_secret_path();
    let hex = std::fs::read_to_string(&live_path).map_err(|e| {
        anyhow::anyhow!(
            "no hub.secret at {} — is the hub running? ({})",
            live_path.display(),
            e
        )
    })?;
    let hex = hex.trim().to_string();
    let bytes = hex.len() / 2;

    if let Some(out_path) = out {
        let path = PathBuf::from(out_path);
        let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(dir).ok();
        let tmp = path.with_extension(format!(
            "tmp.{}",
            std::process::id()
        ));
        {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp)
                .with_context(|| format!("open {} for write", tmp.display()))?;
            f.write_all(hex.as_bytes())?;
            f.sync_all().ok();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))
                .with_context(|| format!("chmod 600 {}", tmp.display()))?;
        }
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;

        if json_output {
            println!(
                "{}",
                json!({
                    "path": live_path.display().to_string(),
                    "out": path.display().to_string(),
                    "bytes": bytes,
                })
            );
        } else {
            println!("Wrote {} ({} bytes, chmod 600)", path.display(), bytes);
            println!("Source: {}", live_path.display());
        }
    } else if json_output {
        println!(
            "{}",
            json!({
                "path": live_path.display().to_string(),
                "hex": hex,
                "bytes": bytes,
            })
        );
    } else {
        // Stdout: just the hex, no trailing newline — pipe-friendly.
        print!("{hex}");
        std::io::stdout().flush().ok();
    }
    Ok(())
}

/// T-1657: print sha256 fingerprint of <runtime_dir>/hub.cert.pem.
///
/// Output matches `KnownHubStore`'s `sha256:<hex>` form, so values printed
/// here are directly comparable against peer pins. Reads the LIVE cert
/// file via `termlink_hub::tls::hub_cert_path()` — same "never cache, always
/// live" discipline as T-1656's `hub export-secret`.
pub(crate) fn cmd_hub_fingerprint(json_output: bool) -> Result<()> {
    use base64::Engine;

    let cert_path = termlink_hub::tls::hub_cert_path();
    let pem = std::fs::read_to_string(&cert_path).map_err(|e| {
        anyhow::anyhow!(
            "no hub.cert.pem at {} — is the hub running? ({})",
            cert_path.display(),
            e
        )
    })?;

    // Extract the first CERTIFICATE block. The hub's PEM always contains
    // exactly one cert, but be defensive in case the file is concatenated.
    let start = pem.find("-----BEGIN CERTIFICATE-----").ok_or_else(|| {
        anyhow::anyhow!("no CERTIFICATE block in {}", cert_path.display())
    })?;
    let end = pem.find("-----END CERTIFICATE-----").ok_or_else(|| {
        anyhow::anyhow!("malformed CERTIFICATE block in {} (no END marker)", cert_path.display())
    })?;
    let body = &pem[start + "-----BEGIN CERTIFICATE-----".len()..end];
    let b64: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    let der = base64::engine::general_purpose::STANDARD
        .decode(b64.as_bytes())
        .map_err(|e| anyhow::anyhow!("base64-decode failed for {}: {}", cert_path.display(), e))?;

    let fingerprint = termlink_session::tofu::cert_fingerprint(&der);

    if json_output {
        println!(
            "{}",
            json!({
                "path": cert_path.display().to_string(),
                "fingerprint": fingerprint,
            })
        );
    } else {
        println!("{fingerprint}");
    }
    Ok(())
}

/// T-1658: TLS-probe a remote hub and print its leaf cert fingerprint.
///
/// Companion to `cmd_hub_fingerprint` (T-1657, local) — `cmd_hub_probe`
/// reads the same value from the wire, no auth required, no profile
/// required, no `KnownHubStore` mutation. Output matches the canonical
/// `sha256:<hex>` form so values are directly comparable to local
/// `hub fingerprint`, `tofu list`, and `KnownHubStore.get(addr)`.
pub(crate) async fn cmd_hub_probe(addr: &str, json_output: bool) -> Result<()> {
    // T-1675: bound the probe at 10s to match `fleet verify` / `fleet doctor
    // --include-pin-check` defaults — otherwise an unreachable host holds
    // the operator's terminal for the OS TCP retry budget (30-60+s).
    //
    // T-1928: align --json envelope with MCP `termlink_hub_probe`. Both
    // sides now emit {ok, address, fingerprint, error}. Failure path
    // routes through json_error_exit so consumers of --json can parse
    // the error instead of getting a non-JSON anyhow bail to stderr.
    let probe = termlink_session::tofu::probe_cert_with_timeout(
        addr, std::time::Duration::from_secs(10),
    ).await;

    match (probe, json_output) {
        (Ok((_der, fingerprint)), true) => {
            println!("{}", json!({
                "ok": true,
                "address": addr,
                "fingerprint": fingerprint,
                "error": serde_json::Value::Null,
            }));
            Ok(())
        }
        (Ok((_der, fingerprint)), false) => {
            println!("{fingerprint}");
            Ok(())
        }
        (Err(e), true) => super::json_error_exit(json!({
            "ok": false,
            "address": addr,
            "fingerprint": serde_json::Value::Null,
            "error": e,
        })),
        (Err(e), false) => Err(anyhow::anyhow!(e)),
    }
}

// === Inbox Commands (T-997) ===

pub(crate) async fn cmd_inbox_status(json_output: bool) -> Result<()> {
    super::print_deprecation_warning("inbox status", "channel info");
    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        // T-1916: honor --json on hub-down (same pattern events.rs uses inline).
        if json_output {
            super::json_error_exit(json!({
                "ok": false,
                "error": format!("Hub is not running (no socket at {})", hub_socket.display()),
            }));
        }
        anyhow::bail!("Hub is not running (no socket at {})", hub_socket.display());
    }

    let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
    let cache = termlink_session::hub_capabilities::shared_cache();
    let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
    let status = termlink_session::inbox_channel::status_via_channel(&addr, cache, &mut ctx)
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
    super::print_deprecation_warning("inbox clear", "channel subscribe --cursor");
    if target.is_none() && !all {
        if json_output {
            super::json_error_exit(json!({
                "ok": false,
                "error": "Specify a target session name, or use --all to clear everything",
            }));
        }
        anyhow::bail!("Specify a target session name, or use --all to clear everything");
    }

    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        // T-1916: honor --json on hub-down.
        if json_output {
            super::json_error_exit(json!({
                "ok": false,
                "error": format!("Hub is not running (no socket at {})", hub_socket.display()),
            }));
        }
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
    let result = termlink_session::inbox_channel::clear_via_channel(&addr, scope, cache, &mut ctx)
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
    super::print_deprecation_warning("inbox list", "channel subscribe");
    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        // T-1916: honor --json on hub-down.
        if json_output {
            super::json_error_exit(json!({
                "ok": false,
                "error": format!("Hub is not running (no socket at {})", hub_socket.display()),
            }));
        }
        anyhow::bail!("Hub is not running (no socket at {})", hub_socket.display());
    }

    let addr = termlink_protocol::TransportAddr::unix(&hub_socket);
    let cache = termlink_session::hub_capabilities::shared_cache();
    let mut ctx = termlink_session::inbox_channel::FallbackCtx::new();
    let entries = termlink_session::inbox_channel::list_via_channel(&addr, target, cache, &mut ctx)
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
            // T-1934: emit `message` for parity with MCP `termlink_tofu_clear`.
            let message = if existed {
                format!("Removed TOFU entry for {host_port}. Next connection will re-trust.")
            } else {
                format!("No TOFU entry found for '{host_port}'")
            };
            println!("{}", json!({
                "ok": existed,
                "host": host_port,
                "removed": existed,
                "message": message,
            }));
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

/// T-1659: probe a hub's wire fingerprint and compare against the stored TOFU pin.
///
/// Deterministic exit codes (script-friendly):
///   0 — match (pin still valid)
///   1 — drift (wire != pin; rotation occurred — heal required)
///   2 — no pin (host not in KnownHubStore)
///   3 — probe failed (unreachable / TLS error)
///
/// In `--json` mode we always exit 0 so callers can parse; the verdict is
/// carried in the `status` field of the JSON object.
///
/// Pure read-only — does NOT mutate `KnownHubStore`.
pub(crate) async fn cmd_tofu_verify(host: &str, json_output: bool) -> Result<()> {
    let store = termlink_session::tofu::KnownHubStore::default_store();
    let pinned: Option<String> = store.get(host);

    // Probe the wire. Capture probe errors as "probe-failed" status.
    // T-1675: 10s timeout — match `fleet verify` / `hub probe` defaults.
    let probe_result = termlink_session::tofu::probe_cert_with_timeout(
        host, std::time::Duration::from_secs(10),
    ).await;

    let (status, wire_fp, match_flag, probe_err): (&str, Option<String>, Option<bool>, Option<String>) =
        match (&probe_result, &pinned) {
            (Ok((_, wire)), Some(pin)) if wire == pin => ("match", Some(wire.clone()), Some(true), None),
            (Ok((_, wire)), Some(_)) => ("drift", Some(wire.clone()), Some(false), None),
            (Ok((_, wire)), None) => ("no-pin", Some(wire.clone()), None, None),
            (Err(e), _) => ("probe-failed", None, None, Some(e.clone())),
        };

    if json_output {
        // T-1927: envelope aligned with MCP `termlink_tofu_verify`.
        // - `ok` reflects pin-matches-wire (status=="match")
        // - `actions` carries heal hints for drift cases (empty otherwise)
        let ok = status == "match";
        let actions: Vec<String> = if status == "drift" {
            vec![
                format!("Heal: termlink fleet reauth <profile-for-{}> --bootstrap-from auto", host),
                format!("Re-pin: termlink tofu clear {}", host),
            ]
        } else { Vec::new() };
        println!(
            "{}",
            json!({
                "ok": ok,
                "address": host,
                "status": status,
                "wire": wire_fp,
                "pinned": pinned,
                "match": match_flag,
                "probe_error": probe_err,
                "actions": actions,
            })
        );
        return Ok(());
    }

    // Human-readable output + deterministic exit code via process::exit.
    match status {
        "match" => {
            println!("[OK] {} — pin matches wire fingerprint", host);
            println!("  {}", wire_fp.as_deref().unwrap_or(""));
        }
        "drift" => {
            println!("[DRIFT] {} — wire fingerprint does NOT match stored pin", host);
            println!("  Pinned: {}", pinned.as_deref().unwrap_or("(none)"));
            println!("  Wire:   {}", wire_fp.as_deref().unwrap_or("(none)"));
            println!();
            println!("  Hub rotated. Heal: termlink fleet reauth <profile> --bootstrap-from auto");
            println!("  Then re-pin:        termlink tofu clear {}", host);
            std::process::exit(1);
        }
        "no-pin" => {
            println!("[NO-PIN] {} — host not in KnownHubStore", host);
            println!("  Wire: {}", wire_fp.as_deref().unwrap_or(""));
            println!();
            println!("  This is the wire fingerprint. To trust it (TOFU), connect once via");
            println!("  any auth-bearing command (e.g. termlink remote ping <profile>).");
            std::process::exit(2);
        }
        "probe-failed" => {
            println!("[PROBE-FAILED] {} — could not retrieve wire fingerprint", host);
            if let Some(e) = &probe_err {
                println!("  {}", e);
            }
            std::process::exit(3);
        }
        _ => unreachable!(),
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
    fix: bool,
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
            // T-1654: when --fix is on, chmod 600 in place — the canonical
            // T-1055 write mode is the only correct state for a cached HMAC
            // secret, so auto-remediation is safe. Drift/divergence issues
            // below are NOT auto-fixed; the operator must decide what's
            // authoritative.
            if fix {
                match std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)) {
                    Ok(()) => {
                        issues.push(format!(
                            "fixed: {} mode 0o{:o} → 0o600",
                            path.display(),
                            mode
                        ));
                    }
                    Err(e) => {
                        issues.push(format!(
                            "{} has mode {:o} (expected 600) — chmod failed: {}",
                            path.display(),
                            mode,
                            e
                        ));
                    }
                }
            } else {
                issues.push(format!(
                    "{} has mode {:o} (expected 600) — world/group-readable cache",
                    path.display(),
                    mode
                ));
            }
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

/// T-1705: group live sessions by their `metadata.identity_fingerprint`,
/// excluding sessions without an FP (pre-T-1436). Returns a Vec of
/// `(fp, names)` so iteration order is stable for the doctor message.
/// Pure function — testable without spinning up a real hub.
fn group_sessions_by_identity(
    sessions: &[termlink_session::registration::Registration],
) -> Vec<(String, Vec<String>)> {
    use std::collections::BTreeMap;
    let mut by_fp: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for s in sessions {
        if let Some(fp) = s.metadata.identity_fingerprint.as_deref() {
            by_fp.entry(fp.to_string()).or_default().push(s.display_name.clone());
        }
    }
    by_fp.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{audit_secret_cache, render_governor_section};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // T-2060 / T-2028 Track C: pin the human-mode `Governor:` section.
    // T-2110: extended to cover the new cv_index telemetry line.
    #[test]
    fn render_governor_section_formats_known_value() {
        let v = serde_json::json!({
            "connections_active": 3,
            "connections_max": 256,
            "capacity_hits_total": 0,
            "rate_buckets_active": 5,
            "rate_hits_total": 0,
            "max_rate_per_sec": 1000,
            "dedupe_entries_active": 12,
            "dedupe_hits_total": 4,
            "dedupe_ttl_ms": 300000,
            // T-2110: cv_index telemetry.
            "cv_index_entries_active": 5,
            "cv_index_topics_active": 2,
            "cv_index_overflow_total": 0,
            "cv_index_cap_per_topic": 1000,
        });
        let s = render_governor_section(&v);
        assert!(s.starts_with("Governor:\n"));
        assert!(s.contains("Connections: 3/256 (capacity_hits_total=0)"));
        assert!(s.contains("Rate buckets: 5 active (rate_hits_total=0, max_rate_per_sec=1000)"));
        assert!(s.contains("Dedupe: 12 entries (hits_total=4, ttl_ms=300000)"));
        assert!(s.contains("cv_index: 5 entries across 2 topic(s) (overflow_total=0, cap_per_topic=1000)"));
    }

    // T-2060: missing fields render as "n/a" rather than panic — the
    // renderer must remain best-effort against an older hub. Smoke-confirmed
    // on the live .107 hub which is pre-T-2049 (no dedupe_* fields).
    // T-2110: extended to cover the new cv_index line (also "n/a" against
    // a hub that pre-dates the field).
    #[test]
    fn render_governor_section_tolerates_missing_fields() {
        let v = serde_json::json!({
            "connections_active": 1,
            "connections_max": 256
            // capacity_hits_total + rate_* + dedupe_* + cv_index_* all absent
        });
        let s = render_governor_section(&v);
        assert!(s.contains("Connections: 1/256 (capacity_hits_total=n/a)"));
        assert!(s.contains("Rate buckets: n/a active"));
        assert!(s.contains("Dedupe: n/a entries (hits_total=n/a, ttl_ms=n/a)"));
        assert!(s.contains("cv_index: n/a entries across n/a topic(s) (overflow_total=n/a, cap_per_topic=n/a)"));
    }

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
        assert!(audit_secret_cache(&missing, None, false).is_empty());
    }

    #[test]
    fn good_perms_no_local_hub_is_empty() {
        let d = tmpdir("good");
        write_hex(&d, "ring20.hex", 0o600);
        assert!(audit_secret_cache(&d, None, false).is_empty());
    }

    #[test]
    fn bad_perms_reported() {
        let d = tmpdir("bad-perms");
        write_hex(&d, "proxmox4.hex", 0o644);
        let issues = audit_secret_cache(&d, None, false);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("mode 644"));
        assert!(issues[0].contains("proxmox4.hex"));
    }

    #[test]
    fn bak_siblings_skipped() {
        let d = tmpdir("bak");
        write_hex(&d, "ring20.hex.bak", 0o644); // deliberately bad perms
        assert!(
            audit_secret_cache(&d, None, false).is_empty(),
            ".bak siblings must not be flagged"
        );
    }

    // T-1654: --fix autoheal contract — auto-chmod 600 on bad-perms cache files.

    #[test]
    fn fix_chmods_bad_perms_and_reports_fixed_message() {
        let d = tmpdir("fix-chmod");
        let p = write_hex(&d, "leaky.hex", 0o644);
        let issues = audit_secret_cache(&d, None, true);
        assert_eq!(issues.len(), 1, "expected one fixed:- line, got: {:?}", issues);
        assert!(issues[0].starts_with("fixed:"),
            "fix-mode message must begin with `fixed:` so doctor renders pass-class: {}", issues[0]);
        assert!(issues[0].contains("0o644"),
            "fixed message must include previous mode: {}", issues[0]);
        assert!(issues[0].contains("0o600"),
            "fixed message must include target mode: {}", issues[0]);
        // Verify the file actually got chmodded.
        let mode = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "post-fix mode must be 0o600, got: 0o{:o}", mode);
    }

    #[test]
    fn fix_no_op_on_already_correct_perms() {
        let d = tmpdir("fix-noop");
        let p = write_hex(&d, "healthy.hex", 0o600);
        let issues = audit_secret_cache(&d, None, true);
        assert!(issues.is_empty(),
            "already-0o600 file must not generate any output even with --fix: {:?}", issues);
        // Mode preserved.
        let mode = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn fix_does_not_chmod_divergence_only_issues() {
        // T-1654: drift/divergence is a semantic decision the operator must
        // make. --fix must not touch the file contents nor mask the warning.
        let d = tmpdir("fix-divergence");
        let cache = write_hex(&d, "stalehub.hex", 0o600);
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        fs::File::options().write(true).open(&cache).unwrap().set_modified(past).unwrap();
        let hub_secret = d.join("hub.secret");
        fs::write(&hub_secret, b"cafebabe").unwrap();
        let issues = audit_secret_cache(
            &d,
            Some((hub_secret.as_path(), std::time::SystemTime::now())),
            true,
        );
        assert_eq!(issues.len(), 1, "divergence must still warn under --fix: {:?}", issues);
        assert!(!issues[0].starts_with("fixed:"),
            "divergence is not auto-fixable; must not pretend it was: {}", issues[0]);
        assert!(issues[0].contains("diverges"),
            "divergence wording must survive --fix mode: {}", issues[0]);
        // File content + perms unchanged.
        let content = fs::read_to_string(&cache).unwrap();
        assert_eq!(content.trim(), "deadbeef", "fix must not rewrite divergent cache");
        let mode = fs::metadata(&cache).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "fix must not chmod already-correct file");
    }

    #[test]
    fn fix_combines_chmod_and_drift_independently() {
        // Real-world: one file has bad perms AND a *different* file is divergent.
        // --fix must heal the perms problem and leave the divergence warning.
        //
        // To isolate the two signals we hand-write content (not write_hex) so
        // the perms-problem file matches hub.secret value (no drift) and the
        // divergent file has correct perms but stale value.
        let d = tmpdir("fix-mixed");
        let p_bad = d.join("leaky.hex");
        fs::write(&p_bad, b"cafebabe").unwrap();
        fs::set_permissions(&p_bad, fs::Permissions::from_mode(0o644)).unwrap();
        let p_drift = d.join("drift.hex");
        fs::write(&p_drift, b"deadbeef").unwrap();
        fs::set_permissions(&p_drift, fs::Permissions::from_mode(0o600)).unwrap();
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        fs::File::options().write(true).open(&p_drift).unwrap().set_modified(past).unwrap();
        let hub_secret = d.join("hub.secret");
        fs::write(&hub_secret, b"cafebabe").unwrap();
        let issues = audit_secret_cache(
            &d,
            Some((hub_secret.as_path(), std::time::SystemTime::now())),
            true,
        );
        // Expect exactly 2 lines (1 fixed: from leaky.hex perms + 1 diverges from drift.hex).
        // readdir order not guaranteed → check by partition.
        assert_eq!(issues.len(), 2, "got: {:?}", issues);
        let fixed_count = issues.iter().filter(|m| m.starts_with("fixed:")).count();
        let drift_count = issues.iter().filter(|m| m.contains("diverges")).count();
        assert_eq!(fixed_count, 1, "exactly one fixed: line expected: {:?}", issues);
        assert_eq!(drift_count, 1, "exactly one diverges line expected: {:?}", issues);
        // Bad-perms file healed.
        let m_bad = fs::metadata(&p_bad).unwrap().permissions().mode() & 0o777;
        assert_eq!(m_bad, 0o600, "leaky.hex must be chmodded to 0o600");
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
            false,
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
            false,
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
            false,
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
            false,
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

    // T-1656 / G-011 R3 facet 2: `hub export-secret` must read the LIVE
    // <runtime_dir>/hub.secret, never the IP-keyed cache. The cmd takes
    // no path parameter; it resolves via `hub_secret_path()` which honors
    // TERMLINK_RUNTIME_DIR. Both tests use ENV_LOCK to serialize env writes.
    mod export_secret {
        use crate::test_env_lock::ENV_LOCK;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        fn unique_dir(label: &str) -> std::path::PathBuf {
            let ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            std::env::temp_dir().join(format!(
                "termlink-export-secret-{}-{}-{}",
                label,
                std::process::id(),
                ns
            ))
        }

        #[test]
        fn export_secret_reads_live_not_cache() {
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

            // Stage two parallel sources of "the secret" — live + cache —
            // with different content so we can prove which one was read.
            let runtime_dir = unique_dir("live");
            fs::create_dir_all(&runtime_dir).unwrap();
            let live = runtime_dir.join("hub.secret");
            fs::write(&live, "aaaaaaaa".repeat(8)).unwrap(); // 64-char "LIVE"

            let home = unique_dir("home");
            let secrets_dir = home.join(".termlink").join("secrets");
            fs::create_dir_all(&secrets_dir).unwrap();
            fs::write(secrets_dir.join("127.0.0.1.hex"), "bbbbbbbb".repeat(8)).unwrap(); // "STALE"

            let out = unique_dir("dest").join("captured.hex");

            let prev_rt = std::env::var_os("TERMLINK_RUNTIME_DIR");
            let prev_home = std::env::var_os("HOME");
            // SAFETY: single-threaded test region (ENV_LOCK).
            unsafe {
                std::env::set_var("TERMLINK_RUNTIME_DIR", &runtime_dir);
                std::env::set_var("HOME", &home);
            }

            let result = super::super::cmd_hub_export_secret(
                Some(out.to_str().unwrap()),
                false,
            );

            // SAFETY: single-threaded test region (ENV_LOCK).
            unsafe {
                match prev_rt {
                    Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                    None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
                }
                match prev_home {
                    Some(v) => std::env::set_var("HOME", v),
                    None => std::env::remove_var("HOME"),
                }
            }

            result.expect("export must succeed when live exists");
            let captured = fs::read_to_string(&out).expect("out file should exist");
            assert_eq!(captured, "a".repeat(64),
                "must read LIVE (aaaa...), not STALE cache (bbbb...). Got: {}",
                captured);

            // chmod 600 invariant.
            let mode = fs::metadata(&out).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600,
                "--out path must be chmod 600, got: 0o{:o}", mode);

            // Cleanup.
            let _ = fs::remove_dir_all(&runtime_dir);
            let _ = fs::remove_dir_all(&home);
            let _ = fs::remove_dir_all(out.parent().unwrap());
        }

        // T-1657: hub fingerprint reads the live cert + emits sha256:<hex>
        // matching `cert_fingerprint(der)`. Same env-lock pattern; serialize
        // TERMLINK_RUNTIME_DIR mutations.
        #[test]
        fn fingerprint_matches_tofu_format() {
            use base64::Engine;
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

            // Hand-craft a minimal DER blob (16 bytes of known content), wrap
            // it as a PEM "CERTIFICATE", and assert the cmd succeeds + the
            // expected sha256:<hex> form is what `cert_fingerprint` would
            // produce. We don't need a real X.509 cert — we're testing the
            // parser + hasher contract, not certificate validity.
            let der: Vec<u8> = (0u8..16).collect();
            let b64 = base64::engine::general_purpose::STANDARD.encode(&der);
            let pem = format!(
                "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
                b64
            );
            let expected = termlink_session::tofu::cert_fingerprint(&der);

            let runtime_dir = unique_dir("fp");
            fs::create_dir_all(&runtime_dir).unwrap();
            fs::write(runtime_dir.join("hub.cert.pem"), &pem).unwrap();

            let prev = std::env::var_os("TERMLINK_RUNTIME_DIR");
            // SAFETY: ENV_LOCK held.
            unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &runtime_dir); }

            let result = super::super::cmd_hub_fingerprint(true);

            // SAFETY: ENV_LOCK held.
            unsafe {
                match prev {
                    Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                    None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
                }
            }

            result.expect("fingerprint must succeed");
            assert!(expected.starts_with("sha256:"),
                "expected fingerprint format sha256:<hex>, got: {}", expected);
            assert_eq!(expected.len(), "sha256:".len() + 64,
                "expected 64 hex chars after prefix, got: {}", expected);

            let _ = fs::remove_dir_all(&runtime_dir);
        }

        #[test]
        fn fingerprint_no_certificate_block_errors() {
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

            let runtime_dir = unique_dir("badpem");
            fs::create_dir_all(&runtime_dir).unwrap();
            // PEM file exists but has no CERTIFICATE block (e.g. operator
            // accidentally pointed at a key.pem instead).
            fs::write(
                runtime_dir.join("hub.cert.pem"),
                "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n",
            ).unwrap();

            let prev = std::env::var_os("TERMLINK_RUNTIME_DIR");
            // SAFETY: ENV_LOCK held.
            unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &runtime_dir); }

            let result = super::super::cmd_hub_fingerprint(false);

            // SAFETY: ENV_LOCK held.
            unsafe {
                match prev {
                    Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                    None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
                }
            }

            let err = result.expect_err("non-cert PEM must error");
            assert!(format!("{err}").contains("no CERTIFICATE block"),
                "error must explain missing CERTIFICATE block; got: {err}");

            let _ = fs::remove_dir_all(&runtime_dir);
        }

        #[test]
        fn fingerprint_missing_cert_errors() {
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

            let runtime_dir = unique_dir("nocert");
            fs::create_dir_all(&runtime_dir).unwrap();
            // Intentionally no hub.cert.pem written.

            let prev = std::env::var_os("TERMLINK_RUNTIME_DIR");
            // SAFETY: ENV_LOCK held.
            unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &runtime_dir); }

            let result = super::super::cmd_hub_fingerprint(false);

            // SAFETY: ENV_LOCK held.
            unsafe {
                match prev {
                    Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                    None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
                }
            }

            let err = result.expect_err("missing live cert must error");
            let msg = format!("{err}");
            assert!(msg.contains("no hub.cert.pem"),
                "error must mention missing cert; got: {}", msg);
            assert!(msg.contains("is the hub running?"),
                "error must hint at hub-not-running; got: {}", msg);

            let _ = fs::remove_dir_all(&runtime_dir);
        }

        #[test]
        fn export_secret_missing_live_errors() {
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

            let runtime_dir = unique_dir("nolive");
            fs::create_dir_all(&runtime_dir).unwrap();
            // Intentionally no hub.secret written.

            let prev = std::env::var_os("TERMLINK_RUNTIME_DIR");
            // SAFETY: ENV_LOCK held.
            unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &runtime_dir); }

            let result = super::super::cmd_hub_export_secret(None, true);

            // SAFETY: ENV_LOCK held.
            unsafe {
                match prev {
                    Some(v) => std::env::set_var("TERMLINK_RUNTIME_DIR", v),
                    None => std::env::remove_var("TERMLINK_RUNTIME_DIR"),
                }
            }

            let err = result.expect_err("missing live secret must error");
            let msg = format!("{err}");
            assert!(msg.contains("no hub.secret"),
                "error must mention missing hub.secret; got: {}", msg);
            assert!(msg.contains("is the hub running?"),
                "error must hint at hub-not-running; got: {}", msg);

            let _ = fs::remove_dir_all(&runtime_dir);
        }
    }

    // T-1705: group_sessions_by_identity. Build Registrations via JSON
    // deserialize (same pattern as metadata.rs tests) so we don't depend
    // on private constructors.
    fn make_reg(id: &str, display_name: &str, identity_fp: Option<&str>)
        -> termlink_session::registration::Registration
    {
        let id_field = identity_fp
            .map(|fp| format!(r#","identity_fingerprint":"{fp}""#))
            .unwrap_or_default();
        let json = format!(
            r#"{{
                "version": 1,
                "id": "{id}",
                "display_name": "{display_name}",
                "pid": 12345,
                "uid": 0,
                "addr": {{ "type": "unix", "path": "/tmp/test.sock" }},
                "created_at": "2026-05-01T17:00:00Z",
                "heartbeat_at": "2026-05-01T17:00:00Z",
                "state": "ready",
                "capabilities": [],
                "roles": [],
                "tags": [],
                "metadata": {{ "cwd": "/tmp"{id_field} }}
            }}"#
        );
        serde_json::from_str(&json).expect("Registration JSON shape valid in test")
    }

    #[test]
    fn group_sessions_by_identity_all_unique_no_shared() {
        let sessions = vec![
            make_reg("tl-a0000000001", "alpha", Some("aaaaaaaaaaaaaaaa")),
            make_reg("tl-b0000000002", "beta",  Some("bbbbbbbbbbbbbbbb")),
            make_reg("tl-c0000000003", "gamma", Some("cccccccccccccccc")),
        ];
        let groups = super::group_sessions_by_identity(&sessions);
        let shared: Vec<_> = groups.iter().filter(|(_, n)| n.len() >= 2).collect();
        assert!(
            shared.is_empty(),
            "unique FPs must produce no shared groups; got {:?}",
            groups
        );
        assert_eq!(groups.len(), 3, "every FP-bearing session must appear in its own group");
    }

    #[test]
    fn group_sessions_by_identity_host_shared_one_group() {
        let host_fp = "d1993c2c3ec44c94";
        let sessions = vec![
            make_reg("tl-h0000000001", "framework-agent", Some(host_fp)),
            make_reg("tl-h0000000002", "termlink-agent",  Some(host_fp)),
            make_reg("tl-h0000000003", "cohort-agent",    Some(host_fp)),
            make_reg("tl-o0000000004", "unrelated",       Some("ffffffffffffffff")),
        ];
        let groups = super::group_sessions_by_identity(&sessions);
        let shared: Vec<_> = groups.iter().filter(|(_, n)| n.len() >= 2).collect();
        assert_eq!(shared.len(), 1, "exactly one shared group expected; got {:?}", groups);
        let (fp, names) = shared[0];
        assert_eq!(fp, host_fp);
        assert_eq!(names.len(), 3, "three co-resident agents share the host FP");
    }

    #[test]
    fn group_sessions_by_identity_absent_fp_excluded() {
        let sessions = vec![
            make_reg("tl-l0000000001", "legacy-1", None),
            make_reg("tl-l0000000002", "legacy-2", None),
            make_reg("tl-n0000000003", "modern",   Some("d1993c2c3ec44c94")),
        ];
        let groups = super::group_sessions_by_identity(&sessions);
        // The two None-FP sessions must not coalesce into a phantom "no-FP" bucket
        // and must not contribute to any group.
        assert_eq!(groups.len(), 1, "only the FP-bearing session forms a group; got {:?}", groups);
        assert_eq!(groups[0].1.len(), 1, "modern session is alone in its FP");
    }
}
