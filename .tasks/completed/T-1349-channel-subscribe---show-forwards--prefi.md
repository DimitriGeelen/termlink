---
id: T-1349
name: "channel subscribe --show-forwards — prefix forwarded envelopes with [fwd] marker"
description: >
  channel subscribe --show-forwards — prefix forwarded envelopes with [fwd] marker

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T21:20:05Z
last_update: 2026-04-27T21:28:38Z
date_finished: 2026-04-27T21:28:38Z
---

# T-1349: channel subscribe --show-forwards — prefix forwarded envelopes with [fwd] marker

## Context

T-1348 added `channel forward`, which writes `metadata.forwarded_from`
and `metadata.forwarded_sender`. Without a render-side hint, forwards
look like normal posts. Add `--show-forwards` so a reader can spot
provenance at a glance: an envelope with forwarded_from gets a
`[fwd from <src>:<off> by <orig_sender>]` prefix line above the main
render. Pure render-side; no protocol change.

## Acceptance Criteria

### Agent
- [x] `--show-forwards` flag added to `channel subscribe`
- [x] When set, every envelope carrying `metadata.forwarded_from` emits a `[fwd from <src>:<off> by <orig_sender>]` prefix line BEFORE the main render line
- [x] When unset, forwarded envelopes render as normal (no prefix); current behavior unchanged
- [x] Pure helper `extract_forward(env) -> Option<(src_topic, offset, orig_sender)>` that pulls the metadata fields and parses src:off; tested for present, missing, and malformed cases
- [x] Unit tests: present (well-formed), missing (no metadata), missing forwarded_from, missing forwarded_sender (defensive), src with embedded colons (split-on-LAST-colon)
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

### 2026-04-27T21:20:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1349-channel-subscribe---show-forwards--prefi.md
- **Context:** Initial task creation

### 2026-04-27T21:28:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
