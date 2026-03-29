---
id: T-750
name: "Fix 3 failing integration tests — cli_discover_json_output, cli_ping_json_output, cli_ping_with_timeout"
description: >
  Fix 3 failing integration tests — cli_discover_json_output, cli_ping_json_output, cli_ping_with_timeout

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-29T14:33:44Z
last_update: 2026-03-29T14:38:20Z
date_finished: 2026-03-29T14:38:20Z
---

# T-750: Fix 3 failing integration tests — cli_discover_json_output, cli_ping_json_output, cli_ping_with_timeout

## Context

3 tests broken by JSON consistency sweep — tests assert old response format (`"status": "ok"`, bare arrays) but commands now use `"ok": true` + wrapped objects.

## Acceptance Criteria

### Agent
- [x] cli_ping_json_output test updated to check `json["ok"] == true`
- [x] cli_ping_with_timeout test updated to check `parsed["ok"] == true`
- [x] cli_discover_json_output test updated to parse wrapped `{"ok": true, "sessions": [...]}`
- [x] `cargo test --workspace` passes with 0 failures

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

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

### 2026-03-29T14:33:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-750-fix-3-failing-integration-tests--clidisc.md
- **Context:** Initial task creation

### 2026-03-29T14:38:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
