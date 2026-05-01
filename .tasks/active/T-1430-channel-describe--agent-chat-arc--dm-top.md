---
id: T-1430
name: "channel describe — agent-chat-arc + dm:* topic self-documentation (T-1425 pick #3)"
description: >
  From T-1425 fast-forward synthesis. No protocol question — pure self-documentation. Run channel describe on agent-chat-arc with the canonical contact-protocol prose (msg_type required, identity authoritative, metadata.thread for threading, in_reply_to for replies, inbox.push deprecated). Dependent on T-1427 (whoami + binding) so the description language reflects the actual strict-reject behavior rather than aspiration. Also: scope a 'self-describe-on-create' helper for T-1429 so the auto-created dm:* topics get a description too. Trivial in scope but high in leverage — every subscriber sees the topic's own canon, no CLAUDE.md cost.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:46Z
last_update: 2026-05-01T10:05:16Z
date_finished: null
---

# T-1430: channel describe — agent-chat-arc + dm:* topic self-documentation (T-1425 pick #3)

## Context

T-1425 RFC §3.2 lists 5 protocol invariants for the chat arc; agents
encountering the topic via `channel info` should see them in-place rather
than needing to read CLAUDE.md. `cmd_channel_describe` already exists
(channel.rs:2000-2019, T-1323) emitting `msg_type=topic_metadata`
envelopes, and `channel info` reads them back via `latest_description`.
This task USES that infrastructure to canonicalise the chat-arc and
sequences the dm:* helper into T-1429.

The strict-reject AC was descoped to T-1427 — describing aspirational
behavior in the topic doc would mislead readers. Instead the description
flags it as "lands in T-1427".

## Acceptance Criteria

### Agent
- [x] Description language reflects current behavior, not aspirational. Strict-reject is flagged as "lands in T-1427" — readers see the convention as a convention until T-1427 makes it enforced
- [x] `agent-chat-arc` topic on the local hub (.107) has a description set via `channel describe` covering all 5 protocol invariants from T-1425 §3.2: msg_type required; identity authoritative via whoami match; `metadata._thread=<task-id>` for threading; `metadata.in_reply_to=<offset>` for replies; deprecation note for inbox.push (see T-1166). Set 2026-05-01T10:05Z, offset=17
- [x] Description is readable via `termlink channel info agent-chat-arc` — verified, output shows "Description: Fleet-wide agent coordination channel..."
- [x] Description text ≤ 500 chars but covers all 5 invariants — actual 334 chars, all 3 grep checks pass (msg_type, deprecated/inbox.push, in_reply_to/thread)
- [x] **Shipped via T-1429.5 (2026-05-01T11:17Z):** `dm:<a>:<b>` auto-creation now self-describes idempotently on FIRST create only. Implementation: hub-side `channel.create` returns `created: bool`, CLI `ensure_topic` reads it, `cmd_channel_dm` posts a topic_metadata envelope iff `created=true`. Verified live: a brand-new dm topic auto-emits "Direct messages between sender_id `<a>` and `<b>`. Same protocol as `agent-chat-arc`. Created by `termlink agent contact` (or `channel dm`) on first use." — visible in `channel info`. Pre-existing dm topics correctly do NOT get re-described (no bloat). Pre-T-1429.5 hubs return no `created` field; clients conservatively treat that as `false`, skipping describe — old fleets continue to work, just without the new self-doc
- [x] Existing topic descriptions (other than `agent-chat-arc` and `dm:*`) are not modified by this task — surgical scope confirmed; only one `channel describe` invocation, on `agent-chat-arc`

### Human
- [ ] [REVIEW] Verify topic self-doc is discoverable from a fresh agent's perspective
  **Steps:**
  1. `termlink channel info agent-chat-arc | head -20`
  2. From a peer with `--hub 192.168.10.107:9100`
  3. Eyeball: would a vendored agent encountering this topic for the first time know what to do without external lookup?
  **Expected:** description is present, complete, answers the "what + how" question in-place
  **If not:** propose wording fix in Updates and re-apply via `channel describe`

## Verification

target/release/termlink channel info agent-chat-arc 2>&1 | grep -qi "msg_type"
target/release/termlink channel info agent-chat-arc 2>&1 | grep -qi "deprecated\|inbox.push"
target/release/termlink channel info agent-chat-arc 2>&1 | grep -qi "in_reply_to\|thread"

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

### 2026-05-01T07:02:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1430-channel-describe--agent-chat-arc--dm-top.md
- **Context:** Initial task creation

### 2026-05-01T10:05:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-01T10:05Z — description-shipped [agent autonomous]
- **Action:** Set canonical description on `agent-chat-arc` via `termlink channel describe`. 334 chars covering msg_type, identity-via-whoami, _thread, in_reply_to, inbox.push deprecation, channel.subscribe.
- **Posted:** offset=17, ts=1777629920658
- **Verification:** all three `grep -qi` checks (msg_type, deprecated/inbox.push, in_reply_to/thread) pass against `channel info` output
- **Deferred:** dm:* self-describe helper migrates into T-1429 (the verb doesn't exist yet); 5/6 agent ACs ticked
- **Owner:** unchanged (human) — closure pending T-1429 ship and human REVIEW
