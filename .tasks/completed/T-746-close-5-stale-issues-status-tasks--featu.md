---
id: T-746
name: "Close 5 stale issues-status tasks — features already implemented"
description: >
  Close 5 stale issues-status tasks — features already implemented

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T14:18:30Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T14:20:08Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:08Z'
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
      blast_radius: 0
      tier: 3
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-746: Close 5 stale issues-status tasks — features already implemented

## Context

5 tasks were created during a batch CLI improvement sweep but set to `issues` because they had placeholder ACs. All 5 features already exist in the codebase.

## Acceptance Criteria

### Agent
- [x] T-630 closed (--raw on kv get already exists)
- [x] T-634 closed (--dry-run on vendor — vendor status --check already covers this)
- [x] T-667 closed (--json on hub start already exists)
- [x] T-670 closed (--json error on connect_remote_hub — already handled)
- [x] T-707 closed (--id on list --first — --ids flag already exists)

## Verification

test -f .tasks/completed/T-630-*.md
test -f .tasks/completed/T-634-*.md
test -f .tasks/completed/T-667-*.md
test -f .tasks/completed/T-670-*.md
test -f .tasks/completed/T-707-*.md

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

### 2026-03-29T14:18:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-746-close-5-stale-issues-status-tasks--featu.md
- **Context:** Initial task creation

### 2026-03-29T14:20:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
