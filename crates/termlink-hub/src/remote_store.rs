//! In-memory store for remote (TCP) session registrations.
//!
//! Remote sessions register via `session.register_remote` RPC and must
//! heartbeat periodically to stay alive. Entries expire after a configurable TTL.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use serde_json::json;

/// Default time-to-live for remote session entries.
pub const DEFAULT_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Default interval for the reaper background task.
pub const REAPER_INTERVAL: Duration = Duration::from_secs(30);

/// A remote session entry stored in the hub's memory.
#[derive(Clone, Debug)]
pub struct RemoteEntry {
    pub id: String,
    pub display_name: String,
    pub host: String,
    pub port: u16,
    pub pid: Option<u32>,
    pub roles: Vec<String>,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub state: String,
    pub registered_at: Instant,
    pub last_heartbeat: Instant,
    pub ttl: Duration,
}

impl RemoteEntry {
    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        self.last_heartbeat.elapsed() > self.ttl
    }

    /// Convert to JSON for discovery responses.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "display_name": self.display_name,
            "state": self.state,
            "capabilities": self.capabilities,
            "roles": self.roles,
            "tags": self.tags,
            "pid": self.pid,
            "addr": {
                "type": "tcp",
                "host": self.host,
                "port": self.port,
            },
            "remote": true,
        })
    }
}

/// Thread-safe store for remote session entries.
#[derive(Clone)]
pub struct RemoteStore {
    entries: Arc<RwLock<HashMap<String, RemoteEntry>>>,
}

impl Default for RemoteStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new remote session. Returns the assigned ID.
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        &self,
        display_name: String,
        host: String,
        port: u16,
        pid: Option<u32>,
        roles: Vec<String>,
        tags: Vec<String>,
        capabilities: Vec<String>,
    ) -> String {
        let id = format!(
            "tl-tcp-{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                & 0xFFFFFFFF
        );
        let now = Instant::now();
        let entry = RemoteEntry {
            id: id.clone(),
            display_name,
            host,
            port,
            pid,
            roles,
            tags,
            capabilities,
            state: "ready".to_string(),
            registered_at: now,
            last_heartbeat: now,
            ttl: DEFAULT_TTL,
        };
        self.entries.write().unwrap().insert(id.clone(), entry);
        id
    }

    /// Refresh the heartbeat for a remote session. Returns true if found.
    pub fn heartbeat(&self, id: &str) -> bool {
        if let Some(entry) = self.entries.write().unwrap().get_mut(id) {
            entry.last_heartbeat = Instant::now();
            true
        } else {
            false
        }
    }

    /// Remove a remote session. Returns true if found.
    pub fn deregister(&self, id: &str) -> bool {
        self.entries.write().unwrap().remove(id).is_some()
    }

    /// Get all live (non-expired) entries.
    pub fn list_live(&self) -> Vec<RemoteEntry> {
        self.entries
            .read()
            .unwrap()
            .values()
            .filter(|e| !e.is_expired())
            .cloned()
            .collect()
    }

    /// Remove all expired entries. Returns the number removed.
    pub fn reap_expired(&self) -> usize {
        let mut store = self.entries.write().unwrap();
        let before = store.len();
        store.retain(|_, e| !e.is_expired());
        before - store.len()
    }

    /// Get an entry by ID (if live).
    pub fn get(&self, id: &str) -> Option<RemoteEntry> {
        self.entries
            .read()
            .unwrap()
            .get(id)
            .filter(|e| !e.is_expired())
            .cloned()
    }

    /// Number of entries (including expired).
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Remove all entries (used in testing).
    pub fn clear(&self) {
        self.entries.write().unwrap().clear();
    }
}

