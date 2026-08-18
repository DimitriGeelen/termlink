---
id: T-777
name: "Fix ARCHITECTURE.md stale component tables — missing modules, wrong FrameType
  names"
description: >
  Fix ARCHITECTURE.md stale component tables — missing modules, wrong FrameType names

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T00:04:37Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T00:06:49Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-777: Fix ARCHITECTURE.md stale component tables — missing modules, wrong FrameType names

## Context

ARCHITECTURE.md component tables are stale: FrameType names wrong (Stdout/Stderr vs actual Output/Input), protocol transport.rs missing, session endpoint/tofu/transport missing, hub missing 7 modules added since initial doc.

## Acceptance Criteria

### Agent
- [x] FrameType names corrected to match actual enum (Output, Input, Resize, Signal, Transfer, Ping, Pong, Close)
- [x] Protocol transport.rs listed in components
- [x] Session endpoint.rs, tofu.rs, transport.rs listed in components
- [x] Hub missing modules listed (bypass, circuit_breaker, remote_store, route_cache, template_cache, tls, trust)

## Verification

grep -q "Output/Input/Resize/Signal" docs/ARCHITECTURE.md
grep -q "transport.rs" docs/ARCHITECTURE.md
grep -q "endpoint" docs/ARCHITECTURE.md
grep -q "circuit_breaker" docs/ARCHITECTURE.md

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-30T00:04:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-777-fix-architecturemd-stale-component-table.md
- **Context:** Initial task creation

### 2026-03-30T00:06:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
