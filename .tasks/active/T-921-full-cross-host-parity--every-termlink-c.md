---
id: T-921
name: "Full cross-host parity — every termlink CLI command works network-wide"
description: >
  Phase 2 of the network-everything program. T-920 wraps the existing remote family in MCP; this inception scopes what it takes to make ALL termlink CLI commands (~50) work cross-host. Questions: unified remote-target parameter on every command, or keep the 'remote' subcommand namespace and add per-command wrappers? Which commands actually need cross-host semantics vs being local-only by design? How do we route RPC through the hub vs direct-to-target? Decisions needed before building anything.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-11T19:33:05Z
last_update: 2026-04-11T20:19:55Z
date_finished: 2026-04-11T20:19:55Z
---

# T-921: Full cross-host parity — every termlink CLI command works network-wide

## Problem Statement

After T-920, MCP agents can call the `remote` family (call/ping/inject) and start a hub with TCP. But the other ~85 CLI commands still assume a local unix socket: `termlink ping`, `termlink output`, `termlink kv set`, `termlink dispatch`, `termlink file send` etc. all hard-wire `default_socket_path()`. A human or agent on host A who wants to inspect, drive, or dispatch a session on host B has two unattractive choices today: (1) SSH to host B and run locally, or (2) manually build a `termlink remote call --method ... --params ...` envelope per operation. Option 2 works but pushes all ergonomics onto the caller. This inception decides **how** to make cross-host a first-class property of the CLI surface, not a separate namespace users must translate into.

**For whom:** dispatch operators driving multi-host agent fleets (T-163 use case), MCP-backed agents needing to drive remote sessions without translating each call into `remote_call`, and humans running `termlink status` against a remote session with zero mental overhead.

**Why now:** T-920 just shipped the primitives (direct-to-hub TCP, `connect_remote_hub`, TOFU TLS, HMAC scope auth). Adding a unified routing layer while the pattern is still one file is much cheaper than after we ship 6 more commands that re-entrench the unix-only assumption. Also T-922 (codify auto-exposure) depends on this decision — we cannot write a "must be MCP-reachable and cross-host" lint until we know what "cross-host" means mechanically.

## Assumptions

- **A1:** Most session-scoped commands (>70%) are naturally cross-host; they just need the RPC to land on the remote hub instead of the local one. Verify by taxonomising the CLI census.
- **A2:** A small set of commands are structurally local-only: `hub start/stop`, `vendor`, `mcp`, `completions`, `version`, `doctor`. These should NOT grow a `--target` flag — it would be dishonest.
- **A3:** The streaming commands (`output --follow`, `stream`, `mirror`, `attach`) have cross-host semantics but harder transport requirements. They might need per-command treatment even in an "option A everywhere" world.
- **A4:** The existing `connect_remote_hub` (TOFU TLS + HMAC scope + `rpc_client.call`) is sufficient transport. T-921 is about where the flag lives and how the CLI dispatches, not new auth/routing primitives.
- **A5:** A `--target HOST:PORT` flag (with implicit secret lookup from `~/.termlink/secrets/`) is what a human would intuitively reach for. Verify by writing the flag out for 5 commands and checking if any look wrong.

## Exploration Plan

Time-box: **one session**, dialogue-driven. No production code. Deliverable = decision + build task list.

- **Spike 1 (~30 min) — CLI census.** Read `cli.rs`, categorise every command into: `local-only` (structural), `session-scoped` (natural cross-host), `session-scoped streaming` (cross-host but needs care), `batch/orchestration` (dispatch, discover, broadcast). Record in research artifact.
- **Spike 2 (~30 min) — Option analysis.** Four options: (A) unified `--target` on every session-scoped command; (B) expand `remote` subcommand with per-command wrappers; (C) generic `remote call` only, MCP-only ergonomics; (D) hybrid — `--target` on simple commands + `remote` namespace for exotic cases. Compare on: ergonomics, code duplication, backward compat, MCP parity, auth.
- **Spike 3 (~20 min) — Routing architecture.** Direct-to-hub (T-920 pattern) vs mesh-routing via local hub vs session-registrar lookup. Decide on: what already works, failure modes, latency.
- **Spike 4 (~20 min) — Decomposition.** Given the chosen option, list concrete build tasks (one per deliverable per CLAUDE.md sizing). Estimate rough sizing. Becomes the input to human GO/NO-GO.

