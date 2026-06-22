use serde::{Deserialize, Serialize};

/// Control plane message methods.
///
/// # Resilience-tier taxonomy (T-1133, from T-1071 GO)
///
/// Every method is tagged **Tier-A** or **Tier-B** to make protocol-skew
/// behavior predictable across the fleet:
///
/// - **Tier-A — opaque / drift-tolerant.** Payload is a raw JSON value, a
///   string, or a free-form envelope. Adding fields to the payload on one
///   side does not cause the other side to reject the message. Safe across
///   version skew without coordination. Examples: `event.emit`,
///   `event.broadcast`, `kv.set`. Use these for best-effort signaling and
///   any cross-version fan-out.
///
/// - **Tier-B — typed / drift-fragile.** Payload deserializes into a named
///   struct with required fields. Adding or renaming fields on one side can
///   cause `serde` failures on older peers (opaque "invalid type" errors).
///   Requires coordinated rollout and protocol_version bumps. Examples:
///   `command.execute`, `command.inject`, `session.update`.
///
/// Fleet observability (T-1132) can flag a fleet where Tier-B methods are in
/// flight across version diversity. `event.broadcast`'s accidental
/// drift-tolerance (T-1071) is the prototype Tier-A property — this
/// taxonomy promotes it from "happy accident" to "documented design tier".
pub mod method {
    /// Tier-B — typed registration payload with session metadata and capabilities.
    pub const SESSION_REGISTER: &str = "session.register";
    /// Tier-B — typed session ID + reason.
    pub const SESSION_DEREGISTER: &str = "session.deregister";
    /// Tier-B — typed filter struct (tags, caps, name) and typed SessionInfo results.
    pub const SESSION_DISCOVER: &str = "session.discover";
    /// Tier-A — opaque liveness ping; no structured payload beyond session id.
    pub const SESSION_HEARTBEAT: &str = "session.heartbeat";
    /// Tier-B — typed update (tags, focus, metadata); adding fields is a breaking change without a protocol bump.
    pub const SESSION_UPDATE: &str = "session.update";
    /// T-1299 / T-1297 — Tier-A — caller asks the hub "who am I on this bus?".
    /// Optional params hint disambiguators: `{ session_id?, display_name? }`.
    /// When neither is provided, hub returns the candidate list (one entry per live session)
    /// and the caller picks. Older hubs that don't implement this return JSON-RPC
    /// `Method not found` (-32601) — a clean upgrade signal.
    pub const SESSION_WHOAMI: &str = "session.whoami";
    /// Tier-B — typed execute params (command, args, env, timeout) and typed result.
    pub const COMMAND_EXECUTE: &str = "command.execute";
    /// Tier-B — typed injection params (text, session, options).
    pub const COMMAND_INJECT: &str = "command.inject";
    /// Tier-B — typed signal name + target.
    pub const COMMAND_SIGNAL: &str = "command.signal";
    /// Tier-B — typed resize dimensions (rows, cols).
    pub const COMMAND_RESIZE: &str = "command.resize";
    /// Tier-B — typed StatusReport response.
    pub const QUERY_STATUS: &str = "query.status";
    /// Tier-B — typed OutputResponse (lines, range, encoding).
    pub const QUERY_OUTPUT: &str = "query.output";
    /// Tier-B — typed Capabilities response (protocol_version, feature flags).
    pub const QUERY_CAPABILITIES: &str = "query.capabilities";
    /// Tier-A — opaque payload (topic + arbitrary JSON value). Drift-tolerant by construction.
    pub const EVENT_EMIT: &str = "event.emit";
    /// Tier-B — typed filter (topic, after, limit) and typed event envelope list.
    pub const EVENT_POLL: &str = "event.poll";
    /// Tier-B — typed topic list response.
    pub const EVENT_TOPICS: &str = "event.topics";
    /// Tier-A — opaque transition payload; receivers ignore unknown fields.
    pub const EVENT_STATE_CHANGE: &str = "event.state_change";
    /// Tier-A — opaque error descriptor; best-effort delivery.
    pub const EVENT_ERROR: &str = "event.error";
    /// Tier-A — opaque fan-out payload. Prototype Tier-A case: drift-tolerant across version skew (T-1071).
    pub const EVENT_BROADCAST: &str = "event.broadcast";
    /// Tier-B — typed collector semantics (tag filter, timeout) and typed result bundle.
    pub const EVENT_COLLECT: &str = "event.collect";
    /// Tier-A — opaque payload addressed to a single session.
    pub const EVENT_EMIT_TO: &str = "event.emit_to";
    /// Tier-B — typed subscription filter + delivery mode.
    pub const EVENT_SUBSCRIBE: &str = "event.subscribe";
    /// Tier-A — key is a string; value is an opaque JSON value.
    pub const KV_SET: &str = "kv.set";
    /// Tier-A — returns the opaque value unchanged.
    pub const KV_GET: &str = "kv.get";
    /// Tier-B — typed list response (key array, prefix filter).
    pub const KV_LIST: &str = "kv.list";
    /// Tier-A — key-only delete; no structured payload.
    pub const KV_DELETE: &str = "kv.delete";
    /// Tier-B — typed token-mint request (scope, ttl) and typed response.
    pub const AUTH_TOKEN: &str = "auth.token";
    /// Tier-B — typed hub handshake (nonce, signature).
    pub const HUB_AUTH: &str = "hub.auth";
    /// Tier-A — opaque list of method names supported by this hub, plus
    /// `hub_version` and `protocol_version`. Used by federating clients
    /// (T-1214/T-1215) to decide whether to call `channel.*` methods or
    /// fall back to `event.broadcast` against a stranger lineage peer.
    pub const HUB_CAPABILITIES: &str = "hub.capabilities";
    /// Tier-B — typed pty mode params (raw/cooked, tty flags).
    pub const PTY_MODE: &str = "pty.mode";
    /// Tier-B — typed route descriptor.
    pub const ORCHESTRATOR_ROUTE: &str = "orchestrator.route";
    /// Tier-B — typed status request/response.
    pub const ORCHESTRATOR_BYPASS_STATUS: &str = "orchestrator.bypass_status";
    /// Tier-B — typed invalidation request.
    pub const ORCHESTRATOR_BYPASS_INVALIDATE: &str = "orchestrator.bypass_invalidate";

