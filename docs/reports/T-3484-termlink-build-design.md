# T-3484: TermLink's build for the shared worker contract (hub backend)

**Status:** draft build design, written before AEF's v0.2 so the build task can be filed the moment
v0.2 is agreed. **Source contract:** AEF `docs/specs/shared-worker-contract-transitions.md` v0.1
(AEF T-4439) plus the three changes AEF accepted for v0.2 from our review
(`docs/reports/T-3483-consult/review-aef-draft-v0.1.md`, AEF msg 188b4445).
**Scope:** the `backend: hub` implementation only. The local backend, the reconciler and the
`fw worker` verbs are AEF's; ring20 conforms by running the vectors (their T-2443 decides).

Every code fact below was read from the tree on 2026-10-20 (file:line given).

## 1. What already exists and is reused

1. **Session ↔ attempt binding (T-3490, deployed in 0.12.800+).** `--worker-binding <dispatch_id>:<attempt>`
   on `spawn`/`register`, stored in the registration, unique among live registrations (local file
   path and `session.register_remote`), returned by `session.discover`. This is the contract's
   `session_binding`, with the hub-enforced uniqueness the review asked for.
2. **Durable store with the right commit semantics.** Bus meta SQLite, WAL + `synchronous=FULL`
   (T-3461), one transaction per write (T-3488). `claims` (meta.rs ~1242) is the closest precedent: a
   CAS-checked row with an owner and a lease, refused with structured errors.
3. **The sweep.** `crates/termlink-hub/src/supervisor.rs`, every 30 s, reads every registration and
   `liveness::is_alive` (pid + socket). Registrations are files, so they survive a hub restart.
4. **Error-code space.** Last assigned is `PLAINTEXT_FIELD_TOO_LARGE` -32025
   (`crates/termlink-protocol/src/control.rs:523`). This build takes -32026 and -32027.

## 2. Storage (meta.db, new tables, one migration)

| Table | Key | Columns (contract field → column) |
|---|---|---|
| `wc_dispatches` | `dispatch_id` | contract_version, backend (always `hub` here), task_id, retry_of_dispatch_id, dispatcher_project_agent, dispatcher_session, claude_session_id, created_at |
| `wc_attempts` | (`dispatch_id`, `attempt`) | nonce_sha256 (the nonce itself is returned once at allocation and never stored), state, outcome, worker_kind, session_name, supervisor_gen, exit_code, result_kind, result_value, evidence_digest, deadline_running, binding_seen_at, created_at |
| `wc_history` | autoincrement | dispatch_id, attempt, from_state, to_state, writer, ts, evidence_digest, fence_reason (NULL unless a refused write) |
| `wc_answers` | (`dispatch_id`, `attempt`, `cmd_id`) | question_id, body, deadline, state (stored / received / applied / undelivered / expired), timestamps |
| `wc_questions` | (`dispatch_id`, `attempt`, `question_id`) | body, ts |

Storing `sha256(nonce)` rather than the nonce follows the contract's tombstone rule (§7.2) from day one.
It also means a database dump does not hand out live write credentials.

## 3. RPCs (all writes are one transaction: check nonce, gen and prior state, then apply)

| Method | Writer | Contract rows | Scope |
|---|---|---|---|
| `dispatch.allocate {dispatch_id?, dispatcher, task_id?, retry_of?}` → `{dispatch_id, attempt, nonce}` | dispatcher | T1 (refused unless the prior attempt is terminal or a resolved unknown, §4.3) | Execute |
| `dispatch.transition {dispatch_id, attempt, nonce, expected_prior, to, supervisor_gen, evidence_digest, exit_code?, result?, handback?}` | supervisor / reconciler | T2, T3, T3b, T-RECLAIM, T5, T7b, T8 | Execute |
| `dispatch.resolve {dispatch_id, attempt, as, reason}` | operator | T7d | Control + operator identity (open question 3) |
| `dispatch.get {dispatch_id}` | anyone | read | Observe |
| `dispatch.ask` / `dispatch.answer` / `dispatch.ack {cmd_id, received\|applied}` | worker / dispatcher | §5 answer path | Interact |

