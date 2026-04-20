---
id: T-1172
name: "T-1161 follow-up: termlink channel queue-status CLI verb"
description: >
  Optional CLI verb from T-1161 AC (punted to follow-up). Adds 'termlink channel queue-status' showing pending count + oldest timestamp from ~/.termlink/outbound.sqlite for operator debugging. ~40 LOC CLI + 1 MCP mirror (R-033).

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [T-1155, bus, cli]
components: []
related_tasks: [T-1161]
created: 2026-04-20T22:19:52Z
last_update: 2026-04-20T22:19:52Z
date_finished: null
---

# T-1172: T-1161 follow-up: termlink channel queue-status CLI verb

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

### 2026-04-20T22:19:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1172-t-1161-follow-up-termlink-channel-queue-.md
- **Context:** Initial task creation
