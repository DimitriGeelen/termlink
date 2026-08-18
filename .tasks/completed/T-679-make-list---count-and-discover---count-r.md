---
id: T-679
name: "Make list --count and discover --count respect --json flag"
description: >
  Make list --count and discover --count respect --json flag

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:51:00Z
last_update: '2026-08-18T18:59:19Z'
date_finished: 2026-03-28T22:52:30Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:06Z'
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
  - ts: '2026-08-18T18:59:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-679: Make list --count and discover --count respect --json flag

## Context

`list --count --json` and `discover --count --json` output bare numbers, ignoring the --json flag. Should output `{"count": N}` when --json is set.

## Acceptance Criteria

### Agent
- [x] `list --count --json` outputs `{"count": N}` instead of bare number
- [x] `discover --count --json` outputs `{"count": N}` instead of bare number
- [x] Without --json, both still output bare number
- [x] Project compiles cleanly

## Verification

grep -q '"count"' /opt/termlink/crates/termlink-cli/src/commands/session.rs

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

### 2026-03-28T22:51:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-679-make-list---count-and-discover---count-r.md
- **Context:** Initial task creation
