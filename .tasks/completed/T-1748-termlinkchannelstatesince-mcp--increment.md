---
id: T-1748
name: "termlink_channel_state_since MCP — incremental cursor sync"
description: >
  termlink_channel_state_since MCP — incremental cursor sync

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T08:07:17Z
last_update: 2026-05-21T08:11:48Z
date_finished: 2026-05-21T08:11:48Z
---

# T-1748: termlink_channel_state_since MCP — incremental cursor sync

## Context

Mirror `cmd_channel_state_since` (channel.rs:6530) to MCP. Adds incremental cursor
sync over `channel_state` — agents poll for state changes since a wall-clock `since_ms`
instead of refetching full topic state every cycle. Helper-port pattern, mirrors
the existing `compute_state_mcp` / `compute_snapshot_mcp` pattern shipped in T-1745/47.

## Acceptance Criteria

### Agent
- [x] Pure helper `compute_state_since_mcp(envelopes, since_ms, include_redacted) -> Vec<StateRowMcp>` added to `crates/termlink-mcp/src/tools.rs` — semantics match CLI's `compute_state_since`: filter rows where `max(ts_ms, latest_edit_ts_ms, redact_ts) >= since_ms`.
- [x] `ChannelStateSinceParams { topic, since_ms, include_redacted: Option<bool> }` params struct exposed.
- [x] `#[tool] termlink_channel_state_since` method returns `{ok, topic, since_ms, include_redacted, rows, count}`.
- [x] Unit tests cover: empty envelopes, all-stale (returns empty), all-fresh (returns full), boundary cutoff (since_ms equals row ts), edited row included via latest_edit_ts_ms, redacted row included via redact_ts, include_redacted=false hides redacted payload.
- [x] `cargo build -p termlink-mcp` is clean (no new warnings).
- [x] `cargo test -p termlink-mcp` passes (new tests included).

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
cargo test -p termlink-mcp --quiet

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

### 2026-05-21T08:07:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1748-termlinkchannelstatesince-mcp--increment.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-66e9c0a7
- **Timestamp:** 2026-05-21T08:12:04Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — Pure helper `compute_state_since_mcp(envelopes, since_ms, include_redacted) -> Vec<StateRowMcp>` added to `crates/termlink-mcp/src/tools.rs` — semantics match CLI's `compute_state_since`: filter rows 
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: Pure helper `compute_state_since_mcp(envelopes, since_ms, include_redacted) -> Vec<StateRowMcp>` added to `crates/termlink-mcp/src/tools.rs` — semanti`

### 2026-05-21T08:11:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
