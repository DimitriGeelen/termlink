//! Shared worker contract, hub backend, slice S3 (T-3503): the answer path (contract §5)
//! and the operator's `resolve` (T7d).
//!
//! A worker asks (`dispatch.ask`, authorised by the attempt nonce). The dispatcher answers
//! (`dispatch.answer`, authorised by the supervisor token). The worker acknowledges the
//! command (`dispatch.ack`, nonce): `stored -> received -> applied`.
//!
//! `undelivered` and `expired` are written ONLY by the sweep (`sweep_answers`), from state:
//! an answer still `stored` whose attempt ended is `undelivered`; one still `stored` past
//! its deadline (`ANSWER_TTL`, 10 min) is `expired` and the question is marked bounced so
//! the asker learns it got no answer. No event ever flips either.
//!
//! Every write is one `IMMEDIATE` transaction with the S1 fence (credential, attempt,
//! prior state); a refusal after authentication leaves a `fence_reason` history row.
//!
//! `resolve` (T7d) takes an `unknown` attempt to `done` (attested) or `lost` on an operator's
//! say-so. It has no RPC: it is reachable only through `resolve_at`, called by the hub-host
//! CLI, until a signed operator identity exists (T-3493).

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::{Value, json};

use super::{
    FenceReason, WcError, WcResult, WcStore, append_history, fence, random_token, sha256_hex,
};
use crate::{Bus, Result};

/// How long an answer may sit `stored` before the sweep bounces it (ms). 10 min: above the
/// measured hub-outage p95 (180 s), agreed with AEF.
pub const ANSWER_TTL_MS: i64 = 600_000;

#[derive(Debug, Default, Clone)]
pub struct AskRequest {
    pub dispatch_id: String,
    pub attempt: i64,
    pub nonce: String,
    /// Caller-chosen id (idempotent retry); minted when absent.
    pub question_id: Option<String>,
    pub body: String,
}

#[derive(Debug, Default, Clone)]
pub struct AnswerRequest {
    pub dispatch_id: String,
    pub attempt: i64,
    pub supervisor_token: Option<String>,
    /// Caller-chosen command id (idempotent retry); minted when absent.
    pub cmd_id: Option<String>,
    /// The question this answers, if any.
    pub question_id: Option<String>,
    pub body: String,
}

#[derive(Debug, Default, Clone)]
pub struct AckRequest {
    pub dispatch_id: String,
    pub attempt: i64,
    pub nonce: String,
    pub cmd_id: String,
    /// `received` or `applied`.
    pub state: String,
}

#[derive(Debug, Default, Clone)]
pub struct ResolveRequest {
    pub dispatch_id: String,
    pub attempt: i64,
    /// `done` or `lost`.
    pub as_state: String,
    pub reason: String,
}

/// Result of an answer-path write: the row as stored, and whether it was a replay.
#[derive(Debug, Clone)]
pub struct AnswerOutcome {
    pub record: Value,
    pub deduped: bool,
}

/// One answer the sweep closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerClosed {
    pub dispatch_id: String,
    pub attempt: i64,
    pub cmd_id: String,
    /// `undelivered` or `expired`.
    pub to: String,
    pub cause: String,
}

pub(super) fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wc_questions (
            dispatch_id TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            question_id TEXT NOT NULL,
            body TEXT NOT NULL,
            ts INTEGER NOT NULL,
            answered_cmd_id TEXT,
            bounced_at INTEGER,
            PRIMARY KEY (dispatch_id, attempt, question_id)
         );
         CREATE TABLE IF NOT EXISTS wc_answers (
            dispatch_id TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            cmd_id TEXT NOT NULL,
            question_id TEXT,
            body TEXT NOT NULL,
            deadline INTEGER NOT NULL,
            state TEXT NOT NULL,
            stored_at INTEGER NOT NULL,
            received_at INTEGER,
            applied_at INTEGER,
            closed_at INTEGER,
            PRIMARY KEY (dispatch_id, attempt, cmd_id)
         );",
    )?;
    Ok(())
}

struct AttemptRow {
    state: String,
    nonce_sha: String,
    token_sha: String,
    current: i64,
}

