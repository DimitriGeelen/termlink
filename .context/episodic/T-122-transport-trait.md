---
task_id: T-122
task_name: "Transport trait + Unix socket adapter"
workflow_type: build
duration_days: 1
tags: [protocol, session, hub, transport]
related_tasks: [T-073, T-011]
---

# T-122: Transport trait + Unix socket adapter

## What Was Done

Implemented the transport abstraction layer from T-073 inception. Created `TransportAddr` enum (Unix/Tcp variants) in protocol crate with serde support. Defined `Transport`, `Connection`, `TransportListener`, and `LivenessProbe` traits in session crate. Built `UnixTransport` adapter wrapping existing Unix socket code. Replaced `Registration.socket: PathBuf` with `Registration.addr: RegistrationAddr` (backward-compatible serde — reads old "socket" format, writes new "addr"). Refactored all 10 coupling points (7 session, 3 hub) to use `socket_path()` accessor.

## Key Files

- `crates/termlink-protocol/src/transport.rs` — TransportAddr enum, serde
- `crates/termlink-session/src/transport.rs` — traits + UnixTransport adapter
- `crates/termlink-session/src/registration.rs` — RegistrationAddr wrapper
- `crates/termlink-hub/src/router.rs` — socket_path() migration
- `crates/termlink-cli/src/main.rs` — socket_path() migration

## Decisions

None — design was pre-decided in T-073 inception.

## Learnings

- Backward-compatible serde with custom deserializer allows smooth migration of on-disk formats
- Separating `LivenessProbe` from `Transport` trait was correct — liveness strategy differs per transport type (Unix: file existence + connect, TCP: connect + heartbeat)

## What Went Well

- All 223+ tests continued passing after refactoring 10 coupling points
- Backward-compatible registration format means no migration needed for existing sessions

## What Could Be Better

- Agent didn't auto-commit in worktree — required manual commit + rebase (root cause for T-126)
- Merge conflict with T-120 in router.rs needed manual resolution (JoinSet pattern + socket_path accessor)
