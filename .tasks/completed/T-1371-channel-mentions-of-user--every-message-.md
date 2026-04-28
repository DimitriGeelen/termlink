---
id: T-1371
name: "channel mentions-of <user> — every message that @-mentions a target user (any author)"
description: >
  channel mentions-of <user> — every message that @-mentions a target user (any author)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T11:57:27Z
last_update: 2026-04-28T12:12:02Z
date_finished: 2026-04-28T12:12:02Z
---

# T-1371: channel mentions-of <user> — every message that @-mentions a target user (any author)

## Context

Per-topic reverse view of `--mention` posting (T-1325 / T-1333). T-1339 (`channel mentions`) is cross-topic and fingerprint-locked to the caller. `mentions-of <topic> <user>` answers "who pinged Alice in this topic, and where?" for any user, regardless of caller identity. Reuses pure helpers `mentions_match` (T-1333) and the `metadata.mentions` extractor.

## Acceptance Criteria

### Agent
- [x] `pub fn compute_mentions_of(envelopes: &[Value], user: &str) -> Vec<MentionsOfRow>` in channel.rs returns rows where `metadata.mentions` CSV `mentions_match`es the user, msg_type is NOT in UNREAD_META_TYPES, and envelope is not redacted; sorted by mention offset desc
- [x] `MentionsOfRow { mention_offset, sender_id, payload, mentions_csv, ts_ms }` with to_json
- [x] CLI: `ChannelAction::MentionsOf { topic, user, hub, json }` — user is required positional
- [x] main.rs dispatch wired
- [x] At least 4 unit tests: happy path desc; wildcard `*` mention matches any specific user; redacted excluded; non-mentioning posts excluded
- [x] cargo test passes; clippy clean
- [x] e2e step 43 exercises positive (Bob pings Alice 2×), negative (no mention of Carol), wildcard (`@room` style ping matches any user), JSON shape

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

### 2026-04-28T11:57:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1371-channel-mentions-of-user--every-message-.md
- **Context:** Initial task creation

### 2026-04-28T12:12:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