    // --- T-1160: channel.* bus surface (T-1155 bus). All Tier-A (opaque payload). ---

    /// Tier-A — create a channel (topic) with a retention policy. Idempotent on name.
    /// Params: `{ name, retention: {kind, value?} }`.
    pub const CHANNEL_CREATE: &str = "channel.create";

    /// Tier-A — change the retention policy of an ALREADY-EXISTING topic
    /// (T-2244 / R2a). `channel.create` refuses a policy change on idempotent
    /// re-create; this is the explicit opt-in. Storage-only — the hub does not
    /// sweep as part of this call. Unknown topic returns an error (no stealth
    /// create). Params: `{ name, retention: {kind, value?} }`.
    pub const CHANNEL_SET_RETENTION: &str = "channel.set_retention";

    /// Tier-A — enforce a topic's retention policy NOW, pruning records that
    /// fall outside it (T-2245 / R2b). The explicit trigger for the otherwise
    /// inert retention subsystem: `channel.create` / `channel.set_retention`
    /// only PERSIST a policy; nothing enforces it until this is called (the bus
    /// runs no background sweep thread, per T-1155). Operator- or cron-invoked.
    /// Enforces whatever policy is set (days / messages / latest /
    /// latest_per_cv_key). Unknown topic returns an error. Params: `{ topic }`
    /// → `{ ok, topic, pruned }`.
    pub const CHANNEL_SWEEP: &str = "channel.sweep";

    /// Tier-A — append a signed envelope to a topic.
    /// Params: `{ topic, msg_type, payload_b64, artifact_ref?, ts, sender_id, sender_pubkey_hex, signature_hex }`.
    /// Hub verifies `signature_hex` against `sender_pubkey_hex` over the canonical bytes
    /// before appending — see `channel::canonical_sign_bytes`.
    pub const CHANNEL_POST: &str = "channel.post";

    /// Tier-A — pull messages from a topic starting at `cursor`.
    /// Params: `{ topic, cursor?, limit? }` → `{ messages: [...], next_cursor }`.
    pub const CHANNEL_SUBSCRIBE: &str = "channel.subscribe";

    /// Tier-A — list existing topics (optional prefix filter).
    /// Params: `{ prefix? }` → `{ topics: [{name, last_offset, retention}] }`.
    pub const CHANNEL_LIST: &str = "channel.list";

    /// Tier-A — destructive trim of a topic. `before_offset=Some(N)` removes
    /// records with offset < N; `before_offset=None` removes ALL records.
    /// Affects ALL subscribers. Mirrors legacy `inbox.clear` semantics.
    /// Pairs with `cursor.advance` (forthcoming) for per-subscriber semantics.
    /// Params: `{ topic, before_offset? }` → `{ ok, deleted, topic }`. T-1234 / T-1230a.
    pub const CHANNEL_TRIM: &str = "channel.trim";

