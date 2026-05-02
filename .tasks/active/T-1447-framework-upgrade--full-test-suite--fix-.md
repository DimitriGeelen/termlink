---
id: T-1447
name: "framework upgrade + full test suite + fix issues"
description: >
  framework upgrade + full test suite + fix issues

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-02T09:59:47Z
last_update: 2026-05-02T09:59:47Z
date_finished: null
---

# T-1447: framework upgrade + full test suite + fix issues

## Context

User requested upgrade of vendored Agentic Engineering Framework on /opt/termlink to latest upstream (https://github.com/DimitriGeelen/agentic-engineering-framework), full test-suite run, classification of all issues, fixes for environmental problems, and findings for framework/upstream bugs (per STEP 5: do NOT edit framework source locally — report instead).

Pre-state: fw 1.5.307 (vendored), pinned 1.5.307.
Post-state: fw 1.6.124 (vendored from upstream).

## Acceptance Criteria

### Agent
- [x] `fw upgrade` ran cleanly from upstream clone (5 changes applied, 1.5.307 → 1.6.124)
- [x] `fw doctor` post-upgrade returns 0 FAIL after enforcement-baseline reset (post-upgrade routine)
- [x] `fw test all` ran end-to-end; per-suite pass/fail counts captured
- [x] Each test failure classified (framework / termlink / environmental)
- [x] CLAUDE.md project-specific governance content preserved or restored after `fw upgrade` clobber
- [x] Findings captured as learnings + bug report posted via termlink (channel post agent-chat-arc)
- [x] Commit + push to OneDev only (memory: never push to GitHub)

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-05-02T09:59:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1447-framework-upgrade--full-test-suite--fix-.md
- **Context:** Initial task creation
