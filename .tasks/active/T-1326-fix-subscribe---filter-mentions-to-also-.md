---
id: T-1326
name: "fix subscribe --filter-mentions to also suppress redaction explicit-render branch (e2e bug)"
description: >
  fix subscribe --filter-mentions to also suppress redaction explicit-render branch (e2e bug)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:45:39Z
last_update: 2026-04-27T15:45:39Z
date_finished: null
---

# T-1326: fix subscribe --filter-mentions to also suppress redaction explicit-render branch (e2e bug)

## Context

E2E walkthrough (alice + bob, two real identities) caught a renderer-ordering bug:
the redaction explicit-render branch in `cmd_channel_subscribe` runs BEFORE the
`--filter-mentions` check, so a `[N redact]` line leaks into a filtered view
even though the redaction envelope has no mentions metadata. Fix is to gate
the redaction render with the same mention filter.

## Acceptance Criteria

### Agent
- [x] redaction render branch checks `filter_mentions` before printing
- [x] e2e walkthrough step 13 (`subscribe --filter-mentions <bob-fp>`) shows
      ONLY the `[8 @<bob>]` mention line; no `[6 redact]`
- [x] `cargo test -p termlink --bins` + clippy clean
- [x] re-run /tmp/e2e-conv.sh end-to-end to completion

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

### 2026-04-27T15:45:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1326-fix-subscribe---filter-mentions-to-also-.md
- **Context:** Initial task creation
