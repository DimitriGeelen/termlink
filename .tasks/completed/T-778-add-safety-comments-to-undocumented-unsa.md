---
id: T-778
name: "Add safety comments to undocumented unsafe blocks in dispatch.rs and endpoint.rs"
description: >
  Add safety comments to undocumented unsafe blocks in dispatch.rs and endpoint.rs

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-session/src/endpoint.rs]
related_tasks: []
created: 2026-03-30T00:08:12Z
last_update: 2026-03-30T00:09:24Z
date_finished: 2026-03-30T00:09:24Z
---

# T-778: Add safety comments to undocumented unsafe blocks in dispatch.rs and endpoint.rs

## Context

Two unsafe blocks lack SAFETY comments: dispatch.rs:303 (libc::kill) and endpoint.rs:100 (ptr::read for ManuallyDrop destructure).

## Acceptance Criteria

### Agent
- [x] dispatch.rs unsafe block has SAFETY comment explaining the libc::kill call
- [x] endpoint.rs unsafe block has SAFETY comment explaining the ManuallyDrop + ptr::read pattern
- [x] `cargo clippy --workspace` passes with no new warnings

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

### 2026-03-30T00:08:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-778-add-safety-comments-to-undocumented-unsa.md
- **Context:** Initial task creation

### 2026-03-30T00:09:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
