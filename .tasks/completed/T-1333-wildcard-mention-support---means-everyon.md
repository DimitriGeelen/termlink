---
id: T-1333
name: "Wildcard mention support — '*' means 'everyone' for posters and filters"
description: >
  Wildcard mention support — '*' means 'everyone' for posters and filters

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/channel.rs]
related_tasks: []
created: 2026-04-27T17:04:35Z
last_update: 2026-04-27T17:09:17Z
date_finished: 2026-04-27T17:09:17Z
---

# T-1333: Wildcard mention support — '*' means 'everyone' for posters and filters

## Context

T-1325 introduced `--mention <id>` and `--filter-mentions <id>` for direct mentions.
Add `*` as the wildcard value: `--mention "*"` writes `metadata.mentions=*` (Matrix
@room analogue), and `--filter-mentions "*"` matches any post that mentions ANYONE
(non-empty mentions metadata) — useful for "show me where anyone got tagged".
Pure helper makes the wildcard-aware match testable.

## Acceptance Criteria

### Agent
- [x] `mentions_match` recognizes `target == "*"` to mean "any non-empty mentions match", and recognizes csv containing `*` as matching any target.
- [x] Unit tests cover: target `*` matches non-empty csv, target `*` does not match empty csv, csv `*` matches any target, csv `alice,*` matches any target.
- [x] `cargo test -p termlink --bins mentions_match` passes (existing tests still pass; new `wildcard` cases pass).
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] Smoke: `--mention "*"` round-trip (post + filter) shows the message; `--filter-mentions "*"` only surfaces posts with any mention.

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

cargo test -p termlink --bins mentions_match
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

### 2026-04-27T17:04:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1333-wildcard-mention-support---means-everyon.md
- **Context:** Initial task creation

### 2026-04-27T17:09:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
