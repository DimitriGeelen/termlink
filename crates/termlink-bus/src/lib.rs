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
mod claim;
mod envelope;
mod error;
mod log;
mod meta;
mod retention;

pub use artifact_store::{ArtifactStore, StreamingPutOutcome};
pub use claim::{ClaimInfo, ClaimsSummary, ReleaseInfo};
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
    ///
    /// Returns `Ok(true)` when the topic was created by this call,
    /// `Ok(false)` when a matching topic already existed. T-1429.5 added
    /// the bool so callers can do "describe-on-first-create" without
    /// re-emitting topic_metadata envelopes on every idempotent re-call.
    pub fn create_topic(&self, name: &str, retention: Retention) -> Result<bool> {
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

    /// T-2029 (arc-parallel-substrate Slice 1): claim `(topic, offset)` for
    /// exclusive processing by `claimer` for the next `ttl_ms` milliseconds.
    /// Returns `BusError::ClaimConflict` when another worker holds an
    /// unexpired claim on the same offset; returns `BusError::UnknownTopic`
    /// when the topic was never registered. Lazy expiry runs inline — no
    /// background reaper (T-1155 invariant).
    pub fn claim_offset(
        &self,
        topic: &str,
        offset: Offset,
        claimer: &str,
        ttl_ms: u32,
    ) -> Result<ClaimInfo> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let now_ms = now_unix_ms();
        self.meta.claim_offset(topic, offset, claimer, ttl_ms, now_ms)
    }

    /// T-2029: release a claim. When `ack=true` the claimer's cursor for
    /// the topic advances past the claimed offset (a subsequent
    /// `get_cursor(claimer, topic)` returns `Some(offset+1)`); when
    /// `ack=false` the cursor is left untouched and another worker can
    /// reclaim the same offset. Returns `BusError::ClaimNotFound` for an
    /// unknown / already-released claim and `BusError::ClaimNotOwned` when
    /// the caller is not the original claimer.
    pub fn release_claim(
        &self,
        claim_id: &str,
        claimer: &str,
        ack: bool,
    ) -> Result<ReleaseInfo> {
        self.meta.release_claim(claim_id, claimer, ack)
    }

    /// T-2030 (arc-parallel-substrate Slice 2): extend the lease on a held
    /// claim by `additional_ttl_ms` past *now*. Used by long-running
    /// workers to retain ownership before `claimed_until` lapses.
    ///
    /// Returns the refreshed `ClaimInfo` (same `claim_id`, same `topic`,
    /// same `offset`, same `claimed_at`, new `claimed_until`).
    ///
    /// Errors:
    /// - `BusError::ClaimNotFound` — unknown / already-released `claim_id`.
    /// - `BusError::ClaimExpired` — row exists but `claimed_until <= now`
    ///   (lazily evicted in the same call so the slot becomes claimable).
    /// - `BusError::ClaimNotOwned` — caller is not the original claimer.
    pub fn renew_claim(
        &self,
        claim_id: &str,
        claimer: &str,
        additional_ttl_ms: u32,
    ) -> Result<ClaimInfo> {
        let now_ms = now_unix_ms();
        self.meta
            .renew_claim(claim_id, claimer, additional_ttl_ms, now_ms)
    }

    /// T-2037 (arc-parallel-substrate Slice 4): list current claim rows for
    /// `topic`. When `include_expired=false` (default at the protocol layer),
    /// rows whose `claimed_until` is in the past are filtered out. Returns
    /// `BusError::UnknownTopic` when the topic was never registered (mirrors
    /// `claim_offset`'s discoverability contract); returns an empty vec when
    /// the topic exists but has no live claims.
    ///
    /// Pure read — no SQL writes, no cursor mutation, no lazy eviction. The
    /// caller chooses whether to surface expired rows for operator forensics.
    pub fn list_claims(&self, topic: &str, include_expired: bool) -> Result<Vec<ClaimInfo>> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let now_ms = now_unix_ms();
        self.meta.list_claims(topic, include_expired, now_ms)
    }

    /// T-2039 (arc-parallel-substrate Slice 6): aggregate claim state for
    /// `topic`. Returns counts (active/expired) plus the oldest-active and
    /// next-expiry markers — operator observability companion to
    /// `list_claims` for "is this topic busy?" and "is anything stuck?"
    /// queries without paying full-list transfer cost.
    ///
    /// Returns `BusError::UnknownTopic` when the topic was never registered
    /// (mirrors `list_claims`); returns a `ClaimsSummary` with zero counts
    /// and `None` markers when the topic exists but has no claim rows.
    /// Pure read — no SQL writes, no cursor mutation, no lazy eviction.
    pub fn claims_summary(&self, topic: &str) -> Result<ClaimsSummary> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let now_ms = now_unix_ms();
        self.meta.claims_summary(topic, now_ms)
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

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
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

    // ── T-2029 (arc-parallel-substrate Slice 1): claim semantics ──

    #[tokio::test]
    async fn claim_offset_is_exclusive_per_topic_offset() {
        // Two workers race for the same (topic, offset). Exactly one wins;
        // the other sees ClaimConflict. Disjoint offsets remain claimable
        // independently — claims are per-(topic, offset), not per-topic.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        for i in 0..3u32 {
            bus.post("work", &env("work", &i.to_le_bytes())).await.unwrap();
        }
        let c1 = bus.claim_offset("work", 1, "worker-A", 30_000).unwrap();
        assert_eq!(c1.offset, 1);
        assert_eq!(c1.claimer, "worker-A");
        let err = bus
            .claim_offset("work", 1, "worker-B", 30_000)
            .unwrap_err();
        assert!(
            matches!(err, BusError::ClaimConflict { ref topic, offset: 1 } if topic == "work"),
            "expected ClaimConflict, got {err:?}"
        );
        // worker-B can still grab a different offset.
        let c2 = bus.claim_offset("work", 2, "worker-B", 30_000).unwrap();
        assert_eq!(c2.offset, 2);
        assert_eq!(c2.claimer, "worker-B");
    }

    #[tokio::test]
    async fn release_with_ack_true_advances_claimers_cursor() {
        // Ack-on-release is the worker's "I processed this — don't ever
        // hand it back to me" signal. The cursor for THIS claimer (and
        // only this claimer) advances past the claimed offset.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        for i in 0..3u32 {
            bus.post("work", &env("work", &i.to_le_bytes())).await.unwrap();
        }
        assert_eq!(bus.get_cursor("worker-A", "work").unwrap(), None);
        let c = bus.claim_offset("work", 1, "worker-A", 30_000).unwrap();
        let r = bus.release_claim(&c.claim_id, "worker-A", true).unwrap();
        assert_eq!(r.offset, 1);
        assert!(r.ack);
        // Cursor advanced past offset 1 → next subscribe with cursor=2 skips it.
        assert_eq!(bus.get_cursor("worker-A", "work").unwrap(), Some(2));
        // Other workers' cursors untouched.
        assert_eq!(bus.get_cursor("worker-B", "work").unwrap(), None);
    }

    #[tokio::test]
    async fn release_with_ack_false_does_not_advance_and_frees_slot() {
        // No-ack release is the "work was returned, not done" signal:
        // cursor stays where it was, and the slot becomes claimable again
        // (e.g. by a peer worker, or by the same worker on retry).
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        for i in 0..3u32 {
            bus.post("work", &env("work", &i.to_le_bytes())).await.unwrap();
        }
        let c = bus.claim_offset("work", 1, "worker-A", 30_000).unwrap();
        let r = bus
            .release_claim(&c.claim_id, "worker-A", false)
            .unwrap();
        assert_eq!(r.offset, 1);
        assert!(!r.ack);
        assert_eq!(bus.get_cursor("worker-A", "work").unwrap(), None);
        // Slot is free — worker-B can now claim it.
        let c2 = bus.claim_offset("work", 1, "worker-B", 30_000).unwrap();
        assert_eq!(c2.claimer, "worker-B");
    }

    #[tokio::test]
    async fn lazy_expiry_allows_reclaim_after_ttl_lapses() {
        // No background reaper (T-1155 no-background-threads invariant) —
        // expired claims are evicted on the next claim attempt that hits
        // the same (topic, offset). A 1ms TTL plus a 20ms sleep guarantees
        // we cross the boundary regardless of system clock resolution.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let first = bus.claim_offset("work", 0, "worker-A", 1).unwrap();
        assert_eq!(first.claimer, "worker-A");
        tokio::time::sleep(Duration::from_millis(20)).await;
        // Worker-B's claim sweeps worker-A's expired row and inserts its own.
        let second = bus.claim_offset("work", 0, "worker-B", 30_000).unwrap();
        assert_eq!(second.claimer, "worker-B");
        assert_ne!(second.claim_id, first.claim_id);
        // worker-A trying to release its (now-evicted) claim sees NotFound.
        let err = bus
            .release_claim(&first.claim_id, "worker-A", true)
            .unwrap_err();
        assert!(matches!(err, BusError::ClaimNotFound(_)), "got {err:?}");
    }

    // ── T-2030 (arc-parallel-substrate Slice 2): renew semantics ──

    #[tokio::test]
    async fn renew_claim_extends_claimed_until_past_original_deadline() {
        // Worker grabs a slot with a short initial TTL; before it lapses
        // they renew with a longer additional_ttl_ms. The refreshed
        // claimed_until must reflect now + additional_ttl_ms — strictly
        // beyond the original deadline.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let initial = bus.claim_offset("work", 0, "worker-A", 200).unwrap();
        // Renew with a 60s extension. The new claimed_until must be at
        // least old_until + 100ms (i.e. the renewal genuinely pushed it
        // forward, not just re-confirmed the same deadline).
        let renewed = bus.renew_claim(&initial.claim_id, "worker-A", 60_000).unwrap();
        assert_eq!(renewed.claim_id, initial.claim_id);
        assert_eq!(renewed.offset, 0);
        assert_eq!(renewed.claimer, "worker-A");
        assert!(
            renewed.claimed_until > initial.claimed_until + 100,
            "expected new claimed_until > old + 100ms, \
             old={}, new={}",
            initial.claimed_until,
            renewed.claimed_until,
        );
    }

    #[tokio::test]
    async fn renew_on_expired_claim_returns_claim_expired() {
        // 1ms TTL + 20ms sleep guarantees we cross the deadline. Renew
        // must refuse with ClaimExpired (NOT ClaimNotFound — the client
        // needs to distinguish "your lease lapsed, fetch a fresh claim"
        // from "wrong id"). The stale row is lazily evicted so a
        // subsequent claim_offset for the same (topic, offset) succeeds.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "worker-A", 1).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        let err = bus
            .renew_claim(&c.claim_id, "worker-A", 60_000)
            .unwrap_err();
        assert!(matches!(err, BusError::ClaimExpired { .. }), "got {err:?}");
        // Slot is free again — eviction was part of the renew path.
        let recl = bus.claim_offset("work", 0, "worker-B", 30_000).unwrap();
        assert_eq!(recl.claimer, "worker-B");
    }

    #[tokio::test]
    async fn renew_by_non_owner_returns_claim_not_owned() {
        // worker-A claims; worker-B tries to renew with worker-A's
        // claim_id. Must fail with ClaimNotOwned — a worker cannot
        // hijack another worker's lease just by knowing its id.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "worker-A", 30_000).unwrap();
        let err = bus
            .renew_claim(&c.claim_id, "worker-B", 60_000)
            .unwrap_err();
        match err {
            BusError::ClaimNotOwned {
                ref claimed_by,
                ref attempted_by,
                ..
            } => {
                assert_eq!(claimed_by, "worker-A");
                assert_eq!(attempted_by, "worker-B");
            }
            other => panic!("expected ClaimNotOwned, got {other:?}"),
        }
        // Original claim is untouched — worker-A can still renew or release.
        let renewed = bus.renew_claim(&c.claim_id, "worker-A", 30_000).unwrap();
        assert_eq!(renewed.claim_id, c.claim_id);
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

    // ── T-2039 (arc-parallel-substrate Slice 6): claims_summary aggregate ──
    //
    // These exercise the real SQLite aggregate (COALESCE(SUM(CASE ...)) +
    // MIN(CASE ...)) directly — the integration tests at
    // `crates/termlink-session/tests/claim_client_integration.rs` use a
    // FakeHub that re-derives the markers approximately, so the real SQL
    // logic only gets exercised here.

    #[tokio::test]
    async fn claims_summary_unknown_topic_returns_error() {
        let (_dir, bus) = tmp_bus();
        let err = bus.claims_summary("ghost").unwrap_err();
        assert!(matches!(err, BusError::UnknownTopic(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn claims_summary_topic_with_no_claims_returns_zero_counts() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"m0")).await.unwrap();
        let s = bus.claims_summary("t").unwrap();
        assert_eq!(s.active_count, 0);
        assert_eq!(s.expired_count, 0);
        assert!(s.oldest_active_at_ms.is_none());
        assert!(s.oldest_active_age_ms.is_none());
        assert!(s.next_active_expiry_ms.is_none());
    }

    #[tokio::test]
    async fn claims_summary_single_active_claim_populates_all_markers() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"m0")).await.unwrap();
        let c = bus.claim_offset("t", 0, "worker-A", 30_000).unwrap();
        let s = bus.claims_summary("t").unwrap();
        assert_eq!(s.active_count, 1);
        assert_eq!(s.expired_count, 0);
        assert_eq!(s.oldest_active_at_ms, Some(c.claimed_at));
        assert!(s.oldest_active_age_ms.is_some());
        assert!(s.oldest_active_age_ms.unwrap() >= 0);
        assert_eq!(s.next_active_expiry_ms, Some(c.claimed_until));
    }

    #[tokio::test]
    async fn claims_summary_mixed_active_and_expired_partitions_correctly() {
        // Three offsets: two get short-TTL claims that lapse, one gets a
        // long-TTL claim that stays live. After the sleep, claims_summary
        // must report exactly active=1, expired=2. Tests the real SQL
        // CASE-partitioning that the integration tests can't fake.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..3 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        let _short_a = bus.claim_offset("t", 0, "worker-A", 1).unwrap();
        let _short_b = bus.claim_offset("t", 1, "worker-B", 1).unwrap();
        let long = bus.claim_offset("t", 2, "worker-C", 60_000).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        let s = bus.claims_summary("t").unwrap();
        assert_eq!(s.active_count, 1, "only worker-C should be active");
        assert_eq!(s.expired_count, 2, "worker-A + worker-B should be expired");
        assert_eq!(
            s.oldest_active_at_ms,
            Some(long.claimed_at),
            "oldest_active_at must point at the only live claim, not the expired ones"
        );
        assert_eq!(s.next_active_expiry_ms, Some(long.claimed_until));
    }

    #[tokio::test]
    async fn claims_summary_only_expired_claims_returns_none_markers() {
        // Edge case the integration tests can't construct cleanly: every
        // row on the topic is past its deadline. Markers must be None
        // (MIN(CASE WHEN active THEN ... ELSE NULL END) returns NULL) and
        // expired_count must show the lapsed row.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"m0")).await.unwrap();
        let _c = bus.claim_offset("t", 0, "worker-A", 1).unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        let s = bus.claims_summary("t").unwrap();
        assert_eq!(s.active_count, 0);
        assert_eq!(s.expired_count, 1);
        assert!(s.oldest_active_at_ms.is_none());
        assert!(s.oldest_active_age_ms.is_none());
        assert!(s.next_active_expiry_ms.is_none());
    }

    #[tokio::test]
    async fn claims_summary_oldest_active_marker_picks_the_earliest_claimed_at() {
        // Two live claims with different claimed_at timestamps. The
        // oldest_active_at_ms must equal the EARLIEST one (MIN
        // semantics), and next_active_expiry_ms must equal the EARLIEST
        // claimed_until (the next slot to free up). With identical TTLs,
        // claimed-at ordering and claimed-until ordering coincide, so we
        // check both pointers track the FIRST claim.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..2 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        let first = bus.claim_offset("t", 0, "worker-A", 60_000).unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;
        let second = bus.claim_offset("t", 1, "worker-B", 60_000).unwrap();
        let s = bus.claims_summary("t").unwrap();
        assert_eq!(s.active_count, 2);
        assert_eq!(s.expired_count, 0);
        assert_eq!(
            s.oldest_active_at_ms,
            Some(first.claimed_at.min(second.claimed_at)),
            "must point at the earliest-claimed-at row"
        );
        assert_eq!(
            s.next_active_expiry_ms,
            Some(first.claimed_until.min(second.claimed_until)),
            "must point at the earliest-claimed-until row (next slot to free up)"
        );
    }
}
