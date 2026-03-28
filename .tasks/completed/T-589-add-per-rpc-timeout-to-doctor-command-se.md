---
id: T-589
name: "Add per-RPC timeout to doctor command session and hub pings"
description: >
  Add per-RPC timeout to doctor command session and hub pings

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs]
related_tasks: []
created: 2026-03-28T16:16:24Z
last_update: 2026-03-28T16:17:28Z
date_finished: 2026-03-28T16:17:28Z
---

# T-589: Add per-RPC timeout to doctor command session and hub pings

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Session ping RPC calls in cmd_doctor wrapped in tokio::time::timeout (3s)
- [x] Hub ping RPC call in cmd_doctor wrapped in tokio::time::timeout (3s)
- [x] cargo build succeeds

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

### 2026-03-28T16:16:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-589-add-per-rpc-timeout-to-doctor-command-se.md
- **Context:** Initial task creation

### 2026-03-28T16:17:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
