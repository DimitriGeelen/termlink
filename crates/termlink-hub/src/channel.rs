//! Hub-side handlers for the T-1160 `channel.*` RPC surface.
//!
//! The hub owns one [`termlink_bus::Bus`] keyed off `<runtime_dir>/bus/`
//! (initialised once at server start by [`init_bus`]). Every `channel.post`
//! arriving over JSON-RPC is verified against the sender's ed25519 public
//! key using the canonical byte layout defined in
//! `termlink_protocol::control::channel::canonical_sign_bytes` before the
//! envelope is appended to the bus. All four verbs are Tier-A per T-1133:
//! payloads are opaque `base64(String)` on the wire.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::{json, Value};

use termlink_bus::{ArtifactStore, Bus, Envelope, Retention};
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_protocol::control::error_code;
use termlink_protocol::jsonrpc::{ErrorResponse, Response, RpcResponse};
use termlink_session::agent_identity::fingerprint_of;

static BUS: OnceLock<Bus> = OnceLock::new();
static ARTIFACT_STORE: OnceLock<ArtifactStore> = OnceLock::new();

/// T-1286 / T-243 — passive multi-turn dialog presence tracker. Maps
/// `(conversation_id, agent_id) → last_seen_unix_ms`. Updated by
/// `handle_channel_post_with` whenever a successful post carries
/// `metadata.conversation_id`; queried via `dialog.presence`.
///
/// Process-global so that the RPC dispatch path and the test path observe
/// the same tracker; tests that need isolation use `presence_tracker_for_tests`
/// to inject their own.
pub(crate) struct PresenceTracker {
    inner: RwLock<HashMap<(String, String), i64>>,
}

impl PresenceTracker {
    pub(crate) fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn record(&self, cid: &str, agent_id: &str, last_seen_ms: i64) {
        if let Ok(mut g) = self.inner.write() {
            g.insert((cid.to_string(), agent_id.to_string()), last_seen_ms);
        }
    }

    pub(crate) fn snapshot(&self, cid: &str) -> Vec<(String, i64)> {
        let Ok(g) = self.inner.read() else {
            return Vec::new();
        };
        let mut out: Vec<(String, i64)> = g
            .iter()
            .filter_map(|((c, a), ts)| if c == cid { Some((a.clone(), *ts)) } else { None })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

static PRESENCE: OnceLock<PresenceTracker> = OnceLock::new();

pub(crate) fn presence() -> &'static PresenceTracker {
    PRESENCE.get_or_init(PresenceTracker::new)
}

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
    match bus.post(&topic_name, &env).await {
        Err(e) => tracing::warn!(topic = %topic_name, error = %e, "T-1163 mirror: bus.post failed"),
        Ok(offset) => if let Some(agg) = crate::router::aggregator() {
            agg.inject(crate::aggregator::AggregatedEvent {
                session_id: "hub".to_string(), session_name: "hub".to_string(), seq: 0,
                topic: termlink_protocol::events::inbox_topic::QUEUED.to_string(),
                payload: serde_json::json!({"schema_version": termlink_protocol::events::SCHEMA_VERSION,
                    "addressee_session_id": target, "channel": &topic_name,
                    "message_offset": offset, "enqueued_at": env.ts_unix_ms}),
                timestamp: env.ts_unix_ms.max(0) as u64,
            });
        }
    }
}

/// T-1300: Topic the soft-lint engine dual-writes warnings to. Subscribers
/// (Watchtower, ad-hoc operator scripts) read from this topic to surface
/// routing violations without disturbing the live emit path.
pub const ROUTING_LINT_TOPIC: &str = "routing:lint";

/// T-1300: Mirror a `routing.lint.warning` envelope into the
/// `routing:lint` channel topic. Best-effort — never blocks the caller; the
/// lint itself is soft (the originating emit always succeeds regardless).
/// `msg_type` is set to the emit method ("event.broadcast" / "event.emit_to")
/// so consumers can filter by call site.
pub async fn mirror_routing_lint_warning(method: &str, payload: &Value) {
    let Some(bus) = bus() else {
        tracing::debug!("T-1300 mirror: bus not initialised — skipping");
        return;
    };
    if let Err(e) = bus.create_topic(ROUTING_LINT_TOPIC, Retention::Messages(1000)) {
        tracing::warn!(
            topic = ROUTING_LINT_TOPIC,
            error = %e,
            "T-1300 mirror: create_topic failed — will still try bus.post"
        );
    }
    let payload_bytes = match serde_json::to_vec(payload) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "T-1300 mirror: failed to serialize payload");
            return;
        }
    };
    let ts_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let env = Envelope {
        topic: ROUTING_LINT_TOPIC.to_string(),
        sender_id: "hub:topic_lint".to_string(),
        msg_type: method.to_string(),
        payload: payload_bytes,
        artifact_ref: None,
        ts_unix_ms,
        metadata: Default::default(),
    };
    if let Err(e) = bus.post(ROUTING_LINT_TOPIC, &env).await {
        tracing::warn!(error = %e, "T-1300 mirror: bus.post failed");
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
        "latest" => Some(Retention::Latest),
        "latest_per_cv_key" => Some(Retention::LatestPerCvKey),
        _ => None,
    }
}

fn retention_to_json(r: Retention) -> Value {
    match r {
        Retention::Forever => json!({"kind": "forever"}),
        Retention::Days(d) => json!({"kind": "days", "value": d}),
        Retention::Messages(n) => json!({"kind": "messages", "value": n}),
        Retention::Latest => json!({"kind": "latest"}),
        Retention::LatestPerCvKey => json!({"kind": "latest_per_cv_key"}),
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

/// T-2058: topic-name patterns that have demonstrated high envelope rates
/// in the running fleet (per T-2057 audit). Combining one of these with
/// `Retention::Forever` is the T-1991/G-058 silent-growth vector — the
/// hub emits a warn log on create so the operator-default path is loud.
/// The matcher is intentionally tight (specific names + prefixes, NOT
/// broad `agent-*`) to avoid noise on legitimate operator-named topics.
pub(crate) fn is_high_rate_pattern(name: &str) -> bool {
    matches!(name, "agent-presence" | "agent-chat-arc")
        || name.starts_with("agent-listeners-")
        || name.starts_with("agent-conv-")
        || name.starts_with("dm:")
}

/// T-2145: topic-name patterns where the topic name IS the key and only
/// the freshest envelope matters (single-value durable state). For these,
/// `Retention::Latest` (T-2142) is the right answer — old envelopes are
/// pure history noise. Combining `state:*` with `Retention::Forever` is
/// the same silent-growth vector as the high-rate case, just slower; the
/// hub emits a warn log on create so the operator-default path is loud.
/// Sibling of `is_high_rate_pattern` — the two predicates partition the
/// "warn on operator-default Retention::Forever" space (disjoint by
/// prefix, no overlap).
pub(crate) fn is_single_value_state_pattern(name: &str) -> bool {
    name.starts_with("state:")
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
    // T-2058: loud-not-silent warn at create time for the known-high-rate
    // operator-default vector. Topic still created with requested
    // retention — this is informational, not a refusal.
    if matches!(retention, Retention::Forever) && is_high_rate_pattern(name) {
        tracing::warn!(
            topic = %name,
            retention = "forever",
            "channel.create on high-rate topic with Retention::Forever — for per-agent state like agent-presence prefer LatestPerCvKey (compacts to one record per cv_key — closes T-1991 count-scaling, T-2245); for stream topics use Messages(N). Prevents silent topic growth (T-1991 / G-058)"
        );
    }
    // T-2145: sibling warn for the single-value-state pattern (`state:*`)
    // where Retention::Latest (T-2142) is the right answer. Same loud-but-
    // not-refuse policy as the high-rate case.
    if matches!(retention, Retention::Forever) && is_single_value_state_pattern(name) {
        tracing::warn!(
            topic = %name,
            retention = "forever",
            "channel.create on single-value-state topic with Retention::Forever — consider Retention::Latest (T-2142) so old envelopes don't accumulate"
        );
    }
    match bus.create_topic(name, retention) {
        Ok(created) => Response::success(
            id,
            json!({
                "ok": true,
                "name": name,
                "retention": retention_to_json(retention),
                // T-1429.5: true if this call inserted the topic, false if
                // it already existed. Lets clients describe-on-first-create
                // without re-emitting topic_metadata envelopes on every
                // idempotent re-call. Old clients ignore the field.
                "created": created,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.create: {e}")).into(),
    }
}

/// `channel.set_retention(name, retention)` — change the retention policy of
/// an EXISTING topic (T-2244 / R2a). Unlike `channel.create` (which refuses a
/// policy change on idempotent re-create), this is the explicit opt-in.
/// Storage-only: does NOT sweep — the operator runs a sweep separately to
/// enforce the new policy against the backlog. Unknown topic returns an error
/// rather than stealth-creating it.
pub async fn handle_channel_set_retention(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_set_retention_with(bus, id, params).await
}

pub(crate) async fn handle_channel_set_retention_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let name = match param_str(params, "name") {
        Some(n) if !n.is_empty() => n,
        _ => return ErrorResponse::new(id, -32602, "Missing 'name' in params").into(),
    };
    // Retention is REQUIRED here (the whole point is to change it) — unlike
    // create, there is no Forever default.
    let retention = match params.get("retention").and_then(retention_from_json) {
        Some(r) => r,
        None => {
            return ErrorResponse::new(id, -32602, "Missing or invalid 'retention' in params")
                .into()
        }
    };
    match bus.set_topic_retention(name, retention) {
        Ok(true) => Response::success(
            id,
            json!({
                "ok": true,
                "name": name,
                "retention": retention_to_json(retention),
                "updated": true,
            }),
        )
        .into(),
        // Unknown topic — loud, not a stealth create (AC: clear error).
        Ok(false) => ErrorResponse::new(
            id,
            -32602,
            &format!("channel.set_retention: unknown topic '{name}' (use channel.create first)"),
        )
        .into(),
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("channel.set_retention: {e}")).into()
        }
    }
}

/// `channel.sweep(topic)` — enforce the topic's retention policy NOW,
/// pruning records outside it. The explicit trigger for the retention
/// subsystem (T-2245 / R2b): create/set_retention only persist a policy;
/// nothing prunes until this runs (no background sweep thread, T-1155).
pub async fn handle_channel_sweep(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_sweep_with(bus, id, params).await
}

pub(crate) async fn handle_channel_sweep_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    match bus.sweep(topic, now) {
        Ok(pruned) => Response::success(
            id,
            json!({ "ok": true, "topic": topic, "pruned": pruned }),
        )
        .into(),
        // Unknown topic is an explicit error (no stealth create), matching
        // set_retention's contract.
        Err(termlink_bus::BusError::UnknownTopic(_)) => ErrorResponse::new(
            id,
            -32602,
            &format!("channel.sweep: unknown topic '{topic}' (use channel.create first)"),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.sweep: {e}")).into(),
    }
}

/// T-2297: enforce the hub-attestation invariant on the `observed_addr` metadata key.
///
/// The hub-observed TCP source address (`peer_addr`) is authoritative:
///   - `Some(addr)` (a TCP connection) → OVERWRITE any client-supplied `observed_addr`
///     with the attested value, so a client cannot forge where it connected from.
///   - `None` (Unix-socket / local caller, not attestable) → STRIP any client-supplied
///     `observed_addr` so an un-attested value can never masquerade as attested.
///
/// Net invariant: after this runs, `metadata["observed_addr"]` is present ONLY when it
/// is the hub's own observation. Metadata is not part of the signed canonical bytes
/// (see `canonical_sign_bytes`), so this mutation never affects signature verification.
pub(crate) fn apply_observed_addr(
    metadata: &mut std::collections::BTreeMap<String, String>,
    peer_addr: Option<&str>,
) {
    match peer_addr {
        Some(addr) => {
            metadata.insert("observed_addr".to_string(), addr.to_string());
        }
        None => {
            metadata.remove("observed_addr");
        }
    }
}

/// `channel.post(topic, msg_type, payload_b64, artifact_ref?, ts, sender_id,
/// sender_pubkey_hex, signature_hex)` — verifies signature then appends.
pub async fn handle_channel_post(
    id: Value,
    params: &Value,
    peer_addr: Option<&str>,
) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_post_with_peer(bus, id, params, peer_addr).await
}

/// T-2297: back-compat wrapper for callers with no hub-attested peer address
/// (the test suite). Delegates with `peer_addr = None`, which STRIPS any
/// client-supplied `observed_addr` (never client-forgeable). Production posts
/// route through `handle_channel_post_with_peer`, so this is `#[cfg(test)]`.
#[cfg(test)]
pub(crate) async fn handle_channel_post_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    handle_channel_post_with_peer(bus, id, params, None).await
}

