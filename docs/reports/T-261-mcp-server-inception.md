# T-261: MCP Server for TermLink — Inception Research

## Core Question

Should TermLink expose its commands as MCP tools so Claude Code, Cursor, and other MCP-capable agents can orchestrate terminals structurally?

## MCP Protocol Summary

- **Spec:** 2025-11-25 (stable)
- **Primitives:** Tools (actions), Resources (data), Prompts (templates)
- **Transports:** stdio (subprocess), Streamable HTTP (remote)
- **Rust SDK:** `rmcp` (official, from modelcontextprotocol/rust-sdk). Proc macro `#[tool]`, pluggable transports.

## Command Mapping

### High Value (MCP Tools)
| TermLink Command | MCP Type | Why |
|---|---|---|
| `exec` | Tool | Execute commands on remote sessions — core agent workflow |
| `inject` | Tool | Send input to a terminal |
| `output` | Tool | Read terminal output |
| `signal` | Tool | Send SIGINT/SIGTERM — cleanup/abort |
| `event emit` | Tool | Publish events for coordination |
| `event emit-to` | Tool | Push events to specific session |
| `event wait` | Tool | Wait for a condition — orchestration primitive |
| `ping` | Tool | Health check before dispatching |
| `list` | Resource | Session discovery |
| `status` | Resource | Session state before acting |
| `discover` | Resource | Find sessions across hubs |

### Medium Value
| Command | Why |
|---|---|
| `event broadcast` | Multi-target coordination |
| `event collect` | Aggregate events |
| `register --self` | Spawn endpoint on demand |
| `register --shell` | Spawn terminal on demand |
| `info` | System diagnostics |

### Skip (interactive-only or infrastructure)
attach, stream, mirror, resize, hub, clean, watch, send

## Architecture

**`termlink mcp serve` subcommand + `crates/termlink-mcp` workspace crate.**

- Separate crate isolates `rmcp` dependency
- Thin adapter: connects to TermLink sessions via existing Unix sockets/JSON-RPC
- Stdio transport for v1 (Claude Code, Cursor spawn as subprocess)
- Streamable HTTP as v2 follow-up (remote agents)

## Auth Model

- **Stdio (v1):** No auth — process isolation is the security boundary
- **HTTP (v2):** TermLink token auth in handshake
- **Scope:** Per-session permissions map to existing `PermissionScope`

## Composition

Framework MCP server (T-599 on .107) is a peer, not a parent. TermLink MCP is a separate server. Framework can list it as a composable MCP server in its config. No merging.

## Estimate

- ~10 high-value tools + 2-3 resources
- `rmcp` handles transport, serialization, protocol
- Each tool is ~30-50 lines (parse params → call TermLink RPC → format result)
- Total: ~800-1200 lines in new crate + CLI wiring
- 1-2 sessions to build
