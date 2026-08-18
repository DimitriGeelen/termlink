---
id: T-760
name: "Update Homebrew formula — add Linux aarch64 variant to match release workflow"
description: >
  Update Homebrew formula — add Linux aarch64 variant to match release workflow

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:06:21Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T20:07:35Z
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-760: Update Homebrew formula — add Linux aarch64 variant to match release workflow

## Context

Homebrew formula only has Linux x86_64. Needs Linux aarch64 variant to match T-759 release workflow update.

## Acceptance Criteria

### Agent
- [x] Linux `on_linux` block split into arm? and x86_64 variants
- [x] Linux aarch64 URL points to termlink-linux-aarch64 release asset
- [x] Formula structure validated: 4 platform variants (darwin-aarch64, darwin-x86_64, linux-x86_64, linux-aarch64)

## Verification

grep -q "termlink-linux-aarch64" homebrew/Formula/termlink.rb

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

### 2026-03-29T20:06:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-760-update-homebrew-formula--add-linux-aarch.md
- **Context:** Initial task creation

### 2026-03-29T20:07:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
