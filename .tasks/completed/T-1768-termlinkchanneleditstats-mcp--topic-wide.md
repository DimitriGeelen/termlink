---
id: T-1768
name: "termlink_channel_edit_stats MCP — topic-wide edit count rollup"
description: >
  Port CLI cmd_channel_edit_stats / compute_edit_stats to MCP. Completes audit trio (pin-history + redactions + edit-stats) at MCP layer; pin-history and redactions already ported. No existing agent_* equivalent — new shape.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T13:19:33Z
last_update: 2026-05-21T13:23:54Z
date_finished: 2026-05-21T13:23:54Z
---

# T-1768: termlink_channel_edit_stats MCP — topic-wide edit count rollup

## Context

Port `cmd_channel_edit_stats` / `compute_edit_stats` (channel.rs:5710..5826) to MCP as `termlink_channel_edit_stats`. Completes the audit-trio at MCP layer (pin_history + redactions are already shipped; edit_stats was the missing third). No existing `agent_*` equivalent — new shape per target offset. Pattern: pure helper `compute_edit_stats_mcp` mirroring CLI semantics 1:1; tool method walks the topic via `walk_topic_full_mcp` and returns `{ok, topic, rows: [...], count}`.

## Acceptance Criteria

### Agent
- [x] `EditStatsRowMcp` struct + `to_json_mcp()` mirrors CLI `EditStatsRow` 1:1 (target_offset, target_sender, target_payload, edit_count, latest_editor, latest_ts_ms).
- [x] `compute_edit_stats_mcp(envelopes)` pure helper matches CLI semantics: skips redacted edits, skips redacted targets, latest_editor/latest_ts_ms tracks the most-recent surviving edit, sort = edit_count desc / target_offset asc tiebreak.
- [x] `termlink_channel_edit_stats` tool method registered, walks topic via `walk_topic_full_mcp`, returns `{ok, topic, rows, count}`.
- [x] `ChannelEditStatsParams { topic }` params struct.
- [x] Unit tests cover: empty topic → empty rows; one edit → count 1; multiple edits same target → count N + latest_editor reflects newest; redacted edit excluded; redacted target dropped entirely; sort by edit_count desc. (12 tests, all pass)
- [x] `cargo build -p termlink-mcp` clean.
- [x] `cargo test -p termlink-mcp compute_edit_stats_mcp` all pass.

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
cargo build -p termlink-mcp 2>&1 | tail -3
cargo test -p termlink-mcp --lib compute_edit_stats_mcp 2>&1 | tail -10

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

### 2026-05-21T13:19:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1768-termlinkchanneleditstats-mcp--topic-wide.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-b4a3e1fd
- **Timestamp:** 2026-05-21T13:23:55Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T13:23:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
