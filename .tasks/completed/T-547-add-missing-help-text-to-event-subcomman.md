---
id: T-547
name: "Add missing help text to event subcommand args"
description: >
  Add missing help text to event subcommand args

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:42:15Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:43:17Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-547: Add missing help text to event subcommand args

## Context

Investigated: the EventCommand enum (primary subcommands) already has full help text. The missing help is only on hidden backward-compat aliases which don't show in `--help`. No changes needed.

## Acceptance Criteria

### Agent
- [x] All event subcommand positional args have `///` help text (already present in EventCommand enum)
- [x] Builds without warnings

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-28T09:42:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-547-add-missing-help-text-to-event-subcomman.md
- **Context:** Initial task creation

### 2026-03-28T09:43:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
