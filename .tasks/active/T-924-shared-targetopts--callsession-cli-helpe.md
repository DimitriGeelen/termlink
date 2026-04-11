---
id: T-924
name: "Shared TargetOpts + call_session CLI helper for cross-host routing"
description: >
  T-921 Spike 3/4 prereq 2 of 2: create cli/src/target.rs with TargetOpts (derived via clap::Args so it composes into any command), a call_session(opts, method, params) async helper that routes through connect_remote_hub + session.forward (when --target set) or client::rpc_call + manager::find_session (when not), and secret-file lookup from ~/.termlink/secrets/<host>.hex. Depends on T-923. Tests: unit tests for validation paths (mirror T-919's pattern for connect_remote_hub).

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-04-11T20:34:01Z
last_update: 2026-04-11T20:34:01Z
date_finished: null
---

# T-924: Shared TargetOpts + call_session CLI helper for cross-host routing

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

### 2026-04-11T20:34:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-924-shared-targetopts--callsession-cli-helpe.md
- **Context:** Initial task creation
