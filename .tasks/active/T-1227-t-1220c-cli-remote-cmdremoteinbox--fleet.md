---
id: T-1227
name: "T-1220c: CLI remote cmd_remote_inbox_* + fleet-doctor migration (T-1220 wedge c)"
description: >
  Migrate remote CLI inbox verbs in crates/termlink-cli/src/commands/remote.rs (@1255/1288/1328 + fleet doctor @2810) to use the T-1220a helper. 4 call sites. Blocked on T-1220a.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-1155, bus, migration, T-1220, wedge-c]
components: []
related_tasks: [T-1220]
created: 2026-04-25T07:00:14Z
last_update: 2026-04-25T07:00:14Z
date_finished: null
---

# T-1227: T-1220c: CLI remote cmd_remote_inbox_* + fleet-doctor migration (T-1220 wedge c)

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

### 2026-04-25T07:00:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1227-t-1220c-cli-remote-cmdremoteinbox--fleet.md
- **Context:** Initial task creation
