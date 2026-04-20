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
}
