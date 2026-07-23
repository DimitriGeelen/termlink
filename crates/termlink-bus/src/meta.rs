use std::path::Path;

use rusqlite::{Connection, params};

use crate::claim::{ClaimInfo, ClaimsSummary, ReleaseInfo, TransferInfo};
use crate::{BusError, Result, Retention};

/// SQLite sidecar tracking topics, cursors, and per-topic offset counters.
/// Schema version is baked into a `schema_version` table — bump when the
/// shape changes.
pub(crate) struct Meta {
    conn: std::sync::Mutex<Connection>,
}

impl Meta {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
        })
    }

    /// Idempotent topic creation. Returns `Ok(true)` when the topic was
    /// newly inserted by this call, `Ok(false)` when a row with the same
    /// (name, retention) already existed. T-1429.5 added the bool so
    /// clients can do "describe-on-first-create" without re-emitting
    /// topic_metadata envelopes on every idempotent re-call.
    pub(crate) fn create_topic(&self, name: &str, retention: Retention) -> Result<bool> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let existing: Option<(String, i64)> = conn
            .query_row(
                "SELECT retention_kind, retention_value FROM topics WHERE name = ?1",
                params![name],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        if let Some((kind, value)) = existing {
            let got = Retention::from_parts(&kind, value).unwrap_or(Retention::Forever);
            if got != retention {
                return Err(BusError::TopicPolicyMismatch {
                    name: name.to_string(),
                    existing: got,
                    requested: retention,
                });
            }
            return Ok(false);
        }
        let now_ms = now_unix_ms();
        conn.execute(
            "INSERT INTO topics (name, retention_kind, retention_value, created_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![name, retention.kind(), retention.value(), now_ms],
        )?;
        conn.execute(
            "INSERT INTO offsets (topic, next_offset) VALUES (?1, 0)",
            params![name],
        )?;
        Ok(true)
    }

    pub(crate) fn list_topics(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = conn.prepare("SELECT name FROM topics ORDER BY name")?;
        let names = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(names)
    }

    pub(crate) fn topic_retention(&self, name: &str) -> Result<Option<Retention>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let row: rusqlite::Result<(String, i64)> = conn.query_row(
            "SELECT retention_kind, retention_value FROM topics WHERE name = ?1",
            params![name],
            |r| Ok((r.get(0)?, r.get(1)?)),
        );
        match row {
            Ok((k, v)) => Ok(Retention::from_parts(&k, v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BusError::Sqlite(e)),
        }
    }

    /// T-2244 (R2a): change the retention policy of an ALREADY-EXISTING
    /// topic. `create_topic` deliberately refuses a policy change (returns
    /// `TopicPolicyMismatch`) so an idempotent re-create can't silently
    /// re-tune a topic; this is the explicit opt-in to change it. Returns
    /// `true` if the topic existed and was updated, `false` if no such topic
    /// (a no-op, NOT an error — caller decides whether absence is a problem;
    /// the CLI surfaces it as a clear "unknown topic" rather than creating).
    /// Does not sweep — the caller runs `Bus::sweep` to enforce the new
    /// policy against existing records.
    pub(crate) fn set_topic_retention(&self, name: &str, retention: Retention) -> Result<bool> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let n = conn.execute(
            "UPDATE topics SET retention_kind = ?1, retention_value = ?2 WHERE name = ?3",
            params![retention.kind(), retention.value(), name],
        )?;
        Ok(n > 0)
    }

    pub(crate) fn topic_exists(&self, name: &str) -> Result<bool> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM topics WHERE name = ?1",
            params![name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    /// Atomically reserve the next offset for `topic` and insert a
    /// `records` row pointing at the given byte position. Returns the
    /// offset assigned.
    pub(crate) fn record_append(
        &self,
        topic: &str,
        byte_pos: u64,
        length: u64,
        ts_unix_ms: i64,
    ) -> Result<u64> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let current: i64 = tx.query_row(
            "SELECT next_offset FROM offsets WHERE topic = ?1",
            params![topic],
            |r| r.get(0),
        )?;
        tx.execute(
            "UPDATE offsets SET next_offset = ?1 WHERE topic = ?2",
            params![current + 1, topic],
        )?;
        tx.execute(
            "INSERT INTO records (topic, offset, byte_pos, length, ts_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![topic, current, byte_pos as i64, length as i64, ts_unix_ms],
        )?;
        tx.commit()?;
        Ok(current as u64)
    }

    /// Delete records from `topic`. `before_offset=Some(N)` removes
    /// records with offset strictly less than N; `before_offset=None`
    /// removes ALL records for the topic. Returns count deleted.
    /// Index-only delete (log file bytes remain — same convention as
    /// `sweep_records`). Unknown topic returns `Ok(0)`. T-1234 / T-1230a.
    pub(crate) fn trim_records(&self, topic: &str, before_offset: Option<u64>) -> Result<u64> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let removed = match before_offset {
            Some(off) => conn.execute(
                "DELETE FROM records WHERE topic = ?1 AND offset < ?2",
                params![topic, off as i64],
            )?,
            None => conn.execute(
                "DELETE FROM records WHERE topic = ?1",
                params![topic],
            )?,
        };
        Ok(removed as u64)
    }

    /// T-2421: delete a topic and ALL its associated state — registry row,
    /// records index, per-subscriber cursors, offset counter, and claims —
    /// in one transaction. Distinct from `trim_records` (which empties a
    /// topic but leaves it registered): after this, the topic no longer
    /// appears in `list_topics` and a re-create starts fresh at offset 0.
    /// Returns `Ok(Some(record_count_removed))` when the topic existed,
    /// `Ok(None)` when no such topic was registered (caller decides whether
    /// absence is an error — the hub reports it loudly, no stealth success).
    pub(crate) fn delete_topic(&self, name: &str) -> Result<Option<u64>> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let existed: i64 = tx.query_row(
            "SELECT COUNT(*) FROM topics WHERE name = ?1",
            params![name],
            |r| r.get(0),
        )?;
        if existed == 0 {
            return Ok(None);
        }
        let records: i64 = tx.query_row(
            "SELECT COUNT(*) FROM records WHERE topic = ?1",
            params![name],
            |r| r.get(0),
        )?;
        tx.execute("DELETE FROM records WHERE topic = ?1", params![name])?;
        tx.execute("DELETE FROM cursors WHERE topic = ?1", params![name])?;
        tx.execute("DELETE FROM offsets WHERE topic = ?1", params![name])?;
        tx.execute("DELETE FROM claims WHERE topic = ?1", params![name])?;
        tx.execute("DELETE FROM topics WHERE name = ?1", params![name])?;
        tx.commit()?;
        Ok(Some(records as u64))
    }

    /// Count records currently indexed for `topic`. Records pruned by
    /// `sweep_records` are not counted even if their bytes remain in the
    /// log file. Unknown topic returns `Ok(0)` rather than an error so
    /// callers can aggregate over a prefix without per-topic existence
    /// checks (T-1233 / T-1229a).
    pub(crate) fn count_records(&self, topic: &str) -> Result<u64> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM records WHERE topic = ?1",
            params![topic],
            |r| r.get(0),
        )?;
        Ok(count as u64)
    }

    /// Smallest offset still indexed for `topic`, or `None` if the topic has
    /// zero live records (either never posted to, or fully swept). Used by
    /// subscribers to detect that their cursor fell behind the retention
    /// window and records were silently dropped (T-1285).
    pub(crate) fn oldest_offset(&self, topic: &str) -> Result<Option<u64>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let v: rusqlite::Result<Option<i64>> = conn.query_row(
            "SELECT MIN(offset) FROM records WHERE topic = ?1",
            params![topic],
            |r| r.get(0),
        );
        match v {
            Ok(Some(n)) => Ok(Some(n as u64)),
            Ok(None) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BusError::Sqlite(e)),
        }
    }

    /// Fetch record locators for `topic` with `offset >= cursor`, ordered.
    pub(crate) fn records_from(&self, topic: &str, cursor: u64) -> Result<Vec<RecordLoc>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT offset, byte_pos, length FROM records \
             WHERE topic = ?1 AND offset >= ?2 ORDER BY offset",
        )?;
        let rows = stmt.query_map(params![topic, cursor as i64], |r| {
            Ok(RecordLoc {
                offset: r.get::<_, i64>(0)? as u64,
                byte_pos: r.get::<_, i64>(1)? as u64,
                length: r.get::<_, i64>(2)? as u64,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(BusError::Sqlite)
    }

    /// Persist a subscriber cursor for the given (subscriber, topic).
    ///
    /// The cursor is a monotonic delivery frontier: the upsert takes
    /// `MAX(existing, incoming)` so a stale or retried advance can never
    /// regress the frontier and re-deliver already-consumed records (T-2462,
    /// round-16 F3). This mirrors the claim-ack cursor-advance path
    /// (`release_claim`, same MAX upsert) — both express the same
    /// at-least-once invariant. `advance_cursor` (the sole caller) is
    /// advance-only by contract; there is no legitimate rewind/seek caller.
    pub(crate) fn put_cursor(
        &self,
        subscriber_id: &str,
        topic: &str,
        offset: u64,
    ) -> Result<()> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        conn.execute(
            "INSERT INTO cursors (subscriber_id, topic, last_offset) VALUES (?1, ?2, ?3) \
             ON CONFLICT(subscriber_id, topic) DO UPDATE SET \
             last_offset = MAX(last_offset, excluded.last_offset)",
            params![subscriber_id, topic, offset as i64],
        )?;
        Ok(())
    }

    /// Read a subscriber cursor. Returns `None` if never persisted.
    pub(crate) fn get_cursor(&self, subscriber_id: &str, topic: &str) -> Result<Option<u64>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let v: rusqlite::Result<i64> = conn.query_row(
            "SELECT last_offset FROM cursors WHERE subscriber_id = ?1 AND topic = ?2",
            params![subscriber_id, topic],
            |r| r.get(0),
        );
        match v {
            Ok(n) => Ok(Some(n as u64)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BusError::Sqlite(e)),
        }
    }

    /// Delete records matching a retention policy. Returns the set of
    /// record locators that were removed (so the caller can rewrite the
    /// on-disk log, if desired). Current callers leave dead bytes in the
    /// file — compaction is a follow-up.
    pub(crate) fn sweep_records(
        &self,
        topic: &str,
        keep_after_ts_ms: Option<i64>,
        keep_last_n: Option<u64>,
    ) -> Result<u64> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let mut deleted: u64 = 0;
        if let Some(ts) = keep_after_ts_ms {
            let n = tx.execute(
                "DELETE FROM records WHERE topic = ?1 AND ts_unix_ms < ?2",
                params![topic, ts],
            )?;
            deleted += n as u64;
        }
        if let Some(n) = keep_last_n {
            let total: i64 = tx.query_row(
                "SELECT COUNT(*) FROM records WHERE topic = ?1",
                params![topic],
                |r| r.get(0),
            )?;
            if (total as u64) > n {
                let drop_n = (total as u64) - n;
                let removed = tx.execute(
                    "DELETE FROM records WHERE rowid IN (\
                        SELECT rowid FROM records WHERE topic = ?1 \
                        ORDER BY offset ASC LIMIT ?2\
                     )",
                    params![topic, drop_n as i64],
                )?;
                deleted += removed as u64;
            }
        }
        tx.commit()?;
        Ok(deleted)
    }

    /// Delete the specific `offsets` from `topic` in a single transaction.
    /// Index-only delete (log file bytes remain — same convention as
    /// `sweep_records`/`trim_records`). Offsets not present are skipped.
    /// Empty input is a no-op. Used by the per-cv_key compaction sweep
    /// (`Bus::compact_per_cv_key`, R2b/T-2245) which computes the exact
    /// stale-record set in the bus layer and asks the meta layer to remove it.
    /// Returns count actually deleted.
    pub(crate) fn delete_records_at(&self, topic: &str, offsets: &[u64]) -> Result<u64> {
        if offsets.is_empty() {
            return Ok(0);
        }
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let mut deleted: u64 = 0;
        {
            let mut stmt =
                tx.prepare("DELETE FROM records WHERE topic = ?1 AND offset = ?2")?;
            for off in offsets {
                deleted += stmt.execute(params![topic, *off as i64])? as u64;
            }
        }
        tx.commit()?;
        Ok(deleted)
    }

    /// T-2029: attempt to claim `(topic, offset)` for `claimer` until
    /// `now_ms + ttl_ms`. Lazily evicts any prior claim past its
    /// `claimed_until` before inserting. Returns `ClaimConflict` when an
    /// unexpired claim still holds the slot.
    pub(crate) fn claim_offset(
        &self,
        topic: &str,
        offset: u64,
        claimer: &str,
        ttl_ms: u32,
        now_ms: i64,
    ) -> Result<ClaimInfo> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        // Lazy expiry: drop any expired claim on this (topic, offset).
        tx.execute(
            "DELETE FROM claims \
             WHERE topic = ?1 AND offset = ?2 AND claimed_until <= ?3",
            params![topic, offset as i64, now_ms],
        )?;
        let claimed_until = now_ms.saturating_add(i64::from(ttl_ms));
        let claim_id = generate_claim_id(topic, offset, now_ms);
        let attempt = tx.execute(
            "INSERT INTO claims \
             (claim_id, topic, offset, claimed_by, claimed_at, claimed_until) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                claim_id,
                topic,
                offset as i64,
                claimer,
                now_ms,
                claimed_until
            ],
        );
        match attempt {
            Ok(_) => {
                tx.commit()?;
                Ok(ClaimInfo {
                    claim_id,
                    topic: topic.to_string(),
                    offset,
                    claimer: claimer.to_string(),
                    claimed_at: now_ms,
                    claimed_until,
                })
            }
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                // UNIQUE(topic, offset) — another worker holds an active claim.
                drop(tx);
                Err(BusError::ClaimConflict {
                    topic: topic.to_string(),
                    offset,
                })
            }
            Err(e) => Err(BusError::Sqlite(e)),
        }
    }

    /// T-2029: release a claim. When `ack=true` the claimer's cursor for
    /// `topic` is advanced past `offset` (so a re-subscribe skips it); when
    /// `ack=false` the cursor is left intact and the slot becomes claimable
    /// again. Both paths delete the claim row.
    pub(crate) fn release_claim(
        &self,
        claim_id: &str,
        claimer: &str,
        ack: bool,
    ) -> Result<ReleaseInfo> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let row: rusqlite::Result<(String, i64, String)> = tx.query_row(
            "SELECT topic, offset, claimed_by FROM claims WHERE claim_id = ?1",
            params![claim_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        );
        let (topic, offset_i, claimed_by) = match row {
            Ok(t) => t,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(BusError::ClaimNotFound(claim_id.to_string()));
            }
            Err(e) => return Err(BusError::Sqlite(e)),
        };
        if claimed_by != claimer {
            return Err(BusError::ClaimNotOwned {
                claim_id: claim_id.to_string(),
                claimed_by,
                attempted_by: claimer.to_string(),
            });
        }
        let offset_u = offset_i as u64;
        if ack {
            // Advance the claimer's cursor monotonically past this offset.
            tx.execute(
                "INSERT INTO cursors (subscriber_id, topic, last_offset) \
                 VALUES (?1, ?2, ?3) \
                 ON CONFLICT(subscriber_id, topic) DO UPDATE SET \
                 last_offset = MAX(last_offset, excluded.last_offset)",
                params![claimer, topic, (offset_u + 1) as i64],
            )?;
        }
        tx.execute(
            "DELETE FROM claims WHERE claim_id = ?1",
            params![claim_id],
        )?;
        tx.commit()?;
        Ok(ReleaseInfo {
            claim_id: claim_id.to_string(),
            topic,
            offset: offset_u,
            ack,
            forced_from: None,
            forced_reason: None,
        })
    }

    /// T-2044 (arc-parallel-substrate Slice 11): operator-Tier-0 force release.
    /// Same DELETE path as `release_claim` but WITHOUT the
    /// `claimed_by == claimer` ownership check — used when an operator needs
    /// to break a stuck claim faster than the natural TTL expiry. Cursor is
    /// NOT advanced (`ack=false` semantics — work returns for retry, not
    /// silently consumed). The `reason` parameter is recorded in the returned
    /// `ReleaseInfo.forced_reason` field for audit-trail surface in higher
    /// layers; not persisted in the claims table (which is current-state only).
    ///
    /// Returns `BusError::ClaimNotFound` for an unknown / already-released
    /// claim. Does NOT return `ClaimNotOwned` — that's the whole point of the
    /// force path.
    pub(crate) fn force_release_claim(
        &self,
        claim_id: &str,
        reason: Option<&str>,
    ) -> Result<ReleaseInfo> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let row: rusqlite::Result<(String, i64, String)> = tx.query_row(
            "SELECT topic, offset, claimed_by FROM claims WHERE claim_id = ?1",
            params![claim_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        );
        let (topic, offset_i, claimed_by) = match row {
            Ok(t) => t,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(BusError::ClaimNotFound(claim_id.to_string()));
            }
            Err(e) => return Err(BusError::Sqlite(e)),
        };
        let offset_u = offset_i as u64;
        tx.execute(
            "DELETE FROM claims WHERE claim_id = ?1",
            params![claim_id],
        )?;
        tx.commit()?;
        Ok(ReleaseInfo {
            claim_id: claim_id.to_string(),
            topic,
            offset: offset_u,
            ack: false,
            forced_from: Some(claimed_by),
            forced_reason: reason.map(|s| s.to_string()),
        })
    }

    /// T-2046 (T-2021 GO, arc-parallel-substrate primitive #3): atomic
    /// ownership transfer of an existing claim. Gates SELECT → expired
    /// (lazy-evict + `ClaimExpired`) → `claimed_by == by` (`ClaimNotOwned`)
    /// → UPDATE `claimed_by = to_owner` in a single transaction. The lease
    /// timestamps (`claimed_at`, `claimed_until`) are preserved — transfer
    /// is an ownership transition, not a renewal. Caller's optional
    /// `reason` is returned verbatim in `TransferInfo.reason` for upstream
    /// audit surface; not persisted in the claims table.
    ///
    /// Distinct from `force_release_claim`: this verb is the cooperative,
    /// owner-checked path used by orchestrators handing a unit of work to
    /// a chosen worker. The operator-Tier-0 ownership-bypass path is
    /// `force_release_claim` followed by a fresh `claim_offset`.
    pub(crate) fn transfer_claim(
        &self,
        claim_id: &str,
        to_owner: &str,
        by: &str,
        reason: Option<&str>,
        now_ms: i64,
    ) -> Result<TransferInfo> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let row: rusqlite::Result<(String, i64, String, i64, i64)> = tx.query_row(
            "SELECT topic, offset, claimed_by, claimed_at, claimed_until \
             FROM claims WHERE claim_id = ?1",
            params![claim_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        );
        let (topic, offset_i, claimed_by, claimed_at, claimed_until) = match row {
            Ok(t) => t,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(BusError::ClaimNotFound(claim_id.to_string()));
            }
            Err(e) => return Err(BusError::Sqlite(e)),
        };
        if claimed_until <= now_ms {
            tx.execute(
                "DELETE FROM claims WHERE claim_id = ?1",
                params![claim_id],
            )?;
            tx.commit()?;
            return Err(BusError::ClaimExpired {
                claim_id: claim_id.to_string(),
            });
        }
        if claimed_by != by {
            return Err(BusError::ClaimNotOwned {
                claim_id: claim_id.to_string(),
                claimed_by,
                attempted_by: by.to_string(),
            });
        }
        tx.execute(
            "UPDATE claims SET claimed_by = ?1 WHERE claim_id = ?2",
            params![to_owner, claim_id],
        )?;
        tx.commit()?;
        Ok(TransferInfo {
            claim_id: claim_id.to_string(),
            topic,
            offset: offset_i as u64,
            from_owner: claimed_by,
            to_owner: to_owner.to_string(),
            claimed_at,
            claimed_until,
            reason: reason.map(|s| s.to_string()),
        })
    }

    /// T-2030: extend the lease on `claim_id` by `additional_ttl_ms`. Gates:
    /// (1) row must exist, (2) row must NOT be past `claimed_until` (else
    /// `ClaimExpired` — and the stale row is lazily evicted so a follow-up
    /// `claim_offset` can succeed), (3) `claimed_by == claimer`. On success
    /// sets `claimed_until = now_ms + additional_ttl_ms` and returns the
    /// refreshed `ClaimInfo`.
    pub(crate) fn renew_claim(
        &self,
        claim_id: &str,
        claimer: &str,
        additional_ttl_ms: u32,
        now_ms: i64,
    ) -> Result<ClaimInfo> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let row: rusqlite::Result<(String, i64, String, i64, i64)> = tx.query_row(
            "SELECT topic, offset, claimed_by, claimed_at, claimed_until \
             FROM claims WHERE claim_id = ?1",
            params![claim_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        );
        let (topic, offset_i, claimed_by, claimed_at, old_until) = match row {
            Ok(t) => t,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(BusError::ClaimNotFound(claim_id.to_string()));
            }
            Err(e) => return Err(BusError::Sqlite(e)),
        };
        if old_until <= now_ms {
            // Lazy evict: drop the stale row so a follow-up `claim_offset` can
            // succeed against this (topic, offset). Caller sees ClaimExpired
            // — a different code than ClaimNotFound so the client can tell
            // "your lease lapsed" from "wrong id".
            tx.execute(
                "DELETE FROM claims WHERE claim_id = ?1",
                params![claim_id],
            )?;
            tx.commit()?;
            return Err(BusError::ClaimExpired {
                claim_id: claim_id.to_string(),
            });
        }
        if claimed_by != claimer {
            return Err(BusError::ClaimNotOwned {
                claim_id: claim_id.to_string(),
                claimed_by,
                attempted_by: claimer.to_string(),
            });
        }
        let new_until = now_ms.saturating_add(i64::from(additional_ttl_ms));
        tx.execute(
            "UPDATE claims SET claimed_until = ?1 WHERE claim_id = ?2",
            params![new_until, claim_id],
        )?;
        tx.commit()?;
        Ok(ClaimInfo {
            claim_id: claim_id.to_string(),
            topic,
            offset: offset_i as u64,
            claimer: claimed_by,
            claimed_at,
            claimed_until: new_until,
        })
    }

    /// T-2037: list current claim rows for `topic`. When
    /// `include_expired=false` (default), rows where `claimed_until <= now_ms`
    /// are filtered out. Ordering: by `offset ASC, claimed_at ASC` for stable
    /// operator-readable output.
    pub(crate) fn list_claims(
        &self,
        topic: &str,
        include_expired: bool,
        now_ms: i64,
    ) -> Result<Vec<ClaimInfo>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = if include_expired {
            conn.prepare(
                "SELECT claim_id, topic, offset, claimed_by, claimed_at, claimed_until \
                 FROM claims WHERE topic = ?1 ORDER BY offset ASC, claimed_at ASC",
            )?
        } else {
            conn.prepare(
                "SELECT claim_id, topic, offset, claimed_by, claimed_at, claimed_until \
                 FROM claims WHERE topic = ?1 AND claimed_until > ?2 \
                 ORDER BY offset ASC, claimed_at ASC",
            )?
        };
        let mapper = |r: &rusqlite::Row| -> rusqlite::Result<ClaimInfo> {
            let offset_i: i64 = r.get(2)?;
            Ok(ClaimInfo {
                claim_id: r.get(0)?,
                topic: r.get(1)?,
                offset: offset_i as u64,
                claimer: r.get(3)?,
                claimed_at: r.get(4)?,
                claimed_until: r.get(5)?,
            })
        };
        let rows = if include_expired {
            stmt.query_map(params![topic], mapper)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map(params![topic, now_ms], mapper)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        Ok(rows)
    }

    /// T-2045: list distinct `claimed_by` values across ALL topics where
    /// the claim is still active (`claimed_until > now`). This is the
    /// anti-join input for `agent.find_idle` — an agent currently holding
    /// any claim on any topic is busy and must be excluded from the idle
    /// roster. Returns a deduped, lexicographically-sorted vector for
    /// deterministic test output.
    pub(crate) fn distinct_active_claimers(&self, now_ms: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT DISTINCT claimed_by FROM claims \
             WHERE claimed_until > ?1 \
             ORDER BY claimed_by ASC",
        )?;
        let rows = stmt
            .query_map(params![now_ms], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// T-2039: aggregate claim state for `topic`. Single SQL over the
    /// `claims` table using `idx_claims_topic_until` — returns counts plus
    /// the oldest-active and next-expiry markers needed for operator
    /// observability ("is this topic busy?", "is anything stuck?").
    ///
    /// Returns zero counts and `None` markers when the topic has no claim
    /// rows at all. Caller is responsible for the topic-exists pre-check
    /// (mirrors `list_claims`).
    pub(crate) fn claims_summary(
        &self,
        topic: &str,
        now_ms: i64,
    ) -> Result<ClaimsSummary> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT \
               COALESCE(SUM(CASE WHEN claimed_until >  ?2 THEN 1 ELSE 0 END), 0) AS active, \
               COALESCE(SUM(CASE WHEN claimed_until <= ?2 THEN 1 ELSE 0 END), 0) AS expired, \
               MIN(CASE WHEN claimed_until >  ?2 THEN claimed_at    ELSE NULL END) AS oldest_active_at, \
               MIN(CASE WHEN claimed_until >  ?2 THEN claimed_until ELSE NULL END) AS next_active_expiry \
             FROM claims WHERE topic = ?1",
        )?;
        let summary = stmt.query_row(params![topic, now_ms], |r| {
            let active_i: i64 = r.get(0)?;
            let expired_i: i64 = r.get(1)?;
            let oldest_active_at_ms: Option<i64> = r.get(2)?;
            let next_active_expiry_ms: Option<i64> = r.get(3)?;
            let oldest_active_age_ms = oldest_active_at_ms
                .map(|t| (now_ms - t).max(0));
            Ok(ClaimsSummary {
                active_count: active_i.max(0) as u64,
                expired_count: expired_i.max(0) as u64,
                oldest_active_at_ms,
                oldest_active_age_ms,
                next_active_expiry_ms,
            })
        })?;
        Ok(summary)
    }
}

