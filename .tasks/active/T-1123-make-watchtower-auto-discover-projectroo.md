---
id: T-1123
name: "Make Watchtower auto-discover PROJECT_ROOT (don't require env var)"
description: >
  Watchtower defaults PROJECT_ROOT to FRAMEWORK_ROOT when env not set. This makes ambient strip read framework's own .context/ instead of project's. Have shared.py walk up from CWD looking for .context/ + .tasks/ to identify the project root, fall back to FRAMEWORK_ROOT only if no project found.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:49:49Z
last_update: 2026-04-18T15:52:31Z
date_finished: 2026-04-18T15:52:31Z
---

# T-1123: Make Watchtower auto-discover PROJECT_ROOT (don't require env var)

## Context

`shared.py:22` resolves `PROJECT_ROOT = env["PROJECT_ROOT"] or FRAMEWORK_ROOT`. When env is unset, the fallback points at the vendored framework, so the ambient strip and every task/handover/audit view reads the framework's own state, not the consumer project's. Operators have to remember `PROJECT_ROOT=/opt/termlink` on every invocation (see T-1121 / last session debugging).

Fix: walk up from CWD looking for a project marker (`.framework.yaml`), then fall back to FRAMEWORK_ROOT only if nothing is found.

## Acceptance Criteria

### Agent
- [x] `shared.py` has `_discover_project_root()` that walks up from CWD for `.framework.yaml`
- [x] Resolution order: env `PROJECT_ROOT` > walk-up from CWD > FRAMEWORK_ROOT fallback
- [x] Discovery stops at filesystem root without looping
- [x] Log line at import records which source supplied PROJECT_ROOT
- [x] Discovery from `/opt/termlink/somedir` finds `/opt/termlink` (verified)
- [x] Discovery from `/tmp` (no marker) falls back to FRAMEWORK_ROOT (verified)
- [x] Env var overrides discovery (verified)

### Human
- [ ] [REVIEW] Confirm ambient strip shows correct project
  **Steps:**
  1. `cd /opt/termlink && unset PROJECT_ROOT && /opt/termlink/.agentic-framework/bin/watchtower.sh restart`
  2. Open http://localhost:3000/
  3. Look at ambient strip project link
  **Expected:** Shows `010-termlink` (not `agentic-engineering-framework`)
  **If not:** Check startup log for `PROJECT_ROOT source:` line


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, playwright, project-root-autodiscover):** Watchtower ambient strip and page footer display project name `termlink` without any `PROJECT_ROOT` env var being passed to the server process (`ps -o cmd= -p $(pgrep -f watchtower)` shows no PROJECT_ROOT). Home page snapshot via playwright confirms `strong: termlink` in nav area. Auto-discovery working. REVIEW-approvable.

## Verification

# Shell commands that MUST pass before work-completed.
python3 -c "import ast; ast.parse(open('.agentic-framework/web/shared.py').read())"
grep -q "_discover_project_root\|_resolve_project_root" .agentic-framework/web/shared.py

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

### 2026-04-18T09:49:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1123-make-watchtower-auto-discover-projectroo.md
- **Context:** Initial task creation

### 2026-04-18T15:50:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-18T15:52:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-18T19:46Z — evidence [agent]
- **Action:** Curled http://localhost:3000/ ambient strip.
- **Result:** Active task list shows `T-160, T-163, T-173, T-174, T-175, T-176, T-180, T-1071, ...` — these are termlink (`/opt/termlink`) task IDs, not framework task IDs. PROJECT_ROOT auto-discovery resolved correctly to /opt/termlink (not falling back to FRAMEWORK_ROOT).
- **Suggest:** Human can check the REVIEW box.
