//! Shared worker contract, hub backend, slice S1 (T-3501, T-3484).
//!
//! Dispatch records with CAS and fencing: `wc_dispatches`, `wc_attempts`, `wc_history`
//! in `meta.db`. Spec: AEF `docs/specs/shared-worker-contract-transitions.md` (wc/0.3);
//! design: `docs/reports/T-3484-termlink-build-design.md`.
//!
//! The store keeps its OWN connection to `meta.db` (WAL, synchronous=FULL, same as
//! `Meta`) so it does not touch `meta.rs`. Every write is one `IMMEDIATE` transaction:
//! credentials, nonce, `supervisor_gen` and the expected prior state are checked in the
//! same step as the write (I3, §4.2).
//!
//! Only the writes reachable through `dispatch.transition` live here: T1 (allocate),
//! T2, T3, T3b, T-RECLAIM, T5, T7b and the T8 response. T4/T6/T6b/T7a/T7c belong to the
//! sweep (S2) and T7d to the operator CLI (S3, `answer::resolve_at`); asking for them here is
//! `forbidden_transition`.
//!
//! The nonce and the supervisor token are returned once at allocation and stored only
//! as sha256, so a database dump does not hand out write credentials.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{BusError, Result};

/// Contract version stamped on every record.
pub const CONTRACT_VERSION: &str = "wc/0.3";
/// Default spawn timeout when the caller gives none (ms).
const DEFAULT_SPAWN_TIMEOUT_MS: i64 = 60_000;
/// `G`: grace for local evidence to become readable (ms); 4 x the 30 s sweep.
const GRACE_MS: i64 = 120_000;
/// Default worker timeout when the caller gives none (ms).
const DEFAULT_WORKER_TIMEOUT_MS: i64 = 3_600_000;
/// `deadline_running` = worker timeout + 10 min.
const RUNNING_SLACK_MS: i64 = 600_000;

/// Why a write was refused, mirrored into `FENCE_REJECTED.data.reason`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceReason {
    StaleAttempt,
    BadNonce,
    BadCredential,
    StaleGen,
    CasMismatch,
    ForbiddenTransition,
    DoneConflict,
}

impl FenceReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaleAttempt => "stale_attempt",
            Self::BadNonce => "bad_nonce",
            Self::BadCredential => "bad_credential",
            Self::StaleGen => "stale_gen",
            Self::CasMismatch => "cas_mismatch",
            Self::ForbiddenTransition => "forbidden_transition",
            Self::DoneConflict => "done_conflict",
        }
    }
}

/// Failure of a worker-contract call.
#[derive(Debug)]
pub enum WcError {
    /// A caller-chosen `dispatch_id` is already in use (never substituted).
    IdTaken(String),
    /// `dispatch_id` is not a well-formed `D-<ulid>`.
    BadId(String),
    /// No such dispatch (or attempt).
    NotFound(String),
    /// Write refused by the fence.
    Fence { reason: FenceReason, detail: String },
    /// Malformed request (missing or invalid field).
    Invalid(String),
    Bus(BusError),
}

impl From<BusError> for WcError {
    fn from(e: BusError) -> Self {
        WcError::Bus(e)
    }
}

impl From<rusqlite::Error> for WcError {
    fn from(e: rusqlite::Error) -> Self {
        WcError::Bus(BusError::from(e))
    }
}

type WcResult<T> = std::result::Result<T, WcError>;

fn fence<T>(reason: FenceReason, detail: impl Into<String>) -> WcResult<T> {
    Err(WcError::Fence { reason, detail: detail.into() })
}

/// `allocate` request (T1).
#[derive(Debug, Default, Clone)]
pub struct AllocateRequest {
    /// Create a NEW dispatch with this id (`id_taken` if used). `None`: backend allocates.
    pub dispatch_id: Option<String>,
    /// Mint the next attempt of this EXISTING dispatch (only when the last is done/lost).
    pub attempt_of: Option<String>,
    pub dispatcher_project_agent: Option<String>,
    pub dispatcher_session: Option<String>,
    pub claude_session_id: Option<String>,
    pub task_id: Option<String>,
    pub retry_of_dispatch_id: Option<String>,
    pub worker_kind: Option<String>,
    pub session_name: Option<String>,
    pub spawn_timeout_ms: Option<i64>,
}