    /// T-2029 — exclusive-delivery claim over a topic offset (arc-parallel-substrate Slice 1).
    /// Hub leases `(topic, offset)` to `claimer` for `ttl_ms` (default 30s).
    /// Params: `{ topic, offset, claimer, ttl_ms? }` →
    /// `{ ok, claim_id, claimed_at, claimed_until }`.
    /// Errors: `CHANNEL_TOPIC_UNKNOWN` (-32013), `CLAIM_CONFLICT` (-32015) when
    /// another worker holds an unexpired claim on the same offset. Old hubs
    /// return `MethodNotFound` (-32601).
    pub const CHANNEL_CLAIM: &str = "channel.claim";

    /// T-2030 — extend the lease on a held claim (arc-parallel-substrate Slice 2).
    /// Long-running workers call this before `claimed_until` to retain ownership;
    /// the hub gates on caller-is-claimer AND `claimed_until > now` so an expired
    /// lease cannot be silently renewed (see `CLAIM_EXPIRED`).
    /// Params: `{ claim_id, claimer, additional_ttl_ms? }` →
    /// `{ ok, claim_id, topic, offset, claimer, claimed_at, claimed_until }`.
    /// Errors: `CLAIM_NOT_FOUND` (-32016), `CLAIM_EXPIRED` (-32018) — stale row
    /// is lazily evicted so a follow-up `channel.claim` for the same offset
    /// succeeds — and `CLAIM_NOT_OWNED` (-32017).
    pub const CHANNEL_RENEW: &str = "channel.renew";

    /// T-2029 — release a previously-issued claim (arc-parallel-substrate Slice 1).
    /// `ack=true` advances the claimer's cursor past the claimed offset (work
    /// completed); `ack=false` leaves the cursor unchanged and frees the slot
    /// for another worker (work returned).
    /// Params: `{ claim_id, claimer, ack }` → `{ ok, topic, offset, ack }`.
    /// Errors: `CLAIM_NOT_FOUND` (-32016), `CLAIM_NOT_OWNED` (-32017).
    pub const CHANNEL_RELEASE: &str = "channel.release";

    /// T-2044 — operator-Tier-0 force release of a held claim
    /// (arc-parallel-substrate Slice 11). Bypasses `channel.release`'s
    /// `claimed_by == claimer` ownership check; semantics match
    /// `release(ack=false)` (cursor untouched, slot freed). For situations
    /// where an operator must clear a stuck claim faster than the natural
    /// TTL expiry path — pairs with `channel.claims_summary --watch` for
    /// detection (Slice 8) and is the operator-intervention companion to
    /// the existing claimer-initiated release.
    /// Params: `{ topic?, claim_id, reason? }` →
    /// `{ ok, claim_id, topic, offset, forced_from, forced_reason }`.
    /// `topic` is accepted for symmetry with surrounding verbs but is not
    /// required — the hub derives it from `claim_id`. `forced_from` echoes
    /// the original claimer for the audit trail; `forced_reason` echoes the
    /// operator-supplied reason (null when omitted).
    /// Errors: `CLAIM_NOT_FOUND` (-32016). Notably does NOT return
    /// `CLAIM_NOT_OWNED` — bypassing that check is the whole point. Old
    /// hubs return `MethodNotFound` (-32601).
    pub const CHANNEL_FORCE_RELEASE: &str = "channel.force_release";

    /// T-2046 — atomic ownership transfer of an existing claim
    /// (T-2021 GO, arc-parallel-substrate primitive #3). Cooperative +
    /// owner-checked counterpart to `CHANNEL_FORCE_RELEASE`: the orchestrator
    /// (or current owner) hands the lease to a chosen worker without
    /// releasing it, eliminating the race window between `release` and the
    /// worker's `claim`. Distinct from `CHANNEL_FORCE_RELEASE` which is the
    /// operator-Tier-0 ownership-bypass verb.
    ///
    /// The lease timestamps (`claimed_at`, `claimed_until`) survive the
    /// transfer — only `claimed_by` mutates. Use `CHANNEL_RENEW` to extend
    /// the lease after transfer if the receiving worker needs more time.
    ///
    /// Params: `{ claim_id, to_owner, by, reason? }` →
    /// `{ ok, claim_id, topic, offset, from_owner, to_owner, claimed_at, claimed_until }`.
    /// `by` is required and must equal the current `claimed_by` — this is
    /// the cooperative path, not a bypass. `reason` is optional audit
    /// metadata, surfaced but not persisted.
    /// Errors: `CLAIM_NOT_FOUND` (-32016), `CLAIM_NOT_OWNED` (-32017) when
    /// `by` ≠ current `claimed_by`, `CLAIM_EXPIRED` (-32018) when the lease
    /// lapsed before transfer (stale row is lazily evicted so the slot
    /// becomes claimable). Old hubs return `MethodNotFound` (-32601).
    pub const CHANNEL_TRANSFER_CLAIM: &str = "channel.transfer_claim";

