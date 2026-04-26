//! Hub-side handlers for the T-1160 `channel.*` RPC surface.
//!
//! The hub owns one [`termlink_bus::Bus`] keyed off `<runtime_dir>/bus/`
//! (initialised once at server start by [`init_bus`]). Every `channel.post`
//! arriving over JSON-RPC is verified against the sender's ed25519 public
//! key using the canonical byte layout defined in
//! `termlink_protocol::control::channel::canonical_sign_bytes` before the
//! envelope is appended to the bus. All four verbs are Tier-A per T-1133:
//! payloads are opaque `base64(String)` on the wire.

use std::path::PathBuf;
use std::sync::OnceLock;

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::{json, Value};

use termlink_bus::{ArtifactStore, Bus, Envelope, Retention};
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_protocol::control::error_code;
use termlink_protocol::jsonrpc::{ErrorResponse, Response, RpcResponse};

static BUS: OnceLock<Bus> = OnceLock::new();
static ARTIFACT_STORE: OnceLock<ArtifactStore> = OnceLock::new();

/// The canonical topic T-1162 mirrors `event.broadcast` into so subscribers
/// can read fan-out events via the new `channel.*` surface without waiting
/// for every producer to migrate.
pub const BROADCAST_GLOBAL_TOPIC: &str = "broadcast:global";

/// Initialise the global bus. Called once by the hub server at startup.
/// Safe to call before any `channel.*` request arrives; panics only on
/// filesystem / SQLite failure (a hub that cannot open its bus cannot serve).
pub fn init_bus(root: PathBuf) {
    let bus = Bus::open(&root)
        .unwrap_or_else(|e| panic!("failed to open channel bus at {}: {e}", root.display()));
    // T-1162: auto-register broadcast:global so the event.broadcast shim can
    // dual-write without a separate bootstrap step. Idempotent on name+policy.
    if let Err(e) = bus.create_topic(BROADCAST_GLOBAL_TOPIC, Retention::Messages(1000)) {
        tracing::warn!(
            topic = BROADCAST_GLOBAL_TOPIC,
            error = %e,
            "T-1162: could not auto-create broadcast mirror topic — shim will no-op"
        );
    }
    let _ = BUS.set(bus);

    // T-1248 / T-1164a: Initialize content-addressed artifact store at
    // <bus-root>/artifacts/. Hub-side handlers in `crate::artifact` use this.
    let store = ArtifactStore::open(root.join("artifacts")).unwrap_or_else(|e| {
        panic!(
            "failed to open artifact store at {}/artifacts: {e}",
            root.display()
        )
    });
    let _ = ARTIFACT_STORE.set(store);
}

/// T-1162: Mirror an `event.broadcast` payload into the `broadcast:global`
/// channel topic. Best-effort — never fails the caller; logs on error.
/// No signature required (hub-originated envelope; `sender_id` marks origin).
pub async fn mirror_event_broadcast(topic: &str, payload: &Value) {
    let Some(bus) = bus() else {
        tracing::debug!("T-1162 mirror: bus not initialised — skipping");
        return;
    };
    mirror_event_broadcast_with(bus, topic, payload).await;
}

/// Test-friendly variant: post into the supplied `Bus` instead of the
/// process-global one so unit tests can verify mirror behaviour against
/// an isolated tempdir-rooted bus.
pub(crate) async fn mirror_event_broadcast_with(bus: &Bus, topic: &str, payload: &Value) {
    let payload_bytes = match serde_json::to_vec(payload) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "T-1162 mirror: failed to serialize payload");
            return;
        }
    };
    let ts_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let env = Envelope {
        topic: BROADCAST_GLOBAL_TOPIC.to_string(),
        sender_id: "hub:event.broadcast".to_string(),
        msg_type: topic.to_string(),
        payload: payload_bytes,
        artifact_ref: None,
        ts_unix_ms,
        metadata: Default::default(),
    };
    if let Err(e) = bus.post(BROADCAST_GLOBAL_TOPIC, &env).await {
        tracing::warn!(error = %e, "T-1162 mirror: bus.post failed");
    }
}

/// T-1163: Mirror a successful `inbox::deposit` into the per-target
/// `inbox:<target>` channel topic. Best-effort — never fails the caller;
/// logs on error. The topic is auto-created on first deposit via the
/// idempotent `Bus::create_topic` (Retention::Messages(1000)).
/// No signature required (hub-originated envelope; `sender_id` marks origin).
pub async fn mirror_inbox_deposit(
    target: &str,
    topic: &str,
    payload: &Value,
    from: Option<&str>,
) {
    let Some(bus) = bus() else {
        tracing::debug!("T-1163 mirror: bus not initialised — skipping");
        return;
    };
    mirror_inbox_deposit_with(bus, target, topic, payload, from).await;
}

