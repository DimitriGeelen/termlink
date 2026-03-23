---
id: T-204
name: "termlink hub profiles — save remote hub configs for quick access"
description: >
  termlink hub profiles — save remote hub configs for quick access

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [cli, cross-machine, ux]
components: []
related_tasks: []
created: 2026-03-21T06:04:50Z
last_update: 2026-03-23T10:24:15Z
date_finished: 2026-03-21T06:14:26Z
---

# T-204: termlink hub profiles — save remote hub configs for quick access

## Context

Every remote command requires `--secret-file /tmp/termlink-107-secret.txt` and the full
`host:port` address. A hub profile system in `~/.termlink/hubs.toml` lets users save
configs and use short aliases: `termlink remote list lab` instead of the full form.

## Acceptance Criteria

### Agent
- [x] `resolve_hub_profile()` function resolves hub arg: if it contains `:`, treat as address; otherwise look up in `~/.termlink/hubs.toml`
- [x] TOML config format: `[hubs.name]` with `address`, `secret_file`, optional `secret`, `scope`
- [x] All 7 remote commands use `resolve_hub_profile()` — profile names work everywhere
- [x] CLI-provided `--secret-file`/`--secret`/`--scope` override profile defaults
- [x] `termlink remote profile add <name> <address> --secret-file <path>` creates/updates profiles
- [x] `termlink remote profile list` shows saved profiles in table format
- [x] `termlink remote profile remove <name>` deletes profiles
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --workspace` passes (297 passed, 0 failed)
- [x] Verified: add/list/remove profile workflow works end-to-end

### Human
- [x] [REVIEW] Test hub profile workflow
  **Steps:**
  1. `termlink remote profile add lab 192.168.10.107:9100 --secret-file /tmp/termlink-107-secret.txt`
  2. `termlink remote ping lab`
  3. `termlink remote list lab`
  **Expected:** Commands work with just the profile name "lab"
  **If not:** Check `~/.termlink/hubs.toml` was created correctly

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "resolve_hub_profile" crates/termlink-cli/src/main.rs
grep -q "hubs.toml" crates/termlink-cli/src/main.rs

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

### 2026-03-21T06:04:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-204-termlink-hub-profiles--save-remote-hub-c.md
- **Context:** Initial task creation

### 2026-03-21T06:14:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
