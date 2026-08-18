---
id: T-259
name: "Pickup from fw T-546: Release build fixes — flaky test ENV_LOCK + macOS runner"
description: >
  From framework agent pickup T-546: 1) Flaky test register_remote_and_discover missing
  ENV_LOCK guard (router.rs line 969), 2) macOS x86_64 CI runner macos-13 deprecated,
  change to macos-14 for cross-compile, 3) bump version and tag after fixes.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [pickup, release, ci]
components: []
related_tasks: []
created: 2026-03-24T08:41:40Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-03-24T08:52:52Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:52Z'
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
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-259: Pickup from fw T-546 — Release build fixes

## Context

Pickup from framework agent (T-546 on .107). Two TermLink-side fixes needed for release builds.

## Acceptance Criteria

### Agent
- [x] Flaky test `router::tests::register_remote_and_discover` fixed with ENV_LOCK guard
- [x] Release workflow macOS x86_64 target uses `macos-14` instead of deprecated `macos-13`
- [x] Tests pass: `cargo test register_remote_and_discover`

## Verification

grep -q "ENV_LOCK" crates/termlink-hub/src/router.rs

## Decisions

## Updates

### 2026-03-24T08:41:40Z — task-created [pickup from fw-agent on .107]
- **Source:** `/pickup fw-agent T-546` via termlink remote inject
- **Original message:** Flaky test router::tests::register_remote_and_discover — missing ENV_LOCK guard causes race condition with parallel tests clearing REMOTE_STORE. Fix: add `let _lock = ENV_LOCK.lock().await;` at top of test (line 969 in router.rs). Release workflow macOS x86_64 build — macos-13 runner deprecated/cancelled immediately. Fix: change os to macos-14 for x86_64-apple-darwin target (cross-compile). After fixes, bump version and tag for release test.

### 2026-03-24T08:52:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
