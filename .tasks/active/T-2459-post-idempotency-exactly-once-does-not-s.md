---
id: T-2459
name: "post-idempotency exactly-once does not survive a hub restart — dedupe LRU is in-memory only, so a persisted offline-queue replay double-applies after restart or beyond the 5min TTL (round-14 F1); decide where restart-durable idempotency lives"
description: >
  Inception: post-idempotency exactly-once does not survive a hub restart — dedupe LRU is in-memory only, so a persisted offline-queue replay double-applies after restart or beyond the 5min TTL (round-14 F1); decide where restart-durable idempotency lives

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-22T18:34:49Z
last_update: 2026-07-22T18:35:12Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2459: post-idempotency exactly-once does not survive a hub restart — dedupe LRU is in-memory only, so a persisted offline-queue replay double-applies after restart or beyond the 5min TTL (round-14 F1); decide where restart-durable idempotency lives

## Problem Statement

TermLink advertises **exactly-once** post delivery: `channel post --client-msg-id`
(T-2049) plus a hub-side LRU dedupe keyed on `(sender_id, client_msg_id)` collapses
a spoke retry to the cached `{offset, ts, deduped:true}` without re-appending. The
substrate ADR §5 leans on this: `--await-ack` "reuses dedupe + the receipt frontier
… (T-2049 dedupe → exactly-once)."

Round-14 review found the guarantee does **not survive a hub restart** (nor the 5min
dedupe TTL). The dedupe map is a process-global in-memory `OnceLock<Mutex<HashMap>>`
(`dedupe.rs:34,49,141`) with a 5-minute TTL (`dedupe.rs:41`) — no persistence. Yet
the client's offline queue durably persists `client_msg_id` to SQLite and replays it
verbatim on flush (`offline_queue.rs:44-48`, "flush-replay reuses the SAME id and the
hub recognises it"). So:

- Spoke posts K → hub commits at offset N and records the dedupe entry → TCP ack lost.
- Hub **restarts** (dedupe map wiped) **or** ≥5min elapses (entry TTL-expires) while
  the durable queue still holds the row.
- Queue flush replays the SAME `client_msg_id` → hub dedupe lookup misses → `Newly`
  → `bus.post` appends the SAME payload again at N+k. Subscribers see it **twice**.

Real guarantee = "exactly-once **within one hub lifetime AND within 5min**;
at-least-once across a restart or a long blip." The `dedupe.rs` module header itself
lists "hub bounce" as a *closed* scenario — it is not closed across a process
restart, because the committed-entry cache dies with the process while the bus-log
offset survives.

**For whom / why now:** the AEF orchestrator (arc-011) relies on `--await-ack` to
deliver worker **completion / ledger** messages exactly once. A double-applied
completion could double-count finished work or double-advance a ledger. The fix
direction spans the collaboration seam (§9): persist substrate-side, or push
idempotency to the AEF consumer — a human-owned call. Full write-up:
`docs/reports/T-2459-post-idempotency-restart-durability-inception.md`.

## Assumptions

- The hub is a single supervised durable process; restart is a "recoverable pause,"
  not a routine event (ADR §8). So the trigger (restart OR >5min blip, coincident
  with a lost ack and a pending queued replay) is **infrequent** but not impossible —
  MED likelihood, HIGH impact when it fires.
- The correct place for durable idempotency may be the consumer, not the substrate:
  an AEF completion-ledger that dedups on `client_msg_id` at the application layer is
  idempotent-by-construction and needs no hub persistence. Whether it already does
  this is unknown to the substrate (a §9 soft-dependency co-discovery).
- Persisting the full 5min-TTL dedupe cache durably (every post) may cost more than
  it's worth; a cheaper middle path persists only the `--await-ack` (exactly-once-
  critical) subset. Hub-restart-durable state belongs in `runtime_dir` alongside
  `hub.secret` (PL-111), not /tmp.

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Where should restart-durable idempotency live — substrate-persisted dedupe, or an idempotent AEF completion-ledger consumer?**
  confidence: 2
  disposition: answered
  rationale: Two viable boundaries. (a) Substrate: persist the dedupe map (or the
  `--await-ack` subset) to `runtime_dir` SQLite (PL-111) so lookup survives restart —
  restores true exactly-once transparently but adds a durable write per tracked post
  and a GC policy. (b) Consumer: document the real guarantee (at-least-once across
  restart) and require the AEF completion-ledger to dedupe on `client_msg_id` at the
  app layer — idempotent-by-construction, zero hub cost, aligns with "restart is a
  recoverable pause." The decision picks the boundary; the design is bounded either
  way. Human owns which, because (b) depends on the AEF layer's ledger design (§9).

- **IW-2: Is at-least-once-across-restart actually acceptable, or is a double-applied completion a correctness breach?**
  confidence: 1
  disposition: deferred
  rationale: Severity hinges on whether the AEF completion-ledger is already
  idempotent on `client_msg_id`. If yes, this is documentation-honesty (the ADR
  overstates "exactly-once") — LOW/MED. If no, a double-applied completion
  double-counts finished work — HIGH. The substrate cannot see the consumer's ledger
  semantics; this is the same collaboration-seam soft-dependency (§9) that gates the
  arc-011 threat model. Human/AEF-layer co-discovery decides.

