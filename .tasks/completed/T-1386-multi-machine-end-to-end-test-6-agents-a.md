---
id: T-1386
name: "Multi-machine end-to-end test: 6 agents across .107 + .122"
description: >
  Multi-machine end-to-end test: 6 agents across .107 + .122

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T19:23:05Z
last_update: 2026-04-28T19:25:58Z
date_finished: 2026-04-28T19:25:58Z
---

# T-1386: Multi-machine end-to-end test: 6 agents across .107 + .122

## Context

Builds on T-1385 (cross-hub TCP). Validates the full agent-conversation arc in a real fleet with two distinct hub-bearing machines: .107 (workstation) and .122 (ring20-management). Six agents post a thread (4 from .107, 2 from .122), the .122 agents using cross-hub TCP `--hub 192.168.10.107:9100`. Includes cross-machine reply chain (m.in_reply_to) to prove parent-resolution works across machines.

## Acceptance Criteria

### Agent
- [x] `tests/e2e/multi-machine-conversation.sh` script exists and is executable
- [x] Script run end-to-end without manual intervention exits 0 (last run: `multi-machine-e2e-3095` topic, all 8 steps OK)
- [x] All 6 agent identities (alice, bob, carol, dave, erin, frank) attributed to posts in canonical state
- [x] 4 posts originated on .107 (payload prefix `.107:`), 2 originated on .122 (payload prefix `.122:`)
- [x] Cross-machine reply chain visible via `channel relations <topic> 0` (frank's reply to alice resolved at offset 6)

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
test -x tests/e2e/multi-machine-conversation.sh
BIN=./target/release/termlink ./tests/e2e/multi-machine-conversation.sh 2>&1 | grep -q "MULTI-MACHINE E2E PASSED"

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

### 2026-04-28T19:23:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1386-multi-machine-end-to-end-test-6-agents-a.md
- **Context:** Initial task creation

### 2026-04-28T19:25:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
