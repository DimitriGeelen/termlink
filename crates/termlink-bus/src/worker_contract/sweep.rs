//! Shared worker contract, hub backend, slice S2 (T-3502): the hub's state sweep.
//!
//! The hub writes T4 (`lost`), T6 (`unknown` by deadline), T7a (`unknown` -> `running`
//! on a live binding) and T7c (`unknown` -> `lost` on the same end evidence as T4).
//! Every one is driven from STATE: the registration files the caller observed, and the
//! records here. Never from events, which a clean exit does not emit and a hub restart
//! loses (design §4).
//!
//! Registry epoch (AEF condition a): the hub bumps `wc_registry.epoch` at start and on
//! any sweep whose listing was incomplete. A sighting records `seen_epoch`. Absence
//! counts toward T4/T7c only when `seen_epoch` equals the sweep's epoch, so a restart or
//! an unreadable directory can only ever lead to `unknown` (through the deadlines).
//!
//! Every write goes through `hub_write`, which applies the same CAS check the fenced
//! `transition` path applies and appends a `writer = hub` history row.

use std::collections::HashSet;

use rusqlite::{OptionalExtension, Transaction, params};

use super::{FenceReason, GRACE_MS, RUNNING_SLACK_MS, WcResult, WcStore, append_history, fence};
use crate::Bus;

/// What one sweep saw. Built by the hub from the sessions dirs.
#[derive(Debug, Default, Clone)]
pub struct SweepObservation {
    /// Every sessions dir was listed without error. `false` means "I could not see":
    /// the epoch is bumped and no absence counts toward T4 in this sweep.
    pub listing_complete: bool,
    /// `(dispatch_id, attempt)` carried by a LIVE registration.
    pub live: HashSet<(String, i64)>,
}

/// One state write made by the sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepTransition {
    pub dispatch_id: String,
    pub attempt: i64,
    pub from: String,
    pub to: String,
    pub cause: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SweepReport {
    /// The registry epoch this sweep ran at.
    pub epoch: i64,
    pub transitions: Vec<SweepTransition>,
    /// `lost` attempts whose binding came back. They stay `lost` (AEF d); the
    /// operator is told here and by a refusal row in history.
    pub notices: Vec<String>,
}

/// An attempt the sweep may act on, with the bookkeeping it needs.
struct Candidate {
    dispatch_id: String,
    attempt: i64,
    state: String,
    binding_seen_at: Option<i64>,
    seen_epoch: Option<i64>,
    end_observed_at: Option<i64>,
    absent_sweeps: i64,
    deadline_dispatched: Option<i64>,
    deadline_running: Option<i64>,
}

impl Bus {
    /// Hub start: bump the registry epoch (T-3502, AEF condition a). Returns the new epoch.
    pub fn wc_begin_hub_epoch(&self) -> std::result::Result<i64, super::WcError> {
        self.wc.begin_hub_epoch()
    }

    /// One state sweep over the observed registrations (T-3502). Time is injected.
    pub fn wc_sweep(
        &self,
        obs: &SweepObservation,
        now_ms: i64,
    ) -> std::result::Result<SweepReport, super::WcError> {
        self.wc.sweep(obs, now_ms)
    }
}

impl WcStore {
    pub(crate) fn begin_hub_epoch(&self) -> WcResult<i64> {
        self.with_tx(|tx| bump_epoch(tx))
    }

    pub(crate) fn sweep(&self, obs: &SweepObservation, now_ms: i64) -> WcResult<SweepReport> {
        self.with_tx(|tx| sweep_tx(tx, obs, now_ms))
    }
}

fn read_epoch(tx: &Transaction) -> WcResult<i64> {
    Ok(tx.query_row("SELECT epoch FROM wc_registry WHERE k = 1", [], |r| r.get(0))?)
}

fn bump_epoch(tx: &Transaction) -> WcResult<i64> {
    tx.execute("UPDATE wc_registry SET epoch = epoch + 1 WHERE k = 1", [])?;
    read_epoch(tx)
}

