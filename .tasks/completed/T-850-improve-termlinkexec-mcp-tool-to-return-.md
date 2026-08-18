---
id: T-850
name: "Improve termlink_exec MCP tool to return structured JSON response"
description: >
  Improve termlink_exec MCP tool to return structured JSON response

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T15:24:05Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T15:26:45Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-850: Improve termlink_exec MCP tool to return structured JSON response

## Context

termlink_exec currently returns plain text (stdout with stderr/exit_code appended). Unlike most other MCP tools that return structured JSON (ok, data fields), this makes it harder for AI agents to parse results programmatically. Change to return JSON with ok, exit_code, stdout, stderr fields.

## Acceptance Criteria

### Agent
- [x] termlink_exec returns JSON: {"ok": true/false, "exit_code": N, "stdout": "...", "stderr": "...", "target": "..."}
- [x] ok=true when exit_code is 0, ok=false otherwise
- [x] Error responses remain as "Error: ..." strings (connection failures, not found)
- [x] Integration test updated to verify JSON response structure (exit_code, stdout, target, ok)
- [x] All tests pass: cargo test -p termlink-mcp (109 tests)
- [x] Zero clippy warnings: cargo clippy -p termlink-mcp

## Verification

cargo test -p termlink-mcp 2>&1 | tail -3
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | tail -3

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

### 2026-04-04T15:24:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-850-improve-termlinkexec-mcp-tool-to-return-.md
- **Context:** Initial task creation

### 2026-04-04T15:26:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
