---
id: T-1167
name: "Pickup to framework — watchtower placeholder detector false-positive on HTML comments"
description: >
  Pickup to framework — watchtower placeholder detector false-positive on HTML comments

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T14:13:01Z
last_update: 2026-04-20T14:13:01Z
date_finished: null
---

# T-1167: Pickup to framework — watchtower placeholder detector false-positive on HTML comments

## Context

Watchtower approvals page surfaces `ERROR: Placeholder content detected in task file` banners for tasks whose Decisions section only contains the default template HTML comment `<!-- ... ### [date] — [topic] ... -->`. The detector should strip `<!-- ... -->` blocks before scanning for placeholders. Observed 2026-04-20 with T-243 and T-235. User-visible noise; blocks the live approvals view.

## Acceptance Criteria

### Agent
- [x] Pickup envelope drafted with symptom, root cause, and proposed fix (strip HTML comments before scan)
- [x] Pickup delivered to framework via termlink + direct inbox drop

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
test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/P-T-1167-bug-report.yaml || test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/processed/P-T-1167-bug-report.yaml

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

### 2026-04-20T14:13:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1167-pickup-to-framework--watchtower-placehol.md
- **Context:** Initial task creation
