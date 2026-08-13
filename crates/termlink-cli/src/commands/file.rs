use std::path::PathBuf;

use anyhow::{Context, Result};

use termlink_session::artifact::{
    download_artifact_via_client, recv_artifacts_via_client, send_artifact_via_client,
    ArtifactManifest, RecvOutcome, SendOutcome, SendPath,
};
use termlink_session::client;
use termlink_session::hub_capabilities::HubCapabilitiesCache;
use termlink_session::inbox_channel::FallbackCtx;
use termlink_session::manager;
use termlink_protocol::TransportAddr;

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
                let resp = tokio::time::timeout(timeout, fut)
                    .await
                    .map_err(|_| anyhow::anyhow!("{}", timeout_message("event.emit", timeout, &self.target_label())))?
                    .context("RPC call failed")?;
                let result = client::unwrap_result(resp)
                    .map_err(|e| anyhow::anyhow!("Session rejected: {e}"))?;
                // Merge delivered flag into response
                let mut out = result;
                if let Some(obj) = out.as_object_mut() {
                    obj.entry("delivered").or_insert(serde_json::json!(true));
                }
                Ok(out)
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
                    .map_err(|_| anyhow::anyhow!("{}", timeout_message("event.emit_to", timeout, &self.target_label())))?
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

    /// Human-readable delivery target for error messages: the session socket
    /// path (direct route) or the hub-relayed target id (hub route). Used to
    /// name *where* a timed-out transfer was headed (T-2626).
    fn target_label(&self) -> String {
        match self {
            DeliveryRoute::Direct(socket) => socket.display().to_string(),
            DeliveryRoute::Hub { target, .. } => target.clone(),
        }
    }
}

/// Build an actionable timeout message for a file-transfer event emit.
///
/// T-2626 (Directive #3 — actionable errors): a bare `"timeout"` dead-ends the
/// operator — they cannot tell whether the session is down, the hub is
/// unreachable, or the transfer stalled. This names the RPC operation, the
/// configured timeout, the delivery target, and a concrete next step. Pure and
/// unit-testable (no live hub). Mirrors the T-2554 / T-2625 convention.
fn timeout_message(operation: &str, timeout: std::time::Duration, target: &str) -> String {
    format!(
        "{operation} timed out after {timeout:?} (target: {target}) — \
         is the session/hub reachable? check `/peers`, then retry"
    )
}

/// Resolve the effective chunk size (0 means use default) and compute
/// the number of chunks needed to transfer `file_size` bytes.
pub(crate) fn calculate_chunks(file_size: usize, chunk_size: usize) -> (usize, u32) {
    let effective = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total = file_size.div_ceil(effective) as u32;
    (effective, total)
}

/// T-1249: Try to send via the new `channel.post` + `artifact.put` path.
///
/// Returns:
/// - `Ok(Some((offset, path)))` -- sent successfully on the new path.
/// - `Ok(None)` -- peer hub advertises neither artifact.put nor channel.post
///   in a way the helper can use; caller should fall back to the legacy
///   3-phase event-emit path.
/// - `Err(_)` -- transport/connect/auth error talking to the local hub
///   socket; caller may also fall back.
async fn try_send_via_artifact(
    hub_socket: &std::path::Path,
    target: &str,
    payload: &[u8],
    filename: &str,
    size: u64,
    expected_sha256: &str,
) -> Result<Option<(i64, SendPath)>> {
    let identity = super::channel::load_identity_or_create()?;
    let addr = TransportAddr::unix(hub_socket);
    let mut client = client::Client::connect_addr(&addr)
        .await
        .with_context(|| format!("connect hub at {}", hub_socket.display()))?;
    let host_port = format!("local:{}", hub_socket.display());
    let cache = HubCapabilitiesCache::new();
    let mut ctx = FallbackCtx::new();
    let manifest = ArtifactManifest {
        filename: filename.to_string(),
        size,
        from: format!("cli-{}", std::process::id()),
        transfer_id: None,
        content_type: None,
    };
    let outcome = send_artifact_via_client(
        &mut client,
        &host_port,
        target,
        payload,
        &manifest,
        &identity,
        &cache,
        &mut ctx,
    )
    .await
    .with_context(|| "send_artifact_via_client failed")?;
    match outcome {
        SendOutcome::LegacyOnly => Ok(None),
        SendOutcome::Sent {
            sha256,
            channel_offset,
            path,
            ..
        } => {
            // Defensive sanity: helper hashes itself; the caller-computed sha256
            // and the helper-returned sha256 must agree. Mismatch would indicate
            // a sha2 implementation bug -- surface loudly.
            if sha256 != expected_sha256 {
                anyhow::bail!(
                    "artifact sha256 mismatch (helper={sha256}, caller={expected_sha256})"
                );
            }
            Ok(Some((channel_offset, path)))
        }
    }
}

