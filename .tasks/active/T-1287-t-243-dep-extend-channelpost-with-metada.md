---
id: T-1287
name: "T-243 dep: Extend channel.post with metadata.conversation_id + metadata.event_type"
description: >
  Per T-243 inception synthesis: one-field code change in channel.post params. Optional metadata.conversation_id (string) — scope events to a conversation. Optional metadata.event_type (turn|typing|receipt|presence|member) — routing/filtering hint, not enforcement. Enables convention-layer multi-turn dialog without new typed-method namespace. Independently testable; can land before or after dialog.heartbeat.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-243, channel, protocol]
components: []
related_tasks: []
created: 2026-04-26T09:32:02Z
last_update: 2026-04-26T09:51:43Z
date_finished: null
---

# T-1287: T-243 dep: Extend channel.post with metadata.conversation_id + metadata.event_type

## Context

Per T-243 inception synthesis (Agent C minimal-surface path): one-field code change in channel.post params adds `metadata.conversation_id` + `metadata.event_type` as optional fields. Routing/filtering hint, not enforcement. Enables convention-layer multi-turn dialog without new typed-method namespace.

## Design decision: unsigned metadata

Metadata is treated as **routing-hint-only**, NOT included in canonical signed bytes. Per Agent C's framing: "routing hint, not enforcement." Trusted-mesh threat model — TLS + hub.secret protect transport; no attacker model where metadata rewrite mid-flight is a concern. Future task can promote to signed metadata if threat model expands.

Backwards compat: Envelope's metadata field uses `#[serde(default)]` — existing JSON envelopes deserialize unchanged. Old signers verify against new posts identically (metadata not in signed bytes). Old subscribers ignore the metadata field.

## Acceptance Criteria

### Agent
- [x] Extend `Envelope` struct with `metadata: BTreeMap<String, String>` — done in envelope.rs, `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]` for backwards-compat
- [x] `handle_channel_post` reads optional `metadata` object (string→string) from params and stores in Envelope — channel.rs
- [x] envelope_to_json serializes metadata back to wire format — channel.rs (omitted for empty metadata to preserve legacy wire format)
- [x] Updated 2 hub-side mirror callers (broadcast, inbox.deposit) with `metadata: Default::default()` — channel.rs
- [x] Updated bus tests env() helper with empty BTreeMap — lib.rs
- [x] `handle_channel_subscribe` accepts optional `conversation_id` filter — applies BEFORE limit, advances last_offset over skipped records so next_cursor moves past them (channel.rs `handle_channel_subscribe_with`)
- [x] Test: post with metadata, subscribe without filter → metadata intact (`post_with_metadata_round_trips_through_subscribe`)
- [x] Test: post 3 messages with mixed conversation_ids, subscribe with filter → only matching, in offset order, cursor advances past skipped (`subscribe_with_conversation_id_filters_and_advances_cursor`)
- [x] Test: post without metadata → envelope JSON omits metadata key entirely (`subscribe_without_metadata_omits_field_on_wire`)
- [x] cargo test passes for `termlink-bus` (31) and `termlink-hub` (227) — full workspace test suite green

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

### 2026-04-26T09:32:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1287-t-243-dep-extend-channelpost-with-metada.md
- **Context:** Initial task creation

### 2026-04-26T09:48:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