/// Process-monotonic disambiguator for `claim_id`. Guarantees no two claim_ids
/// minted in one process lifetime are ever equal, regardless of clock
/// granularity or topic-prefix collisions (T-2461 / round-15 F1).
static CLAIM_ID_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn generate_claim_id(topic: &str, offset: u64, now_ms: i64) -> String {
    // claim_id is the claims-table PRIMARY KEY, so it MUST be unique. It is NOT
    // a lossy projection of the topic: the `topic_tag` below is only a 16-char
    // human-readable HINT, while the FULL uniqueness domain is the topic. Two
    // distinct topics can share a 16-char sanitized prefix, so the tag alone
    // does not guarantee PK uniqueness (T-2461: a shared-prefix collision at the
    // same offset+nanosecond used to fail on the claim_id PK and be misreported
    // as ClaimConflict, spuriously denying a free slot). The monotonic `seq`
    // below makes claim_id collision-proof by construction; `now_ns` still
    // disambiguates across a process restart. With a collision-proof id, any
    // remaining ConstraintViolation on INSERT is genuinely a
    // UNIQUE(topic,offset) conflict, so the ClaimConflict mapping stays correct.
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or((now_ms as u128).saturating_mul(1_000_000));
    let seq = CLAIM_ID_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    compose_claim_id(now_ns, seq, topic, offset)
}

