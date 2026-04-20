---
id: T-1172
name: "T-1161 follow-up: termlink channel queue-status CLI verb"
description: >
  Optional CLI verb from T-1161 AC (punted to follow-up). Adds 'termlink channel queue-status' showing pending count + oldest timestamp from ~/.termlink/outbound.sqlite for operator debugging. ~40 LOC CLI + 1 MCP mirror (R-033).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, cli]
components: []
related_tasks: [T-1161]
created: 2026-04-20T22:19:52Z
last_update: 2026-04-20T22:21:33Z
date_finished: null
---

# T-1172: T-1161 follow-up: termlink channel queue-status CLI verb

## Context

T-1161 landed the durable offline queue + `BusClient` flush task at `~/.termlink/outbound.sqlite`. Operators need a zero-risk read-only view of the queue state (pending count, oldest timestamp, attempts on the head-of-line post) for debugging when posts appear stuck. This task wires that view to the CLI + an MCP mirror per R-033/T-922.

## Acceptance Criteria

### Agent
- [x] New CLI verb `termlink channel queue-status` wired in `crates/termlink-cli/src/cli.rs` (`ChannelAction::QueueStatus { queue_path, json }`) and dispatched in `main.rs`
- [x] Implementation in `crates/termlink-cli/src/commands/channel.rs::cmd_channel_queue_status`:
  - Resolves queue path from `--queue-path` flag, else `termlink_session::offline_queue::default_queue_path()`
  - Opens the queue read-only, prints pending count + oldest enqueued-timestamp + first-few-pending (topic + msg_type + attempts)
  - Respects `--json` for machine-readable output
  - If the queue file doesn't exist, prints `pending: 0 (queue file not created yet)` instead of erroring
- [x] MCP mirror `termlink_channel_queue_status` registered in `crates/termlink-mcp/src/tools.rs` (R-033 / T-922 — every CLI verb must be MCP-reachable)
- [x] CLI help test: `cli_channel_help_lists_four_verbs` extended (or new sibling test) to confirm `queue-status` appears under `termlink channel --help`
- [x] `cargo test --workspace --lib` green; `cargo clippy --workspace --lib --tests -- -D warnings` clean

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

cargo build --workspace
cargo test --workspace --lib
cargo clippy --workspace --lib --tests -- -D warnings
grep -q "QueueStatus" crates/termlink-cli/src/cli.rs
grep -q "cmd_channel_queue_status" crates/termlink-cli/src/commands/channel.rs
grep -q "termlink_channel_queue_status" crates/termlink-mcp/src/tools.rs

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

### 2026-04-20T22:19:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1172-t-1161-follow-up-termlink-channel-queue-.md
- **Context:** Initial task creation

### 2026-04-20T22:21:33Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
