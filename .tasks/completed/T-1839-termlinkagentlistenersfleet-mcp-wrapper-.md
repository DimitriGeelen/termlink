---
id: T-1839
name: "termlink_agent_listeners_fleet MCP wrapper (T-1837 + T-1836 follow-up)"
description: >
  termlink_agent_listeners_fleet MCP wrapper (T-1837 + T-1836 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T13:49:51Z
last_update: 2026-05-28T13:52:34Z
date_finished: 2026-05-28T13:52:34Z
---

# T-1839: termlink_agent_listeners_fleet MCP wrapper (T-1837 + T-1836 follow-up)

## Context

T-1836 shipped MCP parity for single-hub trio. T-1837 added the cross-hub merge script (`scripts/agent-listeners-fleet.sh`). LLM-driven agents that need fleet-wide listener visibility currently have to route through Bash to invoke the fleet sweep. This task closes that gap: a fourth MCP tool, `termlink_agent_listeners_fleet`, that reuses the T-1836 shell-out helper to subprocess the fleet script.

## Acceptance Criteria

### Agent
- [x] New MCP tool `termlink_agent_listeners_fleet` registered
- [x] Subprocesses `${TERMLINK_SCRIPTS_DIR:-/opt/termlink/scripts}/agent-listeners-fleet.sh` via the T-1836 shared `run_t1836_subprocess` helper
- [x] Params: `topic`, `limit`, `include_offline`, `filter_role`, `filter_listen_topic`, `filter_agent_id`, `hubs_file`, `timeout_secs` (default 30, clamped 1..=120 — higher than single-hub because of parallel fan-out)
- [x] Tool description references T-1837 and explains the LIVE > STALE > OFFLINE preference rule
- [x] At least 2 unit tests under #[cfg(test)] covering: parameter forwarding and parsed JSON pass-through
- [x] `cargo build -p termlink-mcp` clean
- [x] `cargo test -p termlink-mcp t1839` passes

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build -p termlink-mcp 2>&1 | tail -3
cargo test -p termlink-mcp t1839 2>&1 | tail -8
grep -q "termlink_agent_listeners_fleet" crates/termlink-mcp/src/tools.rs

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

### 2026-05-28T13:49:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1839-termlinkagentlistenersfleet-mcp-wrapper-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-21fc14c3
- **Timestamp:** 2026-05-28T13:52:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T13:52:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
