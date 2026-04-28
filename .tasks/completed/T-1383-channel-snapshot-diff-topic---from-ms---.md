---
id: T-1383
name: "channel snapshot-diff <topic> --from <ms> --to <ms> — what changed between two timestamps (compose two T-1378 snapshots)"
description: >
  channel snapshot-diff <topic> --from <ms> --to <ms> — what changed between two timestamps (compose two T-1378 snapshots)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T16:00:59Z
last_update: 2026-04-28T16:15:05Z
date_finished: 2026-04-28T16:15:05Z
---

# T-1383: channel snapshot-diff <topic> --from <ms> --to <ms> — what changed between two timestamps (compose two T-1378 snapshots)

## Context

Compose two T-1378 `compute_snapshot` calls to produce a typed diff
between two points in time. Useful for forensic replay ("between
incident T0 and T1, what changed?") and audit reports.

Each row classifies as one of: `added` (offset present at `to` but
absent at `from`), `removed` (present at `from`, absent at `to` —
typically due to redaction-after-from), `edited` (text differs between
the two snapshots), `unchanged` (text identical, no metadata change).
Default text rendering omits `unchanged` (only diff entries shown).
`--include-unchanged` shows all four classes.

Predecessors: T-1376 compute_state, T-1378 compute_snapshot.

## Acceptance Criteria

### Agent
- [x] `compute_snapshot_diff(envelopes, from_ms, to_ms, include_redacted) -> Vec<DiffRow>` added to `crates/termlink-cli/src/commands/channel.rs`. Internally calls `compute_snapshot` twice and classifies the union of offsets.
- [x] `DiffRow` struct with `offset, change_kind: "added"|"removed"|"edited"|"unchanged", from_payload: Option<String>, to_payload: Option<String>, sender_id`.
- [x] `cmd_channel_snapshot_diff` async wrapper with text + JSON output.
- [x] `ChannelAction::SnapshotDiff { topic, from_ms, to_ms, include_redacted, include_unchanged, hub, json }` variant added in `cli.rs`; main.rs dispatches.
- [x] At least 6 unit tests: empty topic, no diff (from==to), pure-add (post between from and to), pure-remove (redaction between from and to), edit between from and to, unchanged-row classification.
- [x] `cargo build --release -p termlink` clean.
- [x] `cargo test -p termlink --bins` green (524 → 530+).
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] One e2e step (55) added to `tests/e2e/agent-conversation.sh`.
- [x] `bash tests/e2e/agent-conversation.sh` passes.
- [x] Section added to `docs/operations/agent-conversations.md`.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build --release -p termlink 2>&1 | tail -3
cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace --quiet -- -D warnings 2>&1 | tail -3
grep -q "compute_snapshot_diff" crates/termlink-cli/src/commands/channel.rs
grep -q "SnapshotDiff" crates/termlink-cli/src/cli.rs
grep -q "channel snapshot-diff" docs/operations/agent-conversations.md

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

### 2026-04-28T16:00:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1383-channel-snapshot-diff-topic---from-ms---.md
- **Context:** Initial task creation

### 2026-04-28T16:15:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
