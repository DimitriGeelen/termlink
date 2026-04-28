---
id: T-1396
name: "Operator runbook for agent-conversation arc e2e suite + wire stress-soak into runner"
description: >
  Operator runbook for agent-conversation arc e2e suite + wire stress-soak into runner

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:45:14Z
last_update: 2026-04-28T21:45:14Z
date_finished: null
---

# T-1396: Operator runbook for agent-conversation arc e2e suite + wire stress-soak into runner

## Context

T-1390..T-1395 produced 6 e2e scripts + a regression runner. There is no operator-facing doc — somebody returning to the project later won't know the suite exists or how to run it. This task adds:

1. `docs/operations/agent-conversation-arc-e2e.md` — runbook (what each script tests, fleet pre-reqs, troubleshooting)
2. Wire `cross-hub-stress-soak.sh` into `arc-suite.sh` (currently standalone) so the regression runner exercises the stress phase too

## Acceptance Criteria

### Agent
- [x] `docs/operations/agent-conversation-arc-e2e.md` exists
- [x] Runbook describes what each of the 6 e2e scripts tests + the suite runner
- [x] Runbook documents fleet pre-reqs (channel.* version watermark + hub reachability)
- [x] Runbook includes a troubleshooting section for common failure modes (session drift, .122 hub auth, version skew)
- [x] `arc-suite.sh` includes `cross-hub-stress-soak.sh` (stress is the final phase of the suite)
- [x] Full suite still passes end-to-end with stress-soak included
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

test -f docs/operations/agent-conversation-arc-e2e.md
grep -q "cross-hub-stress-soak.sh" tests/e2e/arc-suite.sh
out=$(BIN=./target/release/termlink ./tests/e2e/arc-suite.sh 2>&1) && echo "$out" | grep -q "ARC SUITE GREEN"

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

### 2026-04-28T21:45:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1396-operator-runbook-for-agent-conversation-.md
- **Context:** Initial task creation
