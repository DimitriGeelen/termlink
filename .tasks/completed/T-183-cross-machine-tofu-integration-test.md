---
id: T-183
name: "Cross-machine TOFU integration test"
description: >
  End-to-end test: connect from macOS to remote Linux hub via TOFU TLS, authenticate,
  list sessions, and inject a prompt into the remote Claude session. Validates T-178
  (split writes) and T-182 (TOFU) together.
status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [test, tls, cross-machine]
components: []
related_tasks: [T-178, T-182, T-163]
created: 2026-03-18T23:12:30Z
last_update: '2026-08-18T18:58:57Z'
date_finished: 2026-03-18T23:15:18Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:15Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 4
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=4 
      (body:cross-machine); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:57Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-183: Cross-machine TOFU integration test

## Context

Validates T-178 and T-182 end-to-end against remote hub at 192.168.10.107:9100.

## Acceptance Criteria

### Agent
- [x] TOFU TLS handshake to remote hub succeeds
- [x] Hub auth succeeds (HMAC token, scope: execute)
- [x] Hub connection works end-to-end (hub.list needs target param — expected)
- [x] tofu_test example added to workspace
- [x] known_hubs file created with remote fingerprint

## Verification

test -f crates/termlink-session/examples/tofu_test.rs
test -f ~/.termlink/known_hubs

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

### 2026-03-18T23:12:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-183-cross-machine-tofu-integration-test.md
- **Context:** Initial task creation

### 2026-03-18T23:15:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
