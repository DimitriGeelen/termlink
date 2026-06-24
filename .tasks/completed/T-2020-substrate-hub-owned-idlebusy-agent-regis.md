---
id: T-2020
name: "Substrate: hub-owned idle/busy agent registry"
description: >
  §6 primitive 2 (Foundation, hard-dep). No hub-tracked agent state; no next-idle-worker-for-role-X;
  no in-flight counter. Heartbeats land in a topic and LIVE/STALE/OFFLINE classification
  is client-side. Orchestrator needs a reliable picture of who is free to assign safely.

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:20Z
last_update: 2026-06-08T07:03:25Z
date_finished: 2026-06-08T10:04:37Z
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

# T-2020: Substrate: hub-owned idle/busy agent registry

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Foundation. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Heartbeats land in a topic and LIVE/STALE/OFFLINE classification is purely client-side today. There is no hub-tracked agent state, no 'next idle worker for role X', no in-flight counter. The orchestrator cannot make safe assignment without a reliable picture of who is free.

**Why captured now:** T-2021 (pull/assign) needs this. Filing now while §6's intent is in view; design will reflect T-2019's claim shape and T-2025's persistence shape.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Hub-tracked vs server-side-derived from heartbeat topic?**
  confidence: 4
  disposition: answered
  rationale: DERIVE + future hub-side cache. New table duplicates state in `agent-presence` topic + `claims` table — two sources of truth = drift surface (makes IW-4 intractable). Append-log is the substrate's primary surface per ADR §2; a parallel agent_state table contradicts §5's "one writer, serialized" stance. Derivation `idle_agents = LIVE(presence) \ DISTINCT(claimed_by)` is O(presence) + O(claims) — both tiny at fleet scale (≤30 agents per §1). See docs/reports/T-2020-idle-busy-registry-inception.md §4.IW-1.

- **IW-2: Granularity — by agent_id, by role, by capability tag?**
  confidence: 4
  disposition: answered
  rationale: BOTH role AND capability, with capability as a structured array. Single-string role (today) is insufficient for "give me an idle worker that can build AND publish". Per T-1165 federate-don't-converge, metadata fields scale better than naming conventions. Add `metadata.capabilities: [string]` to heartbeat (backward-compat: missing = empty set). agent_id remains primary key. See artifact §4.IW-2.

- **IW-3: Update rate — pushed by worker on transition vs polled by hub on each assign?**
  confidence: 4
  disposition: answered
  rationale: PULL on each assign. Every `claim`/`release` already mutates the `claims` table — the hub can DERIVE busy/idle from that at read time. Pushing busy/idle transitions adds a write hot-path workers don't need. Pull-on-assign is also more failure-tolerant: registry is always consistent with current truth at query time. Future optimization: cache derived snapshot in hub memory with invalidation on `claim`/`release` — not needed for launch. See artifact §4.IW-3.

- **IW-4: Race resolution — worker says BUSY but hub thinks IDLE: who wins?**
  confidence: 4
  disposition: answered
  rationale: HUB WINS — orchestrator's view is authoritative because it's consistent across orchestrators while the worker's view is only consistent with itself. For the orchestration plane to be sound, the hub MUST be authoritative. Workers that disagree reconcile by releasing local state, not by overriding hub state. Edge case "worker holds claim but orchestrator marked it idle" is impossible because claims table is already in the derivation. See artifact §4.IW-4.

## Exploration Plan

After T-2019 lands. (1) Confirm whether claim implicitly tracks worker state — if so, registry may collapse to a derivation; (2) prototype the chosen approach; (3) measure update-rate at fleet scale (T-1991 precedent); (4) lock design.

## Technical Constraints

**Dependencies (upstream):** T-2025 (persistent presence) — registry is useless after a hub blip if liveness state evaporates

