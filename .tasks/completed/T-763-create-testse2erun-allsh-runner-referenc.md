---
id: T-763
name: "Create tests/e2e/run-all.sh runner referenced in README"
description: >
  Create tests/e2e/run-all.sh runner referenced in README

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:24:03Z
last_update: 2026-03-29T20:25:32Z
date_finished: 2026-03-29T20:25:32Z
---

# T-763: Create tests/e2e/run-all.sh runner referenced in README

## Context

README references `./tests/e2e/run-all.sh` but the file doesn't exist. Individual level scripts exist (level1-9) but no runner.

## Acceptance Criteria

### Agent
- [x] `tests/e2e/run-all.sh` created and executable
- [x] Discovers and runs all level*.sh scripts in order (sorted by level number)
- [x] Reports pass/fail summary with box-drawing UI
- [x] Exits non-zero if any level fails
- [x] Supports --level N, --from N, --to N flags

## Verification

test -x tests/e2e/run-all.sh
bash -n tests/e2e/run-all.sh

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

### 2026-03-29T20:24:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-763-create-testse2erun-allsh-runner-referenc.md
- **Context:** Initial task creation

### 2026-03-29T20:25:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
