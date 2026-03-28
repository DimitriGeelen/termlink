---
id: T-677
name: "Make list --first respect --ids and --names flags"
description: >
  Make list --first respect --ids and --names flags

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:47:14Z
last_update: 2026-03-28T22:48:16Z
date_finished: 2026-03-28T22:48:30Z
---

# T-677: Make list --first respect --ids and --names flags

## Context

`list --first` always outputs display name. `discover --first --id` can output the ID. Fix `list --first` to respect `--ids` flag (output ID) and move `--first` check before `--ids`/`--names` to handle combinations.

## Acceptance Criteria

### Agent
- [x] `list --first` moved before `--names` and `--ids` in control flow
- [x] `list --first --ids` outputs the session ID
- [x] `list --first` (without --ids) still outputs display name
- [x] `list --first --json` outputs single JSON object (not array)
- [x] Project compiles cleanly

## Verification

grep -q "if first" /opt/termlink/crates/termlink-cli/src/commands/session.rs

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

### 2026-03-28T22:47:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-677-make-list---first-respect---ids-and---na.md
- **Context:** Initial task creation
