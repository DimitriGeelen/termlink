---
id: T-570
name: "Add --json output to termlink pty resize"
description: >
  Add --json output to termlink pty resize

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T12:46:09Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T12:47:45Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 2
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=2 (body:telemetry-or-audit-entry); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-570: Add --json output to termlink pty resize

## Context

Add `--json` flag to `termlink pty resize` and hidden `termlink resize` for structured output.

## Acceptance Criteria

### Agent
- [x] `PtyCommand::Resize` and hidden `Resize` have `json: bool` field
- [x] `cmd_resize` outputs JSON with cols, rows when --json is passed
- [x] All existing tests pass

## Verification

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

### 2026-03-28T12:46:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-570-add---json-output-to-termlink-pty-resize.md
- **Context:** Initial task creation

### 2026-03-28T12:47:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
