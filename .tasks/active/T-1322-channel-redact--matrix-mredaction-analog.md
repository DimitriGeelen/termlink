---
id: T-1322
name: "channel redact — Matrix m.redaction analogue (msg_type=redaction + metadata.redacts)"
description: >
  channel redact — Matrix m.redaction analogue (msg_type=redaction + metadata.redacts)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:18:34Z
last_update: 2026-04-27T15:18:34Z
date_finished: null
---

# T-1322: channel redact — Matrix m.redaction analogue (msg_type=redaction + metadata.redacts)

## Context

Matrix `m.redaction` lets a sender retract a previous message. Same shape as edits
(T-1321) but a different intent: the original is hidden, not replaced. We use
`msg_type=redaction` carrying `metadata.redacts=<offset>` and an optional `reason`
in metadata. Renderer toggle (`--hide-redacted`) suppresses the redacted parent and
the redaction envelope; default view shows redactions explicitly so the operator
can see what was retracted.

## Acceptance Criteria

### Agent
- [x] `ChannelAction::Redact { topic, redacts, reason, hub, json }` added to cli.rs
- [x] `cmd_channel_redact` in commands/channel.rs — emits `msg_type=redaction`,
      `metadata.redacts=<offset>` (and optional `reason=<text>`), payload empty
- [x] `cmd_channel_subscribe` gains `--hide-redacted` flag — suppresses both the
      redaction envelope itself AND the redacted parent (offset)
- [x] Pure helper `redacted_offsets(msgs) -> HashSet<u64>` accumulates target offsets
- [x] Default render (without flag) shows redaction explicitly: `[N redact] sender → offset M (reason: ...)`
- [x] Unit test `redacted_offsets_collects_targets` — given mixed messages, returns only the redacted-parent offsets
- [x] `cargo test -p termlink --bins` + `cargo clippy --all-targets -- -D warnings` clean
- [x] `agent-conversations.md` gains a short "Redactions (m.redaction)" section

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
cargo test -p termlink --bins redacted_offsets
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

### 2026-04-27T15:18:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1322-channel-redact--matrix-mredaction-analog.md
- **Context:** Initial task creation
