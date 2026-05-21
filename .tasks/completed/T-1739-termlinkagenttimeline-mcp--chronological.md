---
id: T-1739
name: "termlink_agent_timeline MCP — chronological fleet log"
description: >
  Port cmd_agent_timeline to MCP — chronological fleet log with thread/project/msg_type/grep filters. Helper-port pattern: extract_recent_posts_mcp + RecentPostMcp struct. T-1166 MCP-parity arc continuation (after T-1738 snippet).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T06:47:14Z
last_update: 2026-05-21T06:51:55Z
date_finished: 2026-05-21T06:51:55Z
---

# T-1739: termlink_agent_timeline MCP — chronological fleet log

## Context

Port `cmd_agent_timeline` (commands/agent.rs:1826) to MCP-side `termlink_agent_timeline` tool. CLI verb shipped under T-1500 — chronological fleet log of recent posts across all peers/threads/projects, with thread/project/msg_type/grep filters. Helper-port pattern (T-1719): port `extract_recent_posts` + `RecentPost` struct as `_mcp` variants, then add tool method. No subprocess — `fetch_topic_msgs_mcp` already exists for the chat-arc read.

## Acceptance Criteria

### Agent
- [x] `extract_recent_posts_mcp` + `RecentPostMcp` ported in `crates/termlink-mcp/src/tools.rs` — mirrors CLI semantics (META exclusion, filters AND-composed, chrono asc sort, last-N keep, 200-char content cap, payload_b64+text+&str+raw fallback chain)
- [x] `termlink_agent_timeline` MCP tool method added with params: `n` (default 30, clamped 1..=500), `window_secs` (default 86400, clamped 60..=604800), `filter_thread`, `filter_project`, `filter_msg_types` (Vec<String>), `filter_grep` — all optional
- [x] Tool returns `{ok, verb, window_secs, n, filter_*, posts: [...]}` JSON with one entry per post including offset/ts_ms/peer_fp/msg_type/content/thread/project
- [x] Unit tests cover: meta exclusion, peer-implicit (no peer filter — fleet-wide), thread filter, project filter, msg_types allowlist, grep case-insensitive, window cutoff, last-N keep with chrono-asc sort, content truncation, payload_b64 decode
- [x] `cargo build -p termlink-mcp` clean (no new warnings)
- [x] `cargo test -p termlink-mcp` all pass with new tests
- [x] Existing tools.rs tests still pass (no regressions in 99-test baseline)

## Verification
cd /opt/termlink && cargo build -p termlink-mcp --message-format=short 2>&1 | grep -vE 'cur_run_end|generated [0-9]+ warning' | grep -E '^(error|warning):' && exit 1 || exit 0
cd /opt/termlink && cargo test -p termlink-mcp --lib -- timeline 2>&1 | tail -5 | grep -q 'test result: ok'
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

### 2026-05-21T06:47:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1739-termlinkagenttimeline-mcp--chronological.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e1ac680c
- **Timestamp:** 2026-05-21T06:51:56Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `extract_recent_posts_mcp` + `RecentPostMcp` ported in `crates/termlink-mcp/src/tools.rs` — mirrors CLI semantics (META exclusion, filters AND-composed, chrono asc sort, last-N keep, 200-char content 
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `extract_recent_posts_mcp` + `RecentPostMcp` ported in `crates/termlink-mcp/src/tools.rs` — mirrors CLI semantics (META exclusion, filters AND-compose`

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet-wide`

### 2026-05-21T06:51:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
