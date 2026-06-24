---
id: T-1904
name: "MCP client vs direct session: how does termlink connect to its own MCP server"
description: >
  Inception: MCP client vs direct session: how does termlink connect to its own MCP server

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-01T08:22:21Z
last_update: 2026-06-01T09:22:31Z
date_finished: 2026-06-01T09:22:31Z
---

# T-1904: MCP client vs direct session: how does termlink connect to its own MCP server

## Problem Statement

**The question:** When termlink's own components (CLI commands, internal callers, agents-on-the-same-host)
need to perform an operation that the MCP server (`termlink-mcp`) exposes — e.g. `channel.post`,
`hub.capabilities`, `agent.contact` — do they go through an MCP client (proper protocol round-trip
via JSON-RPC over stdio/TCP) or do they call into the session/hub library code directly?

**Why this matters (refined post-Reviewer A):**

1. **Two distinct validation gaps must be named separately:**
   - **Logic-validation gap.** If MCP and CLI re-derive the same operation independently, behavior
     diverges silently. *Cured by:* sharing a primitive (`session::*`).
   - **Transport-validation gap.** If MCP and CLI share a primitive but the CLI never speaks MCP,
     the rmcp **serialization + transport layer** is never exercised by dogfooded code — only by
     external clients. Drift in the codec/transport then surfaces only to claude-code et al.
     *Cured by:* either routing the CLI through MCP, OR a parity-test harness that calls both and
     diffs outputs.
   These two gaps pull in opposite directions and are independently load-bearing.
2. **Maintenance multiplier.** Divergence is most likely to hide in **legacy** operations
   (`inbox.push`, `file.send`, `event.broadcast`) — not the canonical new ones. The T-1166 cut
   left some operations in deprecated states with potentially shimmed handlers; those are exactly
   the surfaces where two implementations might quietly diverge.
3. **Performance.** Internal MCP-client round-trips add JSON ser/deserialize + transport overhead
   direct session calls don't pay. Accidental in-process MCP loopbacks would show up as latency
   that's hard to explain.

(Removed prior Why #4 — "T-1166 consolidation implies CLI-should-be-MCP-client" was post-hoc
rationalization; nothing in T-1166 actually requires the CLI to speak MCP. Reviewer A catch.)

**For whom:** TermLink maintainers (us, future agent sessions) and external integrators who depend
on `termlink-mcp` matching CLI behavior 1:1.

**Why now:** Post-T-1166 cut, the system has converged on a smaller surface area. This is the
right inflection point to decide whether MCP-as-internal-protocol is a goal worth pursuing or
whether the current dual-stack arrangement is intentional and stable.

## Assumptions

A1. `termlink-mcp` tool handlers `use termlink_session::*` (or `termlink_hub::*`) and dispatch into
    shared primitives — they do NOT re-implement the operation independently.

A2-merged. `termlink-cli` `cmd_*` handlers call session/hub primitives DIRECTLY, with NO MCP client
    of any kind — neither in-process rmcp loopback, nor subprocess+stdio to a spawned
    `termlink-mcp`, nor in-process JSON-RPC. (Reviewer A: A2 and A4 merged — A4 was a strict
    sub-case. Reviewer B: subprocess-loopback path must be explicitly disproven, not just inferred
    from Cargo deps. Test: grep `termlink-cli` source for `rmcp` client imports AND for
    `Command::new(... "termlink-mcp"...)` literals.)

A3. `termlink_remote_call` and other cross-host MCP tools ride **hub-rpc JSON-RPC over TCP+TLS**,
    NOT the rmcp MCP protocol. Two distinct protocols are in play:
    rmcp (claude-code ↔ termlink-mcp) and hub-rpc (everything ↔ remote-hub).

## Exploration Plan

**Methodology choice (operator-decided 2026-06-01): Option C — full census.** Instead of sampling
one or a few operations, enumerate EVERY MCP tool exposed by `termlink-mcp` and produce a matrix
of `<MCP-tool, CLI-command-equivalent, shared-primitive>` triples. The matrix is the evidence;
the GO/NO-GO/DEFER decision follows from what the matrix reveals.

Why census over sample: divergence by definition hides in cases the sample doesn't cover.
Reviewer B's critique of the single-sample plan ("`channel post` is the easiest case, hides legacy
divergence") only gets fully addressed by exhaustive coverage. The matrix also becomes a
maintenance artifact going forward — answers "for any future MCP tool, is the CLI equivalent
sharing a primitive?" without re-running the inception.

