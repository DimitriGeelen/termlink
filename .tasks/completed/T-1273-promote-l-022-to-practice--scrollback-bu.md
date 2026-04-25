---
id: T-1273
name: "Promote L-022 to practice — scrollback buffer wraparound handling"
description: >
  Promote L-022 to practice — scrollback buffer wraparound handling

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:35:13Z
last_update: 2026-04-25T20:36:38Z
date_finished: 2026-04-25T20:36:38Z
---

# T-1273: Promote L-022 to practice — scrollback buffer wraparound handling

## Context

L-022 ("Scrollback buffer delta calculations must handle wraparound") has
5 applications across the codebase and is a generic CS-level reliability
concern (off-by-one + circular-buffer semantics). Graduate to a practice
under D2 (Reliability) so future ring-buffer / delta-streaming code has a
named guideline to reference.

## Acceptance Criteria

### Agent
- [x] PP-005 entry exists in `.context/project/practices.yaml` referencing L-022
- [x] L-022 application field updated from "TBD" → "Promoted to PP-005"

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

grep -q '^- id: PP-005' .context/project/practices.yaml
grep -A 11 '^- id: PP-005' .context/project/practices.yaml | grep -q 'promoted_from: L-022'
grep -A 7 '^- id: L-022' .context/project/learnings.yaml | grep -q 'PP-005'

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

### 2026-04-25T20:35:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1273-promote-l-022-to-practice--scrollback-bu.md
- **Context:** Initial task creation

### 2026-04-25T20:36:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
