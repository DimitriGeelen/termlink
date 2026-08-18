---
id: T-014
name: "Rust project scaffold — Cargo workspace and crate structure"
description: >
  Rust project scaffold — Cargo workspace and crate structure

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T15:32:13Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T15:41:05Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
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
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-014: Rust project scaffold — Cargo workspace and crate structure

## Context

Set up Cargo workspace with crate structure for TermLink. Based on T-013 (Rust), T-004 (MCP + custom data plane), T-005 (protocol spec), T-006 (session identity).

## Acceptance Criteria

### Agent
- [x] Cargo workspace compiles with `cargo build`
- [x] Workspace contains: termlink-protocol, termlink-session, termlink-hub, termlink (CLI binary)
- [x] Each crate has appropriate dependencies declared
- [x] `cargo test` passes (19 tests: 8 protocol, 11 session)
- [x] Project structure matches the two-plane architecture

## Verification

source "$HOME/.cargo/env" && cargo build 2>&1 | tail -1
source "$HOME/.cargo/env" && cargo test 2>&1 | tail -1

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

### 2026-03-08T15:32:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-014-rust-project-scaffold--cargo-workspace-a.md
- **Context:** Initial task creation

### 2026-03-08T15:41:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
