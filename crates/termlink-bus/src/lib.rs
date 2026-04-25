//! TermLink agent channel bus (T-1155).
//!
//! Append-only per-topic log, per-recipient cursor store, and per-topic
//! retention. Hub embeds this crate as a passive library — net/RPC lives
//! in `termlink-hub` (T-1160 exposes the channel API).
//!
//! Wedge 1 (T-1158 scaffold): crate skeleton, `Envelope`, `Retention`,
//! `Bus::open`, `create_topic` + `list_topics` via SQLite metadata.
//! Log-append + subscribe + retention sweep land in follow-up wedges.

mod envelope;
mod error;
mod log;
mod meta;
mod retention;

pub use envelope::Envelope;
pub use error::{BusError, Result};
pub use log::Offset;
pub use retention::Retention;

/// Iterator yielded by `Bus::subscribe` — one `(offset, envelope)` per
/// record, or a decode/IO error.
pub type SubscribeIter = Box<dyn Iterator<Item = Result<(Offset, Envelope)>> + Send>;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex as StdMutex;

/// Channel bus instance. Backed by a per-topic append-only log under
/// `<root>/topics/` and a SQLite metadata sidecar at `<root>/meta.db`.
pub struct Bus {
    root: PathBuf,
    meta: meta::Meta,
    appenders: StdMutex<HashMap<String, std::sync::Arc<log::LogAppender>>>,
}

