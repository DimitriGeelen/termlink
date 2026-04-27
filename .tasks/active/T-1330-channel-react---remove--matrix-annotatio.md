---
id: T-1330
name: "channel react --remove — Matrix annotation removal (alias for redact on a reaction offset)"
description: >
  channel react --remove — Matrix annotation removal (alias for redact on a reaction offset)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T16:42:34Z
last_update: 2026-04-27T16:42:34Z
date_finished: null
---

# T-1330: channel react --remove — Matrix annotation removal (alias for redact on a reaction offset)

## Context

Matrix annotation removal: a reactor undoes their reaction by emitting an `m.redaction`
targeting the `m.annotation` event. We have `channel redact <offset>` (T-1322) but
operators don't know the reaction's offset — they know the parent and the emoji they
reacted with. Add `channel react <topic> <parent_offset> <reaction> --remove` that walks
the topic, finds the latest matching reaction (msg_type=reaction, sender_id=me,
in_reply_to=parent, payload=reaction), and emits a redaction targeting that offset.

## Acceptance Criteria

### Agent
- [x] `cli.rs` React variant gains `--remove: bool` flag.
- [x] `cmd_channel_react` takes `remove: bool`. When true, walks topic to find latest matching reaction by (sender, parent, payload) and emits redaction; on no-match returns clear error.
- [x] Pure helper `find_my_reaction_offset(msgs, sender, parent, payload) -> Option<u64>` with unit tests covering: latest-wins among multiple, no-match returns None, sender-mismatch ignored, parent-mismatch ignored, payload-mismatch ignored.
- [x] `cargo test -p termlink --bins find_my_reaction_offset` passes.
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] E2E: append step 12 to `tests/e2e/agent-conversation.sh` — alice reacts then `--remove`s, default `--reactions` view no longer shows that reaction.
- [x] `bash tests/e2e/agent-conversation.sh` passes (12 steps).

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo test -p termlink --bins find_my_reaction_offset
cargo clippy --all-targets --workspace -- -D warnings
bash tests/e2e/agent-conversation.sh

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

### 2026-04-27T16:42:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1330-channel-react---remove--matrix-annotatio.md
- **Context:** Initial task creation
