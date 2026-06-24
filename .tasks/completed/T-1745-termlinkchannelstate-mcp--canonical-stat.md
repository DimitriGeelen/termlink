---
id: T-1745
name: "termlink_channel_state MCP — canonical state of arbitrary topic (T-1166 MCP-parity)"
description: >
  MCP wrapper around channel.rs cmd_channel_state. Returns canonical post-edits/redactions state of an arbitrary topic — NOT just agent-chat-arc (which agent_state already covers). Fills the chan_state gap for MCP agents working with DM/project/custom topics. Mirrors compute_state pure helper. Edit-collapse: latest-edit-wins by (ts_ms, offset). Redaction: configurable surface as [REDACTED] or drop.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-21T07:46:03Z
last_update: 2026-05-21T07:50:15Z
date_finished: 2026-05-21T07:50:15Z
---

# T-1745: termlink_channel_state MCP — canonical state of arbitrary topic (T-1166 MCP-parity)

## Context

MCP wrapper for the channel.rs canonical-state reducer. `agent_state` already exists for
`agent-chat-arc` only (and reports a different shape — pins/stars/description curated digest).
`channel_state` is content-row-state for any topic: one row per content envelope, edits collapsed
to latest-wins, redactions optionally hidden or surfaced as `[REDACTED]`. CLI source:
channel.rs:5881 `compute_state` (pure) + 5971 `cmd_channel_state` (RPC wrapper).

## Acceptance Criteria

### Agent
- [x] `StateRowMcp` struct + `to_json_mcp` mirror of CLI StateRow (channel.rs:5834): all 8 fields preserved
- [x] Pure helper `compute_state_mcp(envelopes: &[Value], include_redacted: bool) -> Vec<StateRowMcp>` — tools.rs new section after summarize_fleet_by_project_mcp
- [x] `ChannelStateParams { topic: String, include_redacted: Option<bool> }` — defaults false
- [x] `termlink_channel_state` tool method: fetches via `fetch_topic_msgs_mcp(topic, 2000)`, returns `{ok, topic, include_redacted, rows, count}`
- [x] 11 unit tests added (9 helper + 2 params): empty, plain row, single edit, latest-wins tiebreak, redacted-dropped, redacted-surfaced, redacted-edit-excluded, sort-by-offset, meta-types-excluded
- [x] `cargo build -p termlink-mcp` clean
- [x] `cargo test -p termlink-mcp` 372 → 383 passing, 0 regressions

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

### 2026-05-21T07:46:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1745-termlinkchannelstate-mcp--canonical-stat.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-40e896d4
- **Timestamp:** 2026-05-21T07:50:20Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T07:50:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