pub(crate) async fn handle_channel_post_with_peer(
    bus: &Bus,
    id: Value,
    params: &Value,
    peer_addr: Option<&str>,
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

    // T-1427: identity authoritative — the claimed `sender_id` MUST match the
    // fingerprint derived from `sender_pubkey_hex`. Closes T-1425 RFC §3.2
    // invariant 2; without this check a client could legally sign with its
    // own key but claim any sender_id, misattributing envelopes.
    let expected_fp = fingerprint_of(&verifying_key);
    if sender_id != expected_fp {
        return ErrorResponse::new(
            id,
            error_code::CHANNEL_IDENTITY_MISMATCH,
            &format!(
                "sender_id={sender_id:?} does not match identity fingerprint {prefix}… derived from sender_pubkey_hex (T-1427)",
                prefix = &expected_fp[..8.min(expected_fp.len())]
            ),
        )
        .into();
    }

    // T-1287: optional metadata routing-hint map. NOT included in canonical
    // signed bytes — trusted-mesh threat model treats it as routing only.
    // Well-known keys: conversation_id, event_type (per T-1288 catalog).
    let mut metadata: std::collections::BTreeMap<String, String> = params
        .get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    // T-2297: stamp the hub-attested observed source address onto the envelope
    // metadata (or strip any client-supplied value when not attestable). MUST run
    // after the client metadata is parsed so the attested value wins.
    apply_observed_addr(&mut metadata, peer_addr);

    // T-2049 Gap A — idempotency via optional client_msg_id. Verified
    // sender_id (T-1427) namespaces the dedupe so the cache cannot be
    // poisoned across senders. Length-bounded to 1..=128 chars to keep
    // the cache key small; longer payloads should hash before submission.
    let client_msg_id = params
        .get("client_msg_id")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() <= 128);
    if let Some(ref cid) = client_msg_id {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        match crate::dedupe::post_dedupe().try_record_or_lookup(&sender_id, cid, now) {
            crate::dedupe::DedupeOutcome::Duplicate { offset, ts_unix_ms } => {
                // Silent no-op — return the cached envelope so the
                // retrying spoke sees success and stops retrying.
                return Response::success(
                    id,
                    json!({"offset": offset, "ts": ts_unix_ms, "deduped": true}),
                )
                .into();
            }
            crate::dedupe::DedupeOutcome::Newly => { /* fall through to post */ }
        }
    }

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
        Ok(offset) => {
            // T-2049 Gap A — record successful offset so retries of this
            // (sender_id, client_msg_id) within the TTL hit the cache
            // path above and return the same offset without re-appending.
            if let Some(ref cid) = client_msg_id {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                crate::dedupe::post_dedupe().record_offset(
                    &env.sender_id,
                    cid,
                    now,
                    offset,
                    env.ts_unix_ms,
                );
            }
            // T-2027/T-2089 slice 1 — broadcast-with-replay cv_index.
            // When the envelope opts into the current-value pattern by
            // setting `metadata.cv_key`, record (topic, cv_key) -> offset
            // with last-write-wins so future
            // `channel.subscribe --include-current-value` (slice 2)
            // can read it in O(K). Cap-overflow refusals are silent at
            // the post layer (post stays atomic); observability comes
            // via `cv_index::overflow_total`.
            if let Some(cv_key) = env.metadata.get("cv_key") {
                let cv_key = cv_key.trim();
                if !cv_key.is_empty() && cv_key.len() <= 256 {
                    let _ = crate::cv_index::record(&topic, cv_key, offset);
                }
            }
            // T-1286 / T-243: passive presence tracking. When the envelope
            // carries metadata.conversation_id, record (cid, sender_id) →
            // ts so dialog.presence can answer "who's active here?"
            if let Some(cid) = env.metadata.get("conversation_id") {
                presence().record(cid, &env.sender_id, env.ts_unix_ms);
            }
            // T-1637: emit inbox.queued for channel.post → inbox:<id> topics so
            // the wakeup contract from T-1636 covers the channel.post delivery
            // path too (post-T-1166 the legacy mirror_inbox_deposit emit site
            // retires; channel.post becomes the only inbox delivery RPC).
            if let Some(addressee) = topic.strip_prefix("inbox:")
                && let Some(agg) = crate::router::aggregator() {
                agg.inject(crate::aggregator::AggregatedEvent {
                    session_id: "hub".to_string(),
                    session_name: "hub".to_string(),
                    seq: 0,
                    topic: termlink_protocol::events::inbox_topic::QUEUED.to_string(),
                    payload: serde_json::json!({
                        "schema_version": termlink_protocol::events::SCHEMA_VERSION,
                        "addressee_session_id": addressee,
                        "channel": &topic,
                        "message_offset": offset,
                        "enqueued_at": env.ts_unix_ms,
                    }),
                    timestamp: env.ts_unix_ms.max(0) as u64,
                });
            }
            // T-2323 (arc-004 S1): sibling emit for the dm rail. A canonical
            // `dm:<a>:<b>` topic carries both participant fingerprints; the
            // addressee is the participant that is NOT the sender. This lets the
            // push-waker ring the receiver when the poster does not itself ring
            // (raw `channel post` / cron / remote peer / MCP `channel_post`). Mirror
            // of the inbox emit above — same placement after the Ok arm so a failed
            // post (unknown topic / error) never emits.
            if let Some(rest) = topic.strip_prefix("dm:")
                && let Some((a, b)) = rest.split_once(':')
                && let Some(agg) = crate::router::aggregator() {
                // Wake the participant who is not the sender. If the two halves
                // are identical (degenerate self-dm) or sender_id matches neither
                // half (relay / third-party post), emit nothing — waking a wrong
                // or self party is worse than falling back to poll.
                let addressee = if a != b && env.sender_id == a {
                    Some(b)
                } else if a != b && env.sender_id == b {
                    Some(a)
                } else {
                    None
                };
                if let Some(addressee) = addressee {
                    agg.inject(crate::aggregator::AggregatedEvent {
                        session_id: "hub".to_string(),
                        session_name: "hub".to_string(),
                        seq: 0,
                        topic: termlink_protocol::events::dm_topic::QUEUED.to_string(),
                        payload: serde_json::json!({
                            "schema_version": termlink_protocol::events::SCHEMA_VERSION,
                            "addressee_session_id": addressee,
                            "channel": &topic,
                            "message_offset": offset,
                            "enqueued_at": env.ts_unix_ms,
                        }),
                        timestamp: env.ts_unix_ms.max(0) as u64,
                    });
                }
            }
            // T-2333 (arc-004 webhook fan-out, Slice 2): fan the successful post
            // out to any configured external webhook target subscribed to this
            // topic. Sibling of the inbox/dm emits above — placed in the Ok arm
            // so a failed post never fans out. fan_out is fire-and-forget and a
            // no-op when the subsystem is disabled (opt-in / no hard dependency),
            // so this is a cheap early-return for the common no-webhook hub.
            if crate::webhook::webhooks().is_some() {
                let body = serde_json::json!({
                    "topic": &topic,
                    "offset": offset,
                    "ts": env.ts_unix_ms,
                    "sender_id": &env.sender_id,
                    "msg_type": &env.msg_type,
                    "event_type": env.metadata.get("event_type"),
                });
                if let Ok(bytes) = serde_json::to_vec(&body) {
                    crate::webhook::fan_out(&topic, bytes);
                }
            }
            Response::success(id, json!({"offset": offset, "ts": ts_unix_ms})).into()
        }
        Err(termlink_bus::BusError::UnknownTopic(t)) => ErrorResponse::new(
            id,
            error_code::CHANNEL_TOPIC_UNKNOWN,
            &format!("unknown topic: {t}"),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.post: {e}")).into(),
    }
}

/// T-2355: server-side record-walk deadline. `spawn_blocking` (T-2258)
/// keeps the reactor alive under concurrent walks, but a slow or wedged
/// walk still holds its blocking-pool thread indefinitely — under K such
/// walks the pool saturates and reads wedge fleet-wide while O(1) posts
/// stay fast (the .122 field symptom). Each walk is therefore bounded by
/// `TERMLINK_WALK_DEADLINE_MS` (default 20_000, clamped 100..=600_000 —
/// deliberately UNDER the T-2354 client read-timeout default of 30s so
/// clients receive the server's structured `WALK_DEADLINE_EXCEEDED`
/// error, not an opaque client-side timeout).
fn walk_deadline_from_env() -> std::time::Duration {
    let ms = std::env::var("TERMLINK_WALK_DEADLINE_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(100, 600_000))
        .unwrap_or(20_000);
    std::time::Duration::from_millis(ms)
}

/// Outcome of a deadline-bounded `channel.subscribe` record walk (T-2355).
struct SubscribeWalk {
    messages: Vec<Value>,
    last_offset: Option<u64>,
    error_msg: Option<String>,
    deadline_hit: bool,
    records_scanned: u64,
}

/// The `channel.subscribe` record walk, factored out of the
/// `spawn_blocking` closure so tests can drive it with an explicit
/// `deadline`. The deadline is checked at the top of each iteration —
/// BEFORE the fetched record is processed — so `last_offset` only ever
/// covers fully-processed records and the caller-computed resume cursor
/// (`last_offset + 1`) never skips one.
fn walk_subscribe_records(
    iter: termlink_bus::SubscribeIter,
    conversation_id_filter: Option<String>,
    in_reply_to_filter: Option<String>,
    limit: usize,
    deadline: std::time::Duration,
) -> SubscribeWalk {
    let started = std::time::Instant::now();
    let mut messages: Vec<Value> = Vec::new();
    let mut last_offset: Option<u64> = None;
    let mut error_msg: Option<String> = None;
    let mut deadline_hit = false;
    let mut records_scanned: u64 = 0;
    for item in iter {
        if started.elapsed() >= deadline {
            deadline_hit = true;
            break;
        }
        let (offset, env) = match item {
            Ok(x) => x,
            Err(e) => {
                error_msg = Some(format!("channel.subscribe decode: {e}"));
                break;
            }
        };
        records_scanned += 1;
        last_offset = Some(offset);
        if let Some(ref cid) = conversation_id_filter
            && env.metadata.get("conversation_id").map(|s| s.as_str()) != Some(cid.as_str())
        {
            continue;
        }
        if let Some(ref parent) = in_reply_to_filter
            && env.metadata.get("in_reply_to").map(|s| s.as_str()) != Some(parent.as_str())
        {
            continue;
        }
        messages.push(envelope_to_json(offset, &env));
        if messages.len() >= limit {
            break;
        }
    }
    SubscribeWalk {
        messages,
        last_offset,
        error_msg,
        deadline_hit,
        records_scanned,
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
    // T-1313: optional in_reply_to filter (Matrix m.in_reply_to analogue).
    // Symmetric to conversation_id_filter — string equality on
    // metadata.in_reply_to (parent envelope's offset, decimal string).
    // Same advance-over-skipped-records semantics.
    let in_reply_to_filter = params
        .get("in_reply_to")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // T-2027/T-2089 slice 2: optional broadcast-with-replay current-value
    // surface. When `include_current_value=true`, the response gains a
    // `current_values: [{cv_key, offset, msg}, ...]` array carrying the
    // hub-side cv_index snapshot for this topic. Backward compatible —
    // when false/absent, the response shape is byte-identical to pre-slice-2.
    let include_current_value = params
        .get("include_current_value")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

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

    // T-2013/T-2258: `iter.next()` calls `File::seek` + `File::read_exact`
    // per record — blocking syscalls. T-2013 wrapped the walk in
    // `block_in_place`, but that only converts the CURRENT worker into a
    // blocking thread: under K concurrent large-topic walks (K > worker
    // threads) the bounded worker pool — and the I/O reactor that reads
    // RPC lines / writes responses — still starves, so reads hang
    // indefinitely while `channel.post` (O(1), never blocks) stays fast.
    // That is the T-2258 field symptom (ring20: `channel state` of a
    // ~1639-row topic killed at 12s under concurrent post; `channel state`
    // pages `channel.subscribe`, so this walk is the one it hits). FIX:
    // run the walk on tokio's dedicated blocking pool via `spawn_blocking`,
    // which never consumes a worker thread, so the reactor and other RPCs
    // keep progressing regardless of how many walks are in flight. The
    // owned `ReaderIter` + owned filters are moved into the closure.
    // T-2355 bounds each walk with a server-side deadline (see
    // `walk_deadline_from_env`) — a wedged walk now LOUD-refuses with
    // `WALK_DEADLINE_EXCEEDED` + resumable `data.next_cursor` instead of
    // holding its blocking-pool thread forever.
    let deadline = walk_deadline_from_env();
    let walk = match tokio::task::spawn_blocking(move || {
        walk_subscribe_records(
            iter,
            conversation_id_filter,
            in_reply_to_filter,
            limit,
            deadline,
        )
    })
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return ErrorResponse::internal_error(
                id,
                &format!("channel.subscribe walk task failed: {e}"),
            )
            .into();
        }
    };
    if let Some(msg) = walk.error_msg {
        return ErrorResponse::internal_error(id, &msg).into();
    }
    let next_cursor = walk.last_offset.map(|o| o + 1).unwrap_or(cursor);
    if walk.deadline_hit {
        return ErrorResponse::with_data(
            id,
            error_code::WALK_DEADLINE_EXCEEDED,
            &format!(
                "channel.subscribe walk deadline exceeded after {}ms on '{topic}' ({} records scanned) — resume from data.next_cursor or raise TERMLINK_WALK_DEADLINE_MS",
                deadline.as_millis(),
                walk.records_scanned
            ),
            json!({
                "deadline_ms": deadline.as_millis() as u64,
                "records_scanned": walk.records_scanned,
                "next_cursor": next_cursor,
            }),
        )
        .into();
    }
    let messages = walk.messages;

    // T-2027/T-2089 slice 2 — assemble the current-value prefix from the
    // cv_index. Each entry is a single O(1) seek-and-read via
    // `bus.subscribe(topic, offset).next()`. Total cost: O(K) where K is
    // the topic's distinct cv_keys. Stale entries (offset references an
    // envelope past retention horizon) are silently skipped — slice 2
    // does not lazy-reconcile. Wrapped in block_in_place per the T-2013
    // invariant (synchronous bus iter walks must not pin tokio workers).
    let current_values_json: Option<Vec<Value>> = if include_current_value {
        let cv_entries = crate::cv_index::current_values(&topic);
        let bus_for_cv = bus;
        let topic_for_cv = topic.clone();
        let cvs = tokio::task::block_in_place(move || {
            let mut out: Vec<Value> = Vec::with_capacity(cv_entries.len());
            for (cv_key, offset) in cv_entries {
                let iter = match bus_for_cv.subscribe(&topic_for_cv, offset) {
                    Ok(i) => i,
                    Err(_) => continue, // topic vanished mid-call; skip
                };
                let mut iter = iter;
                match iter.next() {
                    Some(Ok((env_offset, env))) if env_offset == offset => {
                        out.push(json!({
                            "cv_key": cv_key,
                            "offset": offset,
                            "msg": envelope_to_json(offset, &env),
                        }));
                    }
                    // Stale index entry (offset past retention or moved):
                    // skip silently. Future slice could repair the index
                    // here, but slice 2 prefers a clean read path.
                    _ => continue,
                }
            }
            out
        });
        Some(cvs)
    } else {
        None
    };

    let mut body = json!({"messages": messages, "next_cursor": next_cursor});
    if let Some(cvs) = current_values_json {
        body.as_object_mut()
            .expect("subscribe response body is an object")
            .insert("current_values".to_string(), Value::Array(cvs));
    }
    Response::success(id, body).into()
}

