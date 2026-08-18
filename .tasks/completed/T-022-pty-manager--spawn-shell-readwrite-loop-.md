---
id: T-022
name: "PTY manager — spawn shell, read/write loop, scrollback buffer"
description: >
  PTY manager — spawn shell, read/write loop, scrollback buffer

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T18:23:52Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T18:41:14Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-022: PTY manager — spawn shell, read/write loop, scrollback buffer

## Context

Implements PTY-backed sessions from T-007 GO decision. See `docs/reports/T-007-output-capture-bidirectional.md`.

## Acceptance Criteria

### Agent
- [x] `pty` module with PTY spawn, read loop, write, and resize
- [x] `scrollback` module with byte-oriented ring buffer
- [x] PTY read loop feeds scrollback buffer
- [x] Write to PTY master for input injection
- [x] Tests: spawn shell + echo, scrollback append/query, PTY read/write roundtrip
- [x] All tests pass (`cargo test --workspace`) — 96 tests

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace

## Decisions

### 2026-03-08 — PTY implementation
- **Chose:** Raw libc (`openpty`, `fork`, `exec`) — no extra dependencies
- **Why:** libc is already a workspace dep; avoids pulling in nix or portable-pty for a focused use case
- **Rejected:** `nix` crate (extra dep), `portable-pty` (heavy, cross-platform not needed yet)

## Updates

### 2026-03-08T18:23:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-022-pty-manager--spawn-shell-readwrite-loop-.md
- **Context:** Initial task creation

### 2026-03-08T18:41:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
