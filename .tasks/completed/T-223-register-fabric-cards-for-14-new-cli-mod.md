---
id: T-223
name: "Register fabric cards for 14 new CLI modules"
description: >
  Register fabric cards for 14 new CLI modules

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T07:04:09Z
last_update: 2026-03-21T07:08:36Z
date_finished: 2026-03-21T07:08:36Z
---

# T-223: Register fabric cards for 14 new CLI modules

## Context

T-222 split the CLI monolith (5183 lines) into 14 focused modules. The commit hook flagged that these new files lack fabric component cards. This task registers cards for all 14 files and updates the existing main.rs card and subsystems.yaml.

## Acceptance Criteria

### Agent
- [x] 14 new fabric cards created in `.fabric/components/` (one per new CLI source file)
- [x] Existing `crates-termlink-cli-src-main.yaml` card updated to reflect dispatch-only role
- [x] `subsystems.yaml` CLI subsystem updated with all 15 component paths
- [x] All YAML cards parse correctly (17 total CLI cards including 2 test cards)
- [x] `fw fabric drift` shows no unregistered CLI source files (0 unregistered)

## Verification

python3 -c "import yaml, glob; [yaml.safe_load(open(f)) for f in glob.glob('.fabric/components/crates-termlink-cli-*.yaml')]"
test $(ls .fabric/components/crates-termlink-cli-*.yaml | wc -l) -ge 15

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

### 2026-03-21T07:04:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-223-register-fabric-cards-for-14-new-cli-mod.md
- **Context:** Initial task creation

### 2026-03-21T07:08:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
