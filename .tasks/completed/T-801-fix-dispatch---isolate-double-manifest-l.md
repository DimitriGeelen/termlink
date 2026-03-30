---
id: T-801
name: "Fix dispatch --isolate double manifest load and created_at bug"
description: >
  Fix double manifest load and wrong created_at timestamp in dispatch --isolate

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/manifest.rs]
related_tasks: []
created: 2026-03-30T16:11:36Z
last_update: 2026-03-30T16:12:55Z
date_finished: 2026-03-30T16:12:55Z
---

# T-801: Fix dispatch --isolate double manifest load and created_at bug

## Context

Line 139 of dispatch.rs does `DispatchManifest::load(&project_root)?.last_updated.clone()` to populate `created_at` — this is a redundant second load (manifest was loaded on line 136) and semantically wrong (uses stale `last_updated` instead of current time).

## Acceptance Criteria

### Agent
- [x] `created_at` uses `now_rfc3339()` instead of double-loading the manifest
- [x] `now_rfc3339()` is `pub(crate)` in manifest.rs
- [x] No double manifest load on the dispatch --isolate path
- [x] All tests pass, no clippy warnings

## Verification

grep -q "pub(crate) fn now_rfc3339" crates/termlink-cli/src/manifest.rs
! grep -q "DispatchManifest::load.*last_updated" crates/termlink-cli/src/commands/dispatch.rs

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

### 2026-03-30T16:11:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-801-fix-dispatch---isolate-double-manifest-l.md
- **Context:** Initial task creation

### 2026-03-30T16:11:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T16:12:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
