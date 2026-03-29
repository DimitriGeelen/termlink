---
id: T-757
name: "Add integration tests for vendor JSON output, --check flag, and edge cases"
description: >
  Add integration tests for vendor JSON output, --check flag, and edge cases

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T19:41:33Z
last_update: 2026-03-29T19:41:33Z
date_finished: null
---

# T-757: Add integration tests for vendor JSON output, --check flag, and edge cases

## Context

Expand vendor command test coverage — JSON output, --check mode, and edge cases (existing .gitignore, MCP merge, corrupt settings).

## Acceptance Criteria

### Agent
- [x] `vendor --json` output test validates JSON structure (ok, action, source, binary, version, size_bytes)
- [x] `vendor --json` re-vendor test validates action=updated and previous_version present
- [x] `vendor status --check` exits non-zero when not vendored
- [x] `vendor status --check --json` outputs needs_update=true when not vendored
- [x] Vendor preserves existing .gitignore content while adding .termlink entry
- [x] Vendor merges into existing .claude/settings.local.json preserving other MCP servers
- [x] Vendor handles corrupt settings.local.json gracefully (warns, still copies binary)
- [x] All 528 tests pass (up from 521), 0 failures
- [x] Fix: vendor --json mode no longer leaks gitignore/MCP status messages into JSON output

## Verification

cargo test --workspace 2>&1 | grep -q "0 failed"
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

### 2026-03-29T19:41:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-757-add-integration-tests-for-vendor-json-ou.md
- **Context:** Initial task creation
