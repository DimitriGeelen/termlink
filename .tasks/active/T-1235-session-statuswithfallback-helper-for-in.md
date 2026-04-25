---
id: T-1235
name: "Session: status_with_fallback helper for inbox.status migration (T-1229b)"
description: >
  Add status_with_fallback{,_with_client} helper in inbox_channel.rs that mirrors list_with_fallback (T-1231) for inbox.status. Probes hub.capabilities; uses channel.list(prefix=inbox:) when channel.* is supported, sums per-topic counts; falls back to legacy inbox.status on -32601. Critical dependency for T-1229c/d/e/f/g call-site migrations.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1229, T-1155, bus, channel, session]
components: []
related_tasks: []
created: 2026-04-25T10:19:41Z
last_update: 2026-04-25T10:20:12Z
date_finished: null
---

# T-1235: Session: status_with_fallback helper for inbox.status migration (T-1229b)

## Context

T-1229b per docs/reports/T-1229-inception.md. Critical helper for the 5 inbox.status call-site migrations (T-1229c/d/e/f/g). Mirrors the T-1231 list_with_fallback pattern: probe hub.capabilities, dispatch to channel.list(prefix="inbox:") + sum counts when channel.* is supported, fall back to legacy inbox.status on -32601. Two entry points: (a) `status_with_fallback(addr, ...)` opens its own client; (b) `status_with_fallback_with_client(client, ...)` for callers (CLI/MCP remote) holding an authenticated Client.

## Acceptance Criteria

### Agent
- [x] `InboxStatus { total_transfers: u64, targets: Vec<InboxStatusTarget> }` + `InboxStatusTarget { target: String, pending: u64 }` types added to `crates/termlink-session/src/inbox_channel.rs`. serde-Serialize so callers can re-emit unchanged.
- [x] `pub async fn status_with_fallback(addr, cache, ctx) -> io::Result<InboxStatus>` opens a `Client::connect_addr` and delegates to `status_with_fallback_with_client`.
- [x] `pub async fn status_with_fallback_with_client(client, host_port, cache, ctx) -> io::Result<InboxStatus>` does the probe + dispatch. Uses cap-cache; method-not-found on `channel.list` → flag legacy + warn-once + fall back; warn-once "channel.list" / "inbox.status" tracking matches T-1225 pattern.
- [x] Channel path: call `channel.list({prefix: "inbox:"})`, parse `topics: [{name, count}]`, strip `inbox:` prefix from name to get target, sum counts → InboxStatus.
- [x] Legacy path: call `inbox.status` (no params), parse `{total_transfers, targets: [{target, pending}]}`.
- [x] Unit test 1 (channel path): set cap-cache to include `channel.list`, drive a fake transport that returns 2 inbox: topics with counts 3 and 1; assert InboxStatus.total_transfers=4, targets has 2 entries.
- [x] Unit test 2 (legacy fallback): set cap-cache to NOT include `channel.list`; drive fake transport returning legacy `inbox.status` shape; assert InboxStatus matches.
- [x] Unit test 3 (method-not-found triggers fallback): cap-cache claims channel.list supported but transport returns -32601; assert helper transparently falls back to inbox.status and flags peer legacy-only.
- [x] `cargo build -p termlink-session` clean (0 new warnings).
- [x] `cargo test -p termlink-session inbox_channel::tests::status` passes.

## Verification

cargo build -p termlink-session 2>&1 | tail -5
cargo test -p termlink-session inbox_channel 2>&1 | tail -15

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

### 2026-04-25T10:19:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1235-session-statuswithfallback-helper-for-in.md
- **Context:** Initial task creation

### 2026-04-25T10:20:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
