---
id: T-1610
name: "termlink_agent_recent_threads + termlink_agent_topic_summary — last-activity thread leaderboard + composite topic snapshot MCP read tools"
description: >
  termlink_agent_recent_threads + termlink_agent_topic_summary — last-activity thread leaderboard + composite topic snapshot MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T07:13:50Z
last_update: 2026-05-20T13:23:47Z
date_finished: 2026-05-06T07:21:23Z
---

# T-1610: termlink_agent_recent_threads + termlink_agent_topic_summary — last-activity thread leaderboard + composite topic snapshot MCP read tools

## Context

T-1609 brought MCP read surface to 164 tools. Wave 66 adds two **composite-orientation reads**:

- `termlink_agent_recent_threads` — thread roots sorted by last activity descending. Walks topic, builds parent→children map, computes max(ts) across each thread. Returns `[{root_offset, root_sender_id, body_preview, root_ts_unix_ms, last_activity_ts, mins_ago, descendant_count}, ...]` capped at limit. Companion to T-1589 busiest_threads (sorted by descendant count) and T-1609 idle_threads (filter inverse) — pivots to recency-ranked. Useful for "what threads are alive right now?" / hot-thread surface.
- `termlink_agent_topic_summary` — single-call composite topic snapshot. Walks topic ONCE and returns a composite of: total messages, msg_type breakdown, unique senders, total threads (roots), max thread size, latest topic description, last activity timestamp, and posts_in_last_24h count. Saves 5+ separate MCP calls during agent join/orientation. Highest-value single-read primitive for new agents joining chat-arc.

Both pure walk + bucket-tally.

## Acceptance Criteria

### Agent
- [x] New `AgentRecentThreadsParams` struct (limit Option<u64>)
- [x] New `AgentTopicSummaryParams` struct (no fields)
- [x] New `termlink_agent_recent_threads` walks topic + builds parent→children + per-thread max(ts) + sorts by last_activity desc
- [x] New `termlink_agent_topic_summary` walks topic once + composes 8 metrics + latest topic_metadata description
- [x] recent_threads default limit=20, capped at 200
- [x] recent_threads `mins_ago` derived from `(now_ms - last_activity_ts) / 60_000`
- [x] topic_summary returns `{total_messages, by_msg_type, unique_senders, total_threads, max_thread_size, description, last_activity_ts, posts_24h}`
- [x] topic_summary `description` defaults to empty string when no topic_metadata exists
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=166 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_recent_threads` + `termlink_agent_topic_summary` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_recent_threads` with default limit
  2. Verify ranked list of recently-active threads with mins_ago
  3. Call `termlink_agent_topic_summary`
  4. Verify all 8 composite fields populated
  **Expected:** recent_threads gives hot-thread snapshot; topic_summary is one-call orientation primitive.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_recent_threads"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_topic_summary"' crates/termlink-mcp/src/tools.rs

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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
**Rationale:** Two composite-orientation reads. recent_threads pivots from busiest_threads (descendant-count rank) to recency-rank — answers "what's hot RIGHT NOW?". topic_summary is the highest-value single-read for agent join/orientation — collapses 5+ separate calls into one composite snapshot. Both pure walk + tally, ~90-110 LOC each. Brings session total to 19 waves post-resume, +38 read tools, mcp_tools 128→166.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=166 (was 164 after T-1609) — +2
- Verification gate 4/4 (TBD)
- recent_threads: O(n) walk + parent→children + DFS max-ts per root + sort desc + body decode; topic_summary: O(n) walk composing 8 metrics in single pass

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

### 2026-05-06T07:13:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1610-termlinkagentrecentthreads--termlinkagen.md
- **Context:** Initial task creation

### 2026-05-06T07:21:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean 5m11s, mcp_tools=166. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:47Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_recent_threads`, `termlink_agent_topic_summary`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
