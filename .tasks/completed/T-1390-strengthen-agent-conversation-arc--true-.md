---
id: T-1390
name: "Strengthen agent-conversation arc — true cross-hub 6-agent concurrent live e2e (post-T-1384 GO)"
description: >
  Strengthen agent-conversation arc — true cross-hub 6-agent concurrent live e2e (post-T-1384 GO)

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:24:49Z
last_update: 2026-04-28T21:30:19Z
date_finished: 2026-04-28T21:30:19Z
---

# T-1390: Strengthen agent-conversation arc — true cross-hub 6-agent concurrent live e2e (post-T-1384 GO)

## Context

T-1384 GO'd local-host arc + DEFERRED full fleet rollout citing 3 blockers:
(1) .122 hub at 0.9.844 — RESOLVED, .122 hub now at 0.9.1542 with `channel.*` (verified by probe `channel create` at 21:37Z).
(2) Per-session identity — still per-user; T-1387 worked around with `--sender-id` overrides.
(3) ring20-dashboard heal SSH-blocked — still BLOCKED, .121 not in this run.

T-1387's e2e proves 5 .107-tagged + 1 .122-cross-hub posts converge on **.107 hub**. Still missing: bidirectional cross-hub (concurrent posts to BOTH hubs in one run) + cross-hub READ convergence (read the same topic from the remote hub side).

This task: patch T-1387's stale REMOTE_SESSION (drifted), add `tests/e2e/cross-hub-bidirectional-6agents.sh` covering bidirectional concurrent posts and cross-hub read-side convergence, run both, commit.

## Acceptance Criteria

### Agent
- [x] T-1387 script runs to completion exit 0 against the upgraded fleet (REMOTE_SESSION auto-resolved, not hardcoded)
- [x] New script `tests/e2e/cross-hub-bidirectional-6agents.sh` exists and is executable
- [x] New script creates ONE topic on .107 hub and ONE topic on .122 hub
- [x] 6 concurrent senders post to .107-hub topic (5 local + 1 from .122 cross-hub TCP)
- [x] 6 concurrent senders post to .122-hub topic (5 from .107 cross-hub TCP + 1 local on .122)
- [x] Cross-hub READ test: read both topics from BOTH hubs (4 reads), verify each topic's canonical state has all 6 distinct senders regardless of which hub originates the read
- [x] Script exits 0 with `BIDIRECTIONAL CROSS-HUB E2E PASSED` marker
- [x] All work committed with task reference

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

test -x tests/e2e/live-agents-conversation.sh
test -x tests/e2e/cross-hub-bidirectional-6agents.sh
out1=$(BIN=./target/release/termlink ./tests/e2e/live-agents-conversation.sh 2>&1) && echo "$out1" | grep -q "LIVE-AGENT E2E PASSED"
out2=$(BIN=./target/release/termlink ./tests/e2e/cross-hub-bidirectional-6agents.sh 2>&1) && echo "$out2" | grep -q "BIDIRECTIONAL CROSS-HUB E2E PASSED"

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

### 2026-04-28T21:24:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1390-strengthen-agent-conversation-arc--true-.md
- **Context:** Initial task creation

### 2026-04-28T21:30:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