/// What `allocate` returns. `nonce` and `supervisor_token` are shown once.
#[derive(Debug, Clone)]
pub struct Allocation {
    pub dispatch_id: String,
    pub attempt: i64,
    pub nonce: String,
    pub supervisor_token: String,
    pub deadline_dispatched: i64,
}

/// `transition` request. `to` is `running` or `done`.
#[derive(Debug, Default, Clone)]
pub struct TransitionRequest {
    pub dispatch_id: String,
    pub attempt: i64,
    pub nonce: String,
    /// Required for every lifecycle write this RPC performs.
    pub supervisor_token: Option<String>,
    pub expected_prior: String,
    pub to: String,
    pub supervisor_gen: i64,
    pub evidence_digest: Option<String>,
    pub exit_code: Option<i64>,
    /// The run.sh refusal path (e.g. `exit 3`).
    pub refused: bool,
    pub result_kind: Option<String>,
    pub result_value: Option<String>,
    /// When the supervisor persisted the signed completion (ms); the T5 bound.
    pub persisted_at: Option<i64>,
    pub handback: Option<Handback>,
    pub worker_timeout_ms: Option<i64>,
    /// `deadline_stop` / `revoked` / `operator_stop`, recorded on the history entry.
    pub cause: Option<String>,
}

/// The causes a backend may stop an attempt for (§3.3).
pub const STOP_CAUSES: [&str; 3] = ["deadline_stop", "revoked", "operator_stop"];

/// T-3523: a backend-initiated stop (§3.3, CC-25). The supervisor's credential fences it.
#[derive(Debug, Clone, Default)]
pub struct StopRequest {
    pub dispatch_id: String,
    pub attempt: i64,
    pub supervisor_token: Option<String>,
    /// One of `STOP_CAUSES`.
    pub cause: String,
}

#[derive(Debug, Clone, Default)]
pub struct Handback {
    pub attempt: Option<i64>,
    pub nonce: Option<String>,
    pub status: Option<String>,
    pub summary: Option<String>,
    /// T-3599: the dispatch the worker says it handed back (§3.3 rule 1, ring20 N4).
    pub commission: Option<String>,
}

/// Result of a transition: the attempt as stored and whether it was a T8 replay.
#[derive(Debug, Clone)]
pub struct TransitionOutcome {
    pub record: Value,
    pub deduped: bool,
}

pub(crate) struct WcStore {
    conn: Mutex<Connection>,
}

