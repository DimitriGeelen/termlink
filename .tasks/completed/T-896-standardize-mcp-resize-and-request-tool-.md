---
id: T-896
name: "Standardize MCP resize and request tool outputs to structured JSON"
description: >
  Standardize MCP resize and request tool outputs to structured JSON

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:23:28Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-05T08:29:55Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
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
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-896: Standardize MCP resize and request tool outputs to structured JSON

## Context

Final batch of MCP tools that still return plain text format strings instead of structured JSON.

## Acceptance Criteria

### Agent
- [x] termlink_resize returns JSON with ok, cols, rows
- [x] termlink_request returns JSON with ok, request_id, reply_topic, response fields
- [x] termlink_wait returns JSON with ok, topic, event fields (success) or ok:false, error (timeout)
- [x] Integration tests updated (wait_timeout, request_timeout, wait_receives_event)
- [x] All tests pass, zero clippy warnings

## Verification

cargo test --workspace
cargo clippy --workspace --all-targets

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

### 2026-04-05T08:23:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-896-standardize-mcp-resize-and-request-tool-.md
- **Context:** Initial task creation

### 2026-04-05T08:29:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
