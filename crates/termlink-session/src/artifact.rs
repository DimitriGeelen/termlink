//! Sender helper for the T-1164b artifact migration.
//!
//! Provides `send_artifact_via_client` — a single async entry point that
//! callers (CLI cmd_file_send, CLI remote, MCP termlink_file_send) use to
//! ship a payload via the new `artifact.put` + `channel.post(msg_type=artifact)`
//! flow that landed in T-1248.
//!
//! Capability gate: probes `HubCapabilitiesCache` for `artifact.put`. On
//! unsupported peers, returns `SendOutcome::LegacyOnly` so the caller can
//! fall back to the existing 3-phase event-emit (`file.init` / `file.chunk`
//! / `file.complete`) path. Same warn-once pattern as the T-1235 inbox
//! migration; reuses `inbox_channel::FallbackCtx` so a single ctx tracks
//! both subsystems' legacy-only peers and warn-once dedup.
//!
//! Threshold semantics (T-1164 design):
//! - `payload.len() <= ARTIFACT_INLINE_THRESHOLD` (64KB): inline path.
//!   The manifest *is* the channel payload; no `artifact.put` upload, no
//!   `artifact_ref`. Saves a round-trip for chat-sized blobs.
//! - `payload.len() > ARTIFACT_INLINE_THRESHOLD`: chunked artifact.put,
//!   then `channel.post` with `artifact_ref = sha256` and the manifest as
//!   the channel payload.
//!
//! PL-011 closure: callers receive `{sha256, channel_offset}` from the
//! `Sent` outcome, so delivery is provable by reading the channel log at
//! the returned offset.

use std::io;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use termlink_protocol::control::{self, channel::canonical_sign_bytes};
use termlink_protocol::jsonrpc::RpcResponse;

use crate::agent_identity::Identity;
use crate::client::Client;
use crate::hub_capabilities::HubCapabilitiesCache;
use crate::inbox_channel::FallbackCtx;

/// Payloads at or below this size go inline in `channel.post payload_b64`,
/// skipping the separate `artifact.put` upload entirely.
pub const ARTIFACT_INLINE_THRESHOLD: usize = 64 * 1024;

/// Default chunk size for streaming `artifact.put`. 256KB matches the
/// hub-side `DEFAULT_MAX_CHUNK_BYTES` ceiling and stays well under
/// `MAX_PAYLOAD_SIZE` even after base64 expansion.
pub const ARTIFACT_PUT_CHUNK_SIZE: usize = 256 * 1024;

/// `msg_type` written into `channel.post` for migrated artifacts.
pub const MSG_TYPE_ARTIFACT: &str = "artifact";

const RPC_METHOD_NOT_FOUND: i64 = -32601;

/// Caller-supplied metadata about the payload. Embedded as JSON in the
/// channel envelope's `payload` field — receivers parse it to reconstruct
/// filename / size / sender info without a separate roundtrip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactManifest {
    pub filename: String,
    pub size: u64,
    pub from: String,
    /// Original transfer id (correlation key). Optional; defaults to a
    /// derivative of the sha256.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_id: Option<String>,
    /// Optional content type (MIME). Receivers may use it to set file
    /// extensions; the helper itself does not interpret it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Outcome of `send_artifact_via_client`.
#[derive(Debug, Clone)]
pub enum SendOutcome {
    /// Hub does not advertise `artifact.put` (or `channel.post`). Caller must
    /// fall back to the legacy 3-phase event-emit path.
    LegacyOnly,
    /// Sent successfully. `channel_offset` is the per-topic log offset the
    /// receiver can read at to consume the envelope. PL-011 closure.
    Sent {
        sha256: String,
        channel_offset: i64,
        total_bytes: u64,
        path: SendPath,
    },
}

/// Whether the helper used the inline (small payload) or chunked (large
/// payload) path. Surfaced for callers that want to log/telemeter the choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendPath {
    Inline,
    Chunked,
}

