---
id: T-802
name: "Deduplicate shell_escape in execution.rs"
description: >
  Deduplicate command escaping in execution.rs to use util::shell_escape

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/execution.rs, crates/termlink-cli/src/util.rs]
related_tasks: []
created: 2026-03-30T16:15:04Z
last_update: 2026-03-30T16:17:48Z
date_finished: 2026-03-30T16:17:48Z
---

# T-802: Deduplicate shell_escape in execution.rs

## Context

execution.rs has an inline shell_escape closure that duplicates `util::shell_escape` but also handles backtick. Add backtick to util::shell_escape and replace the inline version.

## Acceptance Criteria

### Agent
- [x] util::shell_escape handles backtick character
- [x] execution.rs uses `crate::util::shell_escape` instead of inline closure
- [x] Tests pass, no clippy warnings

## Verification

grep -q "shell_escape" crates/termlink-cli/src/commands/execution.rs
! grep -q "part.replace" crates/termlink-cli/src/commands/execution.rs

## Updates
### Not applicable
<!-- Deleted Human AC section — all agent-verifiable.
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

### 2026-03-30T16:15:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-802-deduplicate-shellescape-in-executionrs.md
- **Context:** Initial task creation

### 2026-03-30T16:15:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T16:17:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
