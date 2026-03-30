---
id: T-803
name: "Add next_seq to event.subscribe response"
description: >
  Add next_seq to event.subscribe response for cursor-based following

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-session/src/handler.rs]
related_tasks: []
created: 2026-03-30T16:19:41Z
last_update: 2026-03-30T16:21:42Z
date_finished: 2026-03-30T16:21:42Z
---

# T-803: Add next_seq to event.subscribe response

## Context

`event.poll` returns `next_seq` for cursor-based following. `event.subscribe` doesn't, making it harder for clients to track sequence position. Add `next_seq` (max seq + 1 of returned events) to subscribe response.

## Acceptance Criteria

### Agent
- [x] event.subscribe response includes `next_seq` field
- [x] Existing event.subscribe tests still pass
- [x] Test assertions verify next_seq present (events) and absent (empty)

## Verification

grep -q "next_seq" crates/termlink-session/src/handler.rs

## Updates
### Not applicable
<!-- Deleted Human AC section — all agent-verifiable.
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

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-30T16:19:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-803-add-nextseq-to-eventsubscribe-response.md
- **Context:** Initial task creation

### 2026-03-30T16:19:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T16:21:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
