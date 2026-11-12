//! Removal guard for mail topics: no sweep, trim, retention change or ceiling removes an open message silently.
//! T-3701 (arc-011 plan unit P-13, build item B-3; 04 ARC4 6.6.3 and V-3; 06a AL-31).
//!
//! The bus knows nothing about mail. The hub installs a [`RemovalGuard`] that
//! names the topics it guards (`inbox:*`, conversation topics) and reads, for a
//! set of records, whether each one's message is final. With a guard installed,
//! every removal path of the bus on a guarded topic first works out the exact
//! offsets it would remove, asks the guard, and then removes exactly those
//! offsets and no others:
//!
//! | path                         | an open message among the offsets | finality unreadable |
//! |------------------------------|-----------------------------------|---------------------|
//! | `sweep`, `trim_topic`, `set_topic_retention`, `compact_per_cv_key` | refused, [`BusError::OpenMail`] (FCW-126) | refused, [`BusError::FinalityUnreadable`] (FCW-127) |
//! | `sweep_final_only` (periodic) | kept; finals removed               | nothing removed, error |
//! | ceilings (V-3, on post and hub pass) | oldest final first; open only as a last resort, reported for needs-attention (FCW-129) | nothing removed, loud |
//!
//! Concurrency (AL-31 2.15.f). The removal is by explicit offset
//! (`delete_records_at`), so a post that lands between the check and the delete
//! is never removed: it was not among the checked offsets. A final stage never
//! becomes open again, so an offset read final stays final. Two removers (a
//! sweep and the ceiling pass) racing on one offset delete it once: a missing
//! offset is skipped and not counted.
//!
//! Forever topics: the pre-existing post-time ceilings (T-3310) only WARN on a
//! forever topic. A guarded topic gets the V-3 ceilings whatever its retention,
//! which is what bounds the forever inbox topics of T-3382.

use crate::{Bus, BusError, Envelope, Offset, Result, Retention};

/// V-3 per inbox or conversation topic: 50,000 records.
pub const V3_TOPIC_MAX_RECORDS: u64 = 50_000;
/// V-3 per inbox or conversation topic: 32 MiB.
pub const V3_TOPIC_MAX_BYTES: u64 = 32 * 1024 * 1024;
/// V-3 per hub for the arc-011 classes (all guarded topics together): 2 GiB.
pub const V3_HUB_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// The V-3 retention ceilings. `enabled: false` is the rollback ("ceilings off").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailCeilings {
    pub enabled: bool,
    pub topic_max_records: u64,
    pub topic_max_bytes: u64,
    pub hub_max_bytes: u64,
}

impl Default for MailCeilings {
    fn default() -> Self {
        Self {
            enabled: true,
            topic_max_records: V3_TOPIC_MAX_RECORDS,
            topic_max_bytes: V3_TOPIC_MAX_BYTES,
            hub_max_bytes: V3_HUB_MAX_BYTES,
        }
    }
}

/// What a ceiling pass removed from one topic, handed to the guard so each
/// removal is logged and each forced one raises needs-attention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeilingRemoval {
    pub topic: String,
    /// Final records removed (oldest first).
    pub final_offsets: Vec<Offset>,
    /// Open records removed as a last resort (R-35.o (3)); each needs attention.
    pub forced_open_offsets: Vec<Offset>,
    pub reason: &'static str,
}

/// The hub's knowledge of mail, as the bus needs it.
pub trait RemovalGuard: Send + Sync {
    /// Is `topic` a mail topic whose open messages must not be removed?
    fn guards(&self, topic: &str) -> bool;
    /// For each record, in order: `true` when its message is final (or it is
    /// not mail at all), `false` when open. `Err` when finality cannot be read;
    /// the caller then removes nothing.
    fn finality(
        &self,
        bus: &Bus,
        topic: &str,
        records: &[(Offset, Envelope)],
    ) -> std::result::Result<Vec<bool>, String>;
    /// Called once per topic a ceiling pass removed records from.
    fn on_ceiling_removal(&self, _removal: &CeilingRemoval) {}
}

