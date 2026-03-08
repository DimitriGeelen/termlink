---
id: T-013
name: "Tech stack decision — implementation language and project structure"
description: >
  Inception: Tech stack decision — implementation language and project structure

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-08T15:29:23Z
last_update: 2026-03-08T15:31:57Z
date_finished: 2026-03-08T15:31:57Z
---

# T-013: Tech stack decision — implementation language and project structure

## Problem Statement

What language and project structure should TermLink use? The protocol is designed (T-005), identity system designed (T-006), protocol composition decided (T-004). Now we need to pick the implementation language and set up the project. This choice affects build speed, runtime performance, deployment story, and contributor accessibility.

## Requirements from Prior Decisions

- **Unix domain sockets** — first-class, both control and data plane (T-004)
- **JSON-RPC 2.0** — control plane serialization (T-005)
- **Binary frame parsing** — 22-byte fixed header, big-endian, zero-copy desirable (T-005)
- **Filesystem operations** — inotify/FSEvents, atomic file writes, directory scanning (T-006)
- **Cross-platform** — macOS + Linux minimum (D4 portability)
- **Low-latency** — data plane needs sub-millisecond framing overhead
- **Single binary distribution** — `brew install termlink` or equivalent
- **MCP server implementation** — must implement MCP protocol for control plane

## Candidates

| Language | Unix sockets | Binary parsing | JSON-RPC | inotify/FSEvents | Single binary | MCP ecosystem | Agent assist |
|----------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Rust** | Excellent (tokio/mio) | Excellent (zero-copy) | Good (jsonrpsee) | Good (notify crate) | Yes (static) | Growing (mcp-rust-sdk) | Good |
| **Go** | Excellent (net) | Good (encoding/binary) | Good (gorilla-rpc) | Good (fsnotify) | Yes (static) | Growing (mcp-go) | Good |
| **TypeScript/Node** | Good (net module) | Adequate (Buffer) | Excellent (json-rpc libs) | Good (chokidar/fs.watch) | Via pkg/bun compile | Best (official @anthropic-ai/sdk) | Excellent |
| **Python** | Good (asyncio) | Adequate (struct) | Good (jsonrpclib) | Good (watchdog) | Via PyInstaller (heavy) | Good (mcp SDK) | Excellent |

## Analysis

### Rust
**Pros:** Best performance, zero-copy binary parsing, true single binary, strong type system catches protocol errors at compile time, excellent async (tokio), memory safety without GC pauses.
**Cons:** Slower development velocity, steeper learning curve, longer compile times. Fewer MCP libraries (but growing).

### Go
**Pros:** Fast development, excellent concurrency (goroutines map to session handling), single binary, good ecosystem, readable code. Solid MCP support emerging.
**Cons:** No zero-copy without unsafe. GC pauses (microsecond-level, likely fine). Less type safety for protocol code than Rust.

### TypeScript (Bun/Node)
**Pros:** Fastest development velocity, best MCP ecosystem (official SDK), excellent JSON handling, async/await native. Bun can compile to single binary. Largest contributor pool.
**Cons:** Runtime overhead for binary parsing, GC pauses, V8 memory footprint, single-threaded event loop. Binary distribution via Bun compile is newer/less proven.

### Python
**Pros:** Fastest prototyping, good MCP SDK, excellent for AI agent integration.
**Cons:** Slow binary parsing, GIL limits concurrency, heavy distribution (PyInstaller bundles are 50MB+), not suitable for low-latency data plane.

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- A clear language choice emerges that satisfies performance, portability, and development velocity needs
- The chosen language has adequate Unix socket and filesystem watching support

**NO-GO if:**
- No language adequately covers all requirements (would need polyglot approach)

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: Rust chosen: zero-copy binary framing, no GC, single static binary, type-safe protocol code, excellent async (tokio). Human confirmed choice.

**Date**: 2026-03-08T15:31:57Z
## Decision

**Decision**: GO

**Rationale**: Rust chosen: zero-copy binary framing, no GC, single static binary, type-safe protocol code, excellent async (tokio). Human confirmed choice.

**Date**: 2026-03-08T15:31:57Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-08T15:31:47Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Rust chosen: zero-copy binary framing, no GC, single static binary, strong type safety for protocol code, excellent async (tokio), memory safety. Protocol spec (T-005) is detailed enough that implementation is translation. Best alignment with D1 (memory safety), D2 (type-checked protocol), D3 (single binary install), D4 (cross-platform, no runtime deps).

### 2026-03-08T15:31:57Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-08T15:31:57Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Rust chosen: zero-copy binary framing, no GC, single static binary, type-safe protocol code, excellent async (tokio). Human confirmed choice.

### 2026-03-08T15:31:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
