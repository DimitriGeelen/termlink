use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

pub(crate) async fn cmd_events(target: &str, since: Option<u64>, topic: Option<&str>, json: bool, timeout_secs: u64, payload_only: bool) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = serde_json::json!({});
    if let Some(s) = since {
        params["since"] = serde_json::json!(s);
    }
    if let Some(t) = topic {
        params["topic"] = serde_json::json!(t);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc = client::rpc_call(reg.socket_path(), "event.poll", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => r.context("Failed to connect to session")?,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Event poll timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Event poll timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }
            let events = result["events"].as_array().unwrap();

            if payload_only {
                for event in events {
                    let payload = &event["payload"];
                    if !payload.is_null() {
                        println!("{}", serde_json::to_string(payload)?);
                    }
                }
                return Ok(());
            }

            if events.is_empty() {
                println!("No events (next_seq: {})", result["next_seq"]);
                return Ok(());
            }

            for event in events {
                let seq = event["seq"].as_u64().unwrap_or(0);
                let topic = event["topic"].as_str().unwrap_or("?");
                let payload = &event["payload"];
                let ts = event["timestamp"].as_u64().unwrap_or(0);

                if payload.is_null() || (payload.is_object() && payload.as_object().unwrap().is_empty()) {
                    println!("[{seq}] {topic} (t={ts})");
                } else {
                    println!("[{seq}] {topic}: {} (t={ts})", serde_json::to_string(payload)?);
                }
            }
            println!();
            println!("{} event(s), next_seq: {}", result["count"], result["next_seq"]);
            Ok(())
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Event poll failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_emit(target: &str, topic: &str, payload_str: &str, json: bool, timeout_secs: u64) -> Result<()> {
    let payload: serde_json::Value =
        serde_json::from_str(payload_str).context("Invalid JSON payload")?;

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc = client::rpc_call(
        reg.socket_path(),
        "event.emit",
        serde_json::json!({ "topic": topic, "payload": payload }),
    );
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => r.context("Failed to connect to session")?,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Event emit timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Event emit timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "Event emitted: {} (seq: {})",
                    result["topic"].as_str().unwrap_or("?"),
                    result["seq"].as_u64().unwrap_or(0),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Event emit failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_broadcast(topic: &str, payload_str: &str, targets: Vec<String>, json: bool, timeout_secs: u64) -> Result<()> {
    let payload: serde_json::Value =
        serde_json::from_str(payload_str).context("Invalid JSON payload")?;

    let hub_socket = termlink_hub::server::hub_socket_path();
    if !hub_socket.exists() {
        if json {
            println!("{}", serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
            std::process::exit(1);
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    let mut params = serde_json::json!({
        "topic": topic,
        "payload": payload,
    });
    if !targets.is_empty() {
        params["targets"] = serde_json::json!(targets);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc = client::rpc_call(&hub_socket, "event.broadcast", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => r.context("Failed to connect to hub")?,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "topic": topic, "error": format!("Broadcast timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("Broadcast timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let targeted = result["targeted"].as_u64().unwrap_or(0);
                let succeeded = result["succeeded"].as_u64().unwrap_or(0);
                let failed = result["failed"].as_u64().unwrap_or(0);
                println!(
                    "Broadcast '{}': {}/{} succeeded{}",
                    result["topic"].as_str().unwrap_or(topic),
                    succeeded,
                    targeted,
                    if failed > 0 {
                        format!(" ({} failed)", failed)
                    } else {
                        String::new()
                    },
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "topic": topic, "error": format!("{e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("Broadcast failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_emit_to(
    target: &str,
    topic: &str,
    payload_str: &str,
    from: Option<&str>,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let payload: serde_json::Value =
        serde_json::from_str(payload_str).context("Invalid JSON payload")?;

    let hub_socket = termlink_hub::server::hub_socket_path();
    if !hub_socket.exists() {
        if json {
            println!("{}", serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
            std::process::exit(1);
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    let mut params = serde_json::json!({
        "target": target,
        "topic": topic,
        "payload": payload,
    });
    if let Some(sender) = from {
        params["from"] = serde_json::json!(sender);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc = client::rpc_call(&hub_socket, "event.emit_to", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc).await {
        Ok(r) => r.context("Failed to connect to hub")?,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("emit-to timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("emit-to timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "Pushed to {}: {} (seq: {})",
                    result["target"].as_str().unwrap_or(target),
                    result["topic"].as_str().unwrap_or(topic),
                    result["seq"].as_u64().unwrap_or(0),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("{e}")}));
                std::process::exit(1);
            }
            anyhow::bail!("emit-to failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_watch(
    targets: Vec<String>,
    interval_ms: u64,
    topic_filter: Option<&str>,
    json: bool,
    timeout_secs: u64,
    max_count: u64,
) -> Result<()> {
    use std::collections::HashMap;

    // Resolve targets: if empty, watch all live sessions
    let registrations = if targets.is_empty() {
        let sessions = manager::list_sessions(false)
            .context("Failed to list sessions")?;
        if sessions.is_empty() {
            if json {
                println!("{}", serde_json::json!({"ok": false, "error": "No active sessions to watch."}));
                std::process::exit(1);
            }
            anyhow::bail!("No active sessions to watch.");
        }
        sessions
            .iter()
            .filter_map(|s| manager::find_session(s.id.as_str()).ok())
            .collect::<Vec<_>>()
    } else {
        targets
            .iter()
            .map(|t| manager::find_session(t).context(format!("Session '{}' not found", t)))
            .collect::<Result<Vec<_>>>()?
    };

    if registrations.is_empty() {
        if json {
            println!("{}", serde_json::json!({"ok": false, "error": "No reachable sessions to watch."}));
            std::process::exit(1);
        }
        anyhow::bail!("No reachable sessions to watch.");
    }

    let session_names: HashMap<String, String> = registrations
        .iter()
        .map(|r| (r.id.as_str().to_string(), r.display_name.clone()))
        .collect();

    if !json {
        eprintln!(
            "Watching {} session(s): {}. Press Ctrl+C to stop.",
            registrations.len(),
            registrations
                .iter()
                .map(|r| r.display_name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
        if timeout_secs > 0 {
            eprintln!("  Timeout: {}s", timeout_secs);
        }
        eprintln!();
    }

    let mut cursors: HashMap<String, Option<u64>> = registrations
        .iter()
        .map(|r| (r.id.as_str().to_string(), None))
        .collect();

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);
    let deadline = if timeout_secs > 0 {
        Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };
    let mut total_received: u64 = 0;

    loop {
        if let Some(dl) = deadline {
            if std::time::Instant::now() >= dl {
                if !json {
                    eprintln!();
                    eprintln!("Stopped watching (timeout after {}s).", timeout_secs);
                }
                break;
            }
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if !json {
                    eprintln!();
                    eprintln!("Stopped watching.");
                }
                break;
            }
            _ = tokio::time::sleep(poll_interval) => {
                for reg in &registrations {
                    let sid = reg.id.as_str();
                    let name = session_names.get(sid).map(|s| s.as_str()).unwrap_or(sid);

                    let mut params = serde_json::json!({});
                    if let Some(cursor) = cursors.get(sid).and_then(|c| *c) {
                        params["since"] = serde_json::json!(cursor);
                    }
                    if let Some(t) = topic_filter {
                        params["topic"] = serde_json::json!(t);
                    }

                    let resp = match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                        Ok(r) => r,
                        Err(_) => {
                            continue;
                        }
                    };

                    if let Ok(result) = client::unwrap_result(resp) {
                        if let Some(events) = result["events"].as_array() {
                            for event in events {
                                let seq = event["seq"].as_u64().unwrap_or(0);
                                let topic = event["topic"].as_str().unwrap_or("?");
                                let payload = &event["payload"];
                                let ts = event["timestamp"].as_u64().unwrap_or(0);

                                if json {
                                    println!("{}", serde_json::json!({
                                        "session": name,
                                        "session_id": sid,
                                        "seq": seq,
                                        "topic": topic,
                                        "payload": payload,
                                        "timestamp": ts,
                                    }));
                                } else if payload.is_null()
                                    || (payload.is_object()
                                        && payload.as_object().unwrap().is_empty())
                                {
                                    println!("[{name}#{seq}] {topic} (t={ts})");
                                } else {
                                    println!(
                                        "[{name}#{seq}] {topic}: {} (t={ts})",
                                        serde_json::to_string(payload).unwrap_or_default()
                                    );
                                }

                                cursors.insert(sid.to_string(), Some(seq));
                                total_received += 1;
                            }
                        }
                        if let Some(next) = result["next_seq"].as_u64()
                            && cursors.get(sid).and_then(|c| *c).is_none() && next > 0 {
                                cursors.insert(sid.to_string(), Some(next.saturating_sub(1)));
                            }
                    }
                }

                if max_count > 0 && total_received >= max_count {
                    if !json {
                        eprintln!();
                        eprintln!("{} event(s) received (limit reached).", total_received);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

pub(crate) async fn cmd_wait(target: &str, topic: &str, timeout_secs: u64, interval_ms: u64, json: bool) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    if !json {
        eprintln!("Waiting for event topic '{}' from {}...", topic, reg.display_name);
    }

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);
    let deadline = if timeout_secs > 0 {
        Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };

    let mut cursor: Option<u64> = None;

    loop {
        if let Some(dl) = deadline
            && tokio::time::Instant::now() >= dl {
                if json {
                    println!("{}", serde_json::json!({
                        "matched": false,
                        "topic": topic,
                        "target": target,
                        "reason": "timeout",
                        "timeout_secs": timeout_secs,
                    }));
                    std::process::exit(1);
                }
                anyhow::bail!("Timeout waiting for event topic '{}'", topic);
            }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if json {
                    println!("{}", serde_json::json!({
                        "matched": false,
                        "topic": topic,
                        "target": target,
                        "reason": "interrupted",
                    }));
                    std::process::exit(1);
                }
                anyhow::bail!("Interrupted");
            }
            _ = tokio::time::sleep(poll_interval) => {
                let mut params = serde_json::json!({ "topic": topic });
                if let Some(c) = cursor {
                    params["since"] = serde_json::json!(c);
                }
                let resp = match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                    Ok(r) => r,
                    Err(_) => {
                        if json {
                            println!("{}", serde_json::json!({
                                "matched": false,
                                "topic": topic,
                                "target": target,
                                "reason": "disconnected",
                            }));
                            std::process::exit(1);
                        }
                        anyhow::bail!("Session '{}' disconnected while waiting", target);
                    }
                };

                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array()
                        && let Some(event) = events.first() {
                            if json {
                                println!("{}", serde_json::json!({
                                    "matched": true,
                                    "topic": event["topic"],
                                    "seq": event["seq"],
                                    "timestamp": event["timestamp"],
                                    "payload": event["payload"],
                                    "target": target,
                                }));
                            } else {
                                let payload = &event["payload"];
                                if payload.is_null()
                                    || (payload.is_object()
                                        && payload.as_object().unwrap().is_empty())
                                {
                                    println!("{}", topic);
                                } else {
                                    println!("{}", serde_json::to_string(payload)?);
                                }
                            }
                            return Ok(());
                        }
                    if let Some(next) = result["next_seq"].as_u64() {
                        cursor = if next > 0 { Some(next - 1) } else { None };
                    }
                }
            }
        }
    }
}

pub(crate) async fn cmd_topics(target: Option<&str>, json: bool, timeout_secs: u64) -> Result<()> {
    use std::collections::BTreeMap;

    let registrations = if let Some(t) = target {
        vec![manager::find_session(t).context(format!("Session '{}' not found", t))?]
    } else {
        manager::list_sessions(false).context("Failed to list sessions")?
    };

    if registrations.is_empty() {
        if json {
            println!("{}", serde_json::json!({"sessions": [], "total_topics": 0}));
        } else {
            println!("No active sessions.");
        }
        return Ok(());
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let mut session_topics: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for reg in &registrations {
        let rpc_future = client::rpc_call(reg.socket_path(), "event.topics", serde_json::json!({}));
        match tokio::time::timeout(timeout_dur, rpc_future).await {
            Ok(Ok(resp)) => {
                if let Ok(result) = client::unwrap_result(resp)
                    && let Some(topics) = result["topics"].as_array() {
                        let topic_list: Vec<String> = topics
                            .iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect();
                        if !topic_list.is_empty() {
                            session_topics
                                .insert(reg.display_name.clone(), topic_list);
                        }
                    }
            }
            Ok(Err(_)) | Err(_) => continue,
        }
    }

    let total: usize = session_topics.values().map(|v| v.len()).sum();

    if json {
        let sessions: Vec<serde_json::Value> = session_topics
            .iter()
            .map(|(name, topics)| serde_json::json!({"session": name, "topics": topics}))
            .collect();
        println!("{}", serde_json::json!({
            "sessions": sessions,
            "total_topics": total,
            "total_sessions": session_topics.len(),
        }));
        return Ok(());
    }

    if session_topics.is_empty() {
        println!("No event topics found.");
        return Ok(());
    }

    for (name, topics) in &session_topics {
        println!("{}:", name);
        for topic in topics {
            println!("  {}", topic);
        }
    }

    println!();
    println!(
        "{} topic(s) across {} session(s)",
        total,
        session_topics.len()
    );
    Ok(())
}

pub(crate) async fn cmd_collect(
    targets: Vec<String>,
    topic_filter: Option<&str>,
    interval_ms: u64,
    max_count: u64,
    json: bool,
    timeout_secs: u64,
) -> Result<()> {
    let hub_socket = termlink_hub::server::hub_socket_path();
    if !hub_socket.exists() {
        if json {
            println!("{}", serde_json::json!({"ok": false, "error": "Hub is not running. Start it with: termlink hub"}));
            std::process::exit(1);
        }
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    if !json {
        eprintln!("Collecting events via hub. Press Ctrl+C to stop.");
        if let Some(t) = topic_filter {
            eprintln!("  Topic filter: {}", t);
        }
        if !targets.is_empty() {
            eprintln!("  Targets: {}", targets.join(", "));
        }
        if timeout_secs > 0 {
            eprintln!("  Timeout: {}s", timeout_secs);
        }
        eprintln!();
    }

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);
    let deadline = if timeout_secs > 0 {
        Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };
    let mut cursors = serde_json::json!({});
    let mut total_received: u64 = 0;

    loop {
        if let Some(dl) = deadline {
            if std::time::Instant::now() >= dl {
                if !json {
                    eprintln!();
                    eprintln!("{} event(s) collected (timeout after {}s).", total_received, timeout_secs);
                }
                break;
            }
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if !json {
                    eprintln!();
                    eprintln!("Stopped. {} event(s) collected.", total_received);
                }
                break;
            }
            _ = tokio::time::sleep(poll_interval) => {
                let mut params = serde_json::json!({});
                if !targets.is_empty() {
                    params["targets"] = serde_json::json!(targets);
                }
                if let Some(t) = topic_filter {
                    params["topic"] = serde_json::json!(t);
                }
                if !cursors.as_object().unwrap_or(&serde_json::Map::new()).is_empty() {
                    params["since"] = cursors.clone();
                }

                let resp = match client::rpc_call(&hub_socket, "event.collect", params).await {
                    Ok(r) => r,
                    Err(e) => {
                        if !json {
                            eprintln!("Hub connection error: {}. Retrying...", e);
                        }
                        continue;
                    }
                };

                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let session_name = event["session_name"].as_str().unwrap_or("?");
                            let seq = event["seq"].as_u64().unwrap_or(0);
                            let topic = event["topic"].as_str().unwrap_or("?");
                            let payload = &event["payload"];
                            let ts = event["timestamp"].as_u64().unwrap_or(0);

                            if json {
                                println!("{}", serde_json::json!({
                                    "session": session_name,
                                    "seq": seq,
                                    "topic": topic,
                                    "payload": payload,
                                    "timestamp": ts,
                                }));
                            } else if payload.is_null()
                                || (payload.is_object()
                                    && payload.as_object().unwrap().is_empty())
                            {
                                println!("[{session_name}#{seq}] {topic} (t={ts})");
                            } else {
                                println!(
                                    "[{session_name}#{seq}] {topic}: {} (t={ts})",
                                    serde_json::to_string(payload).unwrap_or_default()
                                );
                            }

                            total_received += 1;
                        }
                    }

                    if let Some(new_cursors) = result.get("cursors")
                        && let Some(obj) = new_cursors.as_object() {
                            for (k, v) in obj {
                                cursors[k] = v.clone();
                            }
                        }

                    if max_count > 0 && total_received >= max_count {
                        if !json {
                            eprintln!();
                            eprintln!("{} event(s) collected (limit reached).", total_received);
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