**Dependencies (downstream):** T-2021 (pull/assign must know who's idle to make a safe choice)

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
- Approach chosen and matches T-2019's claim semantics; scale measurement shows <linear traffic growth; build is bounded.

**NO-GO if:**
- Approach conflicts with claim shape; scale grows non-linearly with agent count (T-1991 redux); needs separate inception.

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

**Recommendation:** GO with revised scope (no new persistent table — derive from existing `agent-presence` topic + `claims` table).

**Rationale (one-paragraph):** The registry collapses to a DERIVATION + ONE QUERY VERB, not a new state surface. The substrate already has both data sources (presence topic for liveness + claims table for busy state); the missing piece is the server-side join. Adding a parallel `agent_state` table would duplicate state and create the drift surface that makes IW-4 intractable. Instead: ship `agent.find_idle(role?, capabilities?)` RPC that walks presence (filter to LIVE, apply role/capability predicate) and EXCLUDEs every agent_id in `SELECT DISTINCT claimed_by FROM claims WHERE claimed_until > now`. Extend the heartbeat envelope with `metadata.capabilities: [string]` (backward-compat: missing = empty set). Five small slices following T-2019's vertical pattern; estimated ~150 LOC + ≤1 session. No upstream blockers — T-2025 (persistent presence across restart) is a soft-dep (acceptable degradation: one-heartbeat-interval blackout post-restart). Hard-dep for AEF per §9.

**Full design + IW dispositions:** see [docs/reports/T-2020-idle-busy-registry-inception.md](../../docs/reports/T-2020-idle-busy-registry-inception.md).

**Build slice plan (mirrors T-2019 verticalization):**
- Slice 1: `agent.find_idle` RPC + bus library function + unit tests.
- Slice 2: CLI verb `termlink agent find-idle`.
- Slice 3: MCP tool `termlink_agent_find_idle`.
- Slice 4: Heartbeat schema extension (capabilities) + listener-heartbeat.sh update.
- Slice 5: Documentation + runnable example (orchestrator → find_idle → claim → release flow).
- (Optional Slice 6): hub-side derived-snapshot cache — defer until benchmarks demand.

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ✅ Approach chosen and matches T-2019's claim semantics (anti-joins on `claims.claimed_by`).
- ✅ Scale measurement: O(presence_topic_size) + O(claims_table_size). At fleet scale (≤30 agents per ADR §1), <10ms per call expected. T-1991's perf finding (per-binary-version, not topic-size) clarifies the bloat concern is retention/compaction (T-2028), not registry shape.
- ✅ Build is bounded: ~150 LOC, ≤1 session, 5 vertical slices.

**Open follow-up tasks to file on GO:**
- Build task for Slices 1-5.
- Heartbeat schema migration coordination task (consumers + AEF).

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

**Rationale**: Recommendation: GO with revised scope (no new persistent table — derive from existing `agent-presence` topic + `claims` table).

Rationale (one-paragraph): The registry collapses to a DERIVATION + ONE QUERY VERB, not a new state surface. The substrate already has both data sources (presence topic for liveness + claims table for busy state); the missing piece is the server-side join. Adding a parallel `agent_state` table would duplicate state and create the drift surface that makes IW-4 intractable. Instead: ship `agent.find_idle(role?, capabilities?)` RPC that walks presence (filter to LIVE, apply role/capability predicate) and EXCLUDEs every agent_id in `SELECT DISTINCT claimed_by FROM claims WHERE claimed_until > now`. Extend the heartbeat envelope with `metadata.capabilities: [string]` (backward-compat: missing = empty set). Five small slices following T-2019's vertical pattern; estimated ~150 LOC + ≤1 session. No upstream blockers — T-2025 (persistent presence across restart) is a soft-dep (acceptable degradation: one-heartbeat-interval blackout post-restart). Hard-dep for AEF per §9.

Full design + IW dispositions: see [docs/reports/T-2020-idle-busy-registry-inception.md](../../docs/reports/T-2020-idle-busy-registry-inception.md).

Build slice plan (mirrors T-2019 verticalization):
- Slice 1: `agent.find_idle` RPC + bus library function + unit tests.
- Slice 2: CLI verb `termlink agent find-idle`.
- Slice 3: MCP tool `termlink_agent_find_idle`.
- Slice 4: Heartbeat schema extension (capabilities) + listener-heartbeat.sh update.
- Slice 5: Documentation + runnable example (orchestrator → find_idle → claim → release flow).
- (Optional Slice 6): hub-side derived-snapshot cache — defer until benchmarks demand.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ Approach chosen and matches T-2019's claim semantics (anti-joins on `claims.claimed_by`).
- ✅ Scale measurement: O(presence_topic_size) + O(claims_table_size). At fleet scale (≤30 agents per ADR §1), <10ms per call expected. T-1991's perf finding (per-binary-version, not topic-size) clarifies the bloat concern is retention/compaction (T-2028), not registry shape.
- ✅ Build is bounded: ~150 LOC, ≤1 session, 5 vertical slices.

Open follow-up tasks to file on GO:
- Build task for Slices 1-5.
- Heartbeat schema migration coordination task (consumers + AEF).

**Date**: 2026-06-08T09:59:44Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T06:57:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T09:59:44Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO with revised scope (no new persistent table — derive from existing `agent-presence` topic + `claims` table).

Rationale (one-paragraph): The registry collapses to a DERIVATION + ONE QUERY VERB, not a new state surface. The substrate already has both data sources (presence topic for liveness + claims table for busy state); the missing piece is the server-side join. Adding a parallel `agent_state` table would duplicate state and create the drift surface that makes IW-4 intractable. Instead: ship `agent.find_idle(role?, capabilities?)` RPC that walks presence (filter to LIVE, apply role/capability predicate) and EXCLUDEs every agent_id in `SELECT DISTINCT claimed_by FROM claims WHERE claimed_until > now`. Extend the heartbeat envelope with `metadata.capabilities: [string]` (backward-compat: missing = empty set). Five small slices following T-2019's vertical pattern; estimated ~150 LOC + ≤1 session. No upstream blockers — T-2025 (persistent presence across restart) is a soft-dep (acceptable degradation: one-heartbeat-interval blackout post-restart). Hard-dep for AEF per §9.

Full design + IW dispositions: see [docs/reports/T-2020-idle-busy-registry-inception.md](../../docs/reports/T-2020-idle-busy-registry-inception.md).

Build slice plan (mirrors T-2019 verticalization):
- Slice 1: `agent.find_idle` RPC + bus library function + unit tests.
- Slice 2: CLI verb `termlink agent find-idle`.
- Slice 3: MCP tool `termlink_agent_find_idle`.
- Slice 4: Heartbeat schema extension (capabilities) + listener-heartbeat.sh update.
- Slice 5: Documentation + runnable example (orchestrator → find_idle → claim → release flow).
- (Optional Slice 6): hub-side derived-snapshot cache — defer until benchmarks demand.

GO criteria evaluation (from §Go/No-Go Criteria):
- ✅ Approach chosen and matches T-2019's claim semantics (anti-joins on `claims.claimed_by`).
- ✅ Scale measurement: O(presence_topic_size) + O(claims_table_size). At fleet scale (≤30 agents per ADR §1), <10ms per call expected. T-1991's perf finding (per-binary-version, not topic-size) clarifies the bloat concern is retention/compaction (T-2028), not registry shape.
- ✅ Build is bounded: ~150 LOC, ≤1 session, 5 vertical slices.

Open follow-up tasks to file on GO:
- Build task for Slices 1-5.
- Heartbeat schema migration coordination task (consumers + AEF).
