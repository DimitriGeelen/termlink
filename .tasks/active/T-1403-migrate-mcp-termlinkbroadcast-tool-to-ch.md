---
id: T-1403
name: "Migrate MCP termlink_broadcast tool to channel.post(broadcast:global)"
description: >
  Migrate MCP termlink_broadcast tool to channel.post(broadcast:global)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T08:52:12Z
last_update: 2026-04-29T08:52:12Z
date_finished: null
---

# T-1403: Migrate MCP termlink_broadcast tool to channel.post(broadcast:global)

## Context

T-1401 migrated the CLI `cmd_broadcast` to `channel.post(broadcast:global)`.
The MCP `termlink_broadcast` tool at `crates/termlink-mcp/src/tools.rs:1815`
was missed — it still calls `event.broadcast` directly. Same pattern,
same fallback logic. Surfaced during T-1166 in-repo legacy-caller audit.

## Acceptance Criteria

### Agent
- [x] `termlink_broadcast` MCP tool prefers `channel.post(broadcast:global)` when `targets.is_empty()`; falls back to legacy `event.broadcast` on any failure
- [x] When `!targets.is_empty()`, behavior is unchanged — still calls `event.broadcast`
- [x] Channel post envelope mirrors hub-side T-1162 mirror shape: `topic="broadcast:global"`, `msg_type=<original_topic>`, `payload_b64=<JSON-serialized payload>`, signed with caller's identity
- [x] JSON output preserves legacy keys (`ok`, `topic`, `targeted`, `succeeded`, `failed`) PLUS adds `channel_topic`, `offset` for the channel-post path
- [x] cargo build / cargo clippy clean for `termlink-mcp`

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

cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished"
cargo clippy -p termlink-mcp --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
grep -q "broadcast:global" crates/termlink-mcp/src/tools.rs
grep -q "channel.post" crates/termlink-mcp/src/tools.rs

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

### 2026-04-29T08:52:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1403-migrate-mcp-termlinkbroadcast-tool-to-ch.md
- **Context:** Initial task creation