    /// T-2037 — list current claim rows for `topic` (arc-parallel-substrate
    /// Slice 4). Read-only introspection — answers "what is currently
    /// claimed?" without forcing the caller to attempt a `channel.claim`.
    /// When `include_expired=false` (default), rows whose `claimed_until`
    /// is in the past are filtered out so the response reflects only
    /// live leases. `include_expired=true` is for operator forensics.
    /// Params: `{ topic, include_expired? }` →
    /// `{ ok, topic, claims: [{ claim_id, offset, claimer, claimed_at, claimed_until }, ...] }`.
    /// Errors: `CHANNEL_TOPIC_UNKNOWN` (-32013) — same shape as
    /// `channel.claim`. Old hubs return `MethodNotFound` (-32601).
    pub const CHANNEL_CLAIMS: &str = "channel.claims";

    /// T-2039 — aggregate claim state for `topic` (arc-parallel-substrate
    /// Slice 6). Read-only observability companion to `channel.claims`:
    /// answers "how busy is this topic?" and "is anything stuck?" with a
    /// single O(1) SQL aggregate instead of every-row transfer. The
    /// `oldest_active_age_ms` value is the operator signal — when it
    /// approaches the configured TTL, a worker is either stuck or about
    /// to need `channel.renew`. `next_active_expiry_ms` tells the operator
    /// when the next slot frees up without intervention.
    /// Params: `{ topic }` → `{ ok, topic, active_count, expired_count,
    /// oldest_active_at_ms?, oldest_active_age_ms?, next_active_expiry_ms? }`.
    /// All three `*_ms?` fields are `null` when `active_count == 0`.
    /// Errors: `CHANNEL_TOPIC_UNKNOWN` (-32013) — same shape as
    /// `channel.claims`. Old hubs return `MethodNotFound` (-32601).
    pub const CHANNEL_CLAIMS_SUMMARY: &str = "channel.claims_summary";

    /// T-2106 — operator inspection of the hub-side cv_index for `topic`
    /// (arc-parallel-substrate primitive #9, broadcast-with-replay).
    /// Returns the per-cv_key latest-offset mapping recorded by the hub on
    /// every post carrying `metadata.cv_key` (T-2103). Read-only — no
    /// auth side-effects, no state mutation. The companion to
    /// `channel.subscribe include_current_value=true` (T-2104): subscribe
    /// fetches the cv-indexed ENVELOPES for replay; `channel.cv_keys`
    /// returns just the KEYS + offsets for diagnosis (who's advertising
    /// on this topic? are stale keys still pinned?).
    /// Params: `{ topic }` → `{ ok, topic, count, entries: [{cv_key, offset}, ...] }`.
    /// Entries are sorted by `cv_key` for stable rendering. Empty cv_index
    /// returns `count: 0, entries: []` (NOT an error — a healthy topic
    /// with no cv-tagged posts is a valid state).
    /// Errors: `CHANNEL_TOPIC_UNKNOWN` (-32013) — same shape as
    /// `channel.claims_summary`. Old hubs return `MethodNotFound` (-32601).
    pub const CHANNEL_CV_KEYS: &str = "channel.cv_keys";

    /// T-1329 — server-side aggregation of latest `m.receipt` envelope per sender.
    /// Walks the topic on the hub, keeps only `msg_type=receipt`, picks the latest
    /// (by ts; ties broken by higher up_to), returns a sorted-by-sender list.
    /// Params: `{ topic }` → `{ topic, receipts: [{sender_id, up_to, ts_unix_ms}, ...] }`.
    /// Old hubs return `MethodNotFound` (-32601) — clients fall back to the
    /// existing read-side walker (T-1315).
    pub const CHANNEL_RECEIPTS: &str = "channel.receipts";

    /// T-2045 (T-2020 GO build) — hub-owned idle/busy agent registry, derived.
    /// Server-side derivation: `LIVE(agent-presence)` ∖ `DISTINCT(claims.claimed_by
    /// WHERE claimed_until > now)`. No new persistent state.
    /// Walks the `agent-presence` topic, dedups by `agent_id` keeping latest
    /// heartbeat, filters to LIVE (heartbeat ts newer than 2×interval, default
    /// 60s window), applies optional `role` / `capabilities` predicates, then
    /// excludes every agent_id currently claiming any topic offset. Sorted
    /// by `last_heartbeat_ms` desc (freshest first).
    /// Params: `{ role?: string, capabilities?: [string], limit?: u32 }` →
    /// `{ ok, idle: [{agent_id, last_heartbeat_ms, role, capabilities, hub_id}, ...] }`.
    /// `capabilities` predicate is subset-match: returned agents must advertise
    /// every requested capability. Missing capabilities metadata is empty set
    /// (backward-compat with workers that don't emit the field).
    /// Old hubs return `MethodNotFound` (-32601).
    pub const AGENT_FIND_IDLE: &str = "agent.find_idle";

