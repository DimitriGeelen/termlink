---
id: T-1605
name: "termlink_agent_post_streak + termlink_agent_presence_now — per-peer posting streak + live presence gauge MCP read tools"
description: >
  termlink_agent_post_streak + termlink_agent_presence_now — per-peer posting streak + live presence gauge MCP read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T06:11:32Z
last_update: 2026-05-06T06:11:32Z
date_finished: null
---

# T-1605: termlink_agent_post_streak + termlink_agent_presence_now — per-peer posting streak + live presence gauge MCP read tools

## Context

T-1604 brought MCP read surface to 154 tools. Wave 61 adds two **temporal-pattern reads**:

- `termlink_agent_post_streak` — given a `sender_id`, computes their longest consecutive-day posting streak and current ongoing streak. Walks topic, filters posts by sender, buckets by UTC day index, walks day-set in order tracking max consecutive run + current trailing run. Returns `{sender_id, total_post_days, max_streak_days, current_streak_days, max_streak_start, max_streak_end}`. Habit / consistency detector — answers "longest stretch this peer kept showing up?" / "are they still on a streak?".
- `termlink_agent_presence_now` — live presence gauge. Walks topic, identifies senders who posted in the last `minutes` (default 60, max 1440=24h), returns `[{sender_id, last_post_ts, mins_ago, post_count}, ...]` sorted by last_post_ts desc. Per-sender companion to T-1602 recent_window (which returns posts) — pivots to "who's around right now?". Useful for status-page / fleet-presence / who's-online checks.

Both pure walk + per-sender bucket.

## Acceptance Criteria

### Agent
- [x] New `AgentPostStreakParams` struct (sender_id String)
- [x] New `AgentPresenceNowParams` struct (minutes Option<u64>, limit Option<u64>)
- [x] New `termlink_agent_post_streak` walks topic + filters by sender + buckets by UTC day + walks ordered set tracking max + current streak
- [x] New `termlink_agent_presence_now` walks topic + filters posts in last N minutes + groups by sender + tallies count + sorts by last_post_ts desc
- [x] post_streak returns max_streak_start + max_streak_end as YYYY-MM-DD via existing `epoch_days_to_ymd` helper
- [x] post_streak handles zero-post sender gracefully (returns 0/0 with null dates)
- [x] presence_now default minutes=60, capped at 1440 (24h)
- [x] presence_now default limit=50, capped at 500
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=156 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_post_streak` + `termlink_agent_presence_now` are operator-fluent over MCP
  **Steps:**
  1. Pick a verbose sender_id from `termlink_agent_peers`
  2. Call `termlink_agent_post_streak` with that sender_id
  3. Verify max_streak_days + current_streak_days + start/end dates
  4. Call `termlink_agent_presence_now` with default minutes=60
  5. Verify list of currently-active senders
  **Expected:** post_streak surfaces consistency pattern; presence_now gives status-page snapshot.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_post_streak"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_presence_now"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two temporal-pattern reads. post_streak surfaces consistency / habit pattern per peer (companion to T-1599 user_summary which gives totals not consecutive runs). presence_now is status-page-style live gauge — pivots from recent_window's per-post listing to per-sender presence. Both pure walk + bucket, ~80-90 LOC each. Brings session total to 14 waves post-resume, +28 read tools, mcp_tools 128→156.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=156 (was 154 after T-1604) — +2
- Verification gate 4/4 (TBD)
- post_streak: O(n) walk + sender filter + day-bucket + sorted iteration tracking max+current run; presence_now: O(n) walk + minutes-window filter + per-sender (count, max-ts) tally + sort desc

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

### 2026-05-06T06:11:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1605-termlinkagentpoststreak--termlinkagentpr.md
- **Context:** Initial task creation
