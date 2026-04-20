//! Client-side offline queue for bus posts (T-1161).
//!
//! Backs a durable FIFO of `channel.post` RPC calls that couldn't be
//! delivered directly. Messages are persisted to SQLite so they survive
//! client restarts; a periodic flush task drains the queue when the hub
//! becomes reachable again.
//!
//! Queue is bounded (`TERMLINK_OUTBOUND_CAP`, default 1000) to prevent
//! unbounded growth when the hub is down for long periods. When full,
//! `enqueue` returns `QueueError::QueueFull` so callers fail loudly
//! (per T-1155 §R3) rather than silently drop.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// One `channel.post` RPC request, ready to be replayed against the hub.
/// Mirrors the parameter shape required by `control::method::CHANNEL_POST`
/// (see `termlink-hub/src/channel.rs::handle_channel_post_with`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingPost {
    pub topic: String,
    pub msg_type: String,
    pub payload: Vec<u8>,
    pub artifact_ref: Option<String>,
    pub ts_unix_ms: i64,
    pub sender_id: String,
    pub sender_pubkey_hex: String,
    pub signature_hex: String,
}

/// Row id in the outbound table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueueId(pub i64);

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("outbound queue full ({cap} entries; refusing new posts — R3 loud-fail)")]
    QueueFull { cap: u64 },

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, QueueError>;

/// Default outbound queue cap. Override via `TERMLINK_OUTBOUND_CAP=N`.
pub const DEFAULT_CAP: u64 = 1000;

/// File name of the on-disk queue under `~/.termlink/` (spec'd by T-1155/T-1161).
pub const DEFAULT_FILE_NAME: &str = "outbound.sqlite";

