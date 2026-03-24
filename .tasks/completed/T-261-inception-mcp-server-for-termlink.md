---
id: T-261
name: "Inception: MCP server for TermLink"
description: >
  Pickup from fw-agent T-599. Build MCP server exposing TermLink commands as structured tools for Claude Code, Cursor, etc. Key questions: which commands as MCP tools, standalone binary vs subcommand, auth model, composition with framework MCP server.

status: work-completed
workflow_type: inception
owner: agent
horizon: next
tags: [pickup, mcp]
components: []
related_tasks: []
created: 2026-03-24T09:27:23Z
last_update: 2026-03-24T11:47:56Z
date_finished: 2026-03-24T11:47:56Z
---

# T-261: Inception: MCP server for TermLink

## Problem Statement

MCP-capable agents (Claude Code, Cursor) can't structurally interact with TermLink sessions. They must shell out to `termlink` CLI commands — losing type safety, error handling, and discoverability. An MCP server would expose TermLink as structured tools that agents can discover and call natively.

**For whom:** Any MCP client that wants to orchestrate terminals (Claude Code, Cursor, framework agents).
**Why now:** Framework is building its own MCP server (T-599). TermLink as a composable MCP peer enables structured multi-agent terminal orchestration. The T-233 orchestration system (events, negotiation, trust) is built and ready to be exposed.

## Assumptions

- A1: `rmcp` (official Rust MCP SDK) is stable enough for production use
- A2: Stdio transport is sufficient for v1 (Claude Code/Cursor spawn as subprocess)
- A3: TermLink's existing JSON-RPC over Unix sockets maps cleanly to MCP tool calls
- A4: ~10 high-value tools + 2-3 resources covers 90% of agent use cases

## Exploration Plan

1. **Validate A1:** Check `rmcp` crate maturity — version, download count, API stability (15 min)
2. **Validate A3:** Spike one tool (exec or ping) to confirm TermLink RPC → MCP tool mapping works (30 min)
3. **Design:** Map all high-value commands to MCP tool/resource definitions (30 min)
4. **Estimate:** Size the build task (number of tools, lines of code, sessions needed)

## Technical Constraints

- `rmcp` adds dependency on tokio, serde (already in workspace), plus rmcp-specific deps
- Stdio transport: MCP client spawns `termlink mcp serve` as subprocess — must be fast to start
- Each tool call round-trips through Unix socket to target session — latency ~1-5ms
- MCP spec has no streaming primitive for long-running output — use polling (output) or events (wait)

## Scope Fence

**IN scope:** MCP server design, tool/resource mapping, `rmcp` feasibility, architecture decision, go/no-go
**OUT of scope:** Full implementation (that's the build task), HTTP transport, OAuth, framework composition

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (A1-A4 via research)
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- `rmcp` is stable and handles stdio transport
- TermLink RPC maps cleanly to MCP tools (no impedance mismatch)
- Build is bounded (1-2 sessions, <1500 lines)

**NO-GO if:**
- `rmcp` is unstable or requires significant workarounds
- MCP tool model can't express TermLink's async operations (exec, wait)
- Build cost exceeds value (agents can just shell out to CLI)

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: MCP server replaces CLI string parsing with structured, typed, discoverable tools. Low cost (~1200 lines, rmcp handles protocol). Framework MCP (T-599) + TermLink MCP compose as peers. T-233 orchestration stack is built and ready to be exposed.

**Date**: 2026-03-24T11:47:56Z
## Decision

**Decision**: GO

**Rationale**: MCP server replaces CLI string parsing with structured, typed, discoverable tools. Low cost (~1200 lines, rmcp handles protocol). Framework MCP (T-599) + TermLink MCP compose as peers. T-233 orchestration stack is built and ready to be exposed.

**Date**: 2026-03-24T11:47:56Z

## Updates

### 2026-03-24T09:27:23Z — task-created [pickup from fw-agent T-599 on .107]
- **Source:** File transfer via TermLink (`termlink-pickup-002-mcp-server.md`)
- **Pickup message:** Build MCP server exposing TermLink commands as structured, discoverable tools for MCP-capable agents. Framework is building its own MCP server (T-599) and wants TermLink as a composable MCP peer.

### 2026-03-24T10:57:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-24T11:47:56Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** MCP server replaces CLI string parsing with structured, typed, discoverable tools. Low cost (~1200 lines, rmcp handles protocol). Framework MCP (T-599) + TermLink MCP compose as peers. T-233 orchestration stack is built and ready to be exposed.

### 2026-03-24T11:47:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
