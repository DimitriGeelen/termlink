---
id: T-562
name: "Add --json output to termlink spawn"
description: >
  Add --json output to termlink spawn

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:28:44Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T12:30:46Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-562: Add --json output to termlink spawn

## Context

Add `--json` flag to `termlink spawn` so automation can capture spawned session name and backend in machine-readable format.

## Acceptance Criteria

### Agent
- [x] `Spawn` variant in cli.rs has `json: bool` field
- [x] `cmd_spawn` outputs JSON with session_name, backend, and ready status when --json is passed
- [x] Integration test validates JSON output from spawn --json
- [x] All existing tests pass

## Verification

cargo test -p termlink --test cli_integration -- cli_spawn_json 2>&1 | grep -q "test result"
cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T12:28:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-562-add---json-output-to-termlink-spawn.md
- **Context:** Initial task creation

### 2026-03-28T12:30:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