/// One row of the records index: offset, timestamp, length in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexRow {
    pub offset: Offset,
    pub ts_ms: i64,
    pub len: u64,
}

/// A removal the caller asked for, before it is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalOp {
    /// What a sweep under this retention would remove now.
    Retention(Retention),
    /// What `trim_topic(before_offset)` would remove.
    Trim(Option<u64>),
}

const DAY_MS: i64 = 86_400_000;

/// Pure: the offsets `op` would remove from a topic whose index is `rows`
/// (ascending offset). Mirrors `Meta::sweep_records` / `trim_records`.
/// `LatestPerCvKey` needs the envelopes and is answered by
/// `Bus::per_cv_key_stale`, so it yields nothing here.
pub fn preview(rows: &[IndexRow], op: RemovalOp, now_ms: i64) -> Vec<Offset> {
    match op {
        RemovalOp::Trim(None) => rows.iter().map(|r| r.offset).collect(),
        RemovalOp::Trim(Some(b)) => rows.iter().filter(|r| r.offset < b).map(|r| r.offset).collect(),
        RemovalOp::Retention(r) => match r {
            Retention::Forever | Retention::LatestPerCvKey => Vec::new(),
            Retention::Days(d) => {
                let cutoff = now_ms - i64::from(d) * DAY_MS;
                rows.iter().filter(|r| r.ts_ms < cutoff).map(|r| r.offset).collect()
            }
            Retention::Messages(n) => oldest(rows, n),
            Retention::Latest => oldest(rows, 1),
        },
    }
}

fn oldest(rows: &[IndexRow], keep: u64) -> Vec<Offset> {
    let drop = (rows.len() as u64).saturating_sub(keep) as usize;
    rows.iter().take(drop).map(|r| r.offset).collect()
}

/// Pure (FCW-129): which records a ceiling pass removes. `rows` are
/// `(offset, len, final)` in removal order (oldest first). Finals go first
/// until both `need_records` and `need_bytes` are met; only if the finals are
/// exhausted and the topic is still over do open records go, oldest first.
/// Returns (final offsets, forced open offsets).
pub fn plan_ceiling(rows: &[(Offset, u64, bool)], need_records: u64, need_bytes: u64) -> (Vec<Offset>, Vec<Offset>) {
    let mut n = 0u64;
    let mut b = 0u64;
    let done = |n: u64, b: u64| n >= need_records && b >= need_bytes;
    let mut finals = Vec::new();
    for &(off, len, fin) in rows {
        if done(n, b) {
            break;
        }
        if fin {
            finals.push(off);
            n += 1;
            b = b.saturating_add(len);
        }
    }
    let mut forced = Vec::new();
    for &(off, len, fin) in rows {
        if done(n, b) {
            break;
        }
        if !fin {
            forced.push(off);
            n += 1;
            b = b.saturating_add(len);
        }
    }
    finals.sort_unstable();
    forced.sort_unstable();
    (finals, forced)
}

impl Bus {
    /// T-3701: install the hub's mail guard. Without one, nothing changes.
    pub fn set_removal_guard(&self, guard: std::sync::Arc<dyn RemovalGuard>) {
        *self.removal_guard.lock().expect("removal guard mutex poisoned") = Some(guard);
    }

    /// T-3701: replace the V-3 ceilings (tests shrink them; `enabled:false` is the rollback).
    pub fn set_mail_ceilings(&self, c: MailCeilings) {
        *self.mail_ceilings.lock().expect("mail ceilings mutex poisoned") = c;
    }

    pub fn mail_ceilings(&self) -> MailCeilings {
        *self.mail_ceilings.lock().expect("mail ceilings mutex poisoned")
    }

    /// The guard, when one is installed AND it guards `topic`.
    pub(crate) fn guard_for(&self, topic: &str) -> Option<std::sync::Arc<dyn RemovalGuard>> {
        let g = self.removal_guard.lock().expect("removal guard mutex poisoned").clone()?;
        g.guards(topic).then_some(g)
    }

    /// Is `topic` guarded (a guard is installed and claims it)?
    pub fn is_guarded(&self, topic: &str) -> bool {
        self.guard_for(topic).is_some()
    }

