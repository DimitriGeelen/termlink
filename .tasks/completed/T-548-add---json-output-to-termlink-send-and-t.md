---
id: T-548
name: "Add --json output to termlink send and termlink clean"
description: >
  Add --json output to termlink send and termlink clean

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T09:43:25Z
last_update: 2026-03-28T09:46:08Z
date_finished: 2026-03-28T09:46:08Z
---

# T-548: Add --json output to termlink send and termlink clean

## Context

`termlink send` and `termlink clean` lack `--json` flags for machine-parseable output.

## Acceptance Criteria

### Agent
- [x] `cmd_send` already outputs JSON (raw RPC response) — no change needed
- [x] `--json` flag added to Clean command in cli.rs
- [x] `cmd_clean` outputs JSON when --json is set (dry_run, action, count, sessions array)
- [x] Builds without warnings

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-28T09:43:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-548-add---json-output-to-termlink-send-and-t.md
- **Context:** Initial task creation

### 2026-03-28T09:46:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
