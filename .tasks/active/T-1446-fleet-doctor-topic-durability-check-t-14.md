---
id: T-1446
name: "fleet doctor: topic-durability check (T-1444 follow-up)"
description: >
  Add a fleet-doctor diagnostic that for each reachable hub remote-execs an audit of <runtime_dir>/bus/meta.db (presence + non-/tmp + recent mtime). Closes G-050.what_remains 'periodic sweep' ask. Out of scope for T-1444 (NO-GO inception). Probe-first pattern: if remote-exec available use it; else skip with hint. Similar in shape to --legacy-usage extension (T-1432).

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: [T-1444, T-1432, T-1438]
created: 2026-05-02T05:47:14Z
last_update: 2026-05-02T05:47:14Z
date_finished: null
---

# T-1446: fleet doctor: topic-durability check (T-1444 follow-up)

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

### 2026-05-02T05:47:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1446-fleet-doctor-topic-durability-check-t-14.md
- **Context:** Initial task creation
