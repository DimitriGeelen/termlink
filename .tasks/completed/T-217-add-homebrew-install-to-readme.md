---
id: T-217
name: "Add Homebrew install to README"
description: >
  Add Homebrew install to README

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:14:04Z
last_update: '2026-08-18T18:59:04Z'
date_finished: 2026-03-22T21:15:11Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:32Z'
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
  - ts: '2026-08-18T18:59:04Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-217: Add Homebrew install to README

## Context

Add Homebrew install option to README Installation section. Formula created in T-212.

## Acceptance Criteria

### Agent
- [x] README Installation section includes Homebrew install option
- [x] Quick Start section updated with brew as first install method

## Verification

grep -q 'brew install' README.md
grep -q 'brew tap' README.md

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

### 2026-03-22T21:14:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-217-add-homebrew-install-to-readme.md
- **Context:** Initial task creation

### 2026-03-22T21:15:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
