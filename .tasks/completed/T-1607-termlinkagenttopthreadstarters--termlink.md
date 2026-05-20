---
id: T-1607
name: "termlink_agent_top_thread_starters + termlink_agent_thread_size_dist — root-authorship leaderboard + thread-size distribution MCP read tools"
description: >
  termlink_agent_top_thread_starters + termlink_agent_thread_size_dist — root-authorship leaderboard + thread-size distribution MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T06:36:36Z
last_update: 2026-05-20T13:23:45Z
date_finished: 2026-05-06T06:43:28Z
---

# T-1607: termlink_agent_top_thread_starters + termlink_agent_thread_size_dist — root-authorship leaderboard + thread-size distribution MCP read tools

## Context

T-1606 brought MCP read surface to 158 tools. Wave 63 adds two **structural-pattern reads**:

- `termlink_agent_top_thread_starters` — leaderboard of senders who start the most thread roots. Walks topic, identifies posts WITHOUT in_reply_to (i.e. roots), tallies per author within window, returns `[{sender_id, threads_started, last_root_ts}, ...]` sorted desc. Conversation-initiator pattern detector — different from T-1583 top_repliers (depth/reaction count) and T-1599 user_summary (per-peer composite). Useful for "who drives new conversations?".
- `termlink_agent_thread_size_dist` — distribution of thread sizes across the topic. Walks topic, identifies thread roots, counts ALL descendants (recursive) per root, buckets into size bands: `1` (no replies), `2_3`, `4_10`, `11_50`, `gt_50`. Returns `{total_threads, buckets: {label: count, ...}, max_thread_size, mean_thread_size}`. Topic-shape diagnostic — answers "is this topic mostly one-shots or deep threads?". Companion to T-1603 thread_depth (single-thread shape) — pivots to topic-wide.

Both pure walk + bucket-tally.

## Acceptance Criteria

### Agent
- [x] New `AgentTopThreadStartersParams` struct (window_days Option<u64>, limit Option<u64>)
- [x] New `AgentThreadSizeDistParams` struct (no fields)
- [x] New `termlink_agent_top_thread_starters` walks topic + filters posts without in_reply_to + per-author tally with window cutoff
- [x] New `termlink_agent_thread_size_dist` walks topic + builds parent→children + per-root descendant count + 5-band bucket
- [x] top_thread_starters default window_days=30, limit=20 capped at 200
- [x] thread_size_dist buckets use labels `1`, `2_3`, `4_10`, `11_50`, `gt_50`
- [x] thread_size_dist returns `max_thread_size` + `mean_thread_size` (rounded to 2 decimals)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=160 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_top_thread_starters` + `termlink_agent_thread_size_dist` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_top_thread_starters` with default window
  2. Verify ranked leaderboard of root-starters
  3. Call `termlink_agent_thread_size_dist`
  4. Verify all 5 size-band labels present + max + mean
  **Expected:** top_thread_starters surfaces who drives new conversations; thread_size_dist gives topic-shape sense.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_top_thread_starters"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_thread_size_dist"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two reads on different axes. top_thread_starters surfaces conversation-initiator pattern (distinct from top_repliers / user_summary). thread_size_dist is topic-shape diagnostic — pivots from per-thread thread_depth to topic-wide one-shots-vs-deep distribution. Both pure walk + tally, ~80-90 LOC each. Brings session total to 16 waves post-resume, +32 read tools, mcp_tools 128→160.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=160 (was 158 after T-1606) — +2
- Verification gate 4/4 (TBD)
- top_thread_starters: O(n) walk + root filter + window cutoff + per-author tally + sort desc; thread_size_dist: O(n) walk + parent→children + DFS descendant count per root + 5-band bucket

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

### 2026-05-06T06:36:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1607-termlinkagenttopthreadstarters--termlink.md
- **Context:** Initial task creation

### 2026-05-06T06:43:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean 4m49s, mcp_tools=160. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:45Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_top_thread_starters`, `termlink_agent_thread_size_dist`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
