---
id: T-1117
name: "Add fleet indicator to ambient status strip — every-page visibility"
description: >
  Add fleet indicator to ambient status strip — every-page visibility

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T08:50:02Z
last_update: 2026-04-18T08:52:40Z
date_finished: 2026-04-18T08:52:40Z
---

# T-1117: Add fleet indicator to ambient status strip — every-page visibility

## Context

Ambient strip currently shows focus task, session age, audit, attention count. Adding fleet (up/total + dot color) makes operational state visible on every page, not just /fleet. Loads async via existing /api/fleet/status to avoid blocking page render.

## Acceptance Criteria

### Agent
- [x] base.html ambient strip includes fleet element with dot + text, links to /fleet
- [x] Fleet element loads async via fetch('/api/fleet/status') and stays hidden until data arrives
- [x] Dot color reflects state: green (all up), orange (degraded — some up), red (all down/error), grey (loading/no hubs)
- [x] Text shows "N/M up" with optional "· N down" / "· N auth-fail" suffixes
- [x] All existing pages still render without errors

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

# Verification commands
curl -sf http://localhost:3000/ > /dev/null
curl -sf http://localhost:3000/ | grep -q 'ambient-fleet'
curl -sf http://localhost:3000/api/fleet/status | python3 -c "import sys,json; json.load(sys.stdin)"

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

### 2026-04-18T08:50:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1117-add-fleet-indicator-to-ambient-status-st.md
- **Context:** Initial task creation

### 2026-04-18T08:52:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
