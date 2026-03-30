use serde::{Deserialize, Serialize};

/// Control plane message methods.
pub mod method {
    pub const SESSION_REGISTER: &str = "session.register";
    pub const SESSION_DEREGISTER: &str = "session.deregister";
    pub const SESSION_DISCOVER: &str = "session.discover";
    pub const SESSION_HEARTBEAT: &str = "session.heartbeat";
    pub const SESSION_UPDATE: &str = "session.update";
    pub const COMMAND_EXECUTE: &str = "command.execute";
    pub const COMMAND_INJECT: &str = "command.inject";
    pub const COMMAND_SIGNAL: &str = "command.signal";
    pub const COMMAND_RESIZE: &str = "command.resize";
    pub const QUERY_STATUS: &str = "query.status";
    pub const QUERY_OUTPUT: &str = "query.output";
    pub const QUERY_CAPABILITIES: &str = "query.capabilities";
    pub const EVENT_EMIT: &str = "event.emit";
    pub const EVENT_POLL: &str = "event.poll";
    pub const EVENT_TOPICS: &str = "event.topics";
    pub const EVENT_STATE_CHANGE: &str = "event.state_change";
    pub const EVENT_ERROR: &str = "event.error";
    pub const EVENT_BROADCAST: &str = "event.broadcast";
    pub const EVENT_COLLECT: &str = "event.collect";
    pub const EVENT_EMIT_TO: &str = "event.emit_to";
    pub const EVENT_SUBSCRIBE: &str = "event.subscribe";
    pub const KV_SET: &str = "kv.set";
    pub const KV_GET: &str = "kv.get";
    pub const KV_LIST: &str = "kv.list";
    pub const KV_DELETE: &str = "kv.delete";
    pub const AUTH_TOKEN: &str = "auth.token";
    pub const HUB_AUTH: &str = "hub.auth";
    pub const PTY_MODE: &str = "pty.mode";
    pub const ORCHESTRATOR_ROUTE: &str = "orchestrator.route";
    pub const ORCHESTRATOR_BYPASS_STATUS: &str = "orchestrator.bypass_status";
    pub const ORCHESTRATOR_BYPASS_INVALIDATE: &str = "orchestrator.bypass_invalidate";
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
}