/// Latest receipt seen for one sender during a `channel.receipts` walk
/// (T-1329; hoisted to module scope by T-2355 so the factored walk fn
/// can name it).
struct ReceiptEntry {
    up_to: u64,
    ts: i64,
}

/// Outcome of a deadline-bounded `channel.receipts` record walk (T-2355).
struct ReceiptsWalk {
    latest: HashMap<String, ReceiptEntry>,
    error_msg: Option<String>,
    deadline_hit: bool,
    records_scanned: u64,
}

/// The `channel.receipts` record walk, factored out of the
/// `spawn_blocking` closure so tests can drive it with an explicit
/// `deadline`. Unlike the subscribe walk, a deadline hit here yields NO
/// usable partial result — the receipt map is a whole-topic aggregate and
/// a partial map would silently under-report `up_to` marks — so the
/// handler LOUD-refuses without a resume cursor.
fn walk_receipt_records(
    iter: termlink_bus::SubscribeIter,
    deadline: std::time::Duration,
) -> ReceiptsWalk {
    let started = std::time::Instant::now();
    let mut latest: HashMap<String, ReceiptEntry> = HashMap::new();
    let mut error_msg: Option<String> = None;
    let mut deadline_hit = false;
    let mut records_scanned: u64 = 0;
    for item in iter {
        if started.elapsed() >= deadline {
            deadline_hit = true;
            break;
        }
        let (_offset, env) = match item {
            Ok(x) => x,
            Err(e) => {
                error_msg = Some(format!("channel.receipts decode: {e}"));
                break;
            }
        };
        records_scanned += 1;
        if env.msg_type != "receipt" {
            continue;
        }
        let Some(up_to_str) = env.metadata.get("up_to") else {
            continue;
        };
        let Ok(up_to) = up_to_str.parse::<u64>() else {
            continue;
        };
        let ts = env.ts_unix_ms;
        let sender = env.sender_id.clone();
        match latest.get(&sender) {
            Some(prev) if prev.ts > ts => {}
            Some(prev) if prev.ts == ts && prev.up_to >= up_to => {}
            _ => {
                latest.insert(sender, ReceiptEntry { up_to, ts });
            }
        }
    }
    ReceiptsWalk {
        latest,
        error_msg,
        deadline_hit,
        records_scanned,
    }
}

/// `channel.receipts(topic)` → `{topic, receipts: [{sender_id, up_to, ts_unix_ms}, ...]}`.
/// T-1329. Server-side aggregation of `m.receipt` envelopes — walks the topic
/// once on the hub, keeps only the latest receipt per sender (latest-wins by
/// `ts_unix_ms`; ties broken by higher `up_to`), returns a sorted-by-sender list.
/// Mirrors the read-side walker the CLI used in T-1315 so output is identical.
pub async fn handle_channel_receipts(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_receipts_with(bus, id, params).await
}

pub(crate) async fn handle_channel_receipts_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let iter = match bus.subscribe(&topic, 0) {
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
            return ErrorResponse::internal_error(id, &format!("channel.receipts: {e}")).into();
        }
    };
    // T-2013/T-2258: synchronous bus.subscribe iter walk — same root
    // cause as handle_channel_subscribe_with above. Run it on the
    // dedicated blocking pool via spawn_blocking (NOT block_in_place,
    // which pins a worker and starves the reactor under concurrent
    // large-topic reads — see the T-2258 note on the subscribe walk).
    // T-2355 bounds the walk with the same server-side deadline as the
    // subscribe walk — LOUD-refuse over holding a blocking thread forever.
    let deadline = walk_deadline_from_env();
    let walk = match tokio::task::spawn_blocking(move || walk_receipt_records(iter, deadline))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return ErrorResponse::internal_error(
                id,
                &format!("channel.receipts walk task failed: {e}"),
            )
            .into();
        }
    };
    if let Some(msg) = walk.error_msg {
        return ErrorResponse::internal_error(id, &msg).into();
    }
    if walk.deadline_hit {
        return ErrorResponse::with_data(
            id,
            error_code::WALK_DEADLINE_EXCEEDED,
            &format!(
                "channel.receipts walk deadline exceeded after {}ms on '{topic}' ({} records scanned) — receipts aggregate the whole topic, so no partial result is offered; raise TERMLINK_WALK_DEADLINE_MS or sweep the topic's retention",
                deadline.as_millis(),
                walk.records_scanned
            ),
            json!({
                "deadline_ms": deadline.as_millis() as u64,
                "records_scanned": walk.records_scanned,
            }),
        )
        .into();
    }
    let mut entries: Vec<(String, ReceiptEntry)> = walk.latest.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let arr: Vec<Value> = entries
        .iter()
        .map(|(s, r)| json!({"sender_id": s, "up_to": r.up_to, "ts_unix_ms": r.ts}))
        .collect();
    Response::success(id, json!({"topic": topic, "receipts": arr})).into()
}

/// `dialog.presence(conversation_id)` → `{presences: [{agent_id, last_seen_ms}, ...]}`.
/// T-1286 / T-243. Hub passively tracks senders per conversation_id by
/// observing channel.post metadata. Unknown conversation_id returns an
/// empty list (presence is observational — absence ≠ error).
pub async fn handle_dialog_presence(id: Value, params: &Value) -> RpcResponse {
    let cid = match param_str(params, "conversation_id") {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return ErrorResponse::new(id, -32602, "Missing 'conversation_id' in params").into()
        }
    };
    let snapshot = presence().snapshot(&cid);
    let presences: Vec<Value> = snapshot
        .into_iter()
        .map(|(agent_id, last_seen_ms)| {
            json!({"agent_id": agent_id, "last_seen_ms": last_seen_ms})
        })
        .collect();
    Response::success(id, json!({"presences": presences})).into()
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

/// T-2421: `channel.delete(topic)` → `{ok, deleted_records, topic}`.
/// Destructive: removes the topic ENTIRELY — registry entry, records,
/// cursors, offset counter, claims, on-disk log, and cv_index entries.
/// Exact-name only: wildcards are refused (destructive verbs never glob).
/// Unknown topic is a loud error — no stealth success. Execute scope.
/// Contrast `channel.trim`, which empties a topic but leaves it registered.
pub async fn handle_channel_delete(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_delete_with(bus, id, params).await
}

pub(crate) async fn handle_channel_delete_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    if topic.contains('*') || topic.contains('?') {
        return ErrorResponse::new(
            id,
            -32602,
            &format!(
                "channel.delete: wildcards not allowed — exact-name only (got '{topic}'). \
                 Enumerate with channel.list and delete each name explicitly."
            ),
        )
        .into();
    }
    match bus.delete_topic(topic) {
        Ok(Some(deleted_records)) => {
            let cv_removed = crate::cv_index::remove_topic(topic);
            tracing::info!(
                topic,
                deleted_records,
                cv_entries_removed = cv_removed,
                "channel.delete: topic removed"
            );
            Response::success(
                id,
                json!({"ok": true, "deleted_records": deleted_records, "topic": topic}),
            )
            .into()
        }
        Ok(None) => ErrorResponse::new(
            id,
            -32602,
            &format!("channel.delete: unknown topic '{topic}' (nothing deleted)"),
        )
        .into(),
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("channel.delete: {e}")).into()
        }
    }
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

/// T-2029 (arc-parallel-substrate Slice 1) — `channel.claim(topic, offset, claimer, ttl_ms?)`.
/// Issues an exclusive lease over `(topic, offset)` for `claimer`. Default TTL is 30s; the
/// client must call `channel.release` (or the future `channel.renew`, Slice 2) before
/// `claimed_until` to retain the claim.
pub async fn handle_channel_claim(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_claim_with(bus, id, params).await
}

pub(crate) async fn handle_channel_claim_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let offset = match params.get("offset").and_then(|v| v.as_u64()) {
        Some(o) => o,
        None => return ErrorResponse::new(id, -32602, "Missing or invalid 'offset' in params").into(),
    };
    let claimer = match param_str(params, "claimer") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claimer' in params").into(),
    };
    // Default TTL: 30s. Clamp upper bound to 1 hour to avoid forever-stuck claims
    // from a bug or hostile client.
    let ttl_ms = params
        .get("ttl_ms")
        .and_then(|v| v.as_u64())
        .map(|t| t.min(60 * 60 * 1000) as u32)
        .unwrap_or(30_000);
    match bus.claim_offset(topic, offset, claimer, ttl_ms) {
        Ok(info) => Response::success(
            id,
            json!({
                "ok": true,
                "claim_id": info.claim_id,
                "topic": info.topic,
                "offset": info.offset,
                "claimer": info.claimer,
                "claimed_at": info.claimed_at,
                "claimed_until": info.claimed_until,
            }),
        )
        .into(),
        Err(termlink_bus::BusError::UnknownTopic(_)) => ErrorResponse::new(
            id,
            error_code::CHANNEL_TOPIC_UNKNOWN,
            &format!("channel.claim: topic {topic:?} not found"),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimConflict {
            topic: t,
            offset: o,
        }) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_CONFLICT,
            &format!("channel.claim: offset {o} of topic {t:?} is already claimed"),
            json!({"topic": t, "offset": o}),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.claim: {e}")).into(),
    }
}

/// T-2029 — `channel.release(claim_id, claimer, ack)`. Releases a previously-issued
/// claim. `ack=true` advances the claimer's cursor past the offset (work completed);
/// `ack=false` frees the slot for another worker (work returned).
pub async fn handle_channel_release(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_release_with(bus, id, params).await
}

pub(crate) async fn handle_channel_release_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let claim_id = match param_str(params, "claim_id") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claim_id' in params").into(),
    };
    let claimer = match param_str(params, "claimer") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claimer' in params").into(),
    };
    let ack = params
        .get("ack")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    match bus.release_claim(claim_id, claimer, ack) {
        Ok(info) => Response::success(
            id,
            json!({
                "ok": true,
                "claim_id": info.claim_id,
                "topic": info.topic,
                "offset": info.offset,
                "ack": info.ack,
            }),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotFound(cid)) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_FOUND,
            &format!("channel.release: claim {cid:?} not found"),
            json!({"claim_id": cid}),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotOwned {
            claim_id: cid,
            claimed_by,
            attempted_by,
        }) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_OWNED,
            &format!(
                "channel.release: claim {cid:?} held by {claimed_by:?}, not {attempted_by:?}"
            ),
            json!({
                "claim_id": cid,
                "claimed_by": claimed_by,
                "attempted_by": attempted_by,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.release: {e}")).into(),
    }
}

/// T-2044 (arc-parallel-substrate Slice 11) — `channel.force_release(claim_id, reason?)`.
/// Operator-Tier-0 force-release: bypasses the `claimed_by == claimer`
/// ownership check that `channel.release` enforces. Semantics match
/// `release(ack=false)` — cursor unchanged, slot freed for the next worker.
/// Echoes the original claimer in `forced_from` and the operator-supplied
/// reason in `forced_reason` for the audit trail.
pub async fn handle_channel_force_release(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_force_release_with(bus, id, params).await
}

pub(crate) async fn handle_channel_force_release_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let claim_id = match param_str(params, "claim_id") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claim_id' in params").into(),
    };
    let reason = param_str(params, "reason");
    match bus.force_release_claim(claim_id, reason) {
        Ok(info) => Response::success(
            id,
            json!({
                "ok": true,
                "claim_id": info.claim_id,
                "topic": info.topic,
                "offset": info.offset,
                "forced_from": info.forced_from,
                "forced_reason": info.forced_reason,
            }),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotFound(cid)) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_FOUND,
            &format!("channel.force_release: claim {cid:?} not found"),
            json!({"claim_id": cid}),
        )
        .into(),
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("channel.force_release: {e}")).into()
        }
    }
}

/// T-2046 (T-2021 GO, arc-parallel-substrate primitive #3) — `channel.transfer_claim(claim_id, to_owner, by, reason?)`.
/// Atomic ownership transfer of an existing claim. Cooperative + owner-checked:
/// `by` MUST equal the row's current `claimed_by` (`CLAIM_NOT_OWNED` otherwise) —
/// this is the orchestrator-to-worker handoff path, not the operator-Tier-0
/// bypass (that's `channel.force_release`). Lease timestamps survive the
/// transfer; only `claimed_by` mutates.
pub async fn handle_channel_transfer_claim(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_transfer_claim_with(bus, id, params).await
}

