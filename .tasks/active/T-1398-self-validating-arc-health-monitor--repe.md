---
id: T-1398
name: "Self-validating arc health monitor — repeat suite N times, post results to arc-health topic"
description: >
  Self-validating arc health monitor — repeat suite N times, post results to arc-health topic

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T22:01:40Z
last_update: 2026-04-28T22:01:40Z
date_finished: null
---

# T-1398: Self-validating arc health monitor — repeat suite N times, post results to arc-health topic

## Context

The arc-suite runs once and reports a single PASS/FAIL. To demonstrate sustained arc health (no resource leaks, no degradation, stable behavior across runs), this task adds `tests/e2e/arc-health-monitor.sh` — a self-validating monitor that:

1. Runs the arc-suite N times (default 5) in sequence
2. After each run, posts the PASS/FAIL + duration to a `arc-health:report` channel topic on .107
3. After all runs, computes summary stats (pass rate, min/max/median duration)
4. Cross-hub verification: reads the arc-health topic from .122 to confirm the validator's own outputs are visible cross-hub

Demonstrates two properties at once:
- The arc is reliable across repeated invocations (no degradation)
- The arc-health monitor itself uses the arc to report results (eat your own dog food)

## Acceptance Criteria

### Agent
- [x] `tests/e2e/arc-health-monitor.sh` exists, executable
- [x] Default N=5 (configurable via `RUNS=N`)
- [x] Arc-health topic auto-created with `messages:200` retention
- [x] Each suite run posts a PASS/FAIL + duration_seconds envelope to arc-health topic
- [x] Summary computed: pass count / fail count / min/max/median duration
- [x] Cross-hub read: arc-health topic state read from .122 contains all N runs
- [x] Exits 0 only when ALL runs pass
- [x] Script exits 0 with `ARC HEALTH MONITOR OK` marker
- [x] Soak completes in < 3 minutes wall-clock (3-run actual ~40s; 5-run ~65s)
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

test -x tests/e2e/arc-health-monitor.sh
out=$(RUNS=3 BIN=./target/release/termlink ./tests/e2e/arc-health-monitor.sh 2>&1) && echo "$out" | grep -q "ARC HEALTH MONITOR OK"

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

### 2026-04-28T22:01:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1398-self-validating-arc-health-monitor--repe.md
- **Context:** Initial task creation
