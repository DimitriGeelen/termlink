---
id: T-1251
name: "T-1164d Legacy file.* event deprecation + inbox.rs cleanup + PL-011 closure"
description: >
  Mark legacy file.init/chunk/complete event-name path as deprecated, integrate blob GC with retention engine (T-1158), close PL-011 (send-file delivery confirmation) with structural-fix evidence pointing at T-1164. Depends on T-1164b + T-1164c.

status: captured
workflow_type: decommission
owner: agent
horizon: next
tags: [T-1164, T-1155, bus, artifact, PL-011]
components: []
related_tasks: [T-1164, T-1164b, T-1164c, T-1158]
created: 2026-04-25T11:43:54Z
last_update: 2026-04-25T11:43:54Z
date_finished: null
---

# T-1251: T-1164d Legacy file.* event deprecation + inbox.rs cleanup + PL-011 closure

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

### 2026-04-25T11:43:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1251-t-1164d-legacy-file-event-deprecation--i.md
- **Context:** Initial task creation
