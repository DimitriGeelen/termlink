---
id: T-1389
name: "topic_lint hot-reload bug — files created post-init never reload via SIGHUP"
description: >
  topic_lint hot-reload bug — files created post-init never reload via SIGHUP

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T20:49:13Z
last_update: 2026-04-28T20:49:13Z
date_finished: null
---

# T-1389: topic_lint hot-reload bug — files created post-init never reload via SIGHUP

## Context

`crates/termlink-hub/src/topic_lint.rs::init_topic_lint` records `RULES_PATH`
and `RELAYS_PATH` only when the file existed at hub startup (lines 246, 254,
269, 277 set `Some(path)`; line 285 sets `None`). The SIGHUP reload handler
(line 337: `if let Some(Some(path)) = RELAYS_PATH.get()`) requires the
recorded path to be `Some(Some(path))`. When a hub starts without
relay_declarations.yaml/topic_roles.yaml present and the operator creates
the file later, SIGHUP cannot pick it up — only a full hub restart works.

This was discovered live during T-1300/T-1301 fleet validation 2026-04-28:
created `/var/lib/termlink/relay_declarations.yaml` after hub was already
running, sent SIGHUP, broadcast still produced a lint warning (suppression
silently failed). Restart fixed it. The runbook in `docs/operations/topic-lint.md`
implies SIGHUP suffices, which is wrong for the cold-start case.

## Acceptance Criteria

### Agent
- [x] `init_topic_lint` always records the canonical runtime_dir path for both files (`Some(path)` even when the file is currently absent), so SIGHUP can later pick up newly-created files
- [x] SIGHUP reload handler treats "path recorded, file currently absent" as a no-op with debug log, NOT a warn (operator may legitimately remove the file to revert to defaults)
- [x] If SIGHUP runs after a file was deleted post-init, current state remains unchanged (defaults if previously defaults; previous parsed content otherwise — matches existing reload-error behavior)
- [x] Unit test: write file → init → confirm path recorded → delete file → SIGHUP → confirm no panic, state unchanged → recreate file with new content → SIGHUP → confirm new content reflected
- [x] `cargo test -p termlink-hub topic_lint` passes (21/21 incl. 2 new T-1389 tests)
- [x] `cargo clippy -p termlink-hub --tests -- -D warnings` clean
- [x] Live validation: hub restart without file → broadcast fires WARN. Install file + SIGHUP → next broadcast SUPPRESSED. No hub restart needed.

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

cargo test -p termlink-hub topic_lint 2>&1 | tail -3 | grep -qE "test result: ok"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"

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

### 2026-04-28T20:49:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1389-topiclint-hot-reload-bug--files-created-.md
- **Context:** Initial task creation
