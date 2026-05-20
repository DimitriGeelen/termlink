---
id: T-1578
name: "termlink_agent_unread + termlink_agent_digest — caught-up gap + period-summary MCP read tools"
description: >
  termlink_agent_unread + termlink_agent_digest — caught-up gap + period-summary MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T17:17:54Z
last_update: 2026-05-20T13:23:27Z
date_finished: 2026-05-05T17:23:51Z
---

# T-1578: termlink_agent_unread + termlink_agent_digest — caught-up gap + period-summary MCP read tools

## Context

T-1577 capped basic CLI parity at 100 tools. This wave adds two **action-oriented** reads — tools an MCP-aware agent uses to decide whether to engage:

- `termlink_agent_unread` — given a `sender_id` (default = caller's local identity), count envelopes with offset > that sender's last ack frontier. Returns `{sender_id, ack_up_to, total, unread_count}`. Lets agents detect "new mail since I last said I was caught up" without scanning the topic. Mirrors CLI T-1512.
- `termlink_agent_digest` — period summary on chat-arc. Walks topic, filters envelopes with ts >= since_ts (default = 24h ago), aggregates by msg_type + top-5 senders + total count. Returns single JSON object. Mirrors CLI T-1511. Single-call period awareness — what happened in the last day/hour/week.

Both build on existing patterns. unread combines the ack_status frontier (T-1577) with offset-counting; digest is info (T-1576) windowed to a time range.

## Acceptance Criteria

### Agent
- [x] New `AgentUnreadParams` struct (sender_id Option<String>)
- [x] New `AgentDigestParams` struct (since_ts Option<i64>, window_hours Option<u64>)
- [x] New `termlink_agent_unread` tool method that walks topic + finds sender's max ack + counts above
- [x] New `termlink_agent_digest` tool method that walks topic + windows by ts + aggregates
- [x] unread defaults sender_id to local Identity fingerprint; digest defaults to last 24h
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=102 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_unread` + `termlink_agent_digest` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_unread`
  2. Compare with `target/release/termlink agent unread`
  3. Call `termlink_agent_digest`
  4. Compare with `target/release/termlink agent digest`
  **Expected:** MCP returns matching counts/aggregates; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_unread"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_digest"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two action-oriented read tools — unread (gap-since-last-ack count) + digest (period summary). These are decision-support tools: an MCP-aware agent invokes them to answer "should I engage?" without scanning the topic. unread combines patterns from T-1577 (ack frontier extraction) + T-1576 (offset-counting); digest is info (T-1576) windowed to a time range. Both single-call: one walk, one structured response.
**Evidence:**
- Build clean (4m 03s)
- `termlink version --json` reports mcp_tools=102 (was 100 after T-1577) — +2
- Verification gate 4/4 passed
- unread default-resolves the caller's identity for "what's new for me" semantics; digest defaults 24h window for daily standup-style summaries

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

### 2026-05-05T17:17:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1578-termlinkagentunread--termlinkagentdigest.md
- **Context:** Initial task creation

### 2026-05-05T17:23:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:27Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `termlink_agent_unread`, `termlink_agent_digest`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
