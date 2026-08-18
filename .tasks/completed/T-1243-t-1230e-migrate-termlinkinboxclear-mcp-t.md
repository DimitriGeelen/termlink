---
id: T-1243
name: "T-1230e migrate termlink_inbox_clear MCP to clear_with_fallback"
description: >
  T-1230e migrate termlink_inbox_clear MCP to clear_with_fallback

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-25T10:40:59Z
last_update: '2026-08-18T18:58:46Z'
date_finished: 2026-04-25T10:47:11Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:51Z'
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
  - ts: '2026-08-18T18:58:46Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 5
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-1243: T-1230e migrate termlink_inbox_clear MCP to clear_with_fallback

## Context

T-1230e per inception: migrate `termlink_inbox_clear` MCP tool
(`crates/termlink-mcp/src/tools.rs:4537`) to `clear_with_fallback` (T-1236).
Sibling of T-1242 (CLI local).

## Acceptance Criteria

### Agent
- [x] MCP tool calls `clear_with_fallback(&addr, scope, cache, &mut ctx)`
- [x] JSON output uses serde Serialize on InboxClearResult
- [x] cargo build -p termlink-mcp clean

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification
cargo build -p termlink-mcp 2>&1 | tail -3 | grep -q "Finished"

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

### 2026-04-25T10:40:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1243-t-1230e-migrate-termlinkinboxclear-mcp-t.md
- **Context:** Initial task creation

### 2026-04-25T10:47:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
