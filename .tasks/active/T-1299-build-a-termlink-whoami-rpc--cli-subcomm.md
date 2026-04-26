---
id: T-1299
name: "Build A: termlink whoami RPC + CLI subcommand"
description: >
  Per T-1297 GO: read-only RPC that returns the calling session's identity card (id, display_name, roles, tags, cwd, pid, hub_address). Disambiguator chain: TERMLINK_SESSION_ID env var (primary, set by termlink register) → source-PID tree-walk fallback → ambiguous-result hint with candidates list. Pure exposure of existing session registry — no new data model. Estimate: ½ dev-day. Reversible: additive RPC. Forward-compat: older binaries return Method-not-found cleanly. Evidence: docs/reports/T-1297-termlink-agent-routing-discipline.md § Spike 2.

status: captured
workflow_type: build
owner: human
horizon: next
tags: [termlink, routing, whoami, T-1297-child, hub-rpc]
components: []
related_tasks: [T-1297]
created: 2026-04-26T21:19:36Z
last_update: 2026-04-26T21:19:36Z
date_finished: null
---

# T-1299: Build A: termlink whoami RPC + CLI subcommand

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

### 2026-04-26T21:19:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1299-build-a-termlink-whoami-rpc--cli-subcomm.md
- **Context:** Initial task creation
