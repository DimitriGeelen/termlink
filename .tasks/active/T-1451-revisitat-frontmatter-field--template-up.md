---
id: T-1451
name: "revisit_at frontmatter field + template update (T-1449 Phase-1 #1)"
description: >
  T-1449 Phase-1 deliverable #1: add revisit_at: <ISO-date> and optional revisit_evidence_needed: <one-line> frontmatter fields to task templates. Backward-compatible (opt-in field). Update zzz-default.md + inception.md templates. Teach update-task.sh to preserve the field on status changes. Document in CLAUDE.md inception section. ~30 LOC + 1 template + doc.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [framework, governance, T-1449, phase-1, channel-1-mirror]
components: []
related_tasks: [T-1449, T-1428]
created: 2026-05-02T22:21:29Z
last_update: 2026-05-02T22:21:29Z
date_finished: null
---

# T-1451: revisit_at frontmatter field + template update (T-1449 Phase-1 #1)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-05-02T22:21:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1451-revisitat-frontmatter-field--template-up.md
- **Context:** Initial task creation