fn load_attempt(tx: &Transaction, id: &str, attempt: i64) -> WcResult<AttemptRow> {
    let row = tx
        .query_row(
            "SELECT state, nonce_sha256, supervisor_token_sha256,
                    (SELECT MAX(attempt) FROM wc_attempts WHERE dispatch_id = ?1)
             FROM wc_attempts WHERE dispatch_id = ?1 AND attempt = ?2",
            params![id, attempt],
            |r| Ok(AttemptRow { state: r.get(0)?, nonce_sha: r.get(1)?, token_sha: r.get(2)?, current: r.get(3)? }),
        )
        .optional()?;
    row.ok_or_else(|| WcError::NotFound(format!("{id} attempt {attempt}")))
}

/// A refusal after authentication: history row, then the fence error.
#[allow(clippy::too_many_arguments)]
fn refuse<T>(
    tx: &Transaction,
    id: &str,
    attempt: i64,
    state: &str,
    writer: &str,
    now: i64,
    reason: FenceReason,
    detail: String,
) -> WcResult<T> {
    append_history(tx, id, attempt, Some(state), state, writer, now, None, None, Some(reason.as_str()))?;
    fence(reason, detail)
}

fn question_json(conn: &Connection, id: &str, attempt: i64, qid: &str) -> WcResult<Value> {
    Ok(conn.query_row(
        "SELECT question_id, body, ts, answered_cmd_id, bounced_at FROM wc_questions
         WHERE dispatch_id = ?1 AND attempt = ?2 AND question_id = ?3",
        params![id, attempt, qid],
        |r| {
            Ok(json!({
                "dispatch_id": id, "attempt": attempt,
                "question_id": r.get::<_, String>(0)?, "body": r.get::<_, String>(1)?,
                "ts": r.get::<_, i64>(2)?, "answered_cmd_id": r.get::<_, Option<String>>(3)?,
                "bounced_at": r.get::<_, Option<i64>>(4)?,
            }))
        },
    )?)
}

fn answer_json(conn: &Connection, id: &str, attempt: i64, cmd: &str) -> WcResult<Value> {
    Ok(conn.query_row(
        "SELECT cmd_id, question_id, body, deadline, state, stored_at, received_at, applied_at, closed_at
         FROM wc_answers WHERE dispatch_id = ?1 AND attempt = ?2 AND cmd_id = ?3",
        params![id, attempt, cmd],
        |r| {
            Ok(json!({
                "dispatch_id": id, "attempt": attempt,
                "cmd_id": r.get::<_, String>(0)?, "question_id": r.get::<_, Option<String>>(1)?,
                "body": r.get::<_, String>(2)?, "deadline": r.get::<_, i64>(3)?,
                "state": r.get::<_, String>(4)?, "stored_at": r.get::<_, i64>(5)?,
                "received_at": r.get::<_, Option<i64>>(6)?, "applied_at": r.get::<_, Option<i64>>(7)?,
                "closed_at": r.get::<_, Option<i64>>(8)?,
            }))
        },
    )?)
}

