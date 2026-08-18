---
id: T-719
name: "Add --no-header flag to remote profile list command"
description: >
  Add --no-header flag to remote profile list command

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-03-29T11:31:25Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T11:32:52Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:07Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 4
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-719: Add --no-header flag to remote profile list command

## Context

`remote profile list` prints a table with header but has no `--no-header` flag, unlike other table-output commands (list, discover, topics, remote list).

## Acceptance Criteria

### Agent
- [x] `--no-header` flag added to ProfileAction::List in cli.rs
- [x] cmd_remote_profile passes no_header through
- [x] When --no-header, suppress header row, separator, and summary footer
- [x] Project compiles with `cargo check`

## Verification

cargo check 2>&1 | grep -q 'Finished'
grep -q 'no_header' crates/termlink-cli/src/cli.rs
grep -q 'no_header' crates/termlink-cli/src/commands/remote.rs

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

### 2026-03-29T11:31:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-719-add---no-header-flag-to-remote-profile-l.md
- **Context:** Initial task creation

### 2026-03-29T11:32:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