/// One artifact envelope read from the inbox topic. `payload` is the raw
/// channel payload (manifest JSON for chunked, file bytes for inline).
/// `artifact_ref` is `Some(sha256)` for the chunked path; the receiver must
/// download the bytes via `download_artifact_via_client`. For the inline
/// path, `artifact_ref` is `None` and `payload` already holds the bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecvArtifact {
    pub channel_offset: u64,
    pub artifact_ref: Option<String>,
    pub manifest: Option<ArtifactManifest>,
    pub payload: Vec<u8>,
    pub sender_id: String,
    pub ts_unix_ms: i64,
}

/// Outcome of `recv_artifacts_via_client`.
#[derive(Debug, Clone)]
pub enum RecvOutcome {
    /// Hub does not advertise `channel.subscribe`. Caller should fall back
    /// to the legacy event-stream reassembly path.
    LegacyOnly,
    Received {
        artifacts: Vec<RecvArtifact>,
        next_cursor: u64,
    },
}

#[derive(Debug)]
enum ArtifactRpcError {
    MethodNotFound,
    Other(io::Error),
}

/// Probe + upload + signed-post. Caller must already hold an authenticated
/// `Client` (CLI remote, MCP). For a Unix-socket convenience wrapper, see
/// `send_artifact`.
///
/// `host_port` is used for the capability-cache key and warn-once dedup.
#[allow(clippy::too_many_arguments)]
pub async fn send_artifact_via_client(
    client: &mut Client,
    host_port: &str,
    target: &str,
    payload: &[u8],
    manifest: &ArtifactManifest,
    identity: &Identity,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<SendOutcome> {
    if ctx.is_legacy_only(host_port) {
        if ctx.warn_once(host_port, "artifact.legacy") {
            tracing::info!(
                host = %host_port,
                target = %target,
                "T-1249: peer flagged legacy-only — caller should use file.* event-emit fallback"
            );
        }
        return Ok(SendOutcome::LegacyOnly);
    }

    // Capability probe: need both artifact.put (for chunked path) and
    // channel.post (for either path).
    let methods: Vec<String> =
        crate::inbox_channel::probe_caps_via_client(client, host_port, cache)
            .await
            .unwrap_or_default();
    let has_channel_post = methods.iter().any(|m| m == control::method::CHANNEL_POST);
    let has_artifact_put = methods.iter().any(|m| m == control::method::ARTIFACT_PUT);

    if !has_channel_post {
        ctx.flag_legacy_only(host_port);
        if ctx.warn_once(host_port, "artifact.no_channel_post") {
            tracing::info!(
                host = %host_port,
                target = %target,
                "T-1249: peer lacks channel.post — falling back to legacy file.* events"
            );
        }
        return Ok(SendOutcome::LegacyOnly);
    }

    let sha256 = hex_sha256(payload);
    let total_bytes = payload.len() as u64;
    let inline = payload.len() <= ARTIFACT_INLINE_THRESHOLD;

    // Large-payload chunked upload requires artifact.put.
    if !inline && !has_artifact_put {
        if ctx.warn_once(host_port, "artifact.put.missing") {
            tracing::info!(
                host = %host_port,
                target = %target,
                size = total_bytes,
                "T-1249: peer lacks artifact.put for large payload — falling back to legacy file.* events"
            );
        }
        return Ok(SendOutcome::LegacyOnly);
    }

    let used_path = if inline {
        SendPath::Inline
    } else {
        // Chunked upload via artifact.put. On method-not-found mid-stream we
        // flag legacy and bail; on transport errors we propagate.
        match upload_artifact_chunked(client, payload, &sha256).await {
            Ok(()) => SendPath::Chunked,
            Err(ArtifactRpcError::MethodNotFound) => {
                ctx.flag_legacy_only(host_port);
                if ctx.warn_once(host_port, "artifact.put.methodnotfound") {
                    tracing::warn!(
                        host = %host_port,
                        target = %target,
                        "T-1249: artifact.put returned method-not-found mid-upload"
                    );
                }
                return Ok(SendOutcome::LegacyOnly);
            }
            Err(ArtifactRpcError::Other(e)) => return Err(e),
        }
    };

    // Build the channel envelope. Inline path uses payload as the channel
    // payload directly; chunked path uses a JSON manifest as the channel
    // payload, with artifact_ref carrying the sha256.
    let channel_payload: Vec<u8> = if inline {
        payload.to_vec()
    } else {
        serde_json::to_vec(manifest).map_err(|e| {
            io::Error::other(format!("serialize manifest: {e}"))
        })?
    };
    let artifact_ref: Option<String> = if inline { None } else { Some(sha256.clone()) };

    let topic = format!("inbox:{target}");
    let ts_unix_ms = now_ms();
    let signed = canonical_sign_bytes(
        &topic,
        MSG_TYPE_ARTIFACT,
        &channel_payload,
        artifact_ref.as_deref(),
        ts_unix_ms,
    );
    let sig = identity.sign(&signed);

    let post_params = json!({
        "topic": topic,
        "msg_type": MSG_TYPE_ARTIFACT,
        "payload_b64": B64.encode(&channel_payload),
        "artifact_ref": artifact_ref,
        "ts": ts_unix_ms,
        "sender_id": manifest.from.clone(),
        "sender_pubkey_hex": identity.public_key_hex().to_string(),
        "signature_hex": hex_of(&sig.to_bytes()),
    });

    let resp = client
        .call(
            control::method::CHANNEL_POST,
            json!("artifact-send"),
            post_params,
        )
        .await
        .map_err(|e| io::Error::other(format!("channel.post: {e}")))?;

    let offset = match resp {
        RpcResponse::Success(ok) => ok
            .result
            .get("offset")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| io::Error::other("channel.post reply missing offset"))?,
        RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
            ctx.flag_legacy_only(host_port);
            return Ok(SendOutcome::LegacyOnly);
        }
        RpcResponse::Error(e) => {
            return Err(io::Error::other(format!(
                "channel.post error {}: {}",
                e.error.code, e.error.message
            )));
        }
    };

    if ctx.warn_once(host_port, "artifact.sent") {
        tracing::info!(
            host = %host_port,
            target = %target,
            sha256 = %sha256,
            path = ?used_path,
            "T-1249: artifact sent via channel.post"
        );
    }
    Ok(SendOutcome::Sent {
        sha256,
        channel_offset: offset,
        total_bytes,
        path: used_path,
    })
}

