---
id: T-1611
name: "termlink_agent_top_pinners + termlink_agent_top_starrers — most-active curators leaderboards MCP read tools"
description: >
  termlink_agent_top_pinners + termlink_agent_top_starrers — most-active curators leaderboards MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T07:27:11Z
last_update: 2026-05-06T07:34:28Z
date_finished: 2026-05-06T07:34:28Z
---

# T-1611: termlink_agent_top_pinners + termlink_agent_top_starrers — most-active curators leaderboards MCP read tools

## Context

T-1610 brought MCP read surface to 166 tools. Wave 67 adds two **curator-leaderboard reads**:

- `termlink_agent_top_pinners` — leaderboard of senders who pin the most. Walks topic, filters `msg_type=pin`, tallies actions per `sender_id`, returns `[{sender_id, pin_actions, last_pin_ts}, ...]` sorted desc. Curator-activity leaderboard. Distinct from existing `termlink_agent_pinned_by` (per-curator list of CURRENT pins after latest-wins reduce) — this counts ALL pin actions (raw activity) per sender.
- `termlink_agent_top_starrers` — same shape for `msg_type=star`. Returns `[{sender_id, star_actions, last_star_ts}, ...]` sorted desc.

Both pure walk + per-sender tally.

## Acceptance Criteria

### Agent
- [x] New `AgentTopPinnersParams` struct (limit Option<u64>)
- [x] New `AgentTopStarrersParams` struct (limit Option<u64>)
- [x] New `termlink_agent_top_pinners` walks topic + filters msg_type=pin + per-sender (count, max-ts) tally + sort desc
- [x] New `termlink_agent_top_starrers` walks topic + filters msg_type=star + per-sender (count, max-ts) tally + sort desc
- [x] Both default limit=20, capped at 200
- [x] Both return total + returned + leaderboard
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=168 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_top_pinners` + `termlink_agent_top_starrers` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_top_pinners` with default limit
  2. Verify ranked leaderboard of pin-action counts per sender
  3. Call `termlink_agent_top_starrers`
  4. Verify ranked leaderboard of star-action counts per sender
  **Expected:** both surface curator-activity leaders distinct from the per-curator `pinned_by`/`starred_by` reads.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_top_pinners"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_top_starrers"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two curator-activity leaderboards. top_pinners + top_starrers count RAW pin/star actions per sender, distinct from pinned_by/starred_by which return per-sender CURRENT pins after latest-wins reduce. Surfaces curator behavior pattern. Both pure walk + tally, ~70 LOC each. Brings session total to 20 waves post-resume, +40 read tools, mcp_tools 128→168.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=168 (was 166 after T-1610) — +2
- Verification gate 4/4 (TBD)
- top_pinners: O(n) walk + msg_type=pin filter + per-sender (count, max-ts) tally + sort desc; top_starrers: identical with msg_type=star

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

### 2026-05-06T07:27:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1611-termlinkagenttoppinners--termlinkagentto.md
- **Context:** Initial task creation

### 2026-05-06T07:34:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 8/8, build clean 5m17s, mcp_tools=168. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
