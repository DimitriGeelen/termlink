---
id: T-1355
name: "channel poll start/vote/end/results — Matrix m.poll"
description: >
  channel poll start/vote/end/results — Matrix m.poll

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T06:25:52Z
last_update: 2026-04-28T06:42:23Z
date_finished: 2026-04-28T06:42:23Z
---

# T-1355: channel poll start/vote/end/results — Matrix m.poll

## Context

Matrix `m.poll.start` / `m.poll.response` / `m.poll.end` analog as additive envelopes. Three new envelope types share the topic:
- `msg_type=poll_start`, payload=question, `metadata.poll_options=opt1|opt2|opt3` (pipe-delimited so single-line CSV-safe)
- `msg_type=poll_vote`, `metadata.poll_id=<offset>`, `metadata.poll_choice=<index>`. Latest vote per (poll_id, sender) wins.
- `msg_type=poll_end`, `metadata.poll_id=<offset>`. Closes voting; further votes ignored by aggregator.

`channel poll results <topic> <poll_id>` walks the topic, applies the rules, and renders tallies. `--json` for machine output.

## Acceptance Criteria

### Agent
- [x] `channel poll start <topic> --question <Q> --option <A> --option <B>` posts a poll_start envelope (>=2 options required)
- [x] `channel poll vote <topic> <poll_id> --choice <index>` posts a poll_vote envelope
- [x] `channel poll end <topic> <poll_id>` posts a poll_end envelope
- [x] `channel poll results <topic> <poll_id>` shows tallies (per option: count + voters); `--json` returns object with `closed: bool`
- [x] Vote re-emit replaces prior vote (latest wins); votes after poll_end ignored
- [x] `compute_poll_state` unit-tested across: empty, no votes, one vote, replacement, vote-after-end, multiple voters
- [x] e2e step added covering full poll lifecycle
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

### 2026-04-28T06:25:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1355-channel-poll-startvoteendresults--matrix.md
- **Context:** Initial task creation

### 2026-04-28T06:42:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
