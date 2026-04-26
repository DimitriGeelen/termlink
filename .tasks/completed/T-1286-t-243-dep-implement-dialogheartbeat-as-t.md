---
id: T-1286
name: "T-243 dep: Implement dialog.heartbeat as typed RPC (or hub-tracked invariant)"
description: >
  The single must-be-infrastructure piece per T-243 inception (Agent B reframing + Agent C's own conversion trigger). Heartbeat: every ~5s during processing, responding agent emits lightweight signal on conversation channel. Two jobs at once: (a) resets caller's timeout clock — prevents 30s-timeout death of long LLM turns; (b) typing indicator emerges as side effect. Hub tracks last-heartbeat per (conversation_id, agent_id), evicts stale agents, resets request timeouts. Depends on channel-audit child task being clean.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-243, heartbeat, reliability]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-protocol/src/control.rs]
related_tasks: []
created: 2026-04-26T09:31:58Z
last_update: 2026-04-26T11:03:07Z
date_finished: 2026-04-26T11:03:07Z
---

# T-1286: T-243 dep: Implement dialog.heartbeat as typed RPC (or hub-tracked invariant)

## Context

Per T-243 inception synthesis (Agent B reframing): heartbeat is **must-be-infrastructure**, not cosmetic-typing-indicator. Without it, the 30s default timeout kills long LLM-tool-use turns.

With T-1287 (metadata) and T-1289 (long-poll) landed, the **convention** "post typing/heartbeat on conversation channel every ~5s" already works for keepalive (the long-poll subscriber wakes on the post and resets its window). What's missing is **server-side awareness** of who's alive in a conversation — without that, presence is "did I happen to see a recent post," and a fresh subscriber has no way to ask "who's currently active?"

This task adds the minimum infrastructure piece: hub-side passive presence tracking by observing metadata on every channel.post, plus a typed `dialog.presence(conversation_id)` query RPC.

**Scope fence:** No new signing path, no client-side helper SDK, no eviction thread. Eviction is implicit (callers compare last_seen against now). "Resets request timeouts" is deferred to a future task if/when requests with timeout-reset semantics emerge.

## Design

- Hub state: `Arc<RwLock<HashMap<(String, String), i64>>>` — keys are `(conversation_id, agent_id)`, values are `last_seen_unix_ms`
- `handle_channel_post_with` updates the tracker after successful append when `env.metadata.conversation_id` is present (uses `env.sender_id` as agent_id, `env.ts_unix_ms` as last_seen)
- New RPC `dialog.presence(conversation_id) → {presences: [{agent_id, last_seen_ms}]}`, sorted by agent_id for deterministic output
- Unknown conversation_id returns `{presences: []}` (not an error — presence is observational)

## Acceptance Criteria

### Agent
- [x] Hub gains `PresenceTracker` carrying `(conversation_id, agent_id) → last_seen_ms` map behind `RwLock`, process-global via `OnceLock`. (channel.rs, `pub(crate) struct PresenceTracker`)
- [x] `handle_channel_post_with` records presence after `bus.post` succeeds when `env.metadata.conversation_id` is present. Posts without conversation_id are no-op.
- [x] New JSON-RPC method `dialog.presence` registered in `route()` dispatch (router.rs) and listed in `hub.capabilities` (sorted, unique). Param: `conversation_id` required. Result: `{presences: [{agent_id, last_seen_ms}, ...]}` sorted by `agent_id`. Method constant: `control::method::DIALOG_PRESENCE`.
- [x] Unknown conversation_id returns `{presences: []}` (not an error).
- [x] Test: 3 posts on cid=t1286-c1 with alice/bob/alice → presence returns 2 entries, sorted, alice's last_seen_ms = LATER post's ts (overwrites earlier). (`dialog_presence_tracks_senders_per_conversation`)
- [x] Test: posts with no conversation_id metadata don't appear (`dialog_presence_ignores_posts_without_conversation_id`)
- [x] Test: unknown conversation_id returns empty list, not error (`dialog_presence_unknown_conversation_returns_empty`)
- [x] Test: missing conversation_id param → -32602 (`dialog_presence_missing_conversation_id_is_invalid_params`)
- [x] cargo test passes: 31 (termlink-bus) + 231 (termlink-hub). 0 failed.

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

### 2026-04-26T09:31:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1286-t-243-dep-implement-dialogheartbeat-as-t.md
- **Context:** Initial task creation

### 2026-04-26T10:59:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-26T11:03:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
