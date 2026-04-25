---
id: T-1275
name: "Promote L-006 to practice — hub test runtime_dir isolation"
description: >
  Promote L-006 to practice — hub test runtime_dir isolation

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:38:36Z
last_update: 2026-04-25T20:39:43Z
date_finished: 2026-04-25T20:39:43Z
---

# T-1275: Promote L-006 to practice — hub test runtime_dir isolation

## Context

L-006 ("Hub tests that set TERMLINK_RUNTIME_DIR must use ENV_LOCK and
isolated temp dirs — without isolation, TLS cert from another test leaks
and causes handshake failures") has 3 applications and is operationally
load-bearing for TermLink test correctness. Graduate to D2 (Reliability)
so future hub-test authors find the rule by name rather than rediscovering
it via flaky failures.

## Acceptance Criteria

### Agent
- [x] PP-007 entry exists in `.context/project/practices.yaml` referencing L-006
- [x] L-006 application field updated from "TBD" → "Promoted to PP-007"

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

grep -q '^- id: PP-007' .context/project/practices.yaml
grep -A 11 '^- id: PP-007' .context/project/practices.yaml | grep -q 'promoted_from: L-006'
grep -A 7 '^- id: L-006' .context/project/learnings.yaml | grep -q 'PP-007'

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

### 2026-04-25T20:38:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1275-promote-l-006-to-practice--hub-test-runt.md
- **Context:** Initial task creation

### 2026-04-25T20:39:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
