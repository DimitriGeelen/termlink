---
id: T-216
name: "Strengthen CI — add tests and clippy to workflow"
description: >
  Strengthen CI — add tests and clippy to workflow

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:12:37Z
last_update: '2026-08-18T18:59:04Z'
date_finished: 2026-03-22T21:13:31Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:31Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:04Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-216: Strengthen CI — add tests and clippy to workflow

## Context

CI workflow (.github/workflows/ci.yml) currently only runs `cargo check`. Adding `cargo clippy` and `cargo test` catches regressions earlier. The codebase already has 277 passing tests and 0 clippy warnings.

## Acceptance Criteria

### Agent
- [x] CI workflow runs `cargo clippy --workspace -- -D warnings`
- [x] CI workflow runs `cargo test --workspace`
- [x] Clippy component added to Rust toolchain install step

## Verification

grep -q 'clippy' .github/workflows/ci.yml
grep -q 'cargo test' .github/workflows/ci.yml

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

### 2026-03-22T21:12:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-216-strengthen-ci--add-tests-and-clippy-to-w.md
- **Context:** Initial task creation

### 2026-03-22T21:13:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
