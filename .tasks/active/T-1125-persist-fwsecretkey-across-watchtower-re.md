---
id: T-1125
name: "Persist FW_SECRET_KEY across watchtower restarts (fix CSRF 403 after restart)"
description: >
  Watchtower's FW_SECRET_KEY auto-generates on every restart (app.py:50), invalidating all existing browser session cookies and CSRF tokens. Users hit '403 Forbidden — CSRF token missing or invalid' on any POST form (Record decision, task updates, etc.) if their page was loaded before the restart. Workaround: refresh the page. Fix: persist FW_SECRET_KEY in .context/working/.fw-secret-key (chmod 600) and load on startup, or document setting it in the systemd unit / fw watchtower start.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [watchtower, security, csrf]
components: []
related_tasks: []
created: 2026-04-18T10:01:16Z
last_update: 2026-04-18T10:01:24Z
date_finished: null
---

# T-1125: Persist FW_SECRET_KEY across watchtower restarts (fix CSRF 403 after restart)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

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

### 2026-04-18T10:01:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1125-persist-fwsecretkey-across-watchtower-re.md
- **Context:** Initial task creation
