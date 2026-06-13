---
id: T-2023
name: "Substrate: client-side reconnect + outbound queue for spokes"
description: >
  §6 primitive 5 (Resilience, hard-dep). Channel durability protects READERS. A spoke
  that briefly loses the hub has its outbound post DISCARDED (no client-side reconnect,
  no outbound queue). A worker that finishes during a hub blip loses its completion
  report. Needed so the governance plane does not silently drop ledger messages.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:33Z
last_update: 2026-06-08T07:43:50Z
date_finished: 2026-06-08T10:05:42Z
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-06-07T11:41:30Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal)
    rubric_sha: missing
---

# T-2023: Substrate: client-side reconnect + outbound queue for spokes

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Resilience. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Channel durability protects READERS. A spoke that briefly loses the hub has its outbound post DISCARDED — there is no client-side reconnect or outbound queue. A worker that finishes during a hub blip loses its completion report. The governance plane must never silently drop ledger messages.

**Why captured now:** Resilience is foundational to ring20 ops (hosts are not reliable). Filing now while the loss-mode is sharply named.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Queue location — in-memory (lost on spoke crash) vs spilled to disk (durable across spoke restart)?**
  confidence: 4
  disposition: answered
  rationale: ALREADY DISK-BACKED. `crates/termlink-session/src/offline_queue.rs` (~383 LOC) implements SQLite-backed `pending_posts` table with `default_queue_path()` at `~/.termlink/<name>/outbound.sqlite`. Shipped under T-1439, predates T-2023's filing. See docs/reports/T-2023-client-reconnect-queue-inception.md §2.

- **IW-2: Queue size cap + overflow policy — backpressure to caller, oldest-drop, or fail-loudly?**
  confidence: 4
  disposition: answered
  rationale: ALREADY FAIL-LOUDLY (R3). `DEFAULT_CAP = 1000` configurable via env; `QueueError::Full { cap }` returned when capacity exceeded. Refuses new posts rather than silent-dropping — preserves correctness over throughput, matches ADR's loud-not-silent stance. See artifact §2.

- **IW-3: Reconnect strategy — exponential backoff with jitter, max attempts, explicit fail-permanent signal?**
  confidence: 2
  disposition: answered
  rationale: PARTIAL. `attempts` counter per row exists for poison-pill detection; explicit backoff/jitter/max-attempts/fail-permanent parameters are not visible from offline_queue.rs alone — they live in the flush loop (T-1439, not yet inspected). Audit task needed to document params + identify any with poor defaults. See artifact §4.B.

- **IW-4: Idempotency — dedupe on hub if a queued post was actually delivered before disconnect?**
  confidence: 4
  disposition: deferred
  rationale: MISSING — the real remaining gap. No `client_msg_id` field on post envelope, no hub-side LRU dedupe. The double-apply scenario is reproducible: spoke posts, hub commits at offset N, TCP ack lost, spoke queues + retries, hub commits AGAIN at N+1, subscribers see the same payload twice. Fix shape: client generates `client_msg_id` (UUID or content-hash); hub maintains short-TTL (e.g. 5 min) recently-seen LRU keyed by `(sender_fingerprint, client_msg_id)` and no-ops duplicates. ~80 LOC. See artifact §4.A.

## Exploration Plan

At promotion time: (1) measure ring20 hub-blip frequency and duration on existing telemetry; (2) prototype disk-spill queue; (3) test under simulated partition.

## Technical Constraints

**Dependencies (upstream):** None (client-side feature)

**Dependencies (downstream):** T-2025 (persistent presence) — together they form the resilience layer. AEF dispatch reliability rests on this.

**ADR §9 boundary:** hard-dep per §9

## Scope Fence

**IN scope (this inception):** Validate that §6's description still holds in light of what's been learned from earlier primitives. Refine open questions into a design proposal. Recommend GO / NO-GO / DEFER with rationale. Surface any newly-discovered sub-decomposition.

