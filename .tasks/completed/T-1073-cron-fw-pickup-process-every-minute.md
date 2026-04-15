---
id: T-1073
name: "Cron fw pickup process every minute"
description: >
  Cron fw pickup process every minute

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:29:10Z
last_update: 2026-04-15T21:29:48Z
date_finished: 2026-04-15T21:29:48Z
---

# T-1073: Cron fw pickup process every minute

## Context

`fw pickup process` drains `.context/pickup/inbox/` and routes envelopes to inception/task flows. No schedule exists today — envelopes accumulate until a human or handover hook runs the processor. User ask: schedule it every minute so cross-project envelopes land quickly. Mirrors the existing `agentic-audit-termlink` cron pattern.

## Acceptance Criteria

### Agent
- [x] `/etc/cron.d/agentic-pickup-termlink` installed, runs every minute
- [x] Cron entry uses `PROJECT_ROOT="/opt/termlink"` inline (same pattern as agentic-audit-termlink)
- [x] First run observed in syslog within 2 minutes

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

test -f /etc/cron.d/agentic-pickup-termlink
grep -q "fw.*pickup.*process" /etc/cron.d/agentic-pickup-termlink

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

### 2026-04-15T21:29:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1073-cron-fw-pickup-process-every-minute.md
- **Context:** Initial task creation

### 2026-04-15T21:29:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
