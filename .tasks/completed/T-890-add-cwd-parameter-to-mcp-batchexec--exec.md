---
id: T-890
name: "Add cwd parameter to MCP batch_exec — execute commands in specific working directory"
description: >
  Add cwd parameter to MCP batch_exec — execute commands in specific working directory

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-05T07:32:37Z
last_update: 2026-04-05T07:34:47Z
date_finished: 2026-04-05T07:34:47Z
---

# T-890: Add cwd parameter to MCP batch_exec — execute commands in specific working directory

## Context

Parity: all other exec tools have cwd. Batch_exec got env in T-889, now needs cwd.

## Acceptance Criteria

### Agent
- [x] `BatchExecParams` has `cwd: Option<String>` field
- [x] `termlink_batch_exec` passes cwd to each session's RPC call
- [x] `cargo build` succeeds

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

cargo build

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

### 2026-04-05T07:32:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-890-add-cwd-parameter-to-mcp-batchexec--exec.md
- **Context:** Initial task creation

### 2026-04-05T07:34:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