/// Test-friendly variant: post into the supplied `Bus` instead of the
/// process-global one so unit tests can verify mirror behaviour against
/// an isolated tempdir-rooted bus.
pub(crate) async fn mirror_inbox_deposit_with(
    bus: &Bus,
    target: &str,
    topic: &str,
    payload: &Value,
    from: Option<&str>,
) {
    let topic_name = format!("inbox:{target}");
    if let Err(e) = bus.create_topic(&topic_name, Retention::Messages(1000)) {
        tracing::warn!(
            topic = %topic_name,
            error = %e,
            "T-1163 mirror: create_topic failed — will still try bus.post"
        );
    }
    let mirror_payload = json!({
        "from": from,
        "payload": payload,
    });
    let payload_bytes = match serde_json::to_vec(&mirror_payload) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "T-1163 mirror: failed to serialize payload");
            return;
        }
    };
    let ts_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let env = Envelope {
        topic: topic_name.clone(),
        sender_id: "hub:inbox.deposit".to_string(),
        msg_type: topic.to_string(),
        payload: payload_bytes,
        artifact_ref: None,
        ts_unix_ms,
        metadata: Default::default(),
    };
    if let Err(e) = bus.post(&topic_name, &env).await {
        tracing::warn!(topic = %topic_name, error = %e, "T-1163 mirror: bus.post failed");
    }
}

pub(crate) fn bus() -> Option<&'static Bus> {
    BUS.get()
}

/// Access the hub's content-addressed artifact store. T-1248 / T-1164a.
pub(crate) fn artifact_store() -> Option<&'static ArtifactStore> {
    ARTIFACT_STORE.get()
}

fn bus_or_err(id: Value) -> std::result::Result<&'static Bus, RpcResponse> {
    match bus() {
        Some(b) => Ok(b),
        None => Err(ErrorResponse::internal_error(id, "channel bus not initialized").into()),
    }
}

fn param_str<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(|v| v.as_str())
}

fn retention_from_json(val: &Value) -> Option<Retention> {
    let kind = val.get("kind").and_then(|v| v.as_str())?;
    match kind {
        "forever" => Some(Retention::Forever),
        "days" => {
            let d = val.get("value").and_then(|v| v.as_u64())?;
            Some(Retention::Days(d.min(u64::from(u32::MAX)) as u32))
        }
        "messages" => {
            let n = val.get("value").and_then(|v| v.as_u64())?;
            Some(Retention::Messages(n))
        }
        _ => None,
    }
}

fn retention_to_json(r: Retention) -> Value {
    match r {
        Retention::Forever => json!({"kind": "forever"}),
        Retention::Days(d) => json!({"kind": "days", "value": d}),
        Retention::Messages(n) => json!({"kind": "messages", "value": n}),
    }
}

/// `channel.create(name, retention)` — idempotent on name.
pub async fn handle_channel_create(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_create_with(bus, id, params).await
}

pub(crate) async fn handle_channel_create_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let name = match param_str(params, "name") {
        Some(n) if !n.is_empty() => n,
        _ => return ErrorResponse::new(id, -32602, "Missing 'name' in params").into(),
    };
    let retention = params
        .get("retention")
        .and_then(retention_from_json)
        .unwrap_or(Retention::Forever);
    match bus.create_topic(name, retention) {
        Ok(()) => Response::success(
            id,
            json!({"ok": true, "name": name, "retention": retention_to_json(retention)}),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.create: {e}")).into(),
    }
}

/// `channel.post(topic, msg_type, payload_b64, artifact_ref?, ts, sender_id,
/// sender_pubkey_hex, signature_hex)` — verifies signature then appends.
pub async fn handle_channel_post(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_post_with(bus, id, params).await
}

