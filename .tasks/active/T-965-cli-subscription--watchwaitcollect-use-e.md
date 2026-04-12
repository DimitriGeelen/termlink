---
id: T-965
name: "CLI subscription — watch/wait/collect use event.subscribe with poll fallback"
description: >
  T-690 Phase 3-4: Update watch, wait, collect, and dispatch CLI commands to use event.subscribe when available, falling back to event.poll for older sessions.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:12:43Z
last_update: 2026-04-12T09:12:43Z
date_finished: null
---

# T-965: CLI subscription — watch/wait/collect use event.subscribe with poll fallback

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `watch` command uses event.subscribe with poll fallback (events.rs:429)
- [x] `wait` command uses event.subscribe with poll fallback (events.rs:577)
- [x] Existing poll behavior unchanged when event.subscribe not available
- [x] cargo test passes (39 event tests)

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

### 2026-04-12T09:12:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-965-cli-subscription--watchwaitcollect-use-e.md
- **Context:** Initial task creation
