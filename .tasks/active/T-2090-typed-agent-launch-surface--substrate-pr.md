---
id: T-2090
name: "Typed agent-launch surface — substrate primitive 8"
description: >
  Inception: Typed agent-launch surface — substrate primitive 8

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-09T14:51:17Z
last_update: 2026-06-09T14:51:27Z
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

# T-2090: Typed agent-launch surface — substrate primitive 8

## Problem Statement

ADR §6 #8 (Contract tier) calls for typed `agent.checkout(ref)` / `agent.commit(scope)` / `agent.publish(branch)` verbs to elevate the existing `dispatch --isolate` worktree convention into a substrate primitive. Today: orchestrators use `termlink dispatch --isolate --auto-merge` (`crates/termlink-cli/src/commands/dispatch.rs:36`, `crate::manifest::create_worktree` at manifest.rs:182, `merge_branch` at manifest.rs:287) — a working shell convention, but the substrate has no concept of "an agent owns this ref" or "this branch is a parallel work scope". The question is whether elevating the convention to typed verbs delivers substrate value beyond what `dispatch --isolate` already does.

See `docs/reports/T-2090-typed-agent-launch-surface-inception.md` for design analysis.

## Assumptions

- A1: The existing `dispatch --isolate` + `--auto-merge` already covers the orchestrator's checkout→work→merge happy path (verified in manifest.rs:182,287).
- A2: An orchestrator wanting agent-branch-lifecycle visibility today must track it client-side (no substrate state).
- A3: Adding typed CLI/MCP verbs WITHOUT substrate-tracked state is cheap (~3 thin wrappers). Adding substrate-tracked branch-claim state is substantial (new schema, new claim invariant, new failure modes).

## Open Questions

- **IW-1: Are typed verbs without substrate state still valuable?**
  confidence: 2
  disposition: answered
  rationale: Marginally — typed verbs give MCP-callable substrate primitives that orchestrating agents can rely on by contract rather than shell-string concatenation. But the value is "convention-as-API", not "new substrate capability". Borderline vs. just documenting the existing dispatch verbs.

- **IW-2: Does branch-agent-lifecycle need substrate-tracked state, or can the orchestrator track it?**
  confidence: 2
  disposition: answered
  rationale: The orchestrator CAN track it (client-side map of branch→agent_id). Substrate-tracked state ADDS a "one agent per ref at a time" invariant that the orchestrator can rely on. This is a generalization of T-2019 CLAIM semantics — same shape, different keying. Not strictly necessary but cleaner if a load-bearing consumer demands it.

- **IW-3: Does agent.commit(scope) need new semantics beyond git commit?**
  confidence: 2
  disposition: answered
  rationale: `scope` would be a typed boundary (e.g., "commit only files matching this glob"). Today the agent runs raw `git commit -m`. The substrate value of typed scope is enforcement — preventing an agent from accidentally committing outside its scope. Without enforcement, it's just a label and adds no value over a convention.

- **IW-4: Does this duplicate CLAIM (T-2019) or generalize it?**
  confidence: 3
  disposition: answered
  rationale: CLAIM is keyed by `(topic, offset)`. An agent-branch claim would be keyed by `branch_ref`. Same exclusive-ownership shape, different keying. Generalization, not duplication. If T-2089 (cv_index) ships first, that's the third instance of "in-memory hub map keyed by something" — argues for a generic keyed-claim primitive instead of three special cases.

- **IW-5: Is there a current consumer demanding #8?**
  confidence: 3
  disposition: answered
  rationale: No identified orchestrator today uses `dispatch --isolate --auto-merge` at a scale where shell-string vs typed-verb is the bottleneck. AEF Workflow agents use it inline. Without a load-bearing consumer, building typed verbs is speculative scaffolding.

## Exploration Plan

Inception artifact (docs/reports/T-2090-typed-agent-launch-surface-inception.md):
- §1 Problem framing — what `dispatch --isolate` already does vs. what typed verbs would add
- §2 Four design alternatives (A: thin typed CLI wrappers, B: substrate-tracked branch-claim, C: MCP-only typed surface, D: documentation-only)
- §3 Recommendation
- §4 Slice plan IF GO

