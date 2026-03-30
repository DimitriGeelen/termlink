---
id: T-800
name: "Fix E2E runner help text and small doc cleanups"
description: >
  Fix run-all.sh help text (says default 9 but code uses 99) and small cleanups

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-30T16:08:21Z
last_update: 2026-03-30T16:09:50Z
date_finished: 2026-03-30T16:09:50Z
---

# T-800: Fix E2E runner help text and small doc cleanups

## Context

run-all.sh help text says `--to N (default: 9)` but the actual code default is 99. Also update the help echo to match.

## Acceptance Criteria

### Agent
- [x] run-all.sh help text default for --to matches code behavior (99 / all)
- [x] help echo in the --help handler is consistent with header comments

## Verification

grep -q "default: all" tests/e2e/run-all.sh

## Updates
### Not applicable
<!-- Deleted Human AC section since all criteria are agent-verifiable.
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

### 2026-03-30T16:08:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-800-fix-e2e-runner-help-text-and-small-doc-c.md
- **Context:** Initial task creation

### 2026-03-30T16:08:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T16:09:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
