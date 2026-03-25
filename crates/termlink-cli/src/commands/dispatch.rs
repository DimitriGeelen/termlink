//! `termlink dispatch` — atomic spawn+tag+collect for multi-worker orchestration.
//!
//! Spawns N workers, tags them with a dispatch ID, and collects `task.completed`
//! events (or a custom topic) via the hub. Provides a structural guarantee that
//! collect is always wired, replacing manual 40-line dispatch scripts.

use anyhow::{Context, Result};
use serde_json::json;

use termlink_session::client;
use termlink_session::manager;

use crate::cli::SpawnBackend;
use crate::util::shell_escape;

/// Run the `termlink dispatch` command.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn cmd_dispatch(
    count: u32,
    timeout: u64,
    topic: &str,
    name_prefix: Option<String>,
    tags: Vec<String>,
    backend: SpawnBackend,
    json_output: bool,
    command: Vec<String>,
) -> Result<()> {
    if count == 0 {
        anyhow::bail!("--count must be at least 1");
    }
    if command.is_empty() {
        anyhow::bail!("Command required after --");
    }

    // Check hub is running (needed for collect)
    let hub_socket = termlink_hub::server::hub_socket_path();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running. Start it with: termlink hub start\n(dispatch requires the hub for event collection)");
    }

    // Generate a unique dispatch ID
    let dispatch_id = format!(
        "D-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let prefix = name_prefix.unwrap_or_else(|| "worker".into());

    if !json_output {
        eprintln!("Dispatch {dispatch_id}: spawning {count} worker(s)...");
    }

    // Spawn N workers
    let mut worker_names = Vec::with_capacity(count as usize);
    for i in 1..=count {
        let worker_name = format!("{prefix}-{i}");
        worker_names.push(worker_name.clone());

        // Build tags: user tags + dispatch metadata
        let mut worker_tags = tags.clone();
        worker_tags.push(format!("_dispatch.id:{dispatch_id}"));
        worker_tags.push(format!("_dispatch.worker:{i}"));

        // Build the spawn command with env vars injected
        let termlink_bin = std::env::current_exe()
            .context("Failed to determine termlink binary path")?;
        let termlink_path = termlink_bin.to_string_lossy().to_string();

        let mut register_args = vec![
            "register".to_string(),
            "--name".to_string(),
            worker_name.clone(),
            "--tags".to_string(),
            worker_tags.join(","),
        ];
        register_args.push("--shell".to_string());

        let user_cmd = command
            .iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" ");

        let env_prefix = {
            let mut env = String::new();
            if let Ok(rd) = std::env::var("TERMLINK_RUNTIME_DIR") {
                env.push_str(&format!(
                    "export TERMLINK_RUNTIME_DIR={}; ",
                    shell_escape(&rd)
                ));
            }
            env.push_str(&format!(
                "export TERMLINK_DISPATCH_ID={}; ",
                shell_escape(&dispatch_id)
            ));
            env.push_str(&format!(
                "export TERMLINK_ORCHESTRATOR={}; ",
                shell_escape(&format!("{}", std::process::id()))
            ));
            env
        };

        let mut reg_parts = vec![termlink_path.clone()];
        reg_parts.extend(register_args);

        let shell_cmd = format!(
            "{env_prefix}{} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nkill $TL_PID 2>/dev/null\nwait $TL_PID 2>/dev/null",
            reg_parts.join(" ")
        );

        let resolved = resolve_spawn_backend(&backend);
        match resolved {
            SpawnBackend::Terminal => spawn_via_terminal(&worker_name, &shell_cmd)?,
            SpawnBackend::Tmux => spawn_via_tmux(&worker_name, &shell_cmd)?,
            SpawnBackend::Background => spawn_via_background(&worker_name, &shell_cmd)?,
            SpawnBackend::Auto => unreachable!(),
        }

        if !json_output {
            eprintln!("  Spawned {worker_name} via {resolved}");
        }
    }

    // Wait for all workers to register
    if !json_output {
        eprintln!("Waiting for workers to register...");
    }

    let register_timeout = std::time::Duration::from_secs(30);
    let start = std::time::Instant::now();
    let mut registered = vec![false; count as usize];

    loop {
        let all_registered = registered.iter().all(|r| *r);
        if all_registered {
            break;
        }
        if start.elapsed() > register_timeout {
            let missing: Vec<&str> = worker_names
                .iter()
                .zip(registered.iter())
                .filter(|(_, r)| !**r)
                .map(|(n, _)| n.as_str())
                .collect();
            if !json_output {
                eprintln!(
                    "Warning: {} worker(s) did not register within 30s: {}",
                    missing.len(),
                    missing.join(", ")
                );
            }
            break;
        }

        for (i, name) in worker_names.iter().enumerate() {
            if !registered[i] && manager::find_session(name).is_ok() {
                registered[i] = true;
                if !json_output {
                    eprintln!("  {name} registered");
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    let registered_count = registered.iter().filter(|r| **r).count() as u64;

    if !json_output {
        eprintln!(
            "Collecting events (topic: {topic}, count: {registered_count}, timeout: {timeout}s)..."
        );
    }

    // Collect events via hub
    let collect_timeout = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(500);
    let collect_start = std::time::Instant::now();
    let mut cursors = json!({});
    let mut collected_events = Vec::new();

    loop {
        if collected_events.len() as u64 >= registered_count {
            break;
        }
        if collect_start.elapsed() > collect_timeout {
            break;
        }

        // Filter to our dispatch workers by tag
        let mut params = json!({
            "topic": topic,
        });
        // Use worker names as targets for targeted collection
        let target_names: Vec<&str> = worker_names
            .iter()
            .zip(registered.iter())
            .filter(|(_, r)| **r)
            .map(|(n, _)| n.as_str())
            .collect();
        if !target_names.is_empty() {
            params["targets"] = json!(target_names);
        }
        if !cursors
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .is_empty()
        {
            params["since"] = cursors.clone();
        }

        let resp = match client::rpc_call(&hub_socket, "event.collect", params).await {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(error = %e, "Collect poll error");
                tokio::time::sleep(poll_interval).await;
                continue;
            }
        };

        if let Ok(result) = client::unwrap_result(resp) {
            if let Some(events) = result["events"].as_array() {
                for event in events {
                    let session_name = event["session_name"]
                        .as_str()
                        .unwrap_or("?")
                        .to_string();
                    let payload = event["payload"].clone();

                    if !json_output {
                        eprintln!("  Result from {session_name}");
                    }

                    collected_events.push(json!({
                        "worker": session_name,
                        "payload": payload,
                        "seq": event["seq"],
                        "timestamp": event["timestamp"],
                    }));
                }
            }

            if let Some(new_cursors) = result.get("cursors")
                && let Some(obj) = new_cursors.as_object()
            {
                for (k, v) in obj {
                    cursors[k] = v.clone();
                }
            }
        }

        tokio::time::sleep(poll_interval).await;
    }

    // Output results
    let collected_count = collected_events.len() as u64;
    let timed_out = collected_count < registered_count;

    if json_output {
        let result = json!({
            "dispatch_id": dispatch_id,
            "workers_spawned": count,
            "workers_registered": registered_count,
            "events_collected": collected_count,
            "timed_out": timed_out,
            "topic": topic,
            "results": collected_events,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!();
        println!("Dispatch {dispatch_id} complete:");
        println!(
            "  Workers: {count} spawned, {registered_count} registered, {collected_count} reported"
        );
        if timed_out {
            let missing: Vec<String> = worker_names
                .iter()
                .filter(|n| !collected_events.iter().any(|e| e["worker"].as_str() == Some(n)))
                .cloned()
                .collect();
            println!("  Timed out. Missing: {}", missing.join(", "));
        }

        for event in &collected_events {
            let worker = event["worker"].as_str().unwrap_or("?");
            let payload = &event["payload"];
            println!("  [{worker}] {}", serde_json::to_string(payload)?);
        }
    }

    if timed_out {
        std::process::exit(1);
    }
    Ok(())
}

fn resolve_spawn_backend(backend: &SpawnBackend) -> SpawnBackend {
    match backend {
        SpawnBackend::Auto => {
            #[cfg(target_os = "macos")]
            {
                if std::process::Command::new("pgrep")
                    .args(["-x", "WindowServer"])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    return SpawnBackend::Terminal;
                }
            }

            if std::process::Command::new("tmux")
                .arg("-V")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return SpawnBackend::Tmux;
            }

            SpawnBackend::Background
        }
        other => other.clone(),
    }
}

fn spawn_via_terminal(session_name: &str, shell_cmd: &str) -> Result<()> {
    let escaped_cmd = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
    let applescript = format!(
        r#"tell application "Terminal"
    activate
    do script "{escaped_cmd}"
end tell"#
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&applescript)
        .status()
        .context("Failed to run osascript")?;

    if !status.success() {
        anyhow::bail!(
            "Failed to open Terminal.app for worker '{}'",
            session_name
        );
    }
    Ok(())
}

fn spawn_via_tmux(session_name: &str, shell_cmd: &str) -> Result<()> {
    let tmux_session = format!("tl-{}", session_name);
    let status = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", &tmux_session, shell_cmd])
        .status()
        .context("Failed to run tmux")?;

    if !status.success() {
        anyhow::bail!(
            "Failed to create tmux session for worker '{}'",
            session_name
        );
    }
    Ok(())
}

fn spawn_via_background(session_name: &str, shell_cmd: &str) -> Result<()> {
    let child = std::process::Command::new("setsid")
        .args(["sh", "-c", shell_cmd])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn()
        .or_else(|_| {
            std::process::Command::new("sh")
                .args(["-c", shell_cmd])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .spawn()
        })
        .context("Failed to spawn background worker")?;

    let _ = child;
    let _ = session_name;
    Ok(())
}
