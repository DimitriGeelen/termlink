---
id: T-2026
name: "Substrate: typed agent-launch surface (checkout/commit/publish RPCs)"
description: >
  §6 primitive 8 (Contract, hard-dep). Today the only git awareness is the dispatch
  --isolate worktree wrapper. Coordinating code-plane vs governance-plane is otherwise
  a shell convention sitting above TermLink. Typed agent.checkout(ref) / agent.commit(scope)
  / agent.publish(branch) makes the convention a first-class substrate concept the
  orchestrator can rely on.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-parallel-substrate]
components: []
related_tasks: [T-2018]
created: 2026-06-07T11:36:46Z
last_update: 2026-10-18T20:40:35Z
date_finished:
revisit_at: 2026-09-08            # T-1451: DEFER until Foundation primitives (T-2019/T-2021) ship and stabilize
revisit_evidence_needed: "Foundation primitives (T-2019, T-2020, T-2021, T-2027) shipped
  and in AEF use; ≥1 concrete incident where shell-convention git ops produced an
  integration gap typed RPCs would have caught."
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
  - ts: '2026-09-08T21:30:27Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-08T21:30:38Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 8
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=8 (lines=218,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2026: Substrate: typed agent-launch surface (checkout/commit/publish RPCs)

## Problem Statement

§6 primitive of [arc-parallel-substrate](../../docs/architecture/parallel-execution-substrate.md). **Group:** Contract. **§9 boundary:** hard-dep per §9.

**Role per ADR §6:** Today the only git awareness in TermLink is the `dispatch --isolate` worktree wrapper. Coordinating code-plane vs governance-plane is otherwise a shell convention sitting above the substrate. Typed `agent.checkout(ref)` / `agent.commit(scope)` / `agent.publish(branch)` turns the convention into a substrate concept the orchestrator can rely on.

**Why captured now:** Foundation must land first to inform the RPC shape (especially claim semantics for ledger-write serialization). Filing now while §6's intent and §5's plane-split rationale are in view.

**Status disclosure ([[PL-203]]):** Filed `status: captured, horizon: later`. BVP scores will be estimator-proposed (not confirmed) until promotion. No commitment to RPC shape, build order, or cost-estimate is locked at filing. Per-primitive design phase opens at operator promotion via `fw bvp confirm` + `--horizon now`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Exact RPC signatures — return shape for checkout/commit/publish?**
  confidence: 3
  disposition: deferred
  rationale: Deferred to T-3469, the first build slice (T-2026 GO, 2026-10-18): the June blocker is gone (Foundation shipped), so the shape is now lockable; the slice locks it from the June sketch (report §5) against the shipped claim API before any verb is built. June text: GENUINELY deferred. Return shapes couple to Foundation primitive semantics (T-2019 claim_id vs T-2021 assignment_id as scope handle). Provisional sketch in docs/reports/T-2026-typed-agent-launch-inception.md §5; do not lock until Foundation in production. See artifact §4.a.

- **IW-2: Scope binding — how is 'this agent's worktree' identified — by agent_id, session_id, or claim_id?**
  confidence: 4
  disposition: answered
  rationale: `claim_id` (T-2019), with `channel claim-transfer` (T-2021/T-2046) as the orchestrator → worker handoff: both shipped in June and are in production, which was the June condition. June text: LIKELY `claim_id` from T-2019 (with T-2021's transfer_claim providing the orchestrator → worker handoff path). But final answer hinges on whether T-2021 lands as proposed or with shape changes. See artifact §6.IW-2.

- **IW-3: Worktree lifecycle — owned by hub (current dispatch --isolate model) or owned by spoke (more decentralized)?**
  confidence: 2
  disposition: deferred
  rationale: Deferred to T-3469, the first build slice, where it comes back to the operator as one decision brief: hub-owned vs spoke-owned, and how the surface relates to the operator's standing rule that agents in this repo never use git worktrees (the rule governs this repo's own sessions; the surface is for orchestrated workers fleet-wide, but that boundary must be stated, not assumed). June text: GENUINELY deferred. Hub-owned simplifies merge but couples hosts; spoke-owned matches hub-and-spoke topology but requires worktree-creation primitives on spoke side. Choice hinges on observed cross-host AEF behavior that doesn't exist yet. See artifact §4.b.

- **IW-4: Un-partitionable file handling — §5 says hub-owned regeneration after merge. Does THIS surface expose that, or is it separate?**
  confidence: 4
  disposition: answered
  rationale: SEPARATE (unchanged by the GO): regenerating machine-maintained files after a merge is not a launch-surface concern. June text: SEPARATE (likely primitive #11). Un-partitionable file regeneration is a §5 concern about machine-generated state that doesn't merge cleanly; the typed-launch verbs are about ordinary agent commits. Splitting keeps T-2026 small and surface-focused. See artifact §6.IW-4.

## Exploration Plan

After Foundation primitives land. Design phase reflects their actual shape, not the §6 sketch.

## Technical Constraints

**Dependencies (upstream):** T-2019 (claim — for ledger-write serialization)

**Dependencies (downstream):** AEF orchestrator dispatcher builds against this surface

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
- Signatures locked; lifecycle decided; un-partitionable-file path is either in scope or split.

**NO-GO if:**
- Foundation primitive shapes contradict each other on this seam; need to re-contract at §9.

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

**Recommendation:** GO (operator decision 2026-10-18T20:40:34Z via Watchtower; the operator then confirmed it was intended: "We are now focused on orchestration ... parallelism will be more in vogue. Let's prepare.")

**Rationale:** The June blocker is gone: the Foundation primitives the RPC shape depended on (claim, claim-transfer, broadcast-with-replay) have been in production since June, so the surface can now be designed against real semantics. The operator's direction is orchestration, and parallel orchestrated work is expected to grow; a typed checkout/commit/publish surface turns the current shell convention into something an orchestrator can rely on and audit per agent. The agent's brief recommended NO-GO for lack of a current consumer; the operator weighed that against the orchestration direction and chose GO. The build starts with a design slice that locks the RPC shape and brings the worktree-ownership question (IW-3) back to the operator.

**Context the agent's NO-GO brief rested on (kept for the record, outweighed by the operator):**
- Foundation done: T-2019 (claim), T-2021 (claim-transfer), T-2027 (broadcast-with-replay) are in .tasks/completed/.
- No consumer: no reference to agent.checkout / agent.commit / agent.publish in crates/, scripts/ or the vendored framework (grep, 2026-10-18).
- Existing worktree path stays: `termlink dispatch --isolate` (crates/termlink-cli/src/cli.rs:887).
- T-2022 NO-GO (2026-10-18) removed the path-declaration story this surface would have fed.

**June filing (superseded):** waited on the Foundation primitives T-2019, T-2021 and T-2027; all three have since shipped and are completed.

**Rationale (one-paragraph):** Typed RPCs are a real ergonomics + integration win — structured return shape feeds T-2022's path-declaration story, audit trail per agent, and could auto-release claims when commits succeed. But the verbs' return shapes COUPLE to Foundation primitive semantics: `agent.commit --scope X` needs to know what `scope` means, which depends on whether T-2021 ships `channel.transfer_claim` (likely) or a different shape. Worktree lifecycle (hub-owned vs spoke-owned, IW-3) hinges on observed cross-host AEF behavior that doesn't exist yet. Two of four IW questions are genuinely undecidable without Foundation in production. Designing now risks throwing the design away. The shell-convention path still works for current dispatch model — this is ergonomics, not a blocking gap. Other primitives have higher marginal value at the current point in the build curve.

**Full analysis:** see [docs/reports/T-2026-typed-agent-launch-inception.md](../../docs/reports/T-2026-typed-agent-launch-inception.md).

**Provisional sketch (for orientation only — do NOT lock or build against):**

```
agent.checkout(target_ref, in_worktree?) → {head, dirty}
agent.commit(scope_id, message?, paths?) → {commit_hash, paths_modified, paths_added, paths_deleted}
agent.publish(branch, remote?) → {published, remote_ref, error?}
```

- `scope_id` = `claim_id` from T-2019 (likely, contingent on T-2021 GO)
- Each verb posts structured event to per-agent audit topic
- `agent.commit` optionally fires `channel.release` on the named scope (one-step done-and-release)
- Sketch only — final shape locks at revisit with Foundation in production

**GO criteria evaluation (from §Go/No-Go Criteria):**
- ⏸ "Signatures locked" — deferred, depend on Foundation.
- ⏸ "Lifecycle decided" — deferred, needs cross-host AEF evidence.
- ⏸ "Un-partitionable-file path in scope or split" — provisionally SPLIT (file as primitive #11 if needed).

**Open follow-up tasks to file:**
- *(At revisit 2026-09-08, conditional on Foundation evidence)* Build task: typed `agent.checkout` + `agent.commit` + `agent.publish` RPCs (~200 LOC, 4-5 vertical slices).
- *(Conditional)* Inception for primitive #11: un-partitionable file handling per §5.

## Decisions

### 2026-10-18 — GO (operator, supersedes the NO-GO ruling below)
- **Chose:** GO, recorded 20:40:34Z via Watchtower and confirmed in chat: "I thought that it would be a go. We are now focused on orchestration and that kind of thing. I think that parallelism will be more in swan [in vogue]. Let's prepare."
- **Why:** orchestration is the current direction and parallel orchestrated work is expected to grow; Foundation has shipped, so the shape can be designed against real semantics.
- **Next:** a design slice first (lock the RPC shape; IW-3 worktree ownership and its relation to the no-worktrees rule as an operator decision brief), then the verbs.

### 2026-10-18 — NO-GO (operator ruling, T-3440 brief, option A) — SUPERSEDED by the GO above
- **Chose:** A. Do not build typed agent.checkout/commit/publish. `dispatch --isolate` and the shell convention stay; a future need starts as a new inception.
- **Why:** decidable now (Foundation shipped), but no consumer exists and the worktree-per-agent model it serves is not how parallel work runs here.
- **Rejected:** B (build now: no consumer to validate the shape, built on worktrees the operator forbids here), C (DEFER again: a NO-GO keeps the door open just as well).

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

**Rationale**: The June blocker is gone (the Foundation primitives have been in production since June), so the question is decidable now, and the answer is no: nothing asks for typed checkout/commit/publish. Parallel work runs differently from the worktree-per-agent model this surface was designed for: AEF decides up front with `fw write-set check` and runs workers in the checkout, and this repo does not use worktrees by operator rule. An API with no consumer cannot have its shape validated. If an orchestrator later needs it, that starts as a new inception from its actual need.

Typed RPCs are a real ergonomics + integration win — structured return shape feeds T-2022's path-declaration story, audit trail per agent, and could auto-release claims when commits succeed. But the verbs' return shapes COUPLE to Foundation primitive semantics: `agent.commit --scope X` needs to know what `scope` means, which depends on whether T-2021 ships `channel.transfer_claim` (likely) or a different shape. Worktree lifecycle (hub-owned vs spoke-owned, IW-3) hinges on observed cross-host AEF behavior that doesn't exist yet. Two of four IW questions are genuinely undecidable without Foundation in production. Designing now risks throwing the design away. The shell-convention path still works for current dispatch model — this is ergonomics, not a blocking gap. Other primitives have higher marginal value at the current point in the build curve.

**Date**: 2026-10-18T20:40:34Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-08T07:48:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-08T11:21:11Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER until Foundation primitives (T-2019, T-2021, T-2027) ship and stabilize. revisit_at=2026-09-08.

Rationale (one-paragraph): Typed RPCs are a real ergonomics + integration win — structured return shape feeds T-2022's path-declaration story, audit trail per agent, and could auto-release claims when commits succeed. But the verbs' return shapes COUPLE to Foundation primitive semantics: `agent.commit --scope X` needs to know what `scope` means, which depends on whether T-2021 ships `channel.transfer_claim` (likely) or a different shape. Worktree lifecycle (hub-owned vs spoke-owned, IW-3) hinges on observed cross-host AEF behavior that doesn't exist yet. Two of four IW questions are genuinely undecidable without Foundation in production. Designing now risks throwing the design away. The shell-convention path still works for current dispatch model — this is ergonomics, not a blocking gap. Other primitives have higher marginal value at the current point in the build curve.

Full analysis: see [docs/reports/T-2026-typed-agent-launch-inception.md](../../docs/reports/T-2026-typed-agent-launch-inception.md).

Provisional sketch (for orientation only — do NOT lock or build against):

```
agent.checkout(target_ref, in_worktree?) → {head, dirty}
agent.commit(scope_id, message?, paths?) → {commit_hash, paths_modified, paths_added, paths_deleted}
agent.publish(branch, remote?) → {published, remote_ref, error?}
```

- `scope_id` = `claim_id` from T-2019 (likely, contingent on T-2021 GO)
- Each verb posts structured event to per-agent audit topic
- `agent.commit` optionally fires `channel.release` on the named scope (one-step done-and-release)
- Sketch only — final shape locks at revisit with Foundation in production

GO criteria evaluation (from §Go/No-Go Criteria):
- ⏸ "Signatures locked" — deferred, depend on Foundation.
- ⏸ "Lifecycle decided" — deferred, needs cross-host AEF evidence.
- ⏸ "Un-partitionable-file path in scope or split" — provisionally SPLIT (file as primitive #11 if needed).

Open follow-up tasks to file:
- (At revisit 2026-09-08, conditional on Foundation evidence) Build task: typed `agent.checkout` + `agent.commit` + `agent.publish` RPCs (~200 LOC, 4-5 vertical slices).
- (Conditional) Inception for primitive #11: un-partitionable file handling per §5.

### 2026-06-08T11:21:11Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)
- **Reason:** Inception decision: DEFER — parking task

### 2026-10-18T20:40:34Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** The June blocker is gone (the Foundation primitives have been in production since June), so the question is decidable now, and the answer is no: nothing asks for typed checkout/commit/publish. Parallel work runs differently from the worktree-per-agent model this surface was designed for: AEF decides up front with `fw write-set check` and runs workers in the checkout, and this repo does not use worktrees by operator rule. An API with no consumer cannot have its shape validated. If an orchestrator later needs it, that starts as a new inception from its actual need.

Typed RPCs are a real ergonomics + integration win — structured return shape feeds T-2022's path-declaration story, audit trail per agent, and could auto-release claims when commits succeed. But the verbs' return shapes COUPLE to Foundation primitive semantics: `agent.commit --scope X` needs to know what `scope` means, which depends on whether T-2021 ships `channel.transfer_claim` (likely) or a different shape. Worktree lifecycle (hub-owned vs spoke-owned, IW-3) hinges on observed cross-host AEF behavior that doesn't exist yet. Two of four IW questions are genuinely undecidable without Foundation in production. Designing now risks throwing the design away. The shell-convention path still works for current dispatch model — this is ergonomics, not a blocking gap. Other primitives have higher marginal value at the current point in the build curve.

### 2026-10-18T20:40:35Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
- **Reason:** Inception decision in progress
