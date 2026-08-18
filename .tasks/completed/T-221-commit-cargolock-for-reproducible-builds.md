---
id: T-221
name: "Commit Cargo.lock for reproducible builds"
description: >
  Commit Cargo.lock for reproducible builds

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:24:55Z
last_update: '2026-08-18T18:59:05Z'
date_finished: 2026-03-23T07:48:00Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:33Z'
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
  - ts: '2026-08-18T18:59:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-221: Commit Cargo.lock for reproducible builds

## Context

Per Rust convention, binary crates should commit Cargo.lock for reproducible builds. TermLink is a binary — CI and release builds should use exact same dependency versions. Currently gitignored.

## Acceptance Criteria

### Agent
- [x] Cargo.lock removed from .gitignore
- [x] Cargo.lock tracked in git

## Verification

! grep -q '^Cargo.lock' .gitignore
test -f Cargo.lock
git ls-files --error-unmatch Cargo.lock

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

### 2026-03-22T21:24:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-221-commit-cargolock-for-reproducible-builds.md
- **Context:** Initial task creation

### 2026-03-23T07:48:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