    /// T-1286 / T-243 — query who has been seen in a multi-turn conversation.
    /// Hub passively tracks `(conversation_id, agent_id) → last_seen_unix_ms`
    /// by observing every successful `channel.post` whose
    /// `metadata.conversation_id` is set; `agent_id` is the post's `sender_id`.
    /// Eviction is implicit — callers compare `last_seen_ms` against now.
    /// Params: `{ conversation_id }` → `{ presences: [{agent_id, last_seen_ms}, ...] }`,
    /// sorted by `agent_id`. Unknown conversation_id returns an empty list (not an error).
    pub const DIALOG_PRESENCE: &str = "dialog.presence";

    // --- T-1248 / T-1164a: artifact blob store. Tier-A. ---

    /// Tier-A — upload bytes into the hub's content-addressed artifact store.
    /// Streaming chunked path: `{ staging_id, offset, chunk_b64, is_final, expected_sha256? }`
    /// → on non-final chunk: `{ ok: true, in_progress: true, bytes_received }`;
    /// on final chunk: `{ ok: true, in_progress: false, sha256, total_bytes }`.
    /// Idempotent on the final sha256 — re-uploading already-stored bytes is a no-op.
    /// Pairs with `channel.post { msg_type: "artifact", artifact_ref: <sha256> }`. T-1164a.
    pub const ARTIFACT_PUT: &str = "artifact.put";

    /// Tier-A — fetch bytes from the hub's content-addressed artifact store.
    /// Streaming chunked path: `{ sha256, offset, max_bytes }` →
    /// `{ chunk_b64, bytes_returned, eof, total_bytes }`. Caller iterates until
    /// `eof: true`. Returns `CHANNEL_TOPIC_UNKNOWN`-style error code if the
    /// sha256 isn't present in the store. T-1164a.
    pub const ARTIFACT_GET: &str = "artifact.get";
}

