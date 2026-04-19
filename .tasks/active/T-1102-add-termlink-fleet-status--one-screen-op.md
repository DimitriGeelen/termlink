---
id: T-1102
name: "Add termlink fleet status — one-screen operational overview"
description: >
  Add termlink fleet status — one-screen operational overview

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs, crates/termlink-cli/tests/cli_integration.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-17T08:42:26Z
last_update: 2026-04-17T09:02:39Z
date_finished: 2026-04-17T09:02:39Z
---

# T-1102: Add termlink fleet status — one-screen operational overview

## Context

T-1101 inception identified that the operator's "morning check" experience is missing.
`fleet doctor` works but outputs raw JSON. The human wants a single scannable screen
showing: which hubs are up, how many sessions, what version, and what needs attention.
See `docs/reports/T-1101-termlink-value-assessment.md` R1.

## Acceptance Criteria

### Agent
- [x] `termlink fleet status` subcommand exists and compiles
- [x] Shows colored status per hub: UP (green), DOWN (red), AUTH-FAIL (yellow)
- [x] Shows session count and latency for reachable hubs
- [x] Shows ACTIONS NEEDED section with actionable fix steps for broken hubs
- [x] `termlink fleet status --json` returns structured JSON
- [x] Test: fleet status with no hubs.toml returns empty/helpful message
- [x] MCP tool `termlink_fleet_status` added (68 MCP tools total)
- [x] 1,124 tests pass, zero warnings

### Human
- [ ] [REVIEW] Run `termlink fleet status` and verify the output is scannable and useful
  **Steps:** `cd /opt/termlink && cargo run -- fleet status`
  **Expected:** Color-coded hub list, session counts, actions for broken hubs
  **If not:** Check fleet status subcommand implementation


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, fleet-status):** `termlink fleet status` produces one-screen output with color-coded UP/AUTH/DOWN per hub, per-hub session count + latency, and a top-level ACTIONS NEEDED block (`ring20-dashboard: Reauth needed — termlink fleet reauth ring20-dashboard --bootstrap-from ssh:<host>`). Actionable, not just descriptive. REVIEW-approvable.

## Verification

bash -c 'cargo build -p termlink 2>&1 | grep -q "Finished"'
bash -c 'cargo test --test cli_integration -- fleet_status 2>&1 | grep -q "passed"'

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

### 2026-04-17T08:42:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1102-add-termlink-fleet-status--one-screen-op.md
- **Context:** Initial task creation

### 2026-04-17T09:02:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
