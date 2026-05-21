---
id: T-1719
name: "termlink_agent_dms MCP — DM topic directory + unread (T-1552 parity)"
description: >
  Close MCP-parity gap for the agent dms CLI verb (T-1552). MCP-aware agents currently cannot enumerate their DM topics — they have to call termlink_channel_list and filter manually. This task ships termlink_agent_dms with basic mode (topic+peer rows) AND unread mode (per-DM channel.receipts probe + content envelope walk). Mirrors cmd_channel_dm_list one-to-one. No new RPC surface — uses existing channel.list + channel.receipts + channel.subscribe.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-20T06:16:58Z
last_update: 2026-05-20T19:34:09Z
date_finished: 2026-05-20T19:34:09Z
---

# T-1719: termlink_agent_dms MCP — DM topic directory + unread (T-1552 parity)

## Context

CLI `termlink agent dms [--unread]` (T-1552, work-completed) lists DM topics for the local identity, optionally with unread counts via channel.receipts + topic walk. MCP-aware agents currently can't enumerate "what DM conversations am I in?" without manually calling `termlink_channel_list` and filtering by `dm:` prefix + own FP. This task ships `termlink_agent_dms` with the full CLI shape — basic (topic+peer rows) AND unread (per-DM receipts probe → content envelope count + first_unread offset). Mirrors `cmd_channel_dm_list` (commands/channel.rs:1708) and `dm_list_filter` (commands/channel.rs:1687) one-to-one. No new RPC surface.

## Acceptance Criteria

### Agent
- [x] `AgentDmsParams` struct defined at tools.rs:1347-1356 with `unread: Option<bool>` (default false). Doc-comment cites T-1552 (CLI parity) + T-1719.
- [x] Pure helper `dm_list_filter_mcp` shipped at tools.rs:327-343 — mirror of CLI's `dm_list_filter` one-to-one. Bonus helpers `count_unread_mcp` (tools.rs:349-368, mirrors `count_unread`) and `walk_topic_full_mcp` (tools.rs:371-396, mirrors `walk_topic_full`) also shipped for the unread mode.
- [x] `termlink_agent_dms` tool method registered at tools.rs:9401. Flow shipped: load identity → `channel.list` (full catalog) → `dm_list_filter_mcp` → if `unread=false`: return `{ok, my_id, dms: [{topic, peer}, ...]}`; if `unread=true`: per-DM `channel.receipts` (ack frontier) + `walk_topic_full_mcp` + `count_unread_mcp` → return `{ok, my_id, dms: [{topic, peer, unread, first_unread}, ...]}` sorted unread-first. Per-DM walk failure produces a per-row `{..., error: "..."}` without aborting the whole tool call (graceful degradation).
- [x] Sort semantic matches CLI: unread topics float to top (stable within group). Implemented via `rows.sort_by(|a, b| { ... a_has.cmp(&b_has) })` with the `(unread > 0) ? 0 : 1` key (tools.rs ~9519). Rust's `sort_by` is stable, so original order is preserved within each (unread > 0) / (unread == 0) group — matches CLI's `sort_dm_inbox` (channel.rs:1811).
- [x] Tool description cites T-1552 (CLI parity), explains the channel.list filter, lists both return shapes (basic + unread), and ends with "No new RPC surface — uses channel.list / channel.receipts / channel.subscribe only." (tools.rs:9403).
- [x] `cargo build --release -p termlink-mcp` clean — finished in 1m 11s, only the pre-existing `cur_run_end` warning (now at tools.rs:15470 due to upstream insertions, unrelated to T-1719).
- [x] **7** new unit tests added — exceeded ≥3 target. Coverage: dm_list_filter_mcp-matches-either-side (a), dm_list_filter_mcp-empty-when-no-match (b), params-default-unread-unset + params-explicit-unread (c), count_unread_mcp-skips-below-bound, count_unread_mcp-skips-meta-types, count_unread_mcp-empty-returns-zero. All 7 pass. The 22-test agent_contact + 8-test agent_ping suites also still pass. Full `cargo test --release -p termlink-mcp --lib` runs **236 tests, 236 passed, 0 failed**.

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
cargo build --release -p termlink-mcp
cargo test --release -p termlink-mcp agent_dms

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

### 2026-05-20T06:16:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1719-termlinkagentdms-mcp--dm-topic-directory.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-8ffa171f
- **Timestamp:** 2026-05-20T19:35:18Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T19:34:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
