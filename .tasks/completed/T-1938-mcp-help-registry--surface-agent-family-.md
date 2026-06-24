---
id: T-1938
name: "MCP help registry — surface agent_* family (chat/read/presence/inbox/thread/poll categories)"
description: >
  Add the agent_* conversation/thread family to termlink_help so LLM consumers can discover them. Currently ~100 agent_* MCP tools are registered but invisible via help — agents calling termlink_help to plan their work cannot find post/reply/recent/inbox/threads etc. Scope: workflow tools only (chat=write, read=history, presence=who/where, inbox=mailbox, thread=navigation, poll=lifecycle). Out of scope: admin/aggregate stats (emoji_stats, age_distribution, busiest_threads, top_*, response_latency, daily_volume, etc.) — those are diagnostics, not workflow. Out of scope: channel_* family (separate slice).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T00:33:24Z
last_update: 2026-06-03T00:36:52Z
date_finished: 2026-06-03T00:40:09Z
---

# T-1938: MCP help registry — surface agent_* family (chat/read/presence/inbox/thread/poll categories)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `agent_chat` category exists in `termlink_help` registry with `agent_post`, `agent_reply`, `agent_quote`, `agent_forward`, `agent_edit`, `agent_redact`, `agent_react`, `agent_pin`, `agent_star`, `agent_describe`, `chat_arc_broadcast` — crates/termlink-mcp/src/tools.rs:11118-11129
- [x] `agent_read` category exists with `agent_recent`, `agent_recent_window`, `agent_recent_dm`, `agent_on_thread`, `agent_threads`, `agent_history`, `agent_timeline`, `agent_digest`, `agent_search`, `agent_search_thread`, `agent_recent_decisions`, `agent_envelope`, `agent_chat_arc_recent`, `agent_redactions` — tools.rs:11130-11144
- [x] `agent_presence` category exists with `agent_presence_now`, `agent_listeners`, `agent_listeners_fleet`, `agent_active_now`, `agent_active_in_thread`, `agent_peers`, `agent_who_is`, `agent_identity`, `agent_info`, `agent_state`, `agent_contact`, `agent_ping`, `agent_ask`, `listener_heartbeat`, `check_fleet_doorbell_mail_health` — tools.rs:11145-11160
- [x] `agent_inbox` category exists with `agent_inbox`, `agent_unread`, `agent_dms`, `agent_mentions`, `agent_ack`, `agent_ack_history`, `agent_ack_status`, `agent_response_received` — tools.rs:11161-11169
- [x] `agent_thread` category exists with `agent_thread`, `agent_thread_authors`, `agent_thread_summary`, `agent_thread_path`, `agent_thread_depth`, `agent_ancestors`, `agent_replies_of`, `agent_followups`, `agent_followups_to`, `agent_edits_of`, `agent_pin_history`, `agent_pinned`, `agent_pinned_history`, `agent_starred`, `agent_starred_history`, `agent_reactions`, `agent_relations` — tools.rs:11170-11187
- [x] `agent_poll` category exists with `agent_poll_start`, `agent_poll_vote`, `agent_poll_end` — tools.rs:11188-11192
- [x] Unknown-category error message lists all 6 new categories — tools.rs:11225
- [x] Tool description mentions the new categories so LLMs see them in tool-listing — tools.rs:11041
- [x] `cargo build --release -p termlink-mcp` is warning-free — verified 2026-06-03 (release build finished in 1m35s)

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
! cargo build --release -p termlink-mcp 2>&1 | grep -q "warning:"
grep -q '"agent_chat", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"agent_read", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"agent_presence", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"agent_inbox", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"agent_thread", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"agent_poll", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_post"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_recent"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_presence_now"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_agent_poll_start"' crates/termlink-mcp/src/tools.rs

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

### 2026-06-03T00:33:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1938-mcp-help-registry--surface-agent-family-.md
- **Context:** Initial task creation
