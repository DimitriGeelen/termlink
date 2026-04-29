---
id: T-1393
name: "Arc-conversation e2e regression runner — single command for all 4 cross-hub tests"
description: >
  Arc-conversation e2e regression runner — single command for all 4 cross-hub tests

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:37:59Z
last_update: 2026-04-28T21:39:35Z
date_finished: 2026-04-28T21:39:35Z
---

# T-1393: Arc-conversation e2e regression runner — single command for all 4 cross-hub tests

## Context

T-1387, T-1390, T-1391, T-1392 produced 4 cross-hub e2e scripts. Today there is no single way to run them all and get a clean PASS/FAIL. This task adds `tests/e2e/arc-suite.sh` — a thin runner that does a fleet pre-flight (.107 + .122 hubs reachable, both at >= a minimum version), then runs the 4 scripts in order and emits a summary table. Single command surface for "is the agent-conversation arc still green?".

## Acceptance Criteria

### Agent
- [x] `tests/e2e/arc-suite.sh` exists, executable
- [x] Pre-flight check: aborts with actionable error if either .107 or .122 hub is unreachable
- [x] Pre-flight check: aborts if either hub is below `0.9.1542` (the `channel.*` version watermark)
- [x] Runs the 4 e2e scripts in order: live-agents-conversation, cross-hub-bidirectional-6agents, cross-hub-matrix-flow, cross-hub-presence-flow
- [x] On any failure: stops, prints which script failed, exits non-zero
- [x] On success: prints a summary table with `PASS` per script + total elapsed time + `ARC SUITE GREEN` marker
- [x] All four sub-scripts pass when invoked through the suite
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

test -x tests/e2e/arc-suite.sh
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

### 2026-04-28T21:37:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1393-arc-conversation-e2e-regression-runner--.md
- **Context:** Initial task creation

### 2026-04-28T21:39:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