    /// The records index of `topic`, ascending offset.
    pub fn record_index(&self, topic: &str) -> Result<Vec<IndexRow>> {
        self.meta.record_index(topic)
    }

    /// The envelopes at `offsets` (ascending), read in one pass. An index row
    /// whose envelope cannot be read is an error: finality of a record that
    /// cannot be read cannot be judged.
    pub(crate) fn envelopes_at(&self, topic: &str, offsets: &[Offset]) -> Result<Vec<(Offset, Envelope)>> {
        let Some(&first) = offsets.iter().min() else {
            return Ok(Vec::new());
        };
        let want: std::collections::HashSet<Offset> = offsets.iter().copied().collect();
        let mut out = Vec::with_capacity(offsets.len());
        for item in self.subscribe(topic, first)? {
            let (off, env) = item?;
            if want.contains(&off) {
                out.push((off, env));
            }
        }
        if out.len() != want.len() {
            return Err(BusError::FinalityUnreadable {
                topic: topic.to_string(),
                reason: format!("{} of {} records could not be read", want.len() - out.len(), want.len()),
            });
        }
        Ok(out)
    }

    /// Finality of each record at `offsets` (same order as returned envelopes).
    fn finality_of(
        &self,
        guard: &dyn RemovalGuard,
        topic: &str,
        offsets: &[Offset],
    ) -> Result<Vec<(Offset, u64, bool)>> {
        let envs = self.envelopes_at(topic, offsets)?;
        let fin = guard
            .finality(self, topic, &envs)
            .map_err(|reason| BusError::FinalityUnreadable { topic: topic.to_string(), reason })?;
        if fin.len() != envs.len() {
            return Err(BusError::FinalityUnreadable {
                topic: topic.to_string(),
                reason: format!("guard answered {} of {} records", fin.len(), envs.len()),
            });
        }
        Ok(envs.iter().zip(fin).map(|((off, env), f)| (*off, env.payload.len() as u64, f)).collect())
    }

    /// FCW-126/127: refuse unless every offset's message is final.
    pub(crate) fn check_all_final(&self, guard: &dyn RemovalGuard, topic: &str, offsets: &[Offset]) -> Result<()> {
        if offsets.is_empty() {
            return Ok(());
        }
        for (off, _, fin) in self.finality_of(guard, topic, offsets)? {
            if !fin {
                return Err(BusError::OpenMail { topic: topic.to_string(), offset: off });
            }
        }
        Ok(())
    }

    /// The offsets `op` would remove from `topic` now (no removal).
    pub fn removal_preview(&self, topic: &str, op: RemovalOp, now_ms: i64) -> Result<Vec<Offset>> {
        if !self.meta.topic_exists(topic)? {
            return Err(BusError::UnknownTopic(topic.to_string()));
        }
        if op == RemovalOp::Retention(Retention::LatestPerCvKey) {
            return self.per_cv_key_stale(topic);
        }
        Ok(preview(&self.meta.record_index(topic)?, op, now_ms))
    }

    /// Guarded removal: check every offset, then delete exactly those.
    pub(crate) fn remove_checked(&self, guard: &dyn RemovalGuard, topic: &str, offsets: &[Offset]) -> Result<u64> {
        self.check_all_final(guard, topic, offsets)?;
        let n = self.meta.delete_records_at(topic, offsets)?;
        if n > 0 {
            self.maybe_reclaim(topic);
        }
        Ok(n)
    }

