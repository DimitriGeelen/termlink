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

    // 7. Inbox status (T-1001)
    if hub_socket.exists() {
        match termlink_session::client::rpc_call(&hub_socket, "inbox.status", json!({})).await {
            Ok(resp) => match termlink_session::client::unwrap_result(resp) {
                Ok(result) => {
                    let total = result["total_transfers"].as_u64().unwrap_or(0);
                    if total == 0 {
                        check!("inbox", pass, "no pending transfers");
                    } else {
                        let targets = result["targets"].as_array().map(|t| t.len()).unwrap_or(0);
                        check!("inbox", warn, format!("{total} pending transfer(s) for {targets} target(s)"));
                    }
                }
                Err(e) => check!("inbox", warn, format!("inbox query failed: {e}")),
            },
            Err(e) => check!("inbox", warn, format!("inbox RPC failed: {e}")),
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

    let resp = termlink_session::client::rpc_call(&hub_socket, "inbox.status", json!({}))
        .await
        .context("Failed to query inbox status from hub")?;

    let result = termlink_session::client::unwrap_result(resp)
        .map_err(|e| anyhow::anyhow!("Hub returned error for inbox.status: {e}"))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let total = result["total_transfers"].as_u64().unwrap_or(0);
        let targets = result["targets"].as_array();

        if total == 0 {
            println!("Inbox: empty (no pending transfers)");
        } else {
            println!("Inbox: {} pending transfer(s)", total);
            if let Some(targets) = targets {
                println!();
                for t in targets {
                    let name = t["target"].as_str().unwrap_or("?");
                    let pending = t["pending"].as_u64().unwrap_or(0);
                    println!("  {name}: {pending} transfer(s)");
                }
            }
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

    let params = if all {
        json!({"all": true})
    } else {
        json!({"target": target.unwrap()})
    };

    let resp = termlink_session::client::rpc_call(&hub_socket, "inbox.clear", params)
        .await
        .context("Failed to clear inbox via hub")?;

    let result = termlink_session::client::unwrap_result(resp)
        .map_err(|e| anyhow::anyhow!("Hub returned error for inbox.clear: {e}"))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let cleared = result["cleared"].as_u64().unwrap_or(0);
        let target_name = result["target"].as_str().unwrap_or("?");
        if cleared == 0 {
            println!("No transfers to clear for '{target_name}'");
        } else {
            println!("Cleared {cleared} transfer(s) for '{target_name}'");
        }
    }
    Ok(())
}

pub(crate) async fn cmd_inbox_list(target: &str, json_output: bool) -> Result<()> {
    let (_, hub_socket) = resolve_hub_paths();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running (no socket at {})", hub_socket.display());
    }

    let resp = termlink_session::client::rpc_call(
        &hub_socket,
        "inbox.list",
        json!({"target": target}),
    )
    .await
    .context("Failed to query inbox from hub")?;

    let result = termlink_session::client::unwrap_result(resp)
        .map_err(|e| anyhow::anyhow!("Hub returned error for inbox.list: {e}"))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let transfers = result["transfers"].as_array();
        match transfers {
            Some(t) if t.is_empty() => {
                println!("No pending transfers for '{target}'");
            }
            Some(transfers) => {
                println!("{} pending transfer(s) for '{target}':", transfers.len());
                println!();
                for tr in transfers {
                    let id = tr["transfer_id"].as_str().unwrap_or("?");
                    let file = tr["filename"].as_str().unwrap_or("?");
                    let size = tr["size"].as_u64().unwrap_or(0);
                    let complete = tr["complete"].as_bool().unwrap_or(false);
                    let status = if complete { "complete" } else { "partial" };
                    println!("  {id}  {file} ({size} bytes, {status})");
                }
            }
            None => {
                println!("No pending transfers for '{target}'");
            }
        }
    }
    Ok(())
}
