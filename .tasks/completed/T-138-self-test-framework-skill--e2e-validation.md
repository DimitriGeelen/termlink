---
id: T-138
name: "/self-test framework skill — E2E validation loop"
description: >
  Framework skill that spawns a TermLink session, runs a sequence of framework
  commands interactively, observes output, reports structured results.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [framework, skill, self-test]
components: []
related_tasks: [T-136, T-137]
created: 2026-03-14T17:07:00Z
last_update: '2026-08-18T18:58:49Z'
date_finished: 2026-03-14T21:23:00Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:49Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
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

### 2026-03-14T21:23:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
