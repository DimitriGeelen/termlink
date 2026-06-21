use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::agent_identity::Identity;
use crate::identity::SessionId;
use crate::lifecycle::SessionState;
use termlink_protocol::TransportAddr;

/// Registration entry format version.
pub const REGISTRATION_VERSION: u8 = 1;

/// T-1436: best-effort load of the agent identity fingerprint for inclusion
/// in `SessionMetadata`. Resolution order (highest precedence first):
///
///   1. `TERMLINK_IDENTITY_FILE` — explicit file path (T-1700, per-agent
///      override for shared hosts). Created by `termlink register
///      --identity-key <PATH>` for the registering session.
///   2. `TERMLINK_IDENTITY_DIR/identity.key` — base-dir override (T-1159,
///      matches `crates/termlink-cli/src/commands/{identity,channel}.rs`).
///   3. `$HOME/.termlink/identity.key` — host default.
///
/// Returns `None` on any error (missing HOME, IO failure, or absent key
/// file). Registration is read-only here — creation is the responsibility
/// of the CLI (`termlink register` or `termlink identity init`). This keeps
/// `Registration::new` infallible: pre-T-1436 callers and tests that run
/// without an identity continue to work, the field just stays None.
fn load_identity_fingerprint_best_effort() -> Option<String> {
    let key_path = resolve_identity_key_path(|k| std::env::var(k).ok())?;
    if !key_path.exists() {
        return None;
    }
    Identity::load_or_create_from_file(&key_path)
        .ok()
        .map(|id| id.fingerprint().to_string())
}

