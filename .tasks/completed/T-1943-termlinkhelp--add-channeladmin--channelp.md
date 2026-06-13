---
id: T-1943
name: "termlink_help — add channel_admin + channel_poll categories (8 tools)"
description: >
  Surface 8 remaining channel admin tools: channel_members, channel_queue_status, channel_typing_emit, channel_typing_list (new channel_admin category) + channel_poll_start, channel_poll_vote, channel_poll_end, channel_poll_results (new channel_poll category).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T15:53:47Z
last_update: 2026-06-03T15:55:46Z
date_finished: 2026-06-03T15:57:33Z
---

# T-1943: termlink_help — add channel_admin + channel_poll categories (8 tools)

## Context

T-1941/T-1942 follow-up. 8 channel admin/poll tools remain unsurfaced in the help
registry. Add 2 new categories so LLM consumers can discover them.

- `channel_admin` (4 tools): channel_members, channel_queue_status,
  channel_typing_emit, channel_typing_list
- `channel_poll` (4 tools): channel_poll_start, channel_poll_vote,
  channel_poll_end, channel_poll_results — symmetric with the existing
  `agent_poll` category

Also update the `termlink_help` description to list the new categories so the
LLM-facing description is up-to-date with the available filters.

## Acceptance Criteria

### Agent
- [x] `channel_admin` category added to `help_categories()` with 4 entries
  - Evidence: 4 tuples (channel_members, channel_queue_status, channel_typing_emit, channel_typing_list) inserted before `agent_chat` in `crates/termlink-mcp/src/tools.rs` `help_categories()`
- [x] `channel_poll` category added to `help_categories()` with 4 entries
  - Evidence: 4 tuples (channel_poll_start/vote/end/results) alongside `channel_admin`; symmetric with `agent_poll`
- [x] `termlink_help` `#[tool(description = ...)]` lists both new categories
  - Evidence: description at `tools.rs:11384` now lists `channel_admin (members/queue/typing), channel_poll` and hints `typing` under agent_chat + `listen` under agent_presence
- [x] `cargo test -p termlink-mcp --lib help_` passes (phantom guard verifies all 8 new entries)
  - Evidence: `test result: ok. 6 passed; 0 failed` — `help_registry_has_no_phantom_entries` would have failed if any of the 8 new names didn't resolve
- [x] `cargo build -p termlink-mcp` is warning-free
  - Evidence: `cargo build -p termlink-mcp 2>&1 | grep -E "warning|error"` returned empty

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
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

### 2026-06-03T15:53:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1943-termlinkhelp--add-channeladmin--channelp.md
- **Context:** Initial task creation
