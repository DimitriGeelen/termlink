---
id: T-1391
name: "Cross-hub Matrix-primitive flow — 6 agents thread/react/edit/redact across 2 hubs"
description: >
  Cross-hub Matrix-primitive flow — 6 agents thread/react/edit/redact across 2 hubs

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:30:31Z
last_update: 2026-04-28T21:33:37Z
date_finished: 2026-04-28T21:33:37Z
---

# T-1391: Cross-hub Matrix-primitive flow — 6 agents thread/react/edit/redact across 2 hubs

## Context

T-1390 validated bare posts cross-hub. The arc's value comes from the Matrix-primitive subset: replies (`channel reply`), reactions (`channel react`), edits (`channel edit`), redactions (`channel redact`), receipts (`channel ack`), pins (`channel pin`). T-1386 demonstrated reply chains locally + 1 cross-machine link.

This task: write `tests/e2e/cross-hub-matrix-flow.sh` exercising a realistic 6-agent conversation across .107 + .122 hubs that uses replies, reactions, edits, redactions, and receipts — and verify each primitive is correctly attributed and visible from BOTH hub vantage points.

## Acceptance Criteria

### Agent
- [x] `tests/e2e/cross-hub-matrix-flow.sh` exists, executable
- [x] Test creates a conversation topic on .107 hub
- [x] 6 distinct senders (5 .107 stand-ins + 1 .122 cross-hub) post the conversation: root → reply → reply → reply → reaction → edit → redact
- [x] At least one reaction posted from .122 cross-hub TCP (proves reactions work over cross-hub)
- [x] At least one reply chain spanning a .107 root and a .122 cross-hub reply (proves `in_reply_to` resolves cross-hub)
- [x] At least one edit + one redaction visible in `channel state`
- [x] `channel thread <topic> 0` from .107 shows the full DFS-rendered conversation tree
- [x] `channel thread <topic> 0` from .122 (via cross-hub TCP) shows the same tree (byte-identical structure modulo timestamps)
- [x] All 6 sender_ids attributed in `channel members <topic>` output
- [x] Script exits 0 with `MATRIX-FLOW E2E PASSED` marker
- [x] Work committed with task reference

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

test -x tests/e2e/cross-hub-matrix-flow.sh
out=$(BIN=./target/release/termlink ./tests/e2e/cross-hub-matrix-flow.sh 2>&1) && echo "$out" | grep -q "MATRIX-FLOW E2E PASSED"

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

### 2026-04-28T21:30:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1391-cross-hub-matrix-primitive-flow--6-agent.md
- **Context:** Initial task creation

### 2026-04-28T21:33:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
