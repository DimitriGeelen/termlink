---
id: T-1323
name: "channel describe — set/get topic description (Matrix m.room.topic)"
description: >
  channel describe — set/get topic description (Matrix m.room.topic)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:24:33Z
last_update: 2026-04-27T15:24:33Z
date_finished: null
---

# T-1323: channel describe — set/get topic description (Matrix m.room.topic)

## Context

Matrix `m.room.topic` carries a free-text description of a room. We map this to
channels by emitting `msg_type=topic_metadata` with `metadata.description=<text>`.
Append-only — multiple description posts mean "latest wins" by ts_ms (same shape
as edits). Reader-side, T-1324 (`channel info`) will surface the latest one.
For T-1323 we just add the *emit* path + a tiny pure helper for "latest description"
that T-1324 can reuse.

## Acceptance Criteria

### Agent
- [x] `ChannelAction::Describe { topic, description, hub, json }` added to cli.rs
- [x] `cmd_channel_describe` in commands/channel.rs — emits `msg_type=topic_metadata`
      with `metadata.description=<text>`, payload mirrors metadata for human readability
- [x] Pure helper `latest_description(msgs) -> Option<(u64, String)>` returns the most
      recent (ts_ms, description) from `msg_type=topic_metadata` envelopes
- [x] Helper is also exposed by `pub(crate)` so T-1324 can call it
- [x] Unit test `latest_description_picks_most_recent` — given (desc_v1, desc_v2),
      returns desc_v2 with its ts_ms
- [x] Unit test `latest_description_returns_none_for_empty_or_no_topic_metadata`
- [x] `cargo test -p termlink --bins` + clippy clean
- [x] `agent-conversations.md` gains a short "Channel description (m.room.topic)" section

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
cargo test -p termlink --bins latest_description
cargo clippy -p termlink --all-targets -- -D warnings

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

### 2026-04-27T15:24:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1323-channel-describe--setget-topic-descripti.md
- **Context:** Initial task creation
