---
id: T-267
name: "MCP resources — session list + session detail as read-only data"
description: >
  MCP resources — session list + session detail as read-only data

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [mcp]
components: []
related_tasks: []
created: 2026-03-24T20:57:05Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-03-24T21:32:45Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:55Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-267: MCP resources — session list + session detail as read-only data

## Context

Extends T-264/T-265 MCP server with read-only resources (MCP protocol feature). Sessions are exposed as `termlink://` URIs for AI agent context.

## Acceptance Criteria

### Agent
- [x] `termlink://sessions` resource — JSON list of all active sessions
- [x] `termlink://sessions/{id}` resource — live session detail via RPC
- [x] Resource template for `{session_id}` URI pattern
- [x] Graceful fallback when session is unreachable (returns registration data)
- [x] Resources enabled in ServerCapabilities
- [x] 6 resource integration tests pass (20 total)
- [x] `cargo check -p termlink-mcp` compiles clean

## Verification

/Users/dimidev32/.cargo/bin/cargo check -p termlink-mcp
grep -q "enable_resources" crates/termlink-mcp/src/server.rs
grep -q "termlink://sessions" crates/termlink-mcp/src/server.rs
/Users/dimidev32/.cargo/bin/cargo test -p termlink-mcp --test mcp_integration -- --test-threads=1 2>&1 | grep -q "20 passed"

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

### 2026-03-24T20:57:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-267-mcp-resources--session-list--session-det.md
- **Context:** Initial task creation

### 2026-03-24T21:32:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
