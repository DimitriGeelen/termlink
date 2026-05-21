---
id: T-1741
name: "termlink_agent_overview MCP — composite fleet digest"
description: >
  Port cmd_agent_overview to MCP — composite fleet digest combining presence + by-project + recent posts. Helper-port pattern: summarize_fleet_presence_mcp + summarize_fleet_by_project_mcp + FleetPeerRowMcp + FleetProjectRowMcp. Reuses extract_recent_posts_mcp (T-1739). T-1166 MCP-parity arc continuation.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T07:06:20Z
last_update: 2026-05-21T07:09:59Z
date_finished: 2026-05-21T07:09:59Z
---

# T-1741: termlink_agent_overview MCP — composite fleet digest

## Context

Port `cmd_agent_overview` (agent.rs:2534) + `compose_overview_json` (agent.rs:2724) to MCP. Composite fleet digest: combines `summarize_fleet_presence` (channel.rs:1016) + `summarize_fleet_by_project` (channel.rs:1145) + `extract_recent_posts` (T-1739, already ported). Need two new helper ports + two structs + composite tool method.

## Acceptance Criteria

### Agent
- [x] `FleetPeerRowMcp` struct + `summarize_fleet_presence_mcp` helper ported to `crates/termlink-mcp/src/tools.rs` — mirrors CLI: META exclusion, empty-sender skip, filter_project/filter_thread AND-compose (latter via `metadata._thread`), drop peers with 0 in-window posts, sort by posts desc + peer_fp asc, top_project tie-break alphabetic
- [x] `FleetProjectRowMcp` struct + `summarize_fleet_by_project_mcp` helper ported — mirrors CLI: META exclusion, empty-sender skip, UNTAGGED posts always excluded (project IS the key), filter_project/filter_thread AND-compose, sort by posts desc + project asc, top_peer tie-break alphabetic
- [x] `termlink_agent_overview` MCP tool method composing presence + by_project + recent (via `extract_recent_posts_mcp`); params: `window_secs` (default 86400, clamped 60..=604800), `top` (default 10, clamped 1..=50) — both optional
- [x] Tool returns `{ok, verb, window_secs, top, peers: [...], projects: [...], recent_posts: [...], total_peers, total_projects}` JSON; peers/projects truncated to top-N
- [x] Unit tests cover: META exclusion (both helpers), filter_project AND filter_thread paths, untagged-post drop (by_project), zero-post peer drop (presence), top-N sort order + tie-break, top_project tie-break in presence, top_peer tie-break in by_project, JSON shape
- [x] `cargo build -p termlink-mcp` clean (no new warnings)
- [x] `cargo test -p termlink-mcp` all pass with new tests + no regressions

## Verification
cd /opt/termlink && cargo build -p termlink-mcp --message-format=short 2>&1 | grep -vE 'cur_run_end|generated [0-9]+ warning' | grep -E '^(error|warning):' && exit 1 || exit 0
cd /opt/termlink && cargo test -p termlink-mcp --lib -- fleet_presence_mcp 2>&1 | tail -5 | grep -q 'test result: ok'
cd /opt/termlink && cargo test -p termlink-mcp --lib -- fleet_by_project_mcp 2>&1 | tail -5 | grep -q 'test result: ok'
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

### 2026-05-21T07:06:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1741-termlinkagentoverview-mcp--composite-fle.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-438bc72d
- **Timestamp:** 2026-05-21T07:10:00Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `FleetPeerRowMcp` struct + `summarize_fleet_presence_mcp` helper ported to `crates/termlink-mcp/src/tools.rs` — mirrors CLI: META exclusion, empty-sender skip, filter_project/filter_thread AND-compose
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `FleetPeerRowMcp` struct + `summarize_fleet_presence_mcp` helper ported to `crates/termlink-mcp/src/tools.rs` — mirrors CLI: META exclusion, empty-sen`

### 2026-05-21T07:09:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
