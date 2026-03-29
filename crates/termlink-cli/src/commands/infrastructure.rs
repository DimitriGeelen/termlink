use anyhow::{Context, Result};
use serde_json::json;

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
        println!("Listening for connections... (Ctrl+C to stop)");
    }

    tokio::signal::ctrl_c().await.ok();

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

    // 4. Hub status
    let hub_socket = termlink_hub::server::hub_socket_path();
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            // Verify the socket is actually responsive (with timeout to avoid hanging)
            let hub_rpc = client::rpc_call(&hub_socket, "termlink.ping", json!({}));
            match tokio::time::timeout(ping_timeout, hub_rpc).await {
                Ok(Ok(_)) => check!("hub", pass, format!("running (PID {pid}), responding")),
                Ok(Err(_)) | Err(_) => check!("hub", warn, format!("running (PID {pid}), but not responding on socket")),
            }
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            if fix {
                termlink_hub::pidfile::remove(&pidfile_path);
                let _ = std::fs::remove_file(&hub_socket);
                check!("hub", warn, format!("stale pidfile (PID {pid}) — fixed: removed pidfile and socket"));
            } else {
                check!("hub", warn, format!("stale pidfile (PID {pid} is dead). Run 'termlink doctor --fix' to clean up"));
            }
        }
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            check!("hub", pass, "not running (optional — needed for multi-session routing)");
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

    // 6. Version
    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    check!("version", pass, format!("termlink {version} ({commit})"));

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
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

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
            let socket_path = termlink_hub::server::hub_socket_path();
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

pub(crate) fn cmd_hub_status(json_output: bool, short: bool, check: bool) -> Result<()> {
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
    let socket_path = termlink_hub::server::hub_socket_path();

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
            if json_output {
                println!("{}", json!({
                    "ok": true,
                    "status": "running",
                    "pid": pid,
                    "socket": socket_path.display().to_string(),
                    "pidfile": pidfile_path.display().to_string(),
                }));
            } else if short {
                println!("running {pid}");
            } else {
                println!("Hub: running (PID {pid})");
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
