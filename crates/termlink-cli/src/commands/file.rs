use std::path::PathBuf;

use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::manager;

use termlink_protocol::events::{
    file_topic, FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
};

use crate::util::{generate_request_id, DEFAULT_CHUNK_SIZE};

/// Delivery route for file transfer events.
enum DeliveryRoute {
    /// Direct to session socket.
    Direct(PathBuf),
    /// Via hub's event.emit_to (session offline, may trigger inbox spooling).
    Hub { hub_socket: PathBuf, target: String },
}

impl DeliveryRoute {
    /// Send a file event via the appropriate route.
    async fn emit(
        &self,
        topic: &str,
        payload: serde_json::Value,
        from: Option<&str>,
        timeout: std::time::Duration,
    ) -> Result<serde_json::Value, anyhow::Error> {
        match self {
            DeliveryRoute::Direct(socket) => {
                let params = serde_json::json!({
                    "topic": topic,
                    "payload": payload,
                });
                let fut = client::rpc_call(socket, "event.emit", params);
                tokio::time::timeout(timeout, fut)
                    .await
                    .map_err(|_| anyhow::anyhow!("timeout"))?
                    .context("RPC call failed")?;
                Ok(serde_json::json!({"delivered": true}))
            }
            DeliveryRoute::Hub { hub_socket, target } => {
                let mut params = serde_json::json!({
                    "target": target,
                    "topic": topic,
                    "payload": payload,
                });
                if let Some(f) = from {
                    params["from"] = serde_json::json!(f);
                }
                let fut = client::rpc_call(hub_socket, "event.emit_to", params);
                let resp = tokio::time::timeout(timeout, fut)
                    .await
                    .map_err(|_| anyhow::anyhow!("timeout"))?
                    .context("Hub RPC call failed")?;
                let result = client::unwrap_result(resp)
                    .map_err(|e| anyhow::anyhow!("Hub rejected: {e}"))?;
                Ok(result)
            }
        }
    }

    fn via_label(&self) -> &'static str {
        match self {
            DeliveryRoute::Direct(_) => "direct",
            DeliveryRoute::Hub { .. } => "hub",
        }
    }
}

/// Resolve the effective chunk size (0 means use default) and compute
/// the number of chunks needed to transfer `file_size` bytes.
pub(crate) fn calculate_chunks(file_size: usize, chunk_size: usize) -> (usize, u32) {
    let effective = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total = file_size.div_ceil(effective) as u32;
    (effective, total)
}

