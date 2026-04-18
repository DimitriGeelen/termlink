---
id: T-1121
name: "Show session ID as tooltip on ambient strip session age"
description: >
  Show session ID as tooltip on ambient strip session age

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:36:47Z
last_update: 2026-04-18T09:38:31Z
date_finished: 2026-04-18T09:38:31Z
---

# T-1121: Show session ID as tooltip on ambient strip session age

## Context

The ambient strip shows "Session: 2h ago" with no way to know which session that refers to. Adding the session ID as a tooltip on the session-age span gives operators that context without taking strip space.

## Acceptance Criteria

### Agent
- [x] build_ambient adds session_id derived from latest handover filename (e.g., S-2026-0418-1100)
- [x] base.html session-age span has title attribute showing the session ID
- [x] Pages still render

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
curl -sf http://localhost:3000/ | grep -qE 'title="S-[0-9]+'

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

### 2026-04-18T09:36:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1121-show-session-id-as-tooltip-on-ambient-st.md
- **Context:** Initial task creation

### 2026-04-18T09:38:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
