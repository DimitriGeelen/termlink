---
id: T-1346
name: "channel subscribe --tail N — render only last N envelopes"
description: >
  channel subscribe --tail N — render only last N envelopes

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T20:39:26Z
last_update: 2026-04-27T20:50:50Z
date_finished: 2026-04-27T20:50:50Z
---

# T-1346: channel subscribe --tail N — render only last N envelopes

## Context

Reading a noisy topic with `channel subscribe` floods the terminal. Add
`--tail <N>` so only the last N envelopes are rendered (mirrors `tail -n
N`). Pure render-side filter — pagination/cursor unchanged. Composes with
existing flags (`--show-parent`, `--reactions`, `--collapse-edits`, etc.)
because it's applied after all aggregation passes.

## Acceptance Criteria

### Agent
- [x] `--tail <N>` flag added to `channel subscribe`; conflicts with `--follow` (tail of an unbounded stream is ill-defined)
- [x] When set, only the last N envelopes from the rendered set are emitted; ordering preserved (oldest of the last-N first)
- [x] Pure helper `tail_slice<T: Clone>(items: &[T], tail: Option<usize>) -> Vec<T>` returns last N (or all if N >= len, or empty if N=0); unit-tested
- [x] JSON output applies the same tail; no JSON shape change
- [x] When combined with `--show-parent`, parent cache is built from the FULL topic walk (not just the tail) so quotes still resolve
- [x] Unit tests: empty input, N=0, N >= len, N < len, with non-clone-cheap-but-clone-correct value
- [x] `cargo test -p termlink --bins --quiet` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] e2e step extended; full run green

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build --release -p termlink
cargo test -p termlink --bins --quiet
cargo clippy --all-targets --workspace -- -D warnings
bash tests/e2e/agent-conversation.sh

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

### 2026-04-27T20:39:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1346-channel-subscribe---tail-n--render-only-.md
- **Context:** Initial task creation

### 2026-04-27T20:50:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
