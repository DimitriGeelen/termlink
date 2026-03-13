//! Transport address types for provider-agnostic session addressing.
//!
//! `TransportAddr` represents how to reach a session without coupling to a
//! specific transport implementation. Currently supports Unix sockets and
//! (as a serde-only variant) TCP.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Address at which a session can be reached.
///
/// This is a data-only enum (no runtime transport deps) — it lives in the
/// protocol crate so registration files can reference it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransportAddr {
    /// Unix domain socket.
    Unix {
        /// Path to the socket file.
        path: PathBuf,
    },
    /// TCP socket (serde only — no runtime implementation yet).
    Tcp {
        /// Hostname or IP address.
        host: String,
        /// Port number.
        port: u16,
    },
}

impl TransportAddr {
    /// Create a Unix socket address.
    pub fn unix(path: impl Into<PathBuf>) -> Self {
        Self::Unix { path: path.into() }
    }

    /// Create a TCP address.
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Returns `true` if this is a Unix socket address.
    pub fn is_unix(&self) -> bool {
        matches!(self, Self::Unix { .. })
    }

    /// Returns `true` if this is a TCP address.
    pub fn is_tcp(&self) -> bool {
        matches!(self, Self::Tcp { .. })
    }

    /// Returns the Unix socket path, if this is a Unix address.
    pub fn as_unix_path(&self) -> Option<&Path> {
        match self {
            Self::Unix { path } => Some(path),
            _ => None,
        }
    }
}

impl fmt::Display for TransportAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix { path } => write!(f, "unix:{}", path.display()),
            Self::Tcp { host, port } => write!(f, "tcp:{host}:{port}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_addr_roundtrip() {
        let addr = TransportAddr::unix("/tmp/test.sock");
        assert!(addr.is_unix());
        assert!(!addr.is_tcp());
        assert_eq!(addr.as_unix_path(), Some(Path::new("/tmp/test.sock")));

        let json = serde_json::to_string(&addr).unwrap();
        let parsed: TransportAddr = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, addr);
    }

    #[test]
    fn tcp_addr_roundtrip() {
        let addr = TransportAddr::tcp("localhost", 8080);
        assert!(addr.is_tcp());
        assert!(!addr.is_unix());
        assert_eq!(addr.as_unix_path(), None);

        let json = serde_json::to_string(&addr).unwrap();
        let parsed: TransportAddr = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, addr);
    }

    #[test]
    fn display_format() {
        assert_eq!(
            TransportAddr::unix("/tmp/x.sock").to_string(),
            "unix:/tmp/x.sock"
        );
        assert_eq!(
            TransportAddr::tcp("127.0.0.1", 9000).to_string(),
            "tcp:127.0.0.1:9000"
        );
    }

    #[test]
    fn json_tagged_format() {
        let addr = TransportAddr::unix("/tmp/test.sock");
        let json = serde_json::to_string_pretty(&addr).unwrap();
        assert!(json.contains("\"type\": \"unix\""));
        assert!(json.contains("\"path\":"));
    }
}