**Census procedure:**

1. **Crate-dep audit (5 min, Reviewer B item).** Inspect `Cargo.toml` for each crate. Verify
   `termlink-cli` has NO Cargo dependency on `termlink-mcp`. Then grep `termlink-cli` source for
   `rmcp::` client imports AND `Command::new` invocations targeting `termlink-mcp` — these
   together settle A2-merged at the static level.

2. **Enumerate MCP tools (10 min).** Extract the full tool list from `crates/termlink-mcp/` —
   identify the macro/registration pattern (`#[tool]`, `register_tool!`, an explicit registry, etc.),
   produce a list of `<tool_name, handler_fn, handler_file:line>` for every tool exposed. Expected
   count: ~150 per the deferred-tool list visible to claude-code today.

3. **Enumerate CLI commands (10 min).** Extract the full subcommand list from
   `crates/termlink-cli/` — likely from `cli.rs` clap definitions + the dispatch in `main.rs`.
   Produce `<cli_path, cmd_fn, fn_file:line>` for every leaf command.

4. **Join + classify (30-60 min, the bulk of the work).** For each MCP tool, find its CLI
   equivalent (naming heuristic: `mcp__termlink__foo_bar` ↔ `termlink foo bar`; document
   exceptions). For each pair, read both handler bodies and identify the shared primitive (or
   note "no shared primitive — implementations diverge"). Classify each row:
   - **SHARED** — both call the same `session::*` or `hub::*` function.
   - **DIVERGENT** — both reach the same conceptual operation via different code paths.
   - **MCP-ONLY** — no CLI equivalent exists.
   - **CLI-ONLY** — no MCP equivalent exists.
   - **CROSS-HOST** — operation rides hub-rpc (`termlink_remote_call`-class), not local
     primitive; flag for A3 validation.

5. **Distill findings (15 min).** Aggregate SHARED/DIVERGENT/MCP-ONLY/CLI-ONLY/CROSS-HOST counts.
   Identify which subsystems show divergence. Cross-reference against the legacy operation list
   (T-1166 retired set) to test the "divergence hides in legacy" hypothesis.

**Time-box:** ~90 min total. NOT a single-session task — likely 1-2 fresh sessions to execute.

**Output:** `docs/reports/T-1904-mcp-vs-direct-session.md` becomes the matrix artifact (a table
with one row per MCP tool); the task file holds the methodology and the GO/NO-GO decision.

**Out-of-scope reminder:** Fleet runtime probe (originally considered as a Spike 4) was NOT
adopted in this scope. The architectural question is code-level; runtime dogfood is a separate
inception/build task if needed later.

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
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
The census matrix produces five classification counts: SHARED / DIVERGENT / MCP-ONLY / CLI-ONLY /
CROSS-HOST. Three operator-actionable branches follow:

**GO — route the CLI through MCP** (file build task: "make CLI a thin MCP client") **if**:
- Matrix shows DIVERGENT count is high (e.g. >20% of rows) AND legacy operations dominate
  divergence, AND
- Acceptable latency budget exists for the round-trip, AND
- No simpler convergence path (shared primitive) is available for the divergent rows.

**NO-GO — leave dual-stack as-is, status quo wins** **if**:
- Matrix shows SHARED dominates (e.g. >90% of rows), the existing "factor below" pattern is
  already healthy, AND
- The remaining DIVERGENT rows are few enough to converge individually (each becomes a small
  build task to share a primitive).

**GO-PARITY — file build task: parity test harness** (Reviewer A's addition to the ladder) **if**:
- Matrix shows SHARED dominates but the **transport-validation gap** is acknowledged as real, AND
- Routing the CLI through MCP is unwarranted (latency cost), AND
- A test harness that calls both CLI and MCP and diffs outputs catches the validation gap
  cheaper than a CLI rewrite.

**DEFER — partial findings** **if**:
- Census can't complete in available time (file follow-on task with explicit subset scope), OR
- Census reveals categories the current vocabulary doesn't classify cleanly (refine taxonomy,
  re-run).

Note: "DEFER" used here is the **decision-state** DEFER, distinct from the **filing-state**
PENDING-EVIDENCE used at task creation (Reviewer A disambiguation).

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

