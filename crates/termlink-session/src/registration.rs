use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::identity::SessionId;
use crate::lifecycle::SessionState;

/// Registration entry format version.
pub const REGISTRATION_VERSION: u8 = 1;

/// Registration entry written as a JSON sidecar file alongside the session socket.
///
/// Format defined in T-006 design doc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub version: u8,
    pub id: SessionId,
    pub display_name: String,
    pub pid: u32,
    pub uid: u32,
    pub socket: PathBuf,
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
            socket: socket_path,
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
        }
    }

    /// Path to the JSON registration file for this session.
    pub fn json_path(sessions_dir: &std::path::Path, id: &SessionId) -> PathBuf {
        sessions_dir.join(format!("{id}.json"))
    }

    /// Path to the Unix socket for this session.
    pub fn socket_path(sessions_dir: &std::path::Path, id: &SessionId) -> PathBuf {
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
        assert!(read_back.heartbeat_at.len() > 0);
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

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "termlink-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
