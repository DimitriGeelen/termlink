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

    /// Append an envelope to `topic`'s log. Returns the logical offset
    /// (0-based sequence number) assigned to the new record. The topic
    /// must have been registered via `create_topic` first.
    pub async fn post(&self, topic: &str, env: &Envelope) -> Result<Offset> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let appender = self.appender_for(topic)?;
        let bytes = log::encode_envelope(env)?;
        appender.append(&bytes).await?;
        self.meta.next_offset(topic)
    }

    /// Subscribe to `topic` starting at logical offset `cursor`. Returns
    /// an iterator yielding `(offset, envelope)` for every record whose
    /// offset is >= `cursor`. Empty topics yield an empty iterator.
    pub fn subscribe(&self, topic: &str, cursor: Offset) -> Result<SubscribeIter> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let path = log::topic_log_path(&self.root, topic);
        let reader = log::LogReader::open(&path)?;
        let Some(reader) = reader else {
            return Ok(Box::new(std::iter::empty()));
        };
        let iter = reader.enumerate().filter_map(move |(idx, res)| {
            let offset = idx as Offset;
            if offset < cursor {
                return None;
            }
            Some(res.and_then(|bytes| log::decode_envelope(&bytes).map(|env| (offset, env))))
        });
        Ok(Box::new(iter))
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
}
