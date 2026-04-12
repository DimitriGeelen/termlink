---
id: T-990
name: "Fix MCP crate release build — tool_router private function error (E0624)"
description: >
  Fix MCP crate release build — tool_router private function error (E0624)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T22:16:58Z
last_update: 2026-04-12T22:24:55Z
date_finished: 2026-04-12T22:24:55Z
---

# T-990: Fix MCP crate release build — tool_router private function error (E0624)

## Context

`cargo install --path crates/termlink-cli` fails with E0624 (tool_router is private) because
without `--locked`, cargo resolves fresh dependency versions. A newer transitive dep changes
rmcp macro-generated visibility. Fix: pin rmcp to exact version in Cargo.toml.

## Acceptance Criteria

### Agent
- [x] Root cause identified: `cargo install` resolves fresh deps, transitive dep change breaks rmcp macro visibility
- [x] Fix: pin rmcp to `1.3` (matching Cargo.lock) and require `--locked` for install
- [x] `cargo install --path crates/termlink-cli --force --locked` succeeds
- [x] Updated CLAUDE.md versioning section to document `--locked` requirement

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

### 2026-04-12T22:16:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-990-fix-mcp-crate-release-build--toolrouter-.md
- **Context:** Initial task creation

### 2026-04-12T22:24:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Pinned rmcp, --locked install works
