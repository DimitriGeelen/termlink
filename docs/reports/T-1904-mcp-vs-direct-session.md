# T-1904 — MCP client vs direct session: how does termlink connect to its own MCP server?

**Status:** Inception, exploration phase. No spikes executed yet.
**Filed:** 2026-06-01
**Owner:** human (decision authority); claude-code (advisory)
**Recommendation at filing:** DEFER (no evidence yet)

This is the live research artifact for T-1904. Updated incrementally as spikes
produce findings. The thinking trail IS the artifact — conversations are
ephemeral, files are permanent (C-001).

## The question (one sentence)

Do termlink's own components (CLI, internal callers) reach `termlink-mcp`
operations via an MCP client (proper protocol round-trip) or via direct calls
into shared session/hub library code?

## Why it matters

See `## Problem Statement` in the task file. Headline: dual-stack surfaces
that both call a shared primitive are healthy; dual-stack surfaces that
re-derive the same logic are a divergence risk.

## Hypotheses (filed as assumptions)

- **H1.** `termlink-mcp` tool handlers `use termlink_session::*` and call
  shared primitives directly.
- **H2.** `termlink-cli` command handlers also `use termlink_session::*`
  directly, NOT `termlink-mcp` client APIs.
- **H3.** Cross-host calls (`termlink_remote_call`) ride hub-rpc JSON-RPC over
  TCP+TLS, not the rmcp MCP protocol.
- **H4.** No in-process MCP loopback exists.

If H1+H2 hold, the answer is "we share a library, not a protocol — CLI does
NOT go through MCP."

## Findings

(populated as spikes execute — empty for now per inception discipline)

### Spike 1 — Surface map

_pending_

### Spike 2 — Dependency direction

_pending_

### Spike 3 — Sample one CLI path (`channel post`)

_pending_

## Dialogue Log

### 2026-06-01T — Operator: file inception
- **Question posed:** "please start a new inception to inspect if we are using MCP client to connect to our termlink MCP server or are we calling the client session directly"
- **Agent action:** Filed T-1904, drafted Problem Statement / Assumptions / Exploration Plan / Constraints / Scope Fence / Go-No-Go. Created this artifact. Awaiting operator review before executing spikes.

## Recommendation (pending evidence)

**DEFER** — Investigation has not been executed yet. After spikes 1-3, this
file's Recommendation section will be updated with one of GO/NO-GO/DEFER and a
referenced-evidence rationale per `## Go/No-Go Criteria` in the task file.
