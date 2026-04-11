---
id: T-916
name: "Fix dispatch event.collect masking hub failures via continue path"
description: >
  Discovered 2026-04-11 while smoke-testing T-914 (G-002 fix). When the hub is down or unreachable, dispatch's collect loop in crates/termlink-cli/src/commands/dispatch.rs (around line 406) hits Connection refused on every event.collect RPC and falls through a 'continue' statement that skips the entire rest of the loop body — including the early-crash detection at lines 454-480. Result: dispatch hangs in a tight error loop until --timeout expires, with no signal to the user that the hub is unreachable. Confirmed reproduction: repeated 'I/O error: Connection refused (os error 111)' debug lines during a smoke test where the hub PID was dead but the socket file persisted on disk. RECOMMENDED FIX: move the early-crash check to the TOP of the collect loop (before event.collect) so it always runs regardless of RPC outcome. Additionally, track consecutive event.collect errors and bail with a clear 'hub unreachable' error after N consecutive failures (e.g., 5). PRE-FLIGHT OPTION: ping the hub once before entering the collect loop and fail fast with a clear error message. Symptom is identical to the G-002 fast-fail hang from the user's perspective (silent timeout) but the cause and fix are different. T-914 fix is correct; this is a separate orthogonal bug.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [termlink, dispatch, bug, observability, error-handling]
components: []
related_tasks: [T-914, T-282]
created: 2026-04-11T13:16:45Z
last_update: 2026-04-11T13:16:45Z
date_finished: null
---

# T-916: Fix dispatch event.collect masking hub failures via continue path

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

### 2026-04-11T13:16:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-916-fix-dispatch-eventcollect-masking-hub-fa.md
- **Context:** Initial task creation
