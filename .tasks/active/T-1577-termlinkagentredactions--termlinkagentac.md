---
id: T-1577
name: "termlink_agent_redactions + termlink_agent_ack_status — curation log + receipt frontiers MCP read tools"
description: >
  termlink_agent_redactions + termlink_agent_ack_status — curation log + receipt frontiers MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T17:07:19Z
last_update: 2026-05-05T17:13:09Z
date_finished: 2026-05-05T17:13:09Z
---

# T-1577: termlink_agent_redactions + termlink_agent_ack_status — curation log + receipt frontiers MCP read tools

## Context

T-1576 brought the MCP read surface to 98 tools. This wave hits the **100-tool milestone** with two high-value aggregator reads:

- `termlink_agent_redactions` — list all `msg_type=redaction` envelopes. Returns `[{redacts_offset, sender_id, reason, ts_unix_ms}, ...]` sorted newest-first. Mirrors CLI T-1534. Lets MCP-aware agents see the curation log at a glance — what's been retracted and why.
- `termlink_agent_ack_status` — current receipt frontier per sender. Walks msg_type=receipt envelopes, groups by sender_id, keeps `max(up_to)` and the corresponding ts. Returns `[{sender_id, ack_up_to, last_ack_ts}, ...]` sorted by ack_up_to desc. Mirrors CLI T-1539. Lets agents see who has read what (i.e. who's caught up).

Both pure walk+aggregate. Companion to write tools `termlink_agent_redact` (T-1566) and `termlink_agent_ack` (T-1568). 100-tool milestone — round-number milestone caps the read surface at orientation/curation/engagement parity with CLI.

## Acceptance Criteria

### Agent
- [x] New `AgentRedactionsParams` struct (limit Option<u64>)
- [x] New `AgentAckStatusParams` struct (no params)
- [x] New `termlink_agent_redactions` tool method that walks topic + filters msg_type=redaction
- [x] New `termlink_agent_ack_status` tool method that walks topic + groups receipts by sender + keeps max(up_to)
- [x] redactions returns JSON array sorted ts desc; ack_status returns JSON array sorted ack_up_to desc
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=100 (2 new) — round-number milestone
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_redactions` + `termlink_agent_ack_status` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_redactions`
  2. Compare with `target/release/termlink agent redactions`
  3. Call `termlink_agent_ack_status`
  4. Compare with `target/release/termlink agent ack-status`
  **Expected:** MCP returns matching data; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_redactions"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_ack_status"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two aggregator read tools — redactions (curation log) + ack_status (receipt frontiers per sender). Hit the 100-tool round-number milestone. Both companion reads to existing write tools (T-1566 redact, T-1568 ack). MCP-aware agents now have parity with CLI for the chat-arc operations operators use most: orient (info/peers/recent), search (search/quote), navigate (threads/on_thread/ancestors), curate (pinned/starred/redactions), engage (react/reactions), and acknowledge (ack/ack_status).
**Evidence:**
- Build clean (4m 00s)
- `termlink version --json` reports mcp_tools=100 (was 98 after T-1576) — +2, milestone hit
- Verification gate 4/4 passed
- Read surface now: 14 dedicated agent_* read tools (recent, search, quote, threads, on_thread, reactions, pinned, starred, info, peers, redactions, ack_status, plus generic channel_subscribe / topic listing)

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

### 2026-05-05T17:07:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1577-termlinkagentredactions--termlinkagentac.md
- **Context:** Initial task creation

### 2026-05-05T17:13:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
