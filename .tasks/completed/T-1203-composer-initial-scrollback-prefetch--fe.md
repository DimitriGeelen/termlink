---
id: T-1203
name: "Composer initial-scrollback prefetch — feed query.output into each panel before first frame (T-236 follow-up)"
description: >
  Composer initial-scrollback prefetch — feed query.output into each panel before first frame (T-236 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/mirror_grid_composer.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-23T14:44:26Z
last_update: 2026-04-23T14:46:15Z
date_finished: 2026-04-23T14:46:15Z
---

# T-1203: Composer initial-scrollback prefetch — feed query.output into each panel before first frame (T-236 follow-up)

## Context

T-236's composer connects to each session's data plane and starts reading live
frames, but — unlike single-session `cmd_mirror` which calls
`client::rpc_call(reg.socket_path(), "query.output", ...)` and writes the
returned scrollback to stdout before the grid path starts — the composer
skips this step. Result: when you `mirror --tag` a set of long-running
sessions, each panel starts blank and only shows output produced *after*
connection. Fix: fetch `query.output` per session before spawning readers,
feed the returned bytes into each panel's vte parser so the panel paints
the existing scrollback state on first render.

## Acceptance Criteria

### Agent
- [x] `cmd_mirror_tag` accepts a `scrollback_lines: u64` parameter (matches `cmd_mirror` signature; default 100 wired through CLI)
- [x] For each successfully-connected session, `cmd_mirror_tag` calls `client::rpc_call(reg.socket_path(), "query.output", {"lines": N})` BEFORE spawning the reader task
- [x] Returned `output` string is fed byte-by-byte through the panel's `Panel::feed` (so vte parses it into the grid), not written to stdout
- [x] RPC failure on any individual session is logged to stderr but does NOT abort the whole composer (graceful degradation)
- [x] `--scrollback <N>` CLI option added to both `Mirror` enum variants and plumbed through to `cmd_mirror_tag`
- [x] Binary builds clean (`cargo build -p termlink` no new warnings)
- [x] All 10 composer tests still pass
- [x] Live smoke: spawn 2 tagged sessions, write 'hello' into each, then `mirror --tag` — both panels display 'hello' in their scrollback on first paint (verify stdout contains 'h' 'e' 'l' 'o' bytes within 2s)

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

cargo build -p termlink 2>&1 | grep -E "^(error|warning: unused)" && exit 1 || echo "build OK"
cargo test -p termlink --bin termlink mirror_grid_composer 2>&1 | tail -5 | grep -q "test result: ok"



# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-23T14:44:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1203-composer-initial-scrollback-prefetch--fe.md
- **Context:** Initial task creation

### 2026-04-23T14:46:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