impl Bus {
    /// Open (or create) a bus rooted at `path`. Creates the directory,
    /// the `topics/` subdir, and initializes the SQLite schema if missing.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("topics"))?;
        let meta = meta::Meta::open(&root.join("meta.db"))?;
        Ok(Self {
            root,
            meta,
            appenders: StdMutex::new(HashMap::new()),
        })
    }

    /// Root directory the bus lives under.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Register a new topic with a retention policy. Idempotent — a second
    /// call with the same name and policy is a no-op; a second call with a
    /// different policy returns `BusError::TopicPolicyMismatch`.
    pub fn create_topic(&self, name: &str, retention: Retention) -> Result<()> {
        self.meta.create_topic(name, retention)
    }

    /// List all registered topic names, sorted.
    pub fn list_topics(&self) -> Result<Vec<String>> {
        self.meta.list_topics()
    }

    /// Retention policy for `topic`, or `None` if the topic doesn't exist.
    pub fn topic_retention(&self, topic: &str) -> Result<Option<Retention>> {
        self.meta.topic_retention(topic)
    }

    /// Count records currently indexed for `topic`. Records pruned by
    /// `sweep` are not counted. Unknown topic returns `Ok(0)` rather than
    /// erroring — callers can aggregate over a prefix list without first
    /// checking existence (T-1233 / T-1229a).
    pub fn topic_record_count(&self, topic: &str) -> Result<u64> {
        self.meta.count_records(topic)
    }

    /// Append an envelope to `topic`'s log. Returns the logical offset
    /// (0-based sequence number) assigned to the new record. The topic
    /// must have been registered via `create_topic` first.
    pub async fn post(&self, topic: &str, env: &Envelope) -> Result<Offset> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let appender = self.appender_for(topic)?;
        let bytes = log::encode_envelope(env)?;
        let byte_pos = appender.append(&bytes).await?;
        let length = bytes.len() as u64;
        self.meta.record_append(topic, byte_pos, length, env.ts_unix_ms)
    }

    /// Subscribe to `topic` starting at logical offset `cursor`. Returns
    /// an iterator yielding `(offset, envelope)` for every record whose
    /// offset is >= `cursor`. Empty topics yield an empty iterator.
    ///
    /// The iterator is backed by the SQLite records index plus positional
    /// reads into the log file, so records trimmed by `sweep` are not
    /// yielded even if the underlying file still contains their bytes.
    pub fn subscribe(&self, topic: &str, cursor: Offset) -> Result<SubscribeIter> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let records = self.meta.records_from(topic, cursor)?;
        if records.is_empty() {
            return Ok(Box::new(std::iter::empty()));
        }
        let path = log::topic_log_path(&self.root, topic);
        let reader = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Box::new(std::iter::empty()));
            }
            Err(e) => return Err(BusError::Io(e)),
        };
        let iter = log::ReaderIter::new(reader, records);
        Ok(Box::new(iter))
    }

    /// Persist a subscriber's cursor. Use this after consuming records so
    /// a crash restart resumes where the subscriber left off.
    pub fn advance_cursor(
        &self,
        subscriber_id: &str,
        topic: &str,
        offset: Offset,
    ) -> Result<()> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        self.meta.put_cursor(subscriber_id, topic, offset)
    }

    /// Read the persisted cursor for a subscriber. `None` if never set.
    pub fn get_cursor(&self, subscriber_id: &str, topic: &str) -> Result<Option<Offset>> {
        self.meta.get_cursor(subscriber_id, topic)
    }

    /// Apply the retention policy for `topic`, deleting record index rows
    /// that fall outside the policy. Returns the number of records pruned.
    /// Explicit — the library runs no background thread (per T-1155).
    pub fn sweep(&self, topic: &str, now_unix_ms: i64) -> Result<u64> {
        let retention = match self.meta.topic_retention(topic)? {
            Some(r) => r,
            None => return Err(BusError::UnknownTopic(topic.to_string())),
        };
        let (keep_after, keep_last) = match retention {
            Retention::Forever => return Ok(0),
            Retention::Days(d) => {
                let cutoff = now_unix_ms - i64::from(d) * 86_400_000;
                (Some(cutoff), None)
            }
            Retention::Messages(n) => (None, Some(n)),
        };
        self.meta.sweep_records(topic, keep_after, keep_last)
    }

    fn appender_for(&self, topic: &str) -> Result<std::sync::Arc<log::LogAppender>> {
        let mut guard = self.appenders.lock().expect("appenders mutex poisoned");
        if let Some(a) = guard.get(topic) {
            return Ok(a.clone());
        }
        let path = log::topic_log_path(&self.root, topic);
        let appender = std::sync::Arc::new(log::LogAppender::open(&path)?);
        guard.insert(topic.to_string(), appender.clone());
        Ok(appender)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_bus() -> (TempDir, Bus) {
        let dir = TempDir::new().expect("tempdir");
        let bus = Bus::open(dir.path()).expect("open bus");
        (dir, bus)
    }

    #[test]
    fn open_creates_layout() {
        let (dir, bus) = tmp_bus();
        assert!(dir.path().join("topics").is_dir());
        assert!(dir.path().join("meta.db").is_file());
        assert_eq!(bus.root(), dir.path());
    }

    #[test]
    fn create_topic_roundtrip() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("broadcast:global", Retention::Forever).unwrap();
        bus.create_topic("channel:learnings", Retention::Days(30)).unwrap();
        let topics = bus.list_topics().unwrap();
        assert_eq!(topics, vec!["broadcast:global", "channel:learnings"]);
    }

    #[test]
    fn create_topic_idempotent_on_same_policy() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Messages(100)).unwrap();
        bus.create_topic("t", Retention::Messages(100)).unwrap();
        assert_eq!(bus.list_topics().unwrap().len(), 1);
    }

    #[test]
    fn create_topic_rejects_policy_mismatch() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let err = bus.create_topic("t", Retention::Days(7)).unwrap_err();
        assert!(matches!(err, BusError::TopicPolicyMismatch { .. }));
    }

    fn env(topic: &str, payload: &[u8]) -> Envelope {
        Envelope {
            topic: topic.to_string(),
            sender_id: "test".to_string(),
            msg_type: "note".to_string(),
            payload: payload.to_vec(),
            artifact_ref: None,
            ts_unix_ms: 0,
        }
    }

    #[tokio::test]
    async fn topic_record_count_reflects_posts_and_unknown_topics(
    ) {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("inbox:alice", Retention::Forever).unwrap();
        bus.create_topic("inbox:bob", Retention::Forever).unwrap();
        bus.post("inbox:alice", &env("inbox:alice", b"a")).await.unwrap();
        bus.post("inbox:alice", &env("inbox:alice", b"b")).await.unwrap();
        bus.post("inbox:alice", &env("inbox:alice", b"c")).await.unwrap();
        assert_eq!(bus.topic_record_count("inbox:alice").unwrap(), 3);
        assert_eq!(bus.topic_record_count("inbox:bob").unwrap(), 0);
        // Unknown topic returns 0, not an error — caller-friendly for prefix aggregation.
        assert_eq!(bus.topic_record_count("inbox:nobody").unwrap(), 0);
    }

    #[tokio::test]
    async fn post_then_subscribe_roundtrip() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let o0 = bus.post("t", &env("t", b"alpha")).await.unwrap();
        let o1 = bus.post("t", &env("t", b"beta")).await.unwrap();
        let o2 = bus.post("t", &env("t", b"gamma")).await.unwrap();
        assert_eq!((o0, o1, o2), (0, 1, 2));

        let got: Vec<(u64, Vec<u8>)> = bus
            .subscribe("t", 0)
            .unwrap()
            .map(|r| {
                let (off, e) = r.unwrap();
                (off, e.payload)
            })
            .collect();
        assert_eq!(
            got,
            vec![
                (0, b"alpha".to_vec()),
                (1, b"beta".to_vec()),
                (2, b"gamma".to_vec())
            ]
        );
    }

    #[tokio::test]
    async fn subscribe_advances_past_cursor() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..5 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        let offsets: Vec<u64> = bus
            .subscribe("t", 3)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        assert_eq!(offsets, vec![3, 4]);
    }

    #[tokio::test]
    async fn subscribe_empty_topic_is_empty_iter() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let count = bus.subscribe("t", 0).unwrap().count();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn post_unknown_topic_errors() {
        let (_dir, bus) = tmp_bus();
        let err = bus.post("t", &env("t", b"x")).await.unwrap_err();
        assert!(matches!(err, BusError::UnknownTopic(_)));
    }

    #[tokio::test]
    async fn cursor_persists_and_rounds_trip() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..3 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        assert_eq!(bus.get_cursor("sub-A", "t").unwrap(), None);
        bus.advance_cursor("sub-A", "t", 2).unwrap();
        assert_eq!(bus.get_cursor("sub-A", "t").unwrap(), Some(2));
        bus.advance_cursor("sub-A", "t", 3).unwrap();
        assert_eq!(bus.get_cursor("sub-A", "t").unwrap(), Some(3));
    }

    #[tokio::test]
    async fn sweep_retention_messages_keeps_last_n() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Messages(2)).unwrap();
        for i in 0..5 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        let pruned = bus.sweep("t", 0).unwrap();
        assert_eq!(pruned, 3);
        let offsets: Vec<u64> = bus
            .subscribe("t", 0)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        assert_eq!(offsets, vec![3, 4]);
    }

    #[tokio::test]
    async fn sweep_retention_days_keeps_fresh() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Days(1)).unwrap();
        let day_ms: i64 = 86_400_000;
        let now: i64 = 10 * day_ms;
        let old = Envelope {
            ts_unix_ms: now - 2 * day_ms,
            ..env("t", b"old")
        };
        let fresh = Envelope {
            ts_unix_ms: now,
            ..env("t", b"fresh")
        };
        bus.post("t", &old).await.unwrap();
        bus.post("t", &fresh).await.unwrap();
        let pruned = bus.sweep("t", now).unwrap();
        assert_eq!(pruned, 1);
        let payloads: Vec<Vec<u8>> = bus
            .subscribe("t", 0)
            .unwrap()
            .map(|r| r.unwrap().1.payload)
            .collect();
        assert_eq!(payloads, vec![b"fresh".to_vec()]);
    }

    #[tokio::test]
    async fn sweep_forever_is_noop() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..3 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        assert_eq!(bus.sweep("t", 0).unwrap(), 0);
        assert_eq!(bus.subscribe("t", 0).unwrap().count(), 3);
    }
}
