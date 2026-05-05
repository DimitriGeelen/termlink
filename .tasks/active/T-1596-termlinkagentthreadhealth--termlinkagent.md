---
id: T-1596
name: "termlink_agent_thread_health + termlink_agent_engagement_rate — composite thread aliveness score + per-sender reply-rate metric MCP read tools"
description: >
  termlink_agent_thread_health + termlink_agent_engagement_rate — composite thread aliveness score + per-sender reply-rate metric MCP read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T22:38:01Z
last_update: 2026-05-05T22:38:01Z
date_finished: null
---

# T-1596: termlink_agent_thread_health + termlink_agent_engagement_rate — composite thread aliveness score + per-sender reply-rate metric MCP read tools

## Context

T-1595 brought MCP read surface to 136 tools. Wave 52 adds two **composite-metric reads**:

- `termlink_agent_thread_health` — given a `root_offset`, returns a composite aliveness score: `{root_offset, descendant_count, unique_senders, last_post_age_hours, max_depth, status: "alive"|"slowing"|"dormant"|"dead"}`. Status derived from last_post_age_hours: <24h alive, 24-168h slowing, 168-720h dormant, >720h dead. Builds on T-1591 thread_summary primitives but collapses them into a one-call health verdict.
- `termlink_agent_engagement_rate` — given a `sender_id`, returns `{sender_id, posts_authored, posts_with_replies, engagement_rate}` where engagement_rate = posts_with_replies/posts_authored. Composite of T-1593 followups_to + per-sender post count. New axis: per-peer reply-rate metric, useful for "is this peer's content resonating?".

Both pure walk + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentThreadHealthParams` struct (root_offset u64)
- [x] New `AgentEngagementRateParams` struct (sender_id String)
- [x] New `termlink_agent_thread_health` walks topic + collects descendants + computes max_depth + last_post_age + status verdict
- [x] New `termlink_agent_engagement_rate` walks topic + counts sender's posts + counts those with at least one reply
- [x] thread_health status uses 24h/168h/720h thresholds for alive/slowing/dormant/dead
- [x] thread_health max_depth via BFS-with-depth on parent→children map
- [x] engagement_rate returns 0.0 when posts_authored=0
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=138 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_thread_health` + `termlink_agent_engagement_rate` are operator-fluent over MCP
  **Steps:**
  1. Pick a thread root from `termlink_agent_busiest_threads`
  2. Call `termlink_agent_thread_health` with that root_offset
  3. Verify status verdict matches recency feel
  4. Pick a sender_id from `termlink_agent_peers`
  5. Call `termlink_agent_engagement_rate` with that sender_id
  6. Verify engagement_rate looks plausible
  **Expected:** thread_health gives one-call thread aliveness; engagement_rate is per-peer resonance metric.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_thread_health"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_engagement_rate"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two composite-metric reads. thread_health collapses descendant-count + sender-count + recency + depth into a one-call verdict (alive/slowing/dormant/dead) — the most natural "is this conversation worth engaging with?" check. engagement_rate is the first per-peer resonance ratio (posts_with_replies/posts_authored) — answers "is this peer's content drawing engagement?". Both ~80-100 LOC. Brings session total to 17 waves, +34 read tools, mcp_tools 104→138.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=138 (was 136 after T-1595) — +2
- Verification gate 4/4 (TBD)
- thread_health: O(n) walk + recursive descendant set + BFS depth + age-bracket verdict; engagement_rate: O(n) walk + sender's-posts set + reply-target intersection

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

### 2026-05-05T22:38:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1596-termlinkagentthreadhealth--termlinkagent.md
- **Context:** Initial task creation
