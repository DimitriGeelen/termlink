---
id: T-1230
name: "T-1220b follow-up: cmd_inbox_clear semantic split (Q4 spool deletion)"
description: >
  Migrate cmd_inbox_clear with Q4 semantic split: legacy inbox.clear deletes spool files on hub disk; channel-backed clear advances local cursor only (no hub mutation). Either keep legacy + add cursor reset, or design hub channel.trim RPC. Split from T-1226 because semantic change requires explicit design discussion before edits land.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [T-1155, bus, migration, T-1220, wedge-b-followup]
components: []
related_tasks: []
created: 2026-04-25T08:24:33Z
last_update: 2026-04-25T08:24:33Z
date_finished: null
---

# T-1230: T-1220b follow-up: cmd_inbox_clear semantic split (Q4 spool deletion)

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

### 2026-04-25T08:24:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1230-t-1220b-follow-up-cmdinboxclear-semantic.md
- **Context:** Initial task creation
