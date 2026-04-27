---
id: T-1325
name: "channel mentions — Matrix m.mention analogue (metadata.mentions + --filter-mentions)"
description: >
  channel mentions — Matrix m.mention analogue (metadata.mentions + --filter-mentions)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:35:02Z
last_update: 2026-04-27T15:35:02Z
date_finished: null
---

# T-1325: channel mentions — Matrix m.mention analogue (metadata.mentions + --filter-mentions)

## Context

Matrix `m.mention.user_ids` lets a sender flag a message as relevant for specific
recipients. We map it to channels via `metadata.mentions=<csv-of-ids>` on any
post (typically `chat`). Reader side adds a `--filter-mentions <id>` flag to
subscribe — server-side rejected since hub doesn't parse CSV; we filter
client-side over the existing read pipeline. Renderer shows a `@<id>...` marker
inline so the operator can spot mentions at a glance.

Strictly additive: post side just an extra `--mention <id>` flag (repeatable);
old hubs and old subscribers ignore unknown metadata.

## Acceptance Criteria

### Agent
- [x] `cmd_channel_post` accepts `mentions: &[String]` — when non-empty, sets
      `metadata.mentions=<comma-joined>`
- [x] `Post` and `Dm` CLI variants get `--mention <id>` (repeatable: `Vec<String>`)
- [x] `cmd_channel_subscribe` gains `--filter-mentions <id>` flag (Option<String>)
      — when set, filters render to envelopes whose `metadata.mentions` CSV
      contains `<id>`. JSON mode unaffected.
- [x] Renderer prefixes a mention marker on lines that have `metadata.mentions`:
      `[N @alice,bob]` (truncated to first 3 ids if more).
- [x] Pure helper `mentions_match(metadata_csv: &str, target: &str) -> bool` —
      strict comma-split with whitespace trim; handles empty CSV / target.
- [x] Unit test `mentions_match_csv_lookups` covering: hit, miss, padded with
      whitespace, empty CSV, mid-string-substring-no-match
- [x] `cargo test -p termlink --bins` + clippy clean
- [x] `agent-conversations.md` gains a "Mentions (m.mention)" section

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
cargo test -p termlink --bins mentions_match
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

### 2026-04-27T15:35:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1325-channel-mentions--matrix-mmention-analog.md
- **Context:** Initial task creation
