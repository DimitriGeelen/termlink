---
id: T-1382
name: "channel state --since <ms> — incremental state updates (Matrix /sync analogue)"
description: >
  channel state --since <ms> — incremental state updates (Matrix /sync analogue)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T15:46:26Z
last_update: 2026-04-28T15:59:58Z
date_finished: 2026-04-28T15:59:58Z
---

# T-1382: channel state --since <ms> — incremental state updates (Matrix /sync analogue)

## Context

Matrix `/sync` returns only events that changed since the last sync token.
Mirror that for termlink channels: `channel state --since <ms>` returns
only the rows whose canonical state changed at or after `<ms>` —
new posts, new edits, new redactions. Composes T-1376 `compute_state`
with a per-row "last change" filter.

Predecessors: T-1376 (compute_state), T-1378 (compute_snapshot — temporal
slicing pattern).

## Acceptance Criteria

### Agent
- [x] `compute_state_since(envelopes, since_ms, include_redacted) -> Vec<StateRow>` added to `crates/termlink-cli/src/commands/channel.rs`. Returns rows whose `max(ts_ms, latest_edit_ts_ms, redaction_ts) >= since_ms`. Delegates to `compute_state` for the canonical render, then filters.
- [x] `cmd_channel_state_since` async wrapper added with text + JSON output paths.
- [x] `ChannelAction::StateSince { topic, since_ms, include_redacted, hub, json }` variant added in `cli.rs`; main.rs dispatches to `cmd_channel_state_since`.
- [x] At least 6 unit tests covering: empty envelopes, no rows since cutoff, new post since cutoff, edit-since-cutoff (original was earlier), redaction-since-cutoff, since=0 returns full state.
- [x] `cargo build --release -p termlink` clean.
- [x] `cargo test -p termlink --bins` green (current baseline 517 → expected 523+).
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] One e2e step added to `tests/e2e/agent-conversation.sh` exercising state --since with edits and a cutoff that includes some but not all changes.
- [x] `bash tests/e2e/agent-conversation.sh` passes start to finish.
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
grep -q "compute_state_since" crates/termlink-cli/src/commands/channel.rs
grep -q "StateSince" crates/termlink-cli/src/cli.rs
grep -q "channel state-since" docs/operations/agent-conversations.md

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

### 2026-04-28T15:46:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1382-channel-state---since-ms--incremental-st.md
- **Context:** Initial task creation

### 2026-04-28T15:59:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
