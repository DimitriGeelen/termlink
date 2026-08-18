---
id: T-1105
name: "Add termlink fleet status --verbose — show sessions per hub and fleet status
  as default fleet subcommand"
description: >
  Add termlink fleet status --verbose — show sessions per hub and fleet status as
  default fleet subcommand

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-17T10:42:35Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-04-17T10:47:41Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:45Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:44Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 4
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1105: Add termlink fleet status --verbose — show sessions per hub and fleet status as default fleet subcommand

## Context

Enhance fleet status with --verbose to show session names per hub, and make
`termlink fleet` default to `status` (so the operator can just type `termlink fleet`).

## Acceptance Criteria

### Agent
- [x] `termlink fleet status --verbose` shows session names per UP hub
- [x] `termlink fleet` (no subcommand) defaults to `status`
- [x] `--verbose` in JSON mode includes session_names array per hub
- [x] Builds with zero warnings, 3 fleet status tests pass

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-17T10:42:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1105-add-termlink-fleet-status---verbose--sho.md
- **Context:** Initial task creation

### 2026-04-17T10:47:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
