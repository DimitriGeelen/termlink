---
id: T-1104
name: "Add clickable references in Watchtower — task IDs link to review pages, hub
  profiles link to fleet"
description: >
  Add clickable references in Watchtower — task IDs link to review pages, hub profiles
  link to fleet

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-17T10:08:42Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-04-17T16:05:52Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:45Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 3
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=0 (no-signal); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:44Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1104: Add clickable references in Watchtower — task IDs link to review pages, hub profiles link to fleet

## Context

T-1101 R5: Apply `linkify_tasks` Jinja filter to more Watchtower pages so task IDs
(T-XXX) become clickable links to `/tasks/T-XXX`. The filter already exists in
shared.py and is registered as a Jinja filter. Currently only used in timeline.html.

## Acceptance Criteria

### Agent
- [x] `linkify_tasks` filter applied to task description in task_detail.html
- [x] `linkify_tasks` filter applied to fleet page actions text
- [x] `linkify_tasks` filter applied to AC text in task_detail.html (3 occurrences)
- [x] Verified links render: `curl /tasks/T-1101` shows `<a href="/tasks/T-1102">T-1102</a>`

## Verification

curl -sf http://localhost:3000/tasks/T-1101 | grep -q 'href="/tasks/T-'

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

### 2026-04-17T10:08:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1104-add-clickable-references-in-watchtower--.md
- **Context:** Initial task creation

### 2026-04-17T16:05:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
