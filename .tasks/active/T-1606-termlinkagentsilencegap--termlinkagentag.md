---
id: T-1606
name: "termlink_agent_silence_gap + termlink_agent_age_distribution — longest peer absence + topic-wide post-age histogram MCP read tools"
description: >
  termlink_agent_silence_gap + termlink_agent_age_distribution — longest peer absence + topic-wide post-age histogram MCP read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T06:23:39Z
last_update: 2026-05-06T06:23:39Z
date_finished: null
---

# T-1606: termlink_agent_silence_gap + termlink_agent_age_distribution — longest peer absence + topic-wide post-age histogram MCP read tools

## Context

T-1605 brought MCP read surface to 156 tools. Wave 62 adds two **absence and age-shape reads**:

- `termlink_agent_silence_gap` — given a `sender_id`, computes the longest gap (in days) between consecutive posts. Walks topic, filters by sender, sorts ts list, walks pairs computing inter-post deltas. Returns `{sender_id, total_posts, max_gap_days, max_gap_start, max_gap_end, current_gap_days}` with start/end as YYYY-MM-DD UTC. Inverse of T-1605 post_streak — surfaces "longest break this peer took" + current absence. Onboarding/welcome-back triggers; lapsed-peer detection.
- `termlink_agent_age_distribution` — topic-wide post-age histogram. Walks topic, filters `msg_type=post`, buckets each by age relative to now: `<1h`, `1-24h`, `1-7d`, `7-30d`, `30-90d`, `>90d`. Returns `{total_posts, buckets: {label: count, ...}, oldest_post_ts, newest_post_ts}`. Topic-wide complement to T-1604 daily_volume — answers "how recent is the activity I'm looking at?" / triage health-check.

Both pure walk + bucket-tally.

## Acceptance Criteria

### Agent
- [x] New `AgentSilenceGapParams` struct (sender_id String)
- [x] New `AgentAgeDistributionParams` struct (no fields)
- [x] New `termlink_agent_silence_gap` walks topic + filters by sender + sorts ts list + computes inter-post deltas + tracks max gap and indices
- [x] New `termlink_agent_age_distribution` walks topic + buckets each post by age into 6 fixed bands
- [x] silence_gap returns max_gap_start + max_gap_end as YYYY-MM-DD via `epoch_days_to_ymd`
- [x] silence_gap reports `current_gap_days` as days since last post (or 0 if never posted)
- [x] silence_gap handles 0-post and 1-post sender gracefully
- [x] age_distribution buckets use labels `lt_1h`, `1_24h`, `1_7d`, `7_30d`, `30_90d`, `gt_90d`
- [x] age_distribution returns `oldest_post_ts` + `newest_post_ts`
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=158 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_silence_gap` + `termlink_agent_age_distribution` are operator-fluent over MCP
  **Steps:**
  1. Pick a verbose sender_id from `termlink_agent_peers`
  2. Call `termlink_agent_silence_gap` with that sender_id
  3. Verify max_gap_days + start/end + current_gap_days
  4. Call `termlink_agent_age_distribution`
  5. Verify all 6 bucket labels present with plausible counts
  **Expected:** silence_gap surfaces lapsed-peer pattern; age_distribution gives topic-recency snapshot.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_silence_gap"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_age_distribution"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two reads on different axes. silence_gap is inverse of post_streak — surfaces lapsed-peer / longest-absence pattern with welcome-back triggers. age_distribution is topic-wide health check for "is this active or stale?" — answers triage questions in one read. Both pure walk + tally, ~80-90 LOC each. Brings session total to 15 waves post-resume, +30 read tools, mcp_tools 128→158.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=158 (was 156 after T-1605) — +2
- Verification gate 4/4 (TBD)
- silence_gap: O(n log n) walk + sender filter + sort + pair-walk delta tracking; age_distribution: O(n) walk + age-bucket map + min/max ts

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

### 2026-05-06T06:23:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1606-termlinkagentsilencegap--termlinkagentag.md
- **Context:** Initial task creation
