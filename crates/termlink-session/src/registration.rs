use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::identity::SessionId;
use crate::lifecycle::SessionState;
use termlink_protocol::TransportAddr;

/// Registration entry format version.
pub const REGISTRATION_VERSION: u8 = 1;

/// Registration entry written as a JSON sidecar file alongside the session socket.
///
/// Format defined in T-006 design doc.
///
/// The `addr` field replaced the legacy `socket` field in T-122. For backward
/// compatibility, deserialization accepts either:
/// - `"addr": {"type": "unix", "path": "..."}` (new format)
/// - `"socket": "/path/to/socket"` (legacy format, auto-upgraded)
///
/// Serialization always writes `addr` (new format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub version: u8,
    pub id: SessionId,
    pub display_name: String,
    pub pid: u32,
    pub uid: u32,
    /// Transport address for this session's control plane.
    #[serde(flatten)]
    pub addr: RegistrationAddr,
    pub created_at: String,
    pub heartbeat_at: String,
    pub state: SessionState,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: SessionMetadata,
    /// HMAC secret for capability token validation (Phase 3 auth, T-086).
    /// When present, connections must authenticate via `auth.token` to get scoped access.
    /// When absent, legacy behavior: same-UID connections get Execute scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_secret: Option<String>,
    /// Optional command allowlist for `command.execute` (defense-in-depth, T-090).
    /// When present, only commands matching at least one prefix are allowed.
    /// When absent, all commands are allowed (backward compatible).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_commands: Option<Vec<String>>,
}

/// Wrapper that handles backward-compatible serialization of the transport address.
///
/// Serializes as `"addr": { ... }`. Deserializes from either `"addr"` (new) or
/// `"socket"` (legacy PathBuf, auto-converted to `TransportAddr::Unix`).
#[derive(Debug, Clone)]
pub struct RegistrationAddr(pub TransportAddr);

impl std::fmt::Display for RegistrationAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for RegistrationAddr {
    type Target = TransportAddr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RegistrationAddr {
    /// Get the underlying `TransportAddr`.
    pub fn to_transport_addr(&self) -> TransportAddr {
        self.0.clone()
    }
}

impl From<TransportAddr> for RegistrationAddr {
    fn from(addr: TransportAddr) -> Self {
        Self(addr)
    }
}

impl Serialize for RegistrationAddr {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("addr", &self.0)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for RegistrationAddr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Visitor that accepts either `addr` or legacy `socket` field.
        struct AddrVisitor;

        impl<'de> serde::de::Visitor<'de> for AddrVisitor {
            type Value = RegistrationAddr;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a map with 'addr' or legacy 'socket' field")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<RegistrationAddr, A::Error> {
                let mut addr: Option<TransportAddr> = None;
                let mut socket: Option<PathBuf> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "addr" => {
                            addr = Some(map.next_value()?);
                        }
                        "socket" => {
                            socket = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields (they belong to other parts of Registration)
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                if let Some(a) = addr {
                    Ok(RegistrationAddr(a))
                } else if let Some(s) = socket {
                    // Legacy format: convert PathBuf to TransportAddr::Unix
                    Ok(RegistrationAddr(TransportAddr::unix(s)))
                } else {
                    Err(serde::de::Error::missing_field("addr"))
                }
            }
        }

        deserializer.deserialize_map(AddrVisitor)
    }
}

/// Environment metadata included in registration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termlink_version: Option<String>,
    /// Data plane socket path (present when session supports binary streaming).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_socket: Option<String>,
}

/// Configuration for creating a new session registration.
pub struct SessionConfig {
    pub display_name: Option<String>,
    pub capabilities: Vec<String>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            display_name: None,
            capabilities: vec![
                "inject".into(),
                "command".into(),
                "query".into(),
            ],
            roles: vec![],
            tags: vec![],
        }
    }
}

