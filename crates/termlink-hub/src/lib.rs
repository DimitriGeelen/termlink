pub mod aggregator;
pub mod artifact;
pub mod bypass;
pub mod channel;
pub mod circuit_breaker;
pub mod inbox;
pub mod pidfile;
pub mod remote_store;
pub mod route_cache;
pub mod router;
pub mod template_cache;
pub mod topic_lint;
pub mod trust;
pub mod server;
pub mod supervisor;
pub mod tls;

/// Shared test utilities (crate-internal).
#[cfg(test)]
pub(crate) mod test_util {
    /// Mutex to serialize all tests that modify TERMLINK_RUNTIME_DIR.
    /// Used by both router::tests and server::tests.
    pub static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
}
