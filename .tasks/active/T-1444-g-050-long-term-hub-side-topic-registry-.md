---
id: T-1444
name: "G-050 long-term: hub-side topic registry persistence (option-1 follow-up to T-1443 mitigation)"
description: >
  Inception: G-050 long-term: hub-side topic registry persistence (option-1 follow-up to T-1443 mitigation)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T21:28:03Z
last_update: 2026-05-02T05:35:58Z
date_finished: null
---

# T-1444: G-050 long-term: hub-side topic registry persistence (option-1 follow-up to T-1443 mitigation)

## Problem Statement

G-050 (mitigated via T-1443) — hub channel/topic state lives only in
`RwLock<HashMap<String, Topic>>` with no on-disk backing. Every hub binary swap
silently destroys all topic registrations and retention rules. T-1294 already
solved the parallel problem for `hub.secret` + `hub.cert.pem` by persisting them
to runtime_dir; topic registry needs the same. T-1443 shipped the client-side
opt-in self-heal (`channel post --ensure-topic`), but every caller now has to
remember to pass the flag — not a structural fix. This inception explores
whether to land the proper hub-side persistence (option-1 from G-050).

For whom: vendored Claude Code agents that post to chat-arc, scratchpads, and
DM topics across every hub in the fleet. Currently five hosts (.107, .122,
.141, .143, plus testhub). Why now: T-1443 closes the immediate operational
hole, freeing room to design the long-term fix without a fire under it.

## Assumptions

- A1. The set of state that needs to survive restart is small and bounded —
  topic name + retention policy + (maybe) topic_metadata description.
  Message logs are deliberately out of scope (they're an in-memory ring per
  the existing design, separate axis).
- A2. The runtime_dir filesystem is durable across hub restarts (same
  assumption T-1294 relies on; volatile-/tmp class of failures is its own
  ring (PL-021/T-1294) and out of scope here).
- A3. Atomicity per change (write-then-rename) is acceptable; we don't need
  WAL-grade durability — losing the last few seconds of topic creates on
  power-cut is OK.
- A4. The number of topics per hub stays under 10K — JSON or RON file format
  is fine; we don't need a sqlite/B-tree.

## Exploration Plan

- S1 (1h): Read termlink-hub channel/topic registry code to map exactly what
  state is in the in-memory map. Identify the create/delete/mutate paths.
- S2 (30min): Compare with T-1294's persist-if-present pattern in
  termlink-session/agent_identity.rs; check whether it generalizes via a
  shared persistence helper.
- S3 (30min): Decide file format (JSON vs RON vs sled). Probably JSON for
  ops-readability — operators already cat hub.secret.
- S4 (30min): Identify reload-on-start path; check ordering vs other init
  steps (does anything need topics ready before we read them back?)
- S5 (15min): Sketch a migration plan — old hubs without the persisted file
  start empty; first restart after upgrade writes the file. No migration
  step needed.

## Technical Constraints

- Must not break the single-binary distribution; any persisted file must be
  in runtime_dir (configurable via TERMLINK_RUNTIME_DIR), not /var/lib hard-coded.
- Concurrent access: writes happen on every channel.create/delete; reads only
  on hub start. RwLock semantics already in place; persistence layer must
  serialize writes (probably via a tokio::sync::Mutex around the file writer).
- File handle lifetime: don't hold an fd open across the lifetime of the hub
  (eats fds; surprising for ops). Open-write-close per change.
- Format choice constrained by serde compatibility with the existing
  retention enum (Forever, Days(N), Messages(N)).

## Scope Fence

**IN scope:**
- Topic name + retention policy persistence
- Reload on hub start
- topic_metadata envelope persistence (the description ones — small,
  finite, makes the topic self-describing across restart too)
- Atomic-write semantics (write-then-rename)

**OUT of scope:**
- Message log persistence (separate, much bigger scope; messages are
  retention-bounded already and the current behavior is consistent — restart
  drops them everywhere)
- Subscriber cursor persistence (cursors live client-side already, T-1318)
- Cross-host topic sync (federation; orthogonal to durability)
- Backfill of message history from a peer (out of scope here; that's its own
  inception)

## Acceptance Criteria

### Agent
- [x] Spike S1-S5 conducted; findings recorded in `docs/reports/T-1444-g-050-long-term.md`
  **Evidence:** S1 (15min) + S2 (5min) refuted A1; S3-S5 not reached. Full artifact at `docs/reports/T-1444-g-050-long-term.md`.
- [x] Assumptions A1-A4 each marked validated/refined/refuted with evidence in the same artifact
  **Evidence:** A1 **REFUTED** (topics already in SQLite at `<runtime_dir>/bus/meta.db`); A2 **VALIDATED** (same as T-1294 invariant); A3/A4 **N/A** (SQLite WAL provides ACID, scales beyond 10K). Table in artifact §"Assumption Re-evaluation".
- [x] Recommendation written: GO with bounded plan (file format chosen, write path identified, reload path identified), or NO-GO with rationale, or DEFER with re-trigger condition
  **Evidence:** **NO-GO** — premise refuted. Re-trigger conditions documented (topic-loss on durable runtime_dir = different bug class; new state beyond SQLite coverage = separate inception). See artifact §"Recommendation".

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** **NO-GO**

**Rationale:** The premise (in-memory-only topic registry needing
hand-rolled persistence) is wrong. Topics are already persisted in
SQLite at `<runtime_dir>/bus/meta.db` via `termlink_bus::meta::Meta`.
The observed "topic loss after restart" incidents trace to
runtime_dir volatility (T-1294 territory), not to a missing
persistence layer. Hand-rolling JSON/RON would duplicate SQLite's
existing ACID-backed registry.

**Evidence:**
- `crates/termlink-bus/src/lib.rs:38-92` — `Bus { meta: meta::Meta, ... }`
  delegates create/list/exists/retention/append to SQLite. The
  `RwLock<HashMap<...>>` referenced in this task's premise is
  `appenders` + `notifiers` runtime caches (log-handle pool + tokio
  Notify primitives), not the registry.
- `/var/lib/termlink/bus/meta.db` exists on .107, is 1.3 MB, contains
  all 4 canon topics (`agent-chat-arc`, `broadcast:global`,
  `channel:learnings`, `framework:pickup`) per
  `sqlite3 ... 'SELECT name FROM topics'`. Live-modified (mtime 07:31
  current). Survives every hub restart on durable runtime_dir.
- T-1443's `--ensure-topic` flag remains useful as an idempotent
  client-side shortcut (callers don't track which canon topics need
  pre-creation). T-1445 deployed it in framework scripts. Keep it.

**Long-term G-050 fix is subsumed by T-1294-class work:**
- T-1294 already migrated .122 ring20-management.
- T-1296 (captured/next) covers .121/.143 ring20-dashboard.
- Periodic sweep should audit any other hub still on `/tmp/termlink-0`
  (likely none on current fleet).

**Re-trigger conditions for re-opening this inception:**
1. A topic appears lost on a hub where `<runtime_dir>/bus/meta.db`
   exists, is durable, and contains the topic in SQLite, but
   `termlink topics` does not return it. That would mean the
   cache→SQLite reload path has a bug — different scope.
2. A future requirement to persist state beyond SQLite's coverage
   (e.g., subscriber cursors currently client-side per T-1318) —
   that's a separate inception.

See full research artifact: `docs/reports/T-1444-g-050-long-term.md`.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-01T21:29:10Z — status-update [task-update-agent]
- **Change:** horizon: now → next

### 2026-05-02T05:35:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
