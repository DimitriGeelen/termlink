---
id: T-541
name: "Add filtering to termlink list (--tag, --name)"
description: >
  Add filtering to termlink list (--tag, --name)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/session.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T09:14:37Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:16:22Z
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
      blast_radius: 3
      tier: 2
      effort: 4
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-541: Add filtering to termlink list (--tag, --name)

## Context

`termlink list` shows all sessions with no way to filter. With 58+ sessions, finding relevant ones is noisy. Add `--tag` and `--name` filters.

## Acceptance Criteria

### Agent
- [x] `termlink list --tag foo` shows only sessions tagged "foo"
- [x] `termlink list --name pattern` filters by session name substring
- [x] Filters work with `--json` output
- [x] `cargo build` succeeds

## Verification

cargo build 2>&1
./target/debug/termlink list --help 2>&1 | grep -q "tag"

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

### 2026-03-28T09:14:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-541-add-filtering-to-termlink-list---tag---n.md
- **Context:** Initial task creation

### 2026-03-28T09:16:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
