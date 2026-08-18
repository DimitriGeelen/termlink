---
id: T-989
name: "send-file hub fallback — route through hub when target not found locally (enables
  inbox)"
description: >
  send-file hub fallback — route through hub when target not found locally (enables
  inbox)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/file.rs]
related_tasks: []
created: 2026-04-12T21:59:27Z
last_update: '2026-08-18T18:59:24Z'
date_finished: 2026-04-12T22:04:23Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:16Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 1
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=1 (body:log-or-error-line); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 5
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-989: send-file hub fallback — route through hub when target not found locally (enables inbox)

## Context

Build task from T-946/T-951 GO decisions + T-988 (hub inbox). Currently `send-file` resolves the
target session locally via `manager::find_session()`. If the target is offline or on a different
machine, it fails. With T-988's hub inbox, the hub can spool files for offline sessions. This task
adds a fallback: when local lookup fails, route the file events through the hub's `event.emit_to`,
which triggers inbox spooling for offline targets.

## Acceptance Criteria

### Agent
- [x] `cmd_file_send` falls back to hub `event.emit_to` when `find_session` fails
- [x] Hub fallback sends file.init, file.chunk, file.complete via hub socket
- [x] Response distinguishes direct delivery vs hub-spooled (`via` field in JSON output)
- [x] When neither local session nor hub is available, error message is clear
- [x] All existing CLI tests pass — 165/165 + 83/83 integration

## Verification

cargo test -p termlink
cargo clippy -p termlink -- -D warnings

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

### 2026-04-12T21:59:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-989-send-file-hub-fallback--route-through-hu.md
- **Context:** Initial task creation

### 2026-04-12T22:04:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Hub fallback implemented, 165+83 tests pass
