---
id: T-1400
name: "Migrate fw doctor inbox.status probe to channel.list(prefix=inbox:)"
description: >
  Migrate fw doctor inbox.status probe to channel.list(prefix=inbox:)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T07:55:26Z
last_update: 2026-04-29T08:06:31Z
date_finished: 2026-04-29T08:06:31Z
---

# T-1400: Migrate fw doctor inbox.status probe to channel.list(prefix=inbox:)

## Context

`fw doctor` step 7 (`crates/termlink-cli/src/commands/infrastructure.rs:434`) and the
`termlink_doctor` MCP tool step 3 (`crates/termlink-mcp/src/tools.rs:5166`) call
`inbox.status` directly via `rpc_call` — bypassing `inbox_channel::status_with_fallback`.
Per `fw metrics api-usage --last-60d` on `/var/lib/termlink/rpc-audit.jsonl` today
(2026-04-29): **2453 calls / 5.1% of all hub RPC traffic** are these doctor probes,
all attributed to `(unknown)` caller. They are by far the largest single contributor
to the T-1166 entry-gate failure (5.46% legacy vs 1% threshold).

Both probes have a simple semantic: "is there pending transfer state?". Channel
equivalent: `channel.list(prefix="inbox:")` returns the same topic shape, and any
non-empty topic with messages means a pending transfer.

Migration strategy: try `channel.list(prefix="inbox:")` first; on MethodNotFound
fall back to `inbox.status`. On modern hubs (channel.* supported, which is all
current fleet) this drops `inbox.status` calls to ZERO from the doctor path.

Related: T-1166 retirement gate. Once this task lands and bakes for ~24h, the
`api-usage` legacy% should fall ~5pp.

## Acceptance Criteria

### Agent
- [x] `infrastructure.rs:434` step replaced with try-channel-first / fall-back-on-error logic; output messages preserved exactly ("no pending transfers" / "N pending transfer(s) for M target(s)")
- [x] `tools.rs:5166` ditto for the MCP doctor tool
- [x] Both call sites compute the same semantic ("total transfers", "targets") from `channel.list` reply (prefix=inbox:): topic count = target count, sum of per-topic `count` = total transfers
- [x] On a hub that supports `channel.list` (current fleet), running `fw doctor` once produces ZERO `inbox.status` lines in `<runtime_dir>/rpc-audit.jsonl` (verified live: BEFORE inbox.status=2465, AFTER inbox.status=2465 over a single doctor run; channel.list +1)
- [x] On a hub that does NOT support `channel.list` (any error from the channel.list call), the fallback path runs and falls back to inbox.status with the same data semantics — verified by code path inspection (the `Err(_)` arm of `outcome` and the `_ =>` arm in tools.rs)
- [x] No new dependencies added; uses existing `termlink_session::client::rpc_call` and `RpcClient::call` paths
- [x] `cargo build -p termlink -p termlink-mcp` clean
- [x] `cargo clippy -p termlink -p termlink-mcp --tests -- -D warnings` clean (also fixed 3 pre-existing clippy nits in `commands/channel.rs` that surfaced because clippy hadn't been run on these crates before)
- [x] `cargo test -p termlink -p termlink-mcp` 0 failures (99 passed)

## Verification

cargo build -p termlink -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished"
cargo clippy -p termlink -p termlink-mcp --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink -p termlink-mcp 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
grep -q 'channel.list' crates/termlink-cli/src/commands/infrastructure.rs
grep -q '"prefix": "inbox:"' crates/termlink-mcp/src/tools.rs

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

### 2026-04-29T07:55:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1400-migrate-fw-doctor-inboxstatus-probe-to-c.md
- **Context:** Initial task creation

### 2026-04-29T08:06:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
