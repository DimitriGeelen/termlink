---
id: T-540
name: "Add termlink version subcommand with build info"
description: >
  Add termlink version subcommand with build info

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:10:39Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:12:33Z
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
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-540: Add termlink version subcommand with build info

## Context

`termlink --version` works but `termlink version` doesn't. Add a `version` subcommand showing version, git commit, and build target.

## Acceptance Criteria

### Agent
- [x] `termlink version` outputs version info
- [x] `termlink version --json` outputs JSON with version, commit, target
- [x] build.rs embeds GIT_COMMIT and TARGET at compile time
- [x] `cargo build` and tests pass

## Verification

cargo build 2>&1
./target/debug/termlink version 2>&1 | grep -q "termlink"

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

### 2026-03-28T09:10:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-540-add-termlink-version-subcommand-with-bui.md
- **Context:** Initial task creation

### 2026-03-28T09:12:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
