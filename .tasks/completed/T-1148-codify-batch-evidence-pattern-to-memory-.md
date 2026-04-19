---
id: T-1148
name: "Codify batch-evidence pattern to memory + remaining G-008 sweep (G-008 remediation)"
description: >
  Codify batch-evidence pattern to memory + remaining G-008 sweep (G-008 remediation)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T18:23:07Z
last_update: 2026-04-19T18:24:21Z
date_finished: 2026-04-19T18:24:21Z
---

# T-1148: Codify batch-evidence pattern to memory + remaining G-008 sweep (G-008 remediation)

## Context

Codify the 5-batch G-008 remediation pattern (T-1143-T-1147, 34 ACs evidenced) to agent memory so future sessions can re-apply it. Also sweep a final round of evidenceable candidates if bandwidth allows.

## Acceptance Criteria

### Agent
- [x] Memory file written at `/root/.claude/projects/-opt-termlink/memory/workflow_batch_evidence_g008.md`
- [x] MEMORY.md index updated with a one-line pointer

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

### 2026-04-19T18:23:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1148-codify-batch-evidence-pattern-to-memory-.md
- **Context:** Initial task creation

### 2026-04-19T18:24:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