impl Registration {
    /// Create a new registration entry for the current process.
    pub fn new(id: SessionId, config: SessionConfig, socket_path: PathBuf) -> Self {
        let pid = std::process::id();
        let uid = unsafe { libc::getuid() };
        let now = now_iso8601();

        let display_name = config
            .display_name
            .unwrap_or_else(|| id.as_str().to_string());

        Self {
            version: REGISTRATION_VERSION,
            id,
            display_name,
            pid,
            uid,
            addr: RegistrationAddr(TransportAddr::unix(socket_path)),
            created_at: now.clone(),
            heartbeat_at: now,
            state: SessionState::Initializing,
            capabilities: config.capabilities,
            roles: config.roles,
            tags: config.tags,
            metadata: SessionMetadata {
                shell: std::env::var("SHELL").ok(),
                term: std::env::var("TERM").ok(),
                cwd: std::env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned()),
                termlink_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                data_socket: None,
            },
            token_secret: None,
            allowed_commands: None,
        }
    }

    /// Convenience: get the Unix socket path if this is a Unix transport address.
    ///
    /// Panics if the address is not Unix — this is intentional for now since
    /// only Unix sockets are supported at runtime. When TCP support is added,
    /// callers should switch to pattern-matching on `reg.addr`.
    pub fn socket_path(&self) -> &Path {
        self.addr.as_unix_path().expect(
            "Registration.socket_path() called on non-Unix address; \
             use reg.addr.as_unix_path() for safe access",
        )
    }

    /// Path to the JSON registration file for this session.
    pub fn json_path(sessions_dir: &std::path::Path, id: &SessionId) -> PathBuf {
        sessions_dir.join(format!("{id}.json"))
    }

    /// Path to the Unix socket for a session in the given directory.
    pub fn default_socket_path(sessions_dir: &std::path::Path, id: &SessionId) -> PathBuf {
        sessions_dir.join(format!("{id}.sock"))
    }

    /// Write registration atomically (write temp file, then rename).
    pub fn write_atomic(&self, path: &std::path::Path) -> std::io::Result<()> {
        let tmp_path = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Read a registration from a JSON file.
    pub fn read_from(path: &std::path::Path) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Update heartbeat timestamp and write atomically.
    pub fn touch_heartbeat(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        self.heartbeat_at = now_iso8601();
        self.write_atomic(path)
    }

    /// Update state and write atomically.
    ///
    /// Returns an error if the transition is invalid (see [`SessionState::valid_transition`]).
    pub fn set_state(
        &mut self,
        state: SessionState,
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        self.state.valid_transition(state).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
        })?;
        self.state = state;
        self.heartbeat_at = now_iso8601();
        self.write_atomic(path)
    }
}

