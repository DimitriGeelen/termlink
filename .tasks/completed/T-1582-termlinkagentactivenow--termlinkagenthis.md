---
id: T-1582
name: "termlink_agent_active_now + termlink_agent_history — recent-activity window + per-sender feed MCP read tools"
description: >
  termlink_agent_active_now + termlink_agent_history — recent-activity window + per-sender feed MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T18:14:36Z
last_update: 2026-05-20T13:23:30Z
date_finished: 2026-05-05T18:21:49Z
---

# T-1582: termlink_agent_active_now + termlink_agent_history — recent-activity window + per-sender feed MCP read tools

## Context

T-1581 brought MCP read surface to 108 tools. This wave adds two **decision-support reads** for in-the-moment situational awareness:

- `termlink_agent_active_now` — list senders active within the last N minutes (default 60). Walks topic, filters envelopes with ts >= now - window, groups by sender, returns `[{sender_id, posts_in_window, last_post_ts}, ...]` sorted desc by last_post_ts. Companion to T-1576 `agent_peers` (all-time directory) — answers "who's around right now?" not "who's ever spoken?".
- `termlink_agent_history` — given a `sender_id` (default = caller's local Identity), list that sender's content posts (excluding meta types: reaction/edit/redaction/topic_metadata/receipt/pin/star), sorted newest-first. Returns `[{offset, payload_b64, ts_unix_ms}, ...]`. Per-sender feed, complement to T-1576 peers' aggregate-only view.

Both pure walk + filter. Both ride the established walk-loop pattern.

## Acceptance Criteria

### Agent
- [x] New `AgentActiveNowParams` struct (window_minutes Option<u64>, limit Option<u64>)
- [x] New `AgentHistoryParams` struct (sender_id Option<String>, limit Option<u64>)
- [x] New `termlink_agent_active_now` tool method walks topic + windows by ts + groups by sender
- [x] New `termlink_agent_history` tool method walks topic + filters by sender_id + excludes meta msg_types
- [x] active_now defaults window_minutes=60, sorted by last_post_ts desc
- [x] history defaults sender_id to caller's local Identity, sorted newest-first
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=110 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_active_now` + `termlink_agent_history` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_active_now` with default window
  2. Compare with manual presence check (e.g. `target/release/termlink agent peers`)
  3. Call `termlink_agent_history` with default sender_id
  4. Verify it returns the caller's recent posts on chat-arc
  **Expected:** active_now lists recent senders only; history shows local agent's posts.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_active_now"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_history"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two situational-awareness reads. active_now answers "who's around right now?" (companion to T-1576 peers' all-time directory) — agents joining mid-flow can detect ambient activity without scanning. history answers "what has X been saying?" (per-sender feed, complement to peers' aggregate-only view) — caller defaults to self for "show my recent posts". Both pure walk-loop, ~75 LOC each. Brings MCP read surface to 110 tools (110-tool milestone past 100).
**Evidence:**
- Build clean (4m 57s)
- `termlink version --json` reports mcp_tools=110 (was 108 after T-1581) — +2
- Verification gate 4/4 passed
- active_now: window-filter + HashMap-aggregate; history: sender-filter + meta-exclude + sort

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

### 2026-05-05T18:14:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1582-termlinkagentactivenow--termlinkagenthis.md
- **Context:** Initial task creation

### 2026-05-05T18:21:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean, mcp_tools=110. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:30Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_active_now`, `termlink_agent_history`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
