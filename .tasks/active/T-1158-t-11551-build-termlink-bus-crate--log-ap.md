---
id: T-1158
name: "T-1155/1 Build termlink-bus crate — log-append + cursor + subscribe + retention"
description: >
  Foundation crate for T-1155 channel bus. Append-only per-channel log, per-recipient cursor store, subscribe API, per-channel retention engine. In-hub. See docs/reports/T-1155-agent-communication-bus.md §Recommendation.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [T-1155, bus, foundation]
components: []
related_tasks: [T-1155]
created: 2026-04-20T14:11:33Z
last_update: 2026-04-20T14:11:33Z
date_finished: null
---

# T-1158: T-1155/1 Build termlink-bus crate — log-append + cursor + subscribe + retention

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

### 2026-04-20T14:11:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1158-t-11551-build-termlink-bus-crate--log-ap.md
- **Context:** Initial task creation
