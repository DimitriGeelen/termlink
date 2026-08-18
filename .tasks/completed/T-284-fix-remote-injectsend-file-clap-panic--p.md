---
id: T-284
name: "Fix remote inject/send-file clap panic — positional argument ordering"
description: >
  Fix remote inject/send-file clap panic — positional argument ordering

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-25T19:49:58Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-25T19:54:17Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 6
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-284: Fix remote inject/send-file clap panic — positional argument ordering

## Context

`termlink remote inject` and `termlink remote send-file` panic on ANY invocation (including --help) due to clap positional argument ordering: optional `session: Option<String>` appears before required `text: String`/`path: String`. Discovered while sending pickup to fw-agent on .107 during T-283 investigation. Blocks all remote inject/send-file operations.

## Acceptance Criteria

### Agent
- [x] `termlink remote inject --help` runs without panic
- [x] `termlink remote send-file --help` runs without panic
- [x] `termlink remote inject mint fw-agent "test" --enter --secret <secret>` executes successfully
- [x] `termlink remote send-file mint fw-agent /tmp/test.txt --secret <secret>` executes successfully
- [x] All existing tests pass (`cargo test --workspace`)
- [x] 0 compiler warnings

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-25T19:49:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-284-fix-remote-injectsend-file-clap-panic--p.md
- **Context:** Initial task creation

### 2026-03-25T19:54:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
