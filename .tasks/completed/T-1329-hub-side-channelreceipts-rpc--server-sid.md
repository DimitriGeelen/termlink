---
id: T-1329
name: "Hub-side channel.receipts RPC — server-side aggregation of latest receipt per sender"
description: >
  Hub-side channel.receipts RPC — server-side aggregation of latest receipt per sender

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-protocol/src/control.rs]
related_tasks: []
created: 2026-04-27T16:23:31Z
last_update: 2026-04-27T16:42:08Z
date_finished: 2026-04-27T16:42:08Z
---

# T-1329: Hub-side channel.receipts RPC — server-side aggregation of latest receipt per sender

## Context

T-1315 introduced read-side `channel receipts` (CLI walks the topic, aggregates latest
`msg_type=receipt` per sender). For long-lived topics the O(N) walk is wasteful when
many readers ask the same question. This task moves the aggregation to the hub: a new
`channel.receipts(topic)` RPC returns `{topic, receipts:[{sender_id, up_to, ts_unix_ms}]}`.
Strict additivity — Tier-A, opaque, no signing changes; CLI prefers the new RPC and
gracefully falls back to the client walker when the hub doesn't know the method
(MethodNotFound), so old clients against new hubs and new clients against old hubs both work.

## Acceptance Criteria

### Agent
- [x] `protocol::method::CHANNEL_RECEIPTS` constant added with doc + assertion test in `control.rs`.
- [x] `handle_channel_receipts` + `handle_channel_receipts_with(bus, ...)` in `hub/channel.rs` that paginates the topic, keeps latest receipt per sender (latest-wins by ts, ties broken by higher up_to — same logic as the CLI), returns sorted-by-sender JSON.
- [x] Router dispatches `CHANNEL_RECEIPTS` to the new handler.
- [x] Hub-side unit test posts 4+ receipts (overlapping senders) and asserts aggregation correctness + ordering.
- [x] CLI `cmd_channel_receipts` calls the new RPC first; on `MethodNotFound` (-32601) falls back to existing client walker; on success renders identical text/JSON output.
- [x] `cargo test -p termlink-hub --lib channel::receipts` passes (3/3).
- [x] `cargo test -p termlink --bins` passes (242/242).
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] `bash tests/e2e/agent-conversation.sh` passes (step 9 receipts unchanged in output).

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

cargo test -p termlink-hub --lib channel
cargo test -p termlink --bins channel_receipts
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

### 2026-04-27T16:23:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1329-hub-side-channelreceipts-rpc--server-sid.md
- **Context:** Initial task creation

### 2026-04-27T16:42:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
