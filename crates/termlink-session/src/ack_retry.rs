//! Client-side ack-with-retry helper (T-2286, Design A of T-2285).
//!
//! Closes the producer half of §9 hard-dependency #5 (the parallel-execution
//! harness's "sender detects a dead recipient and retries"). The T-2285
//! inception established that this needs **no hub-side delivery state**:
//!
//!   * the exactly-once leg already exists — hub-side `(sender_id,
//!     client_msg_id)` dedupe (T-2049, `termlink-hub/src/dedupe.rs`);
//!   * the durability pattern already exists — the SQLite offline queue
//!     (T-2051, [`crate::offline_queue`]);
//!   * the recipient-ack signal already exists — the `channel.receipts`
//!     frontier (`up_to >= offset`).
//!
//! This module contributes the missing producer-side piece:
//!
//!   1. [`AwaitingAckTracker`] — a durable record of posts still awaiting a
//!      recipient ack, so an await that is interrupted by a client restart
//!      can be resumed rather than silently abandoned. Mirrors
//!      [`crate::offline_queue::OfflineQueue`] (rusqlite, single-writer
//!      `Mutex<Connection>`, `TERMLINK_IDENTITY_DIR`-aware path).
//!   2. [`await_ack_with_retry`] — the retry loop: post with a **stable**
//!      `client_msg_id`, poll the receipt frontier until the recipient acks
//!      or a per-attempt deadline elapses, then **re-post the SAME
//!      `client_msg_id`** (≤ `max_attempts`). T-2049 dedupe absorbs the
//!      duplicate, so a retry-after-dead-recipient is exactly-once.
//!
//! The loop is generic over its side effects (post / receipts / clock /
//! sleep) so the exactly-once invariant is unit-testable with no hub — the
//! CLI wires real closures via [`await_ack_with_retry_realtime`].
//!
//! The recipient half is an AEF-layer convention, NOT a substrate feature:
//! after consuming a message the harness sidecar emits `channel.ack
//! up_to=<offset>`. See `docs/operations/substrate-ack-with-retry.md`.

use std::future::Future;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

// ───────────────────────────── retry policy ─────────────────────────────

/// Defaults align with AEF ADR §6 heartbeat numbers (T-2285 "co-discover
/// with AEF" open item): the sidecar heartbeats on a ~5s tick and is judged
/// stale after ~30s, so polling the ack frontier every 5s with a 30s
/// per-attempt deadline matches the cadence at which a real ack can arrive.
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 5_000;
pub const DEFAULT_DEADLINE_MS: u64 = 30_000;
pub const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Tunables for [`await_ack_with_retry`]. All three are documented as
/// operator-tunable (CLI flags `--ack-timeout-secs` / `--max-attempts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// How often to re-read the receipt frontier within one attempt.
    pub poll_interval_ms: u64,
    /// Per-attempt window. When it elapses with no ack, the post is
    /// re-sent (reusing the same `client_msg_id`) up to `max_attempts`.
    pub deadline_ms: u64,
    /// Total number of post attempts, including the first. `1` disables
    /// retry (post-then-wait-once). Clamped to `>= 1`.
    pub max_attempts: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
            deadline_ms: DEFAULT_DEADLINE_MS,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }
}

impl RetryPolicy {
    /// Build from an ack-timeout (seconds, per attempt) and max attempts,
    /// the two operator-facing knobs. Keeps the 5s poll cadence. A zero
    /// timeout falls back to the default deadline; attempts clamp to `>=1`.
    pub fn from_operator(ack_timeout_secs: u64, max_attempts: u32) -> Self {
        let deadline_ms = if ack_timeout_secs == 0 {
            DEFAULT_DEADLINE_MS
        } else {
            ack_timeout_secs.saturating_mul(1_000)
        };
        Self {
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
            deadline_ms,
            max_attempts: max_attempts.max(1),
        }
    }
}

// ───────────────────────────── receipt row ──────────────────────────────

/// One `(sender_id, up_to)` pair from a `channel.receipts` response. The
/// helper only needs these two fields; `ts_unix_ms` is dropped at the call
/// boundary. A recipient has acked offset `N` once any of its receipt rows
/// reports `up_to >= N`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptRow {
    pub sender_id: String,
    pub up_to: u64,
}