/// Pure `claim_id` composer — split out so the collision-resistance guarantee
/// (a fixed `now_ns` + shared topic-prefix + same offset still yields distinct
/// ids via `seq`) is unit-testable without wall-clock nondeterminism.
fn compose_claim_id(now_ns: u128, seq: u64, topic: &str, offset: u64) -> String {
    let topic_tag: String = topic
        .chars()
        .take(16)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("clm-{now_ns}-{seq}-{topic_tag}-{offset}")
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
         );
         CREATE TABLE IF NOT EXISTS topics (
            name           TEXT PRIMARY KEY,
            retention_kind TEXT NOT NULL,
            retention_value INTEGER NOT NULL,
            created_at     INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS cursors (
            subscriber_id TEXT NOT NULL,
            topic         TEXT NOT NULL,
            last_offset   INTEGER NOT NULL,
            PRIMARY KEY (subscriber_id, topic)
         );
         CREATE TABLE IF NOT EXISTS offsets (
            topic       TEXT PRIMARY KEY,
            next_offset INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS records (
            topic       TEXT NOT NULL,
            offset      INTEGER NOT NULL,
            byte_pos    INTEGER NOT NULL,
            length      INTEGER NOT NULL,
            ts_unix_ms  INTEGER NOT NULL,
            PRIMARY KEY (topic, offset)
         );
         CREATE INDEX IF NOT EXISTS idx_records_topic_ts
            ON records (topic, ts_unix_ms);
         -- T-2029: exclusive-delivery claims (arc-parallel-substrate Slice 1).
         -- One row per active claim; DELETEd on release or lazily on next
         -- claim attempt past claimed_until.
         CREATE TABLE IF NOT EXISTS claims (
            claim_id      TEXT PRIMARY KEY,
            topic         TEXT NOT NULL,
            offset        INTEGER NOT NULL,
            claimed_by    TEXT NOT NULL,
            claimed_at    INTEGER NOT NULL,
            claimed_until INTEGER NOT NULL
         );
         CREATE UNIQUE INDEX IF NOT EXISTS idx_claims_topic_offset_active
            ON claims (topic, offset);
         CREATE INDEX IF NOT EXISTS idx_claims_topic_until
            ON claims (topic, claimed_until);
         INSERT OR IGNORE INTO schema_version (version) VALUES (1);",
    )?;
    Ok(())
}