fn sweep_tx(tx: &Transaction, obs: &SweepObservation, now: i64) -> WcResult<SweepReport> {
    let mut epoch = read_epoch(tx)?;
    if !obs.listing_complete {
        epoch = bump_epoch(tx)?;
    }
    let mut report = SweepReport { epoch, ..Default::default() };

    let candidates: Vec<Candidate> = {
        let mut st = tx.prepare(
            "SELECT dispatch_id, attempt, state, binding_seen_at, seen_epoch, end_observed_at,
                    absent_sweeps, deadline_dispatched, deadline_running
             FROM wc_attempts WHERE state IN ('dispatched', 'running', 'unknown', 'lost')
             ORDER BY dispatch_id, attempt",
        )?;
        let rows = st.query_map([], |r| {
            Ok(Candidate {
                dispatch_id: r.get(0)?,
                attempt: r.get(1)?,
                state: r.get(2)?,
                binding_seen_at: r.get(3)?,
                seen_epoch: r.get(4)?,
                end_observed_at: r.get(5)?,
                absent_sweeps: r.get(6)?,
                deadline_dispatched: r.get(7)?,
                deadline_running: r.get(8)?,
            })
        })?;
        rows.collect::<std::result::Result<_, _>>()?
    };

    for c in candidates {
        let live = obs.live.contains(&(c.dispatch_id.clone(), c.attempt));
        if c.state == "lost" {
            if live {
                notice_reappeared(tx, &c, epoch, now, &mut report)?;
            }
        } else if live {
            sighting(tx, &c, epoch, now, &mut report)?;
        } else {
            absence(tx, &c, epoch, now, &mut report)?;
        }
    }
    Ok(report)
}

/// The binding is live: record it, then T6 (alive past `deadline_running`) or T7a.
fn sighting(tx: &Transaction, c: &Candidate, epoch: i64, now: i64, report: &mut SweepReport) -> WcResult<()> {
    tx.execute(
        "UPDATE wc_attempts SET binding_seen_at = ?3, seen_epoch = ?4, end_observed_at = NULL, absent_sweeps = 0
         WHERE dispatch_id = ?1 AND attempt = ?2",
        params![c.dispatch_id, c.attempt, now, epoch],
    )?;
    match c.state.as_str() {
        // T7a: the worker is back. Re-arm the running deadline so the next sweep does not
        // undo this at once; a worker that stays past it is flagged again, not silently kept.
        "unknown" => hub_write(
            tx, c, "unknown", "running", now, "binding_reappeared", Some(now + RUNNING_SLACK_MS), report,
        ),
        // T6, live row: alive past its bound. Operator notice comes from the transition.
        "running" if c.deadline_running.is_some_and(|d| now > d) => {
            hub_write(tx, c, "running", "unknown", now, "deadline_running", None, report)
        }
        _ => Ok(()),
    }
}

/// The binding is not live: count the absence (only at the epoch that saw it), then
/// T4 / T7c once it is confirmed twice and G has passed, else the deadlines (T6).
fn absence(tx: &Transaction, c: &Candidate, epoch: i64, now: i64, report: &mut SweepReport) -> WcResult<()> {
    if c.binding_seen_at.is_some() {
        if c.seen_epoch == Some(epoch) {
            let absent = c.absent_sweeps + 1;
            let end = c.end_observed_at.unwrap_or(now);
            tx.execute(
                "UPDATE wc_attempts SET absent_sweeps = ?3, end_observed_at = ?4
                 WHERE dispatch_id = ?1 AND attempt = ?2",
                params![c.dispatch_id, c.attempt, absent, end],
            )?;
            if absent >= 2 && now - end >= GRACE_MS {
                // T4 from running/dispatched; T7c from unknown. Same evidence, same bound.
                let cause = if c.state == "unknown" { "end_observed_unknown" } else { "end_observed" };
                return hub_write(tx, c, &c.state, "lost", now, cause, None, report);
            }
        } else {
            // The epoch moved since the sighting (hub restart, or an unreadable dir):
            // nothing counts, and the count starts again at the next matching sighting.
            tx.execute(
                "UPDATE wc_attempts SET absent_sweeps = 0, end_observed_at = NULL
                 WHERE dispatch_id = ?1 AND attempt = ?2",
                params![c.dispatch_id, c.attempt],
            )?;
        }
    }
    // T6, absent rows: the deadline is the only bound when the registry cannot say more.
    match c.state.as_str() {
        "dispatched" if c.deadline_dispatched.is_some_and(|d| now > d) => {
            hub_write(tx, c, "dispatched", "unknown", now, "deadline_dispatched", None, report)
        }
        "running" if c.deadline_running.is_some_and(|d| now > d) => {
            hub_write(tx, c, "running", "unknown", now, "deadline_running", None, report)
        }
        _ => Ok(()),
    }
}

