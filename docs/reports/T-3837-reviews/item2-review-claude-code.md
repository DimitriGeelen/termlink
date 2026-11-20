# Review: wc/0.3.4 item 2, optional dispatch closure

I judged from the bundle only. The excerpt stops before the `done`/`lost` branches, so the late `lost->done` claims rest on the code comment ("T5/T8 are not gated… CC-07, CC-24"), not on the code itself.

## 1. Weaknesses

**W1. Close is not specified as a fenced write, so close-vs-T1 is decided by implementation accident.** — A ring20 sweeper reads "attempt 1 = lost" and decides to close. A dispatcher's T1 commits attempt 2 (`dispatched`), then the close commits. The dispatch is now closed with a live attempt, which violates the clause's own precondition, and the next T1 after that attempt is lost is refused. Nothing requires close to be one IMMEDIATE transaction with compare-and-set on (last attempt number, last state, not closed), so three backends can resolve this three ways. — **high**

**W2. Attempt writes after close are undefined.** "Closing changes no attempt" does not say whether closing *fences* attempt writes. — Attempt 1 goes `lost` on deadline, the backend closes with reason "abandoned", and then the worker's late `lost->done` arrives with evidence. Backend A accepts it, leaving a closed-as-abandoned dispatch that is actually done. Backend B refuses with `dispatch_closed`, so the evidence is dropped and the dispatcher re-dispatches finished work. That is silent divergence plus duplicated work. — **high**

**W3. The MAY plus "accept either" guts CC-15/CC-19, and "declared route" is undefined.** — A backend with a bug that spuriously refuses T1 passes CC-15, because either outcome is accepted. A wc/0.3 dispatcher with an exhaustive fence-reason enum receives `dispatch_closed` under an unchanged version string and either errors or retries forever. Conformance should be a function of state (does a closure record exist?), not of which backend you asked. — **high**

**W4. "Actor" is recorded but not authenticated.** — In the excerpt T1 itself takes no credential, and the only credentials are per-attempt (nonce, supervisor token). Any caller that can reach close can veto all future retries of someone else's dispatch and write any `actor` string. The refusal is then attributed to a name nobody verified. — **high**

**W5. Terminality and the reader view are unspecified, so closed + paused is a limbo.** — The last attempt is `done` with outcome `paused` and the backend closes. The clause says the dispatch "never counts as finished", yet T1 is refused forever and no reopen or successor path is defined. Readers on different backends will infer "closed ⇒ finished" or not. A premature close is not recoverable by any defined path. — **high**

**W6. Idempotency, audit and cross-backend disagreement are unspecified.** — A close response is lost and the caller retries: is that success, a refusal, or an overwrite of actor/reason/ts? A second closer with a different reason has no defined winner. "Recorded" does not say where, so a `dispatch_closed` refusal may leave no trace (the T1 refusal path in `allocate` writes no history row). If a dispatch id is visible on two backends, one closed and one not, T1 succeeds on one and work is duplicated with no rule for who is authoritative. — **medium**

## 2. Amendments and vectors

Vector numbers are proposals.

**A1 (W1).** "Close is transition T-CLOSE: one IMMEDIATE transaction that names `expected_last_attempt` and `expected_last_state`, and succeeds only if both match and no closure record exists; otherwise `fence_reject(cas_mismatch)`. T1 checks closedness in its own transaction, after dispatch existence and before the terminal-state check. Commit order decides."
- CC-30: last attempt 1 `lost`; T1 commits / then T-CLOSE(expected 1, lost) / `cas_mismatch`, no closure record, attempt 2 intact.
- CC-31: last attempt 1 `lost`; T-CLOSE commits / then T1 / `dispatch_closed`, attempt count stays 1.
- CC-32: last attempt `running` or `unknown` / T-CLOSE / `cas_mismatch`.

