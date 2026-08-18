---
id: T-1341
name: "channel members <topic> — distinct senders with last-seen timestamps"
description: >
  channel members <topic> — distinct senders with last-seen timestamps

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T19:07:13Z
last_update: '2026-08-18T18:58:48Z'
date_finished: 2026-04-27T19:15:50Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:55Z'
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
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-1341: channel members <topic> — distinct senders with last-seen timestamps

## Context

`channel members <topic>` — distinct senders ever-seen on a topic, with per-sender post-count, first-seen ts, and last-seen ts. Sorted by last-seen desc by default. Lighter than `channel info` (which mixes description, retention, top senders, receipts) — just the membership list. Read-only. Pure helper `summarize_members(msgs) -> Vec<MemberRow>` unit-tested.

## Acceptance Criteria

### Agent
- [x] `channel members <topic>` prints `<sender_id>  posts=N  first=ts  last=ts` per row
- [x] Sorted by last-seen ts descending; None-ts members sort last; stable ties (BTreeMap pre-sort by sender_id)
- [x] Counts only content envelopes — meta types (UNREAD_META_TYPES) excluded by default
- [x] `--include-meta` includes meta envelopes too (for full audit)
- [x] `--json` emits `{topic, include_meta, members: [...]}` with per-member records
- [x] Pure helpers `summarize_members` (5 tests) + `MemberRow::to_json` (1 test) unit-tested
- [x] Build/test/clippy all clean (303 tests passing); e2e `agent-conversation.sh` step 19 added and passes (asserts --include-meta grows total post count)

## Verification

cargo build --release -p termlink
cargo test -p termlink --bins --quiet
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

### 2026-04-27T19:07:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1341-channel-members-topic--distinct-senders-.md
- **Context:** Initial task creation

### 2026-04-27T19:15:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
