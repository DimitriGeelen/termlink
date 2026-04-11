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

/// Options for the `termlink dispatch` command.
pub(crate) struct DispatchOpts {
    pub count: u32,
    pub timeout: u64,
    pub topic: String,
    pub name_prefix: Option<String>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
    pub cap: Vec<String>,
    pub env_vars: Vec<String>,
    pub backend: SpawnBackend,
    pub workdir: Option<std::path::PathBuf>,
    pub isolate: bool,
    pub auto_merge: bool,
    pub json_output: bool,
    pub command: Vec<String>,
}

/// Run the `termlink dispatch` command.
pub(crate) async fn cmd_dispatch(opts: DispatchOpts) -> Result<()> {
    let DispatchOpts { count, timeout, topic, name_prefix, roles, tags, cap, env_vars, backend, workdir, isolate, auto_merge, json_output, command } = opts;
    if count == 0 {
        if json_output {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "--count must be at least 1"}));
        }
        anyhow::bail!("--count must be at least 1");
    }
    if command.is_empty() {
        if json_output {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Command required after --"}));
        }
        anyhow::bail!("Command required after --");
    }

    // Validate --workdir if provided
    let resolved_workdir = if let Some(ref wd) = workdir {
        let canonical = wd.canonicalize().with_context(|| {
            format!("--workdir path does not exist or is not accessible: {}", wd.display())
        })?;
        if !canonical.is_dir() {
            if json_output {
                super::json_error_exit(serde_json::json!({"ok": false, "error": format!("--workdir is not a directory: {}", wd.display())}));
            }
            anyhow::bail!("--workdir is not a directory: {}", wd.display());
        }
        Some(canonical)
    } else {
        None
    };

    // Validate --auto-merge requires --isolate
    if auto_merge && !isolate {
        if json_output {
            super::json_error_exit(json!({"ok": false, "error": "--auto-merge requires --isolate"}));
        }
        anyhow::bail!("--auto-merge requires --isolate");
    }

    // Validate --isolate and --workdir mutual exclusion (before git check so error is precise)
    if isolate && workdir.is_some() {
        if json_output {
            super::json_error_exit(json!({"ok": false, "error": "--isolate and --workdir are mutually exclusive (--isolate sets workdir automatically)"}));
        }
        anyhow::bail!("--isolate and --workdir are mutually exclusive (--isolate sets workdir automatically)");
    }

    // Validate --isolate requires a git repository
    let project_root = std::env::current_dir().context("Failed to get current directory")?;
    if isolate && !crate::manifest::is_git_repo(&project_root) {
        if json_output {
            super::json_error_exit(json!({"ok": false, "error": "--isolate requires a git repository"}));
        }
        anyhow::bail!("--isolate requires a git repository");
    }

    // Check hub is running (needed for collect).
    // File-existence is necessary but not sufficient — a stale socket file
    // can persist after the hub process dies. Verify by actually attempting
    // to connect; if no one is accept()ing, fail fast (T-916).
    let hub_socket = termlink_hub::server::hub_socket_path();
    let hub_alive = hub_socket.exists()
        && tokio::net::UnixStream::connect(&hub_socket).await.is_ok();
    if !hub_alive {
        if json_output {
            super::json_error_exit(serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub start (dispatch requires the hub for event collection)"}));
        }
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

    // === Worktree isolation setup ===
    let mut worktree_branches: Vec<crate::manifest::BranchEntry> = Vec::new();
    let base_branch = if isolate {
        let bb = crate::manifest::current_branch(&project_root)?;
        if !json_output {
            eprintln!("Dispatch {dispatch_id}: creating {count} worktree(s) from {bb}...");
        }

        // Create worktrees and record branches
        for i in 1..=count {
            let worker_name = format!("{prefix}-{i}");
            let branch_name = format!("tl-dispatch/{dispatch_id}/{worker_name}");
            let worktree_path = crate::manifest::create_worktree(&project_root, &branch_name)?;

            if !json_output {
                eprintln!("  Worktree: {} → {}", branch_name, worktree_path.display());
            }

            worktree_branches.push(crate::manifest::BranchEntry {
                worker_name,
                branch_name,
                base_branch: bb.clone(),
                worktree_path: worktree_path.to_string_lossy().to_string(),
                has_commits: false,
            });
        }

        // Write dispatch manifest BEFORE spawning workers
        let mut manifest = crate::manifest::DispatchManifest::load(&project_root)?;
        manifest.add_dispatch(crate::manifest::DispatchRecord {
            id: dispatch_id.clone(),
            created_at: crate::manifest::now_rfc3339(),
            status: crate::manifest::DispatchStatus::Pending,
            worker_count: count,
            topic: topic.to_string(),
            prefix: prefix.clone(),
            branches: worktree_branches.clone(),
        });
        manifest.save(&project_root)?;

        Some(bb)
    } else {
        None
    };

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
        let termlink_bin = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                if json_output {
                    super::json_error_exit(serde_json::json!({"ok": false, "error": format!("Failed to determine termlink binary path: {}", e)}));
                }
                return Err(e.into());
            }
        };
        let termlink_path = termlink_bin.to_string_lossy().to_string();

        let mut register_args = vec![
            "register".to_string(),
            "--name".to_string(),
            worker_name.clone(),
            "--tags".to_string(),
            worker_tags.join(","),
        ];
        if !roles.is_empty() {
            register_args.push("--roles".to_string());
            register_args.push(roles.join(","));
        }
        if !cap.is_empty() {
            register_args.push("--cap".to_string());
            register_args.push(cap.join(","));
        }
        // Note: no --shell flag. Dispatch workers are event-only sessions
        // (no PTY needed). This avoids PTY exhaustion on macOS when spawning
        // many workers simultaneously.

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
            env.push_str(&format!(
                "export TERMLINK_WORKER_NAME={}; ",
                shell_escape(&worker_name)
            ));
            // Effective workdir: --isolate worktree path takes precedence over --workdir
            let effective_workdir = if isolate {
                worktree_branches
                    .iter()
                    .find(|b| b.worker_name == worker_name)
                    .map(|b| b.worktree_path.clone())
            } else {
                resolved_workdir.as_ref().map(|wd| wd.to_string_lossy().to_string())
            };
            if let Some(ref wd) = effective_workdir {
                env.push_str(&format!(
                    "export TERMLINK_WORKDIR={}; ",
                    shell_escape(wd)
                ));
            }
            if isolate {
                env.push_str(&format!(
                    "export CARGO_TARGET_DIR={}; ",
                    shell_escape(&format!(
                        "{}/target",
                        effective_workdir.as_deref().unwrap_or(".")
                    ))
                ));
            }
            // User-supplied --env KEY=VALUE pairs
            for kv in &env_vars {
                if let Some((key, val)) = kv.split_once('=') {
                    env.push_str(&format!(
                        "export {}={}; ",
                        shell_escape(key),
                        shell_escape(val)
                    ));
                }
            }
            env
        };

        let mut reg_parts = vec![termlink_path.clone()];
        reg_parts.extend(register_args);

        // Worker captures user_cmd's exit code, then terminates the registrar
        // so dispatch's early-crash detection (find_session check) sees the
        // worker as gone. Without this, a fast-failing user_cmd would leave
        // sh blocked on `wait $TL_PID` forever (G-002).
        let effective_workdir = if isolate {
            worktree_branches
                .iter()
                .find(|b| b.worker_name == worker_name)
                .map(|b| b.worktree_path.clone())
        } else {
            resolved_workdir.as_ref().map(|wd| wd.to_string_lossy().to_string())
        };
        let cd_prefix = if let Some(ref wd) = effective_workdir {
            format!("cd {} && ", shell_escape(wd))
        } else {
            String::new()
        };
        let shell_cmd = build_worker_shell_cmd(
            &cd_prefix,
            &env_prefix,
            &reg_parts.join(" "),
            &user_cmd,
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
    let subscribe_timeout_ms: u64 = 500;
    let collect_start = std::time::Instant::now();
    let mut cursors = json!({});
    let mut collected_events = Vec::new();
    let mut crashed_workers: Vec<String> = Vec::new();
    // Track consecutive event.collect failures so we can bail when the hub
    // becomes unreachable mid-dispatch instead of spinning until --timeout.
    // Reset on each successful call. Bail at MAX_CONSECUTIVE (T-916).
    let mut consecutive_collect_errors: u32 = 0;
    const MAX_CONSECUTIVE_COLLECT_ERRORS: u32 = 5;

    loop {
        if collected_events.len() as u64 >= registered_count {
            break;
        }
        if collect_start.elapsed() > collect_timeout {
            break;
        }

        // Filter to our dispatch workers by tag
        // Use timeout_ms for push-based delivery (hub subscribes to sessions)
        let mut params = json!({
            "topic": topic,
            "timeout_ms": subscribe_timeout_ms,
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
            Ok(r) => {
                consecutive_collect_errors = 0;
                r
            }
            Err(e) => {
                tracing::debug!(error = %e, "Collect error");
                consecutive_collect_errors += 1;
                if consecutive_collect_errors >= MAX_CONSECUTIVE_COLLECT_ERRORS {
                    if !json_output {
                        eprintln!(
                            "Hub unreachable after {MAX_CONSECUTIVE_COLLECT_ERRORS} consecutive event.collect failures (last error: {e}); aborting collection."
                        );
                    }
                    break;
                }
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

            // Only advance cursors when events were actually returned.
            // Hub returns cursors even for empty polls (next_seq from sessions),
            // which would cause us to skip seq 0 events on the next poll.
            let has_events = result["events"]
                .as_array()
                .is_some_and(|a| !a.is_empty());
            if has_events
                && let Some(new_cursors) = result.get("cursors")
                    && let Some(obj) = new_cursors.as_object()
                {
                    for (k, v) in obj {
                        cursors[k] = v.clone();
                    }
                }
        }

        // event.collect with timeout_ms blocks server-side; no sleep needed

        // Early crash detection: check if remaining workers are still alive.
        // Workers that have already emitted results or are already known-dead are skipped.
        let mut alive_remaining = 0u64;
        for (i, name) in worker_names.iter().enumerate() {
            if !registered[i] {
                continue;
            }
            let has_result = collected_events.iter().any(|e| e["worker"].as_str() == Some(name.as_str()));
            let already_dead = crashed_workers.iter().any(|d| d == name);
            if has_result || already_dead {
                continue;
            }
            if manager::find_session(name).is_err() {
                if !json_output {
                    eprintln!("  Warning: {name} exited without emitting result");
                }
                crashed_workers.push(name.clone());
            } else {
                alive_remaining += 1;
            }
        }
        if !crashed_workers.is_empty() && alive_remaining == 0 {
            if !json_output {
                eprintln!("All remaining workers exited — stopping collection early.");
            }
            break;
        }
    }

    // Cleanup: signal all workers to exit
    for name in &worker_names {
        if let Ok(reg) = manager::find_session(name) {
            // SAFETY: reg.pid is a valid PID from a session we spawned.
            // SIGTERM is a standard signal; sending it to our own child is safe.
            unsafe {
                libc::kill(reg.pid as i32, libc::SIGTERM);
            }
        }
    }

    // Worktree cleanup: auto-commit and remove worktrees
    let mut branch_names_created: Vec<String> = Vec::new();
    if isolate {
        // Give workers a moment to finish after SIGTERM
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        if !json_output {
            eprintln!("Cleaning up worktrees...");
        }

        let mut manifest = crate::manifest::DispatchManifest::load(&project_root)?;

        for branch in &mut worktree_branches {
            let wt_path = std::path::Path::new(&branch.worktree_path);

            // Auto-commit any changes in the worktree
            let has_commits = if wt_path.exists() {
                match crate::manifest::auto_commit_worktree(wt_path, &branch.worker_name) {
                    Ok(committed) => committed,
                    Err(e) => {
                        if !json_output {
                            eprintln!("  Warning: auto-commit failed for {}: {e}", branch.worker_name);
                        }
                        false
                    }
                }
            } else {
                false
            };

            branch.has_commits = has_commits;

            if has_commits {
                branch_names_created.push(branch.branch_name.clone());
                if !json_output {
                    eprintln!("  {} — committed, branch preserved", branch.branch_name);
                }
            } else if !json_output {
                eprintln!("  {} — no changes, branch removed", branch.branch_name);
            }

            // Remove worktree (branch preserved if commits exist)
            if let Err(e) = crate::manifest::cleanup_worktree(
                &project_root,
                wt_path,
                &branch.branch_name,
                has_commits,
            ) && !json_output {
                eprintln!("  Warning: cleanup failed for {}: {e}", branch.worker_name);
            }
        }

        // Update manifest with commit status
        if let Some(record) = manifest.find_dispatch_mut(&dispatch_id) {
            record.branches = worktree_branches.clone();
            // If all branches have no commits, mark as merged (nothing to merge)
            if worktree_branches.iter().all(|b| !b.has_commits) {
                record.status = crate::manifest::DispatchStatus::Merged;
            }
        }
        manifest.save(&project_root)?;

        // Auto-merge if requested
        if auto_merge {
            let branches_to_merge: Vec<_> = worktree_branches
                .iter()
                .filter(|b| b.has_commits)
                .collect();

            if branches_to_merge.is_empty() {
                if !json_output {
                    eprintln!("No branches to merge (all workers had no changes).");
                }
            } else {
                if !json_output {
                    eprintln!(
                        "Auto-merging {} branch(es) into {}...",
                        branches_to_merge.len(),
                        branches_to_merge[0].base_branch
                    );
                }

                let mut merge_results: Vec<serde_json::Value> = Vec::new();
                let mut all_merged = true;

                for branch in &branches_to_merge {
                    match crate::manifest::merge_branch(
                        &project_root,
                        &branch.branch_name,
                        &branch.base_branch,
                    ) {
                        Ok(true) => {
                            if !json_output {
                                eprintln!("  {} — merged", branch.branch_name);
                            }
                            merge_results.push(json!({
                                "branch": branch.branch_name,
                                "status": "merged",
                            }));
                        }
                        Ok(false) => {
                            all_merged = false;
                            if !json_output {
                                eprintln!(
                                    "  {} — CONFLICT (branch preserved for manual merge)",
                                    branch.branch_name
                                );
                            }
                            merge_results.push(json!({
                                "branch": branch.branch_name,
                                "status": "conflict",
                            }));
                        }
                        Err(e) => {
                            all_merged = false;
                            if !json_output {
                                eprintln!("  {} — ERROR: {e}", branch.branch_name);
                            }
                            merge_results.push(json!({
                                "branch": branch.branch_name,
                                "status": "error",
                                "error": e.to_string(),
                            }));
                        }
                    }
                }

                // Update manifest with merge results
                let mut manifest = crate::manifest::DispatchManifest::load(&project_root)?;
                if let Some(record) = manifest.find_dispatch_mut(&dispatch_id) {
                    if all_merged {
                        record.status = crate::manifest::DispatchStatus::Merged;
                    } else {
                        record.status = crate::manifest::DispatchStatus::Conflict;
                    }
                }
                manifest.save(&project_root)?;

                // Store merge results for JSON output
                if json_output {
                    // Will be added to output below
                    branch_names_created.clear();
                    for mr in &merge_results {
                        if mr["status"].as_str() == Some("conflict") {
                            branch_names_created.push(
                                mr["branch"].as_str().unwrap_or("?").to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    // Output results
    let collected_count = collected_events.len() as u64;
    let timed_out = collected_count < registered_count;
    let total_elapsed = collect_start.elapsed();
    let total_elapsed_secs = total_elapsed.as_secs_f64();

    // Compute per-worker elapsed from event timestamps relative to collect_start
    let collect_start_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(total_elapsed.as_secs());
    for event in &mut collected_events {
        if let Some(ts) = event["timestamp"].as_u64() {
            let worker_elapsed = ts.saturating_sub(collect_start_unix);
            event["elapsed_secs"] = json!(worker_elapsed);
        }
    }

    if json_output {
        let mut result = json!({
            "ok": !timed_out && crashed_workers.is_empty(),
            "dispatch_id": dispatch_id,
            "workers_spawned": count,
            "workers_registered": registered_count,
            "events_collected": collected_count,
            "timed_out": timed_out,
            "elapsed_secs": (total_elapsed_secs * 10.0).round() / 10.0,
            "topic": topic,
            "results": collected_events,
        });
        if !crashed_workers.is_empty() {
            result["crashed_workers"] = json!(crashed_workers);
        }
        if let Some(ref wd) = resolved_workdir {
            result["workdir"] = json!(wd.to_string_lossy());
        }
        if isolate {
            result["branches"] = json!(branch_names_created);
            if let Some(ref bb) = base_branch {
                result["base_branch"] = json!(bb);
            }
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!();
        println!("Dispatch {dispatch_id} complete ({:.1}s):", total_elapsed_secs);
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
            let elapsed = event["elapsed_secs"].as_u64().map(|s| format!(" ({s}s)")).unwrap_or_default();
            println!("  [{worker}]{elapsed} {}", serde_json::to_string(payload)?);
        }

        if isolate && !branch_names_created.is_empty() {
            println!("  Branches with changes:");
            for branch in &branch_names_created {
                println!("    {branch}");
            }
        }
    }

    if timed_out {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(1);
    }
    Ok(())
}

/// Show dispatch manifest status.
pub(crate) fn cmd_dispatch_status(check: bool, json_output: bool) -> Result<()> {
    let project_root = std::env::current_dir().context("Failed to get current directory")?;
    let manifest = crate::manifest::DispatchManifest::load(&project_root)?;

    let pending = manifest.count_by_status(&crate::manifest::DispatchStatus::Pending);
    let merged = manifest.count_by_status(&crate::manifest::DispatchStatus::Merged);
    let conflict = manifest.count_by_status(&crate::manifest::DispatchStatus::Conflict);
    let deferred = manifest.count_by_status(&crate::manifest::DispatchStatus::Deferred);
    let expired = manifest.count_by_status(&crate::manifest::DispatchStatus::Expired);
    let total = manifest.dispatches.len();

    if json_output {
        let result = json!({
            "ok": pending == 0,
            "total": total,
            "pending": pending,
            "merged": merged,
            "conflict": conflict,
            "deferred": deferred,
            "expired": expired,
            "pending_dispatches": manifest.pending_dispatches().iter().map(|d| {
                json!({
                    "id": d.id,
                    "created_at": d.created_at,
                    "worker_count": d.worker_count,
                    "branches": d.branches.iter()
                        .filter(|b| b.has_commits)
                        .map(|b| &b.branch_name)
                        .collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if total == 0 {
        println!("No dispatch manifest (no dispatches have used --isolate yet).");
    } else {
        println!("Dispatch manifest status:");
        println!("  Total:    {total}");
        println!("  Pending:  {pending}");
        println!("  Merged:   {merged}");
        println!("  Conflict: {conflict}");
        if deferred > 0 {
            println!("  Deferred: {deferred}");
        }
        if expired > 0 {
            println!("  Expired:  {expired}");
        }

        if pending > 0 {
            println!();
            println!("Pending dispatches:");
            for d in manifest.pending_dispatches() {
                let branches_with_commits: Vec<_> = d
                    .branches
                    .iter()
                    .filter(|b| b.has_commits)
                    .collect();
                println!("  {} (created: {}, {} branch(es) with changes)",
                    d.id, d.created_at, branches_with_commits.len());
                for b in &branches_with_commits {
                    println!("    {}", b.branch_name);
                }
            }
        }

        if conflict > 0 {
            println!();
            println!("Conflicting dispatches (need manual merge):");
            for d in &manifest.dispatches {
                if d.status == crate::manifest::DispatchStatus::Conflict {
                    println!("  {}", d.id);
                }
            }
        }
    }

    if check && pending > 0 {
        if !json_output {
            eprintln!();
            eprintln!("ERROR: {} pending dispatch(es). Merge or acknowledge before committing.", pending);
            eprintln!("Run: termlink dispatch-status --json for details");
        }
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

/// Build the shell template that wraps a dispatched worker.
///
/// The template starts the registrar in the background, waits for it to bind,
/// runs the user's command, captures its exit code, then terminates the
/// registrar so dispatch's early-crash detection sees the worker as gone.
/// Worker exits with the user_cmd's rc, not the registrar's.
///
/// Without the explicit kill + exit, a fast-failing user_cmd would leave sh
/// blocked on `wait $TL_PID` forever (registrar is long-lived). See G-002.
fn build_worker_shell_cmd(
    cd_prefix: &str,
    env_prefix: &str,
    register_cmd: &str,
    user_cmd: &str,
) -> String {
    format!(
        "{cd_prefix}{env_prefix}{register_cmd} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nUSER_RC=$?\nkill $TL_PID 2>/dev/null\nwait $TL_PID 2>/dev/null\nexit $USER_RC"
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_opts() -> DispatchOpts {
        DispatchOpts {
            count: 1, timeout: 5, topic: "task.completed".into(), name_prefix: None,
            roles: vec![], tags: vec![], cap: vec![], env_vars: vec![],
            backend: SpawnBackend::Background,
            workdir: None, isolate: false, auto_merge: false, json_output: false,
            command: vec!["echo".into(), "hello".into()],
        }
    }

    #[test]
    fn worker_shell_cmd_captures_exit_kills_registrar() {
        let cmd = build_worker_shell_cmd(
            "cd /tmp && ",
            "export FOO=bar; ",
            "termlink register --name w1",
            "bash -c 'exit 42'",
        );
        // Sanity: all the inputs are present.
        assert!(cmd.contains("cd /tmp && "));
        assert!(cmd.contains("export FOO=bar; "));
        assert!(cmd.contains("termlink register --name w1 &"));
        assert!(cmd.contains("bash -c 'exit 42'"));
        // G-002 fix invariants:
        assert!(cmd.contains("USER_RC=$?"), "must capture user_cmd exit code");
        assert!(cmd.contains("kill $TL_PID"), "must kill registrar after user_cmd");
        assert!(cmd.contains("exit $USER_RC"), "worker must exit with user_cmd rc");
    }

    #[test]
    fn worker_shell_cmd_last_line_is_exit_user_rc() {
        // Regression for G-002: previous template ended with `wait $TL_PID`,
        // which blocked forever when user_cmd fast-failed.
        let cmd = build_worker_shell_cmd("", "", "termlink register", "true");
        let last = cmd.lines().last().expect("non-empty shell cmd");
        assert_eq!(
            last, "exit $USER_RC",
            "last line must be `exit $USER_RC`, was `{last}` (G-002 regression)"
        );
    }

    #[tokio::test]
    async fn workdir_rejects_nonexistent_path() {
        let result = cmd_dispatch(DispatchOpts {
            workdir: Some(std::path::PathBuf::from("/nonexistent/path/that/does/not/exist")),
            ..test_opts()
        }).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("does not exist"),
            "Error should mention path does not exist, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn workdir_rejects_file_not_directory() {
        // Create a temp file (not directory)
        let tmp = std::env::temp_dir().join("termlink-test-workdir-file");
        std::fs::write(&tmp, "not a directory").unwrap();
        let result = cmd_dispatch(DispatchOpts {
            workdir: Some(tmp.clone()),
            ..test_opts()
        }).await;
        std::fs::remove_file(&tmp).ok();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not a directory"),
            "Error should mention not a directory, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn workdir_none_accepted() {
        // With workdir=None, the command should proceed past validation
        // (it will fail at hub check, which is fine for this test)
        let result = cmd_dispatch(test_opts()).await;
        // Should fail at "hub not running", not at workdir validation
        if let Err(e) = result {
            assert!(
                e.to_string().contains("Hub is not running"),
                "Expected hub error, got: {e}"
            );
        }
    }

    #[tokio::test]
    async fn workdir_valid_directory_accepted() {
        // Should proceed past workdir validation to hub check
        let result = cmd_dispatch(DispatchOpts {
            workdir: Some(std::env::temp_dir()),
            ..test_opts()
        }).await;
        if let Err(e) = result {
            assert!(
                e.to_string().contains("Hub is not running"),
                "Expected hub error after workdir validation passed, got: {e}"
            );
        }
    }

    #[tokio::test]
    async fn dispatch_rejects_zero_count() {
        let result = cmd_dispatch(DispatchOpts {
            count: 0,
            ..test_opts()
        }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 1"));
    }

    #[tokio::test]
    async fn dispatch_rejects_empty_command() {
        let result = cmd_dispatch(DispatchOpts {
            command: vec![],
            ..test_opts()
        }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Command required"));
    }

    #[tokio::test]
    async fn isolate_rejects_non_git_dir() {
        let tmp = tempfile::tempdir().unwrap();
        // Run from a non-git temp dir
        let _guard = std::env::set_current_dir(tmp.path());
        let result = cmd_dispatch(DispatchOpts {
            isolate: true,
            ..test_opts()
        }).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("git repository"),
            "Expected git repo error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn isolate_and_workdir_mutually_exclusive() {
        let result = cmd_dispatch(DispatchOpts {
            workdir: Some(std::env::temp_dir()),
            isolate: true,
            ..test_opts()
        }).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("mutually exclusive"),
            "Expected mutual exclusion error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn auto_merge_requires_isolate() {
        let result = cmd_dispatch(DispatchOpts {
            auto_merge: true,
            ..test_opts()
        }).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("requires --isolate"),
            "Expected requires isolate error, got: {err_msg}"
        );
    }
}
