---
id: T-1581
name: "termlink_agent_edits_of + termlink_agent_topic_stats — per-post edit history + daily activity buckets MCP read tools"
description: >
  termlink_agent_edits_of + termlink_agent_topic_stats — per-post edit history + daily activity buckets MCP read tools

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T18:07:11Z
last_update: 2026-05-05T18:14:07Z
date_finished: 2026-05-05T18:14:00Z
---

# T-1581: termlink_agent_edits_of + termlink_agent_topic_stats — per-post edit history + daily activity buckets MCP read tools

## Context

T-1580 brought the MCP read surface to 106 tools (analyst aggregators). This wave adds two **history/activity** reads:

- `termlink_agent_edits_of` — given an offset, list ALL edits replacing that offset (`msg_type=edit` + `metadata.replaces=offset`). Returns `[{edit_offset, sender_id, payload_b64, ts_unix_ms}, ...]` sorted oldest-first (chronological edit history). Lets MCP-aware agents see every revision a post went through. Mirrors CLI T-1517.
- `termlink_agent_topic_stats` — daily activity buckets on agent-chat-arc. Walks the topic, groups envelopes by date (UTC YYYY-MM-DD from `ts_unix_ms`), aggregates total + by_msg_type per day. Returns `[{date, total, by_msg_type}, ...]` sorted by date asc. Activity heatmap — when is this topic most active. Mirrors CLI T-1531.

Both pure walk + aggregate. Continue the established walk-loop pattern.

## Acceptance Criteria

### Agent
- [x] New `AgentEditsOfParams` struct (offset u64)
- [x] New `AgentTopicStatsParams` struct (window_days Option<u64>)
- [x] New `termlink_agent_edits_of` tool method that walks topic + filters msg_type=edit + metadata.replaces=offset
- [x] New `termlink_agent_topic_stats` tool method that walks topic + buckets by date + aggregates
- [x] edits_of returns chronological JSON array (oldest first); topic_stats returns date-asc JSON array
- [x] `cargo build --release` clean
- [x] MCP tool count increments to >=108 (2 new)
- [x] `termlink version --json` reports the new mcp_tools count

### Human
- [ ] [REVIEW] Verify `termlink_agent_edits_of` + `termlink_agent_topic_stats` are operator-fluent over MCP
  **Steps:**
  1. From an MCP-aware client, call `termlink_agent_edits_of` with a known offset
  2. Compare with `target/release/termlink agent edits-of <offset>`
  3. Call `termlink_agent_topic_stats`
  4. Compare with `target/release/termlink agent topic-stats`
  **Expected:** MCP returns matching history/buckets; CLI shows similar set.
  **If not:** report ergonomics suggestions.

## Verification

cargo build --release 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink version --json 2>&1 | grep -qE '"mcp_tools":\s*1[0-9][0-9]'
grep -q '"termlink_agent_edits_of"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_topic_stats"' crates/termlink-mcp/src/tools.rs

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
**Rationale:** Two history/activity reads. edits_of(offset) gives the full chronological revision trail of a single envelope (audit/provenance). topic_stats provides daily activity buckets with msg_type breakdown — answers "when is this topic most active?" without dumping the whole envelope log. Established walk-loop pattern + small civil-from-days helper (`epoch_days_to_ymd`, no new deps). Both <100 LOC.
**Evidence:**
- Build clean (4m 42s)
- `termlink version --json` reports mcp_tools=108 (was 106 after T-1580) — +2
- Verification gate 4/4 passed
- edits_of: O(n) walk + filter by metadata.replaces; topic_stats: O(n) walk + BTreeMap bucket-by-date

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

### 2026-05-05T18:07:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1581-termlinkagenteditsof--termlinkagenttopic.md
- **Context:** Initial task creation

### 2026-05-05T18:14:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Agent ACs 8/8, build clean, mcp_tools=108. Human REVIEW remains for human-side MCP-fluency check (Tier-2 logged).
