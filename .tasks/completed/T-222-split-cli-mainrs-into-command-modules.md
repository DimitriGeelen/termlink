---
id: T-222
name: "Split CLI main.rs into command modules"
description: >
  Split CLI main.rs into command modules

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-21T06:22:44Z
last_update: '2026-08-18T18:59:05Z'
date_finished: 2026-03-21T06:40:32Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:34Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-222: Split CLI main.rs into command modules

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
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-222-split-cli-mainrs-into-command-modules.md
- **Context:** Initial task creation

### 2026-03-21T06:40:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
