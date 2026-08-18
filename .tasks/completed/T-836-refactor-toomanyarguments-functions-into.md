---
id: T-836
name: "Refactor too_many_arguments functions into parameter structs"
description: >
  Refactor too_many_arguments functions into parameter structs

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/agent.rs, 
      crates/termlink-cli/src/commands/dispatch.rs, 
      crates/termlink-cli/src/commands/execution.rs, 
      crates/termlink-cli/src/commands/metadata.rs, 
      crates/termlink-cli/src/commands/mod.rs, 
      crates/termlink-cli/src/commands/push.rs, 
      crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-cli/src/commands/session.rs, 
      crates/termlink-cli/src/main.rs, crates/termlink-hub/src/remote_store.rs, 
      crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-04-03T22:47:41Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-03T23:19:25Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 9
      tier: 3
      effort: 5
    rationale: blast_radius=9 (no-signal); tier=3 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-836: Refactor too_many_arguments functions into parameter structs

## Context

18 functions across CLI crate use `#[allow(clippy::too_many_arguments)]` with 7-15 parameters each.
Worst offenders: `remote.rs` (8 functions, up to 15 args), `session.rs` (2), `metadata.rs` (1), `dispatch.rs` (1), `execution.rs` (1), `push.rs` (2), `agent.rs` (1), `remote_store.rs` (1).
Extract parameter structs to improve readability and remove all `#[allow(clippy::too_many_arguments)]` annotations.

## Acceptance Criteria

### Agent
- [x] Zero `#[allow(clippy::too_many_arguments)]` annotations remain in workspace
- [x] `cargo build --workspace` compiles without errors
- [x] `cargo test --workspace` passes (705+ tests)
- [x] `cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\['` returns 0
- [x] Parameter structs introduced for remote connection, list display, dispatch, and spawn options

## Verification

cargo build --workspace 2>&1 | tail -1
cargo test --workspace 2>&1 | tail -3
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c '^warning\[')" = "0"
test "$(grep -r 'allow(clippy::too_many_arguments)' crates/ | wc -l)" = "0"

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

### 2026-04-03T22:47:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-836-refactor-toomanyarguments-functions-into.md
- **Context:** Initial task creation

### 2026-04-03T23:19:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