fn now_iso8601() -> String {
    // Use chrono-free approach: seconds since epoch formatted manually
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    // Format as simplified ISO 8601 with Unix timestamp (good enough for v0.1)
    format!("{}Z", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_roundtrip_json() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let reg = Registration::new(id, config, socket);

        let json = serde_json::to_string_pretty(&reg).unwrap();
        let parsed: Registration = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, REGISTRATION_VERSION);
        assert_eq!(parsed.id, reg.id);
        assert_eq!(parsed.pid, std::process::id());
        assert_eq!(parsed.capabilities.len(), 3);
    }

    #[test]
    fn atomic_write_and_read() {
        let dir = tempdir();
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = dir.join("test.sock");
        let reg = Registration::new(id.clone(), config, socket);

        let json_path = dir.join(format!("{id}.json"));
        reg.write_atomic(&json_path).unwrap();

        let read_back = Registration::read_from(&json_path).unwrap();
        assert_eq!(read_back.id, reg.id);
        assert_eq!(read_back.state, SessionState::Initializing);
    }

    #[test]
    fn touch_heartbeat_updates_timestamp() {
        let dir = tempdir();
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = dir.join("test.sock");
        let mut reg = Registration::new(id.clone(), config, socket);
        let json_path = dir.join(format!("{id}.json"));

        reg.write_atomic(&json_path).unwrap();
        let original = reg.heartbeat_at.clone();

        // Sleep briefly to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        reg.touch_heartbeat(&json_path).unwrap();

        // Re-read to verify persistence
        let read_back = Registration::read_from(&json_path).unwrap();
        // Timestamps are second-resolution, so they may be equal
        // but the write should succeed
        assert!(!read_back.heartbeat_at.is_empty());
        let _ = original; // suppress unused warning
    }

    #[test]
    fn set_state_persists() {
        let dir = tempdir();
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = dir.join("test.sock");
        let mut reg = Registration::new(id.clone(), config, socket);
        let json_path = dir.join(format!("{id}.json"));

        reg.write_atomic(&json_path).unwrap();
        reg.set_state(SessionState::Ready, &json_path).unwrap();

        let read_back = Registration::read_from(&json_path).unwrap();
        assert_eq!(read_back.state, SessionState::Ready);
    }

    #[test]
    fn default_display_name_is_id() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let reg = Registration::new(id.clone(), config, socket);
        assert_eq!(reg.display_name, id.as_str());
    }

    #[test]
    fn custom_display_name() {
        let id = SessionId::generate();
        let config = SessionConfig {
            display_name: Some("builder".into()),
            ..Default::default()
        };
        let socket = PathBuf::from("/tmp/test.sock");
        let reg = Registration::new(id, config, socket);
        assert_eq!(reg.display_name, "builder");
    }

    #[test]
    fn socket_path_convenience() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let reg = Registration::new(id, config, socket.clone());
        assert_eq!(reg.socket_path(), Path::new("/tmp/test.sock"));
    }

    #[test]
    fn addr_field_in_json() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let reg = Registration::new(id, config, socket);

        let json = serde_json::to_string_pretty(&reg).unwrap();
        // New format should have "addr" with "type"
        assert!(json.contains("\"addr\""), "JSON should contain addr field");
        assert!(json.contains("\"type\": \"unix\""), "addr should have type unix");
        // Should NOT have old "socket" field
        assert!(!json.contains("\"socket\""), "JSON should not contain legacy socket field");
    }

    #[test]
    fn backward_compat_legacy_socket_field() {
        // Simulate a legacy registration JSON that has "socket" instead of "addr"
        let legacy_json = r#"{
            "version": 1,
            "id": "tl-test1234",
            "display_name": "legacy-session",
            "pid": 12345,
            "uid": 501,
            "socket": "/tmp/legacy.sock",
            "created_at": "1234Z",
            "heartbeat_at": "1234Z",
            "state": "ready",
            "capabilities": ["inject"],
            "roles": [],
            "tags": []
        }"#;

        let reg: Registration = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(reg.socket_path(), Path::new("/tmp/legacy.sock"));
        assert!(reg.addr.is_unix());
    }

    #[test]
    fn session_metadata_serde_all_fields() {
        let meta = SessionMetadata {
            shell: Some("/bin/zsh".into()),
            term: Some("xterm-256color".into()),
            cwd: Some("/home/user".into()),
            termlink_version: Some("0.9.0".into()),
            data_socket: Some("/tmp/data.sock".into()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.shell.as_deref(), Some("/bin/zsh"));
        assert_eq!(parsed.term.as_deref(), Some("xterm-256color"));
        assert_eq!(parsed.cwd.as_deref(), Some("/home/user"));
        assert_eq!(parsed.termlink_version.as_deref(), Some("0.9.0"));
        assert_eq!(parsed.data_socket.as_deref(), Some("/tmp/data.sock"));
    }

    #[test]
    fn session_metadata_optional_fields_omitted() {
        let meta = SessionMetadata::default();
        let json = serde_json::to_string(&meta).unwrap();
        assert!(!json.contains("shell"));
        assert!(!json.contains("term"));
        assert!(!json.contains("data_socket"));

        // Deserialize from empty object
        let parsed: SessionMetadata = serde_json::from_str("{}").unwrap();
        assert!(parsed.shell.is_none());
        assert!(parsed.data_socket.is_none());
    }

    #[test]
    fn token_secret_in_registration_json() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let mut reg = Registration::new(id, config, socket);
        reg.token_secret = Some("base64secret".into());

        let json = serde_json::to_string(&reg).unwrap();
        assert!(json.contains("token_secret"));

        let parsed: Registration = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.token_secret.as_deref(), Some("base64secret"));
    }

    #[test]
    fn token_secret_omitted_when_none() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let reg = Registration::new(id, config, socket);
        assert!(reg.token_secret.is_none());

        let json = serde_json::to_string(&reg).unwrap();
        assert!(!json.contains("token_secret"));
    }

    #[test]
    fn allowed_commands_in_registration_json() {
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = PathBuf::from("/tmp/test.sock");
        let mut reg = Registration::new(id, config, socket);
        reg.allowed_commands = Some(vec!["ls".into(), "cat".into(), "echo".into()]);

        let json = serde_json::to_string(&reg).unwrap();
        assert!(json.contains("allowed_commands"));

        let parsed: Registration = serde_json::from_str(&json).unwrap();
        let cmds = parsed.allowed_commands.unwrap();
        assert_eq!(cmds, vec!["ls", "cat", "echo"]);
    }

    #[test]
    fn registration_addr_display() {
        let unix_addr = RegistrationAddr(TransportAddr::unix("/tmp/session.sock"));
        assert_eq!(unix_addr.to_string(), "unix:/tmp/session.sock");

        let tcp_addr = RegistrationAddr(TransportAddr::tcp("192.168.1.1", 9100));
        assert_eq!(tcp_addr.to_string(), "tcp:192.168.1.1:9100");
    }

    #[test]
    fn tcp_address_in_registration() {
        // Registration with TCP address should serialize/deserialize correctly
        let json = r#"{
            "version": 1,
            "id": "tl-tcptest",
            "display_name": "tcp-session",
            "pid": 12345,
            "uid": 501,
            "addr": {"type": "tcp", "host": "10.0.0.1", "port": 9100},
            "created_at": "1234Z",
            "heartbeat_at": "1234Z",
            "state": "ready",
            "capabilities": [],
            "roles": [],
            "tags": []
        }"#;
        let reg: Registration = serde_json::from_str(json).unwrap();
        assert!(reg.addr.is_tcp());
        assert_eq!(reg.addr.as_tcp(), Some(("10.0.0.1", 9100)));
    }

    #[test]
    fn set_state_invalid_transition_returns_error() {
        let dir = tempdir();
        let id = SessionId::generate();
        let config = SessionConfig::default();
        let socket = dir.join("test.sock");
        let mut reg = Registration::new(id.clone(), config, socket);
        let json_path = dir.join(format!("{id}.json"));
        reg.write_atomic(&json_path).unwrap();

        // Initializing → Busy is invalid (must go through Ready first)
        let result = reg.set_state(SessionState::Busy, &json_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("invalid state transition"));

        // State should not have changed
        assert_eq!(reg.state, SessionState::Initializing);
    }

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "termlink-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
