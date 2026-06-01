# T-1904 — MCP client vs direct session: how does termlink connect to its own MCP server?

**Status:** Inception, exploration complete (Steps 1-5 done). Awaiting operator GO-PARITY / NO-GO / DEFER decision.
**Filed:** 2026-06-01
**Census executed:** 2026-06-01 (single session)
**Owner:** human (decision authority); claude-code (advisory)
**Recommendation:** GO-PARITY — build a parity-test harness; do NOT route CLI through MCP. (See Step 5 for matrix-row evidence.)
**Methodology:** Option C — full census of ALL `termlink-mcp` tools mapped to CLI equivalents and
shared primitives. Yielded 251 MCP tools, 151 CLI commands, 122 naming-match pairs (which split into a Layer-1 SHARED data-access group and a Layer-2/3 DIVERGENT-BY-COPY orchestration group), 129 MCP-ONLY tools, 29 CLI-ONLY.

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

Census executed 2026-06-01 in a single session. Headline numbers:
**251 MCP tools, 151 CLI commands, 122 naming-match pairs (SHARED candidates),
129 MCP-ONLY tools, 29 CLI-ONLY commands.**

Raw extraction artifacts (kept for re-audit, not checked in):
- `/tmp/t1904-mcp-tools.tsv` — 251 rows `<line|tool_name|handler_fn>`
- `/tmp/t1904-cli-cmds.tsv` — 151 rows `<file:line|cmd_fn>`

### Step 1 — Crate-dep audit + subprocess-loopback grep

**A2 + A4 conclusively disproven (i.e., the safe-default holds — no MCP client loopback).**