impl WcStore {
    /// Open the store on `meta.db` and create the tables if missing (idempotent).
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
        // T-3531: the same wait as the Meta writer. This connection holds IMMEDIATE
        // locks across its commit fsync, so the Meta writer waits on it.
        conn.execute_batch(&format!(
            "PRAGMA synchronous=FULL; PRAGMA busy_timeout={};",
            crate::meta::WRITE_LOCK_WAIT_MS
        ))?;
        init_schema(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn with_tx<R>(&self, f: impl FnOnce(&Transaction) -> WcResult<R>) -> WcResult<R> {
        crate::run_blocking(|| {
            let mut conn = self.conn.lock().expect("wc mutex poisoned");
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            // A fence refusal may have appended its history row; commit on every
            // outcome that is a deliberate answer, roll back only on real errors.
            match f(&tx) {
                Ok(v) => {
                    tx.commit()?;
                    Ok(v)
                }
                Err(e @ WcError::Fence { .. }) => {
                    tx.commit()?;
                    Err(e)
                }
                Err(e) => Err(e),
            }
        })
    }

    pub(crate) fn allocate(&self, req: &AllocateRequest, now_ms: i64) -> WcResult<Allocation> {
        if req.dispatch_id.is_some() && req.attempt_of.is_some() {
            return Err(WcError::Invalid(
                "give dispatch_id (new dispatch) or attempt_of (next attempt), not both".into(),
            ));
        }
        if let Some(id) = &req.dispatch_id
            && !is_valid_dispatch_id(id) {
                return Err(WcError::BadId(id.clone()));
            }
        if let Some(id) = &req.attempt_of
            && !is_valid_dispatch_id(id) {
                return Err(WcError::BadId(id.clone()));
            }
        self.with_tx(|tx| {
            let spawn = req.spawn_timeout_ms.unwrap_or(DEFAULT_SPAWN_TIMEOUT_MS).max(0);
            let deadline_dispatched = now_ms + spawn + GRACE_MS;
            let (dispatch_id, attempt) = if let Some(id) = &req.attempt_of {
                let last: Option<(i64, String)> = tx
                    .query_row(
                        "SELECT attempt, state FROM wc_attempts WHERE dispatch_id=?1
                         ORDER BY attempt DESC LIMIT 1",
                        params![id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()?;
                let Some((n, state)) = last else {
                    return Err(WcError::NotFound(id.clone()));
                };
                if state != "done" && state != "lost" {
                    return fence(
                        FenceReason::CasMismatch,
                        format!("attempt {n} of {id} is {state}, not terminal"),
                    );
                }
                (id.clone(), n + 1)
            } else {
                let id = match &req.dispatch_id {
                    Some(id) => id.clone(),
                    None => new_dispatch_id(now_ms)?,
                };
                let taken: bool = tx
                    .query_row(
                        "SELECT 1 FROM wc_dispatches WHERE dispatch_id=?1",
                        params![id],
                        |_| Ok(true),
                    )
                    .optional()?
                    .unwrap_or(false);
                if taken {
                    return Err(WcError::IdTaken(id));
                }
                tx.execute(
                    "INSERT INTO wc_dispatches (dispatch_id, contract_version, backend, task_id,
                        retry_of_dispatch_id, dispatcher_project_agent, dispatcher_session,
                        claude_session_id, created_at)
                     VALUES (?1, ?2, 'hub', ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        id,
                        CONTRACT_VERSION,
                        req.task_id,
                        req.retry_of_dispatch_id,
                        req.dispatcher_project_agent,
                        req.dispatcher_session,
                        req.claude_session_id,
                        now_ms
                    ],
                )?;
                (id, 1)
            };
            let nonce = random_token("n")?;
            let token = random_token("s")?;
            tx.execute(
                "INSERT INTO wc_attempts (dispatch_id, attempt, nonce_sha256, supervisor_token_sha256,
                    state, worker_kind, session_name, supervisor_gen, deadline_dispatched, created_at)
                 VALUES (?1, ?2, ?3, ?4, 'dispatched', ?5, ?6, 1, ?7, ?8)",
                params![
                    dispatch_id,
                    attempt,
                    sha256_hex(&nonce),
                    sha256_hex(&token),
                    req.worker_kind,
                    req.session_name,
                    deadline_dispatched,
                    now_ms
                ],
            )?;
            append_history(tx, &dispatch_id, attempt, None, "dispatched", "hub", now_ms, None, None, None)?;
            Ok(Allocation { dispatch_id, attempt, nonce, supervisor_token: token, deadline_dispatched })
        })
    }

    pub(crate) fn get(&self, dispatch_id: &str) -> WcResult<Value> {
        crate::run_blocking(|| {
            let conn = self.conn.lock().expect("wc mutex poisoned");
            let d = conn
                .query_row(
                    "SELECT contract_version, backend, task_id, retry_of_dispatch_id,
                            dispatcher_project_agent, dispatcher_session, claude_session_id, created_at
                     FROM wc_dispatches WHERE dispatch_id=?1",
                    params![dispatch_id],
                    |r| {
                        Ok(json!({
                            "contract_version": r.get::<_, String>(0)?,
                            "dispatch_id": dispatch_id,
                            "backend": r.get::<_, String>(1)?,
                            "task_id": r.get::<_, Option<String>>(2)?,
                            "retry_of_dispatch_id": r.get::<_, Option<String>>(3)?,
                            "dispatcher": {
                                "project_agent": r.get::<_, Option<String>>(4)?,
                                "session": r.get::<_, Option<String>>(5)?,
                                "claude_session_id": r.get::<_, Option<String>>(6)?,
                            },
                            "created_at": r.get::<_, i64>(7)?,
                        }))
                    },
                )
                .optional()?;
            let Some(mut d) = d else {
                return Err(WcError::NotFound(dispatch_id.to_string()));
            };
            let mut st = conn.prepare(&format!(
                "SELECT {ATTEMPT_COLS} FROM wc_attempts WHERE dispatch_id=?1 ORDER BY attempt"
            ))?;
            let mut attempts = Vec::new();
            let nums: Vec<Value> = st
                .query_map(params![dispatch_id], attempt_json)?
                .collect::<std::result::Result<_, _>>()?;
            for mut a in nums {
                let n = a["attempt"].as_i64().unwrap_or(0);
                a["history"] = history_json(&conn, dispatch_id, n)?;
                let (qs, ans) = answer::qa_json(&conn, dispatch_id, n)?;
                a["questions"] = qs;
                a["answers"] = ans;
                a["notices"] = notices::notices_json(&conn, dispatch_id, n)?;
                attempts.push(a);
            }
            d["attempts"] = Value::Array(attempts);
            let (bounced, replays) = answer::bounce_json(&conn, dispatch_id)?;
            d["bounced_to_dispatcher"] = bounced;
            d["retry_ladder_entries"] = json!(replays);
            Ok(d)
        })
    }

    pub(crate) fn transition(&self, req: &TransitionRequest, now_ms: i64) -> WcResult<TransitionOutcome> {
        self.with_tx(|tx| transition_tx(tx, req, now_ms))
    }

    pub(crate) fn stop(&self, req: &StopRequest, now_ms: i64) -> WcResult<Value> {
        self.with_tx(|tx| stop_tx(tx, req, now_ms))
    }
}

const ATTEMPT_COLS: &str = "attempt, state, outcome, worker_kind, session_name, supervisor_gen, \
    exit_code, result_kind, result_value, persisted_at, evidence_digest, attested, attested_by, \
    deadline_dispatched, deadline_running, handback_status, created_at, stop_cause";

fn attempt_json(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    Ok(json!({
        "attempt": r.get::<_, i64>(0)?,
        "state": r.get::<_, String>(1)?,
        "outcome": r.get::<_, Option<String>>(2)?,
        "worker_kind": r.get::<_, Option<String>>(3)?,
        "session_binding": { "session_name": r.get::<_, Option<String>>(4)? },
        "supervisor_gen": r.get::<_, i64>(5)?,
        "exit_code": r.get::<_, Option<i64>>(6)?,
        "result": {
            "kind": r.get::<_, Option<String>>(7)?,
            "value": r.get::<_, Option<String>>(8)?,
            "persisted_at": r.get::<_, Option<i64>>(9)?,
        },
        "evidence_digest": r.get::<_, Option<String>>(10)?,
        "attested": r.get::<_, i64>(11)? != 0,
        "attested_by": r.get::<_, Option<String>>(12)?,
        "deadline_dispatched": r.get::<_, Option<i64>>(13)?,
        "deadline_running": r.get::<_, Option<i64>>(14)?,
        "handback_status": r.get::<_, Option<String>>(15)?,
        "created_at": r.get::<_, i64>(16)?,
        "stop_cause": r.get::<_, Option<String>>(17)?,
    }))
}

fn history_json(conn: &Connection, dispatch_id: &str, attempt: i64) -> WcResult<Value> {
    let mut st = conn.prepare(
        "SELECT from_state, to_state, writer, ts, evidence_digest, cause, fence_reason
         FROM wc_history WHERE dispatch_id=?1 AND attempt=?2 ORDER BY id",
    )?;
    let rows: Vec<Value> = st
        .query_map(params![dispatch_id, attempt], |r| {
            Ok(json!({
                "from": r.get::<_, Option<String>>(0)?,
                "to": r.get::<_, String>(1)?,
                "writer": r.get::<_, String>(2)?,
                "ts": r.get::<_, i64>(3)?,
                "evidence_digest": r.get::<_, Option<String>>(4)?,
                "cause": r.get::<_, Option<String>>(5)?,
                "fence_reason": r.get::<_, Option<String>>(6)?,
            }))
        })?
        .collect::<std::result::Result<_, _>>()?;
    Ok(Value::Array(rows))
}

struct Row {
    state: String,
    nonce_sha: String,
    token_sha: String,
    sgen: i64,
    evidence_digest: Option<String>,
    stop_cause: Option<String>,
}

fn transition_tx(tx: &Transaction, req: &TransitionRequest, now_ms: i64) -> WcResult<TransitionOutcome> {
    let id = &req.dispatch_id;
    let row: Option<Row> = tx
        .query_row(
            "SELECT state, nonce_sha256, supervisor_token_sha256, supervisor_gen, evidence_digest, stop_cause
             FROM wc_attempts WHERE dispatch_id=?1 AND attempt=?2",
            params![id, req.attempt],
            |r| {
                Ok(Row {
                    state: r.get(0)?,
                    nonce_sha: r.get(1)?,
                    token_sha: r.get(2)?,
                    sgen: r.get(3)?,
                    evidence_digest: r.get(4)?,
                    stop_cause: r.get(5)?,
                })
            },
        )
        .optional()?;
    let Some(row) = row else {
        return Err(WcError::NotFound(format!("{id} attempt {}", req.attempt)));
    };
    // The fence is per attempt: the write is checked against ITS OWN attempt's nonce.
    if sha256_hex(&req.nonce) != row.nonce_sha {
        return fence(FenceReason::BadNonce, "nonce does not match this attempt");
    }
    // The nonce authorises worker fields only; every write here is a lifecycle write.
    match &req.supervisor_token {
        Some(t) if sha256_hex(t) == row.token_sha => {}
        _ => return fence(FenceReason::BadCredential, "missing or wrong supervisor credential"),
    }
    let current: i64 = tx.query_row(
        "SELECT MAX(attempt) FROM wc_attempts WHERE dispatch_id=?1",
        params![id],
        |r| r.get(0),
    )?;
    let is_current = req.attempt == current;
    let actual = row.state.as_str();
    let refuse = |reason: FenceReason, detail: String| -> WcResult<TransitionOutcome> {
        // Refusals on an authenticated write leave a history row (outside the lifecycle).
        append_history(tx, id, current, Some(actual), actual, "supervisor", now_ms, None, None, Some(reason.as_str()))?;
        fence(reason, detail)
    };

    match req.to.as_str() {
        "running" => {
            // §3.3 fence order (T-3529, CC-06): attempt currency is tested before transition
            // legality, so a running-path write naming a non-current attempt is stale_attempt
            // whatever its prior state. T5/T8 are not gated here: late evidence for a lost
            // attempt is legitimate (CC-07, CC-24).
            if !is_current {
                return refuse(FenceReason::StaleAttempt, format!("attempt {} is not current", req.attempt));
            }
            if req.expected_prior == "dispatched" && actual == "dispatched" {
                // T2
                if req.supervisor_gen != row.sgen {
                    return refuse(FenceReason::StaleGen, format!("sgen {} != {}", req.supervisor_gen, row.sgen));
                }
                let timeout = req.worker_timeout_ms.unwrap_or(DEFAULT_WORKER_TIMEOUT_MS).max(0);
                tx.execute(
                    "UPDATE wc_attempts SET state='running', deadline_running=?3
                     WHERE dispatch_id=?1 AND attempt=?2",
                    params![id, req.attempt, now_ms + timeout + RUNNING_SLACK_MS],
                )?;
                append_history(tx, id, req.attempt, Some("dispatched"), "running", "supervisor", now_ms, None, None, None)?;
            } else if req.expected_prior == "running" && actual == "running" {
                // T-RECLAIM: claims sgen = prev + 1
                if req.supervisor_gen != row.sgen + 1 {
                    return refuse(
                        FenceReason::StaleGen,
                        format!("reclaim needs sgen {}, got {}", row.sgen + 1, req.supervisor_gen),
                    );
                }
                tx.execute(
                    "UPDATE wc_attempts SET supervisor_gen=?3 WHERE dispatch_id=?1 AND attempt=?2",
                    params![id, req.attempt, req.supervisor_gen],
                )?;
                append_history(tx, id, req.attempt, Some("running"), "running", "supervisor", now_ms, None, req.cause.as_deref(), None)?;
            } else if req.expected_prior != actual {
                return refuse(FenceReason::CasMismatch, format!("expected {}, state is {actual}", req.expected_prior));
            } else {
                return refuse(FenceReason::ForbiddenTransition, format!("{actual} -> running"));
            }
        }
        "done" => {
            let digest = req
                .evidence_digest
                .as_deref()
                .filter(|d| !d.is_empty())
                .ok_or_else(|| WcError::Invalid("transition to done needs evidence_digest".into()))?;
            match actual {
                // T8: a response, not a transition.
                "done" => {
                    if row.evidence_digest.as_deref() == Some(digest) {
                        return Ok(TransitionOutcome { record: attempt_record(tx, id, req.attempt)?, deduped: true });
                    }
                    return refuse(FenceReason::DoneConflict, "done with different evidence_digest".into());
                }
                "running" | "dispatched" => {
                    // T3 / T3b: CAS on the state actually found.
                    if req.expected_prior != actual {
                        return refuse(FenceReason::CasMismatch, format!("expected {}, state is {actual}", req.expected_prior));
                    }
                    if !is_current {
                        return refuse(FenceReason::StaleAttempt, format!("attempt {} is not current", req.attempt));
                    }
                    if req.supervisor_gen != row.sgen {
                        return refuse(FenceReason::StaleGen, format!("sgen {} != {}", req.supervisor_gen, row.sgen));
                    }
                }
                // T5: only evidence that existed when the end was confirmed.
                "lost" => {
                    if !matches!(req.expected_prior.as_str(), "lost" | "running" | "dispatched") {
                        return refuse(FenceReason::CasMismatch, format!("expected {}, state is lost", req.expected_prior));
                    }
                    let lost_ts: i64 = tx.query_row(
                        "SELECT MAX(ts) FROM wc_history WHERE dispatch_id=?1 AND attempt=?2
                           AND to_state='lost' AND fence_reason IS NULL",
                        params![id, req.attempt],
                        |r| r.get(0),
                    )?;
                    match req.persisted_at {
                        Some(p) if p <= lost_ts => {}
                        _ => {
                            // T-3521: the cause marks this refusal so it raises a `late_evidence` notice.
                            append_history(
                                tx, id, current, Some(actual), actual, "supervisor", now_ms, None,
                                Some(notices::CAUSE_LATE_EVIDENCE), Some(FenceReason::ForbiddenTransition.as_str()),
                            )?;
                            return fence(
                                FenceReason::ForbiddenTransition,
                                "evidence was not persisted before the lost entry",
                            );
                        }
                    }
                }
                // T7b: no persisted_at bound.
                "unknown" => {
                    if !matches!(req.expected_prior.as_str(), "unknown" | "running" | "dispatched") {
                        return refuse(FenceReason::CasMismatch, format!("expected {}, state is unknown", req.expected_prior));
                    }
                }
                other => return refuse(FenceReason::ForbiddenTransition, format!("{other} -> done")),
            }
            let outcome = derive_outcome(req);
            tx.execute(
                "UPDATE wc_attempts SET state='done', outcome=?3, exit_code=?4, result_kind=?5,
                    result_value=?6, persisted_at=?7, evidence_digest=?8, handback_status=?9
                 WHERE dispatch_id=?1 AND attempt=?2",
                params![
                    id,
                    req.attempt,
                    outcome,
                    req.exit_code,
                    req.result_kind.as_deref().unwrap_or("none"),
                    req.result_value,
                    req.persisted_at,
                    digest,
                    req.handback.as_ref().and_then(|h| h.status.clone())
                ],
            )?;
            // T-3523: a recorded backend stop gives the confirmed end its cause (§3.3).
            let cause = req.cause.as_deref().or(row.stop_cause.as_deref());
            append_history(tx, id, req.attempt, Some(actual), "done", "supervisor", now_ms, Some(digest), cause, None)?;
            answer::close_ended_answers_tx(tx, id, req.attempt, "done", now_ms)?;
        }
        // lost / unknown are the sweep's (S2); dispatched is only ever T1.
        other => return refuse(FenceReason::ForbiddenTransition, format!("{actual} -> {other}")),
    }
    Ok(TransitionOutcome { record: attempt_record(tx, id, req.attempt)?, deduped: false })
}

/// T-3599: §3.3 rule 1 — a hand-back whose commission names another dispatch is
/// malformed. An external backend may prefix the id (`<backend>:<dispatch_id>`), so the
/// part after the last `:` is compared too. T-3600: commission is part of the hand-back
/// (§2), so an absent one is malformed as well (AEF T-4803 put it in every vector).
fn commission_matches(commission: Option<&str>, dispatch_id: &str) -> bool {
    match commission {
        None => false,
        Some(c) => c == dispatch_id || c.rsplit_once(':').is_some_and(|(_, id)| id == dispatch_id),
    }
}

/// `outcome` derivation, in the spec's order (§3.3).
fn derive_outcome(req: &TransitionRequest) -> &'static str {
    let hb_ok = req.handback.as_ref().is_some_and(|h| {
        h.attempt == Some(req.attempt)
            && h.nonce.as_deref() == Some(req.nonce.as_str())
            && h.status.is_some()
            && commission_matches(h.commission.as_deref(), &req.dispatch_id)
    });
    // The refused flag outranks the handback check (CC-16, CC-21): a refusal carries no
    // handback. An unrefused exit with an absent handback stays malformed (CC-25).
    if req.refused {
        return "refused";
    }
    if !hb_ok {
        return "handback-malformed";
    }
    let status = req.handback.as_ref().and_then(|h| h.status.as_deref()).unwrap_or("");
    let exit = req.exit_code.unwrap_or(0);
    if exit != 0 || status == "failed" {
        return "failed";
    }
    if status == "paused" {
        return "paused";
    }
    "succeeded"
}

