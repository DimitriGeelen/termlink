---
id: T-1321
name: "channel edit — Matrix m.replace analogue (msg_type=edit + metadata.replaces)"
description: >
  channel edit — Matrix m.replace analogue (msg_type=edit + metadata.replaces)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T15:11:16Z
last_update: 2026-04-27T15:18:12Z
date_finished: 2026-04-27T15:18:12Z
---

# T-1321: channel edit — Matrix m.replace analogue (msg_type=edit + metadata.replaces)

## Context

Matrix `m.replace` lets a sender correct a previously-sent message. Apply the same
shape to channels: emit a new post with `msg_type=edit` and
`metadata.replaces=<original-offset>`. Hub stores both records (no rewrite — append-only,
verifiable history). Reader side, when subscribing in a "rendered view" mode, collapses
each chain to the latest version for that original offset.

For T-1321 we add the *emit* side (`channel edit`) and a minimal renderer toggle
(`--collapse-edits`) on `channel subscribe`. Old/un-aware peers see both records as
separate posts — Tier-A additive.

## Acceptance Criteria

### Agent
- [x] `ChannelAction::Edit { topic, replaces, payload, hub, json }` added to cli.rs
- [x] `cmd_channel_edit` in commands/channel.rs — wraps `cmd_channel_post` with
      `msg_type=edit`, `metadata.replaces=<offset>`, payload = new text
- [x] Sender identity/signing path is the existing post path (no signing changes)
- [x] `cmd_channel_subscribe` gains `--collapse-edits` flag — when true, builds a
      `replaces_map: BTreeMap<u64,String>` in the first pass, then renders each
      original offset with its latest edit text (or original if none)
- [x] Edit posts themselves are suppressed in collapsed view (just the merged result
      shown for the parent)
- [x] Unit test `collapse_edits_picks_latest` — given (orig, edit_v1, edit_v2),
      collapsed render shows v2 text against the original offset
- [x] `cargo test -p termlink --bins` passes; clippy clean
- [x] `agent-conversations.md` gains a short "Edits (m.replace)" section

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
cargo test -p termlink --bins collapse_edits
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

### 2026-04-27T15:11:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1321-channel-edit--matrix-mreplace-analogue-m.md
- **Context:** Initial task creation

### 2026-04-27T15:18:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