**Refusals.** `FENCE_REJECTED` (-32026) carries `data.reason ∈ {stale_attempt, bad_nonce, stale_gen,
cas_mismatch, forbidden_transition, done_conflict}` and appends a history row. `DISPATCH_NOT_FOUND`
(-32027) is the other code. Following the T-2699 rule, each new code gets an emission site and a test
in the same change, so `check-error-code-emission.sh` stays clean.

**T8 idempotency.** If the new `evidence_digest` equals the stored one, the write returns the stored
record with no new history row. A different digest gets `done_conflict` plus a WARN.

## 4. The hub-written transitions (T4, T6, T7a, T7c), driven from STATE, never from events

A new step in the existing sweep, `sweep_dispatches`. For each attempt in `dispatched` or `running`:

| Observation this sweep | Action |
|---|---|
| A live registration carries this attempt's binding | set `binding_seen_at` if unset; if the state is `unknown`, apply T7a (back to `running`) |
| No live registration carries the binding, `binding_seen_at` is set, and it was also absent on the previous sweep | **T4 → `lost`** (confirmed end: seen alive before, now absent twice, ≥ 30 s apart) |
| No registration was ever seen and `deadline_running` has passed | **T6 → `unknown`** (never confirmed, so never `lost`) |
| `running`, a live registration exists, and `deadline_running` has passed | **T6 → `unknown`** (alive past its bound: operator notice, no new attempt) |

**Why state and not events.**
1. A clean exit removes its own registration and emits nothing (supervisor.rs: only dead-pid cleanups
   are announced).
2. `session.exited` is fire-and-forget.
3. A hub restart loses in-memory events. The records and the registration files are both on disk, so
   the sweep's view survives a restart. That is the CC-08 behaviour we told AEF the hub has.

The "absent twice" rule rides out the registration gap of a hub restart and the window in which a
session re-registers. **Grace `G`** (contract §3, ≥ 4 × sweep interval = 120 s) then applies before
consumers treat `lost` as final; `lost` stays upgradable by T5.

**Sessions on another hub** are invisible here (hubs do not federate, G-060), so they can only ever
reach `unknown`. The contract requirement AEF adopted ("the record's hub is the hub the worker
registers with") makes that the exception it should be.

## 5. Conformance

The contract's §9 cases that apply to the hub backend, CC-01..10, CC-12, CC-14, CC-15 and CC-16, run
as Rust integration tests with injected time and fake liveness. They use temp dirs only: the T-3293
rule, never the real supervisor loop. CC-11 needs a real `claude -p` worker and runs as a separate
opt-in test. CC-13 (local-backend parity) is AEF's to run against both backends; we expose a test
hub for it. The vectors are written once, as data (YAML in `tests/contract-vectors/`), so AEF and
ring20 can run the same file.

## 6. Slices (each its own task, after v0.2 is agreed)

1. **S1: tables and migration**, plus `dispatch.allocate`, `dispatch.get` and `dispatch.transition`
   with full CAS and fencing; error codes; CC-04, CC-05, CC-06, CC-07, CC-14 and CC-16.
2. **S2: `sweep_dispatches`** (T4, T6, T7a, T7c) using the T-3490 binding; CC-01, CC-02, CC-03,
   CC-08 and CC-09.
3. **S3: answer path** (§5) and `dispatch.resolve` (T7d); CC-10, CC-12 and CC-15.
4. **S4: CLI and MCP surfaces** (`termlink dispatch get/…`), plus the shared vector file for AEF and
   ring20.

AEF's shadow mode (contract §8) needs S1 and S2 at most, so the migration can start before S3.

## 7. Open questions for v0.2 (ours to raise or settle)

1. **`paused` as a fifth outcome.** We prefer it (our review §3.1). AEF has not answered yet.
2. **Who allocates `dispatch_id`.** The contract says the hub or backend allocates. We propose:
   accept a caller-chosen id when unused (AEF already mints `D-<ulid>`), else allocate. Simpler for
   shadow mode.
3. **Operator identity for T7d.** Scope alone cannot tell the operator from an agent holding Control.
   Proposal: `dispatch.resolve` requires a signed request from an identity the hub is configured to
   trust as operator. Until then, it is a CLI on the hub host only. This ties into T-3493 (signed
   requests).
4. **Retention.** Tombstones per §7 (30 days lost-upgradable, 180 days records), swept by the existing
   retention sweeper, which is now on at .107, .122 and .121 (T-3311).
