---
id: T-895
name: "Standardize remaining MCP tool outputs to structured JSON — signal, emit, emit_to,
  clean, tag"
description: >
  Standardize remaining MCP tool outputs to structured JSON — signal, emit, emit_to,
  clean, tag

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-05T08:16:36Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-05T08:21:24Z
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
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-895: Standardize remaining MCP tool outputs to structured JSON — signal, emit, emit_to, clean, tag

## Context

Continuation of T-894. Several MCP tools still return plain text instead of structured JSON: signal, emit, emit_to, broadcast, clean, tag, request.

## Acceptance Criteria

### Agent
- [x] termlink_signal returns JSON with ok, signal, pid fields
- [x] termlink_emit returns JSON with ok, topic, seq fields
- [x] termlink_emit_to returns JSON with ok, target, topic, seq fields
- [x] termlink_broadcast returns JSON with ok, topic, targeted, succeeded, failed fields
- [x] termlink_tag returns JSON with ok, target, tags, roles fields
- [x] Integration tests updated for new output format (emit, tag_add, tag_set_remove)
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

### 2026-04-05T08:16:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-895-standardize-remaining-mcp-tool-outputs-t.md
- **Context:** Initial task creation

### 2026-04-05T08:21:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
