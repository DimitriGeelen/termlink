---
id: T-1064
name: "Investigate .109 + .121 hub outage — both down 2026-04-15 ~15:50Z"
description: >
  ring20-management (.109) and ring20-dashboard (.121) both fail termlink fleet doctor as of 2026-04-15 ~15:50Z. Diagnostics: ping OK on both, but port 9100 connection refused on .121 and timing-out on .109. T-1027 reported both running at session-start two days ago. Operator action: SSH in, check systemd hub service status on both hosts. If restart policy not deployed there, see T-931..T-935. Registered from T-1061 housekeeping session. No code fix needed — this is operational.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T17:09:39Z
last_update: 2026-04-15T17:09:39Z
date_finished: null
---

# T-1064: Investigate .109 + .121 hub outage — both down 2026-04-15 ~15:50Z

## Context

**UPDATE 2026-04-15T17:25Z (user report):** ".109 has become .126" — ring20-management container renumbered. Verified: ping .126 OK (0.15ms), .109 no longer responds, port 9100 still refused on .126 (hub process down). .121 still timing out. **Scope revision:** .109 is not "down" — it's gone. The container migrated. Client profile updated by T-1065. Remaining work here: (1) start the hub process on .126, (2) investigate .121 (may also be renumbered — operator to confirm).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-15T17:09:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1064-investigate-109--121-hub-outage--both-do.md
- **Context:** Initial task creation
