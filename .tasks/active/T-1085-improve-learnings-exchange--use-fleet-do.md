---
id: T-1085
name: "Improve learnings-exchange — use fleet doctor --json instead of ANSI stripping"
description: >
  Improve learnings-exchange — use fleet doctor --json instead of ANSI stripping

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-16T18:38:50Z
last_update: 2026-04-16T18:38:50Z
date_finished: null
---

# T-1085: Improve learnings-exchange — use fleet doctor --json instead of ANSI stripping

## Context

Replace ANSI-stripping + awk block extraction in learnings-exchange.sh with `fleet doctor --json` + python3 JSON parsing. Cleaner, more reliable, no ANSI fragility.

## Acceptance Criteria

### Agent
- [x] Uses `fleet doctor --json` instead of ANSI-stripping hack
- [x] Correctly identifies ok/fail peers from JSON output
- [x] Script runs without errors (tested: 3 peers, 1 ok, 2 skipped)

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

### 2026-04-16T18:38:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1085-improve-learnings-exchange--use-fleet-do.md
- **Context:** Initial task creation
