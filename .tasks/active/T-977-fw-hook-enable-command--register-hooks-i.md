---
id: T-977
name: "fw hook-enable command — register hooks in settings.json from CLI"
description: >
  fw hook-enable command — register hooks in settings.json from CLI

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T10:49:55Z
last_update: 2026-04-12T10:49:55Z
date_finished: null
---

# T-977: fw hook-enable command — register hooks in settings.json from CLI

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `bin/hook-enable.sh` script exists with --matcher and --event flags
- [x] `fw hook-enable` route added in bin/fw
- [x] Idempotent — running twice doesn't duplicate the entry
- [x] T-976 pl007-scanner registered via the new command

### Human
- [ ] [RUBBER-STAMP] Verify pl007-scanner hook fires on bare command output
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

### 2026-04-12T10:49:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-977-fw-hook-enable-command--register-hooks-i.md
- **Context:** Initial task creation