async fn upload_artifact_chunked(
    client: &mut Client,
    payload: &[u8],
    expected_sha256: &str,
) -> Result<(), ArtifactRpcError> {
    // Generate a per-upload staging id so concurrent senders don't collide.
    let staging_id = format!("send-{}-{}", std::process::id(), now_ms());
    let mut iter = payload.chunks(ARTIFACT_PUT_CHUNK_SIZE).peekable();
    let mut offset: u64 = 0;
    while let Some(chunk) = iter.next() {
        let is_final = iter.peek().is_none();
        let mut params = json!({
            "staging_id": staging_id,
            "offset": offset,
            "chunk_b64": B64.encode(chunk),
            "is_final": is_final,
        });
        if is_final {
            params["expected_sha256"] = json!(expected_sha256);
        }
        let resp = client
            .call(
                control::method::ARTIFACT_PUT,
                json!(format!("artifact-put-{offset}")),
                params,
            )
            .await
            .map_err(|e| {
                ArtifactRpcError::Other(io::Error::other(format!("artifact.put: {e}")))
            })?;
        match resp {
            RpcResponse::Success(_) => {}
            RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
                return Err(ArtifactRpcError::MethodNotFound);
            }
            RpcResponse::Error(e) => {
                return Err(ArtifactRpcError::Other(io::Error::other(format!(
                    "artifact.put error {}: {}",
                    e.error.code, e.error.message
                ))));
            }
        }
        offset += chunk.len() as u64;
    }
    Ok(())
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest = h.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

