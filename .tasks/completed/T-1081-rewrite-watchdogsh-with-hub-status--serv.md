---
id: T-1081
name: "Rewrite watchdog.sh with hub status + service checks + logging"
description: >
  Rewrite watchdog.sh with hub status + service checks + logging

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T05:37:56Z
last_update: 2026-04-16T05:41:13Z
date_finished: 2026-04-16T05:41:13Z
---

# T-1081: Rewrite watchdog.sh with hub status + service checks + logging

## Context

Duplicate of T-1072 which already shipped `scripts/watchdog.sh` + `/etc/cron.d/termlink-watchdog` (1-minute cron checking hub + both agent services). Closing as already-done.

## Acceptance Criteria

### Agent
- [x] Already shipped under T-1072 — watchdog.sh checks termlink-hub.service, termlink-framework-agent.service, termlink-termlink-agent.service every minute

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

### 2026-04-16T05:37:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1081-rewrite-watchdogsh-with-hub-status--serv.md
- **Context:** Initial task creation

### 2026-04-16T05:41:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
