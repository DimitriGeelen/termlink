---
id: T-1106
name: "Add termlink net test — layered hub connectivity diagnostic (TCP/TLS/auth)"
description: >
  Add termlink net test — layered hub connectivity diagnostic (TCP/TLS/auth)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs, crates/termlink-cli/tests/cli_integration.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-17T15:49:03Z
last_update: 2026-04-17T16:06:26Z
date_finished: 2026-04-17T16:06:26Z
---

# T-1106: Add termlink net test — layered hub connectivity diagnostic (TCP/TLS/auth)

## Context

Complements `fleet status` (pass/fail) with per-layer diagnostic depth: shows exactly
WHERE a hub connection fails (TCP reachable but TLS fails → cert issue; TLS OK but auth
fails → secret drift; etc). Fulfills T-1101 priority R3 (mesh connectivity diagnostics).

Scope: CLI-only — no new hub RPC method. Tests CLI→hub reachability in layers.
Hub-to-hub mesh testing deferred to a later task.

## Acceptance Criteria

### Agent
- [x] `termlink net test` subcommand runs, tests each configured hub in layers
- [x] Per hub, reports TCP/TLS/AUTH/PING status with pass/fail and latency
- [x] Text output is colored and readable; JSON output via `--json`
- [x] `--profile <name>` filters to one hub
- [x] Classifies failure layer correctly (TCP fail → network; TLS fail → cert; AUTH fail → secret)
- [x] Integration tests: cli_net_test_no_config, cli_net_test_tcp_fail_classifies_network, cli_net_test_profile_filter_unknown
- [x] `cargo build -p termlink` succeeds
- [x] `cargo test -p termlink --test cli_integration -- net_test` passes (3/3)
- [x] MCP tool `termlink_net_test` registered (count 68 → 69)

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

# Shell commands that MUST pass before work-completed.
cargo build -p termlink 2>&1 | tail -3 && cargo build -p termlink 2>&1 | grep -qv "error\["
cargo test -p termlink --test cli_integration -- net_test 2>&1 | tail -5

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

### 2026-04-17T15:49:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1106-add-termlink-net-test--layered-hub-conne.md
- **Context:** Initial task creation

### 2026-04-17T16:06:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
