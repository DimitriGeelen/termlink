---
id: T-213
name: "Fix CI release workflow: wrong package name termlink-cli → termlink"
description: >
  Fix CI release workflow: wrong package name termlink-cli → termlink

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-22T17:08:08Z
last_update: '2026-08-18T18:59:03Z'
date_finished: 2026-03-22T17:09:31Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:29Z'
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
  - ts: '2026-08-18T18:59:03Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-213: Fix CI release workflow: wrong package name termlink-cli → termlink

## Context

release.yml uses `-p termlink-cli` but the CLI package is named `termlink` in Cargo.toml. This will cause all CI release builds to fail.

## Acceptance Criteria

### Agent
- [x] release.yml uses `-p termlink` (not `-p termlink-cli`)
- [x] `cargo build --release -p termlink` succeeds locally

## Verification

grep -q -- "-p termlink" .github/workflows/release.yml
! grep -q "termlink-cli" .github/workflows/release.yml

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

### 2026-03-22T17:08:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-213-fix-ci-release-workflow-wrong-package-na.md
- **Context:** Initial task creation

### 2026-03-22T17:09:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
