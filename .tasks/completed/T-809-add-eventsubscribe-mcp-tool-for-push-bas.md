---
id: T-809
name: "Add event_subscribe MCP tool for push-based event delivery"
description: >
  Add event_subscribe MCP tool for push-based event delivery

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T17:55:36Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T17:59:35Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-809: Add event_subscribe MCP tool for push-based event delivery

## Context

MCP server has `event_poll` but no `event_subscribe` tool. Adding it gives MCP clients (Claude Code, etc.) access to push-based event delivery with optional `since` for cursor-based replay (T-805).

## Acceptance Criteria

### Agent
- [x] `EventSubscribeParams` struct defined with target, timeout_ms, topic, since, max_events
- [x] `termlink_event_subscribe` MCP tool added using `event.subscribe` RPC
- [x] Tool description explains push-based delivery and since parameter
- [x] MCP integration tests for event_subscribe (history replay + timeout empty)
- [x] `cargo check -p termlink-mcp` passes
- [x] `cargo test -p termlink-mcp` passes

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
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

grep -q "event_subscribe" crates/termlink-mcp/src/tools.rs
cargo check -p termlink-mcp 2>&1 | grep -q "Finished"

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

### 2026-03-30T17:55:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-809-add-eventsubscribe-mcp-tool-for-push-bas.md
- **Context:** Initial task creation

### 2026-03-30T17:59:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
