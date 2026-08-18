---
id: T-171
name: "Fix event poll cursor off-by-one in CLI commands"
description: >
  Fix event poll cursor off-by-one in CLI commands

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-18T18:48:54Z
last_update: '2026-08-18T18:58:54Z'
date_finished: 2026-03-18T19:09:02Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:09Z'
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
  - ts: '2026-08-18T18:58:54Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-171: Fix event poll cursor off-by-one in CLI commands

## Context

`event.poll` with `since: 0` returns events after seq 0, skipping the first event. `cmd_file_receive` and `cmd_agent_listen` fetch initial `next_seq` which returns 0 for an empty bus, then pass `since: 0`, missing the first event. Fix: don't pass `since` on initial poll.

## Acceptance Criteria

### Agent
- [x] `cmd_file_receive` starts with `poll_cursor = None` (no `since` on first poll)
- [x] `cmd_agent_listen` starts with `poll_cursor = None`
- [x] All existing tests pass
- [x] File transfer E2E works (validated by T-168 human AC test)

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink 2>&1); echo "$out" | grep -q "0 failed"'

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

### 2026-03-18T18:48:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-171-fix-event-poll-cursor-off-by-one-in-cli-.md
- **Context:** Initial task creation

### 2026-03-18T19:09:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
