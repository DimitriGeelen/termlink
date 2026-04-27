---
id: T-1320
name: "channel dm --list — discover existing DM topics for caller"
description: >
  channel dm --list — discover existing DM topics for caller

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T15:04:09Z
last_update: 2026-04-27T15:10:45Z
date_finished: 2026-04-27T15:10:45Z
---

# T-1320: channel dm --list — discover existing DM topics for caller

## Context

T-1319 added `channel dm <peer>` to canonicalize a DM topic between two identities.
Operators still need a way to *discover* existing DMs for the current identity. Walking
`channel list` and grepping `dm:` works but is awkward. Add `channel dm --list` that
filters topics with prefix `dm:` and containing the caller's identity fingerprint, and
prints peer-side fingerprint for each.

## Acceptance Criteria

### Agent
- [x] `channel dm --list` flag added to ChannelAction::Dm in cli.rs (mutually exclusive with `<peer>` arg)
- [x] Implementation queries `channel.list` (no filter), filters topics matching `dm:<a>:<b>` where `a` or `b` equals caller fingerprint, prints `<topic>  (peer=<other-fp>)` rows
- [x] When no DM topics match: prints `No DM topics found for identity <fp-prefix>` to stderr, exit 0
- [x] Unit test: `dm_list_filters_to_caller_identity` — given list of topics, returns only those containing caller fp
- [x] `cargo test -p termlink` passes
- [x] `cargo clippy --all-targets -- -D warnings` clean for termlink-cli

## Verification

cargo test -p termlink --bins dm_list_filter
cargo clippy -p termlink --all-targets -- -D warnings

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

### 2026-04-27T15:04:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1320-channel-dm---list--discover-existing-dm-.md
- **Context:** Initial task creation

### 2026-04-27T15:10:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
