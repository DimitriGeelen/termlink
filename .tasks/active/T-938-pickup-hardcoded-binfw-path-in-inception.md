---
id: T-938
name: "Pickup: Hardcoded bin/fw path in inception/review error hints breaks vendored consumers (127 exit) (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-921. Type: bug-report.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [pickup, bug-report, framework]
components: []
related_tasks: []
created: 2026-04-11T23:00:03Z
last_update: 2026-04-12T13:04:38Z
date_finished: null
---

# T-938: Pickup: Hardcoded bin/fw path in inception/review error hints breaks vendored consumers (127 exit) (from termlink)

## Context

Hardcoded `bin/fw` paths in `verify-acs.sh` (Python check_command calls) and `block-task-tools.sh` (error hint) fail with exit 127 in consumer projects where the binary lives at `.agentic-framework/bin/fw`. Fix: resolve fw binary path dynamically.

## Acceptance Criteria

### Agent
- [x] `verify-acs.sh` uses dynamic `fw_cmd` variable (resolves `bin/fw` → `.agentic-framework/bin/fw` → `fw`)
- [x] `block-task-tools.sh` uses bare `fw` (resolved by shim or PATH)
- [x] No remaining hardcoded `bin/fw` in execution paths (pattern-matching in scanner/metrics is fine)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q 'fw_cmd' /opt/termlink/.agentic-framework/lib/verify-acs.sh
! grep -q '"bin/fw' /opt/termlink/.agentic-framework/lib/verify-acs.sh

## Decisions

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T13:04:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now