/// Pre-fetched record locator for streaming subscribe reads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RecordLoc {
    pub offset: u64,
    pub byte_pos: u64,
    pub length: u64,
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod claim_id_tests {
    use super::{compose_claim_id, generate_claim_id};

    #[test]
    fn compose_claim_id_distinct_for_shared_prefix_topics_at_same_instant() {
        // T-2461 (round-15 F1): two DISTINCT topics that sanitize to the SAME
        // 16-char tag, same offset, same nanosecond — the pre-fix PK-collision
        // case. The monotonic seq must keep their claim_ids distinct so a free
        // slot on topic B is not spuriously denied as ClaimConflict.
        let ns = 1_700_000_000_000_000_000u128;
        let a = compose_claim_id(ns, 0, "arc-parallel-substrate-a", 5);
        let b = compose_claim_id(ns, 1, "arc-parallel-substrate-b", 5);
        // Both truncate+sanitize to "arc_parallel_sub" — identical topic tag,
        // so the tag alone would collide; only the seq keeps them apart.
        assert!(
            a.contains("arc_parallel_sub") && b.contains("arc_parallel_sub"),
            "both topics must share the 16-char tag (the collision precondition)"
        );
        assert_ne!(
            a, b,
            "shared-prefix topics at the same instant must get distinct claim_ids"
        );
    }

    #[test]
    fn compose_claim_id_seq_disambiguates_identical_inputs() {
        // Even with every other field identical, the seq alone guarantees a
        // distinct id — the collision-proof-by-construction property.
        let ns = 42u128;
        assert_ne!(
            compose_claim_id(ns, 0, "t", 1),
            compose_claim_id(ns, 1, "t", 1)
        );
    }

    #[test]
    fn generate_claim_id_consecutive_calls_never_collide() {
        // The live generator bumps the process seq every call, so back-to-back
        // ids for identical (topic, offset) are always distinct.
        let x = generate_claim_id("same-topic", 7, 1_000);
        let y = generate_claim_id("same-topic", 7, 1_000);
        assert_ne!(x, y, "consecutive generate_claim_id calls must never collide");
    }
}
