//! Client-side helper + cache for the `hub.capabilities` JSON-RPC method (T-1215).
//!
//! Lets a federating caller discover which methods a peer hub serves. Used by
//! the T-1165 pickup→channel bridge to decide between `channel.*` (new) and
//! `event.broadcast` + `inbox.*` (legacy) on a per-peer basis, without
//! requiring fleet-wide install convergence (see
//! `docs/reports/T-1214-fleet-diagnosis.md`).
//!
//! Cache is process-scoped and in-memory. Callers that want cross-process
//! persistence should extend the TOFU store in a follow-up task.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use termlink_protocol::control;
use termlink_protocol::TransportAddr;

use crate::client::rpc_call_addr;

/// In-memory cache of supported-method lists keyed by `host:port`.
#[derive(Default)]
pub struct HubCapabilitiesCache {
    inner: Mutex<HashMap<String, Vec<String>>>,
}

impl HubCapabilitiesCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a cached method list if present. `None` means "not probed yet".
    pub fn get(&self, host_port: &str) -> Option<Vec<String>> {
        self.inner.lock().ok()?.get(host_port).cloned()
    }

    /// Insert/overwrite the cached list for a peer.
    pub fn set(&self, host_port: String, methods: Vec<String>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(host_port, methods);
        }
    }

    /// Return true if the peer has been probed and supports `method`.
    /// Returns `false` if not probed or if the method is absent.
    pub fn supports(&self, host_port: &str, method: &str) -> bool {
        match self.get(host_port) {
            Some(methods) => methods.iter().any(|m| m == method),
            None => false,
        }
    }

    /// Drop all cached entries. Useful in tests and on fleet-config reload.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.clear();
        }
    }
}

/// Process-wide shared cache. First call initializes.
pub fn shared_cache() -> &'static HubCapabilitiesCache {
    static CACHE: OnceLock<HubCapabilitiesCache> = OnceLock::new();
    CACHE.get_or_init(HubCapabilitiesCache::new)
}

/// Probe a hub's supported-method list via the `hub.capabilities` RPC.
///
/// On success, caches the result in `cache` keyed by `host:port` and returns
/// the list. On RPC error, returns the error without caching (so the next
/// call retries).
///
/// Callers wanting the process-wide cache can pass `shared_cache()`.
pub async fn probe(
    host: &str,
    port: u16,
    cache: &HubCapabilitiesCache,
) -> std::io::Result<Vec<String>> {
    let key = format!("{host}:{port}");

    if let Some(cached) = cache.get(&key) {
        return Ok(cached);
    }

    let addr = TransportAddr::Tcp { host: host.to_string(), port };
    let response = rpc_call_addr(&addr, control::method::HUB_CAPABILITIES, serde_json::json!({}))
        .await
        .map_err(|e| std::io::Error::other(format!("hub.capabilities RPC failed: {e}")))?;

    let methods: Vec<String> = match response {
        termlink_protocol::jsonrpc::RpcResponse::Success(ok) => ok
            .result
            .get("methods")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .ok_or_else(|| {
                std::io::Error::other(
                    "hub.capabilities response missing `result.methods` array",
                )
            })?,
        termlink_protocol::jsonrpc::RpcResponse::Error(err) => {
            return Err(std::io::Error::other(format!(
                "hub.capabilities returned error: {}",
                err.error.message
            )));
        }
    };

    cache.set(key, methods.clone());
    Ok(methods)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_capabilities_cache_roundtrip() {
        let cache = HubCapabilitiesCache::new();
        assert!(cache.get("1.2.3.4:9100").is_none());

        cache.set("1.2.3.4:9100".to_string(), vec![
            "channel.post".to_string(),
            "session.discover".to_string(),
        ]);

        let got = cache.get("1.2.3.4:9100").expect("should be cached");
        assert_eq!(got, vec!["channel.post", "session.discover"]);

        assert!(cache.supports("1.2.3.4:9100", "channel.post"));
        assert!(!cache.supports("1.2.3.4:9100", "command.exec"));
        assert!(!cache.supports("5.6.7.8:9100", "channel.post"));
    }

    #[test]
    fn hub_capabilities_cache_overwrite_and_clear() {
        let cache = HubCapabilitiesCache::new();
        cache.set("h:1".to_string(), vec!["a".to_string()]);
        cache.set("h:1".to_string(), vec!["a".to_string(), "b".to_string()]);
        assert_eq!(cache.get("h:1").unwrap().len(), 2);

        cache.clear();
        assert!(cache.get("h:1").is_none());
    }

    #[test]
    fn shared_cache_is_singleton() {
        let a = shared_cache() as *const _;
        let b = shared_cache() as *const _;
        assert_eq!(a, b, "shared_cache should return the same instance");
    }
}
