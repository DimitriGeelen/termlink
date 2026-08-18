---
id: T-870
name: "Add termlink_batch_ping MCP tool — lightweight fleet health check"
description: >
  Add termlink_batch_ping MCP tool — lightweight fleet health check

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T21:47:57Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T21:53:28Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-870: Add termlink_batch_ping MCP tool — lightweight fleet health check

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `BatchPingParams` struct with tag/role/name filters and timeout
- [x] `termlink_batch_ping` tool pings all matching sessions concurrently
- [x] Returns `{ok, results: [{session, display_name, alive, latency_ms, age}...], total, alive, dead}`
- [x] Unit tests for BatchPingParams (full + empty)
- [x] MCP integration tests: batch ping no matches + live sessions
- [x] 44 MCP tools total
- [x] Zero clippy warnings

### Human
<!-- No human ACs.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-04-04T21:47:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-870-add-termlinkbatchping-mcp-tool--lightwei.md
- **Context:** Initial task creation

### 2026-04-04T21:53:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
