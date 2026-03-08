---
id: T-037
name: "Fix broken Claude Code hook paths"
description: >
  Fix broken Claude Code hook paths

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T21:11:12Z
last_update: 2026-03-08T21:11:43Z
date_finished: 2026-03-08T21:11:43Z
---

# T-037: Fix broken Claude Code hook paths

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] All `/usr/local/opt/fw/` references replaced with `/usr/local/opt/agentic-fw/` in `.claude/settings.json`
- [x] Pre-compact hook executes successfully

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

grep -q "agentic-fw" .claude/settings.json
! grep -q "/opt/fw/" .claude/settings.json

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

### 2026-03-08T21:11:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-037-fix-broken-claude-code-hook-paths.md
- **Context:** Initial task creation

### 2026-03-08T21:11:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