    /// For the periodic sweeper on a guarded topic: remove what the retention
    /// policy says, except open messages, which are kept ("never cut while
    /// open", V-3). Returns (removed, kept open). On an unguarded topic this is
    /// `sweep`. Finality unreadable: nothing removed, error.
    pub fn sweep_final_only(&self, topic: &str, now_ms: i64) -> Result<(u64, u64)> {
        let Some(guard) = self.guard_for(topic) else {
            return self.sweep(topic, now_ms).map(|n| (n, 0));
        };
        let retention = self
            .meta
            .topic_retention(topic)?
            .ok_or_else(|| BusError::UnknownTopic(topic.to_string()))?;
        let offsets = self.removal_preview(topic, RemovalOp::Retention(retention), now_ms)?;
        if offsets.is_empty() {
            return Ok((0, 0));
        }
        let rows = self.finality_of(guard.as_ref(), topic, &offsets)?;
        let finals: Vec<Offset> = rows.iter().filter(|r| r.2).map(|r| r.0).collect();
        let kept = (rows.len() - finals.len()) as u64;
        let n = self.meta.delete_records_at(topic, &finals)?;
        if n > 0 {
            self.maybe_reclaim(topic);
        }
        Ok((n, kept))
    }

    /// V-3 per-topic ceiling on a guarded topic (FCW-129). Runs whatever the
    /// topic's retention, forever included. Returns records removed. Finality
    /// unreadable: nothing is removed and the error is returned (never trimmed blind).
    pub fn apply_mail_ceiling(&self, topic: &str) -> Result<u64> {
        let Some(guard) = self.guard_for(topic) else {
            return Ok(0);
        };
        let c = self.mail_ceilings();
        if !c.enabled {
            return Ok(0);
        }
        let stats = self.meta.topic_stats(topic)?;
        let need_records = stats.records.saturating_sub(c.topic_max_records);
        let need_bytes = stats.live_bytes.saturating_sub(c.topic_max_bytes);
        if need_records == 0 && need_bytes == 0 {
            return Ok(0);
        }
        let index = self.meta.record_index(topic)?;
        let offsets: Vec<Offset> = index.iter().map(|r| r.offset).collect();
        let fin = self.finality_of(guard.as_ref(), topic, &offsets)?;
        // Index length, not payload length, is what the byte ceiling counts.
        let rows: Vec<(Offset, u64, bool)> =
            index.iter().zip(fin).map(|(r, (_, _, f))| (r.offset, r.len, f)).collect();
        let reason = if need_records > 0 { "V-3 topic record ceiling" } else { "V-3 topic byte ceiling" };
        self.remove_planned(guard.as_ref(), topic, &rows, need_records, need_bytes, reason)
    }

    fn remove_planned(
        &self,
        guard: &dyn RemovalGuard,
        topic: &str,
        rows: &[(Offset, u64, bool)],
        need_records: u64,
        need_bytes: u64,
        reason: &'static str,
    ) -> Result<u64> {
        let (finals, forced) = plan_ceiling(rows, need_records, need_bytes);
        let mut all = finals.clone();
        all.extend(&forced);
        let n = self.meta.delete_records_at(topic, &all)?;
        if n > 0 {
            eprintln!(
                "termlink-bus: T-3701 V-3 ceiling on {topic} ({reason}): removed {} final record(s){}",
                finals.len(),
                if forced.is_empty() {
                    String::new()
                } else {
                    format!(
                        " and, as a LAST RESORT, {} OPEN message record(s) at offsets {:?} (needs attention)",
                        forced.len(),
                        forced
                    )
                }
            );
            guard.on_ceiling_removal(&CeilingRemoval {
                topic: topic.to_string(),
                final_offsets: finals,
                forced_open_offsets: forced,
                reason,
            });
            self.maybe_reclaim(topic);
        }
        Ok(n)
    }

