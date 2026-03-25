---
id: T-273
name: "termlink doctor — runtime health check command"
description: >
  termlink doctor — runtime health check command

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-25T11:55:53Z
last_update: 2026-03-25T11:55:53Z
date_finished: null
---

# T-273: termlink doctor — runtime health check command

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `termlink doctor` command added with 6 health checks
- [x] Checks: runtime_dir, sessions_dir, session liveness (ping), hub status, orphaned sockets, version
- [x] Human-readable output with colored pass/warn/fail markers
- [x] `--json` flag for machine-readable output
- [x] Exit code 1 on failures, 0 on pass/warn
- [x] All 459 workspace tests pass

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

grep -q "cmd_doctor" crates/termlink-cli/src/commands/infrastructure.rs
grep -q "Doctor" crates/termlink-cli/src/cli.rs

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

### 2026-03-25T11:55:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-273-termlink-doctor--runtime-health-check-co.md
- **Context:** Initial task creation