**Dialogue checkpoints:** pause after Spike 2 for human input on option preference. Do not execute Spike 4 unilaterally.

## Technical Constraints

- **Transport:** TCP + TOFU TLS already working (T-182, T-920). No new transport needed.
- **Auth:** HMAC secret + scope (`read` / `event` / `command` / `admin`) already enforced at hub level. Cross-host commands must pick the right scope; no implicit admin.
- **Streaming:** `output --follow`, `stream`, `mirror`, `attach` pipe PTY bytes. Hub RPC is request/response; streaming over TCP hub RPC has never been exercised in production. Constrains what "cross-host" means for streaming commands — possibly deferrable.
- **Secret distribution:** No key exchange exists; secrets are hand-copied today. T-921 does NOT solve this — out of scope — but must not pretend it does not exist.
- **Dispatch:** `termlink dispatch` spawns local processes. Cross-host dispatch = "spawn on remote host" via `session.spawn` RPC on target hub. Conceptually simple but worth verifying.
- **Single session expectation:** if Spike 2 turns into genuine disagreement, pause and file sub-inceptions rather than forcing a decision.

## Scope Fence

**IN scope:**
- Decide how cross-host targeting is expressed on the CLI (flag shape, discovery, help text)
- Decide which CLI commands become cross-host and which stay local-only
- Decide the routing architecture (direct-to-hub vs alternatives)
- Produce a decomposed build task list with per-task sizing
- Record the decision so T-922 can write auto-exposure rules against it

**OUT of scope:**
- Implementing any cross-host wrappers (that is the build tasks T-921 spawns)
- Secret distribution / key exchange (cite it, do not solve)
- Streaming over TCP hub RPC (may defer to a follow-up task if hard)
- Discovering remote hubs by name / service catalogue (T-163 territory)
- Multi-hop routing (agent on A → hub on B → session on C). Direct-to-hub only.

## Acceptance Criteria

### Agent
- [ ] CLI command census complete — every command in cli.rs categorised in the research artifact
- [ ] Four options (A/B/C/D) compared on ergonomics, code duplication, MCP parity, auth, backward compat
- [ ] Routing architecture chosen with rationale (direct-to-hub vs alternatives)
- [ ] Decomposed build task list produced, each line sized to fit one session
- [ ] Recommendation filled with GO/NO-GO/DEFER + rationale + evidence

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Census shows at least 20 session-scoped commands that would genuinely benefit from cross-host (not just "could theoretically")
- A single option scores best on ≥3 of the 5 comparison axes (ergonomics, duplication, MCP parity, auth, backward compat)
- Routing architecture reuses existing `connect_remote_hub` primitives with ≤50 lines of new shared code
- Decomposition produces build tasks each sized under one session

**NO-GO / DEFER if:**
- Census reveals <10 session-scoped commands actually need cross-host — over-engineering signal, just keep `remote call`
- No single option dominates — disagreement means we do not yet understand the problem
- Streaming + dispatch + file transfer all turn out to need bespoke per-command logic — "full parity" framing is wrong, reframe as "targeted parity"
- Secret distribution gap blocks meaningful real-world use — defer until T-163 / T-182 fleet bootstrapping is designed

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

## Decisions

**Decision**: GO

**Rationale**: Option A (unified --target flag) + Option C (remote call) as escape hatch. Human confirmed 'where good go' after reviewing strawman/steelman and weighted directive scores in docs/reports/T-921-cross-host-parity.md.

**Date**: 2026-04-11T20:19:55Z
## Decision

**Decision**: GO

**Rationale**: Option A (unified --target flag) + Option C (remote call) as escape hatch. Human confirmed 'where good go' after reviewing strawman/steelman and weighted directive scores in docs/reports/T-921-cross-host-parity.md.

**Date**: 2026-04-11T20:19:55Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-11T19:49:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-11T20:19:55Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Option A (unified --target flag) + Option C (remote call) as escape hatch. Human confirmed 'where good go' after reviewing strawman/steelman and weighted directive scores in docs/reports/T-921-cross-host-parity.md.

### 2026-04-11T20:19:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
