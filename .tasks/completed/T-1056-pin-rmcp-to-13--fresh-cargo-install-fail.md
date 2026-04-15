---
id: T-1056
name: "pin rmcp to ~1.3 — fresh cargo install fails on 1.4+ private tool_router"
description: >
  pin rmcp to ~1.3 — fresh cargo install fails on 1.4+ private tool_router

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T20:30:02Z
last_update: 2026-04-15T13:36:15Z
date_finished: 2026-04-15T13:36:15Z
---

# T-1056: pin rmcp to ~1.3 — fresh cargo install fails on 1.4+ private tool_router

## Context

User ran `cargo install --git https://github.com/DimitriGeelen/termlink.git termlink --force`
and got a compile error in `crates/termlink-mcp/src/server.rs:16`:
`associated function 'tool_router' is private`.

Root cause: `crates/termlink-mcp/Cargo.toml` declared `rmcp = { version = "1.3", ... }`.
A caret requirement of `"1.3"` matches `^1.3` — anything `>=1.3.0, <2.0.0`. So a
fresh resolve (no Cargo.lock) picks the latest rmcp on crates.io, which in
1.4+ made `tool_router` private.

Our repo's committed `Cargo.lock` pins `rmcp = 1.3.0` — that's why local
builds (and CI that respects the lockfile) pass. But `cargo install --git`
ignores `Cargo.lock` by default, so external consumers installing from git
get the latest rmcp and break.

Fix: change `"1.3"` → `"~1.3"` in both rmcp lines of `termlink-mcp/Cargo.toml`.
Tilde allows 1.3.x patch releases but blocks 1.4+.

## Acceptance Criteria

### Agent
- [x] Both `rmcp = { version = "1.3", ... }` lines in `crates/termlink-mcp/Cargo.toml` changed to `~1.3`
- [x] `cargo update -p rmcp` confirms the resolved version stays at 1.3.0 (not 1.4+)
- [x] `cargo build` (full workspace) compiles clean
- [x] `cargo test -p termlink --bin termlink` passes (189 tests)
- [x] Cargo.lock unchanged (rmcp still at 1.3.0 with no transitive churn)
- [x] G-019 check: this is a post-fix root cause escalation — the framework didn't flag the Cargo.toml drift-allowance even though it was a known risk class. Noted in commit for future audit.

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

cargo build 2>&1 | tail -3
cargo test -p termlink --bin termlink 2>&1 | grep -E "189 passed"
grep -E '^rmcp\s*=' crates/termlink-mcp/Cargo.toml | grep -qE '"\~1\.3"'

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

### 2026-04-14T20:30:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1056-pin-rmcp-to-13--fresh-cargo-install-fail.md
- **Context:** Initial task creation

### 2026-04-15T13:36:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
