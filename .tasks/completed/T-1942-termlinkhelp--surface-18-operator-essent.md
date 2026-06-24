---
id: T-1942
name: "termlink_help — surface 18 operator-essential missing tools + dedup agent_ask"
description: >
  Help registry has 252 real MCP tools but only surfaces 179. Surface Tier-1 operator-essentials (events, inbox, remote, net_test, recent_dm, agent_listen, agent_overview, agent_typing, agent_help, etc.). Dedup termlink_agent_ask (appears in agent_presence AND diagnostics).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-03T15:49:17Z
last_update: 2026-06-03T15:52:31Z
date_finished: 2026-06-03T15:54:24Z
---

# T-1942: termlink_help — surface 18 operator-essential missing tools + dedup agent_ask

## Context

Audit (T-1941 follow-up) found 73 real MCP tools missing from `help_categories()`.
Surface the Tier-1 operator-essentials so LLM consumers can discover them via
`termlink_help` and `termlink_help name_filter:"..."`. Also drop the duplicate
`termlink_agent_ask` from `diagnostics` (already present in `agent_presence`).

Tier-1 list (18 entries):
- diagnostics: termlink_help, termlink_events, termlink_inbox_status,
  termlink_inbox_list, termlink_inbox_clear, termlink_net_test
- remote: termlink_remote_doctor, termlink_remote_exec, termlink_remote_list,
  termlink_remote_inbox_status, termlink_remote_inbox_list, termlink_remote_inbox_clear
- agent_read: termlink_recent_dm
- agent_presence: termlink_agent_listen, termlink_agent_overview,
  termlink_agent_help, termlink_agent_send_auto_discover
- agent_chat: termlink_agent_typing, termlink_agent_typers (typing indicators)
- dedup: drop termlink_agent_ask from `diagnostics` (kept in agent_presence)

Out of scope: ~35 stats/analytics agent_* (emoji_stats, top_*, response_latency,
etc.) and channel admin (channel_members, channel_typing_emit/list,
channel_queue_status, channel_poll_*).

## Acceptance Criteria

### Agent
- [x] All 18 Tier-1 entries added to `help_categories()` in the right category
  - Evidence: 5 diagnostics adds + 6 remote adds + 1 agent_read add (recent_dm) + 4 agent_presence adds + 2 agent_chat adds (typing/typers) in `crates/termlink-mcp/src/tools.rs` `help_categories()`
- [x] Duplicate `termlink_agent_ask` removed from `diagnostics` (kept in `agent_presence`)
  - Evidence: diagnostics block in `help_categories()` no longer lists agent_ask; still present in agent_presence
- [x] `cargo test -p termlink-mcp --lib help_` passes (including the T-1941 phantom guard)
  - Evidence: `test result: ok. 6 passed; 0 failed` — `help_registry_has_no_phantom_entries` would have failed if any of the 18 new names didn't resolve
- [x] `cargo build -p termlink-mcp` is warning-free
  - Evidence: `cargo build -p termlink-mcp 2>&1 | grep -E "warning|error"` returned empty
- [x] No duplicate tool names within `help_categories()`
  - Evidence: `awk '/^fn help_categories/,/^\}$/' tools.rs | grep -oE '"termlink_[a-z_]+",' | sort | uniq -c | awk '$1 > 1'` returned empty

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

### 2026-06-03T15:49:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1942-termlinkhelp--surface-18-operator-essent.md
- **Context:** Initial task creation
