---
id: T-029
name: "Data plane — async frame codec and streaming server"
description: >
  Data plane — async frame codec and streaming server

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:58:14Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T20:19:55Z
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
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-029: Data plane — async frame codec and streaming server

## Context

Build the data plane infrastructure: async frame codec (FrameReader/FrameWriter) and streaming data server. Uses the binary frame protocol from termlink-protocol for low-latency PTY I/O streaming over a separate Unix socket.

## Acceptance Criteria

### Agent
- [x] `codec` module with `FrameReader<R>` and `FrameWriter<W>` for async frame I/O
- [x] `FrameWriter` auto-increments sequence numbers
- [x] `data_server` module with `run()` function binding `{control}.data` socket
- [x] Output broadcast: PTY output forwarded as Output frames to connected clients
- [x] Input handling: Input frames written to PTY master
- [x] Resize handling: Resize frames trigger PTY resize
- [x] Ping/Pong keepalive support
- [x] PTY `read_loop_with_broadcast` variant for data plane integration
- [x] 8 new tests (4 codec + 4 data server), 110 total passing

## Verification

/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -1

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

### 2026-03-08T19:58:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-029-data-plane--async-frame-codec-and-stream.md
- **Context:** Initial task creation

### 2026-03-08T20:19:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
