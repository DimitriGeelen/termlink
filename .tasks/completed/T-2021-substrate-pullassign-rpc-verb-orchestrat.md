---
id: T-2021
name: "Substrate: pull/assign RPC verb (orchestrator-to-worker handoff)"
description: >
  §6 primitive 3 (Foundation, hard-dep). Every existing path is push (sender picks
  recipient). No give-me-the-next-unit RPC and no clean inverse for the orchestrator
  to hand a specific unit to a specific worker as a first-class operation.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:24Z
last_update: 2026-06-08T07:28:13Z
date_finished: 2026-06-08T10:04:49Z
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

# T-2021: Substrate: pull/assign RPC verb (orchestrator-to-worker handoff)

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Foundation. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Every existing TermLink path is push (sender picks recipient). There is no 'give me the next unit' RPC and no clean inverse for the orchestrator to hand a specific unit to a specific worker as a first-class operation.

**Why captured now:** Build order tail of Foundation. Fully informed by T-2019 + T-2020 once they land.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Push (orchestrator → worker) vs pull (worker requests next) vs both?**
  confidence: 4
  disposition: answered
  rationale: BOTH, but they cost different things. Pull = ZERO new primitives — pure composition of `channel.subscribe` + `channel.claim` (the hub serializes claim attempts, so competing workers all subscribe + claim, whoever wins on offset N processes, others skip to N+1). Push/assign = ONE new RPC `channel.transfer_claim` (atomic ownership transfer). The §6 framing implies both are missing; in practice pull was already there post-T-2019, and assign needs one verb. See docs/reports/T-2021-pull-assign-rpc-inception.md §3, §4.

- **IW-2: Worker selection policy — round-robin, least-loaded, capability-match — hub-side or orchestrator-side?**
  confidence: 4
  disposition: answered
  rationale: ORCHESTRATOR-SIDE. The substrate provides the FILTER (`agent.find_idle` per T-2020 returns LIVE-and-not-busy agents matching role + capabilities). The CHOICE among those candidates is policy and lives in AEF. Keeps the substrate minimal per ADR §4 boundary — substrate ships capabilities, AEF picks policy. Same shape as the conservative/optimistic decision: substrate offers the verb, AEF makes the call. See artifact §6.IW-2.

- **IW-3: Failure mode — assignment unacked within N seconds → reclaim and reassign?**
  confidence: 4
  disposition: answered
  rationale: SOLVED BY T-2019'S LEASE. Orchestrator's `channel.claim --leased <ttl>` IS the failure-mode mechanism. If the assignment envelope is ignored or the target worker dies, the claim auto-expires and the slot returns to the queue. No new reclaim mechanism, no new timer thread, no new state. The lease was designed for exactly this. See artifact §5.

- **IW-4: Is this a new RPC or a composition of (subscribe + claim + ack)?**
  confidence: 4
  disposition: answered
  rationale: BOTH, by mode. Pull is composition (subscribe + claim, zero new code). Assign needs `channel.transfer_claim` — a single new RPC that atomically reassigns claim ownership from orchestrator to worker (a strict generalization of release-then-claim, with the race window removed). Why not just release-then-claim? Because two writes are non-atomic; a second worker can race in. Why not `force_release` + claim? Because force_release bypasses ownership (operator-intervention verb, T-2044); transfer is cooperative and owner-checked. See artifact §4.

## Exploration Plan

After T-2019 + T-2020 land. Likely starts as a composition spike: does (claim + ack) already give us pull/assign for free? If yes, no new RPC; if no, design the minimum new verb.

## Technical Constraints

