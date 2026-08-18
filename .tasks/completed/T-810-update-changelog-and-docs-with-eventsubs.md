---
id: T-810
name: "Update CHANGELOG and docs with event_subscribe, doctor dispatch check"
description: >
  Update CHANGELOG and docs with event_subscribe, doctor dispatch check

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T17:59:58Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T18:05:16Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-810: Update CHANGELOG and docs with event_subscribe, doctor dispatch check

## Context

Update CHANGELOG and ARCHITECTURE.md to reflect T-805 (event.subscribe since), T-806 (doctor dispatch check), T-809 (MCP event_subscribe tool).

## Acceptance Criteria

### Agent
- [x] CHANGELOG.md updated with new features under 0.9.0
- [x] ARCHITECTURE.md test counts updated (684 total)
- [x] MCP tool count updated (27 tools in CHANGELOG, test table)

## Verification

grep -q "event_subscribe" CHANGELOG.md
grep -q "dispatch" CHANGELOG.md

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

### 2026-03-30T17:59:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-810-update-changelog-and-docs-with-eventsubs.md
- **Context:** Initial task creation

### 2026-03-30T18:05:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
