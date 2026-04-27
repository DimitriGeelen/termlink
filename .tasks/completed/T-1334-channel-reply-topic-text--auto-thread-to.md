---
id: T-1334
name: "channel reply <topic> <text> — auto-thread to the topic's latest content message"
description: >
  channel reply <topic> <text> — auto-thread to the topic's latest content message

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T17:09:54Z
last_update: 2026-04-27T17:14:39Z
date_finished: 2026-04-27T17:14:39Z
---

# T-1334: channel reply <topic> <text> — auto-thread to the topic's latest content message

## Context

In interactive use, the natural reply pattern requires (1) subscribe to find offset,
(2) post with `--reply-to N`. Tedious. `channel reply <topic> <text>` walks the topic,
finds the latest content envelope (skipping meta msg_types), and posts with that
offset as `metadata.in_reply_to`. Fail-fast on empty topic. Pure helper for testability.

## Acceptance Criteria

### Agent
- [x] `cli.rs` has new `Reply { topic, payload, mentions, hub, json }` ChannelAction.
- [x] `cmd_channel_reply` walks topic, picks the highest-offset content envelope, and posts with `--reply-to <that-offset>`.
- [x] Pure helper `latest_content_offset(msgs) -> Option<u64>` (uses same UNREAD_META_TYPES filter as T-1332). Unit tests: empty input, all-meta returns None, mixed picks highest content offset.
- [x] On empty topic / no content: returns clear error "No content message found on topic '<t>' to reply to".
- [x] `cargo test -p termlink --bins latest_content_offset` passes.
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] Smoke: post 2 messages on a fresh topic, run `channel reply <t> "got it"` and verify the new envelope's metadata.in_reply_to equals the second post's offset.

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

cargo test -p termlink --bins latest_content_offset
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

### 2026-04-27T17:09:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1334-channel-reply-topic-text--auto-thread-to.md
- **Context:** Initial task creation

### 2026-04-27T17:14:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
