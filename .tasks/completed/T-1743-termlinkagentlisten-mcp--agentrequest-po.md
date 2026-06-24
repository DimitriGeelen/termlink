---
id: T-1743
name: "termlink_agent_listen MCP — agent.request polling wrapper (T-1166 MCP-parity)"
description: >
  MCP wrapper around event.subscribe(agent.request) — closes the agent.rs CLI surface gap noted in S-2026-0521-0919 handover. One-shot polling pattern (caller passes since cursor + timeout, gets events + next_seq for iteration). Returns shaped {seq, from, action, request_id, params, timeout_secs} matching CLI --json output.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-21T07:34:23Z
last_update: 2026-05-21T07:38:41Z
date_finished: 2026-05-21T07:38:41Z
---

# T-1743: termlink_agent_listen MCP — agent.request polling wrapper (T-1166 MCP-parity)

## Context

MCP-parity wedge under T-1166 epic. The CLI's `cmd_agent_listen` is a long-running loop that
polls `event.subscribe` filtered to the `agent.request` topic on a session's bus, and shapes each
event into `{seq, from, action, request_id, params, timeout_secs}`. MCP cannot do streaming
one-shot, so this wraps a single iteration: caller passes optional `since` cursor + `timeout_ms`,
gets back shaped events + `next_seq` for iteration. Same composition pattern as
`termlink_event_subscribe` but topic-pinned and output-shaped.

## Acceptance Criteria

### Agent
- [x] `AgentListenParams` struct in tools.rs: `target: String`, `since: Option<u64>`, `timeout_ms: Option<u64>` (default 1000, clamped 100..=30_000) — tools.rs:4082
- [x] Pure helper `shape_agent_request_events_mcp(events: &[Value]) -> Vec<Value>` produces the shaped output (testable without RPC) — tools.rs:1421
- [x] `termlink_agent_listen` tool method: resolves session via `manager::find_session`, RPCs `event.subscribe` with topic=`agent.request`, returns `{ok, target, events, next_seq, agent_topic}` — tools.rs:~11181
- [x] Returns `{ok: false, target, error}` JSON on session-not-found, NOT a thrown error
- [x] At least 4 unit tests for the shape helper: empty input, single event with full fields, event missing fields, multiple events preserve order — 5 shape tests + 2 params tests = 7 new
- [x] `cargo build -p termlink-mcp` clean (no new warnings — only pre-existing cur_run_end)
- [x] `cargo test -p termlink-mcp` passes — 367 total (was 360, +7), 0 regressions

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
cargo build -p termlink-mcp
cargo test -p termlink-mcp --lib 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-05-21T07:34:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1743-termlinkagentlisten-mcp--agentrequest-po.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-cca380b5
- **Timestamp:** 2026-05-21T07:38:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T07:38:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
