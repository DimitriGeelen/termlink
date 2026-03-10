//! Peer credential extraction, UID-based authentication, and permission scoping
//! for Unix sockets.
//!
//! Security model phases:
//! - Phase 1 (T-077): Extract peer UID, reject different-UID connections
//! - Phase 2 (T-078): 4-tier permission scoping per RPC method
//! - Phase 3 (T-079): Capability tokens for fine-grained multi-agent auth

use std::io;
use std::os::unix::io::AsRawFd;

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
}