    /// V-3 per-hub ceiling over all guarded topics together (2 GiB): oldest
    /// final records across topics first, by timestamp; open ones only as a
    /// last resort. A topic whose finality cannot be read is skipped (nothing
    /// of it removed) and named in the returned errors.
    pub fn apply_mail_hub_ceiling(&self) -> Result<(u64, Vec<String>)> {
        let c = self.mail_ceilings();
        if !c.enabled {
            return Ok((0, Vec::new()));
        }
        let mut total = 0u64;
        let mut guarded = Vec::new();
        for t in self.meta.list_topics()? {
            if let Some(g) = self.guard_for(&t) {
                total = total.saturating_add(self.meta.topic_stats(&t)?.live_bytes);
                guarded.push((t, g));
            }
        }
        let need_bytes = total.saturating_sub(c.hub_max_bytes);
        if need_bytes == 0 {
            return Ok((0, Vec::new()));
        }
        // (ts, topic index, offset, len, final), oldest first across topics.
        let mut rows = Vec::new();
        let mut errors = Vec::new();
        for (i, (t, g)) in guarded.iter().enumerate() {
            let index = self.meta.record_index(t)?;
            let offsets: Vec<Offset> = index.iter().map(|r| r.offset).collect();
            match self.finality_of(g.as_ref(), t, &offsets) {
                Ok(fin) => {
                    for (r, (_, _, f)) in index.iter().zip(fin) {
                        rows.push((r.ts_ms, i, r.offset, r.len, f));
                    }
                }
                Err(e) => errors.push(format!("{t}: {e}")),
            }
        }
        rows.sort_by_key(|r| (r.0, r.1, r.2));
        // Plan over a synthetic global offset (position in `rows`), then map back.
        let flat: Vec<(Offset, u64, bool)> =
            rows.iter().enumerate().map(|(pos, r)| (pos as Offset, r.3, r.4)).collect();
        let (finals, forced) = plan_ceiling(&flat, 0, need_bytes);
        let forced_set: std::collections::HashSet<Offset> = forced.iter().copied().collect();
        let mut per_topic: Vec<Vec<(Offset, u64, bool)>> = vec![Vec::new(); guarded.len()];
        for pos in finals.iter().chain(forced.iter()) {
            let r = rows[*pos as usize];
            per_topic[r.1].push((r.2, r.3, !forced_set.contains(pos)));
        }
        let mut removed = 0u64;
        for (i, chosen) in per_topic.into_iter().enumerate() {
            if chosen.is_empty() {
                continue;
            }
            let (t, g) = &guarded[i];
            // Every chosen row is removed: need = all of them.
            let n = chosen.len() as u64;
            removed += self.remove_planned(g.as_ref(), t, &chosen, n, 0, "V-3 hub byte ceiling")?;
        }
        Ok((removed, errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(n: u64) -> Vec<IndexRow> {
        (0..n).map(|i| IndexRow { offset: i, ts_ms: i as i64 * DAY_MS, len: 10 }).collect()
    }

    #[test]
    fn preview_matches_each_policy() {
        let r = rows(5);
        assert_eq!(preview(&r, RemovalOp::Trim(None), 0), vec![0, 1, 2, 3, 4]);
        assert_eq!(preview(&r, RemovalOp::Trim(Some(2)), 0), vec![0, 1]);
        assert_eq!(preview(&r, RemovalOp::Retention(Retention::Messages(2)), 0), vec![0, 1, 2]);
        assert_eq!(preview(&r, RemovalOp::Retention(Retention::Latest), 0), vec![0, 1, 2, 3]);
        assert_eq!(preview(&r, RemovalOp::Retention(Retention::Forever), 0), Vec::<Offset>::new());
        // now = day 5, keep 2 days: ts < day 3 goes.
        assert_eq!(preview(&r, RemovalOp::Retention(Retention::Days(2)), 5 * DAY_MS), vec![0, 1, 2]);
    }

    #[test]
    fn ceiling_plan_takes_finals_first_and_open_only_as_last_resort() {
        // offsets 0..6, open at 0 and 3.
        let r: Vec<(Offset, u64, bool)> = (0..6).map(|i| (i, 10, i != 0 && i != 3)).collect();
        // Need 2: the two oldest FINAL ones (1, 2), never the older open 0.
        assert_eq!(plan_ceiling(&r, 2, 0), (vec![1, 2], vec![]));
        // Need 5: all four finals, then the oldest open (0) as a last resort.
        assert_eq!(plan_ceiling(&r, 5, 0), (vec![1, 2, 4, 5], vec![0]));
        // Bytes: need 25 bytes = 3 records of 10.
        assert_eq!(plan_ceiling(&r, 0, 25), (vec![1, 2, 4], vec![]));
        assert_eq!(plan_ceiling(&r, 0, 0), (vec![], vec![]));
    }
}
