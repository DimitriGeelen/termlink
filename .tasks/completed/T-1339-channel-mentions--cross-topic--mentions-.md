---
id: T-1339
name: "channel mentions — cross-topic @-mentions inbox"
description: >
  channel mentions — cross-topic @-mentions inbox

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T18:43:35Z
last_update: 2026-04-27T18:51:44Z
date_finished: 2026-04-27T18:51:44Z
---

# T-1339: channel mentions — cross-topic @-mentions inbox

## Context

`channel mentions [--for <id>]` — cross-topic mentions inbox. Scans every topic (or those matching `--prefix`), walks each, and prints content envelopes whose `metadata.mentions` CSV contains the target id OR the wildcard `*`. Defaults `--for` to the caller's identity. Output groups hits by topic; `--json` returns flat array. Reuses T-1325/T-1333 `mentions_match` semantics. Read-only.

## Acceptance Criteria

### Agent
- [x] `channel mentions` (no flags) lists envelopes mentioning the caller's identity across every topic
- [x] `--for <id>` switches the target; `*`-CSV envelopes match any target (T-1333 wildcard semantics — covered in e2e step 17)
- [x] `--prefix <p>` restricts scan to topics whose name starts with `<p>` (perf)
- [x] `--limit N` caps printed hits (0 = unlimited)
- [x] `--json` returns flat array of `{topic, offset, sender_id, ts, payload, mentions}` records
- [x] Reuses pre-existing `extract_mentions` and `mentions_match` helpers (already-unit-tested in T-1325/T-1333)
- [x] Build/test/clippy all clean (291 tests passing); e2e `agent-conversation.sh` step 17 added and passes

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

### 2026-04-27T18:43:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1339-channel-mentions--cross-topic--mentions-.md
- **Context:** Initial task creation

### 2026-04-27T18:51:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
