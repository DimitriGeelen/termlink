---
id: T-1287
name: "T-243 dep: Extend channel.post with metadata.conversation_id + metadata.event_type"
description: >
  Per T-243 inception synthesis: one-field code change in channel.post params. Optional metadata.conversation_id (string) — scope events to a conversation. Optional metadata.event_type (turn|typing|receipt|presence|member) — routing/filtering hint, not enforcement. Enables convention-layer multi-turn dialog without new typed-method namespace. Independently testable; can land before or after dialog.heartbeat.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-243, channel, protocol]
components: []
related_tasks: []
created: 2026-04-26T09:32:02Z
last_update: 2026-04-26T09:32:02Z
date_finished: null
---

# T-1287: T-243 dep: Extend channel.post with metadata.conversation_id + metadata.event_type

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

### 2026-04-26T09:32:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1287-t-243-dep-extend-channelpost-with-metada.md
- **Context:** Initial task creation
