//! Peer credential extraction, UID-based authentication, and permission scoping
//! for Unix sockets.
//!
//! Security model phases:
//! - Phase 1 (T-077): Extract peer UID, reject different-UID connections
//! - Phase 2 (T-078): 4-tier permission scoping per RPC method
//! - Phase 3 (T-079/T-086): Capability tokens for fine-grained multi-agent auth

use std::io;
use std::os::unix::io::AsRawFd;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use termlink_protocol::control;

/// Credentials of a connected peer, extracted from the Unix socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerCredentials {
    pub uid: u32,
    pub gid: u32,
    pub pid: Option<u32>,
}

impl PeerCredentials {
    /// Extract peer credentials from a connected Unix stream.
    ///
    /// Uses `SO_PEERCRED` on Linux and `LOCAL_PEERCRED` + `LOCAL_PEERPID` on macOS.
    pub fn from_raw_fd(fd: std::os::unix::io::RawFd) -> io::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            Self::from_fd_linux(fd)
        }

        #[cfg(target_os = "macos")]
        {
            Self::from_fd_macos(fd)
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = fd;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "peer credential extraction not supported on this platform",
            ))
        }
    }

    /// Extract peer credentials from a tokio UnixStream.
    pub fn from_tokio_stream(stream: &tokio::net::UnixStream) -> io::Result<Self> {
        let std_stream = stream.as_raw_fd();
        Self::from_raw_fd(std_stream)
    }

    /// Check if this peer has the same UID as the given owner.
    pub fn is_same_user(&self, owner_uid: u32) -> bool {
        self.uid == owner_uid
    }

    #[cfg(target_os = "linux")]
    fn from_fd_linux(fd: std::os::unix::io::RawFd) -> io::Result<Self> {
        unsafe {
            let mut cred: libc::ucred = std::mem::zeroed();
            let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
            let ret = libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                &mut cred as *mut _ as *mut libc::c_void,
                &mut len,
            );
            if ret != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(PeerCredentials {
                uid: cred.uid,
                gid: cred.gid,
                pid: Some(cred.pid as u32),
            })
        }
    }

    #[cfg(target_os = "macos")]
    fn from_fd_macos(fd: std::os::unix::io::RawFd) -> io::Result<Self> {
        // LOCAL_PEERCRED for UID/GID
        let (uid, gid) = unsafe {
            let mut cred: libc::xucred = std::mem::zeroed();
            let mut len = std::mem::size_of::<libc::xucred>() as libc::socklen_t;
            let ret = libc::getsockopt(
                fd,
                libc::SOL_LOCAL,
                libc::LOCAL_PEERCRED,
                &mut cred as *mut _ as *mut libc::c_void,
                &mut len,
            );
            if ret != 0 {
                return Err(io::Error::last_os_error());
            }
            (cred.cr_uid, if cred.cr_ngroups > 0 { cred.cr_groups[0] } else { 0 })
        };

        // LOCAL_PEERPID for PID
        let pid = unsafe {
            let mut pid: libc::pid_t = 0;
            let mut len = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
            let ret = libc::getsockopt(
                fd,
                libc::SOL_LOCAL,
                libc::LOCAL_PEERPID,
                &mut pid as *mut _ as *mut libc::c_void,
                &mut len,
            );
            if ret == 0 && pid > 0 {
                Some(pid as u32)
            } else {
                None
            }
        };

        Ok(PeerCredentials { uid, gid, pid })
    }
}

/// Permission scope tiers for RPC method authorization.
///
/// Scopes are hierarchical: a higher scope implicitly grants all lower scopes.
/// `Execute > Control > Interact > Observe`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionScope {
    /// Read-only queries with no side effects.
    /// Methods: ping, query.*, event.poll, event.topics, kv.get, kv.list
    Observe = 0,

    /// Mutates session state or injects events.
    /// Methods: event.emit, event.broadcast, command.resize, session.update,
    ///          session.heartbeat, kv.set, kv.delete
    Interact = 1,

    /// Affects running processes (keystroke injection, signals).
    /// Methods: command.inject, command.signal
    Control = 2,

    /// Runs arbitrary shell commands.
    /// Methods: command.execute
    Execute = 3,
}

impl PermissionScope {
    /// Check if this scope is sufficient for the required scope.
    ///
    /// Higher scopes grant access to lower scopes (hierarchy).
    pub fn satisfies(&self, required: PermissionScope) -> bool {
        *self >= required
    }
}

impl std::fmt::Display for PermissionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => write!(f, "observe"),
            Self::Interact => write!(f, "interact"),
            Self::Control => write!(f, "control"),
            Self::Execute => write!(f, "execute"),
        }
    }
}

