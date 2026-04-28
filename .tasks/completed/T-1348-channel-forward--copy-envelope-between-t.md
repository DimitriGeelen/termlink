---
id: T-1348
name: "channel forward — copy envelope between topics with forwarded_from metadata"
description: >
  channel forward — copy envelope between topics with forwarded_from metadata

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T21:07:48Z
last_update: 2026-04-27T21:16:16Z
date_finished: 2026-04-27T21:16:16Z
---

# T-1348: channel forward — copy envelope between topics with forwarded_from metadata

## Context

Matrix has forwarding — copying a message into another room while preserving
provenance via metadata. Add `channel forward <src> <offset> <dst>` that
reads the envelope at offset on src, then posts a new envelope on dst with
the same payload + msg_type, sender_id = the forwarder (current identity),
and metadata `forwarded_from=<src>:<offset>` + `forwarded_sender=<original
sender_id>`. Readers can then trace back to the source. Tier-A additive —
no hub change.

## Acceptance Criteria

### Agent
- [x] `channel forward <src_topic> <offset> <dst_topic>` reads the envelope at offset on src and posts to dst
- [x] Posted envelope on dst preserves original payload + msg_type
- [x] Posted envelope's sender_id is the forwarder (current identity), not the original sender
- [x] Posted envelope has metadata `forwarded_from=<src_topic>:<offset>` and `forwarded_sender=<original_sender_id>`
- [x] Errors when src_topic is unknown OR offset is out of range
- [x] Pure helper `build_forward_metadata(src_topic, offset, original_sender) -> Vec<String>` returns the K=V strings used for the post
- [x] Unit tests: helper composes the right K=V pairs; smoke through real hub (e2e)
- [x] `cargo test -p termlink --bins --quiet` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] e2e step extended; full run green

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

cargo build --release -p termlink
cargo test -p termlink --bins --quiet
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

### 2026-04-27T21:07:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1348-channel-forward--copy-envelope-between-t.md
- **Context:** Initial task creation

### 2026-04-27T21:16:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
