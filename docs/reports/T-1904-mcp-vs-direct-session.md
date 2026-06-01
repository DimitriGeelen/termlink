# T-1904 — MCP client vs direct session: how does termlink connect to its own MCP server?

**Status:** Inception, exploration phase. Scope finalized (Option C, full census); execution pending.
**Filed:** 2026-06-01
**Owner:** human (decision authority); claude-code (advisory)
**Recommendation at filing:** PENDING-EVIDENCE (filing-state; not the criteria-DEFER)
**Methodology:** Option C — full census of ALL `termlink-mcp` tools mapped to CLI equivalents and
shared primitives. Produces a maintenance-grade matrix (one row per MCP tool, ~150 rows expected).

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

## Hypotheses (filed as assumptions, post-merge)

- **A1.** `termlink-mcp` tool handlers use shared `session::*` / `hub::*` primitives.
- **A2-merged.** `termlink-cli` `cmd_*` handlers call session/hub primitives DIRECTLY, with
  NO MCP client of any kind (in-process rmcp loopback, subprocess+stdio, or in-process JSON-RPC).
  Merged A2+A4 per Reviewer A; subprocess-loopback path explicitly probed per Reviewer B.
- **A3.** Cross-host MCP tools ride hub-rpc JSON-RPC over TCP+TLS, not the rmcp protocol.

If the census shows A1+A2 hold for ≥90% of rows, the answer is "we share a library, not a
protocol — CLI does NOT go through MCP." Divergence count in remaining rows drives the decision
ladder (GO / NO-GO / GO-PARITY / DEFER).

## Findings

(populated as census executes — empty for now per inception discipline; next session's slice)

### Step 1 — Crate-dep audit + subprocess-loopback grep

_pending_

### Step 2 — Enumerate MCP tools (extract from `termlink-mcp` source)

_pending — target output: list of ~150 rows `<tool_name, handler_fn, file:line>`_

### Step 3 — Enumerate CLI commands (extract from `termlink-cli` source)

_pending — target output: list of `<cli_path, cmd_fn, file:line>`_

### Step 4 — Join + classify (the matrix)

_pending — target output: table below with one row per MCP tool, classified as
SHARED / DIVERGENT / MCP-ONLY / CLI-ONLY / CROSS-HOST_

| MCP tool | CLI equivalent | Shared primitive | Classification | Notes |
|---|---|---|---|---|
| _to populate_ | | | | |

### Step 5 — Distill findings

_pending — target output: classification counts, divergence subsystems, GO/NO-GO/GO-PARITY/DEFER
recommendation with rationale referencing matrix rows_

## Dialogue Log

### 2026-06-01T — Operator: file inception
- **Question posed:** "please start a new inception to inspect if we are using MCP client to connect to our termlink MCP server or are we calling the client session directly"
- **Agent action:** Filed T-1904, drafted Problem Statement / Assumptions / Exploration Plan / Constraints / Scope Fence / Go-No-Go. Created this artifact.

### 2026-06-01T — Operator: peer-review the inception
- **Question posed:** "ask two termlink agent to investigate / review the inception definition"
- **Agent action:** Dispatched two `fw termlink dispatch` workers in parallel — Reviewer A
  (framing angle) and Reviewer B (plan angle). Both exited 0; critiques saved to
  `/tmp/t1904-reviewer-{a,b}.md`.
- **Key findings (verbatim summary):**
  - **Reviewer A (framing):** YES-WITH-EDITS. Why #1 conflated logic-validation with
    transport-validation (independently load-bearing gaps). GO/NO-GO ladder hides a
    "parity-harness" option. A2 and A4 redundant. "DEFER" overloaded between filing-state
    and decision-state. Why #4 post-hoc.
  - **Reviewer B (plan):** YES-WITH-ADDITIONS. H4 cannot be disproven by Cargo deps
    (subprocess+stdio loopback evades it). H3 untested by any spike. `channel post`
    sample misses divergence-in-legacy by design.

### 2026-06-01T — Operator: scope decision
- **Question posed:** "should we pull data from all installations? should we broaden to all MCP commands?"
- **Agent response:** Recommended Option B (smart sample of 5 archetypes) over Option C (full
  census) for scope discipline; deferred fleet-runtime probe as separate concern.
- **Operator decision:** Option C — full census.
- **Agent action:** Restructured the Exploration Plan around Option C (~90 min, 1-2 fresh
  sessions to execute). Applied Reviewer A framing fixes (parity-harness branch in GO/NO-GO,
  merged A2+A4, demoted Why #4, disambiguated filing-DEFER → PENDING-EVIDENCE). Applied Reviewer
  B additions (subprocess-loopback grep, cross-host A3 validation in census procedure). Fleet
  runtime probe explicitly out of scope.

## Recommendation (pending evidence)

**PENDING-EVIDENCE (filing-state)** — Census not yet executed. After steps 1-5 complete, this
file's Recommendation section will be updated with one of GO / NO-GO / GO-PARITY / DEFER and a
referenced-evidence rationale citing specific matrix rows.
