---
id: T-1072
name: "Cron watchdog for termlink supervisor stack — catches clean-exit drops"
description: >
  Cron watchdog for termlink supervisor stack — catches clean-exit drops

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:22:18Z
last_update: 2026-04-15T21:23:43Z
date_finished: 2026-04-15T21:23:43Z
---

# T-1072: Cron watchdog for termlink supervisor stack — catches clean-exit drops

## Context

Systemd supervises hub + 2 agent services with `Restart=on-failure`. That catches crashes but NOT clean exits (observed: framework-agent + termlink-agent exited clean 2 days ago and stayed down). Cron watchdog every minute catches clean-exit drop-outs by re-starting any inactive unit. `@reboot` handled by systemd (units are `enabled`), no cron @reboot needed.

## Acceptance Criteria

### Agent
- [x] `scripts/watchdog.sh` exists, executable, checks 3 units
- [x] `/etc/cron.d/termlink-watchdog` runs the watchdog every minute
- [x] All 3 services verified active via `systemctl is-active`

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

test -x scripts/watchdog.sh
test -f /etc/cron.d/termlink-watchdog
systemctl is-active --quiet termlink-hub.service
systemctl is-active --quiet termlink-framework-agent.service
systemctl is-active --quiet termlink-termlink-agent.service

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

### 2026-04-15T21:22:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1072-cron-watchdog-for-termlink-supervisor-st.md
- **Context:** Initial task creation

### 2026-04-15T21:23:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