/// T-3523 (§3.3): "The stop is an action, not a state." It records the cause and nothing
/// else: no history row, no state change, no new attempt. The confirmed end that follows
/// (T3, or the sweep's T4) carries the cause on its history entry.
fn stop_tx(tx: &Transaction, req: &StopRequest, now_ms: i64) -> WcResult<Value> {
    let id = &req.dispatch_id;
    if !STOP_CAUSES.contains(&req.cause.as_str()) {
        return Err(WcError::Invalid(format!("stop cause must be one of {STOP_CAUSES:?}")));
    }
    let row: Option<(String, String, Option<String>)> = tx
        .query_row(
            "SELECT state, supervisor_token_sha256, stop_cause FROM wc_attempts WHERE dispatch_id=?1 AND attempt=?2",
            params![id, req.attempt],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((state, token_sha, recorded)) = row else {
        return Err(WcError::NotFound(format!("{id} attempt {}", req.attempt)));
    };
    match &req.supervisor_token {
        Some(t) if sha256_hex(t) == token_sha => {}
        _ => return fence(FenceReason::BadCredential, "missing or wrong supervisor credential"),
    }
    let refuse = |reason: FenceReason, detail: String| -> WcResult<Value> {
        // Like every refused authenticated write: a history row outside the lifecycle.
        append_history(tx, id, req.attempt, Some(&state), &state, "supervisor", now_ms, None, None, Some(reason.as_str()))?;
        fence(reason, detail)
    };
    if !matches!(state.as_str(), "dispatched" | "running") {
        return refuse(FenceReason::CasMismatch, format!("attempt is {state}; only a live attempt can be stopped"));
    }
    match recorded.as_deref() {
        Some(c) if c == req.cause => {}
        Some(c) => return refuse(FenceReason::CasMismatch, format!("already stopped for {c}")),
        None => {
            tx.execute(
                "UPDATE wc_attempts SET stop_cause=?3 WHERE dispatch_id=?1 AND attempt=?2",
                params![id, req.attempt, req.cause],
            )?;
        }
    }
    attempt_record(tx, id, req.attempt)
}

fn attempt_record(tx: &Transaction, id: &str, attempt: i64) -> WcResult<Value> {
    let mut v = tx.query_row(
        &format!("SELECT {ATTEMPT_COLS} FROM wc_attempts WHERE dispatch_id=?1 AND attempt=?2"),
        params![id, attempt],
        attempt_json,
    )?;
    v["dispatch_id"] = json!(id);
    v["history"] = history_json(tx, id, attempt)?;
    Ok(v)
}

#[allow(clippy::too_many_arguments)]
fn append_history(
    tx: &Transaction,
    id: &str,
    attempt: i64,
    from: Option<&str>,
    to: &str,
    writer: &str,
    ts: i64,
    digest: Option<&str>,
    cause: Option<&str>,
    fence_reason: Option<&str>,
) -> WcResult<()> {
    tx.execute(
        "INSERT INTO wc_history (dispatch_id, attempt, from_state, to_state, writer, ts,
            evidence_digest, cause, fence_reason) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![id, attempt, from, to, writer, ts, digest, cause, fence_reason],
    )?;
    Ok(())
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS wc_dispatches (
            dispatch_id TEXT PRIMARY KEY,
            contract_version TEXT NOT NULL,
            backend TEXT NOT NULL,
            task_id TEXT,
            retry_of_dispatch_id TEXT,
            dispatcher_project_agent TEXT,
            dispatcher_session TEXT,
            claude_session_id TEXT,
            created_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS wc_attempts (
            dispatch_id TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            nonce_sha256 TEXT NOT NULL,
            supervisor_token_sha256 TEXT NOT NULL,
            state TEXT NOT NULL,
            outcome TEXT,
            worker_kind TEXT,
            session_name TEXT,
            supervisor_gen INTEGER NOT NULL DEFAULT 1,
            exit_code INTEGER,
            result_kind TEXT,
            result_value TEXT,
            persisted_at INTEGER,
            evidence_digest TEXT,
            attested INTEGER NOT NULL DEFAULT 0,
            attested_by TEXT,
            handback_status TEXT,
            deadline_dispatched INTEGER,
            deadline_running INTEGER,
            binding_seen_at INTEGER,
            seen_epoch INTEGER,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (dispatch_id, attempt)
         );
         CREATE TABLE IF NOT EXISTS wc_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            dispatch_id TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            from_state TEXT,
            to_state TEXT NOT NULL,
            writer TEXT NOT NULL,
            ts INTEGER NOT NULL,
            evidence_digest TEXT,
            cause TEXT,
            fence_reason TEXT
         );
         CREATE INDEX IF NOT EXISTS wc_history_attempt ON wc_history (dispatch_id, attempt, id);
         CREATE TABLE IF NOT EXISTS wc_registry (
            k INTEGER PRIMARY KEY CHECK (k = 1),
            epoch INTEGER NOT NULL
         );
         INSERT OR IGNORE INTO wc_registry (k, epoch) VALUES (1, 0);",
    )?;
    // T-3502 (S2): sweep bookkeeping. Added after S1 shipped, so existing meta.db
    // files get the columns here rather than through the CREATE above.
    ensure_column(conn, "wc_attempts", "end_observed_at", "INTEGER")?;
    ensure_column(conn, "wc_attempts", "absent_sweeps", "INTEGER NOT NULL DEFAULT 0")?;
    // T-3523: a backend-initiated stop's cause, carried onto the confirmed end's history.
    ensure_column(conn, "wc_attempts", "stop_cause", "TEXT")?;
    // T-3503 (S3): the answer path.
    answer::init_schema(conn)?;
    // T-3521: durable operator notices (triggers on wc_history).
    notices::init_schema(conn)?;
    Ok(())
}

