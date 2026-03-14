---
id: T-138
name: "/self-test framework skill — E2E validation loop"
description: >
  Framework skill that spawns a TermLink session, runs a sequence of framework
  commands interactively, observes output, reports structured results.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [framework, skill, self-test]
components: []
related_tasks: [T-136, T-137]
created: 2026-03-14T17:07:00Z
last_update: 2026-03-14T20:27:58Z
date_finished: null
---

# T-138: /self-test framework skill — E2E validation loop

## Context

Phase 2 from T-136 inception. Depends on T-137 (`termlink interact`).
Creates a `/self-test` skill for Claude Code that automates:

1. Spawn test session (`termlink register --name self-test --shell`)
2. Run a configurable sequence of framework commands via `termlink interact`
3. Collect pass/fail results per command
4. Report structured results to the agent
5. Clean up session

## Acceptance Criteria

### Agent
- [x] `/self-test` skill created in `.claude/commands/self-test.md`
- [x] Runs configurable command sequence (default: `fw doctor`; user can pass custom commands)
- [x] Reports pass/fail per command with output excerpts (structured table + failure details)
- [x] Auto-spawns and cleans up TermLink test session (osascript spawn, PID-based kill)
- [x] Handles timeout and session failure gracefully (TIMEOUT/SESSION_LOST/SKIPPED states)

## Verification

# Skill file exists as Claude Code command
test -f .claude/commands/self-test.md

## Updates

### 2026-03-14T17:07:00Z — task-created
- Phase 2 build task from T-136 inception. Depends on T-137.

### 2026-03-14T20:27:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
