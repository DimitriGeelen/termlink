---
id: T-811
name: "Upgrade termlink watch to use event.subscribe for lower latency"
description: >
  Upgrade termlink watch to use event.subscribe for lower latency

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T18:05:46Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T18:10:41Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
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
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-811: Upgrade termlink watch to use event.subscribe for lower latency

## Context

`termlink watch` currently uses `event.poll` in a sleep loop (250-500ms latency). Replacing with `event.subscribe` gives near-zero latency — the server blocks until events arrive, eliminating the polling delay. The `since` parameter (T-805) enables cursor-based following.

## Acceptance Criteria

### Agent
- [x] `cmd_watch` uses `event.subscribe` RPC instead of `event.poll`
- [x] `cmd_wait` also upgraded to `event.subscribe` for consistency
- [x] Cursor tracking via `since` parameter (replaces manual cursor management)
- [x] `--interval` flag now controls subscribe timeout (server-side blocking) instead of client sleep
- [x] Multi-session watch round-robins subscribe calls across sessions
- [x] Ctrl+C, timeout, and max_count still work correctly
- [x] Backward compatible: same output format for text, JSON, payload-only modes
- [x] `cargo check -p termlink` passes
- [x] Full workspace test suite passes (684 tests, 0 failures)

## Verification

grep -q "event.subscribe" crates/termlink-cli/src/commands/events.rs
cargo check -p termlink 2>&1 | grep -q "Finished"

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

### 2026-03-30T18:05:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-811-upgrade-termlink-watch-to-use-eventsubsc.md
- **Context:** Initial task creation

### 2026-03-30T18:10:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
