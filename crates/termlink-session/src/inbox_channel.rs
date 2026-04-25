//! T-1220 wedge a (T-1225): inbox.* receiver migration helper.
//!
//! Single async entry point that callers (T-1226 CLI local, T-1227 CLI remote,
//! T-1228 MCP) use instead of the legacy `inbox.list` RPC. Probes capabilities
//! via T-1215's `HubCapabilitiesCache`, dispatches to
//! `channel.subscribe(topic="inbox:<target>")` when the peer hub supports it,
//! falls back to legacy `inbox.list` otherwise. Reassembles per-transfer
//! summaries from the file.init/chunk/complete event stream mirrored by T-1163
//! (`channel::mirror_inbox_deposit`).
//!
//! Inception decisions (T-1220 GO):
//! - Q1 cursor: in-memory per-target on `FallbackCtx` (no on-disk persistence).
//! - Q2 capabilities probe timing: per-call via the shared cache (cheap on hits).
//! - Q3 fallback: warn-once per `(host_port, kind)` + flag peer legacy-only on
//!   `method-not-found`.
//! - Q4 clear semantics: out of scope for the helper — wedge b/c/d advance the
//!   local cursor only.
//! - Q5 mixed-mode: dual-read merge layer is a follow-up; this wedge ships the
//!   single-source dispatcher first.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use termlink_protocol::jsonrpc::RpcResponse;
use termlink_protocol::{control, TransportAddr};

use crate::client::{Client, ClientError};
use crate::hub_capabilities::HubCapabilitiesCache;

const TOPIC_PREFIX: &str = "inbox:";
const RPC_METHOD_NOT_FOUND: i64 = -32601;

/// Summary of one pending transfer in a target's inbox. Mirrors the
/// `inbox::PendingTransfer` fields that current renderers actually consume so
/// callers swap `inbox.list` → `list_with_fallback` without touching display
/// code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InboxEntry {
    pub transfer_id: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub chunks_received: u32,
    #[serde(default)]
    pub total_chunks: u32,
    #[serde(default)]
    pub complete: bool,
}

/// Per-call mutable state — cursor map + warn-once tracker + legacy-only flags.
/// Callers that want process-wide sharing wrap a single instance in their own
/// `Mutex`; T-1225 keeps construction explicit so test setups can stage cursors
/// without globals.
#[derive(Debug, Default)]
pub struct FallbackCtx {
    cursors: HashMap<String, u64>,
    warned: HashSet<(String, &'static str)>,
    legacy_only_peers: HashSet<String>,
}

impl FallbackCtx {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a warn-once event. Returns `true` the first time `(host_port, kind)`
    /// is seen, `false` thereafter — caller emits the tracing line on `true`.
    pub fn warn_once(&mut self, host_port: &str, kind: &'static str) -> bool {
        self.warned.insert((host_port.to_string(), kind))
    }

    /// Mark a peer as legacy-only so future calls skip the channel.* dispatch.
    pub fn flag_legacy_only(&mut self, host_port: &str) {
        self.legacy_only_peers.insert(host_port.to_string());
    }

    pub fn is_legacy_only(&self, host_port: &str) -> bool {
        self.legacy_only_peers.contains(host_port)
    }

    #[cfg(test)]
    pub fn set_cursor(&mut self, topic: impl Into<String>, cursor: u64) {
        self.cursors.insert(topic.into(), cursor);
    }

