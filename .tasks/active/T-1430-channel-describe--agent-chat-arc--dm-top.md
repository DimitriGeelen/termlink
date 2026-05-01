---
id: T-1430
name: "channel describe — agent-chat-arc + dm:* topic self-documentation (T-1425 pick #3)"
description: >
  From T-1425 fast-forward synthesis. No protocol question — pure self-documentation. Run channel describe on agent-chat-arc with the canonical contact-protocol prose (msg_type required, identity authoritative, metadata.thread for threading, in_reply_to for replies, inbox.push deprecated). Dependent on T-1427 (whoami + binding) so the description language reflects the actual strict-reject behavior rather than aspiration. Also: scope a 'self-describe-on-create' helper for T-1429 so the auto-created dm:* topics get a description too. Trivial in scope but high in leverage — every subscriber sees the topic's own canon, no CLAUDE.md cost.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:46Z
last_update: 2026-05-01T07:02:46Z
date_finished: null
---

# T-1430: channel describe — agent-chat-arc + dm:* topic self-documentation (T-1425 pick #3)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] T-1427 (whoami + identity binding) has shipped, or this task is sequenced to land in the same release — description language must reflect actual strict-reject behavior, not aspirational
- [ ] `agent-chat-arc` topic on the local hub (.107) has a description set via `channel describe` covering all 5 protocol invariants from T-1425 §3.2: msg_type required; identity (sender_id) authoritative; `metadata.from` must match `whoami`; `metadata.thread=<task-id>` for threading; `metadata.in_reply_to=<offset>` for replies; deprecation note for inbox.push
- [ ] Description is readable via `termlink channel info agent-chat-arc`
- [ ] Description text ≤ 500 chars but covers all 5 invariants
- [ ] T-1429's `cmd_agent_contact` topic-creation helper self-describes any `dm:<a>:<b>` it auto-creates: "Direct messages between sender_id `<a>` and `<b>`. Same protocol as `agent-chat-arc`. Created by `termlink agent contact` on first use." — applied idempotently (re-apply on existing topic is a no-op)
- [ ] Existing topic descriptions (other than `agent-chat-arc` and `dm:*`) are not modified by this task — surgical scope

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
