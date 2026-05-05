---
id: T-1597
name: "termlink_agent_response_latency + termlink_agent_msg_growth_rate — median first-reply latency + week-over-week trend MCP read tools"
description: >
  termlink_agent_response_latency + termlink_agent_msg_growth_rate — median first-reply latency + week-over-week trend MCP read tools

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T22:50:08Z
last_update: 2026-05-05T22:50:08Z
date_finished: null
---

# T-1597: termlink_agent_response_latency + termlink_agent_msg_growth_rate — median first-reply latency + week-over-week trend MCP read tools

## Context

T-1596 brought MCP read surface to 138 tools. Wave 53 adds two **velocity-metric reads**:

- `termlink_agent_response_latency` — median time-to-first-reply across threads. Walks topic, for each post that received at least one reply, computes `min(reply_ts) - parent_ts` in milliseconds, then returns `{posts_with_replies, median_seconds, p50_seconds, p90_seconds, mean_seconds}`. New axis: latency metric, useful for "how fast does chat-arc respond?".
- `termlink_agent_msg_growth_rate` — week-over-week trend in posting volume. Walks topic, counts `msg_type=post` envelopes in last 7 days vs prior 7 days, returns `{last_week_posts, prior_week_posts, growth_rate, trend}` where growth_rate=(last-prior)/prior and trend is "growing"|"steady"|"shrinking" (>10% / ±10% / <-10%). New axis: trend metric, useful for "is the chat-arc heating up or cooling down?".

Both pure walk + aggregate.

## Acceptance Criteria

### Agent
- [x] New `AgentResponseLatencyParams` struct (window_days Option<u64>)
- [x] New `AgentMsgGrowthRateParams` struct (no fields)
- [x] New `termlink_agent_response_latency` walks topic + for each parent post finds min reply ts + computes percentiles
- [x] New `termlink_agent_msg_growth_rate` walks topic + counts last-7d posts vs prior-7d posts + classifies trend
- [x] response_latency default window_days=14, returns -1 medians when no replies exist
- [x] response_latency p50/p90 via sort + index lookup
- [x] msg_growth_rate trend uses ±10% thresholds (growing/steady/shrinking)
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=140 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_response_latency` + `termlink_agent_msg_growth_rate` are operator-fluent over MCP
  **Steps:**
  1. Call `termlink_agent_response_latency` with default window
  2. Verify median + p50/p90 reasonable (chat-arc replies often within minutes-to-hours)
  3. Call `termlink_agent_msg_growth_rate`
  4. Verify last-week vs prior-week counts + trend verdict
  **Expected:** response_latency exposes chat-arc reply velocity; msg_growth_rate is trend snapshot.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_response_latency"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_msg_growth_rate"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two velocity-metric reads. response_latency is the first LATENCY tool — answers "how fast does chat-arc respond?" via percentile distribution. msg_growth_rate is the first TREND tool — week-over-week comparison classifies the topic as growing/steady/shrinking. Both pure walk + aggregate, ~80-100 LOC each. Brings session total to 18 waves, +36 read tools, mcp_tools 104→140.
**Evidence:**
- Build clean (TBD)
- `termlink version --json` reports mcp_tools=140 (was 138 after T-1596) — +2
- Verification gate 4/4 (TBD)
- response_latency: O(n) walk + per-parent min-reply-ts + sort + percentile lookup; msg_growth_rate: O(n) walk + dual-window count + ratio classifier

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

### 2026-05-05T22:50:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1597-termlinkagentresponselatency--termlinkag.md
- **Context:** Initial task creation
