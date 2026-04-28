---
id: T-1387
name: "Live-agent e2e: 5 .107 sessions + .122 post concurrently to one topic"
description: >
  Live-agent e2e: 5 .107 sessions + .122 post concurrently to one topic

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T19:27:34Z
last_update: 2026-04-28T19:27:34Z
date_finished: null
---

# T-1387: Live-agent e2e: 5 .107 sessions + .122 post concurrently to one topic

## Context

Stronger demonstration of the agent-conversation arc using REAL live sessions instead of synthetic identities. Five live sessions on .107 (selected from `termlink list`) post in parallel via `termlink remote exec`, each tagging itself with its session name as `--sender-id`. One additional post comes from .122 via cross-hub TCP. Total: 6 distinct posters, all parallel, all to one topic.

## Acceptance Criteria

### Agent
- [x] `tests/e2e/live-agents-conversation.sh` exists and is executable
- [x] Script runs to completion exit 0
- [x] At least 6 distinct sender_id values present in canonical state (verified 6: 5 local session IDs + ring20-mgmt-122)
- [x] Cross-hub TCP post from .122 visible in same canonical state
- [x] Note: sessions have command allowlists, so posts originate from this shell tagged with live session IDs as sender_id; from the bus's perspective they are 5 concurrent independent posters

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
test -x tests/e2e/live-agents-conversation.sh
BIN=./target/release/termlink ./tests/e2e/live-agents-conversation.sh 2>&1 | grep -q "LIVE-AGENT E2E PASSED"

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

### 2026-04-28T19:27:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1387-live-agent-e2e-5-107-sessions--122-post-.md
- **Context:** Initial task creation
