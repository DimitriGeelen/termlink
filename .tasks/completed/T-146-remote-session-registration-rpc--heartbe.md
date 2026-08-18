---
id: T-146
name: "Remote session registration RPC + heartbeat"
description: >
  Remote session registration RPC with heartbeat and TTL

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [tcp, hub]
components: []
related_tasks: []
created: 2026-03-15T22:06:26Z
last_update: '2026-08-18T18:58:50Z'
date_finished: 2026-03-15T22:49:50Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:59Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:50Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-146: Remote session registration RPC + heartbeat

## Context

Hub-mediated registration for remote TCP sessions. Remote sessions register
via RPC, heartbeat to stay alive, auto-expire after TTL. See T-144 inception.

## Acceptance Criteria

### Agent
- [x] In-memory remote session store in hub (thread-safe, TTL-based)
- [x] `session.register_remote` RPC method stores remote session entry
- [x] `session.heartbeat` RPC method refreshes TTL
- [x] `session.deregister_remote` RPC method removes entry
- [x] Background reaper task expires stale entries (default TTL: 5 min, reap every 30s)
- [x] `session.discover` returns both local FS and remote sessions
- [x] All existing tests pass (262 total)
- [x] 5 new tests for remote store (register, heartbeat, deregister, expiry, JSON format)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

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

### 2026-03-15T22:06:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-146-remote-session-registration-rpc--heartbe.md
- **Context:** Initial task creation

### 2026-03-15T22:12:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-15T22:49:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
