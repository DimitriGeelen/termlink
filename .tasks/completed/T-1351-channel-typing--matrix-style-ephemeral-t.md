---
id: T-1351
name: "channel typing — Matrix-style ephemeral typing indicator with TTL"
description: >
  channel typing — Matrix-style ephemeral typing indicator with TTL

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T21:34:31Z
last_update: 2026-04-27T21:44:00Z
date_finished: 2026-04-27T21:44:00Z
---

# T-1351: channel typing — Matrix-style ephemeral typing indicator with TTL

## Context

Matrix has `m.typing` — ephemeral presence indicator showing who is
currently composing a message. Add an append-only equivalent:
`channel typing <topic> --emit` writes a `msg_type=typing` envelope with
`metadata.expires_at_ms=<wall-clock+ttl>`. `channel typing <topic>`
(default = list) walks the topic, finds typing envelopes whose
`expires_at_ms` is in the future, and reports active typers (one row per
sender, latest envelope wins). Default TTL: 30000ms (Matrix uses 30s
typing windows).

## Acceptance Criteria

### Agent
- [x] `channel typing <topic> --emit [--ttl-ms N]` emits a `msg_type=typing` envelope with `metadata.expires_at_ms=now+ttl` (default ttl 30000ms)
- [x] `channel typing <topic>` (default = list mode) walks the topic and renders rows for senders whose latest typing envelope has not expired
- [x] Pure helper `compute_active_typers(envelopes, now_ms) -> Vec<TyperRow>` — for each sender, picks the latest typing envelope; drops if expires_at_ms <= now_ms; sorts by ts desc
- [x] `channel typing --json` returns `[{sender_id, expires_at_ms, ts}]`
- [x] Unit tests: empty input, single typer (active), single typer (expired), multiple typers some expired, latest-per-sender wins (older non-expired entries replaced by newer expired one == still expired)
- [x] `cargo test -p termlink --bins --quiet` passes
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean
- [x] e2e step extended with emit + list + expiry; full run green

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

### 2026-04-27T21:34:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1351-channel-typing--matrix-style-ephemeral-t.md
- **Context:** Initial task creation

### 2026-04-27T21:44:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