**A2 (W2).** "Closure fences T1 only. Every attempt-level transition legal on an open dispatch is equally legal on a closed one. The closure record stores `closed_at_attempt` and `closed_at_state`; finishedness is always computed from attempts, never from closure."
- CC-33: attempt 1 `lost`, closed / late `lost->done` with valid evidence / accepted; reader shows `done`, `closed_at_state=lost`, closure record unchanged.
- CC-34: same arrange / conflicting second done / `done_conflict`, exactly as on an open dispatch.

**A3 (W3).** "A backend advertises `closure: never | supported` in its capability declaration. `dispatch_closed` MUST be returned only when a readable closure record exists, and the refusal carries that record. With no closure record, T1 on a terminal last attempt MUST succeed on every backend. Clients MUST treat an unrecognised fence reason as a non-retryable refusal." Split CC-15/CC-19 by arrangement instead of accepting either.
- CC-15a/19a (all backends): terminal last attempt, no closure record / T1 / succeeds.
- CC-15b/19b (`closure: supported`): closed dispatch / T1 / `dispatch_closed` with record attached.
- CC-35 (`closure: never`): any terminal dispatch / T-CLOSE / typed refusal (`forbidden_transition`), never a silent no-op.

**A4 (W4).** "T-CLOSE is authorised by the supervisor token of the last attempt, or by the backend's own declared closure policy, recorded as actor class `backend` plus policy id. `actor` is derived from the credential presented, never caller-supplied. Failure is `bad_credential`."
- CC-36: closable dispatch / T-CLOSE with wrong or missing token / `bad_credential`, no closure record, refusal in history.
- CC-37: closable dispatch / T-CLOSE with attempt 1's token while attempt 2 is last / `bad_credential` or `stale_attempt` (owners pick one), not closed.

**A5 (W5).** "Closure is terminal for the dispatch id; there is no reopen in wc/0.3. Recovery is a new dispatch with `retry_of_dispatch_id` set to the closed id, which every backend MUST accept. `reason` is one of `completed | abandoned | superseded`; `completed` is permitted only when the last attempt is `done` with a non-paused outcome. Readers expose `closed`, `finished` and the last attempt's state/outcome as independent fields."
- CC-38: last attempt `done`/`paused` / T-CLOSE(reason=completed) / `forbidden_transition`.
- CC-39: last attempt `done`/`paused`, closed as `abandoned` / read / `closed=true, finished=false, outcome=paused`.
- CC-40: closed dispatch D / allocate new dispatch with `retry_of_dispatch_id=D` / succeeds, link readable from both.

**A6 (W6).** "The first close wins. Any later authorised T-CLOSE returns the stored record with `created=false` and changes nothing. The closure, and every `dispatch_closed` refusal, appends a history row in the same store as lifecycle history; ordering is by history sequence, not `ts`. Closedness is authoritative only on the backend named in the dispatch's `backend` field; any other backend MUST refuse T1 for that id rather than decide it."
- CC-41: closed by actor X, reason abandoned / T-CLOSE again with reason superseded / original record returned, `created=false`, actor/reason/ts unchanged.
- CC-42: closed dispatch / T1 refused twice / two history rows with reason `dispatch_closed`, attempts untouched.
- CC-43: dispatch owned by backend A / T1 for that id against backend B / typed refusal, no attempt allocated on B.

## 3. Recommendation

Adopt with amendments, not as drafted. The draft legalises ring20's behaviour but leaves every hard case to each backend: the close/T1 race, late `lost->done`, who may close, repeat closes and closed-but-paused. Its "accept either" wording turns CC-15/CC-19 from pins into permissions, which is exactly how three backends drift apart while all staying "conformant". The amendments keep the MAY: TermLink's hub implements nothing beyond a typed refusal of T-CLOSE and tolerance of an unknown reason. They also keep wc/0.3, since everything is additive and capability-declared. A1–A3 are the minimum; without them I would not adopt.

Steelman for adopting as drafted: the clause is deliberately permissive, ring20 already closes in production, and a short MAY that matches reality ships now without imposing a credentialed close operation, reason enum and reader fields on backends that never asked for them. Tightening can follow in 0.3.5 once real closures show which of these races actually occur, rather than specifying against hypotheticals.