**Recommendation:** GO-PARITY — build a parity-test harness; do NOT route CLI through MCP.

**Rationale (one paragraph):**

Census executed in one session on 2026-06-01. Headline: 251 MCP tools, 151 CLI commands, 122
naming-match pairs, 129 MCP-ONLY, 29 CLI-ONLY. The unexpected finding (not framed in the
original inception) is a two-layer split inside the naming-match group: **data-access
primitives are SHARED** (both MCP and CLI use `termlink_session::manager::find_session`,
`client::rpc_call`, `termlink_protocol::control::channel::canonical_sign_bytes`, etc.), but
**orchestration/aggregation helpers are DIVERGENT-BY-COPY** — 83 functions in
`crates/termlink-mcp/src/tools.rs` carry a `_mcp` suffix and parallel CLI helpers in
`commands/channel.rs` (8/11 sampled have verified CLI counterparts). Some whole tools like
`termlink_fleet_doctor` are reimplemented MCP-side rather than delegating to the CLI.
**A2 + A4 are conclusively disproven** (no rmcp client in CLI deps, no subprocess+stdio
loopback). **A3 confirmed** (cross-host uses `termlink_protocol::jsonrpc` over TCP+TLS).
GO is rejected because routing CLI through MCP would either force operator-UX for 40 MCP-ONLY
analytics tools or leave them as permanent asymmetry. NO-GO is rejected because SHARED is ~48%
not 90% — the 83 `_mcp` parallel helpers are a real silent-divergence hazard. GO-PARITY catches
that hazard with a bounded harness investment.

**Evidence:**

- Full census + classification + matrix-row evidence in
  `docs/reports/T-1904-mcp-vs-direct-session.md` Steps 1-5.
- Step 1 raw probes: `grep -E "termlink_mcp" crates/termlink-cli/src/` → 3 hits (all server-side
  or info), `grep -E "Command::new.*termlink" crates/termlink-cli/src/` → 0, `cargo tree -p
  termlink --edges normal | grep rmcp` → rmcp under `termlink-mcp` only.
- Step 2 raw: `grep -c "^    #\[tool(" crates/termlink-mcp/src/tools.rs` → 251.
- Step 3 raw: `grep -rEc "(async )?fn cmd_[a-z_]+\(" crates/termlink-cli/src/` → 151.
- Step 4 divergence signal: `grep -cE "fn [a-z_]+_mcp\(" crates/termlink-mcp/src/tools.rs` → 83.
- Step 4 A3 confirmation: `termlink_remote_call` body at `tools.rs:11536` uses
  `connect_remote_hub_mcp()` → `rpc_client.call()` → `termlink_protocol::jsonrpc::RpcResponse`
  (hub-rpc, not rmcp).
- Stale-assumption updates: A1 holds at Layer 1 (data-access), fails at Layer 2/3
  (orchestration helpers + whole-tool reimplementations) — promote refined finding to project
  memory.
- Suggested follow-up build tasks listed in research artifact Step 5 (parity harness v0.1;
  shared `chat-arc-helpers` crate to promote the 8 verified duplicates).

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

**Decision**: GO

**Rationale**: Census executed in one session on 2026-06-01. Headline: 251 MCP tools, 151 CLI commands, 122
naming-match pairs, 129 MCP-ONLY, 29 CLI-ONLY. The unexpected finding (not framed in the
original inception) is a two-layer split inside the naming-match group: **data-access
primitives are SHARED** (both MCP and CLI use `termlink_session::manager::find_session`,
`client::rpc_call`, `termlink_protocol::control::channel::canonical_sign_bytes`, etc.), but

**Date**: 2026-06-01T09:22:31Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-01T08:23:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-01T09:22:31Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Census executed in one session on 2026-06-01. Headline: 251 MCP tools, 151 CLI commands, 122
naming-match pairs, 129 MCP-ONLY, 29 CLI-ONLY. The unexpected finding (not framed in the
original inception) is a two-layer split inside the naming-match group: **data-access
primitives are SHARED** (both MCP and CLI use `termlink_session::manager::find_session`,
`client::rpc_call`, `termlink_protocol::control::channel::canonical_sign_bytes`, etc.), but

## Reviewer Verdict (v1.4)

- **Scan ID:** R-3ef4c50a
- **Timestamp:** 2026-06-01T09:22:31Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-06-01T09:22:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
