---
id: T-686
name: "Add --roles flag to dispatch command for parity with spawn"
description: >
  Add --roles flag to dispatch command for parity with spawn

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T23:28:53Z
last_update: 2026-03-28T23:28:53Z
date_finished: 2026-03-29T00:08:00Z
---

# T-686: Add --roles flag to dispatch command for parity with spawn

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `--roles` flag added to `Dispatch` command in cli.rs
- [x] `roles` param threaded through main.rs dispatch to cmd_dispatch
- [x] cmd_dispatch passes roles to spawned worker register args
- [x] Project compiles cleanly

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

### 2026-03-28T23:28:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-686-add---roles-flag-to-dispatch-command-for.md
- **Context:** Initial task creation
