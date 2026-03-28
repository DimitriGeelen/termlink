---
id: T-602
name: "Add --wait flag to discover command that polls until a match is found"
description: >
  Add --wait flag to discover command that polls until a match is found

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T16:51:54Z
last_update: 2026-03-28T16:53:37Z
date_finished: 2026-03-28T16:53:37Z
---

# T-602: Add --wait flag to discover command that polls until a match is found

## Context

Add `--wait` and `--wait-timeout` flags to discover so scripts can block until a matching session appears.

## Acceptance Criteria

### Agent
- [x] `--wait` and `--wait-timeout` flags added to Discover command in cli.rs
- [x] cmd_discover converted to async, polls until match or timeout when --wait is set
- [x] main.rs dispatch updated with .await
- [x] Project compiles with `cargo check`

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

### 2026-03-28T16:51:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-602-add---wait-flag-to-discover-command-that.md
- **Context:** Initial task creation

### 2026-03-28T16:53:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
