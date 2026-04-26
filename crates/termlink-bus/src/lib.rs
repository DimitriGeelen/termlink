//! TermLink agent channel bus (T-1155).
//!
//! Append-only per-topic log, per-recipient cursor store, and per-topic
//! retention. Hub embeds this crate as a passive library — net/RPC lives
//! in `termlink-hub` (T-1160 exposes the channel API).
//!
//! Wedge 1 (T-1158 scaffold): crate skeleton, `Envelope`, `Retention`,
//! `Bus::open`, `create_topic` + `list_topics` via SQLite metadata.
//! Log-append + subscribe + retention sweep land in follow-up wedges.

mod artifact_store;
mod envelope;
mod error;
mod log;
mod meta;
mod retention;

pub use artifact_store::{ArtifactStore, StreamingPutOutcome};
pub use envelope::Envelope;
pub use error::{BusError, Result};
pub use log::Offset;
pub use retention::Retention;

/// Iterator yielded by `Bus::subscribe` — one `(offset, envelope)` per
/// record, or a decode/IO error.
pub type SubscribeIter = Box<dyn Iterator<Item = Result<(Offset, Envelope)>> + Send>;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use tokio::sync::Notify;
use tokio::time::{timeout, Duration};

/// Channel bus instance. Backed by a per-topic append-only log under
/// `<root>/topics/` and a SQLite metadata sidecar at `<root>/meta.db`.
pub struct Bus {
    root: PathBuf,
    meta: meta::Meta,
    appenders: StdMutex<HashMap<String, Arc<log::LogAppender>>>,
    /// Per-topic notify primitive: `post` calls `notify_waiters()` after a
    /// successful append; `subscribe_blocking` listens on it. Enables
    /// long-poll without busy polling (T-1289 / T-243 dialog liveness).
    notifiers: StdMutex<HashMap<String, Arc<Notify>>>,
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
            notifiers: StdMutex::new(HashMap::new()),
        })
    }

    /// Get or lazily create the per-topic Notify. Cheap — Arc clone, no IO.
    fn notifier_for(&self, topic: &str) -> Arc<Notify> {
        let mut guard = self
            .notifiers
            .lock()
            .expect("notifiers mutex poisoned");
        if let Some(n) = guard.get(topic) {
            return n.clone();
        }
        let n = Arc::new(Notify::new());
        guard.insert(topic.to_string(), n.clone());
        n
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

    /// Destructive trim of `topic`. `before_offset=Some(N)` removes
    /// records with offset < N; `before_offset=None` removes ALL records.
    /// Returns count deleted. Index-only (log file bytes remain).
    /// Affects all subscribers — channel-backed equivalent of legacy
    /// `inbox.clear` semantics (T-1234 / T-1230a). For per-subscriber
    /// "mark as read" semantics use `advance_cursor` instead.
    pub fn trim_topic(&self, topic: &str, before_offset: Option<u64>) -> Result<u64> {
        self.meta.trim_records(topic, before_offset)
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
        let offset = self.meta.record_append(topic, byte_pos, length, env.ts_unix_ms)?;
        // T-1289: wake any subscribers blocked in `subscribe_blocking` for
        // this topic. No-op when there are no waiters; cheap atomic.
        self.notifier_for(topic).notify_waiters();
        Ok(offset)
    }

    /// Long-poll variant of `subscribe`. Behaves like `subscribe(topic, cursor)`
    /// when records >= `cursor` already exist. Otherwise waits up to
    /// `timeout_dur` for the next `post(topic, _)` to fire and re-checks.
    /// Returns an empty iterator on timeout (semantically equivalent to a
    /// snapshot subscribe that found nothing).
    ///
    /// Notes:
    /// - The waiter is registered BEFORE the second SQLite check to avoid
    ///   the lost-wakeup race (post arriving between first check and notify
    ///   registration).
    /// - Multiple concurrent waiters all wake on a single `notify_waiters()`,
    ///   matching `tokio::sync::Notify` broadcast semantics.
    ///
    /// T-1289 / T-243 (dialog liveness — channel.subscribe needs push-like
    /// latency for heartbeats and turn delivery).
    pub async fn subscribe_blocking(
        &self,
        topic: &str,
        cursor: Offset,
        timeout_dur: Duration,
    ) -> Result<SubscribeIter> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }

        // Fast path: already-available records.
        let records = self.meta.records_from(topic, cursor)?;
        if !records.is_empty() {
            return self.subscribe(topic, cursor);
        }

        // Slow path: register a waiter, then re-check (avoids lost-wakeup).
        let notify = self.notifier_for(topic);
        let notified = notify.notified();
        tokio::pin!(notified);

        // Re-check after registration in case `post` raced between the
        // first check and the `notified()` registration.
        let records = self.meta.records_from(topic, cursor)?;
        if !records.is_empty() {
            return self.subscribe(topic, cursor);
        }

        // Wait for either a post or the timeout.
        let _ = timeout(timeout_dur, notified.as_mut()).await;

        // Whether we were notified or timed out, re-read the SQLite index.
        // (Notify wakes ALL waiters, but a different topic/post may have
        // been the trigger; we always reconfirm via the durable index.)
        self.subscribe(topic, cursor)
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

    /// Smallest offset still live in `topic` (after retention sweeps), or
    /// `None` if the topic has zero records.
    ///
    /// Subscribers use this to detect that their cursor fell behind the
    /// retention window: if `cursor < oldest_offset(topic)`, the records in
    /// the gap were swept and the subscriber missed them. T-1285 / T-243
    /// (dialog.heartbeat reconnect must surface gaps, not silently skip turns).
    ///
    /// Returns `BusError::UnknownTopic` if the topic was never registered.
    pub fn oldest_offset(&self, topic: &str) -> Result<Option<Offset>> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        self.meta.oldest_offset(topic)
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

    fn appender_for(&self, topic: &str) -> Result<Arc<log::LogAppender>> {
        let mut guard = self.appenders.lock().expect("appenders mutex poisoned");
        if let Some(a) = guard.get(topic) {
            return Ok(a.clone());
        }
        let path = log::topic_log_path(&self.root, topic);
        let appender = Arc::new(log::LogAppender::open(&path)?);
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
            metadata: std::collections::BTreeMap::new(),
        }
    }

    #[tokio::test]
    async fn trim_topic_before_offset_then_full() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0u32..5 {
            bus.post("t", &env("t", &i.to_le_bytes())).await.unwrap();
        }
        assert_eq!(bus.topic_record_count("t").unwrap(), 5);
        // Trim records with offset < 3 → removes offsets 0,1,2 (3 records)
        let n = bus.trim_topic("t", Some(3)).unwrap();
        assert_eq!(n, 3);
        assert_eq!(bus.topic_record_count("t").unwrap(), 2);
        // Full trim → removes the remaining 2
        let n = bus.trim_topic("t", None).unwrap();
        assert_eq!(n, 2);
        assert_eq!(bus.topic_record_count("t").unwrap(), 0);
        // Unknown topic returns 0 (caller-friendly)
        assert_eq!(bus.trim_topic("nope", None).unwrap(), 0);
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
    async fn oldest_offset_reflects_sweep_drops() {
        // T-1285: subscribers that disconnect across a retention sweep must
        // be able to detect that their cursor fell behind. oldest_offset()
        // is the cheap signal: cursor < oldest_offset == gap.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Messages(2)).unwrap();

        // No records yet — None.
        assert_eq!(bus.oldest_offset("t").unwrap(), None);

        for i in 0..5 {
            bus.post("t", &env("t", format!("m{i}").as_bytes()))
                .await
                .unwrap();
        }
        // 5 posts, offsets 0..=4, oldest = 0.
        assert_eq!(bus.oldest_offset("t").unwrap(), Some(0));

        // Sweep — Retention::Messages(2) keeps offsets 3,4.
        let pruned = bus.sweep("t", 0).unwrap();
        assert_eq!(pruned, 3);

        // Subscriber reconnecting with cursor=1 must be able to see that
        // oldest live offset is now 3 → records at offsets 1,2 are gone.
        let oldest = bus.oldest_offset("t").unwrap();
        assert_eq!(oldest, Some(3));
        let cursor: u64 = 1;
        assert!(cursor < oldest.unwrap(), "gap must be detectable");

        // After full trim: None signals empty topic, distinct from "no gap".
        bus.trim_topic("t", None).unwrap();
        assert_eq!(bus.oldest_offset("t").unwrap(), None);
    }

    #[tokio::test]
    async fn subscribe_blocking_returns_existing_records_immediately() {
        // T-1289: fast path — records already present, return without
        // touching the Notify.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"a")).await.unwrap();
        bus.post("t", &env("t", b"b")).await.unwrap();

        let start = std::time::Instant::now();
        let got: Vec<u64> = bus
            .subscribe_blocking("t", 0, Duration::from_secs(5))
            .await
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        let elapsed = start.elapsed();

        assert_eq!(got, vec![0, 1]);
        // Should return effectively instantly (well under the 5s timeout).
        assert!(
            elapsed < Duration::from_millis(500),
            "fast path took {elapsed:?}, expected < 500ms"
        );
    }

    #[tokio::test]
    async fn subscribe_blocking_wakes_on_concurrent_post() {
        // T-1289 core proof: subscriber blocked on empty topic gets woken
        // when a producer posts. The latency from post to wake should be
        // small (push-like), nowhere near the timeout.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let bus = std::sync::Arc::new(bus);

        let producer = {
            let bus = bus.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                bus.post("t", &env("t", b"hello")).await.unwrap();
            })
        };

        let start = std::time::Instant::now();
        let got: Vec<Vec<u8>> = bus
            .subscribe_blocking("t", 0, Duration::from_secs(5))
            .await
            .unwrap()
            .map(|r| r.unwrap().1.payload)
            .collect();
        let elapsed = start.elapsed();
        producer.await.unwrap();

        assert_eq!(got, vec![b"hello".to_vec()]);
        // Producer slept 50ms before posting, then wake + read should be
        // well under 1s — push-like latency.
        assert!(
            elapsed < Duration::from_secs(1),
            "wake-on-post took {elapsed:?}, expected < 1s"
        );
        assert!(
            elapsed >= Duration::from_millis(50),
            "wake-on-post finished too fast ({elapsed:?}); expected >= 50ms"
        );
    }

    #[tokio::test]
    async fn subscribe_blocking_returns_empty_on_timeout() {
        // T-1289: timeout case — empty iterator, no error. Caller treats
        // this identically to a snapshot subscribe with no available records.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();

        let start = std::time::Instant::now();
        let count = bus
            .subscribe_blocking("t", 0, Duration::from_millis(100))
            .await
            .unwrap()
            .count();
        let elapsed = start.elapsed();

        assert_eq!(count, 0);
        // Should wait at least the timeout — proves we actually blocked.
        assert!(
            elapsed >= Duration::from_millis(100),
            "timeout fired too early at {elapsed:?}"
        );
        // Don't wait far past the timeout (allow ~200ms slack for scheduler).
        assert!(
            elapsed < Duration::from_millis(500),
            "timeout took {elapsed:?}, expected < 500ms"
        );
    }

    #[tokio::test]
    async fn subscribe_blocking_unknown_topic_errors() {
        let (_dir, bus) = tmp_bus();
        // SubscribeIter is `Box<dyn Iterator>` which doesn't implement Debug,
        // so .unwrap_err() can't synthesize a panic message — match instead.
        match bus
            .subscribe_blocking("nope", 0, Duration::from_millis(50))
            .await
        {
            Err(BusError::UnknownTopic(_)) => {}
            Err(e) => panic!("unexpected error variant: {e:?}"),
            Ok(_) => panic!("expected UnknownTopic error, got iterator"),
        }
    }

    #[tokio::test]
    async fn oldest_offset_unknown_topic_errors() {
        let (_dir, bus) = tmp_bus();
        let err = bus.oldest_offset("nope").unwrap_err();
        assert!(matches!(err, BusError::UnknownTopic(_)));
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
