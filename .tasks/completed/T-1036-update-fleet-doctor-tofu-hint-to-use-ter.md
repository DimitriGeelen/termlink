---
id: T-1036
name: "Update fleet-doctor TOFU hint to use termlink tofu clear command"
description: >
  Update fleet-doctor TOFU hint to use termlink tofu clear command

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-session/src/tofu.rs]
related_tasks: []
created: 2026-04-13T18:50:02Z
last_update: 2026-04-13T18:51:35Z
date_finished: 2026-04-13T18:51:35Z
---

# T-1036: Update fleet-doctor TOFU hint to use termlink tofu clear command

## Context

T-1035 added `termlink tofu clear` command. Update fleet-doctor and TOFU violation error messages to reference it instead of "edit ~/.termlink/known_hubs".

## Acceptance Criteria

### Agent
- [x] Fleet-doctor TOFU violation hint uses `termlink tofu clear` command
- [x] TOFU violation error message in tofu.rs references `termlink tofu clear`
- [x] Builds with zero clippy warnings

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

### 2026-04-13T18:50:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1036-update-fleet-doctor-tofu-hint-to-use-ter.md
- **Context:** Initial task creation

### 2026-04-13T18:51:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