/// T-1160 channel.* canonical signing bytes.
///
/// Identity separable from transport trust (T-1159) means every `channel.post`
/// carries an ed25519 signature over the *message intent*, not the transport
/// framing. The canonical byte string below is what both sides (sender + hub)
/// feed into the signer / verifier. Fields are length-prefixed so a later
/// addition cannot retro-validate an old signature.
///
/// Layout (all integers big-endian):
/// ```text
///   u32 len(topic)    | topic bytes
///   u32 len(msg_type) | msg_type bytes
///   u32 len(payload)  | payload bytes
///   u32 len(artifact) | artifact bytes   (empty if absent)
///   i64 ts_unix_ms
/// ```
pub mod channel {
    /// Build the canonical byte vector that is signed and verified for a
    /// `channel.post`. See the module-level doc for the layout.
    pub fn canonical_sign_bytes(
        topic: &str,
        msg_type: &str,
        payload: &[u8],
        artifact_ref: Option<&str>,
        ts_unix_ms: i64,
    ) -> Vec<u8> {
        let artifact = artifact_ref.unwrap_or("").as_bytes();
        let mut buf = Vec::with_capacity(
            4 + topic.len() + 4 + msg_type.len() + 4 + payload.len() + 4 + artifact.len() + 8,
        );
        buf.extend_from_slice(&(topic.len() as u32).to_be_bytes());
        buf.extend_from_slice(topic.as_bytes());
        buf.extend_from_slice(&(msg_type.len() as u32).to_be_bytes());
        buf.extend_from_slice(msg_type.as_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(payload);
        buf.extend_from_slice(&(artifact.len() as u32).to_be_bytes());
        buf.extend_from_slice(artifact);
        buf.extend_from_slice(&ts_unix_ms.to_be_bytes());
        buf
    }
}

/// TermLink-specific JSON-RPC error codes (in addition to standard -32700..-32603).
pub mod error_code {
    pub const SESSION_NOT_FOUND: i64 = -32001;
    pub const SESSION_BUSY: i64 = -32002;
    pub const CAPABILITY_NOT_SUPPORTED: i64 = -32003;
    pub const MESSAGE_EXPIRED: i64 = -32004;
    pub const INJECTION_FAILED: i64 = -32005;
    pub const SIGNAL_FAILED: i64 = -32006;
    pub const OUTPUT_UNAVAILABLE: i64 = -32007;
    pub const RATE_LIMITED: i64 = -32008;
    pub const AUTH_REQUIRED: i64 = -32009;
    pub const AUTH_DENIED: i64 = -32010;
    /// Session's declared protocol_version is older than the target method requires.
    /// Data field carries `{declared, required, method}` so the client can act on it.
    /// T-1131 (from T-1071 GO).
    pub const PROTOCOL_VERSION_TOO_OLD: i64 = -32011;
    /// T-1160 `channel.post` signature did not verify against sender pubkey.
    pub const CHANNEL_SIGNATURE_INVALID: i64 = -32012;
    /// T-1160 `channel.post` referenced a topic that has not been created.
    pub const CHANNEL_TOPIC_UNKNOWN: i64 = -32013;
    /// T-1427 `channel.post` `sender_id` did not match the fingerprint
    /// derived from the supplied `sender_pubkey_hex`. Closes the
    /// "identity authoritative" invariant from T-1425 RFC §3.2: a client
    /// cannot legally claim to be a different identity than its signing
    /// key proves. Backward compat: clients already follow the convention
    /// (CLI defaults `sender_id = identity.fingerprint()`); this code
    /// fires only on impostor-style forgery.
    pub const CHANNEL_IDENTITY_MISMATCH: i64 = -32014;
    /// T-2029 `channel.claim` — another worker holds an unexpired claim on the
    /// same `(topic, offset)`. Data field: `{topic, offset}`.
    pub const CLAIM_CONFLICT: i64 = -32015;
    /// T-2029 `channel.release` — claim_id unknown, already released, or
    /// lazily evicted because the TTL lapsed. Data field: `{claim_id}`.
    pub const CLAIM_NOT_FOUND: i64 = -32016;
    /// T-2029 `channel.release` — the caller is not the original claimer.
    /// Data field: `{claim_id, claimed_by, attempted_by}`.
    pub const CLAIM_NOT_OWNED: i64 = -32017;
    /// T-2030 `channel.renew` — lease has already lapsed (`claimed_until <= now`).
    /// The stale row is lazily evicted in the same call so a follow-up
    /// `channel.claim` for the same `(topic, offset)` can succeed.
    /// Data field: `{claim_id}`.
    pub const CLAIM_EXPIRED: i64 = -32018;
    /// T-2048 / T-2028 Track B — hub is at the per-process connection cap
    /// (`TERMLINK_MAX_CONNECTIONS`, default 256). The accept loop writes
    /// one envelope carrying this code and closes the socket — LOUD
    /// refuse per IW-3, never silent drop. Pair with `RATE_LIMITED`
    /// (-32008) for per-sender rate-limit refusals on already-accepted
    /// connections. Data field: `{retry_after_ms: u64}` — best-effort
    /// (~1000ms) since the accept loop cannot predict when a slot frees.
    /// Operators see this as `hub at capacity (retry in 1000ms)` in the
    /// CLI.
    pub const HUB_AT_CAPACITY: i64 = -32019;
}

/// Default `protocol_version` when the field is missing on the wire.
/// Keeps old clients compatible: absent = "speaks v1".
pub fn default_protocol_version() -> u8 {
    1
}

/// If `declared < required`, build a structured `PROTOCOL_VERSION_TOO_OLD` error
/// response. If the declared version is at least `required`, returns `None`.
///
/// Callers pass the RPC request `id` so the error threads back to the originator.
/// The error's `data` field carries `{declared, required, method}` so the client
/// can tell the operator exactly what to upgrade.
///
/// This replaces the opaque serde-parse failure that Tier-B methods throw when
/// a v1 client hits a v2-only field (the KeyEntry class of bug that T-1071
/// identified). DATA_PLANE_VERSION is currently 1 so no method rejects today
/// — the helper lights up on the next protocol bump.
pub fn check_protocol_version(
    id: serde_json::Value,
    declared: u8,
    required: u8,
    method: &str,
) -> Option<crate::jsonrpc::ErrorResponse> {
    if declared >= required {
        return None;
    }
    Some(crate::jsonrpc::ErrorResponse::with_data(
        id,
        error_code::PROTOCOL_VERSION_TOO_OLD,
        &format!(
            "Method {method} requires protocol_version >= {required}, session declared {declared}"
        ),
        serde_json::json!({
            "declared": declared,
            "required": required,
            "method": method,
        }),
    ))
}

/// Common parameters included in every control plane request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonParams {
    pub target: String,
    pub sender: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
}

/// Key injection entry — one element in a `command.inject` keys array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum KeyEntry {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "key")]
    Key(String),
    #[serde(rename = "raw")]
    Raw(String), // base64-encoded
}

/// Session capabilities declared during registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Defaults to 1 when absent on the wire (T-1131 backward-compat).
    #[serde(default = "default_protocol_version")]
    pub protocol_version: u8,
    #[serde(default)]
    pub data_plane: bool,
    #[serde(default)]
    pub compression: Vec<String>,
    #[serde(default)]
    pub max_frame_size: Option<u32>,
    #[serde(default)]
    pub features: Vec<String>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            protocol_version: crate::DATA_PLANE_VERSION,
            data_plane: false,
            compression: vec![],
            max_frame_size: Some(crate::MAX_PAYLOAD_SIZE),
            features: vec![],
        }
    }
}

