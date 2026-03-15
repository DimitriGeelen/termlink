pub mod pidfile;
pub mod remote_store;
pub mod router;
pub mod server;
pub mod supervisor;

/// Shared test utilities (crate-internal).
#[cfg(test)]
pub(crate) mod test_util {
    /// Mutex to serialize all tests that modify TERMLINK_RUNTIME_DIR.
    /// Used by both router::tests and server::tests.
    pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
}
