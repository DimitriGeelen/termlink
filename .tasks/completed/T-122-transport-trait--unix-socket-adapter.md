---
id: T-122
name: "Transport trait + Unix socket adapter"
description: >
  Define Transport trait + TransportAddr enum. Wrap existing Unix socket code in
  adapter structs. Refactor 10 coupling points (7 session, 3 hub) to use traits.
  From T-073 inception GO.
status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [protocol, session, hub, transport]
components: []
related_tasks: [T-073, T-011]
created: 2026-03-12T20:17:49Z
last_update: '2026-08-18T18:58:46Z'
date_finished: 2026-03-13T09:56:20Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:50Z'
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
  - ts: '2026-08-18T18:58:46Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-122: Transport trait + Unix socket adapter

## Context

From T-073 inception (docs/reports/T-073-exploration.md). 10 Unix socket coupling points
across session (7) and hub (3). Protocol crate is clean. Proposed design: TransportAddr
enum in protocol, Transport/Connection traits in session, Box<dyn Connection> dispatch.

## Acceptance Criteria

### Agent
- [x] `TransportAddr` enum in protocol crate: `Unix { path }`, `Tcp { host, port }` (serde only, no runtime deps)
- [x] `Transport` trait in session crate: `connect(addr) -> Connection`, `bind(addr) -> Listener`
- [x] `Connection` trait: blanket impl over `AsyncRead + AsyncWrite + Send + Unpin`
- [x] `TransportListener` trait: `accept() -> Connection`, `local_addr() -> TransportAddr`
- [x] `LivenessProbe` trait: separate from transport (strategy differs per transport type)
- [x] Unix socket adapter wraps existing `UnixListener`/`UnixStream` — all existing tests pass
- [x] `Registration.socket: PathBuf` replaced with `Registration.addr: TransportAddr`
- [x] All 10 coupling points refactored to use traits (7 session + 3 hub)
- [x] Full test suite passes (223+ tests)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | grep -qv "^error"

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

### 2026-03-12T20:17:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-122-transport-trait--unix-socket-adapter.md
- **Context:** Initial task creation

### 2026-03-13T09:56:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
