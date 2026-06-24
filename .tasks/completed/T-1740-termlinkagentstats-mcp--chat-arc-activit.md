---
id: T-1740
name: "termlink_agent_stats MCP — chat-arc activity stats"
description: >
  Port cmd_agent_stats to MCP — counts by msg_type/peer/project/thread for chat-arc within window. Helper-port pattern: summarize_chat_arc_stats_mcp + ChatArcStatsMcp struct. T-1166 MCP-parity arc.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-21T06:53:08Z
last_update: 2026-05-21T06:57:00Z
date_finished: 2026-05-21T06:57:00Z
---

# T-1740: termlink_agent_stats MCP — chat-arc activity stats

## Context

Port `cmd_agent_stats` (commands/agent.rs:2044) to MCP — chat-arc activity stats within a window, returning total + per-bucket counts (by_msg_type / by_peer / by_project / by_thread), each sorted desc by count + asc by key. Helper-port (T-1719 pattern): `summarize_chat_arc_stats` (channel.rs:1434) + `ChatArcStats` struct as `_mcp` variants. Reuses existing `fetch_topic_msgs_mcp` for the chat-arc read.

## Acceptance Criteria

### Agent
- [x] `summarize_chat_arc_stats_mcp` + `ChatArcStatsMcp` ported in `crates/termlink-mcp/src/tools.rs` — mirrors CLI: META exclusion, empty-sender exclusion, window cutoff (ts < cutoff || ts > now_ms drops), bucket sort desc-count then asc-key
- [x] `termlink_agent_stats` MCP tool method added with params: `window_secs` (default 86400, clamped 60..=604800), `top` (default 10, clamped 1..=100) — both optional
- [x] Tool returns `{ok, verb, window_secs, top, total, by_msg_type, by_peer, by_project, by_thread}` JSON; each bucket is array of `{key, count}` truncated to top-N
- [x] Unit tests cover: META exclusion, empty-sender skip, window cutoff (both pre + post), bucket sort order (count desc, key asc tie-break), top-N truncation, legacy `_thread` key acceptance, JSON shape
- [x] `cargo build -p termlink-mcp` clean (no new warnings)
- [x] `cargo test -p termlink-mcp` all pass with new tests + no regressions

## Verification
cd /opt/termlink && cargo build -p termlink-mcp --message-format=short 2>&1 | grep -vE 'cur_run_end|generated [0-9]+ warning' | grep -E '^(error|warning):' && exit 1 || exit 0
cd /opt/termlink && cargo test -p termlink-mcp --lib -- stats_mcp 2>&1 | tail -5 | grep -q 'test result: ok'
cd /opt/termlink && cargo test -p termlink-mcp --lib 2>&1 | tail -3 | grep -qE 'test result: ok\. [0-9]+ passed'

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

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-21T06:53:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1740-termlinkagentstats-mcp--chat-arc-activit.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-4ad12f3f
- **Timestamp:** 2026-05-21T06:57:01Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `summarize_chat_arc_stats_mcp` + `ChatArcStatsMcp` ported in `crates/termlink-mcp/src/tools.rs` — mirrors CLI: META exclusion, empty-sender exclusion, window cutoff (ts < cutoff || ts > now_ms drops),
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `summarize_chat_arc_stats_mcp` + `ChatArcStatsMcp` ported in `crates/termlink-mcp/src/tools.rs` — mirrors CLI: META exclusion, empty-sender exclusion,`

### 2026-05-21T06:57:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