/// Questions and answers of one attempt, for `dispatch.get`.
pub(super) fn qa_json(conn: &Connection, id: &str, attempt: i64) -> WcResult<(Value, Value)> {
    let qids: Vec<String> = conn
        .prepare("SELECT question_id FROM wc_questions WHERE dispatch_id=?1 AND attempt=?2 ORDER BY ts, question_id")?
        .query_map(params![id, attempt], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    let cmds: Vec<String> = conn
        .prepare("SELECT cmd_id FROM wc_answers WHERE dispatch_id=?1 AND attempt=?2 ORDER BY stored_at, cmd_id")?
        .query_map(params![id, attempt], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    let mut qs = Vec::new();
    for q in qids {
        qs.push(question_json(conn, id, attempt, &q)?);
    }
    let mut ans = Vec::new();
    for c in cmds {
        ans.push(answer_json(conn, id, attempt, &c)?);
    }
    Ok((Value::Array(qs), Value::Array(ans)))
}

fn live_state(state: &str) -> bool {
    matches!(state, "dispatched" | "running" | "unknown")
}

fn ask_tx(tx: &Transaction, req: &AskRequest, now: i64) -> WcResult<AnswerOutcome> {
    let id = &req.dispatch_id;
    if req.body.is_empty() {
        return Err(WcError::Invalid("dispatch.ask needs a non-empty body".into()));
    }
    let row = load_attempt(tx, id, req.attempt)?;
    if sha256_hex(&req.nonce) != row.nonce_sha {
        return fence(FenceReason::BadNonce, "nonce does not match this attempt");
    }
    let fail = |reason, detail: String| refuse(tx, id, row.current, &row.state, "worker", now, reason, detail);
    if req.attempt != row.current {
        return fail(FenceReason::StaleAttempt, format!("attempt {} is not current", req.attempt));
    }
    // A worker that is asking is running; `unknown` is allowed (alive but unconfirmed).
    if !matches!(row.state.as_str(), "running" | "unknown") {
        return fail(FenceReason::ForbiddenTransition, format!("ask while {}", row.state));
    }
    let qid = match &req.question_id {
        Some(q) if !q.is_empty() => q.clone(),
        _ => random_token("q")?,
    };
    let existing: Option<String> = tx
        .query_row(
            "SELECT body FROM wc_questions WHERE dispatch_id=?1 AND attempt=?2 AND question_id=?3",
            params![id, req.attempt, qid],
            |r| r.get(0),
        )
        .optional()?;
    match existing {
        Some(b) if b == req.body => {
            return Ok(AnswerOutcome { record: question_json(tx, id, req.attempt, &qid)?, deduped: true });
        }
        Some(_) => return fail(FenceReason::CasMismatch, format!("question {qid} exists with a different body")),
        None => {}
    }
    tx.execute(
        "INSERT INTO wc_questions (dispatch_id, attempt, question_id, body, ts) VALUES (?1,?2,?3,?4,?5)",
        params![id, req.attempt, qid, req.body, now],
    )?;
    append_history(tx, id, req.attempt, Some(&row.state), &row.state, "worker", now, None, Some("question_asked"), None)?;
    Ok(AnswerOutcome { record: question_json(tx, id, req.attempt, &qid)?, deduped: false })
}

fn answer_tx(tx: &Transaction, req: &AnswerRequest, now: i64) -> WcResult<AnswerOutcome> {
    let id = &req.dispatch_id;
    if req.body.is_empty() {
        return Err(WcError::Invalid("dispatch.answer needs a non-empty body".into()));
    }
    let row = load_attempt(tx, id, req.attempt)?;
    match &req.supervisor_token {
        Some(t) if sha256_hex(t) == row.token_sha => {}
        _ => return fence(FenceReason::BadCredential, "missing or wrong supervisor credential"),
    }
    let fail = |reason, detail: String| refuse(tx, id, row.current, &row.state, "dispatcher", now, reason, detail);
    if req.attempt != row.current {
        return fail(FenceReason::StaleAttempt, format!("attempt {} is not current", req.attempt));
    }
    if !live_state(&row.state) {
        return fail(FenceReason::ForbiddenTransition, format!("answer while {}", row.state));
    }
    if let Some(q) = &req.question_id {
        let known: bool = tx
            .query_row(
                "SELECT 1 FROM wc_questions WHERE dispatch_id=?1 AND attempt=?2 AND question_id=?3",
                params![id, req.attempt, q],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);
        if !known {
            return fail(FenceReason::CasMismatch, format!("no question {q} on this attempt"));
        }
    }
    let cmd = match &req.cmd_id {
        Some(c) if !c.is_empty() => c.clone(),
        _ => random_token("c")?,
    };
    let existing: Option<String> = tx
        .query_row(
            "SELECT body FROM wc_answers WHERE dispatch_id=?1 AND attempt=?2 AND cmd_id=?3",
            params![id, req.attempt, cmd],
            |r| r.get(0),
        )
        .optional()?;
    match existing {
        Some(b) if b == req.body => {
            return Ok(AnswerOutcome { record: answer_json(tx, id, req.attempt, &cmd)?, deduped: true });
        }
        Some(_) => return fail(FenceReason::CasMismatch, format!("command {cmd} exists with a different body")),
        None => {}
    }
    tx.execute(
        "INSERT INTO wc_answers (dispatch_id, attempt, cmd_id, question_id, body, deadline, state, stored_at)
         VALUES (?1,?2,?3,?4,?5,?6,'stored',?7)",
        params![id, req.attempt, cmd, req.question_id, req.body, now + ANSWER_TTL_MS, now],
    )?;
    if let Some(q) = &req.question_id {
        tx.execute(
            "UPDATE wc_questions SET answered_cmd_id=?4 WHERE dispatch_id=?1 AND attempt=?2 AND question_id=?3",
            params![id, req.attempt, q, cmd],
        )?;
    }
    append_history(tx, id, req.attempt, Some(&row.state), &row.state, "dispatcher", now, None, Some("answer_stored"), None)?;
    Ok(AnswerOutcome { record: answer_json(tx, id, req.attempt, &cmd)?, deduped: false })
}

fn ack_tx(tx: &Transaction, req: &AckRequest, now: i64) -> WcResult<AnswerOutcome> {
    let id = &req.dispatch_id;
    if !matches!(req.state.as_str(), "received" | "applied") {
        return Err(WcError::Invalid("dispatch.ack state must be received or applied".into()));
    }
    let row = load_attempt(tx, id, req.attempt)?;
    if sha256_hex(&req.nonce) != row.nonce_sha {
        return fence(FenceReason::BadNonce, "nonce does not match this attempt");
    }
    let fail = |reason, detail: String| refuse(tx, id, row.current, &row.state, "worker", now, reason, detail);
    if req.attempt != row.current {
        return fail(FenceReason::StaleAttempt, format!("attempt {} is not current", req.attempt));
    }
    let found: Option<(String, i64)> = tx
        .query_row(
            "SELECT state, deadline FROM wc_answers WHERE dispatch_id=?1 AND attempt=?2 AND cmd_id=?3",
            params![id, req.attempt, req.cmd_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((cur, deadline)) = found else {
        return Err(WcError::NotFound(format!("{id} attempt {} command {}", req.attempt, req.cmd_id)));
    };
    let replay = |tx: &Transaction| -> WcResult<AnswerOutcome> {
        Ok(AnswerOutcome { record: answer_json(tx, id, req.attempt, &req.cmd_id)?, deduped: true })
    };
    match (cur.as_str(), req.state.as_str()) {
        // Replays and late duplicates: a response, never a regression.
        ("received", "received") | ("applied", "received") | ("applied", "applied") => return replay(tx),
        ("stored", "received") => {
            // The deadline is the sweep's to enforce; an ack that arrives after it is
            // refused here so a late ack cannot resurrect a bounced answer.
            if now > deadline {
                return fail(FenceReason::CasMismatch, "answer deadline passed; the sweep will expire it".into());
            }
            tx.execute(
                "UPDATE wc_answers SET state='received', received_at=?4 WHERE dispatch_id=?1 AND attempt=?2 AND cmd_id=?3",
                params![id, req.attempt, req.cmd_id, now],
            )?;
        }
        ("received", "applied") => {
            tx.execute(
                "UPDATE wc_answers SET state='applied', applied_at=?4 WHERE dispatch_id=?1 AND attempt=?2 AND cmd_id=?3",
                params![id, req.attempt, req.cmd_id, now],
            )?;
        }
        (other, want) => {
            return fail(FenceReason::CasMismatch, format!("command is {other}, cannot ack {want}"));
        }
    }
    let cause = format!("answer_{}", req.state);
    append_history(tx, id, req.attempt, Some(&row.state), &row.state, "worker", now, None, Some(&cause), None)?;
    Ok(AnswerOutcome { record: answer_json(tx, id, req.attempt, &req.cmd_id)?, deduped: false })
}

/// T-3520: the attempt just reached a terminal state (`done`, or `lost` by resolve): every
/// stored or received but unapplied answer of it can no longer be delivered. Read from
/// state, never from events, and written in the same transaction as the transition, so
/// nothing waits on a sweep. Same closure the sweep applies (question bounced, history row).
pub(super) fn close_ended_answers_tx(tx: &Transaction, id: &str, attempt: i64, state: &str, now: i64) -> WcResult<()> {
    let open: Vec<(String, Option<String>)> = {
        let mut st = tx.prepare(
            "SELECT cmd_id, question_id FROM wc_answers
             WHERE dispatch_id = ?1 AND attempt = ?2 AND state IN ('stored', 'received')
             ORDER BY stored_at, cmd_id",
        )?;
        let rows = st.query_map(params![id, attempt], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect::<std::result::Result<_, _>>()?
    };
    for (cmd, question) in open {
        tx.execute(
            "UPDATE wc_answers SET state='undelivered', closed_at=?4 WHERE dispatch_id=?1 AND attempt=?2 AND cmd_id=?3",
            params![id, attempt, cmd, now],
        )?;
        if let Some(q) = question {
            tx.execute(
                "UPDATE wc_questions SET bounced_at=?4 WHERE dispatch_id=?1 AND attempt=?2 AND question_id=?3",
                params![id, attempt, q, now],
            )?;
        }
        append_history(tx, id, attempt, Some(state), state, "hub", now, None, Some("answer_undelivered:attempt_ended"), None)?;
    }
    Ok(())
}

/// The sweep step: close answers that can no longer be delivered. State only, no events.
fn sweep_answers_tx(tx: &Transaction, now: i64) -> WcResult<Vec<AnswerClosed>> {
    struct Open {
        id: String,
        attempt: i64,
        cmd: String,
        question: Option<String>,
        deadline: i64,
        attempt_state: String,
    }
    let open: Vec<Open> = {
        let mut st = tx.prepare(
            "SELECT a.dispatch_id, a.attempt, a.cmd_id, a.question_id, a.deadline, t.state
             FROM wc_answers a JOIN wc_attempts t
               ON t.dispatch_id = a.dispatch_id AND t.attempt = a.attempt
             WHERE a.state = 'stored' ORDER BY a.dispatch_id, a.attempt, a.stored_at, a.cmd_id",
        )?;
        let rows = st.query_map([], |r| {
            Ok(Open {
                id: r.get(0)?,
                attempt: r.get(1)?,
                cmd: r.get(2)?,
                question: r.get(3)?,
                deadline: r.get(4)?,
                attempt_state: r.get(5)?,
            })
        })?;
        rows.collect::<std::result::Result<_, _>>()?
    };
    let mut closed = Vec::new();
    for o in open {
        // The attempt ending beats the clock: nobody is left to receive it.
        let (to, cause) = if matches!(o.attempt_state.as_str(), "done" | "lost") {
            ("undelivered", "attempt_ended")
        } else if now >= o.deadline {
            ("expired", "answer_ttl")
        } else {
            continue;
        };
        tx.execute(
            "UPDATE wc_answers SET state=?4, closed_at=?5 WHERE dispatch_id=?1 AND attempt=?2 AND cmd_id=?3",
            params![o.id, o.attempt, o.cmd, to, now],
        )?;
        if let Some(q) = &o.question {
            // The bounce: the asker's question is marked so the dispatcher side sees it
            // went unanswered in time.
            tx.execute(
                "UPDATE wc_questions SET bounced_at=?4 WHERE dispatch_id=?1 AND attempt=?2 AND question_id=?3",
                params![o.id, o.attempt, q, now],
            )?;
        }
        let cause_tag = format!("answer_{to}:{cause}");
        append_history(
            tx, &o.id, o.attempt, Some(&o.attempt_state), &o.attempt_state, "hub", now, None, Some(&cause_tag), None,
        )?;
        closed.push(AnswerClosed {
            dispatch_id: o.id,
            attempt: o.attempt,
            cmd_id: o.cmd,
            to: to.to_string(),
            cause: cause.to_string(),
        });
    }
    Ok(closed)
}

/// T7d. The reason is mandatory and lands in the history entry.
fn resolve_tx(tx: &Transaction, req: &ResolveRequest, now: i64) -> WcResult<Value> {
    let id = &req.dispatch_id;
    let reason = req.reason.trim();
    if reason.is_empty() {
        return Err(WcError::Invalid("resolve needs a non-empty reason".into()));
    }
    if !matches!(req.as_state.as_str(), "done" | "lost") {
        return Err(WcError::Invalid("resolve --as must be done or lost".into()));
    }
    let row = load_attempt(tx, id, req.attempt)?;
    if row.state != "unknown" {
        return refuse(
            tx, id, req.attempt, &row.state, "operator", now, FenceReason::CasMismatch,
            format!("attempt is {}, resolve only applies to unknown", row.state),
        );
    }
    let cause = format!("operator_resolve: {reason}");
    if req.as_state == "done" {
        // Attested, not evidenced: no outcome is invented, and the record says who vouched.
        let digest = format!("operator:{}", sha256_hex(reason));
        tx.execute(
            "UPDATE wc_attempts SET state='done', attested=1, attested_by='operator', evidence_digest=?3,
                result_kind='none' WHERE dispatch_id=?1 AND attempt=?2",
            params![id, req.attempt, digest],
        )?;
        append_history(tx, id, req.attempt, Some("unknown"), "done", "operator", now, Some(&digest), Some(&cause), None)?;
        close_ended_answers_tx(tx, id, req.attempt, "done", now)?;
    } else {
        tx.execute(
            "UPDATE wc_attempts SET state='lost' WHERE dispatch_id=?1 AND attempt=?2",
            params![id, req.attempt],
        )?;
        append_history(tx, id, req.attempt, Some("unknown"), "lost", "operator", now, None, Some(&cause), None)?;
        close_ended_answers_tx(tx, id, req.attempt, "lost", now)?;
    }
    super::attempt_record(tx, id, req.attempt)
}

impl WcStore {
    pub(crate) fn ask(&self, req: &AskRequest, now: i64) -> WcResult<AnswerOutcome> {
        self.with_tx(|tx| ask_tx(tx, req, now))
    }
    pub(crate) fn answer(&self, req: &AnswerRequest, now: i64) -> WcResult<AnswerOutcome> {
        self.with_tx(|tx| answer_tx(tx, req, now))
    }
    pub(crate) fn ack(&self, req: &AckRequest, now: i64) -> WcResult<AnswerOutcome> {
        self.with_tx(|tx| ack_tx(tx, req, now))
    }
    pub(crate) fn sweep_answers(&self, now: i64) -> WcResult<Vec<AnswerClosed>> {
        self.with_tx(|tx| sweep_answers_tx(tx, now))
    }
    pub(crate) fn resolve(&self, req: &ResolveRequest, now: i64) -> WcResult<Value> {
        self.with_tx(|tx| resolve_tx(tx, req, now))
    }
}

impl Bus {
    /// T-3503: `dispatch.ask` (worker, nonce).
    pub fn wc_ask(&self, req: &AskRequest, now_ms: i64) -> std::result::Result<AnswerOutcome, WcError> {
        self.wc.ask(req, now_ms)
    }
    /// T-3503: `dispatch.answer` (dispatcher, supervisor token).
    pub fn wc_answer(&self, req: &AnswerRequest, now_ms: i64) -> std::result::Result<AnswerOutcome, WcError> {
        self.wc.answer(req, now_ms)
    }
    /// T-3503: `dispatch.ack {cmd_id, received|applied}` (worker, nonce).
    pub fn wc_ack(&self, req: &AckRequest, now_ms: i64) -> std::result::Result<AnswerOutcome, WcError> {
        self.wc.ack(req, now_ms)
    }
    /// T-3503: the answer step of the sweep (`undelivered`, `expired`). Time is injected.
    pub fn wc_sweep_answers(&self, now_ms: i64) -> std::result::Result<Vec<AnswerClosed>, WcError> {
        self.wc.sweep_answers(now_ms)
    }
}

/// T7d, hub-host only. Opens the dispatch store of the bus under `bus_root` directly (its
/// own connection, WAL), so the CLI does not need the hub's RPC surface and no network
/// caller can reach it. Does not create a store that is not there.
pub fn resolve_at(bus_root: &Path, req: &ResolveRequest, now_ms: i64) -> std::result::Result<Value, WcError> {
    let db = bus_root.join("meta.db");
    if !db.exists() {
        return Err(WcError::NotFound(format!("no dispatch store at {}", db.display())));
    }
    WcStore::open(&db)?.resolve(req, now_ms)
}
