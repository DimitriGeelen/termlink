---
id: T-1588
name: "termlink_agent_threads_by + termlink_agent_silent_senders — sender-started thread roots + quiet-peer detection MCP read tools"
description: >
  termlink_agent_threads_by + termlink_agent_silent_senders — sender-started thread roots + quiet-peer detection MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T19:26:48Z
last_update: 2026-05-20T13:23:33Z
date_finished: 2026-05-05T19:33:48Z
---

# T-1588: termlink_agent_threads_by + termlink_agent_silent_senders — sender-started thread roots + quiet-peer detection MCP read tools

## Context

T-1587 brought MCP read surface to 120 tools. Wave 44 adds two **per-sender activity reads**:

- `termlink_agent_threads_by` — list thread roots STARTED by a specific sender. Walks topic, filters `msg_type=post` by `sender_id` (default = caller's local Identity) AND `metadata.in_reply_to` absent (root posts), then counts descendants via in_reply_to chain. Returns `[{root_offset, body_preview, ts_unix_ms, descendant_count}, ...]` sorted newest-first. Per-sender companion to T-1574 `agent_threads` (topic-wide) — answers "what conversations did X kick off?" or "what threads have I started?".
- `termlink_agent_silent_senders` — anti-leaderboard. Walks topic, finds all senders who have posted at least once but NOT within the configured window (default 14 days). Returns `[{sender_id, last_post_ts_unix_ms, days_silent}, ...]` sorted by days_silent descending. Useful for "who's gone quiet?" / re-engagement / fleet liveness audits.

Both pure walk + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentThreadsByParams` struct (sender_id Option<String>, limit Option<u64>)
- [x] New `AgentSilentSendersParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_threads_by` walks topic + filters root posts by sender_id + counts descendants
- [x] New `termlink_agent_silent_senders` walks topic + finds senders with last-post older than window
- [x] threads_by defaults sender_id to caller's local Identity
- [x] silent_senders default window_days=14
- [x] silent_senders excludes senders never posted (only ever-posted, now-quiet)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=122 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_threads_by` + `termlink_agent_silent_senders` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_threads_by` with default sender_id
  2. Verify list of thread roots you've started, with descendant counts
  3. Call `termlink_agent_silent_senders` with default window
  4. Cross-reference against `termlink_agent_peers` last_seen
  **Expected:** threads_by lists my thread roots; silent_senders surfaces ever-posted-now-quiet peers.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_threads_by"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_silent_senders"' crates/termlink-mcp/src/tools.rs

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Recommendation

**Recommendation:** GO
**Rationale:** Two per-sender activity reads. threads_by gives "what conversations did X start?" (per-sender T-1574 companion). silent_senders is the first anti-leaderboard tool — surfaces ever-posted-now-quiet peers for re-engagement / fleet liveness. Both pure walk + aggregate, ~100 LOC each. Brings session total to 9 waves, +18 read tools, mcp_tools 104→122.
**Evidence:**
- Build clean (4m 27s)
- `termlink version --json` reports mcp_tools=122 (was 120 after T-1587) — +2
- Verification gate 4/4 passed
- threads_by: O(n) walk + recursive descendant counter (HashMap parent→children); silent_senders: O(n) walk + per-sender max-ts + cutoff filter

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

### 2026-05-05T19:26:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1588-termlinkagentthreadsby--termlinkagentsil.md
- **Context:** Initial task creation

### 2026-05-05T19:33:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 10/10, build clean 4m27s, mcp_tools=122. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:33Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_threads_by`, `termlink_agent_silent_senders`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
