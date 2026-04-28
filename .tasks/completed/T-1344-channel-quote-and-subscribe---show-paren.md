---
id: T-1344
name: "channel quote and subscribe --show-parent — render parent inline for replies"
description: >
  channel quote and subscribe --show-parent — render parent inline for replies

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T20:10:53Z
last_update: 2026-04-27T20:21:41Z
date_finished: 2026-04-27T20:21:41Z
---

# T-1344: channel quote and subscribe --show-parent — render parent inline for replies

## Context

Reading a reply in isolation requires manual offset lookup to see context.
Add `channel quote <topic> <offset>` (renders the envelope at offset and inlines
its parent if `metadata.in_reply_to` is present) and `channel subscribe
--show-parent` (during streaming, prefix each emitted reply with a quoted block
showing the parent's payload). Matrix analogue: clients render `m.in_reply_to`
inline by fetching and quoting the parent.

## Acceptance Criteria

### Agent
- [x] `channel quote <topic> <offset>` reads the envelope at offset and, if it has `metadata.in_reply_to`, fetches+renders the parent inline as a quoted block; orphan envelope (no parent metadata) renders alone with a note
- [x] `channel subscribe --show-parent` accepts a flag that, when set, prefixes each emitted reply with the parent's payload (single-shot lookup per parent offset, cached for the stream)
- [x] Both surfaces have `--json` form returning `{topic, child: {...}, parent: {...}|null}` for quote; subscribe `--json --show-parent` adds a `parent` field to each emitted envelope (null when not a reply)
- [x] Unit tests cover: orphan envelope (no in_reply_to), normal reply (parent found), reply with broken/missing parent reference (renders gracefully), and helper that formats the quoted block
- [x] `cargo test -p termlink --bins --quiet` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] e2e `tests/e2e/agent-conversation.sh` extended with a quote step + subscribe --show-parent step; full run green

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

### 2026-04-27T20:10:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1344-channel-quote-and-subscribe---show-paren.md
- **Context:** Initial task creation

### 2026-04-27T20:21:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
