---
id: T-572
name: "Add --json output to termlink vendor status"
description: >
  Add --json output to termlink vendor status

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T15:08:50Z
last_update: 2026-03-28T15:11:27Z
date_finished: 2026-03-28T15:11:27Z
---

# T-572: Add --json output to termlink vendor status

## Context

Add `--json` to `termlink vendor status` for machine-readable vendor state output.

## Acceptance Criteria

### Agent
- [x] `VendorAction::Status` has `json: bool` field
- [x] `cmd_vendor_status` outputs JSON with version, path, mcp, gitignore status
- [x] Integration tests: not-vendored and vendored cases both pass
- [x] All existing tests pass (51 total)

## Verification

cargo test -p termlink --test cli_integration -- cli_vendor 2>&1 | grep -q "test result"
cargo clippy -p termlink -- -D warnings 2>&1 | tail -1 | grep -qv error

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

### 2026-03-28T15:08:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-572-add---json-output-to-termlink-vendor-sta.md
- **Context:** Initial task creation

### 2026-03-28T15:11:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
