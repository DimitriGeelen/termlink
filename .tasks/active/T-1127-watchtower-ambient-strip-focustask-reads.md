---
id: T-1127
name: "Watchtower ambient strip focus_task reads first-alphabetical active task, not actual focus.yaml"
description: >
  Watchtower ambient strip focus_task reads first-alphabetical active task, not actual focus.yaml

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T15:56:24Z
last_update: 2026-04-18T15:57:33Z
date_finished: 2026-04-18T15:57:33Z
---

# T-1127: Watchtower ambient strip focus_task reads first-alphabetical active task, not actual focus.yaml

## Context

`shared.py:build_ambient()` currently picks the lowest-numbered active task file (e.g., T-160 from weeks ago) and calls it "focus". The real focus is recorded in `.context/working/focus.yaml` under `current_task:`. Operators see stale/wrong task in the header, which undermines trust in the ambient strip (found while validating T-1123 fix).

## Acceptance Criteria

### Agent
- [x] `build_ambient()` reads `PROJECT_ROOT/.context/working/focus.yaml` for `current_task`
- [x] Falls back to first-sorted active task only if focus.yaml missing/empty
- [x] Ambient strip on `/` shows T-1127 (current focus) after Watchtower reload (verified)

### Human
- [ ] [REVIEW] Verify ambient strip tracks real focus
  **Steps:**
  1. `fw work-on T-1127`
  2. Reload `http://localhost:3000/`
  3. Check ambient strip's focus-task link
  **Expected:** Shows `T-1127` (the task in focus.yaml)
  **If not:** Check the Watchtower log for errors reading focus.yaml


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, playwright, ambient-strip-focus):** Ambient strip shows `T-1143` as the focus task (verified via playwright snapshot of `/` and `/fleet`). `cat .context/working/focus.yaml` confirms `task_id: T-1143` — ambient strip matches actual focus.yaml, not first-alphabetical active task. REVIEW-approvable.

## Verification

# Shell commands that MUST pass before work-completed.
python3 -c "import ast; ast.parse(open('.agentic-framework/web/shared.py').read())"
grep -q "focus.yaml" .agentic-framework/web/shared.py

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

### 2026-04-18T15:56:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1127-watchtower-ambient-strip-focustask-reads.md
- **Context:** Initial task creation

### 2026-04-18T15:57:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-18T19:46Z — evidence [agent]
- **Action:** Curled http://localhost:3000/ after `fw work-on T-1071` set focus to T-1071.
- **Result:** Ambient strip rendered shows `T-1071` (the focus.yaml current_task value), not the first-alphabetical active task. Verifies the fix tracks real focus.
- **Suggest:** Human can check the REVIEW box.