/// Map an RPC method name to its required permission scope.
///
/// Unknown methods default to `Execute` (deny by default).
pub fn method_scope(method: &str) -> PermissionScope {
    match method {
        // Observe: read-only, no side effects
        "termlink.ping"
        | control::method::QUERY_STATUS
        | control::method::QUERY_OUTPUT
        | control::method::QUERY_CAPABILITIES
        | control::method::EVENT_POLL
        | control::method::EVENT_SUBSCRIBE
        | control::method::EVENT_TOPICS
        | control::method::KV_GET
        | control::method::KV_LIST => PermissionScope::Observe,

        // Interact: mutates session state or event bus
        | control::method::EVENT_EMIT
        | control::method::EVENT_BROADCAST
        | control::method::EVENT_COLLECT
        | control::method::COMMAND_RESIZE
        | control::method::SESSION_UPDATE
        | control::method::SESSION_HEARTBEAT
        | control::method::KV_SET
        | control::method::KV_DELETE => PermissionScope::Interact,

        // Control: affects running processes
        control::method::COMMAND_INJECT
        | control::method::COMMAND_SIGNAL => PermissionScope::Control,

        // Execute: runs shell commands
        control::method::COMMAND_EXECUTE => PermissionScope::Execute,

        // Unknown methods: deny by default (require highest scope)
        _ => PermissionScope::Execute,
    }
}

type HmacSha256 = Hmac<Sha256>;

/// Default token time-to-live: 1 hour.
pub const DEFAULT_TOKEN_TTL_SECS: u64 = 3600;

/// A 32-byte secret used for HMAC-SHA256 token signing.
pub type TokenSecret = [u8; 32];

/// Generate a cryptographically random token secret.
pub fn generate_secret() -> TokenSecret {
    use rand::Rng;
    rand::thread_rng().r#gen()
}

/// Token payload that gets serialized and signed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenPayload {
    /// Permission scope granted by this token.
    pub scope: String,
    /// Session ID this token is scoped to (optional — empty means any session).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub session_id: String,
    /// Unix timestamp when the token was issued.
    pub issued_at: u64,
    /// Unix timestamp when the token expires.
    pub expires_at: u64,
    /// Random nonce to prevent replay across different token creations.
    pub nonce: String,
}

/// A signed capability token: base64(json_payload).base64(hmac_signature).
#[derive(Debug, Clone)]
pub struct CapabilityToken {
    /// The raw token string (payload.signature).
    pub raw: String,
    /// The decoded payload.
    pub payload: TokenPayload,
}

/// Errors from token creation or validation.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("token has expired")]
    Expired,
    #[error("invalid token format")]
    InvalidFormat,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid scope: {0}")]
    InvalidScope(String),
    #[error("token is for a different session")]
    SessionMismatch,
}

/// Parse a scope string into a PermissionScope.
pub fn parse_scope(s: &str) -> Result<PermissionScope, TokenError> {
    match s {
        "observe" => Ok(PermissionScope::Observe),
        "interact" => Ok(PermissionScope::Interact),
        "control" => Ok(PermissionScope::Control),
        "execute" => Ok(PermissionScope::Execute),
        _ => Err(TokenError::InvalidScope(s.to_string())),
    }
}

/// Create a signed capability token.
pub fn create_token(
    secret: &TokenSecret,
    scope: PermissionScope,
    session_id: &str,
    ttl_secs: u64,
) -> CapabilityToken {
    use base64::Engine;
    use rand::Rng;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let nonce: [u8; 16] = rand::thread_rng().r#gen();
    let nonce_hex = nonce.iter().map(|b| format!("{b:02x}")).collect::<String>();

    let payload = TokenPayload {
        scope: scope.to_string(),
        session_id: session_id.to_string(),
        issued_at: now,
        expires_at: now + ttl_secs,
        nonce: nonce_hex,
    };

    let payload_json = serde_json::to_string(&payload).expect("payload serializes");
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(payload_json.as_bytes());

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    mac.update(payload_b64.as_bytes());
    let signature = mac.finalize().into_bytes();
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature);

    let raw = format!("{payload_b64}.{sig_b64}");

    CapabilityToken { raw, payload }
}

