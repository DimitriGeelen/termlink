---
id: T-102
name: "Inception — Orchestrator mandatory tool call constraint"
description: >
  Explore the architectural idea: restrict the orchestrator agent so every substantive
  response MUST include a tool call. If the orchestrator needs to explore/discuss,
  it
  spawns a dedicated agent for that. This transforms invisible conversations into
  traceable tool-call sequences. Explore only — understand tradeoffs before deciding.
status: work-completed
workflow_type: inception
owner: human
horizon:
tags: [architecture, orchestrator, tool-call, exploration]
components: []
related_tasks: [T-094, T-099, T-100, T-101]
created: 2026-03-11T12:00:00Z
last_update: '2026-08-18T18:58:42Z'
date_finished: 2026-03-18T21:29:57Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:42Z'
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
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 4
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=4 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-102: Inception — Orchestrator Mandatory Tool Call Constraint

## Problem Statement

What if we change the architectural constraint: the orchestrator MUST make a tool call
for every substantive response. Pure conversation is not allowed at the orchestrator level.
If exploration is needed, it must be delegated to a spawned agent.

## Research Artifact

`docs/reports/T-102-orchestrator-mandatory-tool-call-constraint.md`

## The Idea

Three variants to explore:

**Variant A — Mandatory `fw note` per response:**
Every substantive assistant response must include `fw note "..."` as a tool call.
Conversation is logged structurally. All turns become hookable tool events.
Tradeoff: overhead, changes the feel of interaction.

**Variant B — Spawn-for-conversation:**
Orchestrator never explores inline. "We need to think about X" → spawns an inception
agent → returns a structured result. Orchestrator stays clean; all exploration is
delegated and therefore tracked.
Tradeoff: heavyweight, latency, cost.

**Variant C — Scribe agent:**
Lightweight TermLink session acting as a conversation logger. Orchestrator routes all
responses through the scribe, which logs them as TermLink events (persistent, replayable).
Tradeoff: requires Agent Mesh Phase 1 to be built first.

## Relationship to Other Options

This is the most architectural of the four options — it changes HOW the orchestrator
works rather than capturing what it produces. Orthogonal to T-101 (reading existing
transcript) and T-100 (capturing terminal output). Complementary to T-099 (platform fix).

Could be combined: use JSONL reader (T-101) as the immediate fix, and orchestrator
constraint (T-102) as the long-term architectural norm.

## Scope Fence

**IN:** Understand the tradeoffs of each variant, map implications for human-agent UX
**OUT:** Any implementation — this is exploration and dialogue only

## Acceptance Criteria

### Agent
- [x] Three variants documented with tradeoffs
- [x] Impact on human-agent interaction analyzed
- [x] Relationship to Agent Mesh roadmap mapped
- [x] Go/no-go framed for discussion

### Human
- [x] Variants discussed, preferred direction identified

## Verification

test -f docs/reports/T-102-orchestrator-mandatory-tool-call-constraint.md
grep -q "NO-GO" docs/reports/T-102-orchestrator-mandatory-tool-call-constraint.md

## Decisions

**Decision**: NO-GO

**Rationale**: No response boundary hook in Claude Code

**Date**: 2026-03-18T21:29:57Z
## Decision

**Decision**: NO-GO

**Rationale**: No response boundary hook in Claude Code

**Date**: 2026-03-18T21:29:57Z

## Updates

### 2026-03-18T21:18:53Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-18T21:25:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T21:29:57Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-18T21:29:57Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** No response boundary hook in Claude Code

### 2026-03-18T21:29:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
