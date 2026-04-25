---
id: T-1231
name: "T-1220 follow-up: extend list_with_fallback to accept authenticated RpcClient"
description: >
  T-1225 helper currently takes &TransportAddr and creates new connections via Client::connect_addr (no auth). Wedge-c (CLI remote cmd_remote_inbox_*) and wedge-d-remote (MCP termlink_remote_inbox_*) use connect_remote_hub which produces an authenticated RpcClient. Helper needs an alternative entry point that accepts &mut RpcClient + caller-supplied capabilities — or refactor to dependency-inject the dispatch closure. Blocks T-1227 and the remote half of T-1228.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-1155, bus, migration, T-1220, T-1225-followup]
components: []
related_tasks: []
created: 2026-04-25T08:27:29Z
last_update: 2026-04-25T08:27:29Z
date_finished: null
---

# T-1231: T-1220 follow-up: extend list_with_fallback to accept authenticated RpcClient

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

### 2026-04-25T08:27:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1231-t-1220-follow-up-extend-listwithfallback.md
- **Context:** Initial task creation
