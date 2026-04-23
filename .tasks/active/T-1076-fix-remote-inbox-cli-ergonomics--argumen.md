---
id: T-1076
name: "Fix remote inbox CLI ergonomics — argument ordering, missing default subcommand, option propagation"
description: >
  Fix remote inbox CLI ergonomics — argument ordering, missing default subcommand, option propagation

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-16T04:37:59Z
last_update: 2026-04-16T04:55:01Z
date_finished: 2026-04-16T04:45:32Z
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
- [x] [RUBBER-STAMP] Consumer-friendly syntax verified — ticked by user direction 2026-04-23. Evidence: Live: `termlink remote inbox local-test` and `termlink remote inbox local-test status` both return 'Inbox on 127.0.0.1:9100: empty' (no argument-parsing errors, default subcommand works). Verified 2026-04-23T17:35Z.
  **Steps:**
  1. Run: `cd /opt/termlink && termlink remote inbox local-test status`
  2. Run: `cd /opt/termlink && termlink remote inbox local-test`
  **Expected:** Both show inbox status (or connection error), not argument-parsing errors
  **If not:** Paste the error output


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, live-termlink, remote-inbox-ergonomics):** Live: `termlink remote inbox ring20-management --help` shows: positional `<HUB>` argument (ergonomic ordering), four subcommands (status, list, clear, help), with `clear` supporting `--all` per T-1008 alignment. No argument-ordering surprises remain. RUBBER-STAMPable.

## Verification

cargo test --workspace 2>&1 | tail -5
bash -c '[ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]'

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

### 2026-04-16T04:45:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink remote inbox local-test succeeds without subcommand (defaults to status)
- **Verified by:** automated command execution

