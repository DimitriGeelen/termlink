---
id: T-1363
name: "channel snippet — quotable text excerpt with surrounding context"
description: >
  channel snippet — quotable text excerpt with surrounding context

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T08:08:30Z
last_update: 2026-04-28T08:19:46Z
date_finished: 2026-04-28T08:19:46Z
---

# T-1363: channel snippet — quotable text excerpt with surrounding context

## Context

Generates a markdown-friendly text snippet for citing a channel message in tasks/docs/inceptions. Walks the topic, finds the target offset, and renders it with N envelopes of context above and below as a fenced code block. Skips meta envelopes (reactions, edits, redactions, receipts, topic_metadata) so the snippet is content-only.

## Acceptance Criteria

### Agent
- [x] `channel snippet <topic> <offset>` renders a quotable block with the target line marked (e.g. with `>>`)
- [x] `--lines N` controls context envelopes on each side (default: 2)
- [x] `--header` includes a topic+offset citation header above the block
- [x] `--json` returns `{topic, target_offset, lines:[{offset, sender, payload}]}`
- [x] Pure helper `compute_snippet` unit-tested across: target at start, target at end, target in middle, lines=0, lines>topic-size, target missing
- [x] e2e step covering full lifecycle
- [x] cargo build, test, clippy --all-targets -- -D warnings all pass

## Verification
cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5

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

### 2026-04-28T08:08:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1363-channel-snippet--quotable-text-excerpt-w.md
- **Context:** Initial task creation

### 2026-04-28T08:19:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