/// `ALTER TABLE ... ADD COLUMN` unless the column is already there.
fn ensure_column(conn: &Connection, table: &str, column: &str, decl: &str) -> Result<()> {
    let mut st = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names: Vec<String> = st.query_map([], |r| r.get::<_, String>(1))?.collect::<std::result::Result<_, _>>()?;
    if !names.iter().any(|n| n == column) {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl};"))?;
    }
    Ok(())
}

fn sha256_hex(s: &str) -> String {
    let d = Sha256::digest(s.as_bytes());
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    use std::io::Read;
    let mut buf = [0u8; N];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut buf)?;
    Ok(buf)
}

fn random_token(prefix: &str) -> Result<String> {
    let b = random_bytes::<16>()?;
    Ok(format!("{prefix}{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>()))
}

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// `D-<ulid>`: 26 Crockford base32 characters, first one 0..=7 (128 bits).
pub fn is_valid_dispatch_id(id: &str) -> bool {
    let Some(u) = id.strip_prefix("D-") else { return false };
    let b = u.as_bytes();
    b.len() == 26 && b[0] <= b'7' && b.iter().all(|c| CROCKFORD.contains(c))
}

fn new_dispatch_id(now_ms: i64) -> Result<String> {
    let rnd = random_bytes::<10>()?;
    let mut v: u128 = ((now_ms.max(0) as u128) & 0xFFFF_FFFF_FFFF) << 80;
    let mut r: u128 = 0;
    for b in rnd {
        r = (r << 8) | b as u128;
    }
    v |= r;
    let mut out = String::from("D-");
    for i in (0..26).rev() {
        out.push(CROCKFORD[((v >> (5 * i)) & 0x1F) as usize] as char);
    }
    Ok(out)
}

pub mod answer;
pub mod notices;
pub mod sweep;
#[cfg(test)]
mod answer_tests;
#[cfg(test)]
mod notices_tests;
#[cfg(test)]
mod sweep_tests;
#[cfg(test)]
mod tests;
