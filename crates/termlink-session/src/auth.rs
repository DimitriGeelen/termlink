//! Peer credential extraction and UID-based authentication for Unix sockets.
//!
//! Phase 1 of the security model (T-008 inception → T-077 build):
//! - Extract peer UID/GID/PID on socket accept via OS-specific APIs
//! - Compare peer UID to session owner UID
//! - Same UID → allowed; different UID → AUTH_DENIED

use std::io;
use std::os::unix::io::AsRawFd;

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
}
