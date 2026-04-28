---
id: T-1345
name: "channel pin / unpin / pinned — Matrix-style pinned events on a topic"
description: >
  channel pin / unpin / pinned — Matrix-style pinned events on a topic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T20:25:56Z
last_update: 2026-04-27T20:35:33Z
date_finished: 2026-04-27T20:35:33Z
---

# T-1345: channel pin / unpin / pinned — Matrix-style pinned events on a topic

## Context

Matrix has `m.room.pinned_events` — a state event listing offsets pinned in
the room. We add an append-only equivalent: a `msg_type=pin` envelope that
carries `metadata.pin_target=<offset>` and `metadata.action=pin|unpin`.
`channel pinned <topic>` walks the topic and computes the current pin set
(latest action per target wins). Tier-A additive — no hub change needed.

## Acceptance Criteria

### Agent
- [x] `channel pin <topic> <offset>` emits a `msg_type=pin` envelope with `metadata.pin_target=<offset>` and `metadata.action=pin`
- [x] `channel pin <topic> <offset> --unpin` emits the same envelope shape but `metadata.action=unpin`
- [x] `channel pinned <topic>` walks the topic and prints the current pin set (one row per pinned target with the original sender + payload preview); sorted by most-recently-pinned descending
- [x] `channel pinned --json` returns `[{target, pinned_by, pinned_ts, payload}]`
- [x] Pure helper `compute_pinned_set(envelopes) -> Vec<PinRow>` resolves latest pin/unpin per target (unpin removes); tested against pin→unpin→repin sequences
- [x] Unit tests: empty topic, single pin, pin+unpin (target removed), repin after unpin, multiple distinct targets, ordering by pinned_ts desc
- [x] `cargo test -p termlink --bins --quiet` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] e2e `tests/e2e/agent-conversation.sh` extended with a pin/unpin/pinned step; full run green

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

### 2026-04-27T20:25:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1345-channel-pin--unpin--pinned--matrix-style.md
- **Context:** Initial task creation

### 2026-04-27T20:35:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
