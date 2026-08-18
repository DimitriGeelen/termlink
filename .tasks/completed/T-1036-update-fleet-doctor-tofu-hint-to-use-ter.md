---
id: T-1036
name: "Update fleet-doctor TOFU hint to use termlink tofu clear command"
description: >
  Update fleet-doctor TOFU hint to use termlink tofu clear command

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-session/src/tofu.rs]
related_tasks: []
created: 2026-04-13T18:50:02Z
last_update: '2026-08-18T18:58:42Z'
date_finished: 2026-04-13T18:51:35Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:43Z'
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
  - ts: '2026-08-18T18:58:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 3
      effort: 3
    rationale: blast_radius=3 (no-signal); tier=3 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-1036: Update fleet-doctor TOFU hint to use termlink tofu clear command

## Context

T-1035 added `termlink tofu clear` command. Update fleet-doctor and TOFU violation error messages to reference it instead of "edit ~/.termlink/known_hubs".

## Acceptance Criteria

### Agent
- [x] Fleet-doctor TOFU violation hint uses `termlink tofu clear` command
- [x] TOFU violation error message in tofu.rs references `termlink tofu clear`
- [x] Builds with zero clippy warnings

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T18:50:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1036-update-fleet-doctor-tofu-hint-to-use-ter.md
- **Context:** Initial task creation

### 2026-04-13T18:51:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
