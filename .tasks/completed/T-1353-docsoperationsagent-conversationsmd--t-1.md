---
id: T-1353
name: "docs/operations/agent-conversations.md — T-1351 typing + T-1352 until tail"
description: >
  docs/operations/agent-conversations.md — T-1351 typing + T-1352 until tail

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T22:00:22Z
last_update: 2026-04-27T22:01:39Z
date_finished: 2026-04-27T22:01:39Z
---

# T-1353: docs/operations/agent-conversations.md — T-1351 typing + T-1352 until tail

## Context

T-1351 (typing) and T-1352 (subscribe --until) both shipped without doc
updates. Small docs PR adds: a "Typing indicators" section and a
"Windowed reads" section, plus extends Related and bumps the e2e step
count (27 → 29).

## Acceptance Criteria

### Agent
- [x] New section "Typing indicators" describes `channel typing --emit/--list/--ttl-ms`
- [x] New section "Windowed reads" describes `--since` paired with `--until`
- [x] e2e step count updated 27 → 29 in the End-to-end test paragraph
- [x] Related list extended with T-1351 + T-1352
- [x] `bash tests/e2e/agent-conversation.sh` still green

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

bash tests/e2e/agent-conversation.sh
grep -q "Typing indicators" docs/operations/agent-conversations.md
grep -q "Windowed reads" docs/operations/agent-conversations.md
grep -q "T-1351" docs/operations/agent-conversations.md
grep -q "T-1352" docs/operations/agent-conversations.md
grep -q "29 steps" docs/operations/agent-conversations.md

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

### 2026-04-27T22:00:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1353-docsoperationsagent-conversationsmd--t-1.md
- **Context:** Initial task creation

### 2026-04-27T22:01:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
