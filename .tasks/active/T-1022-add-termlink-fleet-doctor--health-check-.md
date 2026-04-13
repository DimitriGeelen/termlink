---
id: T-1022
name: "Add termlink fleet-doctor — health check all configured hubs"
description: >
  Add 'termlink fleet doctor' command that iterates over all hubs in ~/.termlink/hubs.toml and runs remote doctor on each. Provides a single-command fleet health overview without needing to check each hub individually.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:24:21Z
last_update: 2026-04-13T12:29:29Z
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
- [ ] [REVIEW] Run `termlink fleet doctor` and verify all hubs checked
  **Steps:**
  1. `cd /opt/termlink && cargo run -- fleet doctor`
  **Expected:** Shows health for each hub in hubs.toml with summary
  **If not:** Check fleet command implementation

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