/// Validate a token string and return the decoded payload with its scope.
///
/// Checks: format, HMAC signature, expiry, and optionally session ID.
pub fn validate_token(
    secret: &TokenSecret,
    token_str: &str,
    expected_session_id: Option<&str>,
) -> Result<(TokenPayload, PermissionScope), TokenError> {
    use base64::Engine;

    let parts: Vec<&str> = token_str.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(TokenError::InvalidFormat);
    }

    let (payload_b64, sig_b64) = (parts[0], parts[1]);

    // Verify HMAC signature
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    mac.update(payload_b64.as_bytes());

    let claimed_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| TokenError::InvalidFormat)?;
    mac.verify_slice(&claimed_sig)
        .map_err(|_| TokenError::InvalidSignature)?;

    // Decode payload
    let payload_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| TokenError::InvalidFormat)?;
    let payload: TokenPayload =
        serde_json::from_slice(&payload_json).map_err(|_| TokenError::InvalidFormat)?;

    // Check expiry
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now > payload.expires_at {
        return Err(TokenError::Expired);
    }

    // Check session ID if specified
    if let Some(expected) = expected_session_id
        && !payload.session_id.is_empty()
        && payload.session_id != expected
    {
        return Err(TokenError::SessionMismatch);
    }

    // Parse scope
    let scope = parse_scope(&payload.scope)?;

    Ok((payload, scope))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn peer_credentials_from_connected_socket() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = format!("/tmp/tl-auth-test-{}-{}.sock", std::process::id(), n);
        let _ = std::fs::remove_file(&path);

        let listener = tokio::net::UnixListener::bind(&path).unwrap();

        // Connect as client
        let client = tokio::net::UnixStream::connect(&path).await.unwrap();

        // Accept on server side
        let (server_stream, _) = listener.accept().await.unwrap();

        // Extract credentials from the accepted connection
        let creds = PeerCredentials::from_tokio_stream(&server_stream).unwrap();

        // Should be our own UID (same process)
        let our_uid = unsafe { libc::getuid() };
        assert_eq!(creds.uid, our_uid);
        assert!(creds.is_same_user(our_uid));
        assert!(!creds.is_same_user(our_uid + 1));

        // PID should be present on both Linux and macOS
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        assert!(creds.pid.is_some());

        // Cleanup
        drop(client);
        drop(server_stream);
        drop(listener);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn is_same_user_check() {
        let creds = PeerCredentials {
            uid: 501,
            gid: 20,
            pid: Some(1234),
        };
        assert!(creds.is_same_user(501));
        assert!(!creds.is_same_user(0));
        assert!(!creds.is_same_user(502));
    }

    #[test]
    fn scope_hierarchy() {
        // Higher scopes satisfy lower requirements
        assert!(PermissionScope::Execute.satisfies(PermissionScope::Control));
        assert!(PermissionScope::Execute.satisfies(PermissionScope::Interact));
        assert!(PermissionScope::Execute.satisfies(PermissionScope::Observe));
        assert!(PermissionScope::Control.satisfies(PermissionScope::Interact));
        assert!(PermissionScope::Control.satisfies(PermissionScope::Observe));
        assert!(PermissionScope::Interact.satisfies(PermissionScope::Observe));

        // Same scope satisfies itself
        assert!(PermissionScope::Observe.satisfies(PermissionScope::Observe));
        assert!(PermissionScope::Execute.satisfies(PermissionScope::Execute));

        // Lower scopes don't satisfy higher requirements
        assert!(!PermissionScope::Observe.satisfies(PermissionScope::Interact));
        assert!(!PermissionScope::Observe.satisfies(PermissionScope::Execute));
        assert!(!PermissionScope::Interact.satisfies(PermissionScope::Control));
        assert!(!PermissionScope::Control.satisfies(PermissionScope::Execute));
    }

    #[test]
    fn method_scope_mapping() {
        use termlink_protocol::control::method;

        // Observe tier
        assert_eq!(method_scope("termlink.ping"), PermissionScope::Observe);
        assert_eq!(method_scope(method::QUERY_STATUS), PermissionScope::Observe);
        assert_eq!(method_scope(method::QUERY_OUTPUT), PermissionScope::Observe);
        assert_eq!(method_scope(method::EVENT_POLL), PermissionScope::Observe);
        assert_eq!(method_scope(method::KV_GET), PermissionScope::Observe);
        assert_eq!(method_scope(method::KV_LIST), PermissionScope::Observe);

        // Interact tier
        assert_eq!(method_scope(method::EVENT_EMIT), PermissionScope::Interact);
        assert_eq!(method_scope(method::SESSION_UPDATE), PermissionScope::Interact);
        assert_eq!(method_scope(method::KV_SET), PermissionScope::Interact);
        assert_eq!(method_scope(method::KV_DELETE), PermissionScope::Interact);
        assert_eq!(method_scope(method::COMMAND_RESIZE), PermissionScope::Interact);

        // Control tier
        assert_eq!(method_scope(method::COMMAND_INJECT), PermissionScope::Control);
        assert_eq!(method_scope(method::COMMAND_SIGNAL), PermissionScope::Control);

        // Execute tier
        assert_eq!(method_scope(method::COMMAND_EXECUTE), PermissionScope::Execute);

        // Unknown methods default to Execute (deny by default)
        assert_eq!(method_scope("foo.bar"), PermissionScope::Execute);
        assert_eq!(method_scope("admin.shutdown"), PermissionScope::Execute);
    }

    #[test]
    fn scope_display() {
        assert_eq!(format!("{}", PermissionScope::Observe), "observe");
        assert_eq!(format!("{}", PermissionScope::Interact), "interact");
        assert_eq!(format!("{}", PermissionScope::Control), "control");
        assert_eq!(format!("{}", PermissionScope::Execute), "execute");
    }

    // === Token tests (T-086) ===

    #[test]
    fn generate_secret_is_random() {
        let s1 = generate_secret();
        let s2 = generate_secret();
        assert_ne!(s1, s2);
        assert_eq!(s1.len(), 32);
    }

    #[test]
    fn create_and_validate_token() {
        let secret = generate_secret();
        let token = create_token(&secret, PermissionScope::Interact, "sess-123", 3600);

        let (payload, scope) = validate_token(&secret, &token.raw, Some("sess-123")).unwrap();
        assert_eq!(scope, PermissionScope::Interact);
        assert_eq!(payload.session_id, "sess-123");
        assert!(payload.expires_at > payload.issued_at);
    }

    #[test]
    fn token_with_empty_session_id_matches_any() {
        let secret = generate_secret();
        let token = create_token(&secret, PermissionScope::Observe, "", 3600);

        // Should validate against any session ID
        let (_, scope) = validate_token(&secret, &token.raw, Some("any-session")).unwrap();
        assert_eq!(scope, PermissionScope::Observe);
    }

    #[test]
    fn token_session_mismatch_rejected() {
        let secret = generate_secret();
        let token = create_token(&secret, PermissionScope::Execute, "sess-A", 3600);

        let err = validate_token(&secret, &token.raw, Some("sess-B")).unwrap_err();
        assert!(matches!(err, TokenError::SessionMismatch));
    }

    #[test]
    fn token_wrong_secret_rejected() {
        let secret1 = generate_secret();
        let secret2 = generate_secret();
        let token = create_token(&secret1, PermissionScope::Execute, "", 3600);

        let err = validate_token(&secret2, &token.raw, None).unwrap_err();
        assert!(matches!(err, TokenError::InvalidSignature));
    }

    #[test]
    fn token_tampered_payload_rejected() {
        let secret = generate_secret();
        let token = create_token(&secret, PermissionScope::Observe, "", 3600);

        // Tamper with the payload portion (change first char)
        let parts: Vec<&str> = token.raw.splitn(2, '.').collect();
        let mut tampered = parts[0].as_bytes().to_vec();
        tampered[0] ^= 0xFF;
        let tampered_str = format!("{}.{}", String::from_utf8_lossy(&tampered), parts[1]);

        let err = validate_token(&secret, &tampered_str, None).unwrap_err();
        assert!(matches!(
            err,
            TokenError::InvalidSignature | TokenError::InvalidFormat
        ));
    }

    #[test]
    fn token_expired_rejected() {
        let secret = generate_secret();
        // Create token with 0 TTL (immediately expired)
        let token = create_token(&secret, PermissionScope::Observe, "", 0);

        // Sleep briefly to ensure we're past expiry
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let err = validate_token(&secret, &token.raw, None).unwrap_err();
        assert!(matches!(err, TokenError::Expired));
    }

    #[test]
    fn token_invalid_format_rejected() {
        let secret = generate_secret();

        // No dot separator
        assert!(matches!(
            validate_token(&secret, "nodot", None).unwrap_err(),
            TokenError::InvalidFormat
        ));

        // Empty string
        assert!(matches!(
            validate_token(&secret, "", None).unwrap_err(),
            TokenError::InvalidFormat
        ));
    }

    #[test]
    fn parse_scope_roundtrip() {
        for scope in [
            PermissionScope::Observe,
            PermissionScope::Interact,
            PermissionScope::Control,
            PermissionScope::Execute,
        ] {
            let s = scope.to_string();
            let parsed = parse_scope(&s).unwrap();
            assert_eq!(parsed, scope);
        }
    }

    #[test]
    fn parse_scope_invalid() {
        assert!(matches!(
            parse_scope("admin").unwrap_err(),
            TokenError::InvalidScope(_)
        ));
    }

    #[test]
    fn all_four_scopes_can_be_tokenized() {
        let secret = generate_secret();
        for scope in [
            PermissionScope::Observe,
            PermissionScope::Interact,
            PermissionScope::Control,
            PermissionScope::Execute,
        ] {
            let token = create_token(&secret, scope, "", 3600);
            let (_, validated_scope) = validate_token(&secret, &token.raw, None).unwrap();
            assert_eq!(validated_scope, scope);
        }
    }
}
