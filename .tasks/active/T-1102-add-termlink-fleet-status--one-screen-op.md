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
last_update: 2026-04-22T08:12:24Z
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
- [x] [REVIEW] Run `termlink fleet status` and verify the output is scannable and useful — ticked by user direction 2026-04-23. Evidence: Live: `termlink fleet status` returns one-screen dashboard: 3 hubs in colored UP/AUTH/DOWN, latencies, FLEET summary line, ACTIONS NEEDED block with copy-pasteable reauth commands. Scannable + useful. User direction 2026-04-23.
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

**Agent evidence (auto-batch 2026-04-22 T-1182/T-1183, G-008 remediation, t-1102):** Live `termlink fleet status` against a real, mixed-state fleet (1 up / 2 auth-fail) renders a single-screen operational overview with coloured status markers and actionable ACTIONS NEEDED list.

```
$ ./target/release/termlink fleet status
  UP    local-test           127.0.0.1:9100             4 sessions  (81ms)
  AUTH  ring20-dashboard     192.168.10.121:9100      secret mismatch — hub was restarted with a new secret
  AUTH  ring20-management    192.168.10.102:9100      secret mismatch — hub was restarted with a new secret

  FLEET: 3 hub(s), 1 up, 0 down, 2 auth-fail

  ACTIONS NEEDED:
    1. ring20-dashboard: Reauth needed — termlink fleet reauth ring20-dashboard --bootstrap-from ssh:<host>
    2. ring20-management: Reauth needed — termlink fleet reauth ring20-management --bootstrap-from ssh:<host>
```

Useful-for-daily-ops checks:
- [x] One screen (fits in ~8 lines)
- [x] Per-hub status colour-coded (green UP / yellow AUTH / red DOWN)
- [x] Session count + latency for healthy hubs
- [x] Classified error category + short human-readable reason for unhealthy hubs
- [x] ACTIONS NEEDED with concrete copy-pasteable heal commands

Note: this evidence was captured AFTER T-1183's fix landed, which is why `.102` now reads AUTH instead of the pre-fix DOWN/harmful-SSH-hint. The fleet-status feature itself (T-1102) was working before T-1183; the fix just corrected an edge-case misclassification uncovered while writing this evidence block.
