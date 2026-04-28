---
id: T-1370
name: "channel replies-of <sender> — list replies posted by a sender"
description: >
  channel replies-of <sender> — list replies posted by a sender

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T11:25:33Z
last_update: 2026-04-28T11:25:33Z
date_finished: null
---

# T-1370: channel replies-of <sender> — list replies posted by a sender

## Context

Mirror of T-1367 (`channel forwards-of`) for replies. Lists every reply (msg_type filter via `metadata.in_reply_to` presence) authored by `<sender>` in a topic, ordered by offset desc, with a one-line preview of both the reply and the parent it answered.

## Acceptance Criteria

### Agent
- [x] `pub fn compute_replies_of(envelopes: &[Value], sender: &str) -> Vec<RepliesOfRow>` in `crates/termlink-cli/src/commands/channel.rs` returns rows where `sender_id == sender` AND `metadata.in_reply_to` is present AND envelope is not redacted, sorted by reply offset desc
- [x] Each `RepliesOfRow` carries `reply_offset, parent_offset, parent_sender, reply_payload, parent_payload, ts_ms` (parent_payload may be empty if parent missing/redacted)
- [x] `cli.rs` adds `RepliesOf { topic, sender, hub, json }` ChannelAction variant
- [x] `main.rs` dispatches to `cmd_channel_replies_of`
- [x] At least 3 unit tests in channel.rs cover: happy path (3 replies → 3 rows desc), filter excludes non-replies and other-sender messages, redacted reply is excluded
- [x] `cargo test -p termlink --bins --quiet` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] tests/e2e/agent-conversation.sh gains a step exercising replies-of (positive + JSON shape)
- [x] Smoke test: post 2 replies from Alice + 1 from Bob, `termlink channel replies-of <topic> alice` returns exactly 2

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

cargo test -p termlink --bins --quiet 2>&1 | tail -3 | grep -q "test result: ok"
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -2 | grep -qE "(warning|error): 0|^$|^\s*Finished"
bash tests/e2e/agent-conversation.sh 2>&1 | grep -q "END-TO-END WALKTHROUGH PASSED"

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

### 2026-04-28T11:25:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1370-channel-replies-of-sender--list-replies-.md
- **Context:** Initial task creation
