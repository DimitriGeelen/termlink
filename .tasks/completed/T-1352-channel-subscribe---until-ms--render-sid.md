---
id: T-1352
name: "channel subscribe --until ms — render-side upper-bound timestamp filter"
description: >
  channel subscribe --until ms — render-side upper-bound timestamp filter

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T21:48:04Z
last_update: 2026-04-27T21:56:33Z
date_finished: 2026-04-27T21:56:33Z
---

# T-1352: channel subscribe --until ms — render-side upper-bound timestamp filter

## Context

T-1343 added `subscribe --since <ms>` (lower-bound timestamp filter).
Closing the pair: `--until <ms>` for the upper bound, so operators can
slice an arbitrary `[since, until]` window without follow-up greps.
Pure render-side filter; pagination unchanged.

## Acceptance Criteria

### Agent
- [x] `--until <ms>` flag added to `channel subscribe`
- [x] When set, envelopes whose `ts > <ms>` are dropped from rendered output
- [x] Combines with `--since` to define a `[since, until]` window
- [x] Pure helper `should_emit_for_until(env, until)` mirroring `should_emit_for_since` semantics; ts-less envelopes are kept (defensive — same precedent as --since)
- [x] Unit tests: no filter, equal boundary, before, after, ts-less envelope kept
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

### 2026-04-27T21:48:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1352-channel-subscribe---until-ms--render-sid.md
- **Context:** Initial task creation

### 2026-04-27T21:56:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
