use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

use termlink_protocol::events::{
    file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
};

use crate::util::{generate_request_id, DEFAULT_CHUNK_SIZE};

pub(crate) async fn cmd_file_send(target: &str, path: &str, chunk_size: usize, json: bool, timeout_secs: u64) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
                std::process::exit(1);
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let file_path = std::path::Path::new(path);
    let file_data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Failed to read file '{}': {}", path, e)}));
                std::process::exit(1);
            }
            anyhow::bail!("Failed to read file '{}': {}", path, e);
        }
    };

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let size = file_data.len() as u64;
    let chunk_sz = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total_chunks = file_data.len().div_ceil(chunk_sz) as u32;

    let transfer_id = generate_request_id().replace("req-", "xfer-");

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let sha256 = format!("{:x}", hasher.finalize());

    // Emit file.init
    let init = FileInit {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        filename: filename.clone(),
        size,
        total_chunks,
        from: format!("cli-{}", std::process::id()),
    };
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);

    let init_payload = serde_json::to_value(&init)?;
    let emit_params = serde_json::json!({
        "topic": file_topic::INIT,
        "payload": init_payload,
    });
    let rpc_future = client::rpc_call(reg.socket_path(), "event.emit", emit_params);
    match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => { result.context("Failed to emit file.init")?; }
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("file.init timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("file.init timed out after {}s", timeout_secs);
        }
    }

    if !json {
        eprintln!(
            "Sending '{}' ({} bytes, {} chunks) transfer_id={}",
            filename, size, total_chunks, transfer_id
        );
    }

    // Emit chunks
    let encoder = base64::engine::general_purpose::STANDARD;
    for (i, chunk_data) in file_data.chunks(chunk_sz).enumerate() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: encoder.encode(chunk_data),
        };
        let chunk_payload = serde_json::to_value(&chunk)?;
        let emit_params = serde_json::json!({
            "topic": file_topic::CHUNK,
            "payload": chunk_payload,
        });
        let rpc_future = client::rpc_call(reg.socket_path(), "event.emit", emit_params);
        match tokio::time::timeout(timeout_dur, rpc_future).await {
            Ok(result) => { result.context(format!("Failed to emit chunk {}/{}", i + 1, total_chunks))?; }
            Err(_) => {
                if json {
                    println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("Chunk {}/{} timed out after {}s", i + 1, total_chunks, timeout_secs)}));
                    std::process::exit(1);
                }
                anyhow::bail!("Chunk {}/{} timed out after {}s", i + 1, total_chunks, timeout_secs);
            }
        }

        if !json && total_chunks > 1 {
            eprint!("\r  Chunk {}/{}", i + 1, total_chunks);
        }
    }
    if !json && total_chunks > 1 {
        eprintln!();
    }

    // Emit file.complete
    let complete = FileComplete {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        sha256: sha256.clone(),
    };
    let complete_payload = serde_json::to_value(&complete)?;
    let emit_params = serde_json::json!({
        "topic": file_topic::COMPLETE,
        "payload": complete_payload,
    });
    let rpc_future = client::rpc_call(reg.socket_path(), "event.emit", emit_params);
    match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => { result.context("Failed to emit file.complete")?; }
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({"ok": false, "target": target, "error": format!("file.complete timed out after {}s", timeout_secs)}));
                std::process::exit(1);
            }
            anyhow::bail!("file.complete timed out after {}s", timeout_secs);
        }
    }

    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "filename": filename,
            "size": size,
            "chunks": total_chunks,
            "transfer_id": transfer_id,
            "sha256": sha256,
            "target": target,
        }));
    } else {
        eprintln!("Transfer complete. SHA-256: {}", sha256);
    }
    Ok(())
}

