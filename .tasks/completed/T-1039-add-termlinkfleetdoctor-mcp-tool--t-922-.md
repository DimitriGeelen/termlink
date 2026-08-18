---
id: T-1039
name: "Add termlink_fleet_doctor MCP tool — T-922 codification"
description: >
  Add termlink_fleet_doctor MCP tool — T-922 codification

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-13T19:10:23Z
last_update: '2026-08-18T18:58:42Z'
date_finished: 2026-04-13T19:13:45Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:43Z'
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
  - ts: '2026-08-18T18:58:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 4
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1039: Add termlink_fleet_doctor MCP tool — T-922 codification

## Context

T-922 codification: `termlink fleet doctor` has no MCP exposure. Add `termlink_fleet_doctor` MCP tool that checks all hubs and returns JSON results with diagnostics from T-1034.

## Acceptance Criteria

### Agent
- [x] `termlink_fleet_doctor` MCP tool health-checks all configured hubs
- [x] Returns JSON with per-hub status, latency, diagnostics
- [x] Unit test for params parsing (2 tests)
- [x] Builds with zero clippy warnings

## Verification

cargo build -p termlink 2>&1 | grep -q "Finished"
cargo clippy -p termlink-mcp -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"
cargo test -p termlink-mcp fleet 2>&1 | grep "passed"

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

### 2026-04-13T19:10:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1039-add-termlinkfleetdoctor-mcp-tool--t-922-.md
- **Context:** Initial task creation

### 2026-04-13T19:13:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