/// A `lost` attempt's binding is live again. Its binding was not seen since it was lost
/// means this is the first sighting since then: refuse `lost -> running` in history and
/// tell the operator once. T5 evidence can still upgrade it (AEF d).
fn notice_reappeared(tx: &Transaction, c: &Candidate, epoch: i64, now: i64, report: &mut SweepReport) -> WcResult<()> {
    let lost_ts: Option<i64> = tx.query_row(
        "SELECT MAX(ts) FROM wc_history WHERE dispatch_id = ?1 AND attempt = ?2
           AND to_state = 'lost' AND fence_reason IS NULL",
        params![c.dispatch_id, c.attempt],
        |r| r.get(0),
    )?;
    if lost_ts.is_some_and(|t| c.binding_seen_at.is_some_and(|s| s >= t)) {
        return Ok(());
    }
    tx.execute(
        "UPDATE wc_attempts SET binding_seen_at = ?3, seen_epoch = ?4 WHERE dispatch_id = ?1 AND attempt = ?2",
        params![c.dispatch_id, c.attempt, now, epoch],
    )?;
    append_history(
        tx, &c.dispatch_id, c.attempt, Some("lost"), "lost", "hub", now, None,
        Some("binding_reappeared"), Some(FenceReason::ForbiddenTransition.as_str()),
    )?;
    report.notices.push(format!(
        "{} attempt {}: binding is live again after lost; stays lost (lost -> running is forbidden), operator review",
        c.dispatch_id, c.attempt
    ));
    Ok(())
}

/// The one write the sweep makes. Same CAS discipline as `transition_tx`: the state
/// found must be `expected`, or the whole sweep is refused.
#[allow(clippy::too_many_arguments)]
fn hub_write(
    tx: &Transaction,
    c: &Candidate,
    expected: &str,
    to: &str,
    now: i64,
    cause: &str,
    rearm_running: Option<i64>,
    report: &mut SweepReport,
) -> WcResult<()> {
    let actual: Option<String> = tx
        .query_row(
            "SELECT state FROM wc_attempts WHERE dispatch_id = ?1 AND attempt = ?2",
            params![c.dispatch_id, c.attempt],
            |r| r.get(0),
        )
        .optional()?;
    if actual.as_deref() != Some(expected) {
        return fence(
            FenceReason::CasMismatch,
            format!("sweep expected {expected}, state is {}", actual.unwrap_or_default()),
        );
    }
    match rearm_running {
        Some(deadline) => tx.execute(
            "UPDATE wc_attempts SET state = ?3, deadline_running = ?4 WHERE dispatch_id = ?1 AND attempt = ?2",
            params![c.dispatch_id, c.attempt, to, deadline],
        )?,
        None => tx.execute(
            "UPDATE wc_attempts SET state = ?3 WHERE dispatch_id = ?1 AND attempt = ?2",
            params![c.dispatch_id, c.attempt, to],
        )?,
    };
    append_history(tx, &c.dispatch_id, c.attempt, Some(expected), to, "hub", now, None, Some(cause), None)?;
    report.transitions.push(SweepTransition {
        dispatch_id: c.dispatch_id.clone(),
        attempt: c.attempt,
        from: expected.to_string(),
        to: to.to_string(),
        cause: cause.to_string(),
    });
    Ok(())
}
