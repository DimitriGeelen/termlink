---
id: T-1288
name: "T-243 dep: Document multi-turn dialog convention catalog"
description: >
  Document the convention-layer event types (turn, typing, receipt, presence, member) and recommended subscriber patterns built on channel.post with metadata.conversation_id. Cover: 2-agent dialog, N-agent collaboration, human-in-the-loop, confirmation flows. No code — pure documentation. Output to docs/protocols/multi-turn-dialog.md or similar. Depends on metadata-extension child task being landed.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-243, docs, protocol]
components: [crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-protocol/src/control.rs]
related_tasks: []
created: 2026-04-26T09:32:05Z
last_update: 2026-04-26T11:05:37Z
date_finished: 2026-04-26T11:05:37Z
---

# T-1288: T-243 dep: Document multi-turn dialog convention catalog

## Context

Per T-243 inception synthesis (Agent C minimal-surface path): the multi-turn dialog primitive is **convention over channel.\***, not a new typed namespace. T-1287 added optional `metadata.conversation_id` + `metadata.event_type`. T-1289 added long-poll. T-1286 added `dialog.presence`. With those wedges in place, agents can compose multi-turn dialogs using only:

- `channel.post` (with metadata) for sending
- `channel.subscribe` (with `conversation_id` filter + `timeout_ms`) for receiving
- `dialog.presence` for "who's here"

But conventions only work if everyone reads the same playbook. This task delivers the playbook so future implementers don't have to reverse-engineer the metadata catalog from source.

## Acceptance Criteria

### Agent
- [x] New file `docs/conventions/multi-turn-dialog.md` exists
- [x] Documents the 5 well-known `metadata.event_type` values (`turn`, `typing`, `receipt`, `presence`, `member`) — what each means, who emits it, when, and what subscribers do with it
- [x] Documents the `metadata.conversation_id` convention (string, opaque to hub, agents agree on format) and how subscribers filter on it
- [x] Includes a worked example: 2-agent dialog (request + heartbeat + response) with concrete JSON-RPC params for each step
- [x] Includes one N-agent example (3-agent roundtable) showing how `dialog.presence` is used
- [x] Includes a heartbeat-as-infrastructure section with the load-bearing chain explained
- [x] References T-1285 (oldest_offset / gap detection) and T-1289 (long-poll) for resume-after-disconnect
- [x] Cross-linked from `docs/conventions/agent-delegation-events.md` (`See also` section)

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

### 2026-04-26T09:32:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1288-t-243-dep-document-multi-turn-dialog-con.md
- **Context:** Initial task creation

### 2026-04-26T11:03:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T11:05:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