/// Resolve the identity key file path from the environment. Pure function
/// (env-var lookups go through the supplied closure) so it can be tested
/// without touching global process state. Precedence:
///
///   1. `TERMLINK_IDENTITY_FILE` — explicit file path (T-1700).
///   2. `TERMLINK_IDENTITY_DIR/identity.key` — base-dir override (T-1159).
///   3. `$HOME/.termlink/identity.key` — host default.
///
/// Returns `None` if none of the above can be resolved (e.g. `HOME` unset
/// in a sandboxed test process).
fn resolve_identity_key_path<F>(get_env: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(file) = get_env("TERMLINK_IDENTITY_FILE") {
        return Some(PathBuf::from(file));
    }
    if let Some(dir) = get_env("TERMLINK_IDENTITY_DIR") {
        return Some(crate::agent_identity::identity_path(Path::new(&dir)));
    }
    let home = get_env("HOME")?;
    Some(crate::agent_identity::identity_path(
        &PathBuf::from(home).join(".termlink"),
    ))
}

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
    /// T-1436: hex sha-256 of the agent's ed25519 public key. Same identifier
    /// used as `sender_id` on channel.post envelopes and as the `<a>`/`<b>`
    /// halves of canonical `dm:<a>:<b>` topic names. Present when the session
    /// could load/create an identity at registration time; absent for
    /// pre-T-1436 registrations or when identity loading failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_fingerprint: Option<String>,
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
                // T-1436: best-effort identity fingerprint. Silently None on
                // any failure (no HOME, IO error, etc.) so registration in
                // unprivileged or sandboxed contexts doesn't fail.
                identity_fingerprint: load_identity_fingerprint_best_effort(),
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

    // T-1700: precedence of TERMLINK_IDENTITY_FILE > TERMLINK_IDENTITY_DIR > HOME.
    #[test]
    fn resolve_identity_key_path_prefers_explicit_file() {
        let path = resolve_identity_key_path(|k| match k {
            "TERMLINK_IDENTITY_FILE" => Some("/tmp/agent-a.key".to_string()),
            "TERMLINK_IDENTITY_DIR" => Some("/should/not/be/used".to_string()),
            "HOME" => Some("/should/not/be/used".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(path, PathBuf::from("/tmp/agent-a.key"));
    }

    #[test]
    fn resolve_identity_key_path_falls_back_to_dir() {
        let path = resolve_identity_key_path(|k| match k {
            "TERMLINK_IDENTITY_FILE" => None,
            "TERMLINK_IDENTITY_DIR" => Some("/etc/termlink".to_string()),
            "HOME" => Some("/should/not/be/used".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(path, PathBuf::from("/etc/termlink/identity.key"));
    }

    #[test]
    fn resolve_identity_key_path_falls_back_to_home() {
        let path = resolve_identity_key_path(|k| match k {
            "HOME" => Some("/root".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(path, PathBuf::from("/root/.termlink/identity.key"));
    }

    #[test]
    fn resolve_identity_key_path_none_without_home() {
        let path = resolve_identity_key_path(|_| None);
        assert!(path.is_none());
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

    /// T-2230 regression: the heartbeat timestamp must STRICTLY advance when
    /// touched. The original bug was that `termlink register` never called
    /// touch_heartbeat at all, so `heartbeat_at` stayed frozen at registration
    /// time forever (ring20 "frozen husk" RCA, T-2229). The pre-existing
    /// `touch_heartbeat_persists` test tolerated an unchanged timestamp
    /// ("may be equal"), so it could not catch the freeze. This one waits past
    /// the 1-second clock resolution and asserts the parsed epoch increases.
    #[test]
    fn heartbeat_strictly_advances_over_time() {
        let dir = tempdir();
        let id = SessionId::generate();
        let socket = dir.join("test.sock");
        let mut reg = Registration::new(id.clone(), SessionConfig::default(), socket);
        let json_path = dir.join(format!("{id}.json"));
        reg.write_atomic(&json_path).unwrap();

        let parse = |ts: &str| -> u64 {
            ts.trim_end_matches('Z').parse::<u64>().expect("epoch seconds")
        };
        let before = parse(&reg.heartbeat_at);

        // Sleep past the 1-second timestamp resolution of now_iso8601().
        std::thread::sleep(std::time::Duration::from_millis(1100));
        reg.touch_heartbeat(&json_path).unwrap();

        let after_mem = parse(&reg.heartbeat_at);
        let read_back = Registration::read_from(&json_path).unwrap();
        let after_disk = parse(&read_back.heartbeat_at);

        assert!(after_mem > before, "in-memory heartbeat must advance ({after_mem} > {before})");
        assert!(after_disk > before, "on-disk heartbeat must advance ({after_disk} > {before})");
        assert_eq!(after_mem, after_disk, "in-memory and on-disk heartbeat must match");
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
            identity_fingerprint: Some("a1b2c3d4e5f6".into()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.shell.as_deref(), Some("/bin/zsh"));
        assert_eq!(parsed.term.as_deref(), Some("xterm-256color"));
        assert_eq!(parsed.cwd.as_deref(), Some("/home/user"));
        assert_eq!(parsed.termlink_version.as_deref(), Some("0.9.0"));
        assert_eq!(parsed.data_socket.as_deref(), Some("/tmp/data.sock"));
        assert_eq!(parsed.identity_fingerprint.as_deref(), Some("a1b2c3d4e5f6"));
    }

    #[test]
    fn session_metadata_optional_fields_omitted() {
        let meta = SessionMetadata::default();
        let json = serde_json::to_string(&meta).unwrap();
        assert!(!json.contains("shell"));
        assert!(!json.contains("term"));
        assert!(!json.contains("data_socket"));
        assert!(!json.contains("identity_fingerprint"));

        // Deserialize from empty object
        let parsed: SessionMetadata = serde_json::from_str("{}").unwrap();
        assert!(parsed.shell.is_none());
        assert!(parsed.data_socket.is_none());
        assert!(parsed.identity_fingerprint.is_none());
    }

    /// T-1436: pre-T-1436 registration JSON (no identity_fingerprint key) must
    /// still deserialize cleanly with the field set to `None`. Backward-compat
    /// guarantee.
    #[test]
    fn session_metadata_legacy_json_without_identity_fingerprint() {
        let legacy_json = r#"{
            "shell": "/bin/bash",
            "term": "xterm",
            "cwd": "/tmp",
            "termlink_version": "0.9.1500",
            "data_socket": "/tmp/data.sock"
        }"#;
        let parsed: SessionMetadata = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(parsed.shell.as_deref(), Some("/bin/bash"));
        assert!(
            parsed.identity_fingerprint.is_none(),
            "legacy JSON without identity_fingerprint must deserialize as None"
        );
    }

    /// T-1436: round-trip test for the new field — set explicitly, serialize,
    /// deserialize, fingerprint preserved exactly.
    #[test]
    fn session_metadata_identity_fingerprint_round_trip() {
        let fp = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let mut meta = SessionMetadata::default();
        meta.identity_fingerprint = Some(fp.to_string());
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("identity_fingerprint"));
        let parsed: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.identity_fingerprint.as_deref(), Some(fp));
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
