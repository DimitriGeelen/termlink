---
id: T-786
name: "Add TLS test coverage — certificate validation, expired cert, wrong hostname"
description: >
  Add TLS test coverage — certificate validation, expired cert, wrong hostname

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T12:01:39Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T12:22:00Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-786: Add TLS test coverage — certificate validation, expired cert, wrong hostname

## Context

tls.rs has only 2 tests. Adding edge case coverage for error paths: invalid PEM, mismatched cert/key, missing files, cleanup.

## Acceptance Criteria

### Agent
- [x] Invalid PEM data returns error (not panic)
- [x] Client connector with nonexistent cert file returns error
- [x] Cleanup removes cert and key files
- [x] All new and existing tests pass
- [x] Total test count updated in ARCHITECTURE.md

## Verification

cargo test -p termlink-hub --lib tls::tests

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

### 2026-03-30T12:01:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-786-add-tls-test-coverage--certificate-valid.md
- **Context:** Initial task creation

### 2026-03-30T12:22:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
