---
id: T-122
name: "Transport trait + Unix socket adapter"
description: >
  Define Transport trait + TransportAddr enum. Wrap existing Unix socket code in
  adapter structs. Refactor 10 coupling points (7 session, 3 hub) to use traits.
  From T-073 inception GO.
status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [protocol, session, hub, transport]
components: []
related_tasks: [T-073, T-011]
created: 2026-03-12T20:17:49Z
last_update: 2026-03-12T20:17:49Z
date_finished: null
---

# T-122: Transport trait + Unix socket adapter

## Context

From T-073 inception (docs/reports/T-073-exploration.md). 10 Unix socket coupling points
across session (7) and hub (3). Protocol crate is clean. Proposed design: TransportAddr
enum in protocol, Transport/Connection traits in session, Box<dyn Connection> dispatch.

## Acceptance Criteria

### Agent
- [ ] `TransportAddr` enum in protocol crate: `Unix { path }`, `Tcp { host, port }` (serde only, no runtime deps)
- [ ] `Transport` trait in session crate: `connect(addr) -> Connection`, `bind(addr) -> Listener`
- [ ] `Connection` trait: blanket impl over `AsyncRead + AsyncWrite + Send + Unpin`
- [ ] `TransportListener` trait: `accept() -> Connection`, `local_addr() -> TransportAddr`
- [ ] `LivenessProbe` trait: separate from transport (strategy differs per transport type)
- [ ] Unix socket adapter wraps existing `UnixListener`/`UnixStream` — all existing tests pass
- [ ] `Registration.socket: PathBuf` replaced with `Registration.addr: TransportAddr`
- [ ] All 10 coupling points refactored to use traits (7 session + 3 hub)
- [ ] Full test suite passes (223+ tests)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -1 | grep -q "ok"
/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1 | grep -qv "error"

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
