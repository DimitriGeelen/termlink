---
id: T-808
name: "Wire --since flag to events subscribe CLI command"
description: >
  Wire --since flag to events subscribe CLI command

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T17:53:48Z
last_update: 2026-03-30T17:55:28Z
date_finished: null
---

# T-808: Wire --since flag to events subscribe CLI command

## Context

Investigation: CLI has no `subscribe` subcommand. `event.subscribe` is an RPC-level API consumed by programmatic clients and the MCP server, not directly by CLI users. `watch` uses `event.poll` in a loop. No CLI wiring needed — the T-805 since parameter is already available to RPC callers.

## Acceptance Criteria

### Agent
- [x] Verified no CLI subscribe command exists (RPC-level only)
- [x] Confirmed watch uses event.poll, not event.subscribe

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

# No changes — investigation only
true

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

### 2026-03-30T17:53:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-808-wire---since-flag-to-events-subscribe-cl.md
- **Context:** Initial task creation