fn hex_of(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Subscribe to `inbox:<target_self>` starting at `since_offset`, return all
/// envelopes with `msg_type == "artifact"` plus the new cursor. Reusable by
/// CLI cmd_file_receive and MCP termlink_file_receive (T-1250 / T-1164c).
pub async fn recv_artifacts_via_client(
    client: &mut Client,
    host_port: &str,
    target_self: &str,
    since_offset: u64,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<RecvOutcome> {
    if ctx.is_legacy_only(host_port) {
        return Ok(RecvOutcome::LegacyOnly);
    }
    let methods: Vec<String> =
        crate::inbox_channel::probe_caps_via_client(client, host_port, cache)
            .await
            .unwrap_or_default();
    let has_subscribe = methods
        .iter()
        .any(|m| m == control::method::CHANNEL_SUBSCRIBE);
    if !has_subscribe {
        ctx.flag_legacy_only(host_port);
        if ctx.warn_once(host_port, "artifact.recv.no_subscribe") {
            tracing::info!(
                host = %host_port,
                target = %target_self,
                "T-1250: peer lacks channel.subscribe — caller falls back to legacy receive"
            );
        }
        return Ok(RecvOutcome::LegacyOnly);
    }

    let topic = format!("inbox:{target_self}");
    let resp = client
        .call(
            control::method::CHANNEL_SUBSCRIBE,
            json!("artifact-recv"),
            json!({
                "topic": topic,
                "cursor": since_offset,
                "limit": 1000,
            }),
        )
        .await
        .map_err(|e| io::Error::other(format!("channel.subscribe: {e}")))?;
    let (messages, next_cursor) = match resp {
        RpcResponse::Success(ok) => {
            let msgs = ok
                .result
                .get("messages")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let next = ok
                .result
                .get("next_cursor")
                .and_then(|v| v.as_u64())
                .unwrap_or(since_offset);
            (msgs, next)
        }
        RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
            ctx.flag_legacy_only(host_port);
            return Ok(RecvOutcome::LegacyOnly);
        }
        RpcResponse::Error(e) => {
            return Err(io::Error::other(format!(
                "channel.subscribe error {}: {}",
                e.error.code, e.error.message
            )));
        }
    };

    let mut artifacts = Vec::new();
    for env in messages {
        // Channel envelopes: { offset, msg_type, payload_b64, artifact_ref?, ts, sender_id, ... }
        let msg_type = env.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        if msg_type != MSG_TYPE_ARTIFACT {
            continue;
        }
        let offset = env.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        let artifact_ref = env
            .get("artifact_ref")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let payload_b64 = env
            .get("payload_b64")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let payload = match B64.decode(payload_b64) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sender_id = env
            .get("sender_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ts_unix_ms = env.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
        let manifest: Option<ArtifactManifest> = if artifact_ref.is_some() {
            // Chunked: payload is the manifest JSON.
            serde_json::from_slice(&payload).ok()
        } else {
            None
        };
        artifacts.push(RecvArtifact {
            channel_offset: offset,
            artifact_ref,
            manifest,
            payload,
            sender_id,
            ts_unix_ms,
        });
    }

    Ok(RecvOutcome::Received {
        artifacts,
        next_cursor,
    })
}

/// Download an artifact's bytes by sha256 via chunked `artifact.get`. Verifies
/// the returned bytes hash to the requested key; returns an error on mismatch.
pub async fn download_artifact_via_client(
    client: &mut Client,
    sha256: &str,
) -> io::Result<Vec<u8>> {
    const CHUNK: usize = 256 * 1024;
    let mut out: Vec<u8> = Vec::new();
    let mut offset: u64 = 0;
    loop {
        let resp = client
            .call(
                control::method::ARTIFACT_GET,
                json!(format!("artifact-get-{offset}")),
                json!({
                    "sha256": sha256,
                    "offset": offset,
                    "max_bytes": CHUNK,
                }),
            )
            .await
            .map_err(|e| io::Error::other(format!("artifact.get: {e}")))?;
        let result = match resp {
            RpcResponse::Success(ok) => ok.result,
            RpcResponse::Error(e) => {
                return Err(io::Error::other(format!(
                    "artifact.get error {}: {}",
                    e.error.code, e.error.message
                )));
            }
        };
        let chunk_b64 = result
            .get("chunk_b64")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let bytes = B64
            .decode(chunk_b64)
            .map_err(|e| io::Error::other(format!("artifact.get chunk_b64 decode: {e}")))?;
        out.extend_from_slice(&bytes);
        offset += bytes.len() as u64;
        let eof = result.get("eof").and_then(|v| v.as_bool()).unwrap_or(false);
        if eof {
            break;
        }
        if bytes.is_empty() {
            return Err(io::Error::other("artifact.get returned 0 bytes without eof"));
        }
    }
    let got = hex_sha256(&out);
    if got != sha256 {
        return Err(io::Error::other(format!(
            "artifact.get sha256 mismatch: requested {sha256}, computed {got}"
        )));
    }
    Ok(out)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::sync::{Arc, Mutex};
    use termlink_protocol::TransportAddr;

    fn manifest(name: &str, size: u64) -> ArtifactManifest {
        ArtifactManifest {
            filename: name.into(),
            size,
            from: "test-sender".into(),
            transfer_id: None,
            content_type: None,
        }
    }

    /// In-test fake hub. Serves hub.capabilities (configurable methods),
    /// artifact.put (records params + ack), channel.post (records envelope +
    /// returns incrementing offset), channel.subscribe (returns staged
    /// envelopes), and artifact.get (returns staged blobs).
    struct FakeHub {
        listener: tokio::net::UnixListener,
        addr: TransportAddr,
        seen_posts: Arc<Mutex<Vec<Value>>>,
        seen_puts: Arc<Mutex<Vec<Value>>>,
        capabilities: Vec<String>,
        next_offset: Arc<Mutex<i64>>,
        /// Staged inbox messages to return on channel.subscribe.
        subscribe_messages: Arc<Mutex<Vec<Value>>>,
        /// Staged blobs keyed by sha256.
        blobs: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    }

    impl FakeHub {
        fn new_in(dir: &std::path::Path, capabilities: Vec<String>) -> Self {
            let sock = dir.join("fakehub.sock");
            let listener = tokio::net::UnixListener::bind(&sock).unwrap();
            Self {
                listener,
                addr: TransportAddr::unix(&sock),
                seen_posts: Arc::new(Mutex::new(Vec::new())),
                seen_puts: Arc::new(Mutex::new(Vec::new())),
                capabilities,
                next_offset: Arc::new(Mutex::new(1)),
                subscribe_messages: Arc::new(Mutex::new(Vec::new())),
                blobs: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }

        async fn run_one(&self) -> tokio::task::JoinHandle<()> {
            let posts = self.seen_posts.clone();
            let puts = self.seen_puts.clone();
            let caps = self.capabilities.clone();
            let next_offset = self.next_offset.clone();
            let subscribe_messages = self.subscribe_messages.clone();
            let blobs = self.blobs.clone();
            // Accept one connection, handle frames until peer drops.
            let listener = &self.listener;
            let (mut stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                use termlink_protocol::jsonrpc::{Request, Response};
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = Vec::new();
                loop {
                    let mut tmp = [0u8; 4096];
                    let n = match stream.read(&mut tmp).await {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    buf.extend_from_slice(&tmp[..n]);
                    while let Some(pos) = buf.iter().position(|b| *b == b'\n') {
                        let line: Vec<u8> = buf.drain(..=pos).collect();
                        let req: Request = match serde_json::from_slice(&line[..pos]) {
                            Ok(r) => r,
                            Err(_) => continue,
                        };
                        let result_val = match req.method.as_str() {
                            "hub.capabilities" => {
                                json!({"methods": caps, "hub_version": "test", "protocol_version": 1})
                            }
                            "artifact.put" => {
                                puts.lock().unwrap().push(req.params.clone());
                                let is_final = req
                                    .params
                                    .get("is_final")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                if is_final {
                                    let sha = req
                                        .params
                                        .get("expected_sha256")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    json!({"ok": true, "in_progress": false, "sha256": sha, "total_bytes": 0})
                                } else {
                                    json!({"ok": true, "in_progress": true, "bytes_received": 0})
                                }
                            }
                            "channel.post" => {
                                posts.lock().unwrap().push(req.params.clone());
                                let mut off = next_offset.lock().unwrap();
                                let cur = *off;
                                *off += 1;
                                json!({"ok": true, "offset": cur, "ts": 0})
                            }
                            "channel.subscribe" => {
                                let msgs = subscribe_messages.lock().unwrap().clone();
                                let next = msgs.len() as u64;
                                json!({"messages": msgs, "next_cursor": next})
                            }
                            "artifact.get" => {
                                let sha = req
                                    .params
                                    .get("sha256")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let offset = req
                                    .params
                                    .get("offset")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0)
                                    as usize;
                                let max_bytes = req
                                    .params
                                    .get("max_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(256 * 1024)
                                    as usize;
                                let bytes_opt = blobs.lock().unwrap().get(&sha).cloned();
                                match bytes_opt {
                                    Some(bytes) => {
                                        let total = bytes.len();
                                        let end = (offset + max_bytes).min(total);
                                        let slice = &bytes[offset..end];
                                        json!({
                                            "chunk_b64": base64::engine::general_purpose::STANDARD.encode(slice),
                                            "bytes_returned": slice.len(),
                                            "eof": end == total,
                                            "total_bytes": total,
                                        })
                                    }
                                    None => json!({"error": "unknown blob"}),
                                }
                            }
                            _ => json!({"unhandled": req.method.clone()}),
                        };
                        let resp = Response::success(req.id.unwrap_or(json!(null)), result_val);
                        let mut bytes = serde_json::to_vec(&resp).unwrap();
                        bytes.push(b'\n');
                        if stream.write_all(&bytes).await.is_err() {
                            break;
                        }
                    }
                }
            })
        }
    }

    #[tokio::test]
    async fn small_payload_uses_inline_path_no_artifact_put() {
        let dir = tempfile::tempdir().unwrap();
        let hub = FakeHub::new_in(
            dir.path(),
            vec!["channel.post".into(), "artifact.put".into(), "hub.capabilities".into()],
        );
        let posts = hub.seen_posts.clone();
        let puts = hub.seen_puts.clone();
        let addr = hub.addr.clone();

        let server = tokio::spawn(async move {
            let _ = hub.run_one().await.await;
        });

        let mut client = Client::connect_addr(&addr).await.unwrap();
        let cache = HubCapabilitiesCache::new();
        let mut ctx = FallbackCtx::new();
        let identity = Identity::generate();
        let payload = b"small inline payload";
        let outcome = send_artifact_via_client(
            &mut client,
            "fake:1",
            "alpha",
            payload,
            &manifest("note.txt", payload.len() as u64),
            &identity,
            &cache,
            &mut ctx,
        )
        .await
        .unwrap();
        match outcome {
            SendOutcome::Sent { path, total_bytes, sha256, .. } => {
                assert_eq!(path, SendPath::Inline);
                assert_eq!(total_bytes, payload.len() as u64);
                assert_eq!(sha256, hex_sha256(payload));
            }
            other => panic!("expected Sent, got {other:?}"),
        }
        drop(client);
        let _ = server.await;
        assert_eq!(
            puts.lock().unwrap().len(),
            0,
            "inline path must not call artifact.put"
        );
        assert_eq!(posts.lock().unwrap().len(), 1, "exactly one channel.post");
        let post = &posts.lock().unwrap()[0];
        assert_eq!(post["msg_type"], "artifact");
        assert!(post["artifact_ref"].is_null(), "inline → no artifact_ref");
    }

    #[tokio::test]
    async fn large_payload_uses_chunked_artifact_put() {
        let dir = tempfile::tempdir().unwrap();
        let hub = FakeHub::new_in(
            dir.path(),
            vec!["channel.post".into(), "artifact.put".into(), "hub.capabilities".into()],
        );
        let posts = hub.seen_posts.clone();
        let puts = hub.seen_puts.clone();
        let addr = hub.addr.clone();

        let server = tokio::spawn(async move {
            let _ = hub.run_one().await.await;
        });

        let mut client = Client::connect_addr(&addr).await.unwrap();
        let cache = HubCapabilitiesCache::new();
        let mut ctx = FallbackCtx::new();
        let identity = Identity::generate();
        // 600KB → ~3 chunks of 256KB
        let payload = vec![0xABu8; 600 * 1024];
        let outcome = send_artifact_via_client(
            &mut client,
            "fake:1",
            "beta",
            &payload,
            &manifest("big.bin", payload.len() as u64),
            &identity,
            &cache,
            &mut ctx,
        )
        .await
        .unwrap();
        match outcome {
            SendOutcome::Sent {
                path,
                total_bytes,
                sha256,
                ..
            } => {
                assert_eq!(path, SendPath::Chunked);
                assert_eq!(total_bytes, payload.len() as u64);
                assert_eq!(sha256, hex_sha256(&payload));
            }
            other => panic!("expected Sent, got {other:?}"),
        }
        drop(client);
        let _ = server.await;
        let put_count = puts.lock().unwrap().len();
        assert!(
            put_count >= 2,
            "expected at least 2 chunks, got {put_count}"
        );
        assert_eq!(posts.lock().unwrap().len(), 1);
        let post = &posts.lock().unwrap()[0];
        assert_eq!(post["msg_type"], "artifact");
        assert_eq!(
            post["artifact_ref"].as_str().unwrap(),
            hex_sha256(&payload)
        );
    }

    #[tokio::test]
    async fn legacy_only_when_hub_lacks_channel_post() {
        let dir = tempfile::tempdir().unwrap();
        // Hub advertises ONLY legacy methods, no channel.post.
        let hub = FakeHub::new_in(dir.path(), vec!["hub.capabilities".into()]);
        let addr = hub.addr.clone();
        let server = tokio::spawn(async move {
            let _ = hub.run_one().await.await;
        });

        let mut client = Client::connect_addr(&addr).await.unwrap();
        let cache = HubCapabilitiesCache::new();
        let mut ctx = FallbackCtx::new();
        let identity = Identity::generate();
        let outcome = send_artifact_via_client(
            &mut client,
            "fake:legacy",
            "gamma",
            b"x",
            &manifest("x", 1),
            &identity,
            &cache,
            &mut ctx,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, SendOutcome::LegacyOnly));
        assert!(ctx.is_legacy_only("fake:legacy"));
        drop(client);
        let _ = server.await;
    }

    #[tokio::test]
    async fn legacy_only_short_circuit_when_already_flagged() {
        let dir = tempfile::tempdir().unwrap();
        // Hub WOULD support both, but ctx says legacy-only — should bail
        // without even probing.
        let hub = FakeHub::new_in(
            dir.path(),
            vec!["channel.post".into(), "artifact.put".into(), "hub.capabilities".into()],
        );
        let addr = hub.addr.clone();
        let server = tokio::spawn(async move {
            let _ = hub.run_one().await.await;
        });

        let mut client = Client::connect_addr(&addr).await.unwrap();
        let cache = HubCapabilitiesCache::new();
        let mut ctx = FallbackCtx::new();
        ctx.flag_legacy_only("fake:flagged");
        let identity = Identity::generate();
        let outcome = send_artifact_via_client(
            &mut client,
            "fake:flagged",
            "delta",
            b"x",
            &manifest("x", 1),
            &identity,
            &cache,
            &mut ctx,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, SendOutcome::LegacyOnly));
        drop(client);
        let _ = server.await;
    }

    fn b64_str(b: &[u8]) -> String {
        B64.encode(b)
    }

    #[tokio::test]
    async fn recv_returns_inline_and_chunked_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let hub = FakeHub::new_in(
            dir.path(),
            vec![
                "channel.subscribe".into(),
                "artifact.get".into(),
                "hub.capabilities".into(),
            ],
        );
        // Stage two envelopes: one inline, one chunked.
        let inline_payload = b"inline hello".to_vec();
        let chunked_bytes = vec![0xCDu8; 300 * 1024];
        let chunked_sha = hex_sha256(&chunked_bytes);
        let chunked_manifest = ArtifactManifest {
            filename: "big.bin".into(),
            size: chunked_bytes.len() as u64,
            from: "peer".into(),
            transfer_id: Some("xfer-1".into()),
            content_type: None,
        };
        let chunked_manifest_bytes = serde_json::to_vec(&chunked_manifest).unwrap();
        {
            let mut staged = hub.subscribe_messages.lock().unwrap();
            staged.push(json!({
                "offset": 1,
                "msg_type": "artifact",
                "payload_b64": b64_str(&inline_payload),
                "artifact_ref": null,
                "ts": 1,
                "sender_id": "peer",
            }));
            staged.push(json!({
                "offset": 2,
                "msg_type": "artifact",
                "payload_b64": b64_str(&chunked_manifest_bytes),
                "artifact_ref": chunked_sha.clone(),
                "ts": 2,
                "sender_id": "peer",
            }));
            staged.push(json!({
                "offset": 3,
                "msg_type": "note",
                "payload_b64": b64_str(b"non-artifact"),
                "ts": 3,
                "sender_id": "peer",
            }));
            hub.blobs.lock().unwrap().insert(chunked_sha.clone(), chunked_bytes.clone());
        }
        let addr = hub.addr.clone();

        let server = tokio::spawn(async move {
            let _ = hub.run_one().await.await;
        });

        let mut client = Client::connect_addr(&addr).await.unwrap();
        let cache = HubCapabilitiesCache::new();
        let mut ctx = FallbackCtx::new();

        let outcome = recv_artifacts_via_client(
            &mut client,
            "fake:recv",
            "alpha",
            0,
            &cache,
            &mut ctx,
        )
        .await
        .unwrap();
        let (artifacts, next_cursor) = match outcome {
            RecvOutcome::Received {
                artifacts,
                next_cursor,
            } => (artifacts, next_cursor),
            other => panic!("expected Received, got {other:?}"),
        };
        assert_eq!(artifacts.len(), 2, "non-artifact filtered out");
        assert!(next_cursor >= 2);
        // Inline
        assert!(artifacts[0].artifact_ref.is_none());
        assert_eq!(artifacts[0].payload, inline_payload);
        // Chunked
        assert_eq!(artifacts[1].artifact_ref.as_deref(), Some(chunked_sha.as_str()));
        let m = artifacts[1].manifest.as_ref().unwrap();
        assert_eq!(m.filename, "big.bin");

        // Now download the chunked blob.
        let bytes = download_artifact_via_client(&mut client, &chunked_sha)
            .await
            .unwrap();
        assert_eq!(bytes.len(), chunked_bytes.len());
        assert_eq!(hex_sha256(&bytes), chunked_sha);

        drop(client);
        let _ = server.await;
    }

    #[tokio::test]
    async fn recv_legacy_only_when_subscribe_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let hub = FakeHub::new_in(dir.path(), vec!["hub.capabilities".into()]);
        let addr = hub.addr.clone();
        let server = tokio::spawn(async move {
            let _ = hub.run_one().await.await;
        });
        let mut client = Client::connect_addr(&addr).await.unwrap();
        let cache = HubCapabilitiesCache::new();
        let mut ctx = FallbackCtx::new();
        let outcome = recv_artifacts_via_client(
            &mut client,
            "fake:legacy-recv",
            "beta",
            0,
            &cache,
            &mut ctx,
        )
        .await
        .unwrap();
        assert!(matches!(outcome, RecvOutcome::LegacyOnly));
        assert!(ctx.is_legacy_only("fake:legacy-recv"));
        drop(client);
        let _ = server.await;
    }
}
