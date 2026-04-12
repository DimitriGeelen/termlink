---
id: T-911
name: "lib/harvest.sh use PROJECT_ROOT not FRAMEWORK_ROOT for learnings"
description: >
  Follow-up from T-909. lib/harvest.sh:74-75,363 writes harvested learnings to $FRAMEWORK_ROOT/.context/ which is currently accidentally-correct (writes to live framework via symlink) but will be wrong after any project vendors its framework. Post-T-909, fw harvest from /opt/termlink writes to the static vendored copy instead of the live framework repo. Fix: use $PROJECT_ROOT for per-project learning capture; optionally support a --upstream flag for pushing back to the framework.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [infrastructure, harvest, path-resolution]
components: []
related_tasks: []
created: 2026-04-11T12:28:37Z
last_update: 2026-04-12T20:46:11Z
date_finished: 2026-04-12T20:35:29Z
---

# T-911: lib/harvest.sh use PROJECT_ROOT not FRAMEWORK_ROOT for learnings

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `framework_context` (line ~74) uses `$PROJECT_ROOT` not `$FRAMEWORK_ROOT`
- [x] `harvest_log` (line ~75) uses `$PROJECT_ROOT` not `$FRAMEWORK_ROOT`
- [x] `framework_episodics` (line ~363) uses `$PROJECT_ROOT` not `$FRAMEWORK_ROOT`
- [x] No remaining `FRAMEWORK_ROOT/.context` paths in harvest output destinations

## Verification

# Shell commands that MUST pass before work-completed. One per line.
! grep -q 'FRAMEWORK_ROOT/\.context' /opt/termlink/.agentic-framework/lib/harvest.sh

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

### 2026-04-11T12:28:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-911-libharvestsh-use-projectroot-not-framewo.md
- **Context:** Initial task creation

### 2026-04-12T11:23:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T20:35:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Human reviewed
