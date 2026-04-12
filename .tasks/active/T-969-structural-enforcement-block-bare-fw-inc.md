---
id: T-969
name: "Structural enforcement: block bare fw inception decide in agent output — force fw task review"
description: >
  PL-007 structural fix: Add a PostToolUse hook or output filter that detects when an agent outputs 'fw inception decide' as a suggestion to the user. Warn/block and suggest fw task review instead. Converts aspirational rule into structural gate (P-002).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:47:53Z
last_update: 2026-04-12T09:47:53Z
date_finished: null
---

# T-969: Structural enforcement: block bare fw inception decide in agent output — force fw task review

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `fw inception decide` auto-invokes `fw task review` when no review marker exists (instead of just blocking)
- [x] Human always sees full Watchtower experience (link, QR, recommendation, terminal command)
- [x] `fw task review` still works normally (no change needed)
- [x] PL-007 codified in CLAUDE.md as behavioral rule
- [ ] Pickup sent to framework agent for upstream inclusion

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

### 2026-04-12T09:47:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-969-structural-enforcement-block-bare-fw-inc.md
- **Context:** Initial task creation
