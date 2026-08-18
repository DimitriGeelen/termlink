---
id: T-266
name: "MCP server integration tests — rmcp client + TermLink sessions"
description: >
  MCP server integration tests — rmcp client + TermLink sessions

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: [mcp, testing]
components: []
related_tasks: []
created: 2026-03-24T17:56:33Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-03-24T18:13:51Z
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
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-266: MCP server integration tests — rmcp client + TermLink sessions

## Context

Programmatic integration tests for the MCP server (T-264, T-265). Uses rmcp client connected in-process via tokio::io::duplex, with real TermLink sessions from test-utils as fixtures.

## Acceptance Criteria

### Agent
- [x] Integration test file at `crates/termlink-mcp/tests/mcp_integration.rs`
- [x] In-process MCP client via `tokio::io::duplex` + rmcp client feature
- [x] Tests use real TermLink sessions via `start_session()` fixture
- [x] ENV_LOCK serialization for TERMLINK_RUNTIME_DIR
- [x] 14 tests: list_tools, list_sessions (empty + populated), ping (ok + error), status, discover (role + name + all), kv (set/get/list/del), emit + poll, wait (ok + timeout), run (output + exit code), schema validation
- [x] All 14 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-mcp --test mcp_integration -- --test-threads=1 2>&1 | grep -q "14 passed"

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

### 2026-03-24T17:56:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-266-mcp-server-integration-tests--rmcp-clien.md
- **Context:** Initial task creation

### 2026-03-24T18:13:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