pub(crate) async fn cmd_file_send(target: &str, path: &str, chunk_size: usize, json: bool, timeout_secs: u64) -> Result<()> {
    super::print_deprecation_warning("file send", "channel post --file");
    use base64::Engine;
    use sha2::{Digest, Sha256};

    // T-989: Resolve delivery route -- direct to session, or via hub (enables inbox)
    let route = match manager::find_session(target) {
        Ok(r) => DeliveryRoute::Direct(r.socket_path().to_path_buf()),
        Err(_) => {
            // Session not found locally -- try hub fallback
            let (_, hub_socket) = super::infrastructure::resolve_hub_paths();
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

    // T-1249: Prefer the new channel.post + artifact.put path if a hub is
    // up and advertises both methods. On `LegacyOnly` (older hub) or no hub
    // socket at all, fall through to the legacy 3-phase event-emit below.
    let (_, hub_socket_for_artifact) = super::infrastructure::resolve_hub_paths();
    if hub_socket_for_artifact.exists() {
        match try_send_via_artifact(
            &hub_socket_for_artifact,
            target,
            &file_data,
            &filename,
            size,
            &sha256,
        )
        .await
        {
            Ok(Some((channel_offset, used_path))) => {
                let path_label = match used_path {
                    SendPath::Inline => "channel.inline",
                    SendPath::Chunked => "channel.artifact",
                };
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "filename": filename,
                            "size": size,
                            "via": path_label,
                            "spooled": false,
                            "chunks": total_chunks,
                            "transfer_id": transfer_id,
                            "sha256": sha256,
                            "target": target,
                            "channel_offset": channel_offset,
                            "artifact_sha256": sha256,
                        })
                    );
                } else {
                    eprintln!(
                        "Sent '{}' ({} bytes) via {} → channel.offset={}, sha256={}",
                        filename, size, path_label, channel_offset, sha256
                    );
                }
                return Ok(());
            }
            Ok(None) => {
                tracing::debug!(
                    target = %target,
                    "T-1249: hub doesn't advertise artifact.put -- falling back to legacy events"
                );
            }
            Err(e) => {
                tracing::warn!(
                    target = %target,
                    error = %e,
                    "T-1249: new-path send failed -- falling back to legacy events"
                );
            }
        }
    }

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
    let spooled = match route.emit(file_topic::INIT, init_payload, Some(&from_label), timeout_dur).await {
        Ok(resp) => resp.get("spooled").and_then(|s| s.as_bool()).unwrap_or(false),
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to emit file.init: {}", e)}));
            }
            return Err(e).context("Failed to emit file.init");
        }
    };

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
            "spooled": spooled,
            "chunks": total_chunks,
            "transfer_id": transfer_id,
            "sha256": sha256,
            "target": target,
        }));
    } else if spooled {
        eprintln!("File spooled to hub inbox for '{}'. SHA-256: {}", target, sha256);
        eprintln!("  Target must run 'termlink file receive {}' to assemble the file.", target);
    } else {
        eprintln!("Transfer complete (via {via}). SHA-256: {sha256}");
    }
    Ok(())
}

struct RecvSummary {
    filename: String,
    path: String,
    size: u64,
    sha256: String,
    via: &'static str,
}

