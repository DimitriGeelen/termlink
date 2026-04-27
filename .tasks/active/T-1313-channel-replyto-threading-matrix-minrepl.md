---
id: T-1313
name: "channel reply_to threading (Matrix m.in_reply_to inspired)"
description: >
  channel reply_to threading (Matrix m.in_reply_to inspired)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T13:18:02Z
last_update: 2026-04-27T13:18:02Z
date_finished: null
---

# T-1313: channel reply_to threading (Matrix m.in_reply_to inspired)

## Context

Add Matrix-style threading to the channel bus by wiring `metadata.in_reply_to`
end-to-end: CLI flag → `PendingPost` → hub envelope → subscribe surfacing +
filter. Tier-A (additive metadata field, no signature change). Builds on
T-1287 metadata routing-hint pattern. Inspired by Matrix `m.in_reply_to`
(`m.relates_to` rel_type=`m.in_reply_to`) but simplified — value is the
parent envelope's offset (u64 as decimal string) within the same topic;
cross-topic replies are out of scope for v1.

This is the first installment of the broader "agent conversation feature
using Matrix elements" thread (reactions, read-receipts, edits queued as
follow-up tasks).

## Acceptance Criteria

### Agent
- [x] `PendingPost` carries optional `metadata: BTreeMap<String,String>` with `#[serde(default)]` for backward queue-deser compat
- [x] `bus_client::post_to_params` includes `metadata` when non-empty (omitted otherwise — wire compat)
- [x] CLI `termlink channel post --reply-to <offset>` injects `metadata.in_reply_to=<offset>`
- [x] CLI `termlink channel post --metadata K=V` (repeatable) sets arbitrary metadata keys for forward extensibility
- [x] CLI `termlink channel subscribe --in-reply-to <offset>` filters server-side
- [x] CLI `termlink channel subscribe` non-JSON output prefixes replies with `↳<parent_offset>` for visual threading
- [x] Hub `handle_channel_subscribe_with` filters by `in_reply_to` symmetric to existing `conversation_id` filter
- [x] Hub test: post with metadata.in_reply_to → subscribe filtered by in_reply_to returns only matching envelope
- [x] Hub test: post without metadata still works (no envelope-shape regression)
- [x] All termlink-hub + termlink-session + termlink-cli tests pass; clippy clean

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
cargo test -p termlink-hub --lib channel:: -- --nocapture 2>&1 | tail -30
cargo test -p termlink-session --lib offline_queue::tests bus_client::tests 2>&1 | tail -20
cargo build -p termlink-cli 2>&1 | tail -10
cargo clippy -p termlink-hub -p termlink-session -p termlink-cli -- -D warnings 2>&1 | tail -10

## Decisions

### 2026-04-27 — threading lives in metadata (NOT signed)
- **Chose:** `metadata.in_reply_to=<offset>` as a routing-hint string, NOT included in `canonical_sign_bytes`.
- **Why:** Follows the T-1287 metadata pattern (conversation_id, event_type are also unsigned routing hints) and keeps the change Tier-A — empty metadata → wire shape unchanged for legacy senders. Trusted-mesh threat model already accepts unsigned metadata; making a single Matrix-style rel-type signed would have forced a versioned canonical layout for one field.
- **Rejected:** Adding `reply_to: Option<{topic, offset}>` as a new top-level signed field. Pros: cryptographically anchored; cons: requires conditional canonical layout, breaks any path that hashes the raw envelope. If the threat model later demands signed thread parents, revisit as a separate `reply_to` field with explicit canonical-version bump.

### 2026-04-27 — value is parent's offset, not (topic, offset)
- **Chose:** Single integer (decimal string in metadata) within the same topic.
- **Why:** All Matrix-equivalent use cases fit a single topic — agent conversations happen on a single channel. Cross-topic replies are rare and can be modeled as new posts that reference both topics in payload.
- **Rejected:** `<topic>:<offset>` to enable cross-topic threading. Pros: more general; cons: adds parsing complexity to every consumer + filter and we have zero current use cases.

### Live smoke evidence (2026-04-27)
```
$ termlink channel post test:t-1313-v2 --payload "are you there?"          # offset 0
$ termlink channel post test:t-1313-v2 --payload "yes, ready" --reply-to 0 # offset 1
$ termlink channel post test:t-1313-v2 --payload "(unrelated)"             # offset 2
$ termlink channel post test:t-1313-v2 --payload "what about now?" --reply-to 0  # offset 3

$ termlink channel subscribe test:t-1313-v2
[0] d1993c2c3ec44c94 chat: are you there?
[1 ↳0] d1993c2c3ec44c94 chat: yes, ready
[2] d1993c2c3ec44c94 chat: (unrelated)
[3 ↳0] d1993c2c3ec44c94 chat: what about now?

$ termlink channel subscribe test:t-1313-v2 --in-reply-to 0
[1 ↳0] d1993c2c3ec44c94 chat: yes, ready
[3 ↳0] d1993c2c3ec44c94 chat: what about now?
```

## Updates

### 2026-04-27T13:18:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1313-channel-replyto-threading-matrix-minrepl.md
- **Context:** Initial task creation
