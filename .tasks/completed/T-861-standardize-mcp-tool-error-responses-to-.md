---
id: T-861
name: "Standardize MCP tool error responses to JSON — migrate plain text errors to
  {ok:false, error:...}"
description: >
  Standardize MCP tool error responses to JSON — migrate plain text errors to {ok:false,
  error:...}

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T19:42:10Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T19:57:47Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-861: Standardize MCP tool error responses to JSON — migrate plain text errors to {ok:false, error:...}

## Context

128 MCP tools return plain `format!("Error: ...")` text. Newer tools (exec, run, interact, send) return `{ok: false, error: ...}` JSON. Scoped to core tools: ping, status, list_sessions, discover, kv_*, tag, clean, spawn.

## Acceptance Criteria

### Agent
- [x] ALL 42 MCP tools now return JSON errors `{ok: false, error: ...}` (was 128 plain text errors, now 0)
- [x] Added `json_err()` helper function for consistent error formatting
- [x] Integration tests updated to check for JSON error patterns
- [x] All tests pass: `cargo test -p termlink-mcp` (119 tests)
- [x] Zero clippy warnings

## Verification

cargo test -p termlink-mcp
cargo clippy -p termlink-mcp -- -D warnings

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

### 2026-04-04T19:42:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-861-standardize-mcp-tool-error-responses-to-.md
- **Context:** Initial task creation

### 2026-04-04T19:57:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