pub(crate) async fn handle_channel_post_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let msg_type = match param_str(params, "msg_type") {
        Some(t) => t.to_string(),
        None => return ErrorResponse::new(id, -32602, "Missing 'msg_type' in params").into(),
    };
    let payload_b64 = match param_str(params, "payload_b64") {
        Some(p) => p,
        None => return ErrorResponse::new(id, -32602, "Missing 'payload_b64' in params").into(),
    };
    let payload = match base64::engine::general_purpose::STANDARD.decode(payload_b64) {
        Ok(b) => b,
        Err(e) => return ErrorResponse::new(id, -32602, &format!("payload_b64 decode: {e}")).into(),
    };
    let artifact_ref = param_str(params, "artifact_ref").map(|s| s.to_string());
    let ts_unix_ms = params
        .get("ts")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0)
        });
    let sender_id = match param_str(params, "sender_id") {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return ErrorResponse::new(id, -32602, "Missing 'sender_id' in params").into(),
    };
    let pubkey_hex = match param_str(params, "sender_pubkey_hex") {
        Some(p) => p,
        None => {
            return ErrorResponse::new(id, -32602, "Missing 'sender_pubkey_hex' in params").into();
        }
    };
    let signature_hex = match param_str(params, "signature_hex") {
        Some(s) => s,
        None => return ErrorResponse::new(id, -32602, "Missing 'signature_hex' in params").into(),
    };

    let verifying_key = match parse_pubkey_hex(pubkey_hex) {
        Some(k) => k,
        None => {
            return ErrorResponse::new(
                id,
                error_code::CHANNEL_SIGNATURE_INVALID,
                "sender_pubkey_hex is not a valid 32-byte ed25519 public key",
            )
            .into();
        }
    };
    let signature = match parse_signature_hex(signature_hex) {
        Some(s) => s,
        None => {
            return ErrorResponse::new(
                id,
                error_code::CHANNEL_SIGNATURE_INVALID,
                "signature_hex is not a valid 64-byte ed25519 signature",
            )
            .into();
        }
    };
    let signed_bytes = canonical_sign_bytes(
        &topic,
        &msg_type,
        &payload,
        artifact_ref.as_deref(),
        ts_unix_ms,
    );
    if verifying_key.verify(&signed_bytes, &signature).is_err() {
        return ErrorResponse::new(
            id,
            error_code::CHANNEL_SIGNATURE_INVALID,
            "channel.post signature failed verification",
        )
        .into();
    }

    // T-1287: optional metadata routing-hint map. NOT included in canonical
    // signed bytes — trusted-mesh threat model treats it as routing only.
    // Well-known keys: conversation_id, event_type (per T-1288 catalog).
    let metadata: std::collections::BTreeMap<String, String> = params
        .get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let env = Envelope {
        topic: topic.clone(),
        sender_id,
        msg_type,
        payload,
        artifact_ref,
        ts_unix_ms,
        metadata,
    };
    match bus.post(&topic, &env).await {
        Ok(offset) => Response::success(id, json!({"offset": offset, "ts": ts_unix_ms})).into(),
        Err(termlink_bus::BusError::UnknownTopic(t)) => ErrorResponse::new(
            id,
            error_code::CHANNEL_TOPIC_UNKNOWN,
            &format!("unknown topic: {t}"),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.post: {e}")).into(),
    }
}

/// `channel.subscribe(topic, cursor?, limit?)` → `{messages, next_cursor}`.
pub async fn handle_channel_subscribe(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_subscribe_with(bus, id, params).await
}

pub(crate) async fn handle_channel_subscribe_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let cursor = params
        .get("cursor")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(100)
        .min(1000);
    // T-1289: optional long-poll. When `timeout_ms > 0` and no records are
    // immediately available at `cursor`, block on the bus's per-topic
    // notifier up to `timeout_ms` for the next post. Capped at 60_000 (60s)
    // to bound RPC handler lifetime; clients can re-call to extend.
    let timeout_ms = params
        .get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(60_000);
    // T-1287: optional conversation_id filter. When present, only envelopes
    // whose metadata.conversation_id == filter are included. Filter applies
    // BEFORE the limit, so a small number of matching messages can be
    // surfaced even when many non-matching ones precede them. last_offset
    // still advances over all examined records so the next_cursor moves
    // past skipped records — clients don't redundantly re-scan them.
    let conversation_id_filter = params
        .get("conversation_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let iter = if timeout_ms > 0 {
        match bus
            .subscribe_blocking(
                &topic,
                cursor,
                std::time::Duration::from_millis(timeout_ms),
            )
            .await
        {
            Ok(i) => i,
            Err(termlink_bus::BusError::UnknownTopic(t)) => {
                return ErrorResponse::new(
                    id,
                    error_code::CHANNEL_TOPIC_UNKNOWN,
                    &format!("unknown topic: {t}"),
                )
                .into();
            }
            Err(e) => {
                return ErrorResponse::internal_error(
                    id,
                    &format!("channel.subscribe (long-poll): {e}"),
                )
                .into();
            }
        }
    } else {
        match bus.subscribe(&topic, cursor) {
            Ok(i) => i,
            Err(termlink_bus::BusError::UnknownTopic(t)) => {
                return ErrorResponse::new(
                    id,
                    error_code::CHANNEL_TOPIC_UNKNOWN,
                    &format!("unknown topic: {t}"),
                )
                .into();
            }
            Err(e) => {
                return ErrorResponse::internal_error(id, &format!("channel.subscribe: {e}")).into();
            }
        }
    };

    let mut messages: Vec<Value> = Vec::new();
    let mut last_offset: Option<u64> = None;
    for item in iter {
        let (offset, env) = match item {
            Ok(x) => x,
            Err(e) => return ErrorResponse::internal_error(id, &format!("channel.subscribe decode: {e}")).into(),
        };
        last_offset = Some(offset);
        if let Some(ref cid) = conversation_id_filter {
            if env.metadata.get("conversation_id").map(|s| s.as_str()) != Some(cid.as_str()) {
                continue;
            }
        }
        messages.push(envelope_to_json(offset, &env));
        if messages.len() >= limit {
            break;
        }
    }
    let next_cursor = last_offset.map(|o| o + 1).unwrap_or(cursor);
    Response::success(
        id,
        json!({"messages": messages, "next_cursor": next_cursor}),
    )
    .into()
}

