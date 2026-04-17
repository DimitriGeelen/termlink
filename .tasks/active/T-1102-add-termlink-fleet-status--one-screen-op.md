---
id: T-1102
name: "Add termlink fleet status — one-screen operational overview"
description: >
  Add termlink fleet status — one-screen operational overview

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T08:42:26Z
last_update: 2026-04-17T08:42:26Z
date_finished: null
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
