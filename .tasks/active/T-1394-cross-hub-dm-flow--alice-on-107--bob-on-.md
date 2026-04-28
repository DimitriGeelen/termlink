---
id: T-1394
name: "Cross-hub DM flow — alice on .107 + bob on .122 (Matrix-style direct message)"
description: >
  Cross-hub DM flow — alice on .107 + bob on .122 (Matrix-style direct message)

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:40:07Z
last_update: 2026-04-28T21:40:07Z
date_finished: null
---

# T-1394: Cross-hub DM flow — alice on .107 + bob on .122 (Matrix-style direct message)

## Context

`channel dm` derives a canonical `dm:<a>:<b>` topic from the caller's identity fingerprint and a peer identifier. The agent-conversation arc claims DMs work for two agents on different hubs. We have not validated this cross-hub.

This task adds `tests/e2e/cross-hub-dm-flow.sh` exercising:
1. alice on .107 sends DM to bob (peer-id "bob")
2. bob on .122 (via cross-hub TCP) sends DM to alice on the SAME topic (computed via `--topic-only`)
3. Both sides read the conversation, verify 2 messages with correct sender attribution
4. `channel dm --list` from each side surfaces the DM
5. `channel dm --unread` reports per-caller unread count

## Acceptance Criteria

### Agent
- [x] `tests/e2e/cross-hub-dm-flow.sh` exists, executable
- [x] DM topic resolved via `channel dm <peer> --topic-only` and used consistently on both sides
- [x] alice (.107) posts message to DM topic with `--sender-id alice-107`
- [x] bob (.122 cross-hub TCP) posts message to same DM topic with `--sender-id bob-122`
- [x] `channel state` on the DM topic from .107 shows both messages with correct sender_ids
- [x] `channel state` on the DM topic via .122 cross-hub TCP shows the same data (convergence)
- [x] `channel dm --list` on .107 includes the DM topic
- [x] At least one threaded reply (`channel post --reply-to`) inside the DM works cross-hub
- [x] Script exits 0 with `DM-FLOW E2E PASSED` marker
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

test -x tests/e2e/cross-hub-dm-flow.sh
out=$(BIN=./target/release/termlink ./tests/e2e/cross-hub-dm-flow.sh 2>&1) && echo "$out" | grep -q "DM-FLOW E2E PASSED"

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

### 2026-04-28T21:40:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1394-cross-hub-dm-flow--alice-on-107--bob-on-.md
- **Context:** Initial task creation
