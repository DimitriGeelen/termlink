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
pub use claim::{ClaimInfo, ClaimsSummary, IdleAgent, ReleaseInfo, TransferInfo};
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

    /// T-2244 (R2a): change the retention policy of an EXISTING topic.
    /// Complements `create_topic`, which refuses a policy change on an
    /// idempotent re-create (`TopicPolicyMismatch`). Returns `Ok(true)`
    /// when the topic existed and was re-tuned, `Ok(false)` when no such
    /// topic exists (a no-op — the caller decides whether absence is an
    /// error; e.g. the CLI reports "unknown topic" rather than creating).
    ///
    /// Storage-only: this updates the policy but does NOT trim existing
    /// records. Call `sweep(topic, now)` afterwards to enforce the new
    /// policy against the backlog (e.g. drop `agent-presence` history older
    /// than the newly-set `Days(2)` window).
    pub fn set_topic_retention(&self, topic: &str, retention: Retention) -> Result<bool> {
        self.meta.set_topic_retention(topic, retention)
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

    /// T-2421: delete `topic` ENTIRELY — registry entry, records index,
    /// cursors, offset counter, claims, cached appender/notifier, and the
    /// on-disk log file. The topic disappears from `list_topics`; a
    /// subsequent `create_topic`/`post` under the same name starts a FRESH
    /// topic at offset 0. Returns `Ok(Some(records_removed))` when the
    /// topic existed, `Ok(None)` otherwise. Contrast `trim_topic`, which
    /// empties a topic but leaves it registered forever — the gap that let
    /// production hubs accumulate thousands of dead test topics (T-2419 §5.4).
    pub fn delete_topic(&self, topic: &str) -> Result<Option<u64>> {
        let deleted = self.meta.delete_topic(topic)?;
        if deleted.is_some() {
            self.appenders
                .lock()
                .expect("appenders mutex poisoned")
                .remove(topic);
            self.notifiers
                .lock()
                .expect("notifiers mutex poisoned")
                .remove(topic);
            // Missing log file is fine — topic may never have been posted to.
            let _ = std::fs::remove_file(log::topic_log_path(&self.root, topic));
        }
        Ok(deleted)
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

    /// Read the single envelope at `offset` on `topic`. Returns `None` if
    /// the offset was swept by retention or never existed. Returns
    /// `BusError::UnknownTopic` if the topic was never registered.
    ///
    /// T-2109: substrate primitive #2 (DISPATCH) consumes this for the
    /// cv_index fast path — given a `(cv_key, latest_offset)` pair from
    /// substrate primitive #9 (BROADCAST-WITH-REPLAY), the caller reads
    /// the envelope to extract metadata.role / metadata.capabilities /
    /// ts_unix_ms without walking the whole topic.
    pub fn envelope_at(&self, topic: &str, offset: Offset) -> Result<Option<Envelope>> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        let mut iter = self.subscribe(topic, offset)?;
        match iter.next() {
            None => Ok(None),
            Some(Err(e)) => Err(e),
            Some(Ok((found_offset, env))) if found_offset == offset => Ok(Some(env)),
            // Requested offset was swept; iterator landed on a later record.
            Some(Ok(_)) => Ok(None),
        }
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

    /// T-2044 (arc-parallel-substrate Slice 11): operator-Tier-0 force release
    /// of a claim. Bypasses the `claimed_by == claimer` ownership check
    /// `release_claim` enforces — for situations where an operator must clear
    /// a stuck claim faster than the natural TTL expiry path. Semantics match
    /// `release(ack=false)`: cursor unchanged, slot freed for the next worker.
    /// The optional `reason` is returned in `ReleaseInfo.forced_reason` for
    /// upstream audit trails.
    ///
    /// Errors:
    /// - `BusError::ClaimNotFound` — unknown / already-released `claim_id`.
    ///
    /// Does NOT return `ClaimNotOwned` — bypassing that check is the whole
    /// point of this verb.
    pub fn force_release_claim(
        &self,
        claim_id: &str,
        reason: Option<&str>,
    ) -> Result<ReleaseInfo> {
        self.meta.force_release_claim(claim_id, reason)
    }

    /// T-2046 (T-2021 GO, arc-parallel-substrate primitive #3): atomic
    /// ownership transfer of an existing claim from `by` to `to_owner`.
    /// Cooperative + owner-checked — distinct from `force_release_claim`
    /// (operator-Tier-0 ownership bypass).
    ///
    /// Lease timestamps are preserved; only `claimed_by` advances. Optional
    /// `reason` is surfaced verbatim in `TransferInfo.reason` for upstream
    /// audit, not persisted in the claims table.
    ///
    /// Errors:
    /// - `BusError::ClaimNotFound` — unknown / already-released `claim_id`.
    /// - `BusError::ClaimExpired` — row exists but `claimed_until <= now`
    ///   (lazily evicted in the same call so the slot becomes claimable).
    /// - `BusError::ClaimNotOwned` — `by` is not the current `claimed_by`.
    pub fn transfer_claim(
        &self,
        claim_id: &str,
        to_owner: &str,
        by: &str,
        reason: Option<&str>,
    ) -> Result<TransferInfo> {
        let now_ms = now_unix_ms();
        self.meta.transfer_claim(claim_id, to_owner, by, reason, now_ms)
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

    /// T-2045 (T-2020 GO): server-side derivation of the idle-agent roster.
    ///
    /// Walks the `agent-presence` topic from offset 0, dedups by
    /// `metadata.agent_id` keeping the latest heartbeat per agent, filters
    /// to LIVE (`ts_unix_ms > now - live_window_ms`), applies the optional
    /// `role` and `capabilities` predicates, then excludes every agent_id
    /// currently holding any active claim. Result is sorted by
    /// `last_heartbeat_ms` descending (freshest first).
    ///
    /// `capabilities` predicate is SUBSET match: an agent's parsed
    /// `metadata.capabilities` (comma-separated) must contain EVERY
    /// requested capability. Missing/empty metadata.capabilities = empty
    /// set (backward-compat with workers that don't emit the field).
    ///
    /// Returns an empty vec when the `agent-presence` topic doesn't exist
    /// or contains no LIVE envelopes; never errors on a missing topic
    /// (unlike `subscribe` / `claims_summary`) — a fresh hub with no
    /// heartbeats yet is a normal state, not an error.
    pub fn find_idle_agents(
        &self,
        role_filter: Option<&str>,
        capability_filter: &[String],
        live_window_ms: i64,
        limit: Option<u32>,
    ) -> Result<Vec<IdleAgent>> {
        const PRESENCE_TOPIC: &str = "agent-presence";
        let now_ms = now_unix_ms();
        let cutoff_ms = now_ms - live_window_ms.max(0);

        // Walk presence topic (if it exists). Missing topic = empty fleet.
        if !self.meta.topic_exists(PRESENCE_TOPIC)? {
            return Ok(Vec::new());
        }
        let iter = self.subscribe(PRESENCE_TOPIC, 0)?;

        // Dedup by agent_id, keep latest by ts_unix_ms.
        let mut latest: HashMap<String, IdleAgent> = HashMap::new();
        for item in iter {
            let (_offset, env) = item?;
            let Some(agent_id) = env.metadata.get("agent_id").cloned() else {
                continue;
            };
            let role = env.metadata.get("role").cloned();
            let capabilities: Vec<String> = env
                .metadata
                .get("capabilities")
                .map(|s| {
                    s.split(',')
                        .map(|c| c.trim().to_string())
                        .filter(|c| !c.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            let entry = IdleAgent {
                agent_id: agent_id.clone(),
                last_heartbeat_ms: env.ts_unix_ms,
                role,
                capabilities,
            };
            match latest.get(&agent_id) {
                Some(prev) if prev.last_heartbeat_ms >= entry.last_heartbeat_ms => {}
                _ => {
                    latest.insert(agent_id, entry);
                }
            }
        }

        // LIVE + role + capabilities filter.
        let mut filtered: Vec<IdleAgent> = latest
            .into_values()
            .filter(|a| a.last_heartbeat_ms > cutoff_ms)
            .filter(|a| match role_filter {
                None => true,
                Some(r) => a.role.as_deref() == Some(r),
            })
            .filter(|a| {
                capability_filter
                    .iter()
                    .all(|req| a.capabilities.iter().any(|c| c == req))
            })
            .collect();

        // Anti-join against active claimers (across ALL topics).
        let claimers = self.meta.distinct_active_claimers(now_ms)?;
        let claimers_set: std::collections::HashSet<&str> =
            claimers.iter().map(String::as_str).collect();
        filtered.retain(|a| !claimers_set.contains(a.agent_id.as_str()));

        // Sort freshest first, then apply limit.
        filtered.sort_by(|a, b| b.last_heartbeat_ms.cmp(&a.last_heartbeat_ms));
        if let Some(n) = limit {
            filtered.truncate(n as usize);
        }
        Ok(filtered)
    }

    /// Hint-driven companion to [`find_idle_agents`]: instead of walking the
    /// `agent-presence` topic from offset 0, accept a pre-computed
    /// `[(agent_id, presence_offset)]` list and resolve each agent by a
    /// single-offset read via [`envelope_at`]. Same filter / sort / limit /
    /// claimer-anti-join semantics as the walk path.
    ///
    /// T-2109: substrate primitive #2 (DISPATCH) consumes this with the
    /// hub's cv_index (substrate primitive #9, T-2103) as the hint source.
    /// When the cv_index records `(agent-presence, agent_id) → latest_offset`
    /// for every advertising agent (default since T-2107 wired
    /// `cv_key=$agent_id` into `listener-heartbeat.sh`), discovery cost
    /// drops from O(N_heartbeats) to O(N_agents).
    ///
    /// **Trade-off.** Entries whose envelope has been swept by retention
    /// are silently skipped (cv_index may carry a stale offset). Producers
    /// that opted out of cv_key tagging are invisible to this path —
    /// callers should fall back to [`find_idle_agents`] when their hint
    /// source is empty (cold start, no producers wired).
    ///
    /// Returns an empty vec when the `agent-presence` topic doesn't exist;
    /// never errors on a missing topic.
    pub fn find_idle_agents_from_hint(
        &self,
        role_filter: Option<&str>,
        capability_filter: &[String],
        live_window_ms: i64,
        limit: Option<u32>,
        hint: &[(String, Offset)],
    ) -> Result<Vec<IdleAgent>> {
        const PRESENCE_TOPIC: &str = "agent-presence";
        let now_ms = now_unix_ms();
        let cutoff_ms = now_ms - live_window_ms.max(0);

        if !self.meta.topic_exists(PRESENCE_TOPIC)? {
            return Ok(Vec::new());
        }

        // Resolve each hint entry via single-offset read. Swept offsets are
        // skipped, never panic. Dedup by agent_id keeping the freshest
        // ts_unix_ms in case the hint carries duplicates.
        let mut latest: HashMap<String, IdleAgent> = HashMap::with_capacity(hint.len());
        for (agent_id, offset) in hint {
            let env = match self.envelope_at(PRESENCE_TOPIC, *offset)? {
                Some(e) => e,
                None => continue,
            };
            let role = env.metadata.get("role").cloned();
            let capabilities: Vec<String> = env
                .metadata
                .get("capabilities")
                .map(|s| {
                    s.split(',')
                        .map(|c| c.trim().to_string())
                        .filter(|c| !c.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            let entry = IdleAgent {
                agent_id: agent_id.clone(),
                last_heartbeat_ms: env.ts_unix_ms,
                role,
                capabilities,
            };
            match latest.get(agent_id) {
                Some(prev) if prev.last_heartbeat_ms >= entry.last_heartbeat_ms => {}
                _ => {
                    latest.insert(agent_id.clone(), entry);
                }
            }
        }

        // Same filter chain as find_idle_agents.
        let mut filtered: Vec<IdleAgent> = latest
            .into_values()
            .filter(|a| a.last_heartbeat_ms > cutoff_ms)
            .filter(|a| match role_filter {
                None => true,
                Some(r) => a.role.as_deref() == Some(r),
            })
            .filter(|a| {
                capability_filter
                    .iter()
                    .all(|req| a.capabilities.iter().any(|c| c == req))
            })
            .collect();

        // Anti-join against active claimers (across ALL topics).
        let claimers = self.meta.distinct_active_claimers(now_ms)?;
        let claimers_set: std::collections::HashSet<&str> =
            claimers.iter().map(String::as_str).collect();
        filtered.retain(|a| !claimers_set.contains(a.agent_id.as_str()));

        filtered.sort_by(|a, b| b.last_heartbeat_ms.cmp(&a.last_heartbeat_ms));
        if let Some(n) = limit {
            filtered.truncate(n as usize);
        }
        Ok(filtered)
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
            // Per-key compaction cannot be expressed as a (keep_after, keep_last)
            // tuple — it groups by cv_key — so it has its own routine.
            Retention::LatestPerCvKey => return self.compact_per_cv_key(topic),
            Retention::Days(d) => {
                let cutoff = now_unix_ms - i64::from(d) * 86_400_000;
                (Some(cutoff), None)
            }
            Retention::Messages(n) => (None, Some(n)),
            Retention::Latest => (None, Some(1)),
        };
        self.meta.sweep_records(topic, keep_after, keep_last)
    }

    /// Compaction sweep for [`Retention::LatestPerCvKey`]: keep only the
    /// highest-offset record per distinct `metadata.cv_key`, deleting the
    /// older records that share a key. Records carrying no (or empty) cv_key
    /// are **retained** — un-keyed data is never silently dropped. Returns the
    /// number of records pruned.
    ///
    /// cv_key is recovered by reading each record's envelope from the log
    /// (`metadata` lives in the envelope blob); no `records`-table schema
    /// column is required, and the sweep works on records written before this
    /// mode existed. The sweep is on-demand only (the library runs no
    /// background thread), so the O(N) envelope reads are paid exactly when an
    /// operator chooses to shrink the topic. After one compaction the live set
    /// is one record per key, so subsequent sweeps only re-scan that small set
    /// plus whatever has been posted since.
    ///
    /// R2b / T-2245 — the only retention mode that closes the T-1991
    /// agent-*count* scaling problem for `agent-presence`-style topics.
    pub fn compact_per_cv_key(&self, topic: &str) -> Result<u64> {
        let locs = self.meta.records_from(topic, 0)?;
        // cv_key -> highest offset seen for that key.
        let mut latest: HashMap<String, Offset> = HashMap::new();
        // Every (offset, cv_key) for records that carry a cv_key, so we can
        // compute the delete set = keyed records that are not their key's latest.
        let mut keyed: Vec<(Offset, String)> = Vec::with_capacity(locs.len());
        for loc in &locs {
            let env = match self.envelope_at(topic, loc.offset)? {
                Some(e) => e,
                // Index row with no recoverable blob (e.g. truncated log); leave
                // it alone rather than guess — compaction must not destroy data
                // it cannot read.
                None => continue,
            };
            match env.metadata.get("cv_key") {
                Some(cv) if !cv.is_empty() => {
                    keyed.push((loc.offset, cv.clone()));
                    latest
                        .entry(cv.clone())
                        .and_modify(|o| {
                            if loc.offset > *o {
                                *o = loc.offset;
                            }
                        })
                        .or_insert(loc.offset);
                }
                // No cv_key (or empty): retained, not compacted.
                _ => {}
            }
        }
        let to_delete: Vec<Offset> = keyed
            .iter()
            .filter(|(off, cv)| latest.get(cv).is_some_and(|latest_off| off != latest_off))
            .map(|(off, _)| *off)
            .collect();
        self.meta.delete_records_at(topic, &to_delete)
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

    #[tokio::test]
    async fn delete_topic_removes_everything() {
        let (dir, bus) = tmp_bus();
        bus.create_topic("doomed", Retention::Forever).unwrap();
        bus.post("doomed", &env("doomed", b"a")).await.unwrap();
        bus.post("doomed", &env("doomed", b"b")).await.unwrap();
        bus.advance_cursor("reader-1", "doomed", 1).unwrap();
        bus.claim_offset("doomed", 0, "worker-A", 60_000).unwrap();
        let log_path = log::topic_log_path(dir.path(), "doomed");
        assert!(log_path.is_file(), "log file should exist pre-delete");

        let removed = bus.delete_topic("doomed").unwrap();
        assert_eq!(removed, Some(2), "two records should be reported removed");
        assert!(
            !bus.list_topics().unwrap().contains(&"doomed".to_string()),
            "deleted topic must vanish from list_topics"
        );
        assert_eq!(bus.topic_record_count("doomed").unwrap(), 0);
        assert!(
            matches!(bus.list_claims("doomed", true), Err(BusError::UnknownTopic(_))),
            "claims on a deleted topic must report UnknownTopic, not stale rows"
        );
        assert_eq!(bus.get_cursor("reader-1", "doomed").unwrap(), None);
        assert!(!log_path.exists(), "on-disk log file must be removed");
    }

    #[tokio::test]
    async fn delete_topic_nonexistent_returns_none() {
        let (_dir, bus) = tmp_bus();
        assert_eq!(bus.delete_topic("never-existed").unwrap(), None);
    }

    #[tokio::test]
    async fn delete_topic_then_recreate_starts_fresh_at_offset_zero() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("phoenix", Retention::Forever).unwrap();
        bus.post("phoenix", &env("phoenix", b"old-1")).await.unwrap();
        bus.post("phoenix", &env("phoenix", b"old-2")).await.unwrap();
        assert_eq!(bus.delete_topic("phoenix").unwrap(), Some(2));

        bus.create_topic("phoenix", Retention::Messages(10)).unwrap();
        let off = bus.post("phoenix", &env("phoenix", b"new-1")).await.unwrap();
        assert_eq!(off, 0, "recreated topic must start at offset 0");
        let got: Vec<_> = bus
            .subscribe("phoenix", 0)
            .unwrap()
            .map(|r| r.unwrap().1.payload)
            .collect();
        assert_eq!(got, vec![b"new-1".to_vec()], "no ghost records from the old life");
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
    async fn sweep_retention_latest_keeps_one() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Latest).unwrap();
        for i in 0..5 {
            bus.post("t", &env("t", format!("m{i}").as_bytes())).await.unwrap();
        }
        let pruned = bus.sweep("t", 0).unwrap();
        assert_eq!(pruned, 4);
        let offsets: Vec<u64> = bus
            .subscribe("t", 0)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        assert_eq!(offsets, vec![4]);
        assert_eq!(bus.topic_retention("t").unwrap(), Some(Retention::Latest));
    }

    fn env_cv(topic: &str, payload: &[u8], cv_key: &str) -> Envelope {
        let mut e = env(topic, payload);
        e.metadata.insert("cv_key".to_string(), cv_key.to_string());
        e
    }

    #[tokio::test]
    async fn compact_per_cv_key_keeps_exactly_one_record_per_key() {
        // R2b / T-2245 — PROPERTY assertion (PL-213): post M heartbeats across
        // K distinct cv_keys (M >> K). After compaction EXACTLY K keyed records
        // remain — one per key, each the latest offset for that key — proving
        // record count converges to agent *count*, not heartbeat count (T-1991).
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::LatestPerCvKey).unwrap();
        // Round-trip of the new policy through SQLite (kind/value/from_parts).
        assert_eq!(
            bus.topic_retention("agent-presence").unwrap(),
            Some(Retention::LatestPerCvKey)
        );

        let keys = ["alpha", "beta", "gamma"]; // K = 3
        for round in 0..4u32 {
            // M = 12 keyed beats
            for k in &keys {
                bus.post("agent-presence", &env_cv("agent-presence", format!("{k}-{round}").as_bytes(), k))
                    .await
                    .unwrap();
            }
        }
        // One un-keyed record (offset 12) — must be RETAINED, never silently dropped.
        bus.post("agent-presence", &env("agent-presence", b"no-key")).await.unwrap();
        assert_eq!(bus.topic_record_count("agent-presence").unwrap(), 13);

        let pruned = bus.sweep("agent-presence", 0).unwrap();
        assert_eq!(pruned, 9, "12 keyed beats - 3 keys = 9 stale beats pruned");

        // Survivors: latest offset per key {alpha:9, beta:10, gamma:11} + un-keyed 12.
        let survivors: Vec<u64> = bus
            .subscribe("agent-presence", 0)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        assert_eq!(survivors, vec![9, 10, 11, 12]);
        // PROPERTY: keyed record count == distinct key count, exactly.
        let keyed_survivors = survivors.len() - 1; // minus the un-keyed record
        assert_eq!(keyed_survivors, keys.len());

        // Each surviving keyed record is its key's freshest payload.
        for off in [9u64, 10, 11] {
            let env = bus.envelope_at("agent-presence", off).unwrap().unwrap();
            let cv = env.metadata.get("cv_key").unwrap();
            assert_eq!(env.payload, format!("{cv}-3").into_bytes(), "latest round survives");
        }
    }

    #[tokio::test]
    async fn compact_per_cv_key_is_idempotent_and_handles_empty() {
        // A second sweep on an already-compact topic prunes nothing; an empty
        // topic and an all-un-keyed topic both prune nothing (no silent drops).
        let (_dir, bus) = tmp_bus();
        bus.create_topic("p", Retention::LatestPerCvKey).unwrap();
        // Empty topic.
        assert_eq!(bus.sweep("p", 0).unwrap(), 0);
        // All un-keyed: retained.
        for i in 0..3 {
            bus.post("p", &env("p", format!("u{i}").as_bytes())).await.unwrap();
        }
        assert_eq!(bus.sweep("p", 0).unwrap(), 0, "un-keyed records are never dropped");
        assert_eq!(bus.topic_record_count("p").unwrap(), 3);
        // Keyed: one key, three beats -> compacts to 1, second sweep is a no-op.
        bus.create_topic("q", Retention::LatestPerCvKey).unwrap();
        for i in 0..3 {
            bus.post("q", &env_cv("q", format!("a{i}").as_bytes(), "a")).await.unwrap();
        }
        assert_eq!(bus.sweep("q", 0).unwrap(), 2);
        assert_eq!(bus.sweep("q", 0).unwrap(), 0, "idempotent: already compact");
        assert_eq!(bus.topic_record_count("q").unwrap(), 1);
    }

    #[tokio::test]
    async fn set_topic_retention_changes_policy_and_unknown_is_noop() {
        // T-2244 (R2a): the change-retention-on-existing-topic primitive.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        assert_eq!(
            bus.topic_retention("agent-presence").unwrap(),
            Some(Retention::Forever)
        );
        // Re-tune the existing topic — create_topic would refuse this.
        let updated = bus
            .set_topic_retention("agent-presence", Retention::Days(2))
            .unwrap();
        assert!(updated, "existing topic was updated");
        assert_eq!(
            bus.topic_retention("agent-presence").unwrap(),
            Some(Retention::Days(2)),
            "new policy is persisted"
        );
        // Unknown topic is a no-op (false), not an error or a stealth create.
        let unknown = bus.set_topic_retention("no-such-topic", Retention::Latest).unwrap();
        assert!(!unknown, "unknown topic returns false");
        assert_eq!(bus.topic_retention("no-such-topic").unwrap(), None);
    }

    #[tokio::test]
    async fn set_topic_retention_then_sweep_enforces_new_policy() {
        // T-2244 (R2a): proves the changed policy actually takes effect —
        // not merely stored. A forever topic with old + fresh records, moved
        // to Days(1), must trim the old record on the next sweep.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let day_ms: i64 = 86_400_000;
        let now: i64 = 10 * day_ms;
        bus.post(
            "agent-presence",
            &Envelope { ts_unix_ms: now - 3 * day_ms, ..env("agent-presence", b"stale") },
        )
        .await
        .unwrap();
        bus.post(
            "agent-presence",
            &Envelope { ts_unix_ms: now, ..env("agent-presence", b"live") },
        )
        .await
        .unwrap();
        // Under the original `forever`, a sweep is a no-op.
        assert_eq!(bus.sweep("agent-presence", now).unwrap(), 0);
        // Re-tune to Days(1), then sweep enforces it.
        assert!(bus.set_topic_retention("agent-presence", Retention::Days(1)).unwrap());
        let pruned = bus.sweep("agent-presence", now).unwrap();
        assert_eq!(pruned, 1, "the 3-day-old record is trimmed under the new policy");
        let payloads: Vec<Vec<u8>> = bus
            .subscribe("agent-presence", 0)
            .unwrap()
            .map(|r| r.unwrap().1.payload)
            .collect();
        assert_eq!(payloads, vec![b"live".to_vec()], "only the fresh record survives");
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

    // ── T-2044 (arc-parallel-substrate Slice 11): force-release semantics ──

    #[tokio::test]
    async fn force_release_succeeds_where_release_fails_with_not_owned() {
        // The whole point of the force path: an operator can break a claim
        // they don't own, where ordinary release_claim refuses.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "worker-A", 30_000).unwrap();
        // Ordinary release by a non-owner fails.
        let err = bus
            .release_claim(&c.claim_id, "worker-B", false)
            .unwrap_err();
        assert!(
            matches!(err, BusError::ClaimNotOwned { ref claim_id, .. } if claim_id == &c.claim_id),
            "expected ClaimNotOwned, got {err:?}"
        );
        // Force-release succeeds and surfaces the original claimer.
        let r = bus
            .force_release_claim(&c.claim_id, Some("worker-A wedged"))
            .unwrap();
        assert_eq!(r.offset, 0);
        assert!(!r.ack, "force-release uses ack=false semantics");
        assert_eq!(r.forced_from.as_deref(), Some("worker-A"));
        assert_eq!(r.forced_reason.as_deref(), Some("worker-A wedged"));
        // Cursor for worker-A was NOT advanced.
        assert_eq!(bus.get_cursor("worker-A", "work").unwrap(), None);
        // Slot is free — worker-B can now claim it.
        let c2 = bus.claim_offset("work", 0, "worker-B", 30_000).unwrap();
        assert_eq!(c2.claimer, "worker-B");
        assert_ne!(c2.claim_id, c.claim_id);
    }

    #[tokio::test]
    async fn force_release_on_unknown_claim_returns_not_found() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        let err = bus
            .force_release_claim("nonexistent-claim-id", None)
            .unwrap_err();
        assert!(
            matches!(err, BusError::ClaimNotFound(ref cid) if cid == "nonexistent-claim-id"),
            "expected ClaimNotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn force_release_without_reason_leaves_reason_none() {
        // The reason field is operator-supplied audit metadata, optional.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "worker-A", 30_000).unwrap();
        let r = bus.force_release_claim(&c.claim_id, None).unwrap();
        assert_eq!(r.forced_from.as_deref(), Some("worker-A"));
        assert_eq!(r.forced_reason, None);
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

    // ─────────────────────────────────────────────────────────────────────
    // T-2045 (T-2020 GO) — agent.find_idle derivation tests.
    // ─────────────────────────────────────────────────────────────────────

    fn heartbeat_env(
        agent_id: &str,
        ts_unix_ms: i64,
        role: Option<&str>,
        capabilities: Option<&str>,
    ) -> Envelope {
        let mut metadata: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        metadata.insert("agent_id".to_string(), agent_id.to_string());
        if let Some(r) = role {
            metadata.insert("role".to_string(), r.to_string());
        }
        if let Some(c) = capabilities {
            metadata.insert("capabilities".to_string(), c.to_string());
        }
        Envelope {
            topic: "agent-presence".to_string(),
            sender_id: agent_id.to_string(),
            msg_type: "heartbeat".to_string(),
            payload: b"claude-code".to_vec(),
            artifact_ref: None,
            ts_unix_ms,
            metadata,
        }
    }

    #[tokio::test]
    async fn find_idle_missing_presence_topic_returns_empty() {
        let (_dir, bus) = tmp_bus();
        // No agent-presence topic created — fresh hub, no heartbeats ever.
        let idle = bus.find_idle_agents(None, &[], 60_000, None).unwrap();
        assert!(idle.is_empty(), "fresh hub must return empty, not error");
    }

    #[tokio::test]
    async fn find_idle_presence_only_no_claims_returns_all_live() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 1_000, Some("claude-code"), None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("agent-b", now - 5_000, Some("claude-code"), None))
            .await
            .unwrap();
        let idle = bus.find_idle_agents(None, &[], 60_000, None).unwrap();
        assert_eq!(idle.len(), 2);
        // Sorted freshest-first.
        assert_eq!(idle[0].agent_id, "agent-a");
        assert_eq!(idle[1].agent_id, "agent-b");
    }

    #[tokio::test]
    async fn find_idle_dedups_by_agent_id_keeping_latest_heartbeat() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        // Three heartbeats for same agent — only the LATEST should appear.
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 10_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 1_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 5_000, None, None))
            .await
            .unwrap();
        let idle = bus.find_idle_agents(None, &[], 60_000, None).unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].last_heartbeat_ms, now - 1_000);
    }

    #[tokio::test]
    async fn find_idle_filters_stale_outside_live_window() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        // Fresh agent.
        bus.post("agent-presence", &heartbeat_env("fresh", now - 1_000, None, None))
            .await
            .unwrap();
        // Stale — outside 60s live window.
        bus.post("agent-presence", &heartbeat_env("stale", now - 120_000, None, None))
            .await
            .unwrap();
        let idle = bus.find_idle_agents(None, &[], 60_000, None).unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "fresh");
    }

    #[tokio::test]
    async fn find_idle_excludes_active_claimers() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        bus.create_topic("work-queue", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("worker-busy", now - 500, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("worker-free", now - 500, None, None))
            .await
            .unwrap();
        // Seed a work-queue offset and claim it as worker-busy.
        bus.post("work-queue", &env("work-queue", b"task-1")).await.unwrap();
        bus.claim_offset("work-queue", 0, "worker-busy", 60_000)
            .unwrap();
        let idle = bus.find_idle_agents(None, &[], 60_000, None).unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "worker-free");
    }

    #[tokio::test]
    async fn find_idle_role_filter_narrows_result() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post(
            "agent-presence",
            &heartbeat_env("claude-1", now - 100, Some("claude-code"), None),
        )
        .await
        .unwrap();
        bus.post(
            "agent-presence",
            &heartbeat_env("worker-1", now - 100, Some("test-worker"), None),
        )
        .await
        .unwrap();
        let idle = bus
            .find_idle_agents(Some("claude-code"), &[], 60_000, None)
            .unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "claude-1");
    }

    #[tokio::test]
    async fn find_idle_capabilities_subset_match() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post(
            "agent-presence",
            &heartbeat_env("a-build", now - 100, None, Some("build,test")),
        )
        .await
        .unwrap();
        bus.post(
            "agent-presence",
            &heartbeat_env("a-test-only", now - 100, None, Some("test")),
        )
        .await
        .unwrap();
        bus.post("agent-presence", &heartbeat_env("a-none", now - 100, None, None))
            .await
            .unwrap();
        // Require both 'build' and 'test' — only a-build qualifies.
        let req: Vec<String> = vec!["build".into(), "test".into()];
        let idle = bus.find_idle_agents(None, &req, 60_000, None).unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "a-build");
        assert_eq!(idle[0].capabilities, vec!["build", "test"]);
    }

    #[tokio::test]
    async fn find_idle_limit_truncates_after_sort() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        // Post three agents with increasing freshness; limit=2 should keep
        // the two freshest.
        bus.post("agent-presence", &heartbeat_env("a-old", now - 30_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("a-mid", now - 15_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("a-new", now - 100, None, None))
            .await
            .unwrap();
        let idle = bus.find_idle_agents(None, &[], 60_000, Some(2)).unwrap();
        assert_eq!(idle.len(), 2);
        assert_eq!(idle[0].agent_id, "a-new");
        assert_eq!(idle[1].agent_id, "a-mid");
    }

    // ── T-2046 (T-2021 GO, arc-parallel-substrate primitive #3) ──
    // channel.transfer_claim semantics: atomic ownership transfer with
    // owner-check + expiry-check, distinct from force_release (Tier-0
    // bypass). Lease timestamps preserved across transfer.

    #[tokio::test]
    async fn transfer_claim_happy_path_advances_owner_preserves_lease() {
        // Orchestrator claims, hands the lease to a worker, the worker
        // releases successfully. The claimed_at and claimed_until must
        // survive the transfer untouched.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "orch", 30_000).unwrap();
        let t = bus
            .transfer_claim(&c.claim_id, "worker-A", "orch", Some("assign T-XXX"))
            .unwrap();
        assert_eq!(t.claim_id, c.claim_id);
        assert_eq!(t.topic, "work");
        assert_eq!(t.offset, 0);
        assert_eq!(t.from_owner, "orch");
        assert_eq!(t.to_owner, "worker-A");
        assert_eq!(t.claimed_at, c.claimed_at, "claimed_at must survive transfer");
        assert_eq!(t.claimed_until, c.claimed_until, "claimed_until must survive transfer");
        assert_eq!(t.reason.as_deref(), Some("assign T-XXX"));
        // Worker-A can release; orch cannot.
        let err = bus.release_claim(&c.claim_id, "orch", true).unwrap_err();
        assert!(
            matches!(err, BusError::ClaimNotOwned { ref claim_id, .. } if claim_id == &c.claim_id),
            "post-transfer release by old owner must fail with ClaimNotOwned, got {err:?}"
        );
        let r = bus.release_claim(&c.claim_id, "worker-A", true).unwrap();
        assert_eq!(r.claim_id, c.claim_id);
        assert!(r.ack);
    }

    #[tokio::test]
    async fn transfer_claim_unknown_claim_returns_not_found() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        let err = bus
            .transfer_claim("nonexistent-claim-id", "worker-A", "orch", None)
            .unwrap_err();
        assert!(
            matches!(err, BusError::ClaimNotFound(ref cid) if cid == "nonexistent-claim-id"),
            "expected ClaimNotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn transfer_claim_expired_claim_returns_expired_and_evicts_row() {
        // Orchestrator claims with a sub-second TTL; before transferring,
        // the lease lapses. transfer_claim must report ClaimExpired AND
        // lazily evict the stale row so a follow-up claim_offset succeeds.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "orch", 50).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        let err = bus
            .transfer_claim(&c.claim_id, "worker-A", "orch", None)
            .unwrap_err();
        assert!(
            matches!(err, BusError::ClaimExpired { ref claim_id } if claim_id == &c.claim_id),
            "expected ClaimExpired, got {err:?}"
        );
        // Slot is reclaimable — a fresh claim by anyone now succeeds.
        let c2 = bus.claim_offset("work", 0, "worker-B", 30_000).unwrap();
        assert_eq!(c2.claimer, "worker-B");
        assert_ne!(c2.claim_id, c.claim_id);
    }

    #[tokio::test]
    async fn transfer_claim_wrong_by_returns_not_owned_without_mutating() {
        // The cooperative-with-audit verb refuses when `by` doesn't match
        // the row's claimed_by. The row must remain owned by the original
        // claimer — a release attempt by the original owner still works.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "orch", 30_000).unwrap();
        let err = bus
            .transfer_claim(&c.claim_id, "worker-A", "imposter", None)
            .unwrap_err();
        assert!(
            matches!(err, BusError::ClaimNotOwned { ref claim_id, ref claimed_by, ref attempted_by }
                if claim_id == &c.claim_id && claimed_by == "orch" && attempted_by == "imposter"),
            "expected ClaimNotOwned{{claim_id=c, claimed_by=orch, attempted_by=imposter}}, got {err:?}"
        );
        // Row is intact — original owner can still release.
        let r = bus.release_claim(&c.claim_id, "orch", false).unwrap();
        assert_eq!(r.claim_id, c.claim_id);
    }

    #[tokio::test]
    async fn transfer_claim_to_self_is_idempotent_success() {
        // Self-transfer (to_owner == claimed_by) is legal — useful for
        // setting `reason` for audit without changing ownership.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        let c = bus.claim_offset("work", 0, "worker-A", 30_000).unwrap();
        let t = bus
            .transfer_claim(&c.claim_id, "worker-A", "worker-A", Some("self-tag"))
            .unwrap();
        assert_eq!(t.from_owner, "worker-A");
        assert_eq!(t.to_owner, "worker-A");
        // Owner is unchanged from worker-A's perspective; release still works.
        let r = bus.release_claim(&c.claim_id, "worker-A", true).unwrap();
        assert!(r.ack);
    }

    #[tokio::test]
    async fn transfer_claim_then_release_by_new_owner_advances_new_owner_cursor() {
        // Cursor advances belong to whoever holds the claim at release time.
        // After A → B transfer + ack release by B, B's cursor advances past
        // the offset and A's cursor stays at its prior position (None here).
        let (_dir, bus) = tmp_bus();
        bus.create_topic("work", Retention::Forever).unwrap();
        bus.post("work", &env("work", b"m0")).await.unwrap();
        bus.post("work", &env("work", b"m1")).await.unwrap();
        let c = bus.claim_offset("work", 0, "worker-A", 30_000).unwrap();
        bus.transfer_claim(&c.claim_id, "worker-B", "worker-A", None)
            .unwrap();
        let r = bus.release_claim(&c.claim_id, "worker-B", true).unwrap();
        assert!(r.ack);
        assert_eq!(bus.get_cursor("worker-A", "work").unwrap(), None);
        assert_eq!(bus.get_cursor("worker-B", "work").unwrap(), Some(1));
    }

    // T-2109: find_idle_agents_from_hint — cv_index-driven fast path.
    // Same shape contract as find_idle_agents but driven by a caller-supplied
    // [(agent_id, presence_offset)] list. Tests mirror the walk-path tests
    // above to prove behavior is identical at the boundary.

    #[tokio::test]
    async fn find_idle_from_hint_missing_topic_returns_empty() {
        let (_dir, bus) = tmp_bus();
        let hint: Vec<(String, Offset)> = vec![("agent-a".to_string(), 0)];
        let idle = bus
            .find_idle_agents_from_hint(None, &[], 60_000, None, &hint)
            .unwrap();
        assert!(idle.is_empty(), "missing topic must return empty");
    }

    #[tokio::test]
    async fn find_idle_from_hint_resolves_offsets_to_envelopes() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 1_000, Some("claude-code"), None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("agent-b", now - 5_000, Some("claude-code"), None))
            .await
            .unwrap();
        // cv_index would record agent-a → 0, agent-b → 1. Simulate that.
        let hint = vec![("agent-a".to_string(), 0u64), ("agent-b".to_string(), 1u64)];
        let idle = bus
            .find_idle_agents_from_hint(None, &[], 60_000, None, &hint)
            .unwrap();
        assert_eq!(idle.len(), 2);
        // Sorted freshest-first — must match walk-path behavior.
        assert_eq!(idle[0].agent_id, "agent-a");
        assert_eq!(idle[1].agent_id, "agent-b");
    }

    #[tokio::test]
    async fn find_idle_walk_finds_overflow_agent_that_hint_misses() {
        // T-2458 (round-14 F2): cv_index caps distinct keys per topic and never
        // evicts, so once `agent-presence` saturates its cap a newly-heartbeating
        // LIVE agent overflows and is ABSENT from the hint's cv_entries. This test
        // pins the divergence the handler-side fix (channel.rs) guards against: the
        // authoritative walk MUST still see the overflowed agent that the lossy
        // hint cannot. `agent-b` here stands in for the post-saturation overflow
        // advertiser — present in the log, missing from the hint.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 1_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("agent-b", now - 1_000, None, None))
            .await
            .unwrap();

        // Hint carries ONLY agent-a (agent-b overflowed the cap → not indexed).
        let incomplete_hint = vec![("agent-a".to_string(), 0u64)];
        let via_hint = bus
            .find_idle_agents_from_hint(None, &[], 60_000, None, &incomplete_hint)
            .unwrap();
        let hint_ids: std::collections::HashSet<&str> =
            via_hint.iter().map(|a| a.agent_id.as_str()).collect();
        assert!(
            !hint_ids.contains("agent-b"),
            "the lossy hint must NOT see the overflowed agent (this is the bug)"
        );

        // The authoritative walk sees BOTH — including the overflowed agent-b.
        let via_walk = bus.find_idle_agents(None, &[], 60_000, None).unwrap();
        let walk_ids: std::collections::HashSet<&str> =
            via_walk.iter().map(|a| a.agent_id.as_str()).collect();
        assert!(
            walk_ids.contains("agent-a") && walk_ids.contains("agent-b"),
            "the walk is ground truth — it must find the overflowed agent the hint dropped"
        );
    }

    #[tokio::test]
    async fn find_idle_from_hint_skips_swept_offsets() {
        // cv_index can carry an offset whose envelope has been swept by
        // retention. The fast path must skip such entries cleanly, not panic
        // and not error.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("agent-a", now - 1_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("agent-b", now - 2_000, None, None))
            .await
            .unwrap();
        // Trim records < 1 — sweeps offset 0 (agent-a's heartbeat).
        bus.trim_topic("agent-presence", Some(1)).unwrap();

        // Hint still references the swept offset 0 — fast path skips it.
        let hint = vec![("agent-a".to_string(), 0u64), ("agent-b".to_string(), 1u64)];
        let idle = bus
            .find_idle_agents_from_hint(None, &[], 60_000, None, &hint)
            .unwrap();
        assert_eq!(idle.len(), 1, "swept offset must be skipped");
        assert_eq!(idle[0].agent_id, "agent-b");
    }

    #[tokio::test]
    async fn find_idle_from_hint_applies_role_filter() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post(
            "agent-presence",
            &heartbeat_env("claude-1", now - 100, Some("claude-code"), None),
        )
        .await
        .unwrap();
        bus.post(
            "agent-presence",
            &heartbeat_env("worker-1", now - 100, Some("test-worker"), None),
        )
        .await
        .unwrap();
        let hint = vec![("claude-1".to_string(), 0u64), ("worker-1".to_string(), 1u64)];
        let idle = bus
            .find_idle_agents_from_hint(Some("claude-code"), &[], 60_000, None, &hint)
            .unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "claude-1");
    }

    #[tokio::test]
    async fn find_idle_from_hint_applies_capability_filter() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post(
            "agent-presence",
            &heartbeat_env("alpha", now - 100, None, Some("rust,deploy,review")),
        )
        .await
        .unwrap();
        bus.post(
            "agent-presence",
            &heartbeat_env("beta", now - 100, None, Some("rust,review")),
        )
        .await
        .unwrap();
        let hint = vec![("alpha".to_string(), 0u64), ("beta".to_string(), 1u64)];
        let req = vec!["rust".to_string(), "deploy".to_string()];
        let idle = bus
            .find_idle_agents_from_hint(None, &req, 60_000, None, &hint)
            .unwrap();
        assert_eq!(idle.len(), 1, "only agents matching all caps remain");
        assert_eq!(idle[0].agent_id, "alpha");
    }

    #[tokio::test]
    async fn find_idle_from_hint_excludes_active_claimers() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        bus.create_topic("work-queue", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("worker-busy", now - 500, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("worker-free", now - 500, None, None))
            .await
            .unwrap();
        bus.post("work-queue", &env("work-queue", b"task-1")).await.unwrap();
        bus.claim_offset("work-queue", 0, "worker-busy", 60_000)
            .unwrap();

        let hint = vec![("worker-busy".to_string(), 0u64), ("worker-free".to_string(), 1u64)];
        let idle = bus
            .find_idle_agents_from_hint(None, &[], 60_000, None, &hint)
            .unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "worker-free");
    }

    #[tokio::test]
    async fn find_idle_from_hint_filters_stale_outside_live_window() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("fresh", now - 1_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("stale", now - 120_000, None, None))
            .await
            .unwrap();
        let hint = vec![("fresh".to_string(), 0u64), ("stale".to_string(), 1u64)];
        let idle = bus
            .find_idle_agents_from_hint(None, &[], 60_000, None, &hint)
            .unwrap();
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].agent_id, "fresh");
    }

    #[tokio::test]
    async fn find_idle_from_hint_limit_truncates_after_sort() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("agent-presence", Retention::Forever).unwrap();
        let now = now_unix_ms();
        bus.post("agent-presence", &heartbeat_env("a", now - 3_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("b", now - 1_000, None, None))
            .await
            .unwrap();
        bus.post("agent-presence", &heartbeat_env("c", now - 2_000, None, None))
            .await
            .unwrap();
        let hint = vec![
            ("a".to_string(), 0u64),
            ("b".to_string(), 1u64),
            ("c".to_string(), 2u64),
        ];
        let idle = bus
            .find_idle_agents_from_hint(None, &[], 60_000, Some(2), &hint)
            .unwrap();
        assert_eq!(idle.len(), 2);
        // Freshest first: b (now-1000), c (now-2000)
        assert_eq!(idle[0].agent_id, "b");
        assert_eq!(idle[1].agent_id, "c");
    }

    // T-2109: Bus::envelope_at — single-offset read primitive for substrate
    // primitive #2 (DISPATCH) cv_index fast path.

    #[tokio::test]
    async fn envelope_at_returns_envelope_when_offset_present() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"m0")).await.unwrap();
        bus.post("t", &env("t", b"m1")).await.unwrap();
        bus.post("t", &env("t", b"m2")).await.unwrap();

        let got = bus.envelope_at("t", 1).unwrap();
        assert!(got.is_some(), "expected envelope at offset 1");
        assert_eq!(got.unwrap().payload, b"m1");

        let got0 = bus.envelope_at("t", 0).unwrap();
        assert_eq!(got0.unwrap().payload, b"m0");
        let got2 = bus.envelope_at("t", 2).unwrap();
        assert_eq!(got2.unwrap().payload, b"m2");
    }

    #[tokio::test]
    async fn envelope_at_returns_none_when_offset_swept() {
        // Trim records 0..3 (keeping 3,4). Reading swept offset 1 returns
        // None — subscribe(t,1) lands on record at offset 3, which is NOT
        // the requested offset.
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0u32..5 {
            bus.post("t", &env("t", &i.to_le_bytes())).await.unwrap();
        }
        let trimmed = bus.trim_topic("t", Some(3)).unwrap();
        assert_eq!(trimmed, 3);

        // Requested offset 1 was swept; envelope_at returns None.
        let got = bus.envelope_at("t", 1).unwrap();
        assert!(got.is_none(), "expected None for swept offset");

        // Live offsets still resolve.
        let got3 = bus.envelope_at("t", 3).unwrap();
        assert!(got3.is_some());
    }

    #[tokio::test]
    async fn envelope_at_returns_none_when_offset_beyond_tail() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"m0")).await.unwrap();

        let got = bus.envelope_at("t", 99).unwrap();
        assert!(got.is_none(), "expected None for offset past tail");
    }

    #[tokio::test]
    async fn envelope_at_returns_unknown_topic_when_topic_missing() {
        let (_dir, bus) = tmp_bus();
        let err = bus.envelope_at("nonexistent", 0).unwrap_err();
        assert!(
            matches!(err, BusError::UnknownTopic(t) if t == "nonexistent"),
            "expected UnknownTopic"
        );
    }

    #[tokio::test]
    async fn envelope_at_empty_topic_returns_none() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let got = bus.envelope_at("t", 0).unwrap();
        assert!(got.is_none(), "expected None on empty topic");
    }

    // ── T-2431: async/concurrency test-debt slice ───────────────────────
    // Round-1 review sweep-B verdict: the bus core (append, sweep, claims)
    // carried the risk concentration but was tested almost exclusively
    // single-threaded. These tests exercise the invariants that only break
    // under interleaving — the T-2258 class (read-path stall under
    // concurrent write) taught that this crate's failure modes are
    // concurrency-shaped.

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_posts_all_land_with_unique_gapless_offsets() {
        let (_dir, bus) = tmp_bus();
        let bus = std::sync::Arc::new(bus);
        bus.create_topic("t", Retention::Forever).unwrap();
        const TASKS: usize = 8;
        const PER_TASK: usize = 25;
        let mut handles = Vec::new();
        for w in 0..TASKS {
            let b = bus.clone();
            handles.push(tokio::spawn(async move {
                for i in 0..PER_TASK {
                    b.post("t", &env("t", format!("w{w}-m{i}").as_bytes()))
                        .await
                        .expect("concurrent post must not error");
                }
            }));
        }
        for h in handles {
            h.await.expect("post task panicked");
        }
        let mut offsets: Vec<u64> = bus
            .subscribe("t", 0)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        offsets.sort_unstable();
        let expected: Vec<u64> = (0..(TASKS * PER_TASK) as u64).collect();
        assert_eq!(offsets, expected, "lost or duplicated write under concurrency");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn posts_racing_sweep_never_lose_fresh_records() {
        let (_dir, bus) = tmp_bus();
        let bus = std::sync::Arc::new(bus);
        bus.create_topic("t", Retention::Messages(20)).unwrap();
        const TOTAL: usize = 120;
        let poster = {
            let b = bus.clone();
            tokio::spawn(async move {
                for i in 0..TOTAL {
                    b.post("t", &env("t", format!("m{i}").as_bytes()))
                        .await
                        .expect("post during sweep race must not error");
                    if i % 10 == 0 {
                        tokio::task::yield_now().await;
                    }
                }
            })
        };
        let sweeper = {
            let b = bus.clone();
            tokio::spawn(async move {
                for _ in 0..30 {
                    // Sweeps race the poster; each must be internally
                    // consistent (no panic, no error, never over-prunes).
                    b.sweep("t", 0).expect("sweep during post race must not error");
                    tokio::task::yield_now().await;
                }
            })
        };
        poster.await.expect("poster panicked");
        sweeper.await.expect("sweeper panicked");
        // Final sweep from quiescence: exactly the retention bound remains,
        // and it is exactly the NEWEST records (tail intact, no holes).
        bus.sweep("t", 0).unwrap();
        let offsets: Vec<u64> = bus
            .subscribe("t", 0)
            .unwrap()
            .map(|r| r.unwrap().0)
            .collect();
        let expected: Vec<u64> = ((TOTAL as u64 - 20)..TOTAL as u64).collect();
        assert_eq!(
            offsets, expected,
            "sweep racing posts lost fresh records or left holes"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn claim_race_exactly_one_winner() {
        let (_dir, bus) = tmp_bus();
        let bus = std::sync::Arc::new(bus);
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"unit")).await.unwrap();
        const RACERS: usize = 10;
        let mut handles = Vec::new();
        for w in 0..RACERS {
            let b = bus.clone();
            handles.push(tokio::spawn(async move {
                b.claim_offset("t", 0, &format!("worker-{w}"), 60_000)
            }));
        }
        let mut wins = 0;
        let mut conflicts = 0;
        for h in handles {
            match h.await.expect("claim task panicked") {
                Ok(_) => wins += 1,
                Err(BusError::ClaimConflict { topic, offset }) => {
                    assert_eq!((topic.as_str(), offset), ("t", 0));
                    conflicts += 1;
                }
                Err(e) => panic!("unexpected claim error under race: {e}"),
            }
        }
        assert_eq!(wins, 1, "claim atomicity broken: {wins} winners");
        assert_eq!(conflicts, RACERS - 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn released_offset_reclaimable_while_held_claim_stays_exclusive() {
        let (_dir, bus) = tmp_bus();
        let bus = std::sync::Arc::new(bus);
        bus.create_topic("t", Retention::Forever).unwrap();
        bus.post("t", &env("t", b"u0")).await.unwrap();
        bus.post("t", &env("t", b"u1")).await.unwrap();

        // worker-A holds offset 0; worker-B holds offset 1 then releases
        // WITHOUT ack (return-for-retry).
        let a = bus.claim_offset("t", 0, "worker-A", 60_000).unwrap();
        let b_claim = bus.claim_offset("t", 1, "worker-B", 60_000).unwrap();
        bus.release_claim(&b_claim.claim_id, "worker-B", false).unwrap();

        // Racers: some try the still-held offset 0 (must ALL fail), some
        // try the released offset 1 (exactly one must win).
        let mut held_attempts = Vec::new();
        let mut freed_attempts = Vec::new();
        for w in 0..6 {
            let b = bus.clone();
            held_attempts.push(tokio::spawn(async move {
                b.claim_offset("t", 0, &format!("h{w}"), 60_000)
            }));
            let b = bus.clone();
            freed_attempts.push(tokio::spawn(async move {
                b.claim_offset("t", 1, &format!("f{w}"), 60_000)
            }));
        }
        for h in held_attempts {
            assert!(
                matches!(
                    h.await.expect("racer panicked"),
                    Err(BusError::ClaimConflict { .. })
                ),
                "a held claim leaked to another worker"
            );
        }
        let freed_wins = {
            let mut wins = 0;
            for h in freed_attempts {
                if h.await.expect("racer panicked").is_ok() {
                    wins += 1;
                }
            }
            wins
        };
        assert_eq!(freed_wins, 1, "released offset must be claimable exactly once");
        // Original holder can still release cleanly.
        bus.release_claim(&a.claim_id, "worker-A", true).unwrap();
    }
}
