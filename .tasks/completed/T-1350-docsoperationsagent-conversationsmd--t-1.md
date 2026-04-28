---
id: T-1350
name: "docs/operations/agent-conversations.md — T-1344..T-1349 wave"
description: >
  docs/operations/agent-conversations.md — T-1344..T-1349 wave

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T21:32:26Z
last_update: 2026-04-27T21:33:55Z
date_finished: 2026-04-27T21:33:55Z
---

# T-1350: docs/operations/agent-conversations.md — T-1344..T-1349 wave

## Context

T-1344..T-1349 added 6 user-visible commands/flags: `channel quote`,
`channel pin`, `channel pinned`, `channel forward`, plus `subscribe`
flags `--show-parent`, `--tail`, `--senders`, `--show-forwards`. The
operator runbook at `docs/operations/agent-conversations.md` needs a
section per group so operators can find and use them.

## Acceptance Criteria

### Agent
- [x] New section "Quote rendering" covers `channel quote` + `subscribe --show-parent`
- [x] New section "Pinned events" covers `channel pin` / `channel pinned` (Matrix m.room.pinned_events analogue)
- [x] New section "Render filters" covers `--tail` and `--senders`
- [x] New section "Forwarding" covers `channel forward` + `subscribe --show-forwards`
- [x] Each section includes a copy-pasteable example invocation
- [x] e2e step count updated (19 → 27)
- [x] "Related" task list extended with T-1344..T-1349
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
test -f docs/operations/agent-conversations.md
grep -q "channel quote" docs/operations/agent-conversations.md
grep -q "channel pin" docs/operations/agent-conversations.md
grep -q "channel forward" docs/operations/agent-conversations.md
grep -q "show-parent" docs/operations/agent-conversations.md
grep -q "show-forwards" docs/operations/agent-conversations.md
grep -q -- "--tail" docs/operations/agent-conversations.md
grep -q -- "--senders" docs/operations/agent-conversations.md

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

### 2026-04-27T21:32:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1350-docsoperationsagent-conversationsmd--t-1.md
- **Context:** Initial task creation

### 2026-04-27T21:33:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