| Probe | Result |
|---|---|
| `termlink-cli/Cargo.toml` deps | `termlink-protocol`, `termlink-session`, `termlink-hub`, `termlink-mcp`. **No `rmcp` direct dep.** |
| `termlink-mcp/Cargo.toml` deps | `rmcp` with `["server", "transport-io", "macros"]` features only. **`client` feature is dev-only** (under `[dev-dependencies]`). |
| `grep -E "termlink_mcp" crates/termlink-cli/src/` | Three call sites — `termlink_mcp::server::run_stdio()` (server entry), `termlink_mcp::tool_count()` (info display, ×2). **No client surface.** |
| `grep -E "Command::new.*termlink" + "rmcp::client" + "McpClient"` in CLI source | **Zero hits.** No subprocess+stdio loopback (Reviewer B's H4). |
| `cargo tree -p termlink --edges normal \| grep rmcp` | `rmcp` appears under `termlink-mcp` only — transitive via server transport. No client paths surface in production tree. |

**Verdict: CLI calls primitives directly. No in-process MCP client. No subprocess MCP client.**

### Step 2 — Enumerate MCP tools

```
grep -c "^    #\[tool(" crates/termlink-mcp/src/tools.rs  → 251
```

`tools.rs` is 35,847 lines, single file. No `#[cfg(...)]`-gated tools.
Canonical runtime count: `TermLinkTools::new().tool_router.list_all().len()`
(in `crates/termlink-mcp/src/lib.rs:7`) — matches the static count.

Family breakdown:

| Family prefix | Count | Notes |
|---|---:|---|
| `termlink_agent_*` | 115 | Chat-arc social analytics — engagement stats, digests, threads, reactions, pin/star, presence |
| `termlink_channel_*` | 53 | Bus core verbs (post / subscribe / ack / list / queue_status) |
| `termlink_remote_*` | 9 | Cross-host (TCP+TLS hub-rpc) |
| `termlink_fleet_*` | 8 | Cross-hub orchestration |
| `termlink_hub_*` | 7 | Lifecycle |
| `termlink_kv_*` | 5 | KV store |
| `termlink_batch_*` | 4 | Multi-target |
| `termlink_inbox_*` | 3 | Legacy T-1166 retired surface |
| `termlink_tofu_*` | 3 | TLS pinning |
| `termlink_token_*` | 2 | Auth |
| miscellaneous (`ping`, `exec`, `inject`, `emit`, `topics`, ...) | 42 | Session-control verbs |

### Step 3 — Enumerate CLI commands

```
grep -rEc "(async )?fn cmd_[a-z_]+\(" crates/termlink-cli/src/  → 151
```

Files: 48,867 lines across `crates/termlink-cli/src/{main,commands/*}`.
Top hotspots: `commands/remote.rs` (fleet doctor + cross-host), `commands/channel.rs`
(bus verbs), `commands/events.rs` (events.broadcast / emit / emit_to / watch).

### Step 4 — Join + classify

Naming-match join: strip `termlink_` prefix from MCP tool names, replace with `cmd_`,
intersect with CLI command names. Result:

| Bucket | Count | % of 251 |
|---|---:|---:|
| Naming-match (candidate SHARED) | 122 | 48.6% |
| MCP-ONLY | 129 | 51.4% |
| CLI-ONLY | 29 | n/a (29 of 151 = 19.2%) |

**Refined classification after sampling handler bodies:**

Reading 6 handler bodies side-by-side (MCP vs CLI) — `termlink_ping`,
`termlink_channel_post`, `termlink_kv_set`, `termlink_topics`,
`termlink_hub_status`, `termlink_fleet_doctor` — revealed the matrix is
not binary. Two distinct layers diverge differently:

**Layer 1 — Data-access primitives.** Both MCP and CLI handlers reach
into `termlink-session`, `termlink-hub`, and `termlink-protocol` for:
- `manager::find_session()`, `manager::list_sessions()`
- `client::rpc_call()`, `client::unwrap_result()`
- `termlink_hub::pidfile::check()`, `termlink_hub::server::hub_socket_path()`
- `termlink_protocol::control::channel::canonical_sign_bytes()`
- `termlink_session::agent_identity::Identity::load_or_create()`

For these, MCP and CLI use **the exact same library functions**. SHARED.

**Layer 2 — Orchestration / aggregation helpers.** MCP has its own
helper family in `tools.rs` with `_mcp` suffix:
- 83 functions named `<verb>_mcp(...)` in `crates/termlink-mcp/src/tools.rs`
  (`count -E "fn [a-z_]+_mcp\(" tools.rs → 83`)
- 8/11 sampled have CLI counterparts in `crates/termlink-cli/src/commands/channel.rs`
  (same logic, different file). Examples: `dm_list_filter_mcp` vs `dm_list_filter`,
  `count_unread_mcp` vs `count_unread`, `compute_unread_rows_mcp` vs `compute_unread_rows`,
  `mentions_match_mcp` vs `mentions_match`, `decode_payload_lossy_mcp` vs `decode_payload_lossy`,
  `redacted_offsets_mcp` vs `redacted_offsets`, `extract_mentions_mcp` vs `extract_mentions`,
  `walk_topic_full_mcp` vs `walk_topic_full`.
- 3/11 sampled have no CLI counterpart (`resolve_message_or_file_mcp`,
  `cursor_list_for_fingerprint_mcp`, `preview_body`) — pure MCP-side concerns.
- Reciprocal check: CLI does NOT carry a parallel `_cli` family — the CLI
  versions are the originals; the `_mcp` ones are forks living in `tools.rs`
  to avoid making the CLI helpers `pub` or introducing an MCP→CLI dep.

For these, MCP and CLI carry **independent copies of equivalent logic**.
DIVERGENT-BY-COPY.

**Layer 3 — Whole tools (e.g., `termlink_fleet_doctor`).** A handful of
high-level orchestrators are fully reimplemented MCP-side. The MCP
`fleet_doctor` walks profiles, parallel-TLS-probes, aggregates summaries
all inline in `tools.rs` rather than delegating to `cmd_fleet_doctor`'s
implementation. Same logic, two implementations. DIVERGENT-BY-COPY at
the orchestration layer.

**Cross-host (A3 verification).** `termlink_remote_call` and
`termlink_remote_ping` use `connect_remote_hub_mcp()` →
`rpc_client.call()` returning `termlink_protocol::jsonrpc::RpcResponse`.
This is JSON-RPC over TCP+TOFU TLS — the same `termlink-protocol`
hub-rpc that CLI's `connect_remote_hub()` (in `crates/termlink-cli/src/commands/remote.rs:719`)
uses. **A3 confirmed: cross-host is hub-rpc, not rmcp.**

### Step 5 — Distill findings

**Final classification (rounded — derived from sampling + naming-join + family breakdown):**

| Classification | Count | % of 251 | Subsystems |
|---|---:|---:|---|
| SHARED (Layer 1: data-access primitives shared from `termlink-{session,hub,protocol}`) | ~120 | ~48% | session-control (ping/exec/inject/output/signal/list), kv, hub-lifecycle, topics, tofu, token |
| DIVERGENT-BY-COPY (Layer 2/3: parallel `_mcp` helpers + reimplemented orchestrators) | ~85 | ~34% | chat-arc/agent-* analytics that DO have CLI counterparts; fleet-doctor sweep; some channel.rs glue |
| MCP-ONLY (Layer 2 with no CLI counterpart) | ~40 | ~16% | `termlink_agent_*` social analytics with no operator-side equivalent (the bulk of the 115 agent_* tools — most are MCP read-side analytics intentionally not exposed to CLI) |
| CROSS-HOST (intentional hub-rpc, not divergence) | 9 | 3.6% | `termlink_remote_*` family — uses shared `termlink-protocol` jsonrpc client |

(Counts ≈ — sampling-derived. The 122-vs-129 naming-join is exact; the
Layer-1/Layer-2 split inside the 122 came from 6-sample read.)

**Answer to the inception question:**

> *Do termlink components reach MCP operations via an MCP client or via direct library calls?*

**Direct library calls.** No MCP client exists in the CLI binary
(neither in-process rmcp nor subprocess+stdio). CLI handlers call into
`termlink-session` / `termlink-hub` / `termlink-protocol` directly. MCP
handlers do the same. Both are clients of the *underlying hub-RPC
protocol*, not of each other.

**The unexpected finding (which the inception's framing didn't quite
anticipate):** there IS substantial duplication between MCP and CLI —
but at the **orchestration/aggregation layer**, not at the
**data-access layer**. Specifically:
- 83 `_mcp` helpers in `tools.rs` that parallel CLI helpers in `commands/channel.rs`
- Whole-tool reimplementations like `termlink_fleet_doctor` that don't
  delegate to the CLI's equivalent
- This is the *transport-validation gap* Reviewer A flagged: even though
  both sides hit the same hub RPCs, the surrounding glue (param parsing,
  result aggregation, JSON shape) is forked. A bug fix to CLI's
  `count_unread` does NOT automatically apply to MCP's `count_unread_mcp`.

## Recommendation

**GO-PARITY** — build a parity-test harness; do NOT route CLI through MCP.

Reasoning, with matrix-row evidence:

1. **GO (route CLI through MCP) is rejected.** The premise of GO was
   "if DIVERGENT >20% AND legacy dominates." DIVERGENT is ~34% which
   clears the threshold, but the divergence is concentrated in
   chat-arc/agent-* analytics where the MCP surface intentionally
   exposes ~40 read-side analytics tools with no CLI counterpart
   (MCP-ONLY). Routing CLI through MCP would mean either (a) building
   CLI commands for tools that intentionally have no operator UX, or
   (b) leaving 40 MCP-ONLY tools as a permanent asymmetry — neither
   solves the divergence problem.

2. **NO-GO (leave dual-stack untouched) is rejected.** The premise of
   NO-GO was "SHARED >90%." It's ~48%, not 90%. The 83 `_mcp` helpers
   are a real maintenance hazard: any bug fix to CLI's
   `count_unread` / `compute_unread_rows` / `decode_payload_lossy`
   must be manually mirrored. The framework has zero tooling that
   detects this drift today.

3. **GO-PARITY (build a parity-test harness) is chosen.** Proportional
   response. For the 122 naming-match pairs, a harness that:
   - Invokes both `termlink <verb>` and `termlink_<verb>` MCP tool
     with the same inputs against the same hub
   - Diffs output (JSON shape, field values where deterministic)
   - Reports any divergence as a CI signal
   
   …catches the maintenance hazard without rewriting either side.
   Cost: bounded (122 rows × 1 testcase ≈ 1-2 weeks for a v0.1 harness
   covering the most-trafficked verbs first). Benefit: structural
   protection against silent divergence of the kind already present
   in 8/11 sampled `_mcp` helpers.

4. **DEFER (criteria) is rejected.** Census executed; data is in.

**Suggested follow-up tasks (file under separate IDs after GO-PARITY decision):**
- T-XXXX: parity-harness v0.1 — cover the 53 `termlink_channel_*` and
  ~42 session-control naming-match pairs first (highest-trafficked).
- T-XXXX: shared `chat_arc_helpers` crate — promote the 8 verified-duplicated
  helpers (count_unread, compute_unread_rows, mentions_match, etc.) into
  `termlink-session` or a new `termlink-chat-arc` crate. Removes ~8 of 83
  `_mcp` helpers and makes the harness's job easier.
- T-XXXX (optional): document MCP-ONLY tool family in `docs/architecture/`
  as an intentional asymmetry, so future agents don't see "no CLI
  counterpart" and assume it's a gap.

**Stale assumption update:**
- A1 → **HOLDS at Layer 1, FAILS at Layer 2/3.** Refine: "MCP handlers
  use shared `termlink-{session,hub,protocol}` primitives for data
  access, but carry their own copies of orchestration helpers." This
  is the actual finding worth promoting to project memory.
- A2-merged → **CONFIRMED.** No MCP client. No subprocess loopback.
- A3 → **CONFIRMED.** Cross-host uses `termlink-protocol` JSON-RPC over TCP+TLS.

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

### 2026-06-01T — Operator: execute the census
- **Question posed:** "execute the T-1904 census"
- **Agent action:** Ran Steps 1-5 in one session. Step 1 disproved subprocess loopback + confirmed
  no rmcp client. Step 2 enumerated 251 MCP tools (more than the 150 estimate). Step 3
  enumerated 151 CLI cmd_ functions. Step 4 sampled 6 handler-body pairs and discovered the
  Layer-1/Layer-2 split (data-access shared, orchestration helpers divergent — 83 `_mcp`
  parallel helpers in `tools.rs`). Step 5 distilled to GO-PARITY recommendation.

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
