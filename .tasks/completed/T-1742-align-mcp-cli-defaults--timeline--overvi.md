---
id: T-1742
name: "Align MCP-CLI defaults — timeline + overview (T-1739/41 follow-up)"
description: >
  Parity divergence in T-1739 (timeline) and T-1741 (overview) MCP wedges: defaults differ from CLI. Timeline: CLI uses n=50/window=3600s, MCP used n=30/window=86400s. Overview: CLI uses window=3600/top=5, MCP used window=86400/top=10. Fix MCP defaults to match CLI verbatim; agents calling MCP should get same shape as humans via CLI.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T07:16:27Z
last_update: 2026-05-21T07:18:42Z
date_finished: 2026-05-21T07:18:42Z
---

# T-1742: Align MCP-CLI defaults — timeline + overview (T-1739/41 follow-up)

## Context

Two new MCP tools shipped this session (T-1739 timeline + T-1741 overview) used arbitrary defaults instead of CLI parity. CLI source-of-truth (`crates/termlink-cli/src/cli.rs`):
- `Timeline { n: 50, window_secs: 3600 }` (cli.rs:4103-4111) — MCP used `n=30, window_secs=86400` ❌
- `Overview { window_secs: 3600, top: 5 }` (cli.rs:4159-4168) — MCP used `window_secs=86400, top=10` ❌
- `Stats { window_secs: 86400, top: 10 }` (cli.rs:4196-4204) — MCP already matches ✓

Greenfield tools — no shipped consumers yet, so the contract change is free. Agents calling MCP should get the same defaults as humans calling CLI.

## Acceptance Criteria

### Agent
- [x] `AgentTimelineParams::n.unwrap_or(30)` → `unwrap_or(50)` in tools.rs (matches `cli.rs:4105`)
- [x] `AgentTimelineParams::window_secs.unwrap_or(86_400)` → `unwrap_or(3600)` in tools.rs (matches `cli.rs:4110`)
- [x] `AgentOverviewParams::window_secs.unwrap_or(86_400)` → `unwrap_or(3600)` in tools.rs (matches `cli.rs:4162`)
- [x] `AgentOverviewParams::top.unwrap_or(10)` → `unwrap_or(5)` in tools.rs (matches `cli.rs:4167`)
- [x] Docstrings on both params structs updated to reflect new defaults
- [x] `cargo build -p termlink-mcp` clean (no new warnings)
- [x] `cargo test -p termlink-mcp` 360 tests still pass (no regressions — tests test the pure helpers, not defaults)

## Verification
cd /opt/termlink && cargo build -p termlink-mcp --message-format=short 2>&1 | grep -vE 'cur_run_end|generated [0-9]+ warning' | grep -E '^(error|warning):' && exit 1 || exit 0
cd /opt/termlink && cargo test -p termlink-mcp --lib 2>&1 | tail -3 | grep -qE 'test result: ok\. 360 passed'

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

### 2026-05-21T07:16:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1742-align-mcp-cli-defaults--timeline--overvi.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-09b7ea43
- **Timestamp:** 2026-05-21T07:18:43Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T07:18:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
