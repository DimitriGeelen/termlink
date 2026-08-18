---
id: T-060
name: "CLI request command — emit event and wait for reply (request-reply pattern)"
description: >
  CLI request command — emit event and wait for reply (request-reply pattern)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-09T12:07:16Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-09T12:12:33Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
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

# T-060: CLI request command — emit event and wait for reply (request-reply pattern)

## Context

Phase 1 of T-012 agent-to-agent communication. Adds `termlink request` — a convenience command that emits an event to a target and waits for a reply event on the same target's bus. This is the request-reply pattern for task delegation.

## Acceptance Criteria

### Agent
- [x] `Request` variant added to Command enum
- [x] `cmd_request` emits event to target, then polls target for reply topic
- [x] Auto-generates `request_id` in payload
- [x] Prints reply event payload on success, exits non-zero on timeout
- [x] CLI integration tests: request-reply flow + request timeout (15/15 pass)
- [x] CLI builds and all existing tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | grep -q "Finished"
/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration 2>&1 | grep -q "passed"

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

### 2026-03-09T12:07:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-060-cli-request-command--emit-event-and-wait.md
- **Context:** Initial task creation

### 2026-03-09T12:12:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
