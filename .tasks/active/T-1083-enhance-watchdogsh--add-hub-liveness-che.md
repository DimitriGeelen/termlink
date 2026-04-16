---
id: T-1083
name: "Enhance watchdog.sh — add hub liveness check after systemd check"
description: >
  Enhance watchdog.sh — add hub liveness check after systemd check

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T18:30:35Z
last_update: 2026-04-16T18:30:35Z
date_finished: null
---

# T-1083: Enhance watchdog.sh — add hub liveness check after systemd check

## Context

Watchdog only checks systemd units. Add hub liveness ping after systemd check — catches zombie/hung hub process.

## Acceptance Criteria

### Agent
- [x] Hub liveness check via `termlink ping` + `hub status` after systemd check
- [x] If hub is active but not responding, restart it
- [x] Liveness check has 5s timeout (no blocking cron)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-16T18:30:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1083-enhance-watchdogsh--add-hub-liveness-che.md
- **Context:** Initial task creation