/// `channel.trim(topic, before_offset?)` → `{ok, deleted, topic}`.
/// Destructive: removes records from the hub-side log. Affects ALL
/// subscribers. Mirrors legacy `inbox.clear` semantics. T-1234 / T-1230a.
pub async fn handle_channel_trim(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_trim_with(bus, id, params).await
}

pub(crate) async fn handle_channel_trim_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let before_offset = params.get("before_offset").and_then(|v| v.as_u64());
    let deleted = match bus.trim_topic(topic, before_offset) {
        Ok(n) => n,
        Err(e) => return ErrorResponse::internal_error(id, &format!("channel.trim: {e}")).into(),
    };
    Response::success(
        id,
        json!({"ok": true, "deleted": deleted, "topic": topic}),
    )
    .into()
}

/// `channel.list(prefix?)` → `{topics: [{name, retention}]}`.
pub async fn handle_channel_list(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_list_with(bus, id, params).await
}

pub(crate) async fn handle_channel_list_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let prefix = param_str(params, "prefix").unwrap_or("");
    let names = match bus.list_topics() {
        Ok(v) => v,
        Err(e) => return ErrorResponse::internal_error(id, &format!("channel.list: {e}")).into(),
    };
    let filtered: Vec<Value> = names
        .into_iter()
        .filter(|n| prefix.is_empty() || n.starts_with(prefix))
        .map(|name| {
            let ret = bus
                .topic_retention(&name)
                .ok()
                .flatten()
                .unwrap_or(Retention::Forever);
            // T-1233 / T-1229a: count enables single-round-trip aggregation
            // (e.g. inbox.status replacement). Per-topic count error degrades
            // to 0 rather than failing the whole list.
            let count = bus.topic_record_count(&name).unwrap_or(0);
            json!({"name": name, "retention": retention_to_json(ret), "count": count})
        })
        .collect();
    Response::success(id, json!({"topics": filtered})).into()
}

fn envelope_to_json(offset: u64, env: &Envelope) -> Value {
    let payload_b64 = base64::engine::general_purpose::STANDARD.encode(&env.payload);
    let mut out = json!({
        "offset": offset,
        "topic": env.topic,
        "sender_id": env.sender_id,
        "msg_type": env.msg_type,
        "payload_b64": payload_b64,
        "artifact_ref": env.artifact_ref,
        "ts": env.ts_unix_ms,
    });
    // T-1287: include metadata when non-empty. Omitted entirely for envelopes
    // that don't use metadata so the wire format stays unchanged for them.
    if !env.metadata.is_empty() {
        if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "metadata".to_string(),
                serde_json::to_value(&env.metadata).unwrap_or(Value::Null),
            );
        }
    }
    out
}

fn parse_pubkey_hex(hex: &str) -> Option<VerifyingKey> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
    }
    VerifyingKey::from_bytes(&out).ok()
}