**OUT of scope (this inception):** Build/code work — that's a follow-on task created on GO. Other primitives' shapes — they have their own tasks. AEF orchestration layer integration — that's the §9 collaboration seam, owned at the boundary.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Queue + reconnect implemented; partition-replay test passes; idempotency confirmed.

**NO-GO if:**
- Spike reveals existing transport assumptions that block clean reconnect; needs deeper rework.

**DEFER if:**
- Predecessor primitives have shifted shape in ways that change the open questions; capture the shift, update the ADR, re-file.

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

**Recommendation:** MOSTLY-SHIPPED — partial-GO on three small remaining gaps. The bulk of the primitive shipped under T-1439 before T-2023 was filed.

**Rationale (one-paragraph):** The §6 framing was correct at filing — there was no outbound queue. T-1439 then shipped one: `crates/termlink-session/src/offline_queue.rs` provides SQLite-backed durable queue (`pending_posts` table), `OfflineQueue::open/enqueue/pop`, `DEFAULT_CAP = 1000`, fail-loudly overflow via `QueueError::Full`, `attempts` counter for poison-pill detection, drain/flush task, and CLI integration in both `channel.rs` and `remote.rs`. IW-1 and IW-2 are fully resolved by that work. What remains: (A) idempotency via `client_msg_id` + hub-side LRU dedupe — the concrete double-apply scenario across hub blips is reproducible and the fix is small (~80 LOC); (B) audit + document the flush loop's backoff parameters; (C) write the operator-facing recipe doc. Three small follow-ups rather than re-shipping the bulk.

**Full analysis:** see [docs/reports/T-2023-client-reconnect-queue-inception.md](../../docs/reports/T-2023-client-reconnect-queue-inception.md).

**Build / follow-up tasks to file on GO:**

**Gap A — Idempotency (build task, ~80 LOC):**
- Client generates `client_msg_id` on every post (UUID v4 or content-hash + timestamp).
- Hub maintains short-TTL (~5 min) recently-seen LRU keyed by `(sender_fingerprint, client_msg_id)`.
- On duplicate: hub silently no-ops the second write, returns the original envelope's offset.
- Closes the double-apply gap across hub blips.

