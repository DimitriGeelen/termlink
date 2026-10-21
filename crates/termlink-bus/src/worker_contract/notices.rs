//! Shared worker contract, hub backend: durable operator notices (T-3521).
//!
//! A notice is a durable record that a state transition or a refusal needs a human (or, for
//! `reconciliation_requested`, the reconciler). It is written in the SAME transaction as the
//! event that causes it, and never from a best-effort event or a log line.
//!
//! Mechanism: every history-backed kind is an `AFTER INSERT` trigger on `wc_history`. The history
//! row is the transition (or the refusal), so a writer cannot record one and forget the other,
//! and a writer in another file (the T7d resolve in `answer.rs`) needs no change to be covered.
//! `reconciliation_requested` has no history row (nothing changed state), so the sweep writes it
//! with [`append`] inside its own transaction.
//!
//! | kind                 | cause (row in `wc_history`)                                   | audience   |
//! |----------------------|---------------------------------------------------------------|------------|
//! | `done_conflict`      | refusal `fence_reason = done_conflict` (T8 with other evidence) | operator   |
//! | `late_evidence`      | refusal `forbidden_transition`, `cause = late_evidence` (T5)  | operator   |
//! | `unknown_by_deadline`| hub write to `unknown`, cause `deadline_dispatched/running` (T6b / T6) | operator |
//! | `binding_reappeared` | refusal on `lost`, `cause = binding_reappeared`               | operator   |
//! | `operator_resolved`  | operator write out of `unknown` (T7d)                         | operator   |
//! | `reconciliation_requested` | sweep: absence confirmed twice, G not yet passed          | reconciler |
//!
//! Rows written before this table existed have no notice (no backfill): the history keeps them.
//! Counts exclude the reconciler audience, so `operator_notices` is what a human has to read.

use rusqlite::{Connection, Transaction, params};
use serde_json::{Value, json};

use super::{WcResult, WcStore};
use crate::{Bus, Result};

pub const KIND_DONE_CONFLICT: &str = "done_conflict";
pub const KIND_LATE_EVIDENCE: &str = "late_evidence";
pub const KIND_UNKNOWN_BY_DEADLINE: &str = "unknown_by_deadline";
pub const KIND_BINDING_REAPPEARED: &str = "binding_reappeared";
pub const KIND_OPERATOR_RESOLVED: &str = "operator_resolved";
pub const KIND_RECONCILIATION_REQUESTED: &str = "reconciliation_requested";

/// `cause` the T5 refusal carries so the trigger can tell it from any other forbidden transition.
pub(super) const CAUSE_LATE_EVIDENCE: &str = "late_evidence";

pub(super) fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wc_notices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            dispatch_id TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            kind TEXT NOT NULL,
            audience TEXT NOT NULL DEFAULT 'operator',
            ts INTEGER NOT NULL,
            detail TEXT,
            history_id INTEGER
         );
         CREATE INDEX IF NOT EXISTS wc_notices_attempt ON wc_notices (dispatch_id, attempt, id);

         CREATE TRIGGER IF NOT EXISTS wc_notice_done_conflict AFTER INSERT ON wc_history
         WHEN NEW.fence_reason = 'done_conflict'
         BEGIN
            INSERT INTO wc_notices (dispatch_id, attempt, kind, ts, detail, history_id)
            VALUES (NEW.dispatch_id, NEW.attempt, 'done_conflict', NEW.ts,
                    'done attempt received a completion with different evidence', NEW.id);
         END;

         CREATE TRIGGER IF NOT EXISTS wc_notice_late_evidence AFTER INSERT ON wc_history
         WHEN NEW.fence_reason = 'forbidden_transition' AND NEW.cause = 'late_evidence'
         BEGIN
            INSERT INTO wc_notices (dispatch_id, attempt, kind, ts, detail, history_id)
            VALUES (NEW.dispatch_id, NEW.attempt, 'late_evidence', NEW.ts,
                    'lost attempt got a completion whose evidence postdates the lost entry', NEW.id);
         END;

         CREATE TRIGGER IF NOT EXISTS wc_notice_unknown_by_deadline AFTER INSERT ON wc_history
         WHEN NEW.writer = 'hub' AND NEW.to_state = 'unknown' AND NEW.fence_reason IS NULL
              AND NEW.cause IN ('deadline_dispatched', 'deadline_running')
         BEGIN
            INSERT INTO wc_notices (dispatch_id, attempt, kind, ts, detail, history_id)
            VALUES (NEW.dispatch_id, NEW.attempt, 'unknown_by_deadline', NEW.ts,
                    NEW.from_state || ' -> unknown (' || NEW.cause || ')', NEW.id);
         END;

         CREATE TRIGGER IF NOT EXISTS wc_notice_binding_reappeared AFTER INSERT ON wc_history
         WHEN NEW.cause = 'binding_reappeared' AND NEW.fence_reason IS NOT NULL
         BEGIN
            INSERT INTO wc_notices (dispatch_id, attempt, kind, ts, detail, history_id)
            VALUES (NEW.dispatch_id, NEW.attempt, 'binding_reappeared', NEW.ts,
                    'binding is live again after lost; stays lost', NEW.id);
         END;

         CREATE TRIGGER IF NOT EXISTS wc_notice_operator_resolved AFTER INSERT ON wc_history
         WHEN NEW.writer = 'operator' AND NEW.from_state = 'unknown' AND NEW.fence_reason IS NULL
         BEGIN
            INSERT INTO wc_notices (dispatch_id, attempt, kind, ts, detail, history_id)
            VALUES (NEW.dispatch_id, NEW.attempt, 'operator_resolved', NEW.ts, NEW.cause, NEW.id);
         END;",
    )?;
    Ok(())
}

