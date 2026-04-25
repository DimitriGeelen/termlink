---
id: T-1274
name: "Promote L-011 to practice — CLI parameter defaults None vs zero"
description: >
  Promote L-011 to practice — CLI parameter defaults None vs zero

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:36:56Z
last_update: 2026-04-25T20:37:57Z
date_finished: 2026-04-25T20:37:57Z
---

# T-1274: Promote L-011 to practice — CLI parameter defaults None vs zero

## Context

L-011 ("CLI parameter defaults: None vs zero-value are semantically
different. since=0 means 'after event 0' not 'from the start'") has 5
applications and matches the same off-by-one pattern that surfaced again
in L-018, L-020. Graduate to a D2 (Reliability) practice so future CLI
parameter design has a name to reach for instead of re-deriving the
None-vs-zero distinction each time.

## Acceptance Criteria

### Agent
- [x] PP-006 entry exists in `.context/project/practices.yaml` referencing L-011
- [x] L-011 application field updated from "TBD" → "Promoted to PP-006"

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

grep -q '^- id: PP-006' .context/project/practices.yaml
grep -A 11 '^- id: PP-006' .context/project/practices.yaml | grep -q 'promoted_from: L-011'
grep -A 7 '^- id: L-011' .context/project/learnings.yaml | grep -q 'PP-006'

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

### 2026-04-25T20:36:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1274-promote-l-011-to-practice--cli-parameter.md
- **Context:** Initial task creation

### 2026-04-25T20:37:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