pub(crate) async fn cmd_file_send(target: &str, path: &str, chunk_size: usize, json: bool, timeout_secs: u64) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    // T-989: Resolve delivery route — direct to session, or via hub (enables inbox)
    let route = match manager::find_session(target) {
        Ok(r) => DeliveryRoute::Direct(r.socket_path().to_path_buf()),
        Err(_) => {
            // Session not found locally — try hub fallback
            let hub_socket = termlink_hub::server::hub_socket_path();
            if hub_socket.exists() {
                if !json {
                    eprintln!("  Session '{}' not found locally, routing via hub", target);
                }
                DeliveryRoute::Hub {
                    hub_socket,
                    target: target.to_string(),
                }
            } else {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found and no hub available", target)}));
                }
                anyhow::bail!("Session '{}' not found and no hub available for inbox", target);
            }
        }
    };

    let file_path = std::path::Path::new(path);
    let file_data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to read file '{}': {}", path, e)}));
            }
            anyhow::bail!("Failed to read file '{}': {}", path, e);
        }
    };

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let size = file_data.len() as u64;
    let (chunk_sz, total_chunks) = calculate_chunks(file_data.len(), chunk_size);

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

    let init_payload = match serde_json::to_value(&init) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to serialize file.init: {}", e)}));
            }
            return Err(e).context("Failed to serialize file.init");
        }
    };
    let from_label = format!("cli-{}", std::process::id());
    match route.emit(file_topic::INIT, init_payload, Some(&from_label), timeout_dur).await {
        Ok(_) => {}
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to emit file.init: {}", e)}));
            }
            return Err(e).context("Failed to emit file.init");
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
        let chunk_payload = match serde_json::to_value(&chunk) {
            Ok(v) => v,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to serialize chunk {}: {}", i, e)}));
                }
                return Err(e).context(format!("Failed to serialize chunk {}", i));
            }
        };
        match route.emit(file_topic::CHUNK, chunk_payload, Some(&from_label), timeout_dur).await {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("Chunk {}/{} failed: {}", i + 1, total_chunks, e);
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": &msg}));
                }
                anyhow::bail!("{}", msg);
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
    let complete_payload = match serde_json::to_value(&complete) {
        Ok(v) => v,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to serialize file.complete: {}", e)}));
            }
            return Err(e).context("Failed to serialize file.complete");
        }
    };
    match route.emit(file_topic::COMPLETE, complete_payload, Some(&from_label), timeout_dur).await {
        Ok(_) => {}
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to emit file.complete: {}", e)}));
            }
            return Err(e).context("Failed to emit file.complete");
        }
    }

    let via = route.via_label();
    if json {
        println!("{}", serde_json::json!({
            "ok": true,
            "filename": filename,
            "size": size,
            "via": via,
            "chunks": total_chunks,
            "transfer_id": transfer_id,
            "sha256": sha256,
            "target": target,
        }));
    } else {
        let route_info = match via {
            "hub" => " (via hub — may be spooled for later delivery)",
            _ => "",
        };
        eprintln!("Transfer complete{route_info}. SHA-256: {sha256}");
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

    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let out_path = std::path::Path::new(output_dir);
    if !out_path.exists()
        && let Err(e) = std::fs::create_dir_all(out_path)
    {
        if json {
            super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to create output directory '{}': {}", output_dir, e)}));
        }
        return Err(e).context(format!("Failed to create output directory: {}", output_dir));
    }

    if !json {
        eprintln!("Waiting for file transfer on '{}' (timeout: {}s)...", target, timeout);
    }

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let subscribe_timeout = interval.max(500); // at least 500ms per subscribe call

    let mut poll_cursor: Option<u64> = None;
    let mut is_first_poll = true;

    let mut transfer_id: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut expected_chunks: u32 = 0;
    let mut chunks: std::collections::BTreeMap<u32, Vec<u8>> = std::collections::BTreeMap::new();

    let decoder = base64::engine::general_purpose::STANDARD;

    loop {
        // First poll uses event.poll (returns all historical events including seq 0).
        // Subsequent iterations use event.subscribe (server-side blocking, no sleep needed).
        let rpc_result = if is_first_poll {
            let params = serde_json::json!({});
            let rpc_timeout = std::time::Duration::from_secs(10);
            tokio::time::timeout(rpc_timeout, client::rpc_call(reg.socket_path(), "event.poll", params)).await
        } else {
            let remaining = timeout_dur.saturating_sub(start.elapsed());
            let effective_timeout = subscribe_timeout.min(remaining.as_millis() as u64);
            let mut params = serde_json::json!({"timeout_ms": effective_timeout});
            if let Some(c) = poll_cursor {
                params["since"] = serde_json::json!(c);
            }
            let rpc_timeout = std::time::Duration::from_secs(effective_timeout / 1000 + 5);
            tokio::time::timeout(rpc_timeout, client::rpc_call(reg.socket_path(), "event.subscribe", params)).await
        };
        match rpc_result {
            Err(_) => {
                tracing::warn!("RPC timed out, retrying...");
                continue;
            }
            Ok(Err(e)) => {
                tracing::warn!("RPC error: {}", e);
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
                                                let decoded = match decoder.decode(&chunk.data) {
                                                    Ok(d) => d,
                                                    Err(e) => {
                                                        if json {
                                                            super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Invalid base64 in chunk {}: {}", chunk.index, e)}));
                                                        }
                                                        return Err(e).context(format!("Invalid base64 in chunk {}", chunk.index));
                                                    }
                                                };
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
                                                                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg}));
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
                                                        super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg}));
                                                    }
                                                    anyhow::bail!("{}", msg);
                                                }
                                                let fname = filename.as_deref().unwrap_or("received-file");
                                                let dest = out_path.join(fname);
                                                if let Err(e) = std::fs::write(&dest, &file_data) {
                                                    if json {
                                                        super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to write file '{}': {}", dest.display(), e)}));
                                                    }
                                                    return Err(e).context(format!("Failed to write file: {}", dest.display()));
                                                }
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
                                            let decoded = match decoder.decode(&chunk.data) {
                                                Ok(d) => d,
                                                Err(e) => {
                                                    if json {
                                                        super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Invalid base64 in chunk {}: {}", chunk.index, e)}));
                                                    }
                                                    return Err(e).context(format!("Invalid base64 in chunk {}", chunk.index));
                                                }
                                            };
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
                                                            super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg}));
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
                                                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg}));
                                                }
                                                anyhow::bail!("{}", msg);
                                            }

                                            let fname = filename.as_deref().unwrap_or("received-file");
                                            let dest = out_path.join(fname);
                                            if let Err(e) = std::fs::write(&dest, &file_data) {
                                                if json {
                                                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to write file '{}': {}", dest.display(), e)}));
                                                }
                                                return Err(e).context(format!("Failed to write file: {}", dest.display()));
                                            }

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
                                                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": err_msg}));
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
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg, "chunks_received": chunks.len(), "chunks_expected": expected_chunks}));
                }
                anyhow::bail!("{}", msg);
            } else {
                let msg = format!("Timeout waiting for file transfer ({}s)", timeout);
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg}));
                }
                anyhow::bail!("{}", msg);
            }
        }

        // event.subscribe blocks server-side; no sleep needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_exact_multiple() {
        let (eff, total) = calculate_chunks(100, 10);
        assert_eq!(eff, 10);
        assert_eq!(total, 10);
    }

    #[test]
    fn chunks_with_remainder() {
        let (eff, total) = calculate_chunks(95, 10);
        assert_eq!(eff, 10);
        assert_eq!(total, 10); // 9 full + 1 partial
    }

    #[test]
    fn chunks_single_chunk() {
        let (eff, total) = calculate_chunks(5, 10);
        assert_eq!(eff, 10);
        assert_eq!(total, 1);
    }

    #[test]
    fn chunks_empty_file() {
        let (eff, total) = calculate_chunks(0, 10);
        assert_eq!(eff, 10);
        assert_eq!(total, 0);
    }

    #[test]
    fn chunks_zero_chunk_size_uses_default() {
        let (eff, total) = calculate_chunks(100_000, 0);
        assert_eq!(eff, DEFAULT_CHUNK_SIZE);
        assert_eq!(total, 100_000usize.div_ceil(DEFAULT_CHUNK_SIZE) as u32);
    }

    #[test]
    fn chunks_exact_one_byte() {
        let (_, total) = calculate_chunks(1, 1);
        assert_eq!(total, 1);
    }

    #[test]
    fn chunks_large_file() {
        // 10 MB file with 48 KB chunks
        let (eff, total) = calculate_chunks(10 * 1024 * 1024, 49152);
        assert_eq!(eff, 49152);
        assert_eq!(total, (10 * 1024 * 1024usize).div_ceil(49152) as u32);
    }
}
