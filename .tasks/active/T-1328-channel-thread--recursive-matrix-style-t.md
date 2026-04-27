---
id: T-1328
name: "channel thread — recursive Matrix-style thread view (root + descendants)"
description: >
  channel thread — recursive Matrix-style thread view (root + descendants)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T15:58:55Z
last_update: 2026-04-27T15:58:55Z
date_finished: null
---

# T-1328: channel thread — recursive Matrix-style thread view (root + descendants)

## Context

T-1313 added `--in-reply-to <offset>` server filter for finding direct replies
to a parent. But threads are usually deeper — replies-to-replies form a tree.
Matrix's classic thread view lays this out as: root post + all descendants,
indented by depth. Implement as a one-shot read that walks the topic once,
builds parent→children map, and renders the subtree rooted at `<root>`.
No new metadata, no hub change — pure renderer on top of existing data.

## Acceptance Criteria

### Agent
- [x] `ChannelAction::Thread { topic, root, hub, json }` added to cli.rs
- [x] `cmd_channel_thread` walks the topic via channel.subscribe (page=1000),
      builds a `parent → children` HashMap, then DFS-renders the subtree from `<root>`
- [x] Each line indented `  ` per depth level: `  [N] sender msg_type: payload`
- [x] Root not found → friendly error: `Topic '<topic>' has no message at offset <root>`
- [x] JSON mode emits `{topic, root, tree: [{offset, sender, msg_type, payload, depth, children: [...]}]}`
- [x] Pure helper `build_thread(parents_map, root) -> Vec<(u64, usize)>` returns
      pre-order DFS traversal as (offset, depth) pairs
- [x] Unit test `build_thread_orders_dfs_with_depth` — tree (0→1, 0→2, 1→3) rooted at 0
      returns `[(0,0),(1,1),(3,2),(2,1)]`
- [x] `cargo test -p termlink --bins` + clippy clean
- [x] e2e walkthrough adds a step that verifies thread output for the existing
      reply chain
- [x] `agent-conversations.md` gains a "Thread view" section

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
cargo test -p termlink --bins build_thread
cargo clippy -p termlink --all-targets -- -D warnings

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

### 2026-04-27T15:58:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1328-channel-thread--recursive-matrix-style-t.md
- **Context:** Initial task creation