**Dependencies (upstream):** T-2019 (claim makes assignment exclusive), T-2020 (registry says who's idle)

**Dependencies (downstream):** AEF orchestrator dispatcher — without this verb, the orchestrator pattern is undefined

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
- Composition decision is final; AEF orchestrator can build against the resulting interface; failure mode is named and bounded.

**NO-GO if:**
- Composition route reveals a substrate gap that needs a separate primitive; interface is unstable.

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

**Recommendation:** GO with revised scope (ship `channel.transfer_claim` only; pull is already a pure composition of existing verbs).

**Rationale (one-paragraph):** The §6 framing implies a complex two-way RPC surface; in practice post-T-2019, **pull is already a pure composition** (subscribe + claim, hub serializes, leases catch failures — no new primitive needed) and **assign needs exactly one new RPC** (`channel.transfer_claim(claim_id, to_owner, by, reason?)` — atomic ownership transfer of an existing claim). That verb closes the only remaining gap: orchestrator's claim is owner-bound, so naive release-then-claim leaves a race window; force_release (T-2044) bypasses ownership and is operator-only. Transfer is the cooperative, owner-checked sibling — strict generalization of release-then-claim with the race window removed. Failure modes are inherited from T-2019's lease (no new timer state). Build is ~120 LOC across 4 vertical slices, mirroring T-2044's plumbing. No upstream blockers — T-2019 shipped, T-2020 recommended GO covers the `agent.find_idle` discovery half.

**Full design + IW dispositions:** see [docs/reports/T-2021-pull-assign-rpc-inception.md](../../docs/reports/T-2021-pull-assign-rpc-inception.md).

**Build slice plan (mirrors T-2019 / T-2044 verticalization):**
- Slice 1: `channel.transfer_claim` bus library function + atomic UPDATE in claims table + unit tests (by-mismatch, expired, not-found, happy path).
- Slice 2: Hub handler in `crates/termlink-hub/src/channel.rs` + router allow-list + protocol constant + error-code wiring.
- Slice 3: CLI verb `termlink channel claim-transfer --claim-id C --to-owner W [--reason "..."]` + session-client wrapper + JSON envelope.
- Slice 4: MCP tool `termlink_channel_claim_transfer` + help-registry entry + docs in `docs/operations/substrate-claim-primitive.md` showing the orchestrator → worker assign recipe end-to-end.
- (Optional Slice 5): Pull-recipe documentation — no code, just the worker-loop incantation. Could roll into Slice 4.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ✅ Composition decision is final (pull = composition, assign = `transfer_claim` + envelope convention).
- ✅ AEF orchestrator can build against the resulting interface — five named verbs, concrete payload shape, error codes inherited from T-2019.
- ✅ Failure mode is named and bounded — lease expiry handles unacked assignments; `force_release` handles orchestrator crash mid-handoff; `transfer_claim` itself is atomic.

**Open follow-up tasks to file on GO:**
- Build task for Slices 1-4 (`channel.transfer_claim` end-to-end).
- AEF-side integration task: orchestrator dispatcher pattern over the new verb (not substrate-owned; for the §9 collaboration seam).

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

**Rationale**: Recommendation: GO with revised scope (ship `channel.transfer_claim` only; pull is already a pure composition of existing verbs).

Rationale (one-paragraph): The §6 framing implies a complex two-way RPC surface; in practice post-T-2019, pull is already a pure composition (subscribe + claim, hub serializes, leases catch failures — no new primitive needed) and assign needs exactly one new RPC (`channel.transfer_claim(claim_id, to_owner, by, reason?)` — atomic ownership transfer of an existing claim). That verb closes the only remaining gap: orchestrator's claim is owner-bound, so naive release-then-claim leaves a race window; force_release (T-2044) bypasses ownership and is operator-only. Transfer is the cooperative, owner-checked sibling — strict generalization of release-then-claim with the race window removed. Failure modes are inherited from T-2019's lease (no new timer state). Build is ~120 LOC across 4 vertical slices, mirroring T-2044's plumbing. No upstream blockers — T-2019 shipped, T-2020 recommended GO covers the `agent.find_idle` discovery half.

Full design + IW dispositions: see [docs/reports/T-2021-pull-assign-rpc-inception.md](../../docs/reports/T-2021-pull-assign-rpc-inception.md).

Build slice plan (mirrors T-2019 / T-2044 verticalization):
- Slice 1: `channel.transfer_claim` bus library function + atomic UPDATE in claims table + unit tests (by-mismatch, expired, not-found, happy path).
- Slice 2: Hub handler in `crates/termlink-hub/src/channel.rs` + router allow-list + protocol constant + error-code wiring.
- Slice 3: CLI verb `termlink channel claim-transfer --claim-id C --to-owner W [--reason "..."]` + session-client wrapper + JSON envelope.
- Slice 4: MCP tool `termlink_channel_claim_transfer` + help-registry entry + docs in `docs/operations/substrate-claim-primitive.md` showing the orchestrator → worker assign recipe end-to-end.
- (Optional Slice 5): Pull-recipe documentation — no code, just the worker-loop incantation. Could roll into Slice 4.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ Composition decision is final (pull = composition, assign = `transfer_claim` + envelope convention).
- ✅ AEF orchestrator can build against the resulting interface — five named verbs, concrete payload shape, error codes inherited from T-2019.
- ✅ Failure mode is named and bounded — lease expiry handles unacked assignments; `force_release` handles orchestrator crash mid-handoff; `transfer_claim` itself is atomic.

Open follow-up tasks to file on GO:
- Build task for Slices 1-4 (`channel.transfer_claim` end-to-end).
- AEF-side integration task: orchestrator dispatcher pattern over the new verb (not substrate-owned; for the §9 collaboration seam).

**Date**: 2026-06-08T09:59:56Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:25:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T09:59:56Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO with revised scope (ship `channel.transfer_claim` only; pull is already a pure composition of existing verbs).

Rationale (one-paragraph): The §6 framing implies a complex two-way RPC surface; in practice post-T-2019, pull is already a pure composition (subscribe + claim, hub serializes, leases catch failures — no new primitive needed) and assign needs exactly one new RPC (`channel.transfer_claim(claim_id, to_owner, by, reason?)` — atomic ownership transfer of an existing claim). That verb closes the only remaining gap: orchestrator's claim is owner-bound, so naive release-then-claim leaves a race window; force_release (T-2044) bypasses ownership and is operator-only. Transfer is the cooperative, owner-checked sibling — strict generalization of release-then-claim with the race window removed. Failure modes are inherited from T-2019's lease (no new timer state). Build is ~120 LOC across 4 vertical slices, mirroring T-2044's plumbing. No upstream blockers — T-2019 shipped, T-2020 recommended GO covers the `agent.find_idle` discovery half.

Full design + IW dispositions: see [docs/reports/T-2021-pull-assign-rpc-inception.md](../../docs/reports/T-2021-pull-assign-rpc-inception.md).

Build slice plan (mirrors T-2019 / T-2044 verticalization):
- Slice 1: `channel.transfer_claim` bus library function + atomic UPDATE in claims table + unit tests (by-mismatch, expired, not-found, happy path).
- Slice 2: Hub handler in `crates/termlink-hub/src/channel.rs` + router allow-list + protocol constant + error-code wiring.
- Slice 3: CLI verb `termlink channel claim-transfer --claim-id C --to-owner W [--reason "..."]` + session-client wrapper + JSON envelope.
- Slice 4: MCP tool `termlink_channel_claim_transfer` + help-registry entry + docs in `docs/operations/substrate-claim-primitive.md` showing the orchestrator → worker assign recipe end-to-end.
- (Optional Slice 5): Pull-recipe documentation — no code, just the worker-loop incantation. Could roll into Slice 4.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ Composition decision is final (pull = composition, assign = `transfer_claim` + envelope convention).
- ✅ AEF orchestrator can build against the resulting interface — five named verbs, concrete payload shape, error codes inherited from T-2019.
- ✅ Failure mode is named and bounded — lease expiry handles unacked assignments; `force_release` handles orchestrator crash mid-handoff; `transfer_claim` itself is atomic.

Open follow-up tasks to file on GO:
- Build task for Slices 1-4 (`channel.transfer_claim` end-to-end).
- AEF-side integration task: orchestrator dispatcher pattern over the new verb (not substrate-owned; for the §9 collaboration seam).
