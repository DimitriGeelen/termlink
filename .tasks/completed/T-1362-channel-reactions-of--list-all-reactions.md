---
id: T-1362
name: "channel reactions-of — list all reactions a specific sender posted on a topic"
description: >
  channel reactions-of — list all reactions a specific sender posted on a topic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T07:56:08Z
last_update: 2026-04-28T08:07:32Z
date_finished: 2026-04-28T08:07:32Z
---

# T-1362: channel reactions-of — list all reactions a specific sender posted on a topic

## Context

Reverse view of reactions on a topic: instead of "what reactions exist, by emoji" (T-1359 emoji-stats), this shows "what did sender X react to". Renders rows: `<emoji> on offset <N> ("<parent payload preview>")`. Honours redactions. Defaults to caller's identity if `--sender` omitted.

## Acceptance Criteria

### Agent
- [x] `channel reactions-of <topic>` lists all reactions the calling identity has posted
- [x] `--sender <id>` overrides the default identity scope
- [x] Output rows: `<emoji> → offset <N> (<parent_preview>)` sorted by reaction offset desc (most recent first)
- [x] Redacted reactions excluded
- [x] `--json` returns `[{reaction_offset, parent_offset, emoji, parent_payload, ts}]`
- [x] Empty result prints "No reactions by <sender>" + JSON `[]`
- [x] Pure helper `compute_reactions_of` unit-tested across: empty, single, multiple, redacted excluded, sender filter, sort order
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

### 2026-04-28T07:56:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1362-channel-reactions-of--list-all-reactions.md
- **Context:** Initial task creation

### 2026-04-28T08:07:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
