pub mod agent_identity;
pub mod auth;
pub mod known_peers;
pub mod endpoint;
pub mod identity;
pub mod discovery;
pub mod lifecycle;
pub mod liveness;
pub mod registration;
pub mod manager;
pub mod bus_client;
pub mod client;
pub mod codec;
pub mod offline_queue;
pub mod data_server;
pub mod events;
pub mod executor;
pub mod governance_subscriber;
pub mod handler;
pub mod pty;
pub mod scrollback;
pub mod server;
pub mod tofu;
pub mod transport;

pub use identity::SessionId;
pub use lifecycle::SessionState;
pub use registration::Registration;
pub use manager::{Session, SessionError};

/// Shared test utilities (crate-internal).
#[cfg(test)]
pub(crate) mod test_util {
    /// Serialize PTY-allocating tests to prevent device exhaustion under parallel load.
    /// macOS `openpty()` returns ENXIO when too many PTY devices are allocated concurrently.
    pub static PTY_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
}
