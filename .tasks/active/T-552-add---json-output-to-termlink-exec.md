---
id: T-552
name: "Add --json output to termlink exec"
description: >
  Add --json output to termlink exec

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:52:22Z
last_update: 2026-03-28T09:52:22Z
date_finished: null
---

# T-552: Add --json output to termlink exec

## Context

`termlink exec` outputs raw stdout/stderr but lacks `--json` for structured output with exit code.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to Exec command in cli.rs
- [x] `cmd_exec` outputs JSON with stdout, stderr, exit_code when --json is set
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

### 2026-03-28T09:52:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-552-add---json-output-to-termlink-exec.md
- **Context:** Initial task creation
