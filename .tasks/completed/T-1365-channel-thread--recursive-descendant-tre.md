---
id: T-1365
name: "channel threads — index of threads in a topic with reply counts"
description: >
  Add `channel threads <topic>` — list every top-level post that has at least one
  reply
  (a thread root), with reply count, distinct participant count, and last-activity
  ts.
  Renders as a sortable table or JSON. Builds on the parent→children index already
  used
  by `channel thread` (T-1328); this is the index/overview view, not the single-thread
  drill-down. Matrix m.thread analog at the room-overview level.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [agent-conversation, matrix, threads, channel-cli]
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: [T-1328, T-1362, T-1363]
created: 2026-04-28T09:05:29Z
last_update: '2026-08-18T18:58:48Z'
date_finished: 2026-04-28T09:28:22Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:56Z'
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
  - ts: '2026-08-18T18:58:48Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 5
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-1365: channel threads — index of threads in a topic with reply counts

## Context

`channel thread <topic> <offset>` (T-1328) shows ONE thread tree. There is currently no
way to see WHICH offsets in a topic are thread roots, or how big each thread is, without
walking the topic by hand. `channel threads <topic>` fills the gap — one row per thread
root, with `reply_count`, `participants` (distinct senders in the thread), `last_ts_ms`,
and root payload preview. Sorted by last_ts_ms desc by default. Honors redacted offsets
(redacted root → row dropped). JSON shape mirrors the table.

Pure helper `compute_threads_index(envelopes) -> Vec<ThreadIndexRow>` so unit tests can
exercise the aggregation without an RPC round-trip.

## Acceptance Criteria

### Agent
- [x] CLI variant `Channel threads <topic>` accepted; `--hub`, `--json`, `--top N` flags wired
- [x] `compute_threads_index` (pure helper) added with unit tests covering: no-reply topic (empty result), single thread, multiple threads, redacted root dropped, redacted reply dropped, deeply-nested thread (depth ≥ 3), `--top N` truncation
- [x] Live smoke test against the local hub produces a table matching expected output
- [x] e2e step added to `tests/e2e/agent-conversation.sh` (positive + negative + JSON shape assertions)
- [x] `cargo build --release -p termlink && cargo test -p termlink --bins --quiet && cargo clippy --all-targets --workspace -- -D warnings` all green

## Verification

cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3

## Decisions

## Updates

### 2026-04-28T09:05:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1365-channel-thread--recursive-descendant-tre.md
- **Context:** Initial task creation; refocused from "thread tree view (already shipped T-1328)" to "threads INDEX (plural)" — the actual gap.

### 2026-04-28T09:28:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