fn parse_signature_hex(hex: &str) -> Option<Signature> {
    if hex.len() != 128 {
        return None;
    }
    let mut out = [0u8; 64];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
    }
    Some(Signature::from_bytes(&out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;
    use tempfile::TempDir;
    use termlink_protocol::jsonrpc::RpcResponse;

    fn tmp_bus() -> (TempDir, Bus) {
        let dir = TempDir::new().unwrap();
        let bus = Bus::open(dir.path()).unwrap();
        (dir, bus)
    }

    fn signing_key() -> SigningKey {
        // Deterministic for test stability.
        SigningKey::from_bytes(&[42u8; 32])
    }

    fn hex_of(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            use std::fmt::Write;
            let _ = write!(&mut s, "{b:02x}");
        }
        s
    }

    fn unwrap_success(resp: RpcResponse) -> Value {
        match resp {
            RpcResponse::Success(s) => s.result,
            RpcResponse::Error(e) => panic!("expected success, got error: {:?}", e.error),
        }
    }

    fn unwrap_error(resp: RpcResponse) -> (i64, String) {
        match resp {
            RpcResponse::Error(e) => (e.error.code, e.error.message),
            RpcResponse::Success(_) => panic!("expected error, got success"),
        }
    }

    fn post_params_with_meta(
        key: &SigningKey,
        topic: &str,
        msg_type: &str,
        payload: &[u8],
        ts: i64,
        metadata: Option<Value>,
    ) -> Value {
        let signed = canonical_sign_bytes(topic, msg_type, payload, None, ts);
        let sig = key.sign(&signed);
        let mut p = json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "ts": ts,
            "sender_id": "tester",
            "sender_pubkey_hex": hex_of(key.verifying_key().as_bytes()),
            "signature_hex": hex_of(&sig.to_bytes()),
        });
        if let (Some(meta), Some(obj)) = (metadata, p.as_object_mut()) {
            obj.insert("metadata".to_string(), meta);
        }
        p
    }

    fn post_params(
        key: &SigningKey,
        topic: &str,
        msg_type: &str,
        payload: &[u8],
        ts: i64,
    ) -> Value {
        let signed = canonical_sign_bytes(topic, msg_type, payload, None, ts);
        let sig = key.sign(&signed);
        json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "ts": ts,
            "sender_id": "tester",
            "sender_pubkey_hex": hex_of(key.verifying_key().as_bytes()),
            "signature_hex": hex_of(&sig.to_bytes()),
        })
    }

    #[tokio::test]
    async fn create_then_list_returns_topic() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_create_with(
            &bus,
            json!(1),
            &json!({"name": "broadcast:global", "retention": {"kind": "forever"}}),
        )
        .await;
        let v = unwrap_success(resp);
        assert_eq!(v["ok"], true);
        assert_eq!(v["name"], "broadcast:global");

        let list = handle_channel_list_with(&bus, json!(2), &json!({})).await;
        let v = unwrap_success(list);
        let topics = v["topics"].as_array().unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0]["name"], "broadcast:global");
        assert_eq!(topics[0]["retention"]["kind"], "forever");
    }

    #[tokio::test]
    async fn list_prefix_filters() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("a:x", Retention::Forever).unwrap();
        bus.create_topic("b:y", Retention::Messages(10)).unwrap();
        let resp =
            handle_channel_list_with(&bus, json!(1), &json!({"prefix": "a:"})).await;
        let v = unwrap_success(resp);
        let topics = v["topics"].as_array().unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0]["name"], "a:x");
    }

    #[tokio::test]
    async fn trim_topic_full_then_subscribe_empty() {
        // T-1234 / T-1230a: channel.trim removes hub-side records, affecting all subscribers.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:carol", Retention::Forever).unwrap();
        let key = signing_key();
        for n in 0u32..3 {
            let p = post_params(&key, "inbox:carol", "note", &n.to_le_bytes(), 1_000 + n as i64);
            let _ = handle_channel_post_with(&bus, json!(n), &p).await;
        }
        // Full trim
        let trim = handle_channel_trim_with(
            &bus,
            json!(99),
            &json!({"topic": "inbox:carol"}),
        )
        .await;
        let v = unwrap_success(trim);
        assert_eq!(v["ok"], true);
        assert_eq!(v["deleted"], 3);
        assert_eq!(v["topic"], "inbox:carol");
        // Subsequent subscribe returns empty
        let sub = handle_channel_subscribe_with(
            &bus,
            json!(100),
            &json!({"topic": "inbox:carol", "cursor": 0}),
        )
        .await;
        let v = unwrap_success(sub);
        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 0);
    }

    #[tokio::test]
    async fn subscribe_timeout_ms_returns_empty_on_no_records() {
        // T-1289: long-poll RPC with empty topic + 100ms timeout returns
        // an empty `messages` array (no error). Proves the timeout_ms path
        // is wired through and behaves like snapshot when nothing arrives.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:dora", Retention::Forever).unwrap();
        let start = std::time::Instant::now();
        let resp = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({"topic": "inbox:dora", "cursor": 0, "timeout_ms": 100}),
        )
        .await;
        let elapsed = start.elapsed();
        let v = unwrap_success(resp);
        assert_eq!(v["messages"].as_array().unwrap().len(), 0);
        // Must have actually waited (proves long-poll engaged).
        assert!(
            elapsed >= std::time::Duration::from_millis(80),
            "long-poll returned too quickly: {elapsed:?}"
        );
        // Sanity bound — should not exceed timeout by more than scheduler slack.
        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "long-poll overran timeout: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn subscribe_timeout_ms_zero_is_snapshot() {
        // T-1289: timeout_ms=0 (default) preserves snapshot semantics — no
        // long-poll, returns immediately even on empty topic.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:eve", Retention::Forever).unwrap();
        let start = std::time::Instant::now();
        let resp = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "inbox:eve", "cursor": 0}),
        )
        .await;
        let elapsed = start.elapsed();
        let v = unwrap_success(resp);
        assert_eq!(v["messages"].as_array().unwrap().len(), 0);
        // Snapshot path — should be effectively instant.
        assert!(
            elapsed < std::time::Duration::from_millis(50),
            "snapshot path took {elapsed:?}, expected < 50ms"
        );
    }

    #[tokio::test]
    async fn trim_missing_topic_returns_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_trim_with(&bus, json!(1), &json!({})).await;
        match resp {
            RpcResponse::Error(e) => assert_eq!(e.error.code, -32602),
            _ => panic!("expected error"),
        }
    }

    #[tokio::test]
    async fn list_includes_count_per_topic() {
        // T-1233 / T-1229a: channel.list returns count alongside name + retention
        // so callers can replace inbox.status (server-side aggregation, single round-trip).
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:alice", Retention::Forever).unwrap();
        bus.create_topic("inbox:bob", Retention::Forever).unwrap();
        bus.create_topic("event:noise", Retention::Forever).unwrap();
        let key = signing_key();
        for n in 0..3u32 {
            let p = post_params(&key, "inbox:alice", "note", &n.to_le_bytes(), 1_000 + n as i64);
            let _ = handle_channel_post_with(&bus, json!(n), &p).await;
        }
        let resp = handle_channel_list_with(
            &bus,
            json!(7),
            &json!({"prefix": "inbox:"}),
        )
        .await;
        let v = unwrap_success(resp);
        let topics = v["topics"].as_array().unwrap();
        let alice = topics.iter().find(|t| t["name"] == "inbox:alice").unwrap();
        let bob = topics.iter().find(|t| t["name"] == "inbox:bob").unwrap();
        assert_eq!(alice["count"], 3, "alice should have 3 posted records");
        assert_eq!(bob["count"], 0, "bob should have 0 records");
        // Prefix filter excludes event:noise
        assert!(topics.iter().all(|t| t["name"] != "event:noise"));
    }

    #[tokio::test]
    async fn post_then_subscribe_roundtrip() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let key = signing_key();
        let post = handle_channel_post_with(
            &bus,
            json!(1),
            &post_params(&key, "t", "note", b"hello", 1_000_000),
        )
        .await;
        let v = unwrap_success(post);
        assert_eq!(v["offset"], 0);
        assert_eq!(v["ts"], 1_000_000);

        let sub = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "t", "cursor": 0}),
        )
        .await;
        let v = unwrap_success(sub);
        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["offset"], 0);
        assert_eq!(msgs[0]["sender_id"], "tester");
        assert_eq!(msgs[0]["msg_type"], "note");
        assert_eq!(
            msgs[0]["payload_b64"].as_str().unwrap(),
            base64::engine::general_purpose::STANDARD.encode(b"hello")
        );
        assert_eq!(v["next_cursor"], 1);
    }

    #[tokio::test]
    async fn bad_signature_rejected_with_typed_error() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let key = signing_key();
        // Flip one byte of the signature.
        let mut params = post_params(&key, "t", "note", b"hello", 0);
        let orig = params["signature_hex"].as_str().unwrap().to_string();
        let tampered = format!(
            "{}{}",
            &orig[..orig.len() - 2],
            if &orig[orig.len() - 2..] == "00" { "01" } else { "00" },
        );
        params["signature_hex"] = json!(tampered);
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let (code, msg) = unwrap_error(resp);
        assert_eq!(code, error_code::CHANNEL_SIGNATURE_INVALID);
        assert!(msg.contains("verification") || msg.contains("valid"), "msg={msg}");
    }

    #[tokio::test]
    async fn post_to_unknown_topic_returns_typed_error() {
        let (_d, bus) = tmp_bus();
        let key = signing_key();
        let resp = handle_channel_post_with(
            &bus,
            json!(1),
            &post_params(&key, "never-created", "note", b"hi", 0),
        )
        .await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, error_code::CHANNEL_TOPIC_UNKNOWN);
    }

    #[tokio::test]
    async fn subscribe_cursor_advances_across_calls() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let key = signing_key();
        for i in 0..3i64 {
            let _ = handle_channel_post_with(
                &bus,
                json!(1),
                &post_params(&key, "t", "note", &[i as u8], i),
            )
            .await;
        }
        let first = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({"topic": "t", "cursor": 0, "limit": 2}),
        )
        .await;
        let v = unwrap_success(first);
        assert_eq!(v["messages"].as_array().unwrap().len(), 2);
        let next = v["next_cursor"].as_u64().unwrap();
        assert_eq!(next, 2);

        let second = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "t", "cursor": next}),
        )
        .await;
        let v = unwrap_success(second);
        assert_eq!(v["messages"].as_array().unwrap().len(), 1);
        assert_eq!(v["next_cursor"], 3);
    }

    #[tokio::test]
    async fn subscribe_unknown_topic_returns_typed_error() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({"topic": "nope"}),
        )
        .await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, error_code::CHANNEL_TOPIC_UNKNOWN);
    }

    #[tokio::test]
    async fn create_missing_name_is_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_create_with(&bus, json!(1), &json!({})).await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602);
    }

    // === T-1162: event.broadcast → channel:broadcast:global mirror ===

    #[tokio::test]
    async fn mirror_event_broadcast_lands_envelope_in_broadcast_global() {
        let (_d, bus) = tmp_bus();
        // Caller's responsibility in production is init_bus; replicate here.
        bus.create_topic(BROADCAST_GLOBAL_TOPIC, Retention::Messages(1000))
            .unwrap();

        mirror_event_broadcast_with(&bus, "deploy.start", &json!({"version": "1.0"})).await;

        // Read back via the same path channel.subscribe uses.
        let resp = handle_channel_subscribe_with(
            &bus,
            json!("sub-1"),
            &json!({ "topic": BROADCAST_GLOBAL_TOPIC }),
        )
        .await;
        let result = unwrap_success(resp);
        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1, "expected 1 mirrored envelope");
        let env = &messages[0];
        assert_eq!(env["topic"], BROADCAST_GLOBAL_TOPIC);
        assert_eq!(env["msg_type"], "deploy.start");
        assert_eq!(env["sender_id"], "hub:event.broadcast");
        // Payload round-trips as JSON bytes — decode base64 and parse.
        let b64 = env["payload_b64"].as_str().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let decoded: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, json!({"version": "1.0"}));
    }

    #[tokio::test]
    async fn mirror_event_broadcast_without_bus_is_noop() {
        // Calling the public entry point with no process-global bus set
        // must not panic or error — the shim is best-effort.
        mirror_event_broadcast("x.y", &json!({})).await;
    }

    // === T-1163: inbox.deposit → channel:inbox:<target> mirror ===

    #[tokio::test]
    async fn mirror_inbox_deposit_lands_envelope_in_target_topic() {
        let (_d, bus) = tmp_bus();
        // Note: no pre-create — the mirror helper creates the per-target topic
        // itself on first deposit (idempotent via Bus::create_topic).

        mirror_inbox_deposit_with(
            &bus,
            "test-target",
            "file.init",
            &json!({"transfer_id": "xfer-1", "filename": "doc.pdf", "size": 1024}),
            Some("sender-session"),
        )
        .await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!("sub-1"),
            &json!({ "topic": "inbox:test-target" }),
        )
        .await;
        let result = unwrap_success(resp);
        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1, "expected 1 mirrored envelope");
        let env = &messages[0];
        assert_eq!(env["topic"], "inbox:test-target");
        assert_eq!(env["msg_type"], "file.init");
        assert_eq!(env["sender_id"], "hub:inbox.deposit");
        let b64 = env["payload_b64"].as_str().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let decoded: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded["from"], "sender-session");
        assert_eq!(decoded["payload"]["transfer_id"], "xfer-1");
        assert_eq!(decoded["payload"]["filename"], "doc.pdf");
    }

    #[tokio::test]
    async fn mirror_inbox_deposit_without_bus_is_noop() {
        // Public entry with no process-global bus set must not panic.
        mirror_inbox_deposit("any-target", "file.init", &json!({}), None).await;
    }

    #[tokio::test]
    async fn mirror_inbox_deposit_per_target_isolation() {
        let (_d, bus) = tmp_bus();

        mirror_inbox_deposit_with(&bus, "alice", "file.init", &json!({"n": 1}), None).await;
        mirror_inbox_deposit_with(&bus, "bob", "file.init", &json!({"n": 2}), None).await;
        mirror_inbox_deposit_with(&bus, "alice", "file.chunk", &json!({"n": 3}), None).await;

        let resp_alice = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({ "topic": "inbox:alice", "limit": 100u64 }),
        )
        .await;
        let alice_msgs = unwrap_success(resp_alice)["messages"].as_array().unwrap().len();
        assert_eq!(alice_msgs, 2, "alice should see her 2 deposits only");

        let resp_bob = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({ "topic": "inbox:bob", "limit": 100u64 }),
        )
        .await;
        let bob_msgs = unwrap_success(resp_bob)["messages"].as_array().unwrap().len();
        assert_eq!(bob_msgs, 1, "bob should see his 1 deposit only");
    }

    #[tokio::test]
    async fn mirror_inbox_deposit_null_from_serializes_correctly() {
        let (_d, bus) = tmp_bus();

        mirror_inbox_deposit_with(&bus, "anon-target", "file.init", &json!({"x": 1}), None).await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({ "topic": "inbox:anon-target" }),
        )
        .await;
        let result = unwrap_success(resp);
        let env = &result["messages"].as_array().unwrap()[0];
        let b64 = env["payload_b64"].as_str().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let decoded: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(decoded["from"].is_null(), "from should serialize as null when None");
        assert_eq!(decoded["payload"]["x"], 1);
    }

    #[tokio::test]
    async fn mirror_handles_multiple_events_in_order() {
        let (_d, bus) = tmp_bus();
        bus.create_topic(BROADCAST_GLOBAL_TOPIC, Retention::Messages(1000))
            .unwrap();

        for i in 0..5i32 {
            mirror_event_broadcast_with(&bus, "tick", &json!({"n": i})).await;
        }

        let resp = handle_channel_subscribe_with(
            &bus,
            json!("sub-2"),
            &json!({ "topic": BROADCAST_GLOBAL_TOPIC, "limit": 100u64 }),
        )
        .await;
        let result = unwrap_success(resp);
        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 5);
        for (i, env) in messages.iter().enumerate() {
            let b64 = env["payload_b64"].as_str().unwrap();
            let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
            let decoded: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(decoded["n"], i as i32);
        }
    }

    #[tokio::test]
    async fn post_with_metadata_round_trips_through_subscribe() {
        // T-1287: post with conversation_id + event_type → subscribe (no
        // filter) returns metadata intact on the wire.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:meta", Retention::Forever).unwrap();
        let key = signing_key();
        let p = post_params_with_meta(
            &key,
            "inbox:meta",
            "note",
            b"hello",
            2_000,
            Some(json!({"conversation_id": "c-1", "event_type": "turn"})),
        );
        let post = handle_channel_post_with(&bus, json!(1), &p).await;
        let _ = unwrap_success(post);

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "inbox:meta", "cursor": 0}),
        )
        .await;
        let v = unwrap_success(resp);
        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        let meta = msgs[0]["metadata"].as_object().expect("metadata present");
        assert_eq!(meta.get("conversation_id").and_then(|x| x.as_str()), Some("c-1"));
        assert_eq!(meta.get("event_type").and_then(|x| x.as_str()), Some("turn"));
    }

    #[tokio::test]
    async fn subscribe_with_conversation_id_filters_and_advances_cursor() {
        // T-1287: post 3 messages with mixed conversation_ids. Subscribe
        // with conversation_id filter returns only matching messages, in
        // offset order. next_cursor advances PAST the skipped non-matching
        // record so the client doesn't redundantly re-scan it.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:filt", Retention::Forever).unwrap();
        let key = signing_key();

        let p1 = post_params_with_meta(
            &key, "inbox:filt", "note", b"a1", 3_001,
            Some(json!({"conversation_id": "c-A"})),
        );
        let _ = handle_channel_post_with(&bus, json!(1), &p1).await;
        let p2 = post_params_with_meta(
            &key, "inbox:filt", "note", b"b1", 3_002,
            Some(json!({"conversation_id": "c-B"})),
        );
        let _ = handle_channel_post_with(&bus, json!(2), &p2).await;
        let p3 = post_params_with_meta(
            &key, "inbox:filt", "note", b"a2", 3_003,
            Some(json!({"conversation_id": "c-A"})),
        );
        let _ = handle_channel_post_with(&bus, json!(3), &p3).await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(4),
            &json!({"topic": "inbox:filt", "cursor": 0, "conversation_id": "c-A"}),
        )
        .await;
        let v = unwrap_success(resp);
        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2, "only c-A messages should be surfaced");

        // In-order, payloads are a1 then a2.
        let bytes0 = base64::engine::general_purpose::STANDARD
            .decode(msgs[0]["payload_b64"].as_str().unwrap())
            .unwrap();
        let bytes1 = base64::engine::general_purpose::STANDARD
            .decode(msgs[1]["payload_b64"].as_str().unwrap())
            .unwrap();
        assert_eq!(&bytes0, b"a1");
        assert_eq!(&bytes1, b"a2");

        // next_cursor MUST advance past the skipped c-B record (offset 1)
        // so a follow-up subscribe doesn't re-examine it.
        let next = v["next_cursor"].as_u64().unwrap();
        let last_offset = msgs[1]["offset"].as_u64().unwrap();
        assert_eq!(next, last_offset + 1, "cursor advances over skipped records");
    }

    #[tokio::test]
    async fn subscribe_without_metadata_omits_field_on_wire() {
        // T-1287: posts without metadata produce envelopes whose JSON has
        // NO `metadata` key — preserves legacy wire format for callers
        // that don't use the new field.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:plain", Retention::Forever).unwrap();
        let key = signing_key();
        let p = post_params(&key, "inbox:plain", "note", b"x", 4_000);
        let _ = handle_channel_post_with(&bus, json!(1), &p).await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "inbox:plain", "cursor": 0}),
        )
        .await;
        let v = unwrap_success(resp);
        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        let env = msgs[0].as_object().unwrap();
        assert!(
            !env.contains_key("metadata"),
            "envelope without metadata must omit the key entirely"
        );
    }
}
