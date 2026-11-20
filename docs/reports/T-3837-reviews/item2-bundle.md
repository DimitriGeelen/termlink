# Shared worker contract v0.3.4 item 2 — optional dispatch closure (review bundle)

## Context
Three projects (AEF, ring20, TermLink) co-own a "shared worker contract" (wc/0.3) for pool jobs. A *dispatch* is one
job request; it has one or more *attempts* (retries are new attempts, allocated by transition T1). Attempt states:
dispatched -> running -> done | lost | unknown (and paused as a done outcome). Every write is one IMMEDIATE SQLite
transaction with compare-and-set on the expected prior state, a per-attempt nonce, a supervisor token and a
supervisor generation (fencing). Refusals are typed fence reasons (cas_mismatch, stale_attempt, stale_gen,
forbidden_transition, done_conflict, bad_nonce, bad_credential, ...). Conformance vectors CC-1..CC-29 pin behaviour
across the three backends.

## Item 2 as drafted (AEF's summary, verbatim)
"Dispatch closure (§2, §3.3 T1 and fence_reject, §4.7, §8.4, CC-15, CC-19): a backend MAY close a dispatch once its
last attempt is terminal (done or lost), recorded with actor, reason and ts; a T1 on a closed dispatch is
fence_reject(dispatch_closed), a new reason; CC-15/CC-19 accept either T1 or fence_reject(dispatch_closed) on a
declared route. Closing changes no attempt, and a closed dispatch whose last attempt is paused never counts as
finished. A backend that never closes a dispatch (TermLink's hub) is unaffected and stays conformant. Version field
stays wc/0.3."
Origin: ring20 closes finished dispatches on its own backend and asked for the spec to allow it.

## TermLink's backend: allocation (T1) and the transition fence (excerpts, crates/termlink-bus/src/worker_contract.rs)
```rust
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
```

```rust
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
```