pub(crate) async fn handle_channel_transfer_claim_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let claim_id = match param_str(params, "claim_id") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claim_id' in params").into(),
    };
    let to_owner = match param_str(params, "to_owner") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'to_owner' in params").into(),
    };
    let by = match param_str(params, "by") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'by' in params").into(),
    };
    let reason = param_str(params, "reason");
    match bus.transfer_claim(claim_id, to_owner, by, reason) {
        Ok(info) => Response::success(
            id,
            json!({
                "ok": true,
                "claim_id": info.claim_id,
                "topic": info.topic,
                "offset": info.offset,
                "from_owner": info.from_owner,
                "to_owner": info.to_owner,
                "claimed_at": info.claimed_at,
                "claimed_until": info.claimed_until,
                "reason": info.reason,
            }),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotFound(cid)) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_FOUND,
            &format!("channel.transfer_claim: claim {cid:?} not found"),
            json!({"claim_id": cid}),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimExpired { claim_id: cid }) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_EXPIRED,
            &format!("channel.transfer_claim: claim {cid:?} lease has lapsed"),
            json!({"claim_id": cid}),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotOwned {
            claim_id: cid,
            claimed_by,
            attempted_by,
        }) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_OWNED,
            &format!(
                "channel.transfer_claim: claim {cid:?} held by {claimed_by:?}, not {attempted_by:?}"
            ),
            json!({
                "claim_id": cid,
                "claimed_by": claimed_by,
                "attempted_by": attempted_by,
            }),
        )
        .into(),
        Err(e) => {
            ErrorResponse::internal_error(id, &format!("channel.transfer_claim: {e}")).into()
        }
    }
}

/// T-2030 (arc-parallel-substrate Slice 2) — `channel.renew(claim_id, claimer, additional_ttl_ms?)`.
/// Extends a worker's lease before `claimed_until`. Gates on caller-is-original-claimer
/// AND not-yet-expired; returns the refreshed `ClaimInfo` shape on success.
pub async fn handle_channel_renew(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_renew_with(bus, id, params).await
}

pub(crate) async fn handle_channel_renew_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let claim_id = match param_str(params, "claim_id") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claim_id' in params").into(),
    };
    let claimer = match param_str(params, "claimer") {
        Some(c) if !c.is_empty() => c,
        _ => return ErrorResponse::new(id, -32602, "Missing 'claimer' in params").into(),
    };
    // Default extension: 30s. Same clamp as channel.claim (1h max).
    let additional_ttl_ms = params
        .get("additional_ttl_ms")
        .and_then(|v| v.as_u64())
        .map(|t| t.min(60 * 60 * 1000) as u32)
        .unwrap_or(30_000);
    match bus.renew_claim(claim_id, claimer, additional_ttl_ms) {
        Ok(info) => Response::success(
            id,
            json!({
                "ok": true,
                "claim_id": info.claim_id,
                "topic": info.topic,
                "offset": info.offset,
                "claimer": info.claimer,
                "claimed_at": info.claimed_at,
                "claimed_until": info.claimed_until,
            }),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotFound(cid)) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_FOUND,
            &format!("channel.renew: claim {cid:?} not found"),
            json!({"claim_id": cid}),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimExpired { claim_id: cid }) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_EXPIRED,
            &format!("channel.renew: claim {cid:?} lease has lapsed"),
            json!({"claim_id": cid}),
        )
        .into(),
        Err(termlink_bus::BusError::ClaimNotOwned {
            claim_id: cid,
            claimed_by,
            attempted_by,
        }) => ErrorResponse::with_data(
            id,
            error_code::CLAIM_NOT_OWNED,
            &format!(
                "channel.renew: claim {cid:?} held by {claimed_by:?}, not {attempted_by:?}"
            ),
            json!({
                "claim_id": cid,
                "claimed_by": claimed_by,
                "attempted_by": attempted_by,
            }),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.renew: {e}")).into(),
    }
}

/// T-2037 (arc-parallel-substrate Slice 4) — `channel.claims(topic, include_expired?)`.
/// Read-only listing of claim rows for `topic`. Default surfaces only live
/// leases (rows where `claimed_until > now`); `include_expired=true` returns
/// all rows for operator forensics. No state mutation — no lazy eviction
/// here either; the next `channel.claim` for the same `(topic, offset)`
/// handles that.
pub async fn handle_channel_claims(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_claims_with(bus, id, params).await
}

pub(crate) async fn handle_channel_claims_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    let include_expired = params
        .get("include_expired")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    match bus.list_claims(topic, include_expired) {
        Ok(rows) => {
            let claims: Vec<Value> = rows
                .iter()
                .map(|c| {
                    json!({
                        "claim_id": c.claim_id,
                        "offset": c.offset,
                        "claimer": c.claimer,
                        "claimed_at": c.claimed_at,
                        "claimed_until": c.claimed_until,
                    })
                })
                .collect();
            Response::success(
                id,
                json!({
                    "ok": true,
                    "topic": topic,
                    "claims": claims,
                }),
            )
            .into()
        }
        Err(termlink_bus::BusError::UnknownTopic(_)) => ErrorResponse::new(
            id,
            error_code::CHANNEL_TOPIC_UNKNOWN,
            &format!("channel.claims: topic {topic:?} not found"),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(id, &format!("channel.claims: {e}")).into(),
    }
}

/// T-2039 — `channel.claims_summary`: aggregate claim state for a topic.
/// Read-only observability companion to `channel.claims`. Returns counts
/// (active/expired) plus oldest-active and next-expiry markers so an
/// operator can answer "is this topic busy / is anything stuck?" with one
/// O(1) call instead of paying full-list transfer cost.
pub async fn handle_channel_claims_summary(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_claims_summary_with(bus, id, params).await
}

pub(crate) async fn handle_channel_claims_summary_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    match bus.claims_summary(topic) {
        Ok(s) => Response::success(
            id,
            json!({
                "ok": true,
                "topic": topic,
                "active_count": s.active_count,
                "expired_count": s.expired_count,
                "oldest_active_at_ms": s.oldest_active_at_ms,
                "oldest_active_age_ms": s.oldest_active_age_ms,
                "next_active_expiry_ms": s.next_active_expiry_ms,
            }),
        )
        .into(),
        Err(termlink_bus::BusError::UnknownTopic(_)) => ErrorResponse::new(
            id,
            error_code::CHANNEL_TOPIC_UNKNOWN,
            &format!("channel.claims_summary: topic {topic:?} not found"),
        )
        .into(),
        Err(e) => ErrorResponse::internal_error(
            id,
            &format!("channel.claims_summary: {e}"),
        )
        .into(),
    }
}

/// T-2106 — `channel.cv_keys`: operator inspection of the per-topic cv_index
/// (substrate primitive #9). Read-only. Returns `[{cv_key, offset}, ...]`
/// sorted by cv_key. Empty cv_index → `count: 0, entries: []` (NOT an
/// error; healthy topic with no cv-tagged posts is a valid state).
/// Missing topic → CHANNEL_TOPIC_UNKNOWN (mirror of claims_summary).
pub async fn handle_channel_cv_keys(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_channel_cv_keys_with(bus, id, params).await
}

pub(crate) async fn handle_channel_cv_keys_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let topic = match param_str(params, "topic") {
        Some(t) if !t.is_empty() => t,
        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
    };
    // Topic existence check via list_topics — operator verb, not hot path.
    // We could call bus.topic_record_count() and match on UnknownTopic, but
    // list_topics has the simplest contract.
    let topics = match bus.list_topics() {
        Ok(t) => t,
        Err(e) => {
            return ErrorResponse::internal_error(id, &format!("channel.cv_keys: {e}")).into();
        }
    };
    if !topics.iter().any(|t| t == topic) {
        return ErrorResponse::new(
            id,
            error_code::CHANNEL_TOPIC_UNKNOWN,
            &format!("channel.cv_keys: topic {topic:?} not found"),
        )
        .into();
    }
    let mut entries = crate::cv_index::current_values(topic);
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let entries_json: Vec<Value> = entries
        .iter()
        .map(|(cv_key, offset)| json!({"cv_key": cv_key, "offset": offset}))
        .collect();
    Response::success(
        id,
        json!({
            "ok": true,
            "topic": topic,
            "count": entries_json.len(),
            "entries": entries_json,
        }),
    )
    .into()
}

/// T-2045 (T-2020 GO) — `agent.find_idle`: derived idle-agent roster.
/// Server-side join of `agent-presence` (LIVE) ∖ active claims (any topic).
/// Params: `{ role?: string, capabilities?: [string], limit?: u32 }` →
/// `{ ok, idle: [...] }`. Default LIVE window is 2× the standard 30s
/// heartbeat interval (60_000 ms). No new persistent state — pure read.
pub async fn handle_agent_find_idle(id: Value, params: &Value) -> RpcResponse {
    let bus = match bus_or_err(id.clone()) {
        Ok(b) => b,
        Err(r) => return r,
    };
    handle_agent_find_idle_with(bus, id, params).await
}