/// Terminal metadata included in session registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_entry_serialization() {
        let keys = vec![
            KeyEntry::Text("ls -la".into()),
            KeyEntry::Key("Enter".into()),
            KeyEntry::Raw("Aw==".into()), // 0x03 = Ctrl+C in base64
        ];
        let json = serde_json::to_string(&keys).unwrap();
        let parsed: Vec<KeyEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn capabilities_default() {
        let caps = Capabilities::default();
        assert_eq!(caps.protocol_version, 1);
        assert!(!caps.data_plane);
    }

    #[test]
    fn common_params_serde_roundtrip() {
        let params = CommonParams {
            target: "tl-abc123".into(),
            sender: "tl-def456".into(),
            timestamp: "2026-03-30T00:00:00Z".into(),
            correlation_id: Some("corr-1".into()),
            ttl: Some(30),
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: CommonParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.target, "tl-abc123");
        assert_eq!(parsed.sender, "tl-def456");
        assert_eq!(parsed.correlation_id, Some("corr-1".into()));
        assert_eq!(parsed.ttl, Some(30));
    }

    #[test]
    fn common_params_optional_fields_omitted() {
        let params = CommonParams {
            target: "tl-abc".into(),
            sender: "tl-def".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            correlation_id: None,
            ttl: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.contains("correlation_id"));
        assert!(!json.contains("ttl"));

        // Should still deserialize with missing optional fields
        let minimal = r#"{"target":"a","sender":"b","timestamp":"t"}"#;
        let parsed: CommonParams = serde_json::from_str(minimal).unwrap();
        assert_eq!(parsed.correlation_id, None);
        assert_eq!(parsed.ttl, None);
    }

    #[test]
    fn terminal_info_serde_roundtrip() {
        let info = TerminalInfo {
            term: Some("xterm-256color".into()),
            cols: Some(120),
            rows: Some(40),
            shell: Some("/bin/zsh".into()),
            pid: Some(12345),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: TerminalInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.term, Some("xterm-256color".into()));
        assert_eq!(parsed.cols, Some(120));
        assert_eq!(parsed.rows, Some(40));
        assert_eq!(parsed.shell, Some("/bin/zsh".into()));
        assert_eq!(parsed.pid, Some(12345));
    }

    #[test]
    fn terminal_info_all_optional() {
        let info = TerminalInfo {
            term: None,
            cols: None,
            rows: None,
            shell: None,
            pid: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        // All fields should be omitted
        assert_eq!(json, "{}");

        // Deserialize from empty object
        let parsed: TerminalInfo = serde_json::from_str("{}").unwrap();
        assert!(parsed.term.is_none());
        assert!(parsed.cols.is_none());
    }

    #[test]
    fn capabilities_serde_with_features() {
        let caps = Capabilities {
            protocol_version: 1,
            data_plane: true,
            compression: vec!["zstd".into()],
            max_frame_size: Some(1048576),
            features: vec!["stream".into(), "mirror".into()],
        };
        let json = serde_json::to_string(&caps).unwrap();
        let parsed: Capabilities = serde_json::from_str(&json).unwrap();
        assert!(parsed.data_plane);
        assert_eq!(parsed.compression, vec!["zstd"]);
        assert_eq!(parsed.features, vec!["stream", "mirror"]);
        assert_eq!(parsed.max_frame_size, Some(1048576));
    }

    #[test]
    fn key_entry_text_variant() {
        let entry: KeyEntry = serde_json::from_str(r#"{"type":"text","value":"hello"}"#).unwrap();
        assert!(matches!(entry, KeyEntry::Text(ref s) if s == "hello"));
    }

    #[test]
    fn key_entry_key_variant() {
        let entry: KeyEntry = serde_json::from_str(r#"{"type":"key","value":"Enter"}"#).unwrap();
        assert!(matches!(entry, KeyEntry::Key(ref s) if s == "Enter"));
    }

    #[test]
    fn key_entry_raw_variant() {
        let entry: KeyEntry = serde_json::from_str(r#"{"type":"raw","value":"Aw=="}"#).unwrap();
        assert!(matches!(entry, KeyEntry::Raw(ref s) if s == "Aw=="));
    }

    #[test]
    fn method_constants_are_correct() {
        // Verify key method names are stable (breaking changes would break clients)
        assert_eq!(method::COMMAND_EXECUTE, "command.execute");
        assert_eq!(method::QUERY_STATUS, "query.status");
        assert_eq!(method::EVENT_EMIT, "event.emit");
        assert_eq!(method::KV_SET, "kv.set");
        assert_eq!(method::AUTH_TOKEN, "auth.token");
        assert_eq!(method::SESSION_DISCOVER, "session.discover");
        assert_eq!(method::PTY_MODE, "pty.mode");
    }

    // T-1131: protocol_version enforcement

    #[test]
    fn capabilities_missing_protocol_version_defaults_to_1() {
        // Backward-compat: a wire payload without `protocol_version` must deserialize.
        let caps: Capabilities =
            serde_json::from_str(r#"{"data_plane":false}"#).expect("must deserialize");
        assert_eq!(caps.protocol_version, 1);
    }

    #[test]
    fn check_protocol_version_accepts_when_declared_equals_required() {
        assert!(check_protocol_version(serde_json::json!(1), 1, 1, "session.update").is_none());
    }

    #[test]
    fn check_protocol_version_accepts_when_declared_exceeds_required() {
        assert!(check_protocol_version(serde_json::json!(1), 5, 3, "command.execute").is_none());
    }

    #[test]
    fn channel_method_constants_are_stable() {
        assert_eq!(method::CHANNEL_CREATE, "channel.create");
        assert_eq!(method::CHANNEL_POST, "channel.post");
        assert_eq!(method::CHANNEL_SUBSCRIBE, "channel.subscribe");
        assert_eq!(method::CHANNEL_LIST, "channel.list");
        assert_eq!(method::CHANNEL_TRIM, "channel.trim");
        assert_eq!(method::CHANNEL_RECEIPTS, "channel.receipts");
        // T-2029 (arc-parallel-substrate Slice 1).
        assert_eq!(method::CHANNEL_CLAIM, "channel.claim");
        assert_eq!(method::CHANNEL_RELEASE, "channel.release");
        // T-2030 (arc-parallel-substrate Slice 2).
        assert_eq!(method::CHANNEL_RENEW, "channel.renew");
        // T-2037 (arc-parallel-substrate Slice 4).
        assert_eq!(method::CHANNEL_CLAIMS, "channel.claims");
        // T-2039 (arc-parallel-substrate Slice 6).
        assert_eq!(method::CHANNEL_CLAIMS_SUMMARY, "channel.claims_summary");
        // T-2044 (arc-parallel-substrate Slice 11).
        assert_eq!(method::CHANNEL_FORCE_RELEASE, "channel.force_release");
        assert_eq!(method::CHANNEL_TRANSFER_CLAIM, "channel.transfer_claim");
        // T-2045 (T-2020 GO build — idle/busy registry).
        assert_eq!(method::AGENT_FIND_IDLE, "agent.find_idle");
    }

    #[test]
    fn artifact_method_constants_are_stable() {
        assert_eq!(method::ARTIFACT_PUT, "artifact.put");
        assert_eq!(method::ARTIFACT_GET, "artifact.get");
    }

    #[test]
    fn channel_canonical_sign_bytes_is_deterministic_and_length_prefixed() {
        let a = channel::canonical_sign_bytes("t", "note", b"hi", None, 0);
        let b = channel::canonical_sign_bytes("t", "note", b"hi", None, 0);
        assert_eq!(a, b, "same inputs → same bytes");
        // Length prefix of topic "t" (1 byte) is [0,0,0,1].
        assert_eq!(&a[..4], &[0, 0, 0, 1]);
    }

    #[test]
    fn channel_canonical_sign_bytes_distinguishes_artifact_from_empty() {
        let none = channel::canonical_sign_bytes("t", "note", b"hi", None, 0);
        let some = channel::canonical_sign_bytes("t", "note", b"hi", Some(""), 0);
        // Both encode zero-length — they must produce identical bytes.
        assert_eq!(none, some);
        let real = channel::canonical_sign_bytes("t", "note", b"hi", Some("ref://x"), 0);
        assert_ne!(none, real);
    }

    #[test]
    fn check_protocol_version_rejects_when_declared_is_older() {
        let err =
            check_protocol_version(serde_json::json!(42), 1, 2, "command.execute").expect("reject");
        assert_eq!(err.error.code, error_code::PROTOCOL_VERSION_TOO_OLD);
        assert_eq!(err.id, serde_json::json!(42));
        let data = err.error.data.as_ref().expect("data");
        assert_eq!(data["declared"], 1);
        assert_eq!(data["required"], 2);
        assert_eq!(data["method"], "command.execute");
    }

    #[test]
    fn hub_at_capacity_const_is_stable_wire_value() {
        // T-2048 — locks the wire value so clients can pin the code in their
        // error taxonomy. Bumping requires a coordinated client+hub release.
        assert_eq!(error_code::HUB_AT_CAPACITY, -32019);
    }
}
