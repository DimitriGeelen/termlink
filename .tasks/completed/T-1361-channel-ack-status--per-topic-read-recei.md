---
id: T-1361
name: "channel ack-status — per-topic read-receipt overview across all senders"
description: >
  channel ack-status — per-topic read-receipt overview across all senders

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T07:42:58Z
last_update: '2026-08-18T18:58:48Z'
date_finished: 2026-04-28T07:55:10Z
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
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-1361: channel ack-status — per-topic read-receipt overview across all senders

## Context

Read-receipt dashboard for a topic. Composes existing receipts data with the latest topic offset and the topic's member set into a status view: per-member, where they are vs. latest. Adds a "lag" column and surfaces members who have never posted a receipt.

Distinct from `channel receipts` (raw list, no lag computation) and `channel unread <topic>` (single-sender unread count).

## Acceptance Criteria

### Agent
- [x] `channel ack-status <topic>` renders rows: `<sender_id> ack=<up_to> latest=<L> lag=<N>` sorted by lag desc
- [x] Members who have posted content but no receipt appear with `ack=- lag=L+1`
- [x] `--pending-only` filters to lag>0 entries
- [x] `--json` returns `[{sender_id, up_to, lag, ts}]`
- [x] Empty topic prints "Topic '<t>' is empty"
- [x] Pure helper `compute_ack_status` unit-tested across: empty, caught-up, mixed lag, member without receipt, sorted-by-lag, pending-only filter
- [x] e2e step covering full lifecycle
- [x] cargo build, test, clippy --all-targets -- -D warnings all pass

## Verification
cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5

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

### 2026-04-28T07:42:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1361-channel-ack-status--per-topic-read-recei.md
- **Context:** Initial task creation

### 2026-04-28T07:55:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
