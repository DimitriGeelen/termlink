---
id: T-1076
name: "Fix remote inbox CLI ergonomics — argument ordering, missing default subcommand, option propagation"
description: >
  Fix remote inbox CLI ergonomics — argument ordering, missing default subcommand, option propagation

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T04:37:59Z
last_update: 2026-04-16T04:37:59Z
date_finished: null
---

# T-1076: Fix remote inbox CLI ergonomics — argument ordering, missing default subcommand, option propagation

## Context

Consumer on .107 hit 3 failures in 4 attempts trying `termlink remote inbox status`. Root cause: clap struct in cli.rs:1428 nests `<HUB>` positional before subcommand, and `--secret-file`/`--timeout` options are parent-scoped (not propagated to subcommands). Additionally, bare `remote inbox <hub>` gives unhelpful "requires subcommand" instead of defaulting to `status`.

## Acceptance Criteria

### Agent
- [x] `termlink remote inbox <hub> status --secret-file <path>` works (options after subcommand)
- [x] `termlink remote inbox <hub> --secret-file <path> status` also works (options before subcommand)
- [x] `termlink remote inbox <hub>` defaults to `status` (no bare "requires subcommand" error)
- [x] `cargo test` passes
- [x] `cargo clippy` clean

### Human
- [ ] [RUBBER-STAMP] Consumer-friendly syntax verified
  **Steps:**
  1. Run: `cd /opt/termlink && termlink remote inbox local-test status`
  2. Run: `cd /opt/termlink && termlink remote inbox local-test`
  **Expected:** Both show inbox status (or connection error), not argument-parsing errors
  **If not:** Paste the error output

## Verification

cargo test --workspace 2>&1 | tail -5
cargo clippy --workspace 2>&1 | grep -c "^error" | grep -q "^0$"

## Decisions

### 2026-04-16 — Fix approach
- **Chose:** `#[arg(global = true)]` on parent options + `Option<RemoteInboxAction>` defaulting to Status
- **Why:** Minimal change (2 files, ~5 lines), no API break, clap-native solution
- **Rejected:** Flattening auth args into each variant (too much duplication), restructuring to flat commands like `inbox-status` (breaking change)

## Updates

### 2026-04-16T04:37:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1076-fix-remote-inbox-cli-ergonomics--argumen.md
- **Context:** Initial task creation
