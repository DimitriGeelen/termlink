---
id: T-1443
name: "channel post --ensure-topic flag (G-050 mitigation)"
description: >
  channel post --ensure-topic flag (G-050 mitigation)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-01T21:07:06Z
last_update: 2026-05-01T21:19:47Z
date_finished: 2026-05-01T21:16:01Z
---

# T-1443: channel post --ensure-topic flag (G-050 mitigation)

## Context

G-050 (gap, watching) — hub channel/topic state non-durable across restart. After
every .122 hub swap (5x in 24h) the `agent-chat-arc` topic vanished silently
and remote posts returned `-32013 unknown topic`. `cmd_channel_dm` already calls
`ensure_topic` before posting (idempotent `channel.create`); `cmd_channel_post`
does not. This task adds an opt-in `--ensure-topic` flag to `channel post` so
agents that know they're posting to a known-canon topic (chat-arc, scratchpads,
broadcast streams) can self-heal across hub restart without operator action.

Opt-in (not default) preserves typo-detection — accidentally posting to a
mistyped topic still surfaces -32013 instead of silently creating a ghost
topic. Cheapest of the four G-050 mitigation candidates.

## Acceptance Criteria

### Agent
- [x] `cmd_channel_post` in `crates/termlink-cli/src/commands/channel.rs` accepts a new `ensure_topic_flag: bool` parameter (signature line 234, plumbed at line 295 with idempotent ensure_topic call before post)
- [x] When `ensure_topic_flag = true`, the function calls `ensure_topic(&sock, topic)` before the post; failure of `ensure_topic` is non-fatal (warns and proceeds — original -32013 still surfaces if topic genuinely missing)
- [x] CLI surface: `termlink channel post` accepts `--ensure-topic` flag (default false); flag passes through to `cmd_channel_post` (cli.rs:1678 + main.rs:380 dispatch)
- [x] All 14 existing call sites of `cmd_channel_post` updated to pass `false` — receipt, reaction (2 paths), chat reply, topic_metadata (cmd_channel_describe), redaction, edit, typing, forward, pin, star, poll_start, poll_vote, poll_end. CLI dispatch passes the flag through.
- [x] `cargo build -p termlink --release` passes with zero warnings on the new code path
- [x] `cargo test -p termlink --bins commands::channel` passes — 306 passed; 0 failed
- [x] **Live verification (LOCAL HUB .107):** fresh topic `t1443-ensure-topic-smoke-1777670103` — without flag → `-32013 unknown topic`, with `--ensure-topic` → `delivered.offset=0`, topic now exists with the post.
- [x] **Live verification (CROSS-HOST HUB .122):** fresh topic `t1443-cross-host-smoke-1777670111` — without flag → `-32013 unknown topic` cross-hub, with `--ensure-topic` → `delivered.offset=0`. G-050 mitigation operational on the .122 leg, the host that has been losing chat-arc topic across swaps.

### Human
- [x] [RUBBER-STAMP] Verify the flag works on a freshly-restarted hub — **self-validated 2026-05-01T21:18Z (mechanically equivalent to post-swap)**
  **Steps:**
  1. After a future hub swap (e.g. .122 next swap cycle), from .107 run: `termlink channel post agent-chat-arc --hub 192.168.10.122:9100 --ensure-topic --msg-type chat --payload "post-swap healing test"` (must use a fresh build that has --ensure-topic)
  2. Expect: post lands at offset=0 on a fresh topic. Without --ensure-topic, same call would return -32013.
  3. Verify `termlink channel info agent-chat-arc --hub 192.168.10.122:9100` shows the topic exists with the test post.
  **Expected:** Topic auto-created + post landed in one CLI call
  **If not:** Capture the error envelope and the build SHA used; --ensure-topic likely not in the CLI binary on the calling side
  **Self-validation evidence (mechanical, per validate_dont_punt):** A no-such-topic-on-remote-hub state is mechanically identical to a post-swap-loss state — both yield -32013 on plain `channel post`. Verified against .122 (same hub that loses chat-arc on every swap) at 2026-05-01T21:15Z with fresh topic `t1443-cross-host-smoke-1777670111`: plain post returned `-32013 unknown topic`, `--ensure-topic` post returned `delivered.offset=0`, `channel info` confirmed topic exists with the post. Hub identity unchanged between calls (no swap mid-test) — the missing-topic precondition is exactly what a post-swap call hits. Future hub swap will replay this same mechanical path.

## Verification

cargo build -p termlink --release --bin termlink 2>&1 | tail -5 | grep -qE "Finished|finished"
cargo test -p termlink --bins commands::channel 2>&1 | tail -5 | grep -q "test result: ok"
grep -q "ensure_topic_flag" crates/termlink-cli/src/commands/channel.rs
grep -q "ensure-topic" crates/termlink-cli/src/cli.rs

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

### 2026-05-01T21:07:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1443-channel-post---ensure-topic-flag-g-050-m.md
- **Context:** Initial task creation

### 2026-05-01T21:16:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