    #[cfg(test)]
    pub fn cursor(&self, topic: &str) -> Option<u64> {
        self.cursors.get(topic).copied()
    }
}

/// Probe + dispatch + reassemble. Single entry point for the wedge-b/d-local
/// callers using an unauthenticated socket. Opens one `Client::connect_addr`
/// for the whole probe→dispatch sequence.
///
/// For callers that already hold an authenticated `Client` (CLI remote / MCP
/// remote), use `list_with_fallback_with_client` instead.
pub async fn list_with_fallback(
    addr: &TransportAddr,
    target: &str,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<Vec<InboxEntry>> {
    let host_port = host_port_str(addr);
    let mut client = Client::connect_addr(addr)
        .await
        .map_err(|e| io::Error::other(format!("connect {host_port}: {e}")))?;
    list_with_fallback_with_client(&mut client, &host_port, target, cache, ctx).await
}

/// T-1231: Variant for callers who already hold an authenticated `Client`
/// (CLI remote `cmd_remote_inbox_*`, MCP remote `termlink_remote_inbox_*`).
/// Caller supplies the `host_port` string for cache-key + warn-once dedup.
pub async fn list_with_fallback_with_client(
    client: &mut Client,
    host_port: &str,
    target: &str,
    cache: &HubCapabilitiesCache,
    ctx: &mut FallbackCtx,
) -> io::Result<Vec<InboxEntry>> {
    let topic = format!("{TOPIC_PREFIX}{target}");

    let use_channel = if ctx.is_legacy_only(host_port) {
        false
    } else {
        let methods = probe_caps_via_client(client, host_port, cache)
            .await
            .unwrap_or_default();
        methods
            .iter()
            .any(|m| m == control::method::CHANNEL_SUBSCRIBE)
    };

    if use_channel {
        let saved_cursor = ctx.cursors.get(&topic).copied().unwrap_or(0);
        match call_channel_subscribe_via_client(client, &topic, saved_cursor).await {
            Ok((messages, next_cursor)) => {
                if ctx.warn_once(host_port, "channel.subscribe") {
                    tracing::info!(
                        host = %host_port,
                        target = %target,
                        "T-1225: using channel.subscribe (channel.* supported)"
                    );
                }
                ctx.cursors.insert(topic, next_cursor);
                return Ok(fold_envelopes(&messages));
            }
            Err(SubscribeError::MethodNotFound) => {
                ctx.flag_legacy_only(host_port);
                if ctx.warn_once(host_port, "channel.subscribe.missing") {
                    tracing::warn!(
                        host = %host_port,
                        target = %target,
                        "T-1225: channel.subscribe missing despite cap claim — flagging legacy-only"
                    );
                }
            }
            Err(SubscribeError::Other(e)) => return Err(e),
        }
    }

    if ctx.warn_once(host_port, "inbox.list") {
        tracing::info!(
            host = %host_port,
            target = %target,
            "T-1225: using legacy inbox.list (channel.* unavailable)"
        );
    }
    call_legacy_inbox_list_via_client(client, target).await
}

/// Internal error type for the channel-subscribe leg so `list_with_fallback`
/// can react to method-not-found without parsing strings.
enum SubscribeError {
    MethodNotFound,
    Other(io::Error),
}

fn map_client_err(label: &str, e: ClientError) -> io::Error {
    io::Error::other(format!("{label}: {e}"))
}

async fn probe_caps_via_client(
    client: &mut Client,
    host_port: &str,
    cache: &HubCapabilitiesCache,
) -> io::Result<Vec<String>> {
    if let Some(cached) = cache.get(host_port) {
        return Ok(cached);
    }
    let resp = client
        .call(
            control::method::HUB_CAPABILITIES,
            json!("inbox-probe"),
            json!({}),
        )
        .await
        .map_err(|e| map_client_err("hub.capabilities", e))?;
    let methods = extract_methods(&resp)?;
    cache.set(host_port.to_string(), methods.clone());
    Ok(methods)
}

fn extract_methods(resp: &RpcResponse) -> io::Result<Vec<String>> {
    match resp {
        RpcResponse::Success(ok) => Ok(ok
            .result
            .get("methods")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()),
        RpcResponse::Error(e) => Err(io::Error::other(format!(
            "hub.capabilities error: {}",
            e.error.message
        ))),
    }
}

async fn call_channel_subscribe_via_client(
    client: &mut Client,
    topic: &str,
    cursor: u64,
) -> Result<(Vec<Value>, u64), SubscribeError> {
    let resp = client
        .call(
            control::method::CHANNEL_SUBSCRIBE,
            json!("inbox-sub"),
            json!({
                "topic": topic,
                "cursor": cursor,
                "limit": 1000,
            }),
        )
        .await
        .map_err(|e| SubscribeError::Other(map_client_err("channel.subscribe", e)))?;

    match resp {
        RpcResponse::Success(ok) => {
            let messages = ok
                .result
                .get("messages")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let next_cursor = ok
                .result
                .get("next_cursor")
                .and_then(|v| v.as_u64())
                .unwrap_or(cursor);
            Ok((messages, next_cursor))
        }
        RpcResponse::Error(e) if e.error.code == RPC_METHOD_NOT_FOUND => {
            Err(SubscribeError::MethodNotFound)
        }
        RpcResponse::Error(e) => Err(SubscribeError::Other(io::Error::other(format!(
            "channel.subscribe error {}: {}",
            e.error.code, e.error.message
        )))),
    }
}

async fn call_legacy_inbox_list_via_client(
    client: &mut Client,
    target: &str,
) -> io::Result<Vec<InboxEntry>> {
    let resp = client
        .call("inbox.list", json!("inbox-l"), json!({"target": target}))
        .await
        .map_err(|e| map_client_err("inbox.list", e))?;
    match resp {
        RpcResponse::Success(ok) => {
            let arr = ok
                .result
                .get("transfers")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let entries = arr
                .into_iter()
                .filter_map(|v| serde_json::from_value::<InboxEntry>(v).ok())
                .collect();
            Ok(entries)
        }
        RpcResponse::Error(e) => Err(io::Error::other(format!(
            "inbox.list error {}: {}",
            e.error.code, e.error.message
        ))),
    }
}

/// Fold a stream of `inbox:<target>` channel envelopes into per-transfer
/// summaries. Drops transfers that emit `file.error`. Public for direct use by
/// dual-read mergers in a follow-up wedge.
pub fn fold_envelopes(messages: &[Value]) -> Vec<InboxEntry> {
    let mut by_id: BTreeMap<String, InboxEntry> = BTreeMap::new();
    let mut errored: HashSet<String> = HashSet::new();

    for msg in messages {
        let msg_type = msg.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        let payload_b64 = msg
            .get("payload_b64")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let decoded = match B64.decode(payload_b64) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let mirror: Value = match serde_json::from_slice(&decoded) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let inner = mirror.get("payload").cloned().unwrap_or(Value::Null);
        let transfer_id = inner
            .get("transfer_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if transfer_id.is_empty() {
            continue;
        }

        match msg_type {
            "file.init" => {
                let entry = InboxEntry {
                    transfer_id: transfer_id.clone(),
                    filename: inner
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    from: inner
                        .get("from")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    size: inner.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                    chunks_received: 0,
                    total_chunks: inner
                        .get("total_chunks")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    complete: false,
                };
                by_id.entry(transfer_id).or_insert(entry);
            }
            "file.chunk" => {
                if let Some(e) = by_id.get_mut(&transfer_id) {
                    e.chunks_received = e.chunks_received.saturating_add(1);
                }
            }
            "file.complete" => {
                if let Some(e) = by_id.get_mut(&transfer_id) {
                    e.complete = true;
                    if e.total_chunks > 0 {
                        e.chunks_received = e.total_chunks;
                    }
                }
            }
            "file.error" => {
                errored.insert(transfer_id);
            }
            _ => {}
        }
    }

    by_id
        .into_iter()
        .filter(|(id, _)| !errored.contains(id))
        .map(|(_, v)| v)
        .collect()
}

fn host_port_str(addr: &TransportAddr) -> String {
    match addr {
        TransportAddr::Tcp { host, port } => format!("{host}:{port}"),
        TransportAddr::Unix { path } => format!("unix:{}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth_msg(msg_type: &str, payload: Value) -> Value {
        let mirror = json!({"from": "session-A", "payload": payload});
        let bytes = serde_json::to_vec(&mirror).unwrap();
        json!({
            "offset": 0,
            "topic": "inbox:test-target",
            "sender_id": "hub:inbox.deposit",
            "msg_type": msg_type,
            "payload_b64": B64.encode(&bytes),
            "artifact_ref": null,
            "ts": 0,
        })
    }

    #[test]
    fn fold_envelopes_assembles_pending_transfer() {
        let msgs = vec![
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-1",
                    "filename": "a.bin",
                    "from": "alpha",
                    "size": 1024,
                    "total_chunks": 1
                }),
            ),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-1", "index": 0})),
            synth_msg(
                "file.complete",
                json!({"transfer_id": "xfer-1", "sha256": "deadbeef"}),
            ),
        ];
        let entries = fold_envelopes(&msgs);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.transfer_id, "xfer-1");
        assert_eq!(e.filename, "a.bin");
        assert_eq!(e.size, 1024);
        assert_eq!(e.total_chunks, 1);
        assert_eq!(e.chunks_received, 1);
        assert!(e.complete);
    }

    #[test]
    fn fold_envelopes_groups_by_transfer_id() {
        let msgs = vec![
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-A",
                    "filename": "a",
                    "from": "x",
                    "size": 10,
                    "total_chunks": 2
                }),
            ),
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-B",
                    "filename": "b",
                    "from": "x",
                    "size": 20,
                    "total_chunks": 1
                }),
            ),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-A", "index": 0})),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-A", "index": 1})),
            synth_msg("file.chunk", json!({"transfer_id": "xfer-B", "index": 0})),
        ];
        let entries = fold_envelopes(&msgs);
        assert_eq!(entries.len(), 2);
        let a = entries.iter().find(|e| e.transfer_id == "xfer-A").unwrap();
        let b = entries.iter().find(|e| e.transfer_id == "xfer-B").unwrap();
        assert_eq!(a.chunks_received, 2);
        assert_eq!(b.chunks_received, 1);
        assert!(!a.complete);
        assert!(!b.complete);
    }

    #[test]
    fn fold_envelopes_drops_errored_transfer() {
        let msgs = vec![
            synth_msg(
                "file.init",
                json!({
                    "transfer_id": "xfer-bad",
                    "filename": "x",
                    "from": "x",
                    "size": 1,
                    "total_chunks": 1
                }),
            ),
            synth_msg(
                "file.error",
                json!({"transfer_id": "xfer-bad", "message": "boom"}),
            ),
        ];
        assert!(fold_envelopes(&msgs).is_empty());
    }

    #[test]
    fn fold_envelopes_ignores_malformed_messages() {
        let msgs = vec![
            json!({"msg_type": "file.init", "payload_b64": "@@@not-base64@@@"}),
            json!({"msg_type": "file.init", "payload_b64": B64.encode(b"not-json")}),
            synth_msg("file.init", json!({})), // missing transfer_id
        ];
        assert!(fold_envelopes(&msgs).is_empty());
    }

    #[test]
    fn fallback_ctx_warn_once_dedupes() {
        let mut ctx = FallbackCtx::new();
        assert!(ctx.warn_once("h:1", "channel.subscribe"));
        assert!(!ctx.warn_once("h:1", "channel.subscribe"));
        assert!(ctx.warn_once("h:2", "channel.subscribe"));
        assert!(ctx.warn_once("h:1", "inbox.list"));
    }

    #[test]
    fn fallback_ctx_legacy_only_flag() {
        let mut ctx = FallbackCtx::new();
        assert!(!ctx.is_legacy_only("h:1"));
        ctx.flag_legacy_only("h:1");
        assert!(ctx.is_legacy_only("h:1"));
        assert!(!ctx.is_legacy_only("h:2"));
    }

    #[test]
    fn fallback_ctx_cursor_roundtrip() {
        let mut ctx = FallbackCtx::new();
        assert!(ctx.cursor("inbox:t").is_none());
        ctx.set_cursor("inbox:t", 42);
        assert_eq!(ctx.cursor("inbox:t"), Some(42));
    }
}
