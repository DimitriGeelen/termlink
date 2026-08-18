---
id: T-1239
name: "T-1229e migrate cmd_remote_inbox_inner Status arm to status_with_fallback_with_client"
description: >
  T-1229e migrate cmd_remote_inbox_inner Status arm to status_with_fallback_with_client

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-mcp/src/tools.rs, 
      crates/termlink-session/src/inbox_channel.rs]
related_tasks: []
created: 2026-04-25T10:36:21Z
last_update: '2026-08-18T18:58:46Z'
date_finished: 2026-04-25T10:47:05Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:51Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:46Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 6
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-1239: T-1229e migrate cmd_remote_inbox_inner Status arm to status_with_fallback_with_client

## Context

T-1229e per inception (`docs/reports/T-1229-inception.md`): migrate
`cmd_remote_inbox_inner` Status arm (`crates/termlink-cli/src/commands/remote.rs:1253`)
from legacy `inbox.status` RPC to `status_with_fallback_with_client` helper
(T-1235). Sibling List arm (line 1286) was migrated by T-1227 — same pattern.

Legacy renderer reads `t["transfer_count"]` (likely a typo — InboxStatus
struct exposes `pending`). Switch to typed struct fields.

## Acceptance Criteria

### Agent
- [x] Status arm calls `status_with_fallback_with_client(&mut rpc_client, conn.hub, cache, &mut ctx)`
- [x] Renderer reads typed `InboxStatus` struct fields (`status.total_transfers`, `t.target`, `t.pending`)
- [x] JSON output preserves shape via serde Serialize
- [x] cargo build -p termlink clean

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
cargo build -p termlink 2>&1 | tail -3 | grep -q "Finished"

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

### 2026-04-25T10:36:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1239-t-1229e-migrate-cmdremoteinboxinner-stat.md
- **Context:** Initial task creation

### 2026-04-25T10:47:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
