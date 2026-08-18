---
id: T-172
name: "Fix forward_to_remote_session_via_tcp test broken by TLS cert detection"
description: >
  Fix forward_to_remote_session_via_tcp test broken by TLS cert detection

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-18T21:09:20Z
last_update: '2026-08-18T18:58:54Z'
date_finished: 2026-03-18T21:12:11Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:10Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:54Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-172: Fix forward_to_remote_session_via_tcp test broken by TLS cert detection

## Context

T-165 added TLS cert detection in `connect_addr` — if `hub.cert.pem` exists in runtime dir, TCP connections use TLS. The `forward_to_remote_session_via_tcp` test uses a plain TCP proxy, but doesn't isolate `TERMLINK_RUNTIME_DIR`, so a stale cert from prior hub runs triggers TLS handshake failure.

## Acceptance Criteria

### Agent
- [x] Test sets `TERMLINK_RUNTIME_DIR` to a clean temp dir (no stale cert)
- [x] Test acquires `ENV_LOCK` (required when setting env vars)
- [x] `cargo test --package termlink-hub` passes all tests (0 failures)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub --lib forward_to_remote_session_via_tcp 2>&1 | grep -q "1 passed"

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

### 2026-03-18T21:09:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-172-fix-forwardtoremotesessionviatcp-test-br.md
- **Context:** Initial task creation

### 2026-03-18T21:12:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
