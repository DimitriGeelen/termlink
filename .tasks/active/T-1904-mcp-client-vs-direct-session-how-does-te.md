---
id: T-1904
name: "MCP client vs direct session: how does termlink connect to its own MCP server"
description: >
  Inception: MCP client vs direct session: how does termlink connect to its own MCP server

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T08:22:21Z
last_update: 2026-06-01T08:23:58Z
date_finished: null
---

# T-1904: MCP client vs direct session: how does termlink connect to its own MCP server

## Problem Statement

**The question:** When termlink's own components (CLI commands, internal callers, agents-on-the-same-host)
need to perform an operation that the MCP server (`termlink-mcp`) exposes — e.g. `channel.post`,
`hub.capabilities`, `agent.contact` — do they go through an MCP client (proper protocol round-trip
via JSON-RPC over stdio/TCP) or do they call into the session/hub library code directly?

**Why this matters:**

1. **Surface validation.** If everything internal bypasses MCP and only external clients
   (claude-code, etc.) hit `termlink-mcp`, then the MCP surface is structurally under-validated —
   it works in tests but isn't exercised by the dogfooded path. Drift between CLI behavior and
   MCP behavior is then invisible until an external consumer hits it.
2. **Maintenance multiplier.** Two implementations of the same operation (CLI handler + MCP
   tool handler) means every feature must be wired in twice. If they call a shared session
   primitive, that's fine; if they re-derive the same logic, that's a divergence risk.
3. **Performance.** Internal MCP-client round-trips add JSON ser/deserialize + transport
   overhead that direct session calls do not pay. If the codebase has accidental in-process
   MCP loopbacks they show up as latency that's hard to explain.
4. **Goal alignment.** T-1166 has been about consolidating TermLink to a "channel" abstraction
   exposed via MCP. If the CLI itself doesn't go through MCP, the consolidation lives only at
   the external boundary — that's a partial win, and the next operator pull may be "make the
   CLI a thin MCP client" to close the loop.

**For whom:** TermLink maintainers (us, future agent sessions) and external integrators who depend
on `termlink-mcp` matching CLI behavior 1:1.

**Why now:** Post-T-1166 cut, the system has converged on a smaller surface area. This is the
right inflection point to decide whether MCP-as-internal-protocol is a goal worth pursuing or
whether the current dual-stack arrangement is intentional and stable.

## Assumptions

A1. The `termlink-mcp` crate is a rmcp-based MCP server with one tool per public operation. Tools
    dispatch into shared session/hub library code, not into a separate logic implementation.
    (Register with `fw assumption add`.)

A2. The `termlink-cli` crate's command handlers (`cmd_*` in `crates/termlink-cli/src/commands/`)
    call session/hub library code DIRECTLY — they do not instantiate an MCP client and they do
    not make in-process JSON-RPC calls to a colocated `termlink-mcp` server.

A3. The `termlink_remote_call` MCP tool (and similar) wrap a TCP+TLS RPC to a remote hub. That
    TCP path is JSON-RPC at the hub-router layer, NOT the rmcp MCP protocol. I.e. there are two
    distinct protocols in play: rmcp (claude-code ↔ termlink-mcp) and hub-rpc (termlink-mcp ↔
    remote-hub, and termlink-cli ↔ remote-hub).

A4. No in-process MCP loopback exists: termlink components never spawn a local `termlink-mcp`
    server and connect to it as an MCP client just to call a tool.

## Exploration Plan

Three time-boxed read-only spikes (≤15 min each, no code edits):

**Spike 1 — Surface map (5 min).** Enumerate the three crates' public boundaries:
- `crates/termlink-mcp/` — what tools are exposed? Do tool handlers `use termlink_session::*`?
  Grep for `rmcp::tool`, `#[tool(`, or whatever macro registers the tools.
- `crates/termlink-cli/` — what commands exist? Do they `use termlink_session::*` or
  `use termlink_mcp::*`?
- `crates/termlink-session/` and `crates/termlink-hub/` — the shared library layer everyone else
  presumably depends on.

**Spike 2 — Dependency direction (5 min).** Inspect `Cargo.toml` for each crate. Verify:
- `termlink-mcp` depends on `termlink-session` / `termlink-hub` (yes expected)
- `termlink-cli` depends on `termlink-session` / `termlink-hub` (yes expected)
- `termlink-cli` does NOT depend on `termlink-mcp` (would invalidate A2/A4 if it does)
- No circular deps via dev-deps or build-deps

**Spike 3 — Sample one CLI path (5 min).** Pick `termlink channel post` (a representative new
operation introduced for T-1166). Trace from `cmd_channel_post` → which session/hub function does
it call? Then look at the corresponding MCP tool `termlink_channel_post` — does it call the same
function? If both paths reach the same `session::channel::post()` primitive, that's the
canonical dogfooded shape and the answer is "we share a library, not a protocol."

Output: a one-page artifact at `docs/reports/T-1904-mcp-vs-direct-session.md` summarizing which
of the four assumptions held, plus a recommendation (GO / NO-GO / DEFER) on whether further
build work (e.g. "make CLI a thin MCP client" or "auto-test CLI vs MCP parity") is warranted.

## Technical Constraints

- **Read-only investigation.** No code changes during the inception phase. Only `cargo metadata`,
  grep, and source-file reads.
- **No spawning of MCP servers.** We don't need to runtime-instrument anything to answer the
  static-architecture question.
- **No external dependencies probed.** The MCP protocol spec, rmcp version, and stdio/TCP
  transport are out of scope here (covered separately by T-1060 / rmcp pin work).

## Scope Fence

**IN scope:**
- Static analysis of which crate calls which crate
- Identifying whether MCP tool handlers and CLI command handlers share a common library layer
- Recommending whether dogfooding (CLI-as-MCP-client) is worth a follow-up build task

**OUT of scope:**
- Any code changes (this is inception only)
- Performance benchmarking of MCP vs direct calls
- Changes to the rmcp dep, MCP tool surface, or CLI command surface
- The hub-rpc protocol (different layer; the question is about MCP specifically)
- Cross-host concerns — those route via TCP+TOFU+TLS regardless of caller type

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- CLI bypasses MCP today AND there is identifiable value in routing it through MCP
  (e.g. parity testing, single source of truth, performance acceptable). File follow-up
  build task to make CLI a thin MCP client.

**NO-GO if:**
- CLI and MCP already share a session-library layer (the canonical "factor below" pattern),
  in which case "use MCP from the CLI" would add latency for no architectural win. Document
  the finding in the research artifact + close.

**DEFER if:**
- Investigation reveals partial sharing — some operations share a primitive, others diverge.
  File one build task per divergence to converge them on a shared primitive (lower cost than
  switching the CLI to MCP-client).

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:**

No evidence gathered yet — this inception IS the investigation. Operator-requested probe to determine whether termlink's CLI/internal callers use a proper MCP client to reach termlink-mcp, or whether they bypass the MCP layer and call the session/agent code directly. Answer informs whether termlink-mcp is structurally validated as a public surface or only exists as a wrapper for external consumers (claude-code, etc). DEFER means: produce findings, then file separate build tasks for any structural changes a GO would warrant.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-01T08:23:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
