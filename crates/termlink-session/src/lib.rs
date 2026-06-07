pub mod agent_identity;
pub(crate) mod ansi;
pub mod artifact;
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
pub mod claim_client;
pub mod client;
pub mod codec;
pub mod offline_queue;
pub mod data_server;
pub mod events;
pub mod executor;
pub mod governance_subscriber;
pub mod handler;
pub mod hub_capabilities;
pub mod inbox_channel;
pub mod pty;
pub mod scrollback;
pub mod server;
pub mod tofu;
pub mod transport;

pub use identity::SessionId;
pub use lifecycle::SessionState;
pub use registration::Registration;
pub use manager::{Session, SessionError};

// T-2031: arc-parallel-substrate Slice 3 — claim client surface.
// T-2037: Slice 4 adds `channel_claims` (read-only listing).
// T-2039: Slice 6 adds `channel_claims_summary` (aggregate observability).
pub use claim_client::{
    channel_claim, channel_claims, channel_claims_summary, channel_release, channel_renew,
    ClaimError, ClaimSummary, ClaimsAggregate, LeasedClaim, ReleaseSummary,
};

/// Shared test utilities (crate-internal).
#[cfg(test)]
pub(crate) mod test_util {
    /// Serialize PTY-allocating tests to prevent device exhaustion under parallel load.
    /// macOS `openpty()` returns ENXIO when too many PTY devices are allocated concurrently.
    pub static PTY_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
}
