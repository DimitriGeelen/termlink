---
id: T-1285
name: "T-243 dep: Audit + harden channel.* ordered durable delivery"
description: >
  Verify channel.* guarantees ordered durable delivery under: (a) concurrent multi-publisher writes, (b) hub restart, (c) subscriber reconnect mid-stream. This is Agent A's NO-GO check from T-243 inception and Agent C's crash gap. If gaps found, fix before any dialog.heartbeat or metadata-extension work proceeds — append-only log model collapses without it. Foundation for all other T-243 child tasks.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-243, channel, reliability]
components: []
related_tasks: []
created: 2026-04-26T09:31:53Z
last_update: 2026-04-26T09:31:53Z
date_finished: null
---

# T-1285: T-243 dep: Audit + harden channel.* ordered durable delivery

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

### 2026-04-26T09:31:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1285-t-243-dep-audit--harden-channel-ordered-.md
- **Context:** Initial task creation
