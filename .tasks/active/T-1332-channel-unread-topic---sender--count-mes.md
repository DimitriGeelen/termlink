---
id: T-1332
name: "channel unread <topic> [--sender] — count messages newer than the sender's last receipt"
description: >
  channel unread <topic> [--sender] — count messages newer than the sender's last receipt

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T16:58:31Z
last_update: 2026-04-27T16:58:31Z
date_finished: null
---

# T-1332: channel unread <topic> [--sender] — count messages newer than the sender's last receipt

## Context

`channel receipts` (T-1315/T-1329) tells you who has acked which offset.
The natural follow-up: "what's new for me?" Add `channel unread <topic>`
which: (1) finds the caller's (or `--sender` override's) latest receipt
up_to (default 0 if none), (2) counts non-meta envelopes with offset > up_to,
and (3) reports the first-unread offset. Pure helper makes the math testable;
backed by the existing `channel.receipts` + `channel.subscribe` RPCs (no
new hub method). Provides Slack-style "3 new" UX without needing a UI.

## Acceptance Criteria

### Agent
- [x] `cli.rs` has new `Unread { topic, sender, hub, json }` ChannelAction.
- [x] `cmd_channel_unread` resolves sender (override or local identity), calls `channel.receipts` to get up_to, walks topic from up_to+1, counts content envelopes (excludes msg_type in {receipt, reaction, redaction, edit, topic_metadata}).
- [x] Pure helper `count_unread(msgs, up_to) -> (count, Option<first_unread_offset>)` with unit tests: empty input, all before bound returns 0, mixed content+meta only counts content, returns first content offset > up_to.
- [x] Text output: `Topic foo: 3 unread for <sender> (first new offset 7, latest 9)` or `up to date` when 0.
- [x] JSON: `{topic, sender_id, up_to, unread_count, first_unread, last_offset}`.
- [x] `cargo test -p termlink --bins count_unread` passes.
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] Smoke: post 5 messages, ack to offset 2, run `channel unread` for that sender, expect count=2 first_unread=3.

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

cargo test -p termlink --bins count_unread
cargo clippy --all-targets --workspace -- -D warnings

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

### 2026-04-27T16:58:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1332-channel-unread-topic---sender--count-mes.md
- **Context:** Initial task creation