/// True if `receipts` show `recipient` has acked through `offset`.
fn recipient_acked(receipts: &[ReceiptRow], recipient: &str, offset: u64) -> bool {
    receipts
        .iter()
        .filter(|r| r.sender_id == recipient)
        .any(|r| r.up_to >= offset)
}

// ───────────────────────────── outcome / error ──────────────────────────

/// Result of an [`await_ack_with_retry`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckOutcome {
    /// The recipient acked through the posted offset. The tracker row has
    /// been confirmed (deleted).
    Acked { offset: u64, attempts: u32 },
    /// Every attempt's deadline elapsed without an ack. The tracker row is
    /// retained (durable) for an operator/recovery sweep to act on. The
    /// post itself was delivered to the hub — only the *recipient* ack is
    /// outstanding.
    Exhausted { offset: u64, attempts: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum AckRetryError {
    #[error("post failed: {0}")]
    Post(String),
    #[error("receipts read failed: {0}")]
    Receipts(String),
    #[error("tracker error: {0}")]
    Tracker(#[from] TrackerError),
}

// ───────────────────────────── the retry loop ───────────────────────────

/// Post `client_msg_id` and retry until the recipient acks or attempts run
/// out. Generic over its side effects so the exactly-once invariant is
/// testable without a hub:
///
///   * `post_fn(client_msg_id) -> offset` — posts (first call) or re-posts
///     (retries). Because the id is stable, a hub honouring T-2049 dedupe
///     returns the SAME offset and appends nothing the second time.
///   * `receipts_fn() -> Vec<ReceiptRow>` — reads the `channel.receipts`
///     frontier.
///   * `now_ms` / `sleep_fn` — injectable clock so tests advance time
///     instantly and deterministically.
///
/// The tracker row is recorded before the first wait and confirmed
/// (deleted) on ack; on exhaustion it is left in place (durable).
#[allow(clippy::too_many_arguments)]
pub async fn await_ack_with_retry<PF, PFut, RF, RFut, NF, SF, SFut>(
    tracker: &AwaitingAckTracker,
    dm_topic: &str,
    recipient_sender_id: &str,
    client_msg_id: &str,
    policy: &RetryPolicy,
    mut post_fn: PF,
    mut receipts_fn: RF,
    mut now_ms: NF,
    mut sleep_fn: SF,
) -> std::result::Result<AckOutcome, AckRetryError>
where
    PF: FnMut(String) -> PFut,
    PFut: Future<Output = std::result::Result<u64, String>>,
    RF: FnMut() -> RFut,
    RFut: Future<Output = std::result::Result<Vec<ReceiptRow>, String>>,
    NF: FnMut() -> i64,
    SF: FnMut(u64) -> SFut,
    SFut: Future<Output = ()>,
{
    let max_attempts = policy.max_attempts.max(1);
    let mut offset: Option<u64> = None;
    let mut recorded = false;

    for attempt in 1..=max_attempts {
        // Post (attempt 1) or re-post (retries). The SAME client_msg_id is
        // reused every time — that is what makes the retry exactly-once.
        let posted = post_fn(client_msg_id.to_string())
            .await
            .map_err(AckRetryError::Post)?;
        offset = Some(posted);

        // Record durably on the first successful post so a crash mid-await
        // is recoverable; bump the attempt counter on each re-post.
        if !recorded {
            tracker.record(dm_topic, posted, client_msg_id, recipient_sender_id, now_ms())?;
            recorded = true;
        } else {
            tracker.bump_attempts(client_msg_id)?;
        }

        // Poll the receipt frontier until ack or this attempt's deadline.
        let deadline = now_ms().saturating_add(policy.deadline_ms as i64);
        loop {
            let receipts = receipts_fn().await.map_err(AckRetryError::Receipts)?;
            if recipient_acked(&receipts, recipient_sender_id, posted) {
                tracker.confirm(client_msg_id)?;
                return Ok(AckOutcome::Acked { offset: posted, attempts: attempt });
            }
            if now_ms() >= deadline {
                break; // deadline hit — fall through to retry (or exhaust)
            }
            sleep_fn(policy.poll_interval_ms).await;
        }
    }

    Ok(AckOutcome::Exhausted {
        offset: offset.unwrap_or(0),
        attempts: max_attempts,
    })
}

/// Real-time wrapper used by the CLI: fills in the system clock and
/// `tokio::time::sleep`. Tests call [`await_ack_with_retry`] directly with
/// a fake clock.
#[allow(clippy::too_many_arguments)]
pub async fn await_ack_with_retry_realtime<PF, PFut, RF, RFut>(
    tracker: &AwaitingAckTracker,
    dm_topic: &str,
    recipient_sender_id: &str,
    client_msg_id: &str,
    policy: &RetryPolicy,
    post_fn: PF,
    receipts_fn: RF,
) -> std::result::Result<AckOutcome, AckRetryError>
where
    PF: FnMut(String) -> PFut,
    PFut: Future<Output = std::result::Result<u64, String>>,
    RF: FnMut() -> RFut,
    RFut: Future<Output = std::result::Result<Vec<ReceiptRow>, String>>,
{
    await_ack_with_retry(
        tracker,
        dm_topic,
        recipient_sender_id,
        client_msg_id,
        policy,
        post_fn,
        receipts_fn,
        now_unix_ms,
        |ms| tokio::time::sleep(Duration::from_millis(ms)),
    )
    .await
}

// ─────────────────────────── awaiting-ack tracker ───────────────────────

#[derive(Debug, thiserror::Error)]
pub enum TrackerError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type TrackerResult<T> = std::result::Result<T, TrackerError>;

/// File name of the on-disk tracker under `~/.termlink/`.
pub const DEFAULT_FILE_NAME: &str = "awaiting_ack.sqlite";

/// Resolve the default tracker path: `$TERMLINK_IDENTITY_DIR/awaiting_ack.sqlite`
/// when that env is set (per-fleet test isolation, mirrors
/// [`crate::offline_queue::default_queue_path`]), else
/// `$HOME/.termlink/awaiting_ack.sqlite`.
pub fn default_tracker_path() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("TERMLINK_IDENTITY_DIR") {
        return std::path::PathBuf::from(dir).join(DEFAULT_FILE_NAME);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(home).join(".termlink").join(DEFAULT_FILE_NAME)
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS awaiting_ack (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    dm_topic            TEXT    NOT NULL,
    msg_offset          INTEGER NOT NULL,
    client_msg_id       TEXT    NOT NULL,
    recipient_sender_id TEXT    NOT NULL,
    attempts            INTEGER NOT NULL DEFAULT 1,
    enqueued_ms         INTEGER NOT NULL
);
-- client_msg_id is the stable idempotency key (T-2049); one awaiting-ack
-- row per outstanding post. UNIQUE so a re-record is an upsert, never a dup.
CREATE UNIQUE INDEX IF NOT EXISTS awaiting_ack_cmid ON awaiting_ack(client_msg_id);
"#;

/// One outstanding post awaiting a recipient ack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwaitingRow {
    pub dm_topic: String,
    pub msg_offset: u64,
    pub client_msg_id: String,
    pub recipient_sender_id: String,
    pub attempts: u64,
    pub enqueued_ms: i64,
}

/// Durable record of posts still awaiting a recipient ack. SQLite-backed
/// for crash recovery; single-writer via `Mutex<Connection>` (rusqlite is
/// not `Sync`) — same discipline as [`crate::offline_queue::OfflineQueue`].
pub struct AwaitingAckTracker {
    conn: Mutex<Connection>,
}

impl AwaitingAckTracker {
    /// Open (or create) the tracker at `path`, creating parent dirs.
    pub fn open(path: impl AsRef<Path>) -> TrackerResult<Self> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(p)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Open an in-memory tracker (tests only).
    pub fn open_in_memory() -> TrackerResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Number of outstanding awaiting-ack rows.
    pub fn size(&self) -> TrackerResult<u64> {
        let conn = self.conn.lock().expect("tracker mutex poisoned");
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM awaiting_ack", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    /// Record a post now awaiting an ack. Idempotent on `client_msg_id`:
    /// re-recording the same id refreshes the row (the post was re-sent)
    /// rather than inserting a duplicate.
    pub fn record(
        &self,
        dm_topic: &str,
        msg_offset: u64,
        client_msg_id: &str,
        recipient_sender_id: &str,
        now_ms: i64,
    ) -> TrackerResult<()> {
        let conn = self.conn.lock().expect("tracker mutex poisoned");
        conn.execute(
            "INSERT INTO awaiting_ack \
                 (dm_topic, msg_offset, client_msg_id, recipient_sender_id, attempts, enqueued_ms) \
             VALUES (?1, ?2, ?3, ?4, 1, ?5) \
             ON CONFLICT(client_msg_id) DO UPDATE SET \
                 msg_offset = excluded.msg_offset, \
                 dm_topic = excluded.dm_topic, \
                 recipient_sender_id = excluded.recipient_sender_id",
            params![dm_topic, msg_offset as i64, client_msg_id, recipient_sender_id, now_ms],
        )?;
        Ok(())
    }

    /// Increment the attempt counter for an outstanding row (a re-post).
    pub fn bump_attempts(&self, client_msg_id: &str) -> TrackerResult<()> {
        let conn = self.conn.lock().expect("tracker mutex poisoned");
        conn.execute(
            "UPDATE awaiting_ack SET attempts = attempts + 1 WHERE client_msg_id = ?1",
            params![client_msg_id],
        )?;
        Ok(())
    }

    /// Confirm (delete) an awaiting-ack row — the recipient acked. No-op if
    /// already gone (forgiving, like [`crate::offline_queue::OfflineQueue::pop`]).
    pub fn confirm(&self, client_msg_id: &str) -> TrackerResult<()> {
        let conn = self.conn.lock().expect("tracker mutex poisoned");
        conn.execute(
            "DELETE FROM awaiting_ack WHERE client_msg_id = ?1",
            params![client_msg_id],
        )?;
        Ok(())
    }

    /// Fetch a single outstanding row by id, if present.
    pub fn get(&self, client_msg_id: &str) -> TrackerResult<Option<AwaitingRow>> {
        let conn = self.conn.lock().expect("tracker mutex poisoned");
        let row = conn
            .query_row(
                "SELECT dm_topic, msg_offset, client_msg_id, recipient_sender_id, attempts, enqueued_ms \
                 FROM awaiting_ack WHERE client_msg_id = ?1",
                params![client_msg_id],
                map_row,
            )
            .optional()?;
        Ok(row)
    }

    /// List all outstanding rows, oldest first — the recovery-sweep view
    /// (resume awaits interrupted by a client restart).
    pub fn list(&self) -> TrackerResult<Vec<AwaitingRow>> {
        let conn = self.conn.lock().expect("tracker mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT dm_topic, msg_offset, client_msg_id, recipient_sender_id, attempts, enqueued_ms \
             FROM awaiting_ack ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], map_row)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<AwaitingRow> {
    Ok(AwaitingRow {
        dm_topic: r.get::<_, String>(0)?,
        msg_offset: r.get::<_, i64>(1)?.max(0) as u64,
        client_msg_id: r.get::<_, String>(2)?,
        recipient_sender_id: r.get::<_, String>(3)?,
        attempts: r.get::<_, i64>(4)?.max(0) as u64,
        enqueued_ms: r.get::<_, i64>(5)?,
    })
}

fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::rc::Rc;

    // ───────────────────────── tracker unit tests ─────────────────────────

    #[test]
    fn record_get_confirm_roundtrip() {
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        t.record("dm:a:b", 7, "cmid1", "recip", 100).unwrap();
        assert_eq!(t.size().unwrap(), 1);
        let row = t.get("cmid1").unwrap().unwrap();
        assert_eq!(row.msg_offset, 7);
        assert_eq!(row.recipient_sender_id, "recip");
        assert_eq!(row.attempts, 1);
        t.confirm("cmid1").unwrap();
        assert_eq!(t.size().unwrap(), 0);
        assert!(t.get("cmid1").unwrap().is_none());
    }

    #[test]
    fn record_is_idempotent_on_client_msg_id() {
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        t.record("dm:a:b", 7, "cmid1", "recip", 100).unwrap();
        // Re-record same id (a re-post) refreshes, does NOT duplicate.
        t.record("dm:a:b", 7, "cmid1", "recip", 200).unwrap();
        assert_eq!(t.size().unwrap(), 1, "no duplicate row for the same client_msg_id");
    }

    #[test]
    fn bump_attempts_increments() {
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        t.record("dm:a:b", 1, "cmid1", "recip", 0).unwrap();
        t.bump_attempts("cmid1").unwrap();
        t.bump_attempts("cmid1").unwrap();
        assert_eq!(t.get("cmid1").unwrap().unwrap().attempts, 3);
    }

    #[test]
    fn confirm_missing_is_noop() {
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        t.confirm("never-existed").unwrap(); // must not error
        assert_eq!(t.size().unwrap(), 0);
    }

    #[test]
    fn survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ack.sqlite");
        {
            let t = AwaitingAckTracker::open(&path).unwrap();
            t.record("dm:persist", 42, "cmidP", "recip", 5).unwrap();
        }
        // Durability: an await interrupted by a client restart is recoverable.
        let t2 = AwaitingAckTracker::open(&path).unwrap();
        assert_eq!(t2.size().unwrap(), 1);
        let rows = t2.list().unwrap();
        assert_eq!(rows[0].client_msg_id, "cmidP");
        assert_eq!(rows[0].msg_offset, 42);
    }

    // ───────────────────────── retry-loop tests ───────────────────────────

    /// A fake hub honouring T-2049 dedupe: the first post for a given
    /// `client_msg_id` assigns a fresh offset and counts as ONE append;
    /// every re-post of the same id returns the cached offset and appends
    /// nothing. This is the exact contract the real hub enforces
    /// (`termlink-hub/src/channel.rs` dedupe path, proven by
    /// `dedupe_with_client_msg_id_duplicate_returns_cached_offset`).
    #[derive(Default)]
    struct DedupeHub {
        by_id: HashMap<String, u64>,
        next_offset: u64,
        append_count: u64,
    }
    impl DedupeHub {
        fn post(&mut self, client_msg_id: &str) -> u64 {
            if let Some(off) = self.by_id.get(client_msg_id) {
                return *off; // duplicate — no append (T-2049)
            }
            let off = self.next_offset;
            self.next_offset += 1;
            self.by_id.insert(client_msg_id.to_string(), off);
            self.append_count += 1;
            off
        }
    }

    /// Fake clock: `now_ms` reads a cell; `sleep_fn` advances it. Lets the
    /// retry loop run instantly and deterministically.
    fn fake_clock() -> (Rc<Cell<i64>>, impl FnMut() -> i64, impl FnMut(u64) -> std::future::Ready<()>) {
        let clock = Rc::new(Cell::new(0i64));
        let now = {
            let c = clock.clone();
            move || c.get()
        };
        let sleep = {
            let c = clock.clone();
            move |ms: u64| {
                c.set(c.get() + ms as i64);
                std::future::ready(())
            }
        };
        (clock, now, sleep)
    }

    #[tokio::test]
    async fn acks_on_first_attempt_no_retry() {
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        let hub = Rc::new(RefCell::new(DedupeHub::default()));
        let (_clock, now, sleep) = fake_clock();

        let hub_p = hub.clone();
        let post_fn = move |cmid: String| {
            let off = hub_p.borrow_mut().post(&cmid);
            std::future::ready(Ok(off))
        };
        // Recipient has already acked well past the offset.
        let receipts_fn = || {
            std::future::ready(Ok(vec![ReceiptRow { sender_id: "recip".into(), up_to: 99 }]))
        };

        let out = await_ack_with_retry(
            &t, "dm:a:recip", "recip", "cmid-ok",
            &RetryPolicy::default(), post_fn, receipts_fn, now, sleep,
        )
        .await
        .unwrap();

        assert_eq!(out, AckOutcome::Acked { offset: 0, attempts: 1 });
        assert_eq!(hub.borrow().append_count, 1, "exactly one append");
        assert_eq!(t.size().unwrap(), 0, "row confirmed (deleted) on ack");
    }

    #[tokio::test]
    async fn retry_after_dead_recipient_is_exactly_once() {
        // THE headline AC: the recipient withholds its ack until after one
        // retry. The post must be re-sent (same client_msg_id) and the hub
        // dedupe must absorb it — exactly ONE envelope appended across both
        // attempts, and the helper returns success once the frontier catches up.
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        let hub = Rc::new(RefCell::new(DedupeHub::default()));
        let (_clock, now, sleep) = fake_clock();

        let hub_p = hub.clone();
        let post_fn = move |cmid: String| {
            let off = hub_p.borrow_mut().post(&cmid);
            std::future::ready(Ok(off))
        };

        // Receipts: deaf (up_to=0, below offset) until the Nth read, then
        // the ack lands. The first attempt's polls all miss → deadline →
        // retry; the ack appears during the second attempt.
        let reads = Rc::new(Cell::new(0u32));
        let reads_p = reads.clone();
        let receipts_fn = move || {
            let n = reads_p.get();
            reads_p.set(n + 1);
            // A deaf recipient has NO receipt row (the real hub returns one
            // row per sender that has acked). ~6 polls per 30s attempt at 5s
            // cadence; the ack lands on the 8th read (well into attempt 2).
            let rows = if n >= 8 {
                vec![ReceiptRow { sender_id: "recip".into(), up_to: 5 }]
            } else {
                Vec::new()
            };
            std::future::ready(Ok(rows))
        };

        let out = await_ack_with_retry(
            &t, "dm:a:recip", "recip", "cmid-retry",
            &RetryPolicy::default(), post_fn, receipts_fn, now, sleep,
        )
        .await
        .unwrap();

        match out {
            AckOutcome::Acked { offset, attempts } => {
                assert_eq!(offset, 0);
                assert_eq!(attempts, 2, "needed exactly one retry");
            }
            other => panic!("expected Acked after retry, got {other:?}"),
        }
        // The decisive assertion: the retry reused the client_msg_id, so the
        // dedupe hub appended the envelope EXACTLY ONCE despite two posts.
        assert_eq!(hub.borrow().append_count, 1, "exactly-once across the retry");
        assert_eq!(t.size().unwrap(), 0, "row confirmed on eventual ack");
    }

    #[tokio::test]
    async fn exhausts_after_max_attempts_and_retains_row() {
        // Recipient never acks. The helper posts max_attempts times (all
        // deduped to one append), gives up, and LEAVES the durable row for
        // a recovery sweep — it does not silently drop the obligation.
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        let hub = Rc::new(RefCell::new(DedupeHub::default()));
        let (_clock, now, sleep) = fake_clock();

        let hub_p = hub.clone();
        let post_fn = move |cmid: String| {
            let off = hub_p.borrow_mut().post(&cmid);
            std::future::ready(Ok(off))
        };
        // Deaf forever: never any receipt row for the recipient.
        let receipts_fn = || std::future::ready(Ok(Vec::<ReceiptRow>::new()));

        let policy = RetryPolicy { poll_interval_ms: 5_000, deadline_ms: 30_000, max_attempts: 3 };
        let out = await_ack_with_retry(
            &t, "dm:a:recip", "recip", "cmid-dead",
            &policy, post_fn, receipts_fn, now, sleep,
        )
        .await
        .unwrap();

        assert_eq!(out, AckOutcome::Exhausted { offset: 0, attempts: 3 });
        assert_eq!(hub.borrow().append_count, 1, "all retries deduped to one append");
        let row = t.get("cmid-dead").unwrap().expect("durable row retained on exhaustion");
        assert_eq!(row.attempts, 3, "attempt count reflects every re-post");
    }

    #[tokio::test]
    async fn post_failure_propagates() {
        let t = AwaitingAckTracker::open_in_memory().unwrap();
        let (_clock, now, sleep) = fake_clock();
        let post_fn = |_cmid: String| std::future::ready(Err("hub unreachable".to_string()));
        let receipts_fn = || std::future::ready(Ok(vec![]));
        let err = await_ack_with_retry(
            &t, "dm:a:recip", "recip", "cmid-x",
            &RetryPolicy::default(), post_fn, receipts_fn, now, sleep,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AckRetryError::Post(_)));
        assert_eq!(t.size().unwrap(), 0, "no row recorded when the first post fails");
    }

    #[test]
    fn from_operator_clamps_and_defaults() {
        let p = RetryPolicy::from_operator(0, 0);
        assert_eq!(p.deadline_ms, DEFAULT_DEADLINE_MS, "zero timeout → default deadline");
        assert_eq!(p.max_attempts, 1, "attempts clamp to >= 1");
        let p2 = RetryPolicy::from_operator(45, 5);
        assert_eq!(p2.deadline_ms, 45_000);
        assert_eq!(p2.max_attempts, 5);
        assert_eq!(p2.poll_interval_ms, DEFAULT_POLL_INTERVAL_MS);
    }

    #[test]
    fn recipient_acked_filters_by_sender() {
        let receipts = vec![
            ReceiptRow { sender_id: "other".into(), up_to: 100 },
            ReceiptRow { sender_id: "recip".into(), up_to: 3 },
        ];
        assert!(recipient_acked(&receipts, "recip", 3));
        assert!(!recipient_acked(&receipts, "recip", 4), "below frontier");
        assert!(!recipient_acked(&receipts, "absent", 1));
    }
}
