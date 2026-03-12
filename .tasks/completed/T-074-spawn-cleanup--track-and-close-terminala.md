---
id: T-074
name: "Spawn cleanup — track and close Terminal.app windows on test exit"
description: >
  E2e tests spawn Terminal.app windows via osascript but never close them. Window IDs are returned by spawn (tab 1 of window id XXXX) but not captured. Cleanup kills processes but leaves windows. Need: capture window IDs, store in runtime dir, close only tracked windows on cleanup.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T09:05:49Z
last_update: 2026-03-12T00:38:50Z
date_finished: 2026-03-12T00:38:50Z
---

# T-074: Spawn cleanup — track and close Terminal.app windows on test exit

## Context

Discovered during T-063 reflection fleet: spawning 10 agents left 31 Terminal.app windows open. Attempting to close all windows via AppleScript killed the user's own sessions too. The spawn command returns window IDs (`tab 1 of window id 7340`) but they're never captured for cleanup. Related: T-071 (e2e portability).

## Acceptance Criteria

### Agent
- [x] E2e test cleanup functions close spawned Terminal.app windows (not all windows)
- [x] Window IDs captured from `termlink spawn` output and stored in `$RUNTIME_DIR/window-ids.txt`
- [x] Cleanup uses stored window IDs to close only tracked windows via AppleScript
- [x] All 6 e2e tests (level1-level6) updated with shared e2e-helpers.sh

### Human
- [x] [RUBBER-STAMP] — human delegated closure Run a multi-agent test, verify spawned windows close on exit, verify your own terminal stays open
  **Steps:**
  1. Note how many Terminal.app windows you have before the test
  2. Run `./tests/e2e/level5-role-specialists.sh`
  3. After test completes, verify spawned windows are closed and yours remain
  **Expected:** Only test-spawned windows close; your windows untouched
  **If not:** Report which windows were incorrectly closed

## Verification

# Shared helpers exist with window tracking
grep -q "window-ids" tests/e2e/e2e-helpers.sh
grep -q "spawn_tracked" tests/e2e/e2e-helpers.sh
grep -q "cleanup_all" tests/e2e/e2e-helpers.sh
# All 6 tests source the helpers
grep -q "e2e-helpers.sh" tests/e2e/level1-echo.sh
grep -q "e2e-helpers.sh" tests/e2e/level2-file-task.sh
grep -q "e2e-helpers.sh" tests/e2e/level3-persistent-agent.sh
grep -q "e2e-helpers.sh" tests/e2e/level4-multi-specialist.sh
grep -q "e2e-helpers.sh" tests/e2e/level5-role-specialists.sh
grep -q "e2e-helpers.sh" tests/e2e/level6-reflection-fleet.sh

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

### 2026-03-10T09:05:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-074-spawn-cleanup--track-and-close-terminala.md
- **Context:** Initial task creation

### 2026-03-10T09:18:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T09:21:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-12T00:38:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
