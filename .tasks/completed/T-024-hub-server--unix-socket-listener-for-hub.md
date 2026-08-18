---
id: T-024
name: "Hub server — Unix socket listener for hub routing"
description: >
  Hub server — Unix socket listener for hub routing

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:13:02Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T19:30:21Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
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

# T-024: Hub server — Unix socket listener for hub routing

## Context

The hub crate has routing logic (`router.rs`) but no listener. This task adds a Unix socket server for the hub at `runtime_dir()/hub.sock`, plus a `termlink hub` CLI subcommand to start it. The hub server accepts JSON-RPC requests and routes them via `router::route` (session.discover handled locally, other methods forwarded to target sessions).

## Acceptance Criteria

### Agent
- [x] `hub::server` module with `run` function that binds `hub.sock` and accepts connections
- [x] Hub connection handler routes requests through `router::route`
- [x] `hub_socket_path()` function returns `runtime_dir()/hub.sock`
- [x] CLI `hub` subcommand starts the hub server
- [x] Tests: hub serves `session.discover`, hub forwards to target session, malformed JSON returns parse error
- [x] All existing tests still pass (102 total)

## Verification

export PATH="$HOME/.cargo/bin:$PATH" && cd /Users/dimidev32/001-projects/010-termlink && cargo test --workspace 2>&1 | tail -1
export PATH="$HOME/.cargo/bin:$PATH" && cd /Users/dimidev32/001-projects/010-termlink && cargo build 2>&1 | tail -1

## Decisions

### 2026-03-08 — Fix router id forwarding
- **Chose:** Use `Client::call` directly with original request id instead of `rpc_call` (which hardcodes `"cli-1"`)
- **Why:** Hub must preserve the caller's request id through forwarding
- **Rejected:** Patching the response id after the fact (fragile, breaks RpcResponse enum)

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-08T19:13:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-024-hub-server--unix-socket-listener-for-hub.md
- **Context:** Initial task creation

### 2026-03-08T19:30:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
