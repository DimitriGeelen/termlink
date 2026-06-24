---
id: T-1945
name: "termlink_help — close coverage gap with agent_stats + agent_thread_health (28 tools)"
description: >
  Final 28 unsurfaced agent_* tools across 2 new categories: agent_stats (16 counters/distributions/aggregates) + agent_thread_health (12 thread-quality/activity-pattern queries). After this slice the help registry covers 100% of real MCP tools.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T19:45:49Z
last_update: 2026-06-03T19:48:40Z
date_finished: 2026-06-03T19:49:54Z
---

# T-1945: termlink_help — close coverage gap with agent_stats + agent_thread_health (28 tools)

## Context

T-1944 follow-up. Final 28 unsurfaced agent_* tools split into 2 new categories
to bring help-registry coverage to 100% of real MCP tools.

`agent_stats` (16): stats, daily_volume, msg_growth_rate, activity_rhythm,
age_distribution, response_latency, silence_gap, post_streak, topic_stats,
topic_summary, topic_metadata_history, user_summary, snippet, search_by,
recent_threads, forwards_of.

`agent_thread_health` (12): thread_health, thread_size_dist, threads_by,
busiest_threads, idle_threads, quiet_threads, orphan_replies, unanswered,
co_posters, burst_detect, silent_senders, self_replies.

After this slice every real `#[tool(name = ...)]` entry in tools.rs is
discoverable via `termlink_help`. The phantom-guard test (T-1941) validates the
direction "no help entry without a real tool"; coverage to 100% completes the
discovery arc from the other direction.

## Acceptance Criteria

### Agent
- [x] `agent_stats` category added to `help_categories()` with all 16 entries
  - Evidence: 16 tuples (stats, daily_volume, msg_growth_rate, activity_rhythm, age_distribution, response_latency, silence_gap, post_streak, topic_stats, topic_summary, topic_metadata_history, user_summary, snippet, search_by, recent_threads, forwards_of) after `agent_rankings` in `crates/termlink-mcp/src/tools.rs` `help_categories()`
- [x] `agent_thread_health` category added to `help_categories()` with all 12 entries
  - Evidence: 12 tuples (thread_health, thread_size_dist, threads_by, busiest_threads, idle_threads, quiet_threads, orphan_replies, unanswered, co_posters, burst_detect, silent_senders, self_replies) alongside `agent_stats`
- [x] `termlink_help` `#[tool(description = ...)]` lists both new categories
  - Evidence: description at `tools.rs:11384` lists `agent_stats (counters/distributions/growth/activity-rhythm), agent_thread_health (thread-quality, busiest/idle/orphan)`
- [x] `cargo test -p termlink-mcp --lib help_` passes
  - Evidence: `test result: ok. 6 passed; 0 failed` — phantom guard validates all 28 new entries resolve to real tools
- [x] `cargo build -p termlink-mcp` is warning-free
  - Evidence: `cargo build -p termlink-mcp 2>&1 | grep -E "warning|error"` returned empty
- [x] Coverage: 100% of real `termlink_*` tools surfaced
  - Evidence: `real=252 surfaced=252 missing=0` per `comm -23` diff of `name = "..."` macro names vs `help_categories()` tuples

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

### 2026-06-03T19:45:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1945-termlinkhelp--close-coverage-gap-with-ag.md
- **Context:** Initial task creation
