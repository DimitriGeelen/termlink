---
id: T-1327
name: "persist e2e agent-conversation walkthrough script as repo asset"
description: >
  persist e2e agent-conversation walkthrough script as repo asset

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:51:14Z
last_update: 2026-04-27T15:53:55Z
date_finished: 2026-04-27T15:53:55Z
---

# T-1327: persist e2e agent-conversation walkthrough script as repo asset

## Context

The /tmp/e2e-conv.sh walkthrough that exercised every Matrix-style feature
end-to-end with two real identities lives in /tmp and gets wiped. Persist it
in the repo so future sessions can re-run it as a regression smoke and
operators have a reference for "the official two-party walkthrough." Also
parameterise the identity-dir and topic salt so the script doesn't accumulate
state across runs.

## Acceptance Criteria

### Agent
- [x] `tests/e2e/agent-conversation.sh` exists, executable, runs to completion
- [x] Topic name includes a unique salt (timestamp or PID) so re-runs don't
      pile onto the same topic
- [x] Script self-tests for required prereqs (termlink in PATH, hub running,
      tmp identity dirs writable) and exits with a clear message on failure
- [x] README-style block at the top documents what it tests and how to invoke
- [x] `agent-conversations.md` references the script under "End-to-end test"
- [x] Script exits 0 on full success

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
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
test -x tests/e2e/agent-conversation.sh
bash -n tests/e2e/agent-conversation.sh

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

### 2026-04-27T15:51:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1327-persist-e2e-agent-conversation-walkthrou.md
- **Context:** Initial task creation

### 2026-04-27T15:53:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
