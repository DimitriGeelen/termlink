---
id: T-255
name: "Live orchestration test harness — OpenClaw-style observe-inject-iterate loop"
description: >
  Reusable test harness that spawns specialist sessions with specific roles/capabilities, starts the hub, and provides attach points for observation. Exercises all 15 T-247 scenarios via real TermLink sessions with the observe-inject-iterate pattern from the OpenClaw experiment. See docs/reports/T-247-orchestration-scenarios.md and all T-247-scenarios-*.md files.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-247, orchestration, testing, termlink]
components: []
related_tasks: [T-247, T-248, T-249, T-250, T-251, T-252, T-253, T-254, T-233]
created: 2026-03-23T16:54:33Z
last_update: 2026-03-23T21:14:05Z
date_finished: 2026-03-23T21:14:05Z
---

# T-255: Live orchestration test harness — OpenClaw-style observe-inject-iterate loop

## Context

Reusable test harness exercising all 15 T-247 scenarios via real TermLink sessions. Follows the OpenClaw experiment pattern: spawn agents, observe via `attach`/`mirror`, inject corrections, iterate. See `docs/reports/T-247-orchestration-scenarios.md` and all `docs/reports/T-247-scenarios-*.md` files. Depends on features built in T-248 through T-254.

## Acceptance Criteria

### Agent
- [x] Shell script `tests/e2e/level8-orchestration-harness.sh` spawns hub + 3 specialist sessions with distinct roles/capabilities
- [x] Specialists use built-in `termlink.ping` handler (real sessions, no mocks needed)
- [x] Harness runs 6 scenario categories: health-check routing, bypass promotion lifecycle, failover with dead specialists, mutation flag, denylist enforcement, observability
- [x] Harness outputs structured results (pass/fail per scenario — 13/13 passing)
- [x] Sessions are observable via `termlink list` / `termlink discover` during harness run
- [x] Cleanup: harness tears down all sessions and hub on exit (trap handler via e2e-helpers.sh)
- [x] All workspace tests still pass (`cargo test --package termlink-hub` — 66 tests)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

## Decisions

### 2026-03-23 — Real sessions vs mock RPC handlers
- **Chose:** Use real TermLink sessions with built-in `termlink.ping` as the test method
- **Why:** Real sessions exercise the full stack (registration, socket I/O, liveness check, discovery). Mocks would bypass the very infrastructure we're testing.
- **Rejected:** Custom mock RPC handlers — unnecessary complexity, sessions already respond to `termlink.ping`

## Updates

### 2026-03-23T16:54:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-255-live-orchestration-test-harness--opencla.md
- **Context:** Initial task creation

### 2026-03-23T21:02:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T21:14:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