No code spike. The existing `dispatch.rs:36-200` is the reference implementation. Comparison is analytical.

## Technical Constraints

- Existing surface: `dispatch --isolate` (worktree create) + `--auto-merge` (post-completion merge-back). Both bounded by git semantics.
- Branch-claim state, if added, would mirror T-2019 CLAIM keying — in-memory hub map, restart-safe via O(N) scan of historical envelopes.
- Backward compatibility: new verbs MUST not break the existing `dispatch --isolate` consumer surface.
- Cardinality: bounded by active branches per repo (typically 1-20 in a homelab).

## Scope Fence

**IN:** Inception decision (GO/NO-GO/DEFER), design alternatives, slice plan structure if GO.

**OUT (build artifacts — only after GO):**
- Typed CLI verbs `termlink agent checkout/commit/publish`
- MCP parity tools
- Substrate-tracked branch-claim state (if Design B chosen)
- Migration from existing `dispatch --isolate` consumers

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

**GO if:**
- A load-bearing consumer (orchestrator agent, parallel-build dispatcher) ships that needs typed-verb contract
- Existing `dispatch --isolate` shell-string interface measurably bottlenecks orchestrator velocity
- Design A (thin CLI wrappers) is chosen as the entry point; Design B (substrate-tracked branch-claim) is later, separate inception

**NO-GO if:**
- The ADR §6 #8 requirement is rescinded
- Design B (substrate-tracked branch-claim) is the only acceptable shape AND no consumer demands it (speculative substrate state)

**DEFER if:**
- No measured pain on the existing `dispatch --isolate` surface (TODAY'S STATE)
- No identified orchestrator consumer demands the typed contract
- #9 (cv_index, T-2089) has higher ROI for the same engineering attention

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

**Recommendation:** DEFER (with concrete revisit trigger)

**Rationale:**

Research artifact at `docs/reports/T-2090-typed-agent-launch-surface-inception.md` analyzed four designs (A: thin CLI wrappers, B: substrate-tracked branch-claim, C: MCP-only, D: documentation-only) and recommends DEFER. The existing `dispatch --isolate` + `--auto-merge` surface (`crates/termlink-cli/src/commands/dispatch.rs:36`, `crate::manifest::create_worktree` at manifest.rs:182, `merge_branch` at manifest.rs:287) already covers the orchestrator's checkout→work→merge happy path. There is no measured pain today and no identified load-bearing consumer demanding the typed contract. Adding typed verbs without a consumer is speculative scaffolding; adding substrate-tracked branch-claim state (Design B) is substantial work without a concrete invariant requirement.

DEFER preserves the ADR §6 #8 requirement (vs NO-GO which would contradict it), time-boxes the decision, and ties reopening to a concrete trigger: a real orchestrator (AEF Workflow agent, parallel-build dispatcher, or other) that uses `dispatch --isolate` at a scale where the untyped shell-string interface is measured friction. At that point, Design A is the natural starting point.

#9 (cv_index, T-2089) has measurably higher ROI for the same engineering attention (real `/peers` O(N_heartbeats) pain), and should be funded first.

**Evidence:**

- ADR §6 #8 explicit primitive requirement: `docs/architecture/parallel-execution-substrate.md:251-255`
- Existing dispatch surface (covers happy path today): `crates/termlink-cli/src/commands/dispatch.rs:36-200`
- Existing worktree machinery (analog of agent.checkout): `crates/termlink-cli/src/manifest.rs:182`
- Existing merge-back machinery (analog of agent.publish): `crates/termlink-cli/src/manifest.rs:287`
- Sibling inception with higher ROI: T-2089 (substrate #9, GO-recommended)
- No incident or consumer task identified as needing the typed contract today

**Revisit trigger:** Reopen when a consumer task is filed that specifies "needs typed agent-launch verbs" — typically when an orchestrator hits the 5+ parallel-worker scale and shell-string composition becomes brittle.

**revisit_at:** Not set (no calendar trigger — trigger is consumer-demand-based, not date-based).

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

### 2026-06-09T14:51:27Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
