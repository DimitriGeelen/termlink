---
id: T-1374
name: "channel reactions-on <topic> <offset> — per-message reaction aggregate (Matrix annotation rollup)"
description: >
  channel reactions-on <topic> <offset> — per-message reaction aggregate (Matrix annotation rollup)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T12:42:56Z
last_update: 2026-04-28T12:56:12Z
date_finished: 2026-04-28T12:56:12Z
---

# T-1374: channel reactions-on <topic> <offset> — per-message reaction aggregate (Matrix annotation rollup)

## Context

Three reaction views exist already: `subscribe --aggregate-reactions` (inline, live), `emoji-stats` (T-1359, topic-wide tally), `reactions-of` (T-1362, per-sender). Missing: per-target rollup — "how is THIS message being received?" Matrix annotation API equivalent for a specific event.

## Acceptance Criteria

### Agent
- [x] `pub fn compute_reactions_on(envelopes: &[Value], target_offset: u64) -> Vec<ReactionsOnRow>` — walks the topic, filters `msg_type=reaction` with `metadata.in_reply_to == target_offset`, drops redacted reactions, groups by emoji, returns one row per emoji
- [x] `ReactionsOnRow { emoji, count, senders: Vec<String> }` (senders deduplicated, sorted asc); to_json
- [x] CLI: `ChannelAction::ReactionsOn { topic, offset, hub, json }`
- [x] main.rs dispatch
- [x] At least 4 unit tests: 2 emojis with different counts → sort desc; same sender twice with same emoji counts once in `senders`; redacted reaction excluded; non-target reactions excluded
- [x] cargo test passes; clippy clean
- [x] e2e step 46: 3 reactions on offset 0 (alice 👍, bob 👍, alice 🎉), 1 reaction on offset 1 → reactions-on 0 returns 2 rows (👍 count=2 senders=[alice,bob], 🎉 count=1 senders=[alice]); JSON shape

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

cargo test -p termlink --bins --quiet 2>&1 | tail -3 | grep -q "test result: ok"
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -2 | grep -qE "(warning|error): 0|^$|^\s*Finished"
bash tests/e2e/agent-conversation.sh 2>&1 | grep -q "END-TO-END WALKTHROUGH PASSED"

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

### 2026-04-28T12:42:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1374-channel-reactions-on-topic-offset--per-m.md
- **Context:** Initial task creation

### 2026-04-28T12:56:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
