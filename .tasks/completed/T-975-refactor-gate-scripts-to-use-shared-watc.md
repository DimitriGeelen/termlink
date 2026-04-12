---
id: T-975
name: "Refactor gate scripts to use shared watchtower helper — eliminate bare command output"
description: >
  Refactor check-tier0.sh, inception.sh, update-task.sh, verify-acs.sh, and init.sh to use _watchtower_url/_watchtower_open from lib/watchtower.sh instead of hardcoding ports and outputting bare commands. From T-972 RC-1 fix.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T10:27:22Z
last_update: 2026-04-12T10:41:26Z
date_finished: 2026-04-12T10:41:26Z
---

# T-975: Refactor gate scripts to use shared watchtower helper — eliminate bare command output

## Context

T-972 RC-1/RC-3: gate scripts construct Watchtower URLs inline with hardcoded ports. T-974 created the shared helper. Now refactor consumers.

## Acceptance Criteria

### Agent
- [x] check-tier0.sh uses `_watchtower_url` instead of inline URL construction (line ~354)
- [x] verify-acs.sh uses `_watchtower_url` for bash port detection (line ~54) and passes base URL to Python subprocess
- [x] No remaining hardcoded `3000` port in URL construction (excluding config defaults and documentation)
- [x] `fw tier0 status` still works (smoke test)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q '_watchtower_url' /opt/termlink/.agentic-framework/agents/context/check-tier0.sh
grep -q '_watchtower_url\|watchtower.sh' /opt/termlink/.agentic-framework/lib/verify-acs.sh

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

### 2026-04-12T10:27:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-975-refactor-gate-scripts-to-use-shared-watc.md
- **Context:** Initial task creation

### 2026-04-12T10:39:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T10:41:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
