---
id: T-872
name: "termlink event subscribe CLI subcommand — direct event.subscribe exposure for
  scripting and debugging"
description: >
  termlink event subscribe CLI subcommand — direct event.subscribe exposure for scripting
  and debugging

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/events.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-04T22:36:53Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T22:49:22Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
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
      blast_radius: 3
      tier: 2
      effort: 5
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-872: termlink event subscribe CLI subcommand — direct event.subscribe exposure for scripting and debugging

## Context

T-690 event subscription is fully implemented but `event watch` has two gaps:
1. Multi-session watch calls `event.subscribe` sequentially (one session at a time), creating latency proportional to session count. Should be concurrent like `event collect` via hub.
2. No `--since` flag — users can't replay history when starting a watch.

Related: T-690 (inception), T-280 (dispatch readiness).

## Acceptance Criteria

### Agent
- [x] `event watch` with multiple sessions dispatches subscribe calls concurrently via tokio::JoinSet
- [x] `--since` flag added to `event watch` CLI, passed to event.subscribe RPC
- [x] Existing watch tests still pass
- [x] `cargo clippy --workspace` passes with no warnings (0 warnings)
- [x] `cargo test --workspace` passes (857 tests, 0 failures)

## Verification

# Clippy clean
cargo clippy --workspace 2>&1 | grep -v "^$" | tail -5 | grep -q "warning generated\|could not compile" && exit 1 || true
# Tests pass
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T22:36:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-872-termlink-event-subscribe-cli-subcommand-.md
- **Context:** Initial task creation

### 2026-04-04T22:49:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
