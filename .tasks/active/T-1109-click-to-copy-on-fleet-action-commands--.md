---
id: T-1109
name: "Click-to-copy on /fleet action commands + bundle session learnings"
description: >
  Make each action line in the /fleet page's Actions Needed section click-to-copy (so the operator can paste the fix command into their terminal in one click). Bundles PL-026 and PL-027 learnings from T-1106/T-1107 via the commit.

status: started-work
workflow_type: build
owner: claude
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T16:42:59Z
last_update: 2026-04-17T16:42:59Z
date_finished: null
---

# T-1109: Click-to-copy on /fleet action commands + bundle session learnings

## Context

T-1101 R5: "CARD OR REFERENCE MAKE IT CLICKABLE". The Actions Needed section on
/fleet already lists fix commands (e.g. `termlink fleet reauth ... --bootstrap-from ssh:...`)
but the operator has to select+copy. One-click copy reduces friction — matches
the T-1107 "net-test" button pattern.

## Acceptance Criteria

### Agent
- [x] Each action line in /fleet's Actions Needed section is click-to-copy
- [x] Clicking shows a brief "copied" indicator (visual feedback)
- [x] Falls back gracefully if Clipboard API is unavailable (textarea + execCommand)
- [x] curl /fleet contains the copy handler function name (2 occurrences)

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

curl -sf http://localhost:3000/fleet | grep -q "copyAction"

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

### 2026-04-17T16:42:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1109-click-to-copy-on-fleet-action-commands--.md
- **Context:** Initial task creation
