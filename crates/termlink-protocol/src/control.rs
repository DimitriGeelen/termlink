use serde::{Deserialize, Serialize};

/// Control plane message methods.
pub mod method {
    pub const SESSION_REGISTER: &str = "session.register";
    pub const SESSION_DEREGISTER: &str = "session.deregister";
    pub const SESSION_DISCOVER: &str = "session.discover";
    pub const SESSION_HEARTBEAT: &str = "session.heartbeat";
    pub const COMMAND_EXECUTE: &str = "command.execute";
    pub const COMMAND_INJECT: &str = "command.inject";
    pub const COMMAND_SIGNAL: &str = "command.signal";
    pub const QUERY_STATUS: &str = "query.status";
    pub const QUERY_OUTPUT: &str = "query.output";
    pub const QUERY_CAPABILITIES: &str = "query.capabilities";
    pub const EVENT_STATE_CHANGE: &str = "event.state_change";
    pub const EVENT_ERROR: &str = "event.error";
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
}
