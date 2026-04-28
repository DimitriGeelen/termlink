---
id: T-1347
name: "channel subscribe --senders csv — filter by sender id"
description: >
  channel subscribe --senders csv — filter by sender id

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T20:54:47Z
last_update: 2026-04-27T21:03:59Z
date_finished: 2026-04-27T21:03:59Z
---

# T-1347: channel subscribe --senders csv — filter by sender id

## Context

In a multi-party topic, an operator often wants to focus on what one
specific peer said. Add `subscribe --senders <csv>` — render-side filter
that drops envelopes whose `sender_id` is not in the CSV. Strict equality
match (no substring); empty entries ignored. JSON mode applies the same
filter. Composes with all other passes (reactions/edits/redactions still
process the full set; only the rendered subset is filtered).

## Acceptance Criteria

### Agent
- [x] `--senders <csv>` flag added to `channel subscribe`
- [x] When set, only envelopes whose `sender_id` is in the CSV (strict equality, comma-split + trim) are rendered
- [x] JSON output applies the same filter (filtered envelopes are dropped)
- [x] Pure helper `sender_in_csv(sender, csv) -> bool` — empty target list returns false; empty sender returns false; whitespace trimmed; case-sensitive
- [x] Unit tests: empty csv, single-id csv, multi-id csv, whitespace tolerance, no-match returns false, exact match returns true
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

### 2026-04-27T20:54:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1347-channel-subscribe---senders-csv--filter-.md
- **Context:** Initial task creation

### 2026-04-27T21:03:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
