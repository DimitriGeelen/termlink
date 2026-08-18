---
id: T-1336
name: "channel search <topic> <pattern> — read-only payload grep"
description: >
  channel search <topic> <pattern> — read-only payload grep

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T18:05:22Z
last_update: '2026-08-18T18:58:48Z'
date_finished: 2026-04-27T18:15:11Z
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

# T-1336: channel search <topic> <pattern> — read-only payload grep

## Context

`channel search <topic> <pattern>` — walks a topic once and prints content envelopes whose decoded payload contains `<pattern>` (case-insensitive substring by default; `--regex` for full regex; `--case-sensitive` to disable folding). Read-only client-side filter, no new hub RPC. Skips meta envelopes (UNREAD_META_TYPES) by default — searches only content unless `--all` is given. `--limit N` caps printed hits; `--json` outputs `{offset, sender_id, ts, payload}` records. Tier-A: hub never sees the pattern.

## Acceptance Criteria

### Agent
- [x] `channel search <topic> <pattern>` prints matching content envelopes by offset
- [x] Default mode: case-insensitive substring; `--case-sensitive` exact, `--regex` full regex
- [x] `--all` includes meta envelopes (otherwise meta types from UNREAD_META_TYPES are skipped)
- [x] `--limit N` caps printed matches (0 = unlimited)
- [x] `--json` returns array of `{offset, sender_id, ts, payload}` records
- [x] Pure helper `payload_matches` unit-tested across substring/regex/case modes (6 cases) + `decode_payload_lossy` (3 cases)
- [x] Build/test/clippy all clean; e2e `agent-conversation.sh` step 14 added and passes

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

### 2026-04-27T18:05:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1336-channel-search-topic-pattern--read-only-.md
- **Context:** Initial task creation

### 2026-04-27T18:15:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
