---
id: T-824
name: "Add termlink_collect MCP tool — multi-session event fan-in via hub"
description: >
  Add termlink_collect MCP tool — multi-session event fan-in via hub

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:10:08Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-04-03T20:13:59Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 1
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=1 (body:log-or-error-line); D3=0 
      (no-signal); D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); 
      F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-824: Add termlink_collect MCP tool — multi-session event fan-in via hub

## Context

Add `termlink_collect` MCP tool — fan-in events from multiple sessions via hub. Single-shot collect with timeout for MCP compatibility. Requires hub running. This is the 31st MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_collect` MCP tool implemented — calls hub `event.collect` with targets, topic, timeout_ms
- [x] Params: targets (optional list), topic (optional filter), timeout_ms (optional, default 5000), since (optional cursors)
- [x] Returns: collected events array with session_name, topic, payload, seq, timestamp + cursors for continuation
- [x] Returns error message when hub is not running
- [x] Integration test: collect with no hub returns error
- [x] `cargo test -p termlink-mcp` passes (52 tests)
- [x] `cargo clippy --workspace` clean (0 warnings)

## Verification

cargo test -p termlink-mcp
cargo clippy --workspace
grep -q 'termlink_collect' crates/termlink-mcp/src/tools.rs

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

### 2026-04-03T20:10:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-824-add-termlinkcollect-mcp-tool--multi-sess.md
- **Context:** Initial task creation

### 2026-04-03T20:13:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
