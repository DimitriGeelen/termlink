---
id: T-1220
name: "T-1163 follow-up: CLI/MCP inbox receiver migration to channel.{subscribe,list}"
description: >
  Receiver-side migration following T-1163's hub dual-write shim. CLI verbs 'inbox {list,status,clear}' + MCP tools termlink_inbox_* + remote inbox verbs switch to channel.{subscribe,list} on topic 'inbox:<target>' with capabilities fallback to legacy inbox.* when peer lacks channel API.

status: captured
workflow_type: refactor
owner: agent
horizon: next
tags: [T-1155, bus, migration, T-1163-followup]
components: []
related_tasks: []
created: 2026-04-24T15:10:01Z
last_update: 2026-04-24T15:10:01Z
date_finished: null
---

# T-1220: T-1163 follow-up: CLI/MCP inbox receiver migration to channel.{subscribe,list}

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

### 2026-04-24T15:10:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1220-t-1163-follow-up-climcp-inbox-receiver-m.md
- **Context:** Initial task creation