/// Writes a notice that has no history row. Call inside the transaction that causes it.
pub(super) fn append(
    tx: &Transaction,
    dispatch_id: &str,
    attempt: i64,
    kind: &str,
    audience: &str,
    ts: i64,
    detail: &str,
) -> WcResult<()> {
    tx.execute(
        "INSERT INTO wc_notices (dispatch_id, attempt, kind, audience, ts, detail)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![dispatch_id, attempt, kind, audience, ts, detail],
    )?;
    Ok(())
}

/// The notices of one attempt, oldest first, as `dispatch.get` shows them.
pub(super) fn notices_json(conn: &Connection, dispatch_id: &str, attempt: i64) -> WcResult<Value> {
    let mut st = conn.prepare(
        "SELECT kind, audience, ts, detail FROM wc_notices
         WHERE dispatch_id = ?1 AND attempt = ?2 ORDER BY id",
    )?;
    let rows: Vec<Value> = st
        .query_map(params![dispatch_id, attempt], |r| {
            Ok(json!({
                "kind": r.get::<_, String>(0)?,
                "audience": r.get::<_, String>(1)?,
                "ts": r.get::<_, i64>(2)?,
                "detail": r.get::<_, Option<String>>(3)?,
            }))
        })?
        .collect::<std::result::Result<_, _>>()?;
    Ok(Value::Array(rows))
}

impl WcStore {
    /// Operator notices (not reconciler requests) on a dispatch, optionally one attempt.
    pub(crate) fn operator_notice_count(&self, dispatch_id: &str, attempt: Option<i64>) -> WcResult<i64> {
        crate::run_blocking(|| {
            let conn = self.conn.lock().expect("wc mutex poisoned");
            let n: i64 = conn.query_row(
                "SELECT COUNT(*) FROM wc_notices
                 WHERE dispatch_id = ?1 AND audience = 'operator' AND (?2 IS NULL OR attempt = ?2)",
                params![dispatch_id, attempt],
                |r| r.get(0),
            )?;
            Ok(n)
        })
    }

    pub(crate) fn notice_list(&self, dispatch_id: &str, attempt: i64) -> WcResult<Value> {
        crate::run_blocking(|| {
            let conn = self.conn.lock().expect("wc mutex poisoned");
            notices_json(&conn, dispatch_id, attempt)
        })
    }
}

impl Bus {
    /// How many operator notices a dispatch has raised (one attempt when `attempt` is given).
    /// Reconciler requests are not counted.
    pub fn wc_operator_notice_count(
        &self,
        dispatch_id: &str,
        attempt: Option<i64>,
    ) -> std::result::Result<i64, super::WcError> {
        self.wc.operator_notice_count(dispatch_id, attempt)
    }

    /// The notices of one attempt (`kind`, `audience`, `ts`, `detail`), oldest first.
    pub fn wc_notices(&self, dispatch_id: &str, attempt: i64) -> std::result::Result<Value, super::WcError> {
        self.wc.notice_list(dispatch_id, attempt)
    }
}
