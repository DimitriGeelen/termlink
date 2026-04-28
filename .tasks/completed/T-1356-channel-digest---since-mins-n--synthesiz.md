---
id: T-1356
name: "channel digest --since-mins N — synthesized topic activity highlights"
description: >
  channel digest --since-mins N — synthesized topic activity highlights

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T06:43:25Z
last_update: 2026-04-28T06:55:12Z
date_finished: 2026-04-28T06:55:12Z
---

# T-1356: channel digest --since-mins N — synthesized topic activity highlights

## Context

Synthesized "what happened recently" view of a topic. Walks the topic, applies a `--since-mins N` time filter (or `--since MS` absolute), and produces a compact summary:
- post count, distinct senders
- top 3 senders by post count
- reactions summary (top 3 emoji × count)
- pin/star count delta in window
- forward count
- last 3 chat lines (plain text, no metadata)

Distinct from `channel info` (which is a topic-level synthesis, no time window) and from `channel stats` (which is global counts). Digest is for "I was away — what did I miss?".

## Acceptance Criteria

### Agent
- [x] `channel digest <topic> --since-mins N` walks the topic and renders a digest scoped to envelopes with ts within last N minutes
- [x] `--since MS` accepts an absolute lower bound (mutually exclusive with `--since-mins`)
- [x] Output sections: posts, senders, top reactions, pins added/removed, forwards in, last 3 chat snippets
- [x] `--json` returns structured object with all sections
- [x] Pure helper `compute_digest(envelopes, since_ms)` unit-tested across: empty topic, no envelopes in window, mixed types, multiple senders, top-3 truncation, edits/redactions excluded from chat snippets
- [x] e2e step covering the lifecycle
- [x] cargo build, test, clippy --all-targets -- -D warnings all pass

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
cargo test -p termlink --bins --quiet 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5

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

### 2026-04-28T06:43:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1356-channel-digest---since-mins-n--synthesiz.md
- **Context:** Initial task creation

### 2026-04-28T06:55:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
