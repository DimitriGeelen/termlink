---
id: T-1104
name: "Add clickable references in Watchtower — task IDs link to review pages, hub profiles link to fleet"
description: >
  Add clickable references in Watchtower — task IDs link to review pages, hub profiles link to fleet

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T10:08:42Z
last_update: 2026-04-17T10:08:42Z
date_finished: null
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
- [ ] Verified links render in browser (requires watchtower restart with template reload)

## Verification

bash -c 'curl -sf http://localhost:3002/tasks | grep -q "href=\"/tasks/T-"'

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
