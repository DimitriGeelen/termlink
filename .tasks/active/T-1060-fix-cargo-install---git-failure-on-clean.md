---
id: T-1060
name: "Fix cargo install --git failure on clean hosts — forward-compat rmcp-macros tool_router vis"
description: >
  Fix cargo install --git failure on clean hosts — forward-compat rmcp-macros tool_router vis

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T21:42:54Z
last_update: 2026-04-14T21:42:54Z
date_finished: null
---

# T-1060: Fix cargo install --git failure on clean hosts — forward-compat rmcp-macros tool_router vis

## Context

T-1056 pinned `rmcp = "~1.3"` to unblock `cargo install --git`. That fixed
local builds (which respect Cargo.lock) but users hitting `cargo install
--git <url> termlink` on hosts without our Cargo.lock STILL fail with
`E0624: associated function tool_router is private`.

Root cause (diagnosed in this session):
- `rmcp 1.3.0` has `rmcp-macros = "1.3.0"` in its own Cargo.toml. That's a
  CARET requirement (`^1.3.0` = `>=1.3.0, <2.0.0`) — it allows 1.4+.
- `cargo install --git` without `--locked` ignores our committed
  Cargo.lock and does a FRESH resolve. The resolver picks rmcp=1.3.0
  (our `~1.3` pin holds) but rmcp-macros=**1.4.0** (latest compatible).
- rmcp-macros 1.4.0's `#[tool_handler]` expands to
  `Self::tool_router()` (FUNCTION call). rmcp-macros 1.3.0's version
  used `self.tool_router` (FIELD access).
- Both versions of `#[tool_router]` default the generated method to
  PRIVATE unless `vis = "..."` is passed. With 1.4.0's function-call
  expansion, the `impl ServerHandler` block in server.rs can't reach
  the private method in the `impl TermLinkTools` block in tools.rs.

Fix: pass `vis = "pub(crate)"` to `#[tool_router]` in tools.rs. One
line. The resulting method is still crate-private (no API surface
change) but reachable from other impl blocks in the same crate. Works
under BOTH rmcp-macros 1.3.x (field-access path) and 1.4.x+
(function-call path).

## Acceptance Criteria

### Agent
- [ ] `crates/termlink-mcp/src/tools.rs` changes `#[tool_router]` to `#[tool_router(vis = "pub(crate)")]` with a comment block explaining WHY for future maintainers
- [ ] `cargo clean && cargo build -p termlink-mcp` passes (rmcp-macros 1.3.0 path)
- [ ] Fresh `cargo install --git <onedev-or-github-url> termlink --force` WITHOUT `--locked` succeeds (this is the failure mode the fix targets — reproduces the user's exact scenario)
- [ ] `cargo test -p termlink --bin termlink` passes (189 tests)
- [ ] No clippy warnings
- [ ] G-019 post-fix root cause: register a gap concern — `cargo install --git` (no `--locked`) is the documented external install path yet has no CI coverage. Reason this was missed: CI runs with lockfile, which the resolver actually respects. External-consumer install is in a blind spot.

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

grep -q 'tool_router(vis = "pub(crate)")' crates/termlink-mcp/src/tools.rs
cargo build -p termlink-mcp 2>&1 | tail -1 | grep -q "Finished"

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

### 2026-04-14T21:42:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1060-fix-cargo-install---git-failure-on-clean.md
- **Context:** Initial task creation