/// Run the reaper background task that periodically removes expired entries.
pub async fn run_reaper(
    store: RemoteStore,
    interval: Duration,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                let reaped = store.reap_expired();
                if reaped > 0 {
                    tracing::info!(reaped, "Remote store: expired {} session(s)", reaped);
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_list() {
        let store = RemoteStore::new();
        let id = store.register(
            "worker-1".into(),
            "192.168.1.50".into(),
            9001,
            Some(12345),
            vec!["compute".into()],
            vec!["prod".into()],
            vec![],
        );
        assert!(id.starts_with("tl-tcp-"));

        let entries = store.list_live();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].display_name, "worker-1");
        assert_eq!(entries[0].host, "192.168.1.50");
        assert_eq!(entries[0].port, 9001);
    }

    #[test]
    fn heartbeat_refreshes() {
        let store = RemoteStore::new();
        let id = store.register("hb-test".into(), "10.0.0.1".into(), 8000, None, vec![], vec![], vec![]);

        // Heartbeat should succeed
        assert!(store.heartbeat(&id));

        // Unknown ID should fail
        assert!(!store.heartbeat("nonexistent"));
    }

    #[test]
    fn deregister_removes() {
        let store = RemoteStore::new();
        let id = store.register("dereg".into(), "10.0.0.1".into(), 8000, None, vec![], vec![], vec![]);
        assert_eq!(store.len(), 1);

        assert!(store.deregister(&id));
        assert_eq!(store.len(), 0);

        // Double deregister returns false
        assert!(!store.deregister(&id));
    }

    #[test]
    fn expired_entries_filtered() {
        let store = RemoteStore::new();
        let id = store.register("exp".into(), "10.0.0.1".into(), 8000, None, vec![], vec![], vec![]);

        // Manually set TTL to zero to force expiry
        {
            let mut entries = store.entries.write().unwrap();
            entries.get_mut(&id).unwrap().ttl = Duration::from_secs(0);
        }

        // list_live should filter out expired
        assert_eq!(store.list_live().len(), 0);

        // get should return None
        assert!(store.get(&id).is_none());

        // reap should remove it
        assert_eq!(store.reap_expired(), 1);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn empty_store() {
        let store = RemoteStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.list_live().is_empty());
        assert!(store.get("nonexistent").is_none());
        assert_eq!(store.reap_expired(), 0);
    }

    #[test]
    fn clear_removes_all() {
        let store = RemoteStore::new();
        store.register("a".into(), "10.0.0.1".into(), 8000, None, vec![], vec![], vec![]);
        // IDs use ms timestamps — sleep to ensure distinct IDs
        std::thread::sleep(Duration::from_millis(2));
        store.register("b".into(), "10.0.0.2".into(), 8001, None, vec![], vec![], vec![]);
        assert_eq!(store.len(), 2);

        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn multiple_entries_independent() {
        let store = RemoteStore::new();
        let id1 = store.register("worker-1".into(), "10.0.0.1".into(), 8000, Some(100), vec!["agent".into()], vec!["prod".into()], vec!["execute".into()]);
        // IDs use ms timestamps — sleep to ensure distinct IDs
        std::thread::sleep(Duration::from_millis(2));
        let id2 = store.register("worker-2".into(), "10.0.0.2".into(), 8001, Some(200), vec!["compute".into()], vec!["staging".into()], vec![]);

        assert_eq!(store.len(), 2);
        assert_ne!(id1, id2);

        let e1 = store.get(&id1).unwrap();
        let e2 = store.get(&id2).unwrap();
        assert_eq!(e1.display_name, "worker-1");
        assert_eq!(e2.display_name, "worker-2");
        assert_eq!(e1.port, 8000);
        assert_eq!(e2.port, 8001);

        // Deregister one, other remains
        store.deregister(&id1);
        assert_eq!(store.len(), 1);
        assert!(store.get(&id1).is_none());
        assert!(store.get(&id2).is_some());
    }

    #[test]
    fn default_impl() {
        let store = RemoteStore::default();
        assert!(store.is_empty());
    }

    #[test]
    fn to_json_format() {
        let store = RemoteStore::new();
        let id = store.register(
            "json-test".into(),
            "10.0.0.1".into(),
            9100,
            Some(999),
            vec!["agent".into()],
            vec!["test".into()],
            vec!["execute".into()],
        );

        let entry = store.get(&id).unwrap();
        let json = entry.to_json();

        assert_eq!(json["display_name"], "json-test");
        assert_eq!(json["addr"]["type"], "tcp");
        assert_eq!(json["addr"]["host"], "10.0.0.1");
        assert_eq!(json["addr"]["port"], 9100);
        assert_eq!(json["remote"], true);
    }
}
