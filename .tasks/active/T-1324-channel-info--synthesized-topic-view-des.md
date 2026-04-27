---
id: T-1324
name: "channel info — synthesized topic view (description + senders + receipts + post count)"
description: >
  channel info — synthesized topic view (description + senders + receipts + post count)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:29:19Z
last_update: 2026-04-27T15:29:19Z
date_finished: null
---

# T-1324: channel info — synthesized topic view (description + senders + receipts + post count)

## Context

A synthesized read-only view of a topic: description (T-1323 latest), retention,
post count, distinct senders, latest receipt per sender. Composes existing
primitives: `channel.list` (count + retention), one full read for description
and sender summary, optionally `channel receipts` for the receipt summary
(reused logic from T-1315).

This is informational only — no new RPC, no state mutation. Useful to
operators wanting a "what's this topic about and who's caught up" snapshot.

## Acceptance Criteria

### Agent
- [x] `ChannelAction::Info { topic, hub, json }` added to cli.rs
- [x] `cmd_channel_info` in commands/channel.rs renders multi-line text:
      `Topic: <name>`, `Retention: <kind>[:<value>]`, `Posts: <count>`,
      `Description: <latest description text or "(none)">`,
      `Senders: <count distinct>`, optional latest sender list (top 5),
      `Receipts: <count>` with up_to per sender
- [x] JSON mode emits a single struct: `{topic, retention, count, description, senders, receipts}`
- [x] Pure helper `summarize_senders(msgs) -> Vec<(String, u64)>` returns
      `(sender_id, post_count)` sorted by count desc; ignores reaction/edit/redaction/topic_metadata/receipt envelopes
- [x] Unit test `summarize_senders_counts_only_content_msgs`
- [x] `cargo test -p termlink --bins` + clippy clean
- [x] `agent-conversations.md` mentions `channel info` under the channel description section

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
cargo test -p termlink --bins summarize_senders
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

### 2026-04-27T15:29:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1324-channel-info--synthesized-topic-view-des.md
- **Context:** Initial task creation
