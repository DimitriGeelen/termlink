---
id: T-1392
name: "Cross-hub presence + ack + forward + DM flow — complete Matrix-primitive surface"
description: >
  Cross-hub presence + ack + forward + DM flow — complete Matrix-primitive surface

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:34:26Z
last_update: 2026-04-28T21:37:25Z
date_finished: 2026-04-28T21:37:25Z
---

# T-1392: Cross-hub presence + ack + forward + DM flow — complete Matrix-primitive surface

## Context

T-1390 + T-1391 covered post/reply/react/edit/redact/thread/members/state cross-hub. Remaining Matrix-primitive surface: ack/receipts, typing, pin/pinned, star/starred, forward, ancestors, quote, describe.

This task writes `tests/e2e/cross-hub-presence-flow.sh` exercising the lot, verifying each primitive emits and that state reads from .122 cross-hub TCP match .107-native.

## Acceptance Criteria

### Agent
- [x] `tests/e2e/cross-hub-presence-flow.sh` exists, executable
- [x] Topic created with 6 senders posting; topic gets a description via `channel describe`
- [x] `channel ack` posted from .107 (caller-1) and from .122 cross-hub TCP (caller-2); `channel receipts` shows both senders
- [x] `channel typing --emit` from .107 + cross-hub from .122; `channel typing` list shows at least one entry
- [x] `channel pin <offset>` + `channel pinned` shows the pinned target
- [x] `channel star <offset>` + `channel starred` shows the starred target
- [x] `channel forward <src> <offset> <dst>` copies an envelope; dst topic exists with the forwarded envelope carrying `forwarded_from` provenance
- [x] `channel ancestors <topic> <leaf>` returns a chain from root to leaf
- [x] `channel quote <topic> <child>` returns `{child, parent}` JSON
- [x] At least 3 of the above primitives are also read successfully via `--hub 192.168.10.122:9100` proving cross-hub read works for ack/typing/pin/star
- [x] Script exits 0 with `PRESENCE-FLOW E2E PASSED` marker
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

test -x tests/e2e/cross-hub-presence-flow.sh
out=$(BIN=./target/release/termlink ./tests/e2e/cross-hub-presence-flow.sh 2>&1) && echo "$out" | grep -q "PRESENCE-FLOW E2E PASSED"

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

### 2026-04-28T21:34:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1392-cross-hub-presence--ack--forward--dm-flo.md
- **Context:** Initial task creation

### 2026-04-28T21:37:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