/// T-1250: Try to receive via the new `channel.subscribe` + `artifact.get` path.
///
/// Returns:
/// - `Ok(Some(summary))` -- artifact received successfully, written to disk.
/// - `Ok(None)` -- peer hub doesn't advertise `channel.subscribe`; caller
///   should fall back to the legacy event-stream reassembly path.
/// - `Err(_)` -- transport, protocol, sha-mismatch, or timeout-with-cap-present.
///
/// Note: when the hub advertises `channel.subscribe`, this consumes the full
/// `timeout`. A legacy-only sender concurrent with a new-capable hub will
/// only be picked up after fallthrough -- worst-case 2× wait during migration.
async fn try_recv_via_artifact(
    hub_socket: &std::path::Path,
    target: &str,
    out_path: &std::path::Path,
    timeout_secs: u64,
    interval_ms: u64,
    replay: bool,
) -> Result<Option<RecvSummary>> {
    let addr = TransportAddr::unix(hub_socket);
    let mut client = client::Client::connect_addr(&addr)
        .await
        .with_context(|| format!("connect hub at {}", hub_socket.display()))?;
    let host_port = format!("local:{}", hub_socket.display());
    let cache = HubCapabilitiesCache::new();
    let mut ctx = FallbackCtx::new();

    // Establish starting cursor: replay → 0; fresh-only → take initial next_cursor.
    let initial = recv_artifacts_via_client(&mut client, &host_port, target, 0, &cache, &mut ctx)
        .await
        .with_context(|| "recv_artifacts_via_client (initial)")?;
    let mut cursor = match initial {
        RecvOutcome::LegacyOnly => return Ok(None),
        RecvOutcome::Received { artifacts, next_cursor } => {
            if replay
                && let Some(s) = process_artifact_batch(&mut client, &artifacts, out_path).await?
            {
                return Ok(Some(s));
            }
            next_cursor
        }
    };

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    loop {
        let outcome = recv_artifacts_via_client(
            &mut client, &host_port, target, cursor, &cache, &mut ctx,
        )
        .await
        .with_context(|| "recv_artifacts_via_client (poll)")?;
        match outcome {
            RecvOutcome::LegacyOnly => return Ok(None),
            RecvOutcome::Received { artifacts, next_cursor } => {
                cursor = next_cursor;
                if let Some(s) = process_artifact_batch(&mut client, &artifacts, out_path).await? {
                    return Ok(Some(s));
                }
            }
        }
        if start.elapsed() > timeout_dur {
            anyhow::bail!(
                "Timeout waiting for artifact via channel.subscribe ({}s)",
                timeout_secs
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
    }
}

/// T-2473 (OBS-108 bug #1): pick which artifact in a batch to serve. Returns the
/// NEWEST (highest `channel_offset`) artifact, not the oldest.
///
/// The old code took `artifacts.first()` — which, because the replay path
/// subscribes at cursor 0 (oldest-first), pinned `--replay` permanently to the
/// EARLIEST historical transfer and made every newer one unreachable. Selecting
/// by max offset makes `--replay` serve the most-recent transfer instead. This is
/// the minimal fix for a retirement-track verb (T-1166); for full addressable
/// history use `channel subscribe`.
fn select_newest_artifact(
    artifacts: &[termlink_session::artifact::RecvArtifact],
) -> Option<&termlink_session::artifact::RecvArtifact> {
    artifacts.iter().max_by_key(|a| a.channel_offset)
}

/// T-2520: reduce a wire-supplied filename to a trusted final basename before it is
/// joined onto the receive output dir. `FileInit.filename` / `ArtifactManifest.filename`
/// are documented "basename only, no path" (protocol events.rs:499) but arrive verbatim
/// from an untrusted peer. Without this, an absolute or `..`-bearing name escapes
/// `out_path` via `Path::join` (arbitrary file write outside `-o <dir>`). `Path::file_name`
/// returns the final component only and yields `None` for `""`, `.`, `..`, and path-only
/// inputs (e.g. `foo/..`, `a/b/`), so a rejected name never produces a parent/absolute path.
fn sanitize_recv_filename(wire_name: &str) -> Option<String> {
    let base = std::path::Path::new(wire_name).file_name()?;
    let s = base.to_string_lossy();
    if s.is_empty() || s == "." || s == ".." {
        return None;
    }
    Some(s.into_owned())
}

/// Process a batch of received artifact envelopes; on the NEWEST writable artifact
/// (T-2473), write it to disk and return the summary. Idempotency: if
/// `<out_path>/<filename>` already exists with matching sha256, skip the
/// download/write and return the pre-existing file's summary.
async fn process_artifact_batch(
    client: &mut client::Client,
    artifacts: &[termlink_session::artifact::RecvArtifact],
    out_path: &std::path::Path,
) -> Result<Option<RecvSummary>> {
    use sha2::{Digest, Sha256};
    if let Some(a) = select_newest_artifact(artifacts) {
        let (bytes, sha256_hex, filename, via) = if let Some(sha) = &a.artifact_ref {
            // Chunked path: manifest carries filename, bytes via artifact.get.
            // T-2520: sanitize wire filename to a basename; a missing/traversal/absolute
            // name falls back to the safe synthesized default, never escaping out_path.
            let manifest_filename = a
                .manifest
                .as_ref()
                .and_then(|m| sanitize_recv_filename(&m.filename))
                .unwrap_or_else(|| format!("received-{}.bin", &sha[..16.min(sha.len())]));
            let dest = out_path.join(&manifest_filename);
            // Idempotency check: if dest already has matching sha, skip download.
            if dest.exists()
                && let Ok(existing) = std::fs::read(&dest)
            {
                let mut h = Sha256::new();
                h.update(&existing);
                let existing_sha = format!("{:x}", h.finalize());
                if existing_sha == *sha {
                    return Ok(Some(RecvSummary {
                        filename: manifest_filename,
                        path: dest.display().to_string(),
                        size: existing.len() as u64,
                        sha256: sha.clone(),
                        via: "channel.artifact.skip-existing",
                    }));
                }
            }
            let bytes = download_artifact_via_client(client, sha)
                .await
                .with_context(|| format!("download_artifact_via_client {sha}"))?;
            (bytes, sha.clone(), manifest_filename, "channel.artifact")
        } else {
            // Inline path: payload IS the bytes; no manifest → synthesize a filename.
            let mut h = Sha256::new();
            h.update(&a.payload);
            let computed = format!("{:x}", h.finalize());
            let filename = format!("received-{}.bin", &computed[..16.min(computed.len())]);
            (a.payload.clone(), computed, filename, "channel.inline")
        };
        let dest = out_path.join(&filename);
        std::fs::write(&dest, &bytes)
            .with_context(|| format!("write {}", dest.display()))?;
        return Ok(Some(RecvSummary {
            filename,
            path: dest.display().to_string(),
            size: bytes.len() as u64,
            sha256: sha256_hex,
            via,
        }));
    }
    Ok(None)
}

/// T-2472 (OBS-108): reconcile a received file's ACTUAL sha256 against an
/// operator-supplied EXPECTED digest and return the honest human-readable label
/// for the sha line.
///
/// - `expected = Some(x)` and `x == actual` → `Ok("SHA-256 verified against expected: …")`.
/// - `expected = Some(x)` and `x != actual` → `Err(..)` -- a LOUD failure the caller must
///   surface with a non-zero exit. This is the receiver's "on mismatch, don't re-pin,
///   tell me" contract; it is the check that was structurally missing (the old code only
///   ever compared bytes against their OWN or the SENDER's digest, never the receiver's).
/// - `expected = None` → `Ok(label)` where the label states the digest's PROVENANCE
///   (computed-from-received-bytes / sender-manifest) and deliberately avoids the word
///   "verified", because no independent verification was performed.
///
/// `expected` is trimmed and lower-cased before comparison; `actual` is assumed to already
/// be lower-case hex (as produced by `format!("{:x}", Sha256::finalize())`).
fn reconcile_expected_sha256(actual: &str, expected: Option<&str>, via: &str) -> Result<String> {
    match expected {
        Some(exp) => {
            let exp = exp.trim().to_lowercase();
            if exp != actual {
                anyhow::bail!(
                    "SHA-256 MISMATCH: expected {exp}, received file hashes to {actual} \
                     (via {via}). Not re-pinning -- the served bytes are NOT the file you \
                     expected. (OBS-108: `file receive --replay` can re-serve an older \
                     transfer; confirm the sender exposed a pullable ref, or use \
                     `channel subscribe` for addressable history.)"
                );
            }
            Ok(format!("SHA-256 verified against expected: {actual}"))
        }
        None => {
            let provenance = match via {
                "channel.inline" => {
                    "computed from received bytes, NOT independently verified \
                     (pass --expected-sha256 to verify)"
                }
                "channel.artifact" | "channel.artifact.skip-existing" => {
                    "matches sender manifest, NOT verified against a receiver digest \
                     (pass --expected-sha256 to verify)"
                }
                _ => {
                    "matches sender-declared digest, NOT verified against a receiver digest \
                     (pass --expected-sha256 to verify)"
                }
            };
            Ok(format!("SHA-256 {provenance}: {actual}"))
        }
    }
}

pub(crate) async fn cmd_file_receive(
    target: &str,
    output_dir: &str,
    timeout: u64,
    interval: u64,
    replay: bool,
    expected_sha256: Option<&str>,
    json: bool,
) -> Result<()> {
    super::print_deprecation_warning("file receive", "channel subscribe");
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

    // T-1250: Try the new channel.subscribe + artifact.get path first when a
    // local hub is up. On `LegacyOnly` (no channel.subscribe), fall through to
    // the existing event-stream reassembly. On transport/timeout, also fall
    // through (best-effort migration coexistence).
    let (_, hub_socket_for_artifact) = super::infrastructure::resolve_hub_paths();
    if hub_socket_for_artifact.exists() {
        match try_recv_via_artifact(
            &hub_socket_for_artifact, target, out_path, timeout, interval, replay,
        )
        .await
        {
            Ok(Some(s)) => {
                // T-2472 (OBS-108): reconcile against operator-supplied expected digest
                // BEFORE reporting success. A mismatch is a loud, non-zero-exit failure --
                // never a clean "verified" + exit 0 on the wrong bytes.
                match reconcile_expected_sha256(&s.sha256, expected_sha256, s.via) {
                    Ok(label) => {
                        if json {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "ok": true,
                                    "filename": s.filename,
                                    "path": s.path,
                                    "size": s.size,
                                    "sha256": s.sha256,
                                    "sha256_expected": expected_sha256,
                                    "sha256_verified": expected_sha256.is_some(),
                                    "target": target,
                                    "via": s.via,
                                })
                            );
                        } else {
                            eprintln!(
                                "File saved: {} ({} bytes) via {}", s.path, s.size, s.via
                            );
                            eprintln!("{label}");
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        if json {
                            super::json_error_exit(serde_json::json!({
                                "ok": false,
                                "target": target,
                                "error": e.to_string(),
                                "sha256_received": s.sha256,
                                "sha256_expected": expected_sha256,
                                "path": s.path,
                            }));
                        }
                        return Err(e);
                    }
                }
            }
            Ok(None) => {
                tracing::debug!(
                    target = %target,
                    "T-1250: hub doesn't advertise channel.subscribe -- falling back to legacy events"
                );
            }
            Err(e) => {
                tracing::warn!(
                    target = %target,
                    error = %e,
                    "T-1250: new-path receive failed -- falling back to legacy events"
                );
            }
        }
    }

    if !json {
        if replay {
            eprintln!("Waiting for file transfer on '{}' (replay mode, timeout: {}s)...", target, timeout);
        } else {
            eprintln!("Waiting for file transfer on '{}' (timeout: {}s)...", target, timeout);
        }
    }

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let subscribe_timeout = interval.max(500); // at least 500ms per subscribe call

    let mut poll_cursor: Option<u64> = None;
    let mut is_first_poll = replay; // Only do first-poll (historical scan) in replay mode

    // T-1018: In fresh-only mode (default), get current cursor to skip stale events
    if !replay {
        let params = serde_json::json!({});
        let rpc_timeout = std::time::Duration::from_secs(10);
        if let Ok(Ok(resp)) = tokio::time::timeout(
            rpc_timeout,
            client::rpc_call(reg.socket_path(), "event.poll", params),
        ).await
            && let Ok(result) = client::unwrap_result(resp)
            && let Some(next) = result["next_seq"].as_u64()
        {
            poll_cursor = Some(next);
            if !json {
                eprintln!("  Skipping {} historical event(s), waiting for fresh transfers...",
                    result["events"].as_array().map(|a| a.len()).unwrap_or(0));
            }
        }
    }

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
                // T-2673: back off on the INSTANT-error path (busy-spin class,
                // T-2670/T-2671). The Err(_) arm above waited the full rpc_timeout before
                // continue (naturally paced), but Ok(Err(e)) is the instant-error path —
                // connection-refused on a dead/half-open hub returns near-instantly, so a
                // bare fall-through re-dispatches with zero delay and pins a CPU core
                // (silently — warn gated at default level). 500ms sleep-on-error is the
                // established convention (events.rs:805/900/1349).
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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
                                                // T-2472 (OBS-108): independent verification against a receiver-supplied digest.
                                                // A mismatch here is a loud, non-zero-exit failure -- never a clean "verified".
                                                if let Some(exp) = expected_sha256 {
                                                    let exp = exp.trim().to_lowercase();
                                                    if exp != actual_sha256 {
                                                        let msg = format!("SHA-256 MISMATCH: expected {exp}, received file hashes to {actual_sha256}. Not re-pinning -- served bytes are NOT the expected file (OBS-108).");
                                                        if json { super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg, "sha256_received": actual_sha256, "sha256_expected": exp})); }
                                                        anyhow::bail!("{}", msg);
                                                    }
                                                }
                                                // T-2520: confine wire filename to a basename before join.
                                                let fname = filename
                                                    .as_deref()
                                                    .and_then(sanitize_recv_filename)
                                                    .unwrap_or_else(|| "received-file".to_string());
                                                let dest = out_path.join(&fname);
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
                                                    if expected_sha256.is_some() {
                                                        eprintln!("SHA-256 verified against expected: {actual_sha256}");
                                                    } else {
                                                        eprintln!("SHA-256 matches sender-declared digest, NOT verified against a receiver digest (pass --expected-sha256 to verify): {actual_sha256}");
                                                    }
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

                                            // T-2472 (OBS-108): independent verification against a receiver-supplied digest.
                                            // A mismatch here is a loud, non-zero-exit failure -- never a clean "verified".
                                            if let Some(exp) = expected_sha256 {
                                                let exp = exp.trim().to_lowercase();
                                                if exp != actual_sha256 {
                                                    let msg = format!("SHA-256 MISMATCH: expected {exp}, received file hashes to {actual_sha256}. Not re-pinning -- served bytes are NOT the expected file (OBS-108).");
                                                    if json { super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": msg, "sha256_received": actual_sha256, "sha256_expected": exp})); }
                                                    anyhow::bail!("{}", msg);
                                                }
                                            }
                                            // T-2520: confine wire filename to a basename before join.
                                            let fname = filename
                                                .as_deref()
                                                .and_then(sanitize_recv_filename)
                                                .unwrap_or_else(|| "received-file".to_string());
                                            let dest = out_path.join(&fname);
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
                                                if expected_sha256.is_some() {
                                                    eprintln!("SHA-256 verified against expected: {actual_sha256}");
                                                } else {
                                                    eprintln!("SHA-256 matches sender-declared digest, NOT verified against a receiver digest (pass --expected-sha256 to verify): {actual_sha256}");
                                                }
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

    // T-2626: the file-transfer timeout message must be actionable — name the
    // operation, the configured timeout, the target, and a next step. A bare
    // "timeout" (the pre-fix behavior) fails these assertions (load-bearing).
    #[test]
    fn timeout_message_names_operation_duration_target_and_next_step() {
        let msg = timeout_message(
            "event.emit_to",
            std::time::Duration::from_secs(30),
            "worker-a",
        );
        assert!(msg.contains("event.emit_to"), "must name the operation: {msg}");
        assert!(msg.contains("30s"), "must name the configured timeout: {msg}");
        assert!(msg.contains("worker-a"), "must name the target: {msg}");
        assert!(
            msg.contains("/peers") || msg.to_lowercase().contains("reachable"),
            "must suggest a concrete next step: {msg}"
        );
    }

    // T-2520: wire filenames must confine to a basename — a traversal/absolute name
    // must never yield a parent/absolute path that out_path.join would follow.
    #[test]
    fn sanitize_recv_filename_confines_to_basename() {
        use std::path::Path;
        let out = Path::new("/out/dir");

        // Traversal: reduces to the final component; join stays under out_path.
        let s = sanitize_recv_filename("../../etc/cron.d/x").expect("basename recovered");
        assert_eq!(s, "x");
        assert_eq!(out.join(&s), Path::new("/out/dir/x"));

        // Absolute path: reduces to basename, does NOT replace the base.
        let s = sanitize_recv_filename("/etc/passwd").expect("basename recovered");
        assert_eq!(s, "passwd");
        assert_eq!(out.join(&s), Path::new("/out/dir/passwd"));

        // Nested relative: only the leaf survives.
        let s = sanitize_recv_filename("a/b/c.txt").expect("basename recovered");
        assert_eq!(s, "c.txt");
        assert_eq!(out.join(&s), Path::new("/out/dir/c.txt"));

        // Path-only / degenerate inputs yield None (caller falls back to a safe default).
        assert_eq!(sanitize_recv_filename(""), None);
        assert_eq!(sanitize_recv_filename("."), None);
        assert_eq!(sanitize_recv_filename(".."), None);
        assert_eq!(sanitize_recv_filename("foo/.."), None);
        // Trailing slash is stripped by file_name(); leaf survives, still confined.
        assert_eq!(sanitize_recv_filename("a/b/").as_deref(), Some("b"));

        // Ordinary name passes through unchanged.
        assert_eq!(sanitize_recv_filename("report.pdf").as_deref(), Some("report.pdf"));

        // Invariant: no accepted name ever escapes out_path.
        for probe in ["../../../root/.ssh/authorized_keys", "/tmp/evil", "x/../../../y", "normal.bin"] {
            if let Some(base) = sanitize_recv_filename(probe) {
                let joined = out.join(&base);
                assert!(joined.starts_with(out), "escaped out_path: {probe} -> {joined:?}");
                assert!(!base.contains('/'), "basename retained a separator: {base}");
            }
        }
    }

    // T-2472 (OBS-108): the digest-reconcile contract that kills the false-green.
    const A: &str = "093858aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn reconcile_mismatch_is_loud_error() {
        // Expected digest supplied but the served file hashes differently → Err
        // (the caller turns this into a non-zero exit). This is the whole point.
        let r = reconcile_expected_sha256(A, Some("efb53839deadbeef"), "channel.inline");
        assert!(r.is_err(), "mismatch must be a hard error, not a clean pass");
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("MISMATCH"), "error must name the mismatch: {msg}");
    }

    #[test]
    fn reconcile_match_is_verified() {
        // Expected digest supplied and equal (case/whitespace-insensitive) → verified.
        let r = reconcile_expected_sha256(A, Some(&format!("  {}  ", A.to_uppercase())), "channel.artifact")
            .expect("matching digest must succeed");
        assert!(r.contains("verified against expected"), "label must say verified: {r}");
    }

    #[test]
    fn reconcile_absent_is_not_verified() {
        // No expected digest → NO independent verification happened, so the label
        // must never claim a bare "verified"; it states provenance instead.
        let inline = reconcile_expected_sha256(A, None, "channel.inline").unwrap();
        assert!(inline.contains("computed from received bytes"), "inline label: {inline}");
        assert!(!inline.contains("verified against expected"), "must not overclaim: {inline}");
        let chunked = reconcile_expected_sha256(A, None, "channel.artifact").unwrap();
        assert!(chunked.contains("sender manifest"), "chunked label: {chunked}");
    }

    // T-2473 (OBS-108 bug #1): replay must serve the NEWEST transfer, not the oldest.
    fn mk_artifact(offset: u64) -> termlink_session::artifact::RecvArtifact {
        termlink_session::artifact::RecvArtifact {
            channel_offset: offset,
            artifact_ref: None,
            manifest: None,
            payload: vec![offset as u8],
            sender_id: "s".to_string(),
            ts_unix_ms: 0,
        }
    }

    #[test]
    fn select_newest_artifact_picks_highest_offset() {
        // Batch arrives oldest-first (as channel.subscribe cursor 0 returns it).
        let batch = vec![mk_artifact(3), mk_artifact(41), mk_artifact(17)];
        let picked = select_newest_artifact(&batch).expect("non-empty batch");
        assert_eq!(picked.channel_offset, 41, "must serve newest, not first/oldest");
        // Regression guard: the OLD code did artifacts.first() → offset 3.
        assert_ne!(picked.channel_offset, batch.first().unwrap().channel_offset);
    }

    #[test]
    fn select_newest_artifact_empty_is_none() {
        assert!(select_newest_artifact(&[]).is_none());
    }

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
