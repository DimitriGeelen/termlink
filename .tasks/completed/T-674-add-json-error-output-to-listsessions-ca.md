---
id: T-674
name: "Add JSON error output to list_sessions calls in metadata.rs and session.rs"
description: >
  Add JSON error output to list_sessions calls in metadata.rs and session.rs

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:38:13Z
last_update: '2026-08-18T18:59:19Z'
date_finished: 2026-03-28T22:39:30Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:06Z'
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
  - ts: '2026-08-18T18:59:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-674: Add JSON error output to list_sessions calls in metadata.rs and session.rs

## Context

metadata.rs has 2 bare `.context()?` calls on list_sessions in cmd_discover (lines 127, 143). session.rs has 2 bare `do_filter()?` calls in cmd_list (lines 295, 309) that propagate list_sessions errors without JSON output.

## Acceptance Criteria

### Agent
- [x] metadata.rs: Both list_sessions calls wrapped with JSON error output
- [x] session.rs: Both do_filter calls wrapped with JSON error output
- [x] Project compiles cleanly

## Verification

grep -c "process::exit" /opt/termlink/crates/termlink-cli/src/commands/metadata.rs | grep -q "[0-9]"

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

### 2026-03-28T22:38:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-674-add-json-error-output-to-listsessions-ca.md
- **Context:** Initial task creation
