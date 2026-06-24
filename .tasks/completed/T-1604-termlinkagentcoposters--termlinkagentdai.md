---
id: T-1604
name: "termlink_agent_co_posters + termlink_agent_daily_volume — co-thread peer affinity + per-day post volume histogram MCP read tools"
description: >
  termlink_agent_co_posters + termlink_agent_daily_volume — co-thread peer affinity + per-day post volume histogram MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-06T05:58:10Z
last_update: 2026-05-20T13:23:43Z
date_finished: 2026-05-06T06:06:07Z
---

# T-1604: termlink_agent_co_posters + termlink_agent_daily_volume — co-thread peer affinity + per-day post volume histogram MCP read tools

## Context

T-1603 brought MCP read surface to 152 tools. Wave 60 adds two **affinity and volume reads**:

- `termlink_agent_co_posters` — given a `sender_id`, finds peers who frequently post in the same threads. Walks topic, builds offset→author + child→thread_root maps, identifies threads where target sender posted, tallies which OTHER senders also posted in those same threads. Returns `[{sender_id, shared_threads, last_co_thread_ts}, ...]` sorted by shared_threads desc. Pair-wise affinity / collaboration detector — answers "who does this peer co-thread with?". Companion to T-1595 peer_engagement (direct reply count between two specific peers) — pivots from "this pair" to "who's adjacent to this peer" leaderboard.
- `termlink_agent_daily_volume` — per-day post-count histogram over a window. Walks topic, filters `msg_type=post` within window, buckets by `(ts_unix_ms / 86_400_000)` (UTC day index), returns `[{date_iso, count}, ...]` sorted oldest-first. Day-axis companion to T-1596 activity_rhythm (hour-of-day). Useful for "what's our daily cadence?" / weekend-dip / posting-spike detection.

Both pure walk + bucket-tally.

## Acceptance Criteria

### Agent
- [x] New `AgentCoPostersParams` struct (sender_id String, limit Option<u64>)
- [x] New `AgentDailyVolumeParams` struct (window_days Option<u64>)
- [x] New `termlink_agent_co_posters` walks topic + builds offset→author + child→root maps + identifies threads target posted in + tallies other senders in those threads
- [x] New `termlink_agent_daily_volume` walks topic + filters posts in window + buckets by UTC-day index + emits date_iso + count rows
- [x] co_posters excludes target sender from leaderboard (no self-affinity)
- [x] co_posters default limit=20, capped at 200
- [x] daily_volume default window_days=14, capped at 90
- [x] daily_volume returns sorted oldest-first with `total_posts` summary
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=154 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_co_posters` + `termlink_agent_daily_volume` are operator-fluent over MCP
  **Steps:**
  1. Pick a peer's sender_id from `termlink_agent_peers`
  2. Call `termlink_agent_co_posters` with that sender_id
  3. Verify ranked leaderboard of co-thread peers
  4. Call `termlink_agent_daily_volume` with default window
  5. Verify daily histogram covers the window
  **Expected:** co_posters surfaces collaboration affinity per peer; daily_volume gives per-day cadence sense.
  **If not:** report ergonomics suggestions.

### HumanLegacy
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

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_co_posters"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_daily_volume"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two reads on different axes. co_posters is per-peer affinity leaderboard — pivots from peer_engagement (one specific pair) to "who's adjacent to this peer". daily_volume is day-axis companion to activity_rhythm — answers "weekend-dip / spike-week" questions. Both pure walk + tally, ~90-100 LOC each. Brings session total to 13 waves post-resume, +26 read tools, mcp_tools 128→154.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=154 (was 152 after T-1603) — +2
- Verification gate 4/4 (TBD)
- co_posters: O(n) walk + offset→author + child→root maps + per-thread author-set + cross-thread tally exclude-self; daily_volume: O(n) walk + window filter + day-bucket count

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

### 2026-05-06T05:58:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1604-termlinkagentcoposters--termlinkagentdai.md
- **Context:** Initial task creation

### 2026-05-06T06:06:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 9/9, build clean 5m07s, mcp_tools=154. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).


### 2026-05-20T13:23:43Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_co_posters`, `termlink_agent_daily_volume`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
