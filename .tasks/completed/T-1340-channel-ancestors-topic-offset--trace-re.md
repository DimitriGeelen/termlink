---
id: T-1340
name: "channel ancestors <topic> <offset> — trace reply chain upward"
description: >
  channel ancestors <topic> <offset> — trace reply chain upward

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T18:55:11Z
last_update: 2026-04-27T19:03:42Z
date_finished: 2026-04-27T19:03:42Z
---

# T-1340: channel ancestors <topic> <offset> — trace reply chain upward

## Context

`channel ancestors <topic> <offset>` — given a leaf envelope, walk the `metadata.in_reply_to` chain upward and render the lineage from root → leaf. Inverse of `channel thread` (which walks down). Useful for "give me the conversation context above this offset" — handy when a thread fragment is shared without context.

Reuses the in_reply_to plumbing from T-1313. Read-only. Pure helper `build_ancestors(msgs, leaf) -> Vec<u64>` returns offsets root-first; cycle-safe (caps depth at 1024). `--json` returns array of envelope records preserving root→leaf order.

## Acceptance Criteria

### Agent
- [x] `channel ancestors <topic> <offset>` prints lineage from root to leaf, indented by depth
- [x] Pure helper `build_ancestors(by_off, leaf)` unit-tested: 6 cases (linear, root-only, missing-leaf, missing-parent, cycle, non-numeric parent)
- [x] Cycle detection: HashSet visited tracking + MAX_DEPTH=1024 cap; doesn't infinite-loop
- [x] `--json` returns `{topic, leaf, ancestors: [...]}` with envelope records in root→leaf order
- [x] Build/test/clippy all clean (297 tests passing); e2e `agent-conversation.sh` step 18 added and passes

## Verification

cargo build --release -p termlink
cargo test -p termlink --bins --quiet
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

### 2026-04-27T18:55:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1340-channel-ancestors-topic-offset--trace-re.md
- **Context:** Initial task creation

### 2026-04-27T19:03:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