- **IW-3: If persist, persist everything or only the `--await-ack` subset, and where?**
  confidence: 2
  disposition: answered
  rationale: Persisting every post's dedupe entry durably (a 5-min-TTL cache made
  permanent) is likely over-engineering — most posts are fire-and-forget where
  at-least-once is already tolerated. The exactly-once-critical path is `--await-ack`
  (completion/ledger). A cheap middle path persists ONLY await-ack client_msg_ids to
  `runtime_dir` SQLite alongside `hub.secret` (PL-111 — never /tmp, per PL-021), with
  a bounded retention (e.g. 24h). This scopes the durable-write cost to the messages
  that actually need exactly-once. Confirmed direction, not yet a build.

## Exploration Plan

The exploration (source-cited adversarial review) is DONE — see the research
artifact. On a GO the work is: (1) design spike — decide IW-1's boundary
(substrate-persist vs consumer-idempotent) and, if persist, the await-ack-subset
schema in `runtime_dir` (time-box 2h); then one-bug-one-task builds: (2) EITHER a
persistent await-ack dedupe store OR (2') an ADR/ops-doc correction of the guarantee
+ an AEF-consumer idempotency contract; (3) a regression test that survives a
simulated hub restart (post → drop dedupe map → replay → assert single append).

## Technical Constraints

- Hub-restart-durable state MUST live under `runtime_dir` (PL-111), never `/tmp`
  (PL-021 volatile-runtime_dir class) — else the persistence itself evaporates on the
  next reboot, reintroducing the gap it was meant to close.
- Any durable write is on the post hot-path; it must not add unbounded latency —
  favors the await-ack-subset scope (IW-3) over persisting every post.

## Scope Fence

**IN:** the boundary decision (substrate-persist vs consumer-idempotent) and, on GO,
the chosen build (persistent await-ack dedupe store in runtime_dir, OR contract
correction + consumer idempotency requirement) plus a restart-surviving regression
test.
**OUT (separate / not this):** the T-2457 identity-binding class (dedupe keys on the
verified `sender_id` already — not an identity gap); the offline-queue poison-pill /
cap behavior (verified working, T-2051); full multi-hub federation durability (ADR §8
open question); changing the 5-min in-lifetime TTL (orthogonal tuning).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
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
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:**

Real documented-vs-real contract gap: ADR §5 says await-ack 'reuses dedupe → exactly-once' but the dedupe map is a process-global in-memory OnceLock (dedupe.rs:49) with a 5min TTL, while the offline queue durably persists client_msg_id and replays it verbatim after a hub restart (offline_queue.rs). Across a restart the replayed post double-applies — exactly-once is really 'exactly-once within one hub lifetime + TTL, at-least-once across restart'. The fix direction (persist dedupe vs correct the contract + require idempotent AEF completion-ledger consumer) spans the collaboration seam (§9 soft-dependency), so human owns the call. GO to decide the boundary.

**Evidence:**

- **In-memory dedupe (no persistence):** `crates/termlink-hub/src/dedupe.rs:34`
  (`use std::sync::{Mutex, OnceLock}`), `:49` (`static POST_DEDUPE: OnceLock<PostDedupe>`),
  `:141` (`map: Mutex<HashMap<(String,String), DedupeEntry>>`). TTL 5min at `:41`
  (`DEFAULT_DEDUPE_TTL_MS = 300_000`); LRU capacity note `:26-27`. Nothing writes this
  map to disk — it dies with the process.
- **Durable client-side replay of the SAME id:** `crates/termlink-session/src/offline_queue.rs:44-48`
  — `client_msg_id` is `#[serde]`-persisted with the queue row so "a flush-replay reuses
  the SAME id and the hub recognises it"; `:4` "persisted to SQLite so they survive."
  Mint-once at `:51-56`.
- **The overstated contract:** ADR `docs/architecture/parallel-execution-substrate.md`
  §5 — `--await-ack` "reuses dedupe + the receipt frontier … (T-2049 dedupe →
  exactly-once)." True only within one hub lifetime + TTL.
- **Restart wipes the recogniser but not the offset:** `dedupe.rs` module header lists
  "hub bounce" as closed — it is not, across a process restart. Replay after wipe →
  `try_record_or_lookup` returns `Newly` (`dedupe.rs:126`) → `bus.post` re-appends.
- **Relevant prior:** PL-111 (restart-durable hub state belongs in runtime_dir),
  PL-021 (volatile /tmp wipes runtime_dir — the persistence must go to the persistent
  runtime_dir, not /tmp). T-2049 (the dedupe primitive), T-2286 (await-ack).
- Full write-up: `docs/reports/T-2459-post-idempotency-restart-durability-inception.md`.

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

### 2026-07-22T18:35:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
