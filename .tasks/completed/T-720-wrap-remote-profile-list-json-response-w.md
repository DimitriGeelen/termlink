---
id: T-720
name: "Wrap remote profile list JSON response with ok:true for consistency"
description: >
  Wrap remote profile list JSON response with ok:true for consistency

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-03-29T11:33:00Z
last_update: 2026-03-29T11:33:56Z
date_finished: 2026-03-29T11:33:56Z
---

# T-720: Wrap remote profile list JSON response with ok:true for consistency

## Context

`remote profile list --json` returns a bare array instead of wrapping with `{"ok": true, "profiles": [...]}` like other list commands.

## Acceptance Criteria

### Agent
- [x] JSON success response wraps profiles array with `{"ok": true, "profiles": [...]}`
- [x] Project compiles with `cargo check`

### Human
<!-- Remove this section if all criteria are agent-verifiable.
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

cargo check 2>&1 | grep -q 'Finished'
grep -q '"ok": true' crates/termlink-cli/src/commands/remote.rs

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

### 2026-03-29T11:33:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-720-wrap-remote-profile-list-json-response-w.md
- **Context:** Initial task creation

### 2026-03-29T11:33:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
