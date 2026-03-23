---
id: T-255
name: "Live orchestration test harness — OpenClaw-style observe-inject-iterate loop"
description: >
  Reusable test harness that spawns specialist sessions with specific roles/capabilities, starts the hub, and provides attach points for observation. Exercises all 15 T-247 scenarios via real TermLink sessions with the observe-inject-iterate pattern from the OpenClaw experiment. See docs/reports/T-247-orchestration-scenarios.md and all T-247-scenarios-*.md files.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-247, orchestration, testing, termlink]
components: []
related_tasks: [T-247, T-248, T-249, T-250, T-251, T-252, T-253, T-254, T-233]
created: 2026-03-23T16:54:33Z
last_update: 2026-03-23T16:54:33Z
date_finished: null
---

# T-255: Live orchestration test harness — OpenClaw-style observe-inject-iterate loop

## Context

Reusable test harness exercising all 15 T-247 scenarios via real TermLink sessions. Follows the OpenClaw experiment pattern: spawn agents, observe via `attach`/`mirror`, inject corrections, iterate. See `docs/reports/T-247-orchestration-scenarios.md` and all `docs/reports/T-247-scenarios-*.md` files. Depends on features built in T-248 through T-254.

## Acceptance Criteria

### Agent
- [ ] Shell script `tests/orchestration-harness.sh` spawns hub + 3 specialist sessions with distinct roles/capabilities
- [ ] Each specialist has a mock RPC handler that responds to specific methods with canned data
- [ ] Harness runs 5 scenario categories: health-check routing, parallel fan-out, bypass promotion lifecycle, failover with dead specialists, bypass de-promotion
- [ ] Harness outputs structured results (pass/fail per scenario with timing)
- [ ] `termlink attach` / `termlink mirror` work on specialist sessions during harness run
- [ ] Cleanup: harness tears down all sessions and hub on exit (trap handler)
- [ ] All workspace tests still pass (`cargo test --package termlink-hub`)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

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

### 2026-03-23T16:54:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-255-live-orchestration-test-harness--opencla.md
- **Context:** Initial task creation
