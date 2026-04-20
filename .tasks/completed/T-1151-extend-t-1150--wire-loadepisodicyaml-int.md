---
id: T-1151
name: "Extend T-1150 — wire load_episodic_yaml into tasks.py and inception.py task-detail pages"
description: >
  Extend T-1150 — wire load_episodic_yaml into tasks.py and inception.py task-detail pages

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T23:59:42Z
last_update: 2026-04-20T00:01:17Z
date_finished: 2026-04-20T00:01:17Z
---

# T-1151: Extend T-1150 — wire load_episodic_yaml into tasks.py and inception.py task-detail pages

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Replace `yaml.safe_load` in `web/blueprints/tasks.py:400` (task detail page) with `load_episodic_yaml`
- [x] Replace `yaml.safe_load` in `web/blueprints/inception.py:348` (inception detail page) with `load_episodic_yaml`
- [x] `curl http://localhost:3000/tasks/T-927` returns 200 (T-927 had anchor-error before T-1150)
- [x] `curl http://localhost:3000/tasks/T-121` returns 200 (T-121 had frontmatter-error before T-1150)

## Verification

curl -sf http://localhost:3000/tasks/T-927 -o /dev/null
curl -sf http://localhost:3000/tasks/T-121 -o /dev/null

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

### 2026-04-19T23:59:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1151-extend-t-1150--wire-loadepisodicyaml-int.md
- **Context:** Initial task creation

### 2026-04-20T00:01:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