pub(crate) async fn cmd_file_receive(
    target: &str,
    output_dir: &str,
    timeout: u64,
    interval: u64,
    json: bool,
) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let out_path = std::path::Path::new(output_dir);
    if !out_path.exists() {
        std::fs::create_dir_all(out_path)
            .context(format!("Failed to create output directory: {}", output_dir))?;
    }

    if !json {
        eprintln!("Waiting for file transfer on '{}' (timeout: {}s)...", target, timeout);
    }

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(interval);

    let mut poll_cursor: Option<u64> = None;
    let mut is_first_poll = true;

    let mut transfer_id: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut expected_chunks: u32 = 0;
    let mut chunks: std::collections::BTreeMap<u32, Vec<u8>> = std::collections::BTreeMap::new();

    let decoder = base64::engine::general_purpose::STANDARD;

    loop {
        let mut params = serde_json::json!({});
        if let Some(c) = poll_cursor {
            params["since"] = serde_json::json!(c);
        }

        let rpc_timeout = std::time::Duration::from_secs(10);
        let rpc_result = tokio::time::timeout(rpc_timeout, client::rpc_call(reg.socket_path(), "event.poll", params)).await;
        match rpc_result {
            Err(_) => {
                tracing::warn!("Poll RPC timed out (10s), retrying...");
                continue;
            }
            Ok(Err(e)) => {
                tracing::warn!("Poll error: {}", e);
            }
            Ok(Ok(resp)) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        if is_first_poll {
                            let mut last_init: Option<FileInit> = None;
                            for event in events.iter() {
                                let topic = event["topic"].as_str().unwrap_or("");
                                if topic == file_topic::INIT
                                    && let Ok(init) = serde_json::from_value::<FileInit>(event["payload"].clone()) {
                                        last_init = Some(init);
                                    }
                            }
                            if let Some(init) = last_init {
                                transfer_id = Some(init.transfer_id.clone());
                                filename = Some(init.filename.clone());
                                expected_chunks = init.total_chunks;
                                chunks.clear();
                                if !json {
                                    eprintln!(
                                        "Receiving '{}' ({} bytes, {} chunks) from {}",
                                        init.filename, init.size, init.total_chunks, init.from
                                    );
                                }
                                for event in events.iter() {
                                    let topic = event["topic"].as_str().unwrap_or("");
                                    let payload = &event["payload"];
                                    if topic == file_topic::CHUNK
                                        && let Ok(chunk) = serde_json::from_value::<FileChunk>(payload.clone())
                                            && transfer_id.as_deref() == Some(&chunk.transfer_id) {
                                                let decoded = decoder.decode(&chunk.data)
                                                    .context(format!("Invalid base64 in chunk {}", chunk.index))?;
                                                chunks.insert(chunk.index, decoded);
                                            }
                                }
                            }
                            is_first_poll = false;
                            if let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                            if transfer_id.is_some() && chunks.len() as u32 == expected_chunks {
                                for event in events.iter() {
                                    let topic = event["topic"].as_str().unwrap_or("");
                                    let payload = &event["payload"];
                                    if topic == file_topic::COMPLETE
                                        && let Ok(complete) = serde_json::from_value::<FileComplete>(payload.clone())
                                            && transfer_id.as_deref() == Some(&complete.transfer_id) {
                                                let mut file_data = Vec::new();
                                                for i in 0..expected_chunks {
                                                    match chunks.get(&i) {
                                                        Some(data) => file_data.extend_from_slice(data),
                                                        None => {
                                                            let msg = format!("Missing chunk {} of {}", i, expected_chunks);
                                                            if json {
                                                                println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg}));
                                                                std::process::exit(1);
                                                            }
                                                            anyhow::bail!("{}", msg);
                                                        }
                                                    }
                                                }
                                                let mut hasher = Sha256::new();
                                                hasher.update(&file_data);
                                                let actual_sha256 = format!("{:x}", hasher.finalize());
                                                if actual_sha256 != complete.sha256 {
                                                    let msg = format!("SHA-256 mismatch! Expected: {}, Got: {}", complete.sha256, actual_sha256);
                                                    if json {
                                                        println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg}));
                                                        std::process::exit(1);
                                                    }
                                                    anyhow::bail!("{}", msg);
                                                }
                                                let fname = filename.as_deref().unwrap_or("received-file");
                                                let dest = out_path.join(fname);
                                                std::fs::write(&dest, &file_data)
                                                    .context(format!("Failed to write file: {}", dest.display()))?;
                                                if json {
                                                    println!("{}", serde_json::json!({
                                                        "ok": true,
                                                        "filename": fname,
                                                        "path": dest.display().to_string(),
                                                        "size": file_data.len(),
                                                        "sha256": actual_sha256,
                                                        "transfer_id": transfer_id,
                                                        "target": target,
                                                    }));
                                                } else {
                                                    eprintln!("File saved: {} ({} bytes)", dest.display(), file_data.len());
                                                    eprintln!("SHA-256 verified: {}", actual_sha256);
                                                }
                                                return Ok(());
                                            }
                                }
                            }
                            continue;
                        }

                        for event in events {
                            let topic = event["topic"].as_str().unwrap_or("");
                            let payload = &event["payload"];

                            match topic {
                                t if t == file_topic::INIT => {
                                    if let Ok(init) = serde_json::from_value::<FileInit>(payload.clone()) {
                                        if !json {
                                            eprintln!(
                                                "Receiving '{}' ({} bytes, {} chunks) from {}",
                                                init.filename, init.size, init.total_chunks, init.from
                                            );
                                        }
                                        transfer_id = Some(init.transfer_id);
                                        filename = Some(init.filename);
                                        expected_chunks = init.total_chunks;
                                        chunks.clear();
                                    }
                                }
                                t if t == file_topic::CHUNK => {
                                    if let Ok(chunk) = serde_json::from_value::<FileChunk>(payload.clone())
                                        && transfer_id.as_deref() == Some(&chunk.transfer_id) {
                                            let decoded = decoder.decode(&chunk.data)
                                                .context(format!("Invalid base64 in chunk {}", chunk.index))?;
                                            chunks.insert(chunk.index, decoded);

                                            if !json && expected_chunks > 1 {
                                                eprint!("\r  Chunk {}/{}", chunks.len(), expected_chunks);
                                            }
                                        }
                                }
                                t if t == file_topic::COMPLETE => {
                                    if let Ok(complete) = serde_json::from_value::<FileComplete>(payload.clone())
                                        && transfer_id.as_deref() == Some(&complete.transfer_id) {
                                            if !json && expected_chunks > 1 {
                                                eprintln!();
                                            }

                                            let mut file_data = Vec::new();
                                            for i in 0..expected_chunks {
                                                match chunks.get(&i) {
                                                    Some(data) => file_data.extend_from_slice(data),
                                                    None => {
                                                        let msg = format!("Missing chunk {} of {}", i, expected_chunks);
                                                        if json {
                                                            println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg}));
                                                            std::process::exit(1);
                                                        }
                                                        anyhow::bail!("{}", msg);
                                                    }
                                                }
                                            }

                                            let mut hasher = Sha256::new();
                                            hasher.update(&file_data);
                                            let actual_sha256 = format!("{:x}", hasher.finalize());

                                            if actual_sha256 != complete.sha256 {
                                                let msg = format!(
                                                    "SHA-256 mismatch! Expected: {}, Got: {}",
                                                    complete.sha256, actual_sha256
                                                );
                                                if json {
                                                    println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg}));
                                                    std::process::exit(1);
                                                }
                                                anyhow::bail!("{}", msg);
                                            }

                                            let fname = filename.as_deref().unwrap_or("received-file");
                                            let dest = out_path.join(fname);
                                            std::fs::write(&dest, &file_data)
                                                .context(format!("Failed to write file: {}", dest.display()))?;

                                            if json {
                                                println!("{}", serde_json::json!({
                                                    "ok": true,
                                                    "filename": fname,
                                                    "path": dest.display().to_string(),
                                                    "size": file_data.len(),
                                                    "sha256": actual_sha256,
                                                    "transfer_id": transfer_id,
                                                    "target": target,
                                                }));
                                            } else {
                                                eprintln!("File saved: {} ({} bytes)", dest.display(), file_data.len());
                                                eprintln!("SHA-256 verified: {}", actual_sha256);
                                            }
                                            return Ok(());
                                        }
                                }
                                t if t == file_topic::ERROR => {
                                    if let Some(msg) = payload.get("message").and_then(|m| m.as_str()) {
                                        let xfer = payload.get("transfer_id").and_then(|t| t.as_str()).unwrap_or("?");
                                        if transfer_id.as_deref() == Some(xfer) || transfer_id.is_none() {
                                            let err_msg = format!("Transfer error: {}", msg);
                                            if json {
                                                println!("{}", serde_json::json!({"ok": false, "target": target, "error": err_msg}));
                                                std::process::exit(1);
                                            }
                                            anyhow::bail!("{}", err_msg);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(events) = result["events"].as_array()
                        && !events.is_empty()
                            && let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                }
            }
        }

        if start.elapsed() > timeout_dur {
            if transfer_id.is_some() {
                let msg = format!("Timeout: received {}/{} chunks before timeout ({}s)", chunks.len(), expected_chunks, timeout);
                if json {
                    println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg, "chunks_received": chunks.len(), "chunks_expected": expected_chunks}));
                    std::process::exit(1);
                }
                anyhow::bail!("{}", msg);
            } else {
                let msg = format!("Timeout waiting for file transfer ({}s)", timeout);
                if json {
                    println!("{}", serde_json::json!({"ok": false, "target": target, "error": msg}));
                    std::process::exit(1);
                }
                anyhow::bail!("{}", msg);
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}
