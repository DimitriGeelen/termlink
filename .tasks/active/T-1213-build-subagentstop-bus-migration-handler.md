---
id: T-1213
name: "Build SubagentStop bus-migration handler (T-1209 follow-up)"
description: >
  Implement per T-1209 GO: S1' spike first (test if non-zero SubagentStop exit mutates orchestrator-visible response); then bus-migration handler — over-threshold (T=8KB) returns auto-migrate to fw bus, orchestrator sees R-NNN pointer. Retires advisory check-dispatch.sh when live. Goal is no information loss. See docs/reports/T-1209-subagentstop-hook-inception.md.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [hook, dispatch, bus, framework-bridge]
components: []
related_tasks: [T-1209, T-175]
created: 2026-04-24T10:05:14Z
last_update: 2026-04-24T10:05:14Z
date_finished: null
---

# T-1213: Build SubagentStop bus-migration handler (T-1209 follow-up)

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

### 2026-04-24T10:05:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1213-build-subagentstop-bus-migration-handler.md
- **Context:** Initial task creation
