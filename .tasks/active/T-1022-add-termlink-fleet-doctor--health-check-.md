---
id: T-1022
name: "Add termlink fleet-doctor — health check all configured hubs"
description: >
  Add 'termlink fleet doctor' command that iterates over all hubs in ~/.termlink/hubs.toml and runs remote doctor on each. Provides a single-command fleet health overview without needing to check each hub individually.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:24:21Z
last_update: 2026-04-19T12:12:00Z
date_finished: 2026-04-13T12:29:29Z
---

# T-1022: Add termlink fleet-doctor — health check all configured hubs

## Context

Reads all hubs from ~/.termlink/hubs.toml and runs remote doctor on each. Provides fleet-level health summary.

## Acceptance Criteria

### Agent
- [x] `termlink fleet doctor` command reads all hubs from hubs.toml
- [x] Checks each hub sequentially via connect_remote_hub
- [x] Reports per-hub and aggregate pass/warn/fail
- [x] JSON output supported
- [x] Builds and passes clippy

### Human
- [x] [REVIEW] Run `termlink fleet doctor` and verify all hubs checked — ticked by user direction 2026-04-23. Evidence: Live: `termlink fleet doctor` returns 'Fleet doctor: 3 hub(s) configured', enumerates each (UP/AUTH/DOWN), shows ACTIONS NEEDED block per hub. Output scannable + actionable. User direction 2026-04-23.
  **Steps:**
  1. `cd /opt/termlink && cargo run -- fleet doctor`
  **Expected:** Shows health for each hub in hubs.toml with summary
  **If not:** Check fleet command implementation

  **Agent evidence (2026-04-15T19:01Z):** Feature used live throughout T-1065/T-1067/T-1064 sessions. Most recent run (commit 26fe959a) with 3 hubs configured:
  ```
  Fleet doctor: 3 hub(s) configured
  --- local-test (127.0.0.1:9100) --- [PASS] connected in 81ms
  --- ring20-dashboard (192.168.10.121:9100) --- [FAIL] Cannot connect — hint + secret path
  --- ring20-management (192.168.10.122:9100) --- [FAIL] Cannot connect — hint + secret path
  Fleet summary: 3 hub(s), 1 ok, 0 warn, 2 fail
  ```
  Each hub checked, per-hub diagnostic shown, fleet summary aggregated. Output matches Expected. Human may tick and close.

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T12:24:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1022-add-termlink-fleet-doctor--health-check-.md
- **Context:** Initial task creation

### 2026-04-13T12:29:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T19:00:39Z — programmatic-evidence [T-1087]
- **Evidence:** termlink fleet doctor checks all 3 configured hubs (local-test, ring20-dashboard, ring20-management)
- **Verified by:** automated command execution


### 2026-04-19T12:12:00Z — status-update [task-update-agent]
- **Change:** owner: agent → human
