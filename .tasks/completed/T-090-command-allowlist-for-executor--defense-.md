---
id: T-090
name: "Command allowlist for executor — defense-in-depth for G-001"
description: >
  Command allowlist for executor — defense-in-depth for G-001

status: work-completed
workflow_type: build
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-11T09:00:42Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-11T23:44:27Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:41Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:concern-ref); D2=0 (no-signal); D3=0 (no-signal); D4=0
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-090: Command allowlist for executor — defense-in-depth for G-001

## Context

G-001 (critical): executor.rs passes user-controlled strings to `sh -c`. The 3-phase security model (T-077/T-078/T-086-088) mitigates "any process can connect" but doesn't restrict WHAT authenticated clients can execute. Add an optional command allowlist for defense-in-depth.

## Acceptance Criteria

### Agent
- [x] `Registration` has optional `allowed_commands: Vec<String>` field (backward compatible)
- [x] `executor::execute()` accepts optional allowlist and validates commands against prefix patterns
- [x] `handle_command_execute` passes allowlist from registration to executor
- [x] CLI `register` supports `--allowed-commands` flag
- [x] Tests: allowlist blocks disallowed commands, allows matching commands, absent allowlist allows all
- [x] G-001 updated in gaps.yaml with resolution details

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- executor 2>&1 | tail -5
/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- handler::tests::command_execute 2>&1 | tail -5

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

### 2026-03-11T09:00:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-090-command-allowlist-for-executor--defense-.md
- **Context:** Initial task creation

### 2026-03-11T23:44:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
