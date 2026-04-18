---
id: T-1118
name: "Remove duplicate fleet widgets from home and cockpit (now in ambient strip)"
description: >
  Remove duplicate fleet widgets from home and cockpit (now in ambient strip)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T08:55:50Z
last_update: 2026-04-18T08:57:48Z
date_finished: 2026-04-18T08:57:48Z
---

# T-1118: Remove duplicate fleet widgets from home and cockpit (now in ambient strip)

## Context

T-1117 added the fleet indicator to the ambient strip (visible on every page). The fleet widgets on / (index.html) and /cockpit (cockpit.html) added in T-1116 are now redundant — they show the same data the ambient strip shows. Removing them reduces UI clutter and avoids stale-data inconsistency.

## Acceptance Criteria

### Agent
- [x] index.html no longer contains the fleet-widget div or its fetch script
- [x] cockpit.html no longer contains the fleet-widget div or its fetch script
- [x] Home (/) loads without errors and ambient strip still shows fleet
- [x] Cockpit (/cockpit) loads without errors and ambient strip still shows fleet

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

curl -sf http://localhost:3000/ > /dev/null
test "$(curl -sf http://localhost:3000/ | grep -c 'fleet-widget')" = "0"
test "$(curl -sf http://localhost:3000/ | grep -c 'ambient-fleet')" -ge "1"
! grep -q 'fleet-widget' .agentic-framework/web/templates/index.html
! grep -q 'fleet-widget' .agentic-framework/web/templates/cockpit.html

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

### 2026-04-18T08:55:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1118-remove-duplicate-fleet-widgets-from-home.md
- **Context:** Initial task creation

### 2026-04-18T08:57:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