**Gap B — Backoff parameter audit (≤1 session, doc-only):**
- Locate flush loop implementation (somewhere downstream of T-1439's queue).
- Document initial delay, max delay, jitter, max attempts, dead-letter behavior.
- Conditional <50 LOC follow-up if any params are poor.

**Gap C — Operator recipe documentation (~50 lines docs):**
- Add offline-queue recipe to `docs/operations/`.
- Describe: how a CLI handles hub-blip, where the queue lives, how to inspect it, how poison-pill rows surface.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ✅ "Queue + reconnect implemented" — already shipped under T-1439.
- ⏸ "Partition-replay test passes" — needs Gap A + integration test.
- ⏸ "Idempotency confirmed" — exactly Gap A. Open.

**Why not full GO of T-2023 as captured:** the majority of the work is already shipped. A new build task duplicating that would just re-discover existing code. Three small follow-ups are honest about what's left.

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

**Decision**: GO

**Rationale**: Recommendation: MOSTLY-SHIPPED — partial-GO on three small remaining gaps. The bulk of the primitive shipped under T-1439 before T-2023 was filed.

Rationale (one-paragraph): The §6 framing was correct at filing — there was no outbound queue. T-1439 then shipped one: `crates/termlink-session/src/offline_queue.rs` provides SQLite-backed durable queue (`pending_posts` table), `OfflineQueue::open/enqueue/pop`, `DEFAULT_CAP = 1000`, fail-loudly overflow via `QueueError::Full`, `attempts` counter for poison-pill detection, drain/flush task, and CLI integration in both `channel.rs` and `remote.rs`. IW-1 and IW-2 are fully resolved by that work. What remains: (A) idempotency via `client_msg_id` + hub-side LRU dedupe — the concrete double-apply scenario across hub blips is reproducible and the fix is small (~80 LOC); (B) audit + document the flush loop's backoff parameters; (C) write the operator-facing recipe doc. Three small follow-ups rather than re-shipping the bulk.

Full analysis: see [docs/reports/T-2023-client-reconnect-queue-inception.md](../../docs/reports/T-2023-client-reconnect-queue-inception.md).

Build / follow-up tasks to file on GO:

Gap A — Idempotency (build task, ~80 LOC):
- Client generates `client_msg_id` on every post (UUID v4 or content-hash + timestamp).
- Hub maintains short-TTL (~5 min) recently-seen LRU keyed by `(sender_fingerprint, client_msg_id)`.
- On duplicate: hub silently no-ops the second write, returns the original envelope's offset.
- Closes the double-apply gap across hub blips.

Gap B — Backoff parameter audit (≤1 session, doc-only):
- Locate flush loop implementation (somewhere downstream of T-1439's queue).
- Document initial delay, max delay, jitter, max attempts, dead-letter behavior.
- Conditional <50 LOC follow-up if any params are poor.

Gap C — Operator recipe documentation (~50 lines docs):
- Add offline-queue recipe to `docs/operations/`.
- Describe: how a CLI handles hub-blip, where the queue lives, how to inspect it, how poison-pill rows surface.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ "Queue + reconnect implemented" — already shipped under T-1439.
- ⏸ "Partition-replay test passes" — needs Gap A + integration test.
- ⏸ "Idempotency confirmed" — exactly Gap A. Open.

Why not full GO of T-2023 as captured: the majority of the work is already shipped. A new build task duplicating that would just re-discover existing code. Three small follow-ups are honest about what's left.

**Date**: 2026-06-08T10:00:19Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:41:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T10:00:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: MOSTLY-SHIPPED — partial-GO on three small remaining gaps. The bulk of the primitive shipped under T-1439 before T-2023 was filed.

Rationale (one-paragraph): The §6 framing was correct at filing — there was no outbound queue. T-1439 then shipped one: `crates/termlink-session/src/offline_queue.rs` provides SQLite-backed durable queue (`pending_posts` table), `OfflineQueue::open/enqueue/pop`, `DEFAULT_CAP = 1000`, fail-loudly overflow via `QueueError::Full`, `attempts` counter for poison-pill detection, drain/flush task, and CLI integration in both `channel.rs` and `remote.rs`. IW-1 and IW-2 are fully resolved by that work. What remains: (A) idempotency via `client_msg_id` + hub-side LRU dedupe — the concrete double-apply scenario across hub blips is reproducible and the fix is small (~80 LOC); (B) audit + document the flush loop's backoff parameters; (C) write the operator-facing recipe doc. Three small follow-ups rather than re-shipping the bulk.

Full analysis: see [docs/reports/T-2023-client-reconnect-queue-inception.md](../../docs/reports/T-2023-client-reconnect-queue-inception.md).

Build / follow-up tasks to file on GO:

Gap A — Idempotency (build task, ~80 LOC):
- Client generates `client_msg_id` on every post (UUID v4 or content-hash + timestamp).
- Hub maintains short-TTL (~5 min) recently-seen LRU keyed by `(sender_fingerprint, client_msg_id)`.
- On duplicate: hub silently no-ops the second write, returns the original envelope's offset.
- Closes the double-apply gap across hub blips.

Gap B — Backoff parameter audit (≤1 session, doc-only):
- Locate flush loop implementation (somewhere downstream of T-1439's queue).
- Document initial delay, max delay, jitter, max attempts, dead-letter behavior.
- Conditional <50 LOC follow-up if any params are poor.

Gap C — Operator recipe documentation (~50 lines docs):
- Add offline-queue recipe to `docs/operations/`.
- Describe: how a CLI handles hub-blip, where the queue lives, how to inspect it, how poison-pill rows surface.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ "Queue + reconnect implemented" — already shipped under T-1439.
- ⏸ "Partition-replay test passes" — needs Gap A + integration test.
- ⏸ "Idempotency confirmed" — exactly Gap A. Open.

Why not full GO of T-2023 as captured: the majority of the work is already shipped. A new build task duplicating that would just re-discover existing code. Three small follow-ups are honest about what's left.
