---
id: T-205
name: "Split CLI main.rs into command modules"
description: >
  Split CLI main.rs into command modules

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T06:22:44Z
last_update: 2026-03-21T06:22:44Z
date_finished: null
---

# T-205: Split CLI main.rs into command modules

## Context

`crates/termlink-cli/src/main.rs` is 5183 lines — a monolith containing all 26+ CLI commands,
hub profiles, remote operations, PTY handling, events, spawn utilities, etc. Split into focused
modules for maintainability, readability, and easier onboarding.

## Acceptance Criteria

### Agent
- [x] `main.rs` reduced to CLI struct definitions + dispatch (under 800 lines)
- [x] Module `commands/session.rs` — register, list, clean, ping, status, exec, send, signal
- [x] Module `commands/pty.rs` — interact, attach, output, inject, resize, stream + attach_loop/stream_loop
- [x] Module `commands/events.rs` — events, emit, broadcast, wait, watch, topics, collect
- [x] Module `commands/remote.rs` — all remote hub operations + connect_remote_hub
- [x] Module `commands/metadata.rs` — tag, discover, kv
- [x] Module `commands/execution.rs` — run, request, spawn + spawn utilities
- [x] Module `commands/infrastructure.rs` — hub start/stop/status
- [x] Module `commands/token.rs` — token create/inspect
- [x] Module `commands/agent.rs` — agent ask/listen
- [x] Module `commands/file.rs` — file send/receive
- [x] Module `config.rs` — hub profiles (HubProfile, HubsConfig, HubEntry, resolve_hub_profile)
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --workspace` passes (297 passed, 0 failed)
- [x] `cargo clippy --package termlink` has 0 warnings

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -v "^$" | tail -1 | grep -q "0 failed"
/Users/dimidev32/.cargo/bin/cargo clippy --package termlink -- -D warnings 2>&1 | grep -v warning || true

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

### 2026-03-21T06:22:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-205-split-cli-mainrs-into-command-modules.md
- **Context:** Initial task creation