/// Resolve the default queue path: `$HOME/.termlink/outbound.sqlite` (or
/// `$TERMLINK_IDENTITY_DIR/outbound.sqlite` when that env is set — keeps
/// per-fleet test isolation alongside the identity key).
pub fn default_queue_path() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("TERMLINK_IDENTITY_DIR") {
        return std::path::PathBuf::from(dir).join(DEFAULT_FILE_NAME);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(home).join(".termlink").join(DEFAULT_FILE_NAME)
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS pending_posts (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    post_json     TEXT    NOT NULL,
    enqueued_ms   INTEGER NOT NULL,
    attempts      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS pending_posts_enqueued ON pending_posts(enqueued_ms);
"#;

/// Persistent FIFO of pending `channel.post` calls.
///
/// SQLite-backed for durability across client restarts. Single-writer via
/// internal `Mutex<Connection>`; `rusqlite` is not `Sync` so multi-task
/// access goes through the lock.
pub struct OfflineQueue {
    conn: Mutex<Connection>,
    cap: u64,
}

impl OfflineQueue {
    /// Open (or create) the SQLite-backed queue at `path`. Creates parent
    /// dirs as needed. Reads `TERMLINK_OUTBOUND_CAP` for cap override.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(p)?;
        conn.execute_batch(SCHEMA)?;
        let cap = read_cap_from_env();
        Ok(Self { conn: Mutex::new(conn), cap })
    }

    /// Open an in-memory queue (tests only).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn: Mutex::new(conn), cap: read_cap_from_env() })
    }

    /// Configured cap (max retained entries).
    pub fn cap(&self) -> u64 {
        self.cap
    }

    /// Number of pending posts currently retained.
    pub fn size(&self) -> Result<u64> {
        let conn = self.conn.lock().expect("queue mutex poisoned");
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM pending_posts", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    /// Append a pending post. Returns `QueueFull` without persisting if
    /// the queue already holds `cap` entries (loud reject — R3).
    pub fn enqueue(&self, post: &PendingPost) -> Result<QueueId> {
        let conn = self.conn.lock().expect("queue mutex poisoned");
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM pending_posts", [], |r| r.get(0))?;
        if (n as u64) >= self.cap {
            return Err(QueueError::QueueFull { cap: self.cap });
        }
        let json = serde_json::to_string(post)?;
        let now_ms = now_unix_ms();
        conn.execute(
            "INSERT INTO pending_posts (post_json, enqueued_ms) VALUES (?1, ?2)",
            params![json, now_ms],
        )?;
        Ok(QueueId(conn.last_insert_rowid()))
    }

    /// Peek the oldest pending post (lowest id), without removing it.
    pub fn peek_oldest(&self) -> Result<Option<(QueueId, PendingPost)>> {
        let conn = self.conn.lock().expect("queue mutex poisoned");
        let row = conn
            .query_row(
                "SELECT id, post_json FROM pending_posts ORDER BY id ASC LIMIT 1",
                [],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some((id, json)) => {
                let post: PendingPost = serde_json::from_str(&json)?;
                Ok(Some((QueueId(id), post)))
            }
        }
    }

    /// Remove the entry with the given id. No-op if not present.
    pub fn pop(&self, id: QueueId) -> Result<()> {
        let conn = self.conn.lock().expect("queue mutex poisoned");
        conn.execute("DELETE FROM pending_posts WHERE id = ?1", params![id.0])?;
        Ok(())
    }

    /// Increment the attempt counter for a row (used after a failed flush
    /// so operators can diagnose poison messages via `queue-status`).
    pub fn bump_attempts(&self, id: QueueId) -> Result<()> {
        let conn = self.conn.lock().expect("queue mutex poisoned");
        conn.execute(
            "UPDATE pending_posts SET attempts = attempts + 1 WHERE id = ?1",
            params![id.0],
        )?;
        Ok(())
    }
}

fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn read_cap_from_env() -> u64 {
    std::env::var("TERMLINK_OUTBOUND_CAP")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_CAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_post(topic: &str, payload: &[u8]) -> PendingPost {
        PendingPost {
            topic: topic.to_string(),
            msg_type: "chat".to_string(),
            payload: payload.to_vec(),
            artifact_ref: None,
            ts_unix_ms: 42,
            sender_id: "abc".to_string(),
            sender_pubkey_hex: "00".repeat(32),
            signature_hex: "00".repeat(64),
        }
    }

    #[test]
    fn open_creates_empty_queue() {
        let q = OfflineQueue::open_in_memory().unwrap();
        assert_eq!(q.size().unwrap(), 0);
        assert!(q.peek_oldest().unwrap().is_none());
    }

    #[test]
    fn enqueue_peek_pop_roundtrip() {
        let q = OfflineQueue::open_in_memory().unwrap();
        let p = sample_post("t1", b"hello");
        let id = q.enqueue(&p).unwrap();
        assert_eq!(q.size().unwrap(), 1);
        let (peek_id, peek_post) = q.peek_oldest().unwrap().unwrap();
        assert_eq!(peek_id, id);
        assert_eq!(peek_post, p);
        q.pop(id).unwrap();
        assert_eq!(q.size().unwrap(), 0);
    }

    #[test]
    fn fifo_order_preserved() {
        let q = OfflineQueue::open_in_memory().unwrap();
        for i in 0..5u8 {
            q.enqueue(&sample_post("t", &[i])).unwrap();
        }
        assert_eq!(q.size().unwrap(), 5);
        for i in 0..5u8 {
            let (id, post) = q.peek_oldest().unwrap().unwrap();
            assert_eq!(post.payload, vec![i]);
            q.pop(id).unwrap();
        }
        assert_eq!(q.size().unwrap(), 0);
    }

    #[test]
    fn cap_enforced_rejects_overflow() {
        let q = OfflineQueue { conn: Mutex::new({
            let c = Connection::open_in_memory().unwrap();
            c.execute_batch(SCHEMA).unwrap();
            c
        }), cap: 3 };
        q.enqueue(&sample_post("t", &[1])).unwrap();
        q.enqueue(&sample_post("t", &[2])).unwrap();
        q.enqueue(&sample_post("t", &[3])).unwrap();
        let err = q.enqueue(&sample_post("t", &[4])).unwrap_err();
        assert!(matches!(err, QueueError::QueueFull { cap: 3 }));
        assert_eq!(q.size().unwrap(), 3);
    }

    #[test]
    fn survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.sqlite");
        {
            let q = OfflineQueue::open(&path).unwrap();
            q.enqueue(&sample_post("persist", b"abc")).unwrap();
        }
        let q2 = OfflineQueue::open(&path).unwrap();
        assert_eq!(q2.size().unwrap(), 1);
        let (_, post) = q2.peek_oldest().unwrap().unwrap();
        assert_eq!(post.topic, "persist");
        assert_eq!(post.payload, b"abc".to_vec());
    }

    #[test]
    fn concurrent_enqueue_preserves_fifo_per_topic() {
        // SQLite AUTOINCREMENT + BEGIN IMMEDIATE serializes writes; drain
        // order must match per-topic submission order even when many
        // threads enqueue in parallel.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("concurrent.sqlite");
        let q = std::sync::Arc::new(OfflineQueue::open(&path).unwrap());
        let mut handles = vec![];
        for topic_idx in 0..3 {
            let q = q.clone();
            let topic = format!("t{topic_idx}");
            handles.push(std::thread::spawn(move || {
                for i in 0..20u8 {
                    q.enqueue(&sample_post(&topic, &[topic_idx as u8, i])).unwrap();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(q.size().unwrap(), 60);
        // Drain and verify: within each topic, markers appear in ascending order.
        let mut seen: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
        while let Some((id, post)) = q.peek_oldest().unwrap() {
            seen.entry(post.topic.clone()).or_default().push(post.payload[1]);
            q.pop(id).unwrap();
        }
        let expected: Vec<u8> = (0..20u8).collect();
        for order in seen.values() {
            assert_eq!(order, &expected);
        }
    }

    #[test]
    fn bump_attempts_persists() {
        let q = OfflineQueue::open_in_memory().unwrap();
        let id = q.enqueue(&sample_post("t", &[1])).unwrap();
        q.bump_attempts(id).unwrap();
        q.bump_attempts(id).unwrap();
        let conn = q.conn.lock().unwrap();
        let a: i64 = conn
            .query_row("SELECT attempts FROM pending_posts WHERE id = ?1", params![id.0], |r| r.get(0))
            .unwrap();
        assert_eq!(a, 2);
    }
}
