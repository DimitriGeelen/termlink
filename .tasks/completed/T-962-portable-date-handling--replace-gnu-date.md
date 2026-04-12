---
id: T-962
name: "Portable date handling — replace GNU date -d with macOS-compatible alternatives"
description: >
  Fix D from T-292: Replace GNU date -d usage in framework scripts with portable alternatives (macOS bash 3.2 has no GNU date). Prevents silent episodic generation failures on macOS.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T08:47:05Z
last_update: 2026-04-12T09:07:33Z
date_finished: 2026-04-12T09:07:33Z
---

# T-962: Portable date handling — replace GNU date -d with macOS-compatible alternatives

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] All `date -d` usage in framework shell scripts replaced with portable alternative (python3 or `date -j` with fallback)
- [x] Episodic generation succeeds on both Linux and macOS
- [x] No GNU-only date flags remain in .agentic-framework/ shell scripts
- [x] Pickup sent to framework agent (this is a framework-side fix)

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

### 2026-04-12T08:47:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-962-portable-date-handling--replace-gnu-date.md
- **Context:** Initial task creation

### 2026-04-12T09:07:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
