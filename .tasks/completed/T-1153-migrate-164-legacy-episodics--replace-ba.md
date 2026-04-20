---
id: T-1153
name: "Migrate 164 legacy episodics — replace bare > summary with task name"
description: >
  Migrate 164 legacy episodics — replace bare > summary with task name

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T07:36:50Z
last_update: 2026-04-20T07:38:18Z
date_finished: 2026-04-20T07:38:18Z
---

# T-1153: Migrate 164 legacy episodics — replace bare > summary with task name

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Write `/tmp/fix-episodic-summaries.py` that finds episodics where `summary:` block is just `>` and replaces it with `task_name` from the matching task file (active or completed)
- [x] Dry-run: count degenerate episodics before migration is >100 (actual: 155)
- [x] Run migration; count degenerate episodics after is 0
- [x] Spot-check: T-1142 summary now reads the task name, not `>`
- [x] Watchtower still serves home/tasks/search pages (200s)

## Verification

python3 -c "import os,pathlib,sys; os.environ['PROJECT_ROOT']=str(pathlib.Path('.').resolve()); sys.path.insert(0,'.agentic-framework'); from web.search_utils import load_episodic_yaml; bad=[f.name for f in pathlib.Path('.context/episodic').glob('T-*.yaml') if (d:=load_episodic_yaml(f)) and isinstance(d.get('summary'), str) and d['summary'].strip() in ('>','|')]; print(f'degenerate: {len(bad)}'); sys.exit(1 if bad else 0)"
curl -sf http://localhost:3000/ -o /dev/null
curl -sf http://localhost:3000/tasks -o /dev/null

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

### 2026-04-20T07:36:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1153-migrate-164-legacy-episodics--replace-ba.md
- **Context:** Initial task creation

### 2026-04-20T07:38:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