pub(crate) async fn handle_agent_find_idle_with(
    bus: &Bus,
    id: Value,
    params: &Value,
) -> RpcResponse {
    let role = param_str(params, "role").map(String::from);
    let capabilities: Vec<String> = params
        .get("capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32);
    // 2× the canonical 30s heartbeat interval — matches /be-reachable default.
    const DEFAULT_LIVE_WINDOW_MS: i64 = 60_000;

    // T-2109: cv_index fast path — substrate primitives 2 + 9 cross-reference.
    // When the cv_index records `(agent-presence, agent_id) → latest_offset`
    // for every advertiser (default since T-2107 wired cv_key=$agent_id
    // into listener-heartbeat.sh), resolve idle agents in O(N_agents)
    // single-offset reads instead of walking the whole topic. Empty cv_index
    // (cold start, no producers wired) falls back to the walk path.
    let cv_entries = crate::cv_index::current_values("agent-presence");
    let outcome = if !cv_entries.is_empty() {
        bus.find_idle_agents_from_hint(
            role.as_deref(),
            &capabilities,
            DEFAULT_LIVE_WINDOW_MS,
            limit,
            &cv_entries,
        )
    } else {
        bus.find_idle_agents(
            role.as_deref(),
            &capabilities,
            DEFAULT_LIVE_WINDOW_MS,
            limit,
        )
    };

    match outcome {
        Ok(idle) => {
            let entries: Vec<Value> = idle
                .into_iter()
                .map(|a| {
                    json!({
                        "agent_id": a.agent_id,
                        "last_heartbeat_ms": a.last_heartbeat_ms,
                        "role": a.role,
                        "capabilities": a.capabilities,
                    })
                })
                .collect();
            Response::success(
                id,
                json!({
                    "ok": true,
                    "idle": entries,
                }),
            )
            .into()
        }
        Err(e) => ErrorResponse::internal_error(
            id,
            &format!("agent.find_idle: {e}"),
        )
        .into(),
    }
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
    if !env.metadata.is_empty()
        && let Some(obj) = out.as_object_mut()
    {
        obj.insert(
            "metadata".to_string(),
            serde_json::to_value(&env.metadata).unwrap_or(Value::Null),
        );
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

    // ── T-2355: deadline-bounded record walks ────────────────────────────

    fn walk_test_env(n: u64, msg_type: &str, up_to: Option<u64>) -> Envelope {
        let mut metadata = std::collections::BTreeMap::new();
        if let Some(u) = up_to {
            metadata.insert("up_to".to_string(), u.to_string());
        }
        Envelope {
            topic: "walk-test".into(),
            sender_id: format!("sender-{n}"),
            msg_type: msg_type.into(),
            payload: format!("m{n}").into_bytes(),
            artifact_ref: None,
            ts_unix_ms: 1000 + n as i64,
            metadata,
        }
    }

    /// Synthetic SubscribeIter: `count` note records at offsets 0..count.
    /// When `sleep_from_second_ms > 0`, every fetch AFTER the first sleeps
    /// that long — lets tests deterministically trip a small deadline
    /// between record 1 and record 2.
    fn walk_test_iter(count: u64, sleep_from_second_ms: u64) -> termlink_bus::SubscribeIter {
        let mut n = 0u64;
        Box::new(std::iter::from_fn(move || {
            if n >= count {
                return None;
            }
            if n > 0 && sleep_from_second_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(sleep_from_second_ms));
            }
            let item = Ok((n, walk_test_env(n, "note", None)));
            n += 1;
            Some(item)
        }))
    }

    #[test]
    fn subscribe_walk_zero_deadline_hits_before_any_record() {
        let walk = walk_subscribe_records(
            walk_test_iter(5, 0),
            None,
            None,
            100,
            std::time::Duration::ZERO,
        );
        assert!(walk.deadline_hit, "zero deadline must trip immediately");
        assert_eq!(walk.records_scanned, 0);
        assert_eq!(walk.last_offset, None, "no record processed → resume at original cursor");
        assert!(walk.messages.is_empty());
        assert!(walk.error_msg.is_none());
    }

    #[test]
    fn subscribe_walk_generous_deadline_completes_full_topic() {
        let walk = walk_subscribe_records(
            walk_test_iter(5, 0),
            None,
            None,
            100,
            std::time::Duration::from_secs(60),
        );
        assert!(!walk.deadline_hit);
        assert_eq!(walk.records_scanned, 5);
        assert_eq!(walk.messages.len(), 5);
        assert_eq!(walk.last_offset, Some(4));
        assert!(walk.error_msg.is_none());
    }

    #[test]
    fn subscribe_walk_mid_deadline_preserves_resume_cursor() {
        // Record 0 fetches instantly and is processed inside the 5ms
        // budget; record 1's fetch sleeps 50ms, so the top-of-loop check
        // trips BEFORE record 1 is processed. last_offset must cover only
        // the fully-processed record 0 → resume cursor 1 re-reads nothing
        // and skips nothing.
        let walk = walk_subscribe_records(
            walk_test_iter(3, 50),
            None,
            None,
            100,
            std::time::Duration::from_millis(5),
        );
        assert!(walk.deadline_hit);
        assert_eq!(walk.records_scanned, 1);
        assert_eq!(walk.last_offset, Some(0));
        assert_eq!(walk.messages.len(), 1);
    }

    #[test]
    fn receipts_walk_zero_deadline_hits_before_any_record() {
        let walk = walk_receipt_records(walk_test_iter(3, 0), std::time::Duration::ZERO);
        assert!(walk.deadline_hit);
        assert_eq!(walk.records_scanned, 0);
        assert!(walk.latest.is_empty());
        assert!(walk.error_msg.is_none());
    }

    #[test]
    fn receipts_walk_generous_deadline_aggregates_latest_per_sender() {
        let mut n = 0u64;
        let iter: termlink_bus::SubscribeIter = Box::new(std::iter::from_fn(move || {
            if n >= 3 {
                return None;
            }
            // Two receipts from the same sender (up_to 5 then 9) + one note.
            let env = match n {
                0 => {
                    let mut e = walk_test_env(0, "receipt", Some(5));
                    e.sender_id = "peer-a".into();
                    e
                }
                1 => walk_test_env(1, "note", None),
                _ => {
                    let mut e = walk_test_env(2, "receipt", Some(9));
                    e.sender_id = "peer-a".into();
                    e
                }
            };
            let item = Ok((n, env));
            n += 1;
            Some(item)
        }));
        let walk = walk_receipt_records(iter, std::time::Duration::from_secs(60));
        assert!(!walk.deadline_hit);
        assert_eq!(walk.records_scanned, 3);
        assert_eq!(walk.latest.len(), 1);
        assert_eq!(walk.latest.get("peer-a").expect("peer-a receipt").up_to, 9);
    }

    #[test]
    fn walk_deadline_env_default_and_clamp() {
        // No env (or unparseable) → 20s default; clamp bounds 100..=600_000.
        // set_var/remove_var within one test — walk_deadline_from_env is
        // only read by handlers, whose test walks finish well inside even
        // the clamped 100ms floor, so transient exposure is harmless.
        unsafe { std::env::remove_var("TERMLINK_WALK_DEADLINE_MS") };
        assert_eq!(walk_deadline_from_env().as_millis(), 20_000);
        unsafe { std::env::set_var("TERMLINK_WALK_DEADLINE_MS", "1") };
        assert_eq!(walk_deadline_from_env().as_millis(), 100, "clamped to floor");
        unsafe { std::env::set_var("TERMLINK_WALK_DEADLINE_MS", "999999999") };
        assert_eq!(walk_deadline_from_env().as_millis(), 600_000, "clamped to ceiling");
        unsafe { std::env::set_var("TERMLINK_WALK_DEADLINE_MS", "garbage") };
        assert_eq!(walk_deadline_from_env().as_millis(), 20_000, "unparseable → default");
        unsafe { std::env::remove_var("TERMLINK_WALK_DEADLINE_MS") };
    }

    // T-2058: high-rate-pattern matcher exhaustiveness.
    #[test]
    fn high_rate_matches_named_topics() {
        assert!(is_high_rate_pattern("agent-presence"));
        assert!(is_high_rate_pattern("agent-chat-arc"));
    }

    #[test]
    fn high_rate_matches_known_prefixes() {
        assert!(is_high_rate_pattern("agent-listeners-test-T11-1234"));
        assert!(is_high_rate_pattern("agent-conv-demo-99"));
        assert!(is_high_rate_pattern("dm:abc:def"));
    }

    #[test]
    fn high_rate_does_not_match_legitimate_operator_topics() {
        // Operator-named durable records should NOT trigger the warn.
        assert!(!is_high_rate_pattern("channel:learnings"));
        assert!(!is_high_rate_pattern("policy-decisions"));
        assert!(!is_high_rate_pattern("framework:pickup"));
        assert!(!is_high_rate_pattern("broadcast:global"));
        // Broad `agent-*` prefix is intentionally NOT matched — avoid noise.
        assert!(!is_high_rate_pattern("agent-my-custom-topic"));
        // Empty name doesn't match (handled separately in caller).
        assert!(!is_high_rate_pattern(""));
    }

    // T-2145: single-value-state-pattern matcher exhaustiveness.
    #[test]
    fn is_single_value_state_pattern_matches_state_prefix() {
        assert!(is_single_value_state_pattern("state:deploy-mode"));
        assert!(is_single_value_state_pattern("state:current-leader"));
        assert!(is_single_value_state_pattern("state:active-version"));
        // Empty suffix is still a match — caller decides whether to reject.
        assert!(is_single_value_state_pattern("state:"));
    }

    #[test]
    fn is_single_value_state_pattern_rejects_non_state() {
        // High-rate patterns must NOT trip the state warn (and vice versa).
        assert!(!is_single_value_state_pattern("agent-presence"));
        assert!(!is_single_value_state_pattern("agent-chat-arc"));
        assert!(!is_single_value_state_pattern("dm:abc:def"));
        // Substring "state" elsewhere doesn't match — prefix-only.
        assert!(!is_single_value_state_pattern("smoke:state:x"));
        assert!(!is_single_value_state_pattern("statebook"));
        // Empty name doesn't match.
        assert!(!is_single_value_state_pattern(""));
    }

    #[test]
    fn high_rate_and_state_predicates_are_disjoint() {
        // The two predicates must never both match the same name — they
        // emit different "consider X instead of Forever" warnings and
        // overlap would double-log.
        for name in [
            "agent-presence",
            "agent-chat-arc",
            "agent-listeners-host1",
            "agent-conv-thread-42",
            "dm:alice:bob",
            "state:deploy-mode",
            "state:current-leader",
        ] {
            assert!(
                !(is_high_rate_pattern(name) && is_single_value_state_pattern(name)),
                "predicate overlap on {name}"
            );
        }
    }

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
        // T-1427: post fixtures must use the fp derived from the signing
        // key so they pass the strict-reject in handle_channel_post_with.
        let sender_id = fingerprint_of(&key.verifying_key());
        let mut p = json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "ts": ts,
            "sender_id": sender_id,
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
        let sender_id = fingerprint_of(&key.verifying_key());
        json!({
            "topic": topic,
            "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload),
            "ts": ts,
            "sender_id": sender_id,
            "sender_pubkey_hex": hex_of(key.verifying_key().as_bytes()),
            "signature_hex": hex_of(&sig.to_bytes()),
        })
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn trim_missing_topic_returns_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_trim_with(&bus, json!(1), &json!({})).await;
        match resp {
            RpcResponse::Error(e) => assert_eq!(e.error.code, -32602),
            _ => panic!("expected error"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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
        // T-1427: sender_id is the fp derived from the test signing key,
        // not the legacy literal "tester" — strict-reject enforces this.
        assert_eq!(
            msgs[0]["sender_id"].as_str().unwrap(),
            fingerprint_of(&key.verifying_key())
        );
        assert_eq!(msgs[0]["msg_type"], "note");
        assert_eq!(
            msgs[0]["payload_b64"].as_str().unwrap(),
            base64::engine::general_purpose::STANDARD.encode(b"hello")
        );
        assert_eq!(v["next_cursor"], 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn create_missing_name_is_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_create_with(&bus, json!(1), &json!({})).await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602);
    }

    // === T-2244 (R2a): channel.set_retention ===

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_retention_changes_existing_topic() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let resp = handle_channel_set_retention_with(
            &bus,
            json!(1),
            &json!({"name": "agent-presence", "retention": {"kind": "days", "value": 2}}),
        )
        .await;
        let v = unwrap_success(resp);
        assert_eq!(v["ok"], true);
        assert_eq!(v["updated"], true);
        assert_eq!(v["retention"]["kind"], "days");
        assert_eq!(v["retention"]["value"], 2);
        // Confirm the policy actually changed in the bus.
        assert_eq!(bus.topic_retention("agent-presence").unwrap(), Some(Retention::Days(2)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_retention_unknown_topic_errors_not_creates() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_set_retention_with(
            &bus,
            json!(1),
            &json!({"name": "no-such-topic", "retention": {"kind": "latest"}}),
        )
        .await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602, "unknown topic is a clear error");
        // Must NOT have stealth-created the topic.
        assert_eq!(bus.topic_retention("no-such-topic").unwrap(), None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_retention_missing_retention_is_invalid_params() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let resp = handle_channel_set_retention_with(&bus, json!(1), &json!({"name": "t"})).await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602);
    }

    // === T-2245 (R2b): latest_per_cv_key retention + channel.sweep trigger ===

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn set_retention_latest_per_cv_key_roundtrips() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let resp = handle_channel_set_retention_with(
            &bus,
            json!(1),
            &json!({"name": "agent-presence", "retention": {"kind": "latest_per_cv_key"}}),
        )
        .await;
        let v = unwrap_success(resp);
        assert_eq!(v["retention"]["kind"], "latest_per_cv_key");
        assert_eq!(
            bus.topic_retention("agent-presence").unwrap(),
            Some(Retention::LatestPerCvKey)
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sweep_enforces_latest_per_cv_key_and_reports_pruned() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::LatestPerCvKey).unwrap();
        // 2 agents × 3 beats = 6 keyed records.
        for round in 0..3 {
            for key in ["alpha", "beta"] {
                let mut metadata = std::collections::BTreeMap::new();
                metadata.insert("cv_key".to_string(), key.to_string());
                let env = Envelope {
                    topic: "agent-presence".to_string(),
                    sender_id: key.to_string(),
                    msg_type: "presence".to_string(),
                    payload: format!("{key}-{round}").into_bytes(),
                    artifact_ref: None,
                    ts_unix_ms: round,
                    metadata,
                };
                bus.post("agent-presence", &env).await.unwrap();
            }
        }
        let resp =
            handle_channel_sweep_with(&bus, json!(1), &json!({"topic": "agent-presence"})).await;
        let v = unwrap_success(resp);
        assert_eq!(v["ok"], true);
        assert_eq!(v["pruned"], 4, "6 beats - 2 keys = 4 pruned");
        // PROPERTY: record count converges to the agent count.
        assert_eq!(bus.topic_record_count("agent-presence").unwrap(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sweep_unknown_topic_errors_not_creates() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_sweep_with(&bus, json!(1), &json!({"topic": "no-such"})).await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602, "unknown topic is a clear error, not a stealth create");
        assert_eq!(bus.topic_retention("no-such").unwrap(), None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sweep_missing_topic_param_is_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_sweep_with(&bus, json!(1), &json!({})).await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602);
    }

    // === T-2421: channel.delete handler ===

    fn delete_test_env(topic: &str, payload: &[u8]) -> Envelope {
        Envelope {
            topic: topic.to_string(),
            sender_id: "test".to_string(),
            msg_type: "note".to_string(),
            payload: payload.to_vec(),
            artifact_ref: None,
            ts_unix_ms: 0,
            metadata: std::collections::BTreeMap::new(),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn delete_removes_topic_and_reports_count() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("debris", Retention::Forever).unwrap();
        bus.post("debris", &delete_test_env("debris", b"a")).await.unwrap();
        bus.post("debris", &delete_test_env("debris", b"b")).await.unwrap();
        let resp =
            handle_channel_delete_with(&bus, json!(1), &json!({"topic": "debris"})).await;
        let v = unwrap_success(resp);
        assert_eq!(v["ok"], true);
        assert_eq!(v["deleted_records"], 2);
        assert_eq!(v["topic"], "debris");
        assert!(!bus.list_topics().unwrap().contains(&"debris".to_string()));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn delete_unknown_topic_is_loud_error() {
        let (_d, bus) = tmp_bus();
        let resp =
            handle_channel_delete_with(&bus, json!(1), &json!({"topic": "no-such"})).await;
        let (code, msg) = unwrap_error(resp);
        assert_eq!(code, -32602);
        assert!(msg.contains("unknown topic"), "got: {msg}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn delete_rejects_wildcards() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("smoke-1", Retention::Forever).unwrap();
        for pattern in ["smoke-*", "smoke-?", "*"] {
            let resp =
                handle_channel_delete_with(&bus, json!(1), &json!({"topic": pattern})).await;
            let (code, msg) = unwrap_error(resp);
            assert_eq!(code, -32602, "wildcard '{pattern}' must be refused");
            assert!(msg.contains("wildcards not allowed"), "got: {msg}");
        }
        // The real topic survives every refused wildcard attempt.
        assert!(bus.list_topics().unwrap().contains(&"smoke-1".to_string()));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn delete_missing_topic_param_is_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_delete_with(&bus, json!(1), &json!({})).await;
        let (code, _) = unwrap_error(resp);
        assert_eq!(code, -32602);
    }

    // === T-1162: event.broadcast → channel:broadcast:global mirror ===

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mirror_event_broadcast_without_bus_is_noop() {
        // Calling the public entry point with no process-global bus set
        // must not panic or error — the shim is best-effort.
        mirror_event_broadcast("x.y", &json!({})).await;
    }

    // === T-1163: inbox.deposit → channel:inbox:<target> mirror ===

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mirror_inbox_deposit_without_bus_is_noop() {
        // Public entry with no process-global bus set must not panic.
        mirror_inbox_deposit("any-target", "file.init", &json!({}), None).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dialog_presence_tracks_senders_per_conversation() {
        // T-1286: post 3 messages on cid=t1286-c1 with two distinct sender
        // identities. dialog.presence("t1286-c1") returns 2 entries
        // (alice + bob), sorted by agent_id. alice's last_seen_ms is the
        // ts of the LATER alice post (overwrites earlier).
        //
        // T-1427: senders use distinct signing keys so the strict-reject
        // pass (sender_id must match fingerprint_of(pubkey)). Pre-T-1427
        // this test labelled posts "alice"/"bob" with one shared key —
        // legitimate clients never do that.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:pres", Retention::Forever).unwrap();
        let alice_key = SigningKey::from_bytes(&[0xA1u8; 32]);
        let bob_key = SigningKey::from_bytes(&[0xB0u8; 32]);
        let alice_fp = fingerprint_of(&alice_key.verifying_key());
        let bob_fp = fingerprint_of(&bob_key.verifying_key());
        // Sort by fp so we can index assertions stably regardless of which
        // key happens to come first lexicographically.
        let (lo_fp, hi_fp) = if alice_fp < bob_fp {
            (alice_fp.clone(), bob_fp.clone())
        } else {
            (bob_fp.clone(), alice_fp.clone())
        };

        // alice's first post — ts 5_001
        let p_a1 = post_params_with_meta(
            &alice_key, "inbox:pres", "note", b"a1", 5_001,
            Some(json!({"conversation_id": "t1286-c1"})),
        );
        let _ = handle_channel_post_with(&bus, json!(1), &p_a1).await;
        // bob's post — ts 5_002
        let p_b = post_params_with_meta(
            &bob_key, "inbox:pres", "note", b"b1", 5_002,
            Some(json!({"conversation_id": "t1286-c1"})),
        );
        let _ = handle_channel_post_with(&bus, json!(2), &p_b).await;
        // alice's second post — ts 5_003 (must overwrite the 5_001)
        let p_a2 = post_params_with_meta(
            &alice_key, "inbox:pres", "note", b"a2", 5_003,
            Some(json!({"conversation_id": "t1286-c1"})),
        );
        let _ = handle_channel_post_with(&bus, json!(3), &p_a2).await;

        let resp = handle_dialog_presence(
            json!(4),
            &json!({"conversation_id": "t1286-c1"}),
        )
        .await;
        let v = unwrap_success(resp);
        let presences = v["presences"].as_array().unwrap();
        assert_eq!(presences.len(), 2, "alice + bob, alice deduped");

        // Sorted by agent_id (which is sender_id = identity fingerprint).
        let lo_ts = if lo_fp == alice_fp { 5_003i64 } else { 5_002i64 };
        let hi_ts = if hi_fp == alice_fp { 5_003i64 } else { 5_002i64 };
        assert_eq!(presences[0]["agent_id"], lo_fp);
        assert_eq!(presences[0]["last_seen_ms"], lo_ts);
        assert_eq!(presences[1]["agent_id"], hi_fp);
        assert_eq!(presences[1]["last_seen_ms"], hi_ts);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dialog_presence_ignores_posts_without_conversation_id() {
        // T-1286: posts that have no metadata.conversation_id must NOT
        // appear in any presence query.
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:plainpres", Retention::Forever).unwrap();
        let key = signing_key();
        // No metadata at all.
        let p = post_params(&key, "inbox:plainpres", "note", b"x", 6_000);
        let _ = handle_channel_post_with(&bus, json!(1), &p).await;
        // Metadata present but no conversation_id key.
        let p2 = post_params_with_meta(
            &key, "inbox:plainpres", "note", b"y", 6_001,
            Some(json!({"event_type": "turn"})),
        );
        let _ = handle_channel_post_with(&bus, json!(2), &p2).await;

        // Use a unique cid that no post used so we know any entry would
        // be from cross-contamination of the global tracker.
        let resp = handle_dialog_presence(
            json!(3),
            &json!({"conversation_id": "t1286-noexist"}),
        )
        .await;
        let v = unwrap_success(resp);
        assert_eq!(v["presences"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dialog_presence_unknown_conversation_returns_empty() {
        // T-1286: dialog.presence on a conversation_id that's never been
        // seen returns {presences: []} (not an error) — presence is
        // observational, absence is information.
        let resp = handle_dialog_presence(
            json!(1),
            &json!({"conversation_id": "t1286-truly-unknown-xyz"}),
        )
        .await;
        let v = unwrap_success(resp);
        assert_eq!(v["presences"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dialog_presence_missing_conversation_id_is_invalid_params() {
        // T-1286: required param. Missing → -32602.
        let resp = handle_dialog_presence(json!(1), &json!({})).await;
        match resp {
            RpcResponse::Error(e) => assert_eq!(e.error.code, -32602),
            _ => panic!("expected error"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_in_reply_to_filter_returns_only_replies_to_parent() {
        // T-1313: post a parent + two replies (one to parent, one to a different
        // offset) + an unrelated message. Subscribe filtered by in_reply_to=<parent>
        // returns only the matching reply.
        let (_d, bus) = tmp_bus();
        bus.create_topic("conv:thread", Retention::Forever).unwrap();
        let key = signing_key();

        // offset 0 — parent
        let parent = post_params(&key, "conv:thread", "note", b"hi", 5_001);
        let _ = handle_channel_post_with(&bus, json!(1), &parent).await;

        // offset 1 — reply to parent (offset=0)
        let reply_to_parent = post_params_with_meta(
            &key,
            "conv:thread",
            "note",
            b"answer",
            5_002,
            Some(json!({"in_reply_to": "0"})),
        );
        let _ = handle_channel_post_with(&bus, json!(2), &reply_to_parent).await;

        // offset 2 — reply to a different offset
        let reply_other = post_params_with_meta(
            &key,
            "conv:thread",
            "note",
            b"unrelated reply",
            5_003,
            Some(json!({"in_reply_to": "99"})),
        );
        let _ = handle_channel_post_with(&bus, json!(3), &reply_other).await;

        // offset 3 — unrelated post (no metadata)
        let unrelated = post_params(&key, "conv:thread", "note", b"chitchat", 5_004);
        let _ = handle_channel_post_with(&bus, json!(4), &unrelated).await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(5),
            &json!({"topic": "conv:thread", "cursor": 0, "in_reply_to": "0"}),
        )
        .await;
        let v = unwrap_success(resp);
        let msgs = v["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1, "only the reply pointing at offset 0 should match");
        let payload_b64 = msgs[0]["payload_b64"].as_str().unwrap();
        let payload = base64::engine::general_purpose::STANDARD
            .decode(payload_b64)
            .unwrap();
        assert_eq!(&payload, b"answer");

        // next_cursor advances past all examined records (last offset + 1 = 4),
        // mirroring the conversation_id filter semantics.
        let next = v["next_cursor"].as_u64().unwrap();
        assert_eq!(next, 4, "cursor advances over skipped records");
    }

    /// T-1329: post 5 envelopes (3 receipts for alice with overlapping
    /// timestamps, 1 receipt for bob, 1 chat for charlie that must be ignored).
    /// Verify the aggregator returns latest-per-sender, sorted by sender,
    /// with both ts-tiebreak and msg_type filtering applied correctly.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn receipts_aggregates_latest_per_sender() {
        use std::collections::BTreeMap;
        let (_d, bus) = tmp_bus();
        bus.create_topic("dm:rcpt-test", Retention::Forever).unwrap();

        let mk = |sender: &str, msg_type: &str, ts: i64, up_to: Option<&str>| -> Envelope {
            let mut metadata: BTreeMap<String, String> = BTreeMap::new();
            if let Some(u) = up_to {
                metadata.insert("up_to".to_string(), u.to_string());
            }
            Envelope {
                topic: "dm:rcpt-test".to_string(),
                sender_id: sender.to_string(),
                msg_type: msg_type.to_string(),
                payload: vec![],
                artifact_ref: None,
                ts_unix_ms: ts,
                metadata,
            }
        };

        // alice: ts=100 up_to=5  → superseded
        bus.post("dm:rcpt-test", &mk("alice", "receipt", 100, Some("5"))).await.unwrap();
        // bob: ts=110 up_to=10
        bus.post("dm:rcpt-test", &mk("bob", "receipt", 110, Some("10"))).await.unwrap();
        // alice: ts=200 up_to=15 → newer ts, supersedes
        bus.post("dm:rcpt-test", &mk("alice", "receipt", 200, Some("15"))).await.unwrap();
        // alice: ts=200 up_to=20 → same ts, higher up_to wins tiebreak
        bus.post("dm:rcpt-test", &mk("alice", "receipt", 200, Some("20"))).await.unwrap();
        // charlie: ts=150 chat — NOT a receipt, must be filtered out
        bus.post("dm:rcpt-test", &mk("charlie", "chat", 150, None)).await.unwrap();

        let resp = handle_channel_receipts_with(
            &bus,
            json!(1),
            &json!({"topic": "dm:rcpt-test"}),
        )
        .await;
        let v = unwrap_success(resp);
        assert_eq!(v["topic"], "dm:rcpt-test");
        let arr = v["receipts"].as_array().unwrap();
        assert_eq!(arr.len(), 2, "should have 2 senders (alice + bob), charlie excluded");
        assert_eq!(arr[0]["sender_id"], "alice");
        assert_eq!(arr[0]["up_to"], 20, "tiebreak: higher up_to wins at same ts");
        assert_eq!(arr[0]["ts_unix_ms"], 200);
        assert_eq!(arr[1]["sender_id"], "bob");
        assert_eq!(arr[1]["up_to"], 10);
        assert_eq!(arr[1]["ts_unix_ms"], 110);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn receipts_unknown_topic_returns_error() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_receipts_with(
            &bus,
            json!(1),
            &json!({"topic": "no-such-topic"}),
        )
        .await;
        let (code, _msg) = unwrap_error(resp);
        assert_eq!(code, error_code::CHANNEL_TOPIC_UNKNOWN);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn receipts_missing_topic_param_returns_invalid_params() {
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_receipts_with(&bus, json!(1), &json!({})).await;
        let (code, _msg) = unwrap_error(resp);
        assert_eq!(code, -32602);
    }

    /// T-1427: hub rejects channel.post when claimed sender_id does not
    /// match fingerprint_of(sender_pubkey_hex). Closes the
    /// "identity authoritative" gap from T-1425 RFC §3.2.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn handle_channel_post_with_rejects_mismatched_sender_id() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let key = signing_key();
        // Build a valid signed envelope but claim a bogus sender_id.
        let mut params = post_params(&key, "t", "chat", b"hi", 1);
        params["sender_id"] = json!("imposter");
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let (code, msg) = unwrap_error(resp);
        assert_eq!(
            code,
            error_code::CHANNEL_IDENTITY_MISMATCH,
            "expected -32014, got code={code} msg={msg}"
        );
        assert!(
            msg.contains("imposter"),
            "error message should echo the bogus sender_id, got: {msg}"
        );
        assert!(
            msg.contains("T-1427"),
            "error message should cite T-1427 for traceability, got: {msg}"
        );
    }

    /// T-1427: hub accepts channel.post when sender_id matches the
    /// pubkey-derived fingerprint (the legitimate path the CLI default
    /// already takes via `identity.fingerprint()`).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn handle_channel_post_with_accepts_matching_sender_id() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let key = signing_key();
        // post_params already sets sender_id = fingerprint_of(key) post-T-1427.
        let params = post_params(&key, "t", "chat", b"hi", 2);
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let v = unwrap_success(resp);
        assert_eq!(v["offset"], 0);
        assert_eq!(v["ts"], 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn inbox_queued_fires_for_no_consumer() {
        let (_d, bus) = tmp_bus();
        crate::router::init_aggregator();
        let mut rx = crate::router::aggregator().unwrap().subscribe();
        mirror_inbox_deposit_with(&bus, "target-a", "file.init",
            &json!({"transfer_id": "x1"}), Some("sender")).await;
        // Aggregator is a process-global singleton; parallel tests may also
        // inject. Filter to this test's addressee.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        let evt = loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let r = tokio::time::timeout(remaining, rx.recv()).await
                .expect("timeout waiting for inbox.queued for target-a").expect("closed");
            if r.topic == termlink_protocol::events::inbox_topic::QUEUED
                && r.payload["addressee_session_id"] == "target-a" {
                break r;
            }
        };
        assert_eq!(evt.payload["channel"], "inbox:target-a");
        assert!(evt.payload["message_offset"].as_u64().is_some());
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn inbox_queued_not_emitted_without_deposit() {
        let agg = crate::aggregator::EventAggregator::new(16);
        let mut rx = agg.subscribe();
        let r = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
        assert!(r.is_err(), "no inbox.queued without deposit");
    }

    /// T-1637: channel.post to `inbox:<id>` topic fires inbox.queued, matching
    /// the T-1636 emit contract on the channel.post delivery path (the natural
    /// route AEF subscribers use, and the only route after T-1166 retirement).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn channel_post_inbox_topic_fires_inbox_queued() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("inbox:bob", Retention::Forever).unwrap();
        crate::router::init_aggregator();
        let mut rx = crate::router::aggregator().unwrap().subscribe();
        let key = signing_key();
        let params = post_params(&key, "inbox:bob", "file.init", b"{}", 42);
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let v = unwrap_success(resp);
        assert_eq!(v["offset"], 0);
        // Aggregator is a process-global singleton; other parallel tests may
        // inject too. Filter to the addressee this test posted to.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        let evt = loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let r = tokio::time::timeout(remaining, rx.recv()).await
                .expect("timeout waiting for inbox.queued for bob").expect("closed");
            if r.topic == termlink_protocol::events::inbox_topic::QUEUED
                && r.payload["addressee_session_id"] == "bob" {
                break r;
            }
        };
        assert_eq!(evt.payload["channel"], "inbox:bob");
        assert_eq!(evt.payload["message_offset"], 0);
        assert_eq!(evt.payload["enqueued_at"], 42);
    }

    /// T-1637 negative: channel.post to non-inbox topics MUST NOT fire
    /// inbox.queued (no false-positive wakeups on routine channel traffic).
    /// Uses a unique topic name so parallel tests' emits don't false-positive
    /// this negative assertion via the shared aggregator singleton.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn channel_post_non_inbox_topic_does_not_fire() {
        let (_d, bus) = tmp_bus();
        let topic = "t1637-not-inbox";
        bus.create_topic(topic, Retention::Forever).unwrap();
        crate::router::init_aggregator();
        let mut rx = crate::router::aggregator().unwrap().subscribe();
        let key = signing_key();
        let params = post_params(&key, topic, "msg", b"hi", 7);
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let _ = unwrap_success(resp);
        // Drain rx for 80ms and assert NO emit with our channel name appears.
        // Cross-test events (other inbox:* emits) are tolerated and skipped.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(80);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() { break; }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Ok(evt)) => {
                    assert_ne!(evt.payload["channel"], topic,
                        "inbox.queued must not fire for non-inbox topic '{topic}'");
                }
                _ => break,
            }
        }
    }

    /// T-2323 (arc-004 S1) positive: channel.post to a canonical `dm:<a>:<b>`
    /// topic fires `dm.queued` addressed to the participant that is NOT the
    /// sender — so the push-waker can ring the receiver even when the poster
    /// does not itself ring (raw post / cron / remote peer / MCP channel_post).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn channel_post_dm_topic_fires_dm_queued() {
        let (_d, bus) = tmp_bus();
        let key = signing_key();
        let sender_fp = fingerprint_of(&key.verifying_key());
        let addressee = "t2323-dm-addressee";
        let topic = format!("dm:{sender_fp}:{addressee}");
        bus.create_topic(&topic, Retention::Forever).unwrap();
        crate::router::init_aggregator();
        let mut rx = crate::router::aggregator().unwrap().subscribe();
        let params = post_params(&key, &topic, "dm.msg", b"hello", 99);
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let v = unwrap_success(resp);
        assert_eq!(v["offset"], 0);
        // Aggregator is a process-global singleton; filter to our addressee.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        let evt = loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let r = tokio::time::timeout(remaining, rx.recv()).await
                .expect("timeout waiting for dm.queued").expect("closed");
            if r.topic == termlink_protocol::events::dm_topic::QUEUED
                && r.payload["addressee_session_id"] == addressee {
                break r;
            }
        };
        assert_eq!(evt.payload["channel"], topic);
        assert_eq!(evt.payload["message_offset"], 0);
        assert_eq!(evt.payload["enqueued_at"], 99);
    }

    /// T-2323 (arc-004 S1) negative: a dm post whose authenticated sender is
    /// NEITHER participant (relay / third party) fires nothing — we never wake a
    /// party we cannot attribute. Guards the `None` branch of the addressee
    /// derivation. Uses unique halves so parallel emits don't false-positive.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn channel_post_dm_topic_sender_not_participant_does_not_fire() {
        let (_d, bus) = tmp_bus();
        let key = signing_key();
        let topic = "dm:t2323-other-a:t2323-other-b"; // sender fp is neither half
        bus.create_topic(topic, Retention::Forever).unwrap();
        crate::router::init_aggregator();
        let mut rx = crate::router::aggregator().unwrap().subscribe();
        let params = post_params(&key, topic, "dm.msg", b"x", 5);
        let resp = handle_channel_post_with(&bus, json!(1), &params).await;
        let _ = unwrap_success(resp);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(80);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() { break; }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Ok(evt)) => {
                    assert!(!(evt.topic == termlink_protocol::events::dm_topic::QUEUED
                        && evt.payload["channel"] == topic),
                        "dm.queued must not fire when sender is not a participant");
                }
                _ => break,
            }
        }
    }

    /// T-2013 regression: under multi-thread tokio runtime, sequential
    /// `channel.subscribe` walks of a large topic concurrent with a
    /// background writer must each complete promptly. Pre-fix code
    /// blocks the calling worker for the entire walk; under sequential
    /// load + concurrent writes the worker pool saturates and walks
    /// stretch into seconds. With `block_in_place` the worker yields
    /// to a fresh thread for the syscalls and other tasks make
    /// progress concurrently.
    ///
    /// This test posts 1100 envelopes to a topic, then runs 5
    /// sequential `handle_channel_subscribe_with` calls while a
    /// concurrent task posts every 100ms. Each subscribe call must
    /// complete in under 2 seconds (generous bound — typical wall
    /// clock is well under 500ms even with the concurrent writer).
    /// Test runs only under `flavor = "multi_thread"` — the entire
    /// channel::tests module already uses this flavor for T-2013.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn channel_subscribe_no_worker_starvation_under_concurrent_writes() {
        let (_d, bus) = tmp_bus();
        let topic = "t2013-starvation-probe";
        bus.create_topic(topic, Retention::Forever).unwrap();
        let key = signing_key();

        // Seed: 1100 envelopes. Each .next() does seek+read_exact —
        // that's the per-record blocking syscall we're proving is
        // safely yielded.
        for n in 0..1100u32 {
            let p = post_params(&key, topic, "seed", &n.to_le_bytes(), 1000 + n as i64);
            let _ = handle_channel_post_with(&bus, json!(n), &p).await;
        }

        // Background writer: posts continuously for the duration of
        // the test. Uses a separate signing key so the writes don't
        // interleave with the seed's offsets in a confusing way (any
        // valid post would do — the point is to keep the bus busy).
        let writer_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let writer_running_clone = writer_running.clone();
        // Workaround: we need to share &bus with the spawned task. The
        // tmp_bus() returns owned Bus, but we can't clone Bus. Use
        // unsafe-free pattern: keep this single-threaded background
        // writer via tokio::spawn on a 'static future bound by the
        // outer scope guard. Since Bus is !Sync-shareable across
        // threads here, we run the writer inside the same task as the
        // subscribe loop using tokio::join + select on a tick stream.
        // That's simpler than mixing threads and proves the same point:
        // concurrent posts interleaved with subscribes.
        drop(writer_running);
        drop(writer_running_clone);

        // Time 5 sequential subscribes. Between each, post one more
        // envelope (interleaved write). Each subscribe must walk all
        // 1100+ seed envelopes and complete in <2s.
        let mut interleave_seq: u32 = 9_000_000;
        for trial in 0..5 {
            // Interleave a write before each subscribe.
            let p = post_params(&key, topic, "interleaved", &interleave_seq.to_le_bytes(), 9_000_000 + interleave_seq as i64);
            let _ = handle_channel_post_with(&bus, json!(interleave_seq), &p).await;
            interleave_seq += 1;

            let start = std::time::Instant::now();
            let resp = handle_channel_subscribe_with(
                &bus,
                json!(2000 + trial),
                &json!({"topic": topic, "cursor": 0, "limit": 1000}),
            )
            .await;
            let elapsed = start.elapsed();
            let v = unwrap_success(resp);
            let msgs = v["messages"].as_array().unwrap();
            assert!(
                msgs.len() >= 1000,
                "trial {trial}: expected at least 1000 messages, got {}",
                msgs.len()
            );
            assert!(
                elapsed < std::time::Duration::from_secs(2),
                "trial {trial}: subscribe took {elapsed:?}, expected <2s (T-2013 regression — block_in_place not engaged?)"
            );
        }
    }

    /// T-2258 regression: TRUE concurrent subscribe walks of a large topic
    /// must not hang. The T-2013 test above runs walks SEQUENTIALLY — its
    /// author dropped the concurrent writer believing `Bus` couldn't be
    /// shared across threads — so K concurrent walks against a bounded
    /// worker pool (the field failure mode: ring20 read of framework:pickup
    /// ~1639 rows killed at 12s under concurrent post) was never exercised.
    /// `Bus` IS `Send + Sync`, so here we share it via `Arc` and spawn
    /// genuinely concurrent walkers + writers, then bound the whole join.
    /// Pre-fix (`block_in_place` pins the calling worker for the full walk)
    /// K>worker_threads concurrent walks starve the pool and the join
    /// exceeds the bound; the correct fix (`spawn_blocking`) offloads the
    /// walk to the blocking pool so the worker threads stay free.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn channel_subscribe_no_hang_under_concurrent_walks_t2258() {
        use std::sync::Arc;
        let (_d, bus) = tmp_bus();
        let topic = "t2258-concurrent-walk-probe";
        bus.create_topic(topic, Retention::Forever).unwrap();
        let key = signing_key();

        // Seed a large topic (~1500 envelopes — each walk does that many
        // seek+read_exact syscalls).
        for n in 0..1500u32 {
            let p = post_params(&key, topic, "seed", &n.to_le_bytes(), 1000 + n as i64);
            let _ = handle_channel_post_with(&bus, json!(n), &p).await;
        }

        let bus = Arc::new(bus);
        let mut handles = Vec::new();

        // 8 concurrent full-topic walkers (>> worker_threads=2).
        for r in 0..8u32 {
            let b = bus.clone();
            let t = topic.to_string();
            handles.push(tokio::spawn(async move {
                let resp = handle_channel_subscribe_with(
                    &b,
                    json!(5000 + r),
                    &json!({"topic": t, "cursor": 0, "limit": 1000}),
                )
                .await;
                let v = unwrap_success(resp);
                let msgs = v["messages"].as_array().unwrap();
                assert!(msgs.len() >= 1000, "walker {r}: expected >=1000 messages, got {}", msgs.len());
            }));
        }

        // 3 concurrent writers hammering the SAME topic during the walks.
        for w in 0..3u32 {
            let b = bus.clone();
            let t = topic.to_string();
            let wkey = signing_key();
            handles.push(tokio::spawn(async move {
                for i in 0..50u32 {
                    let seq = 8_000_000 + w * 1000 + i;
                    let p = post_params(&wkey, &t, "writer", &seq.to_le_bytes(), 8_000_000 + seq as i64);
                    let _ = handle_channel_post_with(&b, json!(seq), &p).await;
                }
            }));
        }

        // Bound the whole thing. Pre-fix this exceeds the bound (hang);
        // post-fix it completes in well under a second.
        let collect = async {
            for h in handles {
                h.await.expect("task panicked");
            }
        };
        let res = tokio::time::timeout(std::time::Duration::from_secs(10), collect).await;
        assert!(
            res.is_ok(),
            "T-2258: concurrent subscribe walks + writes did not complete within 10s — read-path worker starvation / hang"
        );
    }

    // T-2049 Gap A — client_msg_id idempotency integration tests.
    //
    // These exercise the full `handle_channel_post_with` path: optional
    // `client_msg_id` param, hub-side dedupe, cached-offset replay on
    // duplicate. The PostDedupe is a process-global, so we use
    // distinct (sender_id, client_msg_id) pairs per test to avoid
    // cross-test pollution under cargo's parallel harness — matching
    // the dual-tracker discipline from T-2048 slice 2.

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dedupe_no_client_msg_id_bypasses_dedupe_and_posts_normally() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("dedupe:bypass", Retention::Forever).unwrap();
        let key = SigningKey::from_bytes(&[51u8; 32]);

        let p1 = post_params(&key, "dedupe:bypass", "test", b"first", 1_000);
        let p2 = post_params(&key, "dedupe:bypass", "test", b"second", 2_000);

        let r1 = unwrap_success(handle_channel_post_with(&bus, json!(1), &p1).await);
        let r2 = unwrap_success(handle_channel_post_with(&bus, json!(2), &p2).await);

        // Without client_msg_id both posts append normally — two distinct offsets.
        assert_ne!(r1["offset"], r2["offset"]);
        assert!(r1.get("deduped").is_none());
        assert!(r2.get("deduped").is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dedupe_with_client_msg_id_first_post_succeeds_and_records() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("dedupe:first", Retention::Forever).unwrap();
        let key = SigningKey::from_bytes(&[52u8; 32]);

        let mut p = post_params(&key, "dedupe:first", "test", b"hello", 1_000);
        p.as_object_mut()
            .unwrap()
            .insert("client_msg_id".into(), json!("uniq-T2049-first-52"));

        let r = unwrap_success(handle_channel_post_with(&bus, json!(1), &p).await);
        // First post: real append, no deduped marker on success path.
        assert!(r["offset"].as_i64().unwrap() >= 0);
        assert!(r.get("deduped").is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dedupe_with_client_msg_id_duplicate_returns_cached_offset() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("dedupe:dup", Retention::Forever).unwrap();
        let key = SigningKey::from_bytes(&[53u8; 32]);

        let mut p = post_params(&key, "dedupe:dup", "test", b"world", 1_000);
        p.as_object_mut()
            .unwrap()
            .insert("client_msg_id".into(), json!("uniq-T2049-dup-53"));

        let r1 = unwrap_success(handle_channel_post_with(&bus, json!(1), &p).await);
        let offset1 = r1["offset"].as_i64().unwrap();

        // Replay the SAME params — hub sees same (sender_id, client_msg_id) → cached.
        let r2 = unwrap_success(handle_channel_post_with(&bus, json!(2), &p).await);
        let offset2 = r2["offset"].as_i64().unwrap();

        // Same offset → no double-apply.
        assert_eq!(offset1, offset2);
        // Second response carries the deduped marker.
        assert_eq!(r2["deduped"], json!(true));

        // Subscribe confirms only ONE envelope landed on the topic.
        let sub = unwrap_success(
            handle_channel_subscribe_with(&bus, json!(3), &json!({"topic": "dedupe:dup"})).await,
        );
        let msgs = sub["messages"].as_array().unwrap();
        assert_eq!(
            msgs.len(),
            1,
            "expected 1 envelope (dedupe absorbed retry), got {}",
            msgs.len()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dedupe_oversized_client_msg_id_is_ignored() {
        let (_d, bus) = tmp_bus();
        bus.create_topic("dedupe:over", Retention::Forever).unwrap();
        let key = SigningKey::from_bytes(&[54u8; 32]);

        // 129-char id — past the 128 ceiling. Treated as if absent.
        let huge = "x".repeat(129);
        let mut p = post_params(&key, "dedupe:over", "test", b"a", 1_000);
        p.as_object_mut()
            .unwrap()
            .insert("client_msg_id".into(), json!(huge));

        let r1 = unwrap_success(handle_channel_post_with(&bus, json!(1), &p).await);
        let r2 = unwrap_success(handle_channel_post_with(&bus, json!(2), &p).await);

        // Oversized id is filtered out → both posts append normally.
        assert_ne!(r1["offset"], r2["offset"]);
    }

    // ─── T-2104: substrate primitive 9 slice 2 — channel.subscribe ───
    // ─── include_current_value tests. Each test uses a unique topic ───
    // ─── name to avoid cv_index global-state cross-test interference. ─

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_omit_include_current_value_omits_field() {
        // T-2104 back-compat — subscribe without the new param returns no
        // `current_values` key (pre-slice-2 response shape unchanged).
        let (_d, bus) = tmp_bus();
        bus.create_topic("cv:omit", Retention::Forever).unwrap();
        let resp = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({"topic": "cv:omit", "cursor": 0}),
        )
        .await;
        let v = unwrap_success(resp);
        assert!(v.get("current_values").is_none(), "field must be absent when param omitted");
        assert!(v.get("messages").is_some());
        assert!(v.get("next_cursor").is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_include_current_value_false_omits_field() {
        // T-2104 — explicit false also omits the field (vs empty array).
        let (_d, bus) = tmp_bus();
        bus.create_topic("cv:false", Retention::Forever).unwrap();
        let resp = handle_channel_subscribe_with(
            &bus,
            json!(1),
            &json!({"topic": "cv:false", "cursor": 0, "include_current_value": false}),
        )
        .await;
        let v = unwrap_success(resp);
        assert!(v.get("current_values").is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_include_current_value_true_empty_returns_empty_array() {
        // T-2104 — true on a topic with no cv-tagged posts → present-but-empty.
        let (_d, bus) = tmp_bus();
        bus.create_topic("cv:empty-true", Retention::Forever).unwrap();
        let key = signing_key();
        // Post without cv_key — shouldn't populate the index.
        let p = post_params(&key, "cv:empty-true", "n", b"x", 1_000);
        let _ = handle_channel_post_with(&bus, json!(1), &p).await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "cv:empty-true", "cursor": 0, "include_current_value": true}),
        )
        .await;
        let v = unwrap_success(resp);
        let cvs = v["current_values"].as_array().expect("current_values present");
        assert!(cvs.is_empty(), "no cv_key posts → empty current_values");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_include_current_value_true_one_key() {
        // T-2104 — one cv-tagged post → one current_values entry pointing
        // at the right offset and carrying the envelope inline.
        let (_d, bus) = tmp_bus();
        bus.create_topic("cv:one-key", Retention::Forever).unwrap();
        let key = signing_key();
        let p = post_params_with_meta(
            &key, "cv:one-key", "presence", b"alive", 2_000,
            Some(json!({"cv_key": "agent-alpha"})),
        );
        let post = unwrap_success(handle_channel_post_with(&bus, json!(1), &p).await);
        let offset = post["offset"].as_u64().unwrap();

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(2),
            &json!({"topic": "cv:one-key", "cursor": 99, "include_current_value": true}),
        )
        .await;
        let v = unwrap_success(resp);
        let cvs = v["current_values"].as_array().expect("current_values present");
        assert_eq!(cvs.len(), 1);
        assert_eq!(cvs[0]["cv_key"].as_str(), Some("agent-alpha"));
        assert_eq!(cvs[0]["offset"].as_u64(), Some(offset));
        // Envelope inline + signal that envelope_to_json wrapped it.
        let msg = &cvs[0]["msg"];
        assert!(msg.is_object(), "msg should be the JSON-encoded envelope");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(msg["payload_b64"].as_str().unwrap())
            .unwrap();
        assert_eq!(&bytes, b"alive");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_include_current_value_multi_key() {
        // T-2104 — distinct cv_keys → distinct entries.
        let (_d, bus) = tmp_bus();
        bus.create_topic("cv:multi", Retention::Forever).unwrap();
        let key = signing_key();

        let p_a = post_params_with_meta(
            &key, "cv:multi", "p", b"a", 3_001,
            Some(json!({"cv_key": "alice"})),
        );
        let p_b = post_params_with_meta(
            &key, "cv:multi", "p", b"b", 3_002,
            Some(json!({"cv_key": "bob"})),
        );
        let _ = handle_channel_post_with(&bus, json!(1), &p_a).await;
        let _ = handle_channel_post_with(&bus, json!(2), &p_b).await;

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(3),
            &json!({"topic": "cv:multi", "cursor": 0, "include_current_value": true}),
        )
        .await;
        let v = unwrap_success(resp);
        let cvs = v["current_values"].as_array().expect("current_values present");
        assert_eq!(cvs.len(), 2);
        let keys: std::collections::HashSet<String> = cvs
            .iter()
            .map(|e| e["cv_key"].as_str().unwrap().to_string())
            .collect();
        assert!(keys.contains("alice"));
        assert!(keys.contains("bob"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn subscribe_include_current_value_returns_latest_after_update() {
        // T-2104 — last-write-wins. Two posts with cv_key=alice; the
        // second wins and the cv entry points at offset 1 (the second).
        let (_d, bus) = tmp_bus();
        bus.create_topic("cv:latest", Retention::Forever).unwrap();
        let key = signing_key();

        let p1 = post_params_with_meta(
            &key, "cv:latest", "p", b"v1", 4_001,
            Some(json!({"cv_key": "alice"})),
        );
        let p2 = post_params_with_meta(
            &key, "cv:latest", "p", b"v2", 4_002,
            Some(json!({"cv_key": "alice"})),
        );
        let post1 = unwrap_success(handle_channel_post_with(&bus, json!(1), &p1).await);
        let post2 = unwrap_success(handle_channel_post_with(&bus, json!(2), &p2).await);
        let off1 = post1["offset"].as_u64().unwrap();
        let off2 = post2["offset"].as_u64().unwrap();
        assert!(off2 > off1);

        let resp = handle_channel_subscribe_with(
            &bus,
            json!(3),
            &json!({"topic": "cv:latest", "cursor": 0, "include_current_value": true}),
        )
        .await;
        let v = unwrap_success(resp);
        let cvs = v["current_values"].as_array().expect("current_values present");
        assert_eq!(cvs.len(), 1, "single distinct cv_key → single entry");
        assert_eq!(cvs[0]["cv_key"].as_str(), Some("alice"));
        assert_eq!(cvs[0]["offset"].as_u64(), Some(off2), "latest offset wins");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(cvs[0]["msg"]["payload_b64"].as_str().unwrap())
            .unwrap();
        assert_eq!(&bytes, b"v2", "payload reflects the latest post");
    }

    // ── T-2106 — channel.cv_keys handler tests ──

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cv_keys_empty_topic_returns_zero_entries() {
        // T-2106 — healthy topic with no cv-tagged posts → count: 0, entries: [].
        // Not an error. Per ADR §6 #9 — empty cv_index is valid steady state.
        let (_d, bus) = tmp_bus();
        bus.create_topic("cvk:empty", Retention::Forever).unwrap();
        let resp = handle_channel_cv_keys_with(
            &bus, json!(1), &json!({"topic": "cvk:empty"}),
        ).await;
        let v = unwrap_success(resp);
        assert_eq!(v["ok"].as_bool(), Some(true));
        assert_eq!(v["topic"].as_str(), Some("cvk:empty"));
        assert_eq!(v["count"].as_u64(), Some(0));
        assert!(v["entries"].as_array().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cv_keys_one_key_after_post() {
        // T-2106 — cv-tagged post populates cv_index → cv_keys returns it.
        let (_d, bus) = tmp_bus();
        bus.create_topic("cvk:one", Retention::Forever).unwrap();
        let key = signing_key();
        let p = post_params_with_meta(
            &key, "cvk:one", "presence", b"x", 1_000,
            Some(json!({"cv_key": "agent-charlie"})),
        );
        let post = unwrap_success(handle_channel_post_with(&bus, json!(1), &p).await);
        let off = post["offset"].as_u64().unwrap();
        let resp = handle_channel_cv_keys_with(
            &bus, json!(2), &json!({"topic": "cvk:one"}),
        ).await;
        let v = unwrap_success(resp);
        assert_eq!(v["count"].as_u64(), Some(1));
        let entries = v["entries"].as_array().unwrap();
        assert_eq!(entries[0]["cv_key"].as_str(), Some("agent-charlie"));
        assert_eq!(entries[0]["offset"].as_u64(), Some(off));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cv_keys_multi_returns_sorted() {
        // T-2106 — multiple distinct cv_keys → entries sorted by cv_key
        // (ASCII order; predictable rendering).
        let (_d, bus) = tmp_bus();
        bus.create_topic("cvk:sort", Retention::Forever).unwrap();
        let key = signing_key();
        for (i, ck) in ["zebra", "alpha", "mango"].iter().enumerate() {
            let p = post_params_with_meta(
                &key, "cvk:sort", "n", b"x", 4_000 + i as i64,
                Some(json!({"cv_key": *ck})),
            );
            let _ = handle_channel_post_with(&bus, json!(i as u64), &p).await;
        }
        let resp = handle_channel_cv_keys_with(
            &bus, json!(99), &json!({"topic": "cvk:sort"}),
        ).await;
        let v = unwrap_success(resp);
        assert_eq!(v["count"].as_u64(), Some(3));
        let entries = v["entries"].as_array().unwrap();
        let keys: Vec<&str> = entries.iter().map(|e| e["cv_key"].as_str().unwrap()).collect();
        assert_eq!(keys, vec!["alpha", "mango", "zebra"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cv_keys_unknown_topic_returns_error() {
        // T-2106 — missing topic → CHANNEL_TOPIC_UNKNOWN (-32013).
        // Mirror of claims_summary error shape.
        let (_d, bus) = tmp_bus();
        let resp = handle_channel_cv_keys_with(
            &bus, json!(1), &json!({"topic": "cvk:does-not-exist"}),
        ).await;
        match resp {
            RpcResponse::Error(e) => {
                assert_eq!(e.error.code, error_code::CHANNEL_TOPIC_UNKNOWN);
            }
            _ => panic!("expected CHANNEL_TOPIC_UNKNOWN error response"),
        }
    }
}

#[cfg(test)]
mod observed_addr_tests {
    use super::apply_observed_addr;
    use std::collections::BTreeMap;

    // TCP caller (peer_addr = Some): the attested address is stamped in.
    #[test]
    fn observed_addr_stamped_when_attested() {
        let mut md: BTreeMap<String, String> = BTreeMap::new();
        apply_observed_addr(&mut md, Some("192.168.10.141:51234"));
        assert_eq!(md.get("observed_addr").map(String::as_str), Some("192.168.10.141:51234"));
    }

    // TCP caller that tries to FORGE observed_addr: the attested value overwrites it.
    #[test]
    fn observed_addr_overwrites_client_forged_value() {
        let mut md: BTreeMap<String, String> = BTreeMap::new();
        md.insert("observed_addr".to_string(), "10.0.0.1:9999".to_string()); // forged
        md.insert("conversation_id".to_string(), "cid-1".to_string());
        apply_observed_addr(&mut md, Some("192.168.10.141:51234"));
        // attested wins…
        assert_eq!(md.get("observed_addr").map(String::as_str), Some("192.168.10.141:51234"));
        // …and unrelated metadata is untouched.
        assert_eq!(md.get("conversation_id").map(String::as_str), Some("cid-1"));
    }

    // Unix-socket / local caller (peer_addr = None): any client value is STRIPPED,
    // so an un-attested observed_addr can never masquerade as attested.
    #[test]
    fn observed_addr_stripped_when_not_attestable() {
        let mut md: BTreeMap<String, String> = BTreeMap::new();
        md.insert("observed_addr".to_string(), "10.0.0.1:9999".to_string()); // client-supplied
        md.insert("addr".to_string(), "self-reported:9100".to_string());
        apply_observed_addr(&mut md, None);
        assert!(md.get("observed_addr").is_none(), "un-attested observed_addr must be stripped");
        // self-reported addr (T-2293) is a different key and stays.
        assert_eq!(md.get("addr").map(String::as_str), Some("self-reported:9100"));
    }
}
