---
id: T-1944
name: "termlink_help — add agent_engagement_metrics + agent_rankings categories (18 tools)"
description: >
  Surface engagement analytics (emoji_stats/users, reactions_by/of, pinned_by, starred_by, reaction_rate/summary, engagement_rate, peer_engagement) and ranking/leaderboard tools (top_pinners, top_reacted, top_replied/repliers, top_starrers, top_thread_starters, first_post_by, first_responders).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T15:56:57Z
last_update: 2026-06-03T15:58:57Z
date_finished: 2026-06-03T15:59:58Z
---

# T-1944: termlink_help — add agent_engagement_metrics + agent_rankings categories (18 tools)

## Context

T-1943 follow-up. Surface the agent_* analytics long-tail in 2 new categories so
LLM consumers running community-health / engagement analyses can discover the
relevant tools via `termlink_help name_filter:"top"` or browsing by category.

`agent_engagement_metrics` (10): emoji_stats, emoji_users, reactions_by,
reactions_of, reaction_rate, reaction_summary, pinned_by, starred_by,
engagement_rate, peer_engagement.

`agent_rankings` (8): top_pinners, top_reacted, top_replied, top_repliers,
top_starrers, top_thread_starters, first_post_by, first_responders.

Out of scope (Tier-3, ~28 tools): the agent_* counter/distribution long-tail
(stats, daily_volume, msg_growth_rate, activity_rhythm, response_latency,
silence_gap, thread_health, busiest_threads, etc.). Those go in a follow-up if
demand justifies.

## Acceptance Criteria

### Agent
- [x] `agent_engagement_metrics` category added to `help_categories()` with all 10 entries
  - Evidence: 10 tuples (emoji_stats, emoji_users, reactions_by/of, reaction_rate/summary, pinned_by, starred_by, engagement_rate, peer_engagement) after `agent_poll` in `crates/termlink-mcp/src/tools.rs` `help_categories()`
- [x] `agent_rankings` category added to `help_categories()` with all 8 entries
  - Evidence: 8 tuples (top_pinners, top_reacted, top_replied, top_repliers, top_starrers, top_thread_starters, first_post_by, first_responders) alongside `agent_engagement_metrics`
- [x] `termlink_help` `#[tool(description = ...)]` lists both new categories
  - Evidence: description at `tools.rs:11384` lists `agent_engagement_metrics (emoji/reactions/pin/star analytics), agent_rankings (top_*/first_* leaderboards)`
- [x] `cargo test -p termlink-mcp --lib help_` passes (phantom guard verifies all 18 new entries map to real tools)
  - Evidence: `test result: ok. 6 passed; 0 failed`
- [x] `cargo build -p termlink-mcp` is warning-free
  - Evidence: `cargo build -p termlink-mcp 2>&1 | grep -E "warning|error"` returned empty
- [x] No duplicate tool names within `help_categories()`
  - Evidence: `awk ... | uniq -c | awk '$1 > 1'` returned empty

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

### 2026-06-03T15:56:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1944-termlinkhelp--add-agentengagementmetrics.md
- **Context:** Initial task creation
