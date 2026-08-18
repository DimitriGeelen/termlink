---
id: T-920
name: "Expose termlink remote CLI family + hub TCP as MCP tools"
description: >
  Expose termlink remote CLI family + hub TCP as MCP tools

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-11T19:32:03Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-11T19:40:00Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:14Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 3
      D4: 4
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=4 (body:cross-machine); F-RECALL=0 
      (no-signal); F-ORCH=0 (body:wrap-phrase-without-substrate)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-920: Expose termlink remote CLI family + hub TCP as MCP tools

## Context

Discovered 2026-04-11 via RCA of a different session: an MCP agent tried to
"connect over the network" to another host, called `termlink_doctor` and
`termlink_info`, saw a Unix-socket hub and no remote-family tools, and
concluded termlink was local-only. The CLI has a complete cross-machine family
(`termlink remote ping/list/status/inject/send-file/events/exec`, hub
`--tcp <addr>`, TOFU TLS, HMAC auth — all shipped under T-163/T-164/T-182/T-186)
but none of it is exposed through MCP. The agent then proposed SSH tunnels and
resurrecting a parallel TCP hub from memory. Pure discoverability failure.

Scope of this task: Phase 1 unblock. Wrap the existing `commands/remote.rs`
functions as MCP tools and add a `tcp_addr` parameter to `termlink_hub_start`.
No new CLI features, no new protocol work — just MCP surface parity with what
already exists. Phase 2 (full cross-host parity for all CLI commands) is a
separate inception task.

Related: T-163, T-164, T-182, T-186, T-919 (remote.rs tests).

## Acceptance Criteria

### Agent
- [x] `termlink_hub_start` MCP tool accepts an optional `tcp_addr` parameter and starts the hub with TCP binding when set (calls `termlink_hub::server::run_with_tcp` instead of `run` when provided)
- [x] `termlink_remote_call` MCP tool exists — generic wrapper over `connect_remote_hub` + `rpc_client.call`, accepts `hub`, `method`, `params`, `secret_file?`, `secret?`, `scope?`, `timeout?`; returns the full RPC result as JSON. This single tool exposes every hub RPC method (session.discover, termlink.ping, command.inject, hub.auth, event.broadcast, etc.) over the network.
- [x] `termlink_remote_ping` MCP tool — convenience wrapper for the common "is this hub/session alive?" flow
- [x] `termlink_remote_inject` MCP tool — convenience wrapper for the high-value "inject text into remote session" flow
- [x] `termlink_doctor` reports hub transport (Unix-only vs TCP+Unix) so an MCP agent can tell whether cross-host is available
- [x] `termlink_overview` / `termlink_help` lists a new "remote" group so agents can discover the new tools
- [x] Param deserialization tests for every new MCP tool (same pattern as `dispatch_params_*` tests in tools.rs)
- [x] `cargo build --workspace` clean
- [x] `cargo test -p termlink-mcp` passes

## Decisions

### 2026-04-11 — Generic `remote_call` over per-command wrappers

- **Chose:** One generic `termlink_remote_call` tool + 2 convenience wrappers (`termlink_remote_ping`, `termlink_remote_inject`)
- **Why:** Every termlink feature is already reachable as a hub RPC method. A generic wrapper exposes ALL of them in one code path; adding a new hub method automatically makes it MCP-reachable cross-host. Matches the `connect_remote_hub` pattern already proven in `commands/remote.rs`. Minimizes duplication, maximises surface area.
- **Rejected:** 8 individual per-command tools (list, status, send-file, exec, profile-add/list/remove). Higher code volume for strictly less coverage — the convenience tools pay off only for the most common flows, the generic tool pays off for everything else. Can still add per-command tools later if specific flows warrant it.

## Verification

cargo build --workspace --quiet
cargo test -p termlink-mcp --lib 2>&1 | tail -3

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-11T19:32:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-920-expose-termlink-remote-cli-family--hub-t.md
- **Context:** Initial task creation

### 2026-04-11T19:40:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
