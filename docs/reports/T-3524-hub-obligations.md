# T-3524 — which remaining wc/0.3 vector capabilities are hub obligations

**Status:** research artifact (C-001). Shaping only; no decision is recorded here. The recommendation in the task
stays DEFER until the operator rules. Agent leans below are proposals, not rulings. The questions are IW-1..IW-5 in
the task file.

**Measured state (2026-10-22, AEF vectors `25f56ed68`, `scripts/run-wc-vectors.sh`):** 25 cases, pass 18, fail 0,
not-implemented 6, n/a 1 (CC-11, `real_worker` not offered). The six not-implemented cases each stop at one op the
hub backend does not have. The adapter reports these as `Fail::Missing`, never as a pass
(`crates/termlink-hub/tests/wc_conformance/hub.rs`).

## The question

AEF's vectors are written for any backend. Some ops describe things a backend clearly owns (`allocate`, `complete`,
`confirm_end`). Others describe things that may belong to the dispatcher, the operator tooling, or AEF's evaluation
layer. For each remaining op: should the TermLink hub provide it (build), or is it outside the hub's job? If it is
outside, AEF would mark it n/a for this backend, or move it to another subject. A third option is that an existing
hub behaviour already IS the op, and only the mapping needs agreeing.

## The six cases

| # | Case | Op that stops it | What the vector asks | What the hub has today |
|---|------|------------------|----------------------|------------------------|
| 1 | CC-08 Hub restart (variant `undecidable-then-alive`) | `wait` | A consumer waits on the attempt with a bound and gets its state. | `dispatch.get` (a read). No blocking wait on a dispatch record. |
| 2 | CC-09 Partition | `partition`, `heal` | The worker host loses contact with the authority, then regains it. | No model of worker-host contact. A lost contact shows up as an absent registration, which the sweep already handles (T4 after two absent sweeps and grace G; `probe_undecided` for an undecidable end). |
| 3 | CC-10 Parent-session loss | `inspect operator_queue`, `adopt` | The question reaches the operator's queue while the dispatcher is gone; a named successor adopts ownership. | Durable operator notices exist (T-3521, `audience='operator'`), but no notice is raised for "question with no dispatcher". `dispatcher.session` is recorded, but nothing transfers it. |
| 4 | CC-13 Local backend parity (variant `lock-timeout-fails-closed`) | `hold_lock` | Another holder keeps the write lock; the write must fail closed after the backend's bound. | Every write is SQLite `BEGIN IMMEDIATE`, with a 30 s bound (`WRITE_LOCK_WAIT_MS`, T-3531). Past the bound the Meta path raises a named `WriteLockTimeout`; `WcStore` surfaces it as `WcError::Bus`. T-3531's tests already hold the lock from a second connection. |
| 5 | CC-19 Paused is terminal, not success | `consume` | An evaluation-layer consumer (review verdict, pass rate) refuses a `paused` attempt as `not_finished_work`. | Nothing. Consumers live in AEF, outside the store. |
| 6 | CC-22 Attested done is not finished work | `consume` | The same consumer refuses an attested `done`. | Same as row 5. The record now carries what a consumer needs (`attested`, `evidence_digest` NULL, `outcome`; T-3539). |

## Options per question, with the agent's lean

1. **IW-1, CC-08 `wait`.**
   1a. Build `dispatch.wait {dispatch_id, attempt, timeout_ms}`, a bounded long-poll that returns when the state changes or the bound ends. This mirrors `event.poll`. Small, and the hub is the natural place, since only the store knows when a state changes.
   1b. Treat `wait` as a client-side poll of `dispatch.get`; the adapter would poll. That is honest only if AEF agrees a poll satisfies "wait".
   1c. *Lean:* 1a, a small build with an obvious owner.
2. **IW-2, CC-09 `partition` / `heal`.**
   2a. Agree with AEF that, for a hub backend, a partition is an absent registration while the worker runs (the existing two-sweep-plus-G rule, `probe_undecided` when undecidable). The adapter would map `partition` to an absent observation and `heal` to a live one. No hub change.
   2b. Build an explicit worker-host contact model in the hub.
   2c. *Lean:* 2a, with AEF's agreement on record. 2b duplicates what the registry already observes.
3. **IW-3, CC-10 `inspect operator_queue` and `adopt`.**
   3a. Raise an operator notice when a question is asked and `dispatcher.session` is not live (it fits the existing T-3521 notice table), and map `operator_queue` to the attempt's operator notices. Build `adopt` as a supervisor-or-operator write of `dispatcher.session` with a history row.
   3b. Declare dispatcher ownership out of the hub's scope, with AEF marking CC-10 n/a for this backend.
   3c. *Lean:* 3a. Spec §4.6 makes "bounces surface in the project operator queue" the hub's job until adoption.
4. **IW-4, CC-13 `hold_lock`.**
   4a. Map `hold_lock` in the adapter to a second connection holding `BEGIN IMMEDIATE` (as T-3531's tests do), and have `WcStore` map a busy timeout to a named fail-closed error.
   4b. Treat CC-13's lock-timeout variant as covered by T-3531's unit tests and ask AEF to accept them as the evidence.
   4c. *Lean:* 4a. It is mostly test plumbing plus naming one error.
5. **IW-5, CC-19 and CC-22 `consume`.**
   5a. Not a hub obligation. The consumers are AEF's evaluation layer. The hub's part is to carry the facts a consumer needs on the record: `outcome`, `attested`, `evidence_digest`, all present. AEF would mark `consume` steps as consumer-subject (n/a for a backend), or run them against its own consumer with the hub as the store.
   5b. Build a hub-side `finished_work: bool` derivation (done and not attested, outcome succeeded) as a read-only field, and let the adapter answer `consume` from it. That puts an evaluation rule into the hub.
   5c. *Lean:* 5a, possibly with 5b's field as a convenience that AEF can choose to read, never as the hub deciding.

## What a ruling would authorise

1. **If the leans are taken:** three small builds (`dispatch.wait`, CC-10's notice plus `adopt`, CC-13's lock plumbing). One mapping (partition), recorded in the conformance map. One message to AEF asking that `consume` steps be marked consumer-subject, and that partition-as-absence be accepted for a hub backend. T-3540 (consume) would close as "not a hub obligation" under 5a.
2. **If the operator prefers a narrower hub:** 1b, 3b and 4b keep the hub unchanged and move the asks to AEF.
3. **Left open either way:** whether AEF accepts the mappings. Their vectors are theirs (no copy is kept here).

## Dialogue Log

1. **2026-10-22.** Artifact written after T-3523, T-3527, T-3528 and T-3539 took the vectors to 18/0. No operator dialogue yet.
