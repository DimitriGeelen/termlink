---
id: T-1716
name: "termlink_agent_contact MCP v2 — add require_online + ack_required (T-1715 follow-up)"
description: >
  termlink_agent_contact MCP v2 — add require_online + ack_required (T-1715 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-19T22:36:17Z
last_update: 2026-05-20T19:25:33Z
date_finished: 2026-05-20T19:25:33Z
---

# T-1716: termlink_agent_contact MCP v2 — add require_online + ack_required (T-1715 follow-up)

## Context

T-1715 shipped v1 of `termlink_agent_contact` MCP — covering target / target_fp / message / thread / dry_run / sender_id. The CLI verb's full Phase-2 surface also includes `--require-online` + `--online-window-secs` (T-1480) and `--ack-required` + `--ack-timeout-secs` (T-1485), which v1 explicitly deferred because they need MCP-side wiring for the presence-probe + ack-poll loops. This task closes that gap so MCP-aware agents (Claude Code, ntb-atc-plugin, etc.) get the same synchronous-engagement semantics as CLI callers.

## Acceptance Criteria

### Agent
- [x] `AgentContactParams` extended with: `require_online` (Option<bool>), `online_window_secs` (Option<u64>), `ack_required` (Option<bool>), `ack_timeout_secs` (Option<u64>). Defaults preserve v1 byte-identical behavior — `unwrap_or(false)` / `unwrap_or(...)` on the optional fields, unset = no probe / no poll (PL-172 byte-identity rule)
- [x] When `require_online=true`: probes `agent-chat-arc` via `fetch_topic_msgs_mcp` (slice=500) BEFORE posting, computes `PresenceMcp` via `evaluate_presence_msgs`. Returns `{ok: false, peer_fp, online_check: {online, last_seen_ms, posts_in_window, window_secs}, error: "..."}` when `posts_in_window == 0` — matches CLI exit-9 shape
- [x] When `require_online=false` (default): no probe, behavior identical to v1 (verified by `agent_contact_params_minimal_message_only_target_fp` test still passing — same shape unchanged)
- [x] When `ack_required=true`: AFTER posting, polls dm topic at ~1s cadence (via `tokio::time::sleep`) up to clamped `ack_timeout_secs`. Returns `ack: {received: bool, ts_ms?, wait_secs, timeout_secs?}` on the success envelope — `received=true` on first matching non-meta envelope from peer_fp with `ts > send_ts_ms_for_ack`, `received=false` on timeout
- [x] When `ack_required=false` (default): no poll, single round-trip post then return
- [x] Pure helpers shipped: `evaluate_presence_msgs(msgs, peer_fp, now_ms, window_ms) -> PresenceMcp` and `detect_ack_in_msgs_mcp(msgs, peer_fp, send_ts_ms) -> Option<i64>`. Mirror CLI's `evaluate_presence` + `detect_ack_in_msgs` one-to-one (same META_MSG_TYPES filter, `ts_unix_ms` preferred with `ts` fallback, case-sensitive sender_id match)
- [x] `fetch_topic_msgs_mcp(hub_socket, topic, slice_size)` helper does `channel.list` + `channel.subscribe` round-trip — uses local hub_socket like `termlink_agent_post`; symmetric with CLI's `fetch_topic_msgs` (commands/channel.rs:693)
- [x] `cargo build --release -p termlink-mcp` clean — 1m 11s, only the pre-existing `cur_run_end` warning
- [x] **7** unit tests for new helpers + extended params — exceeded the 6-test target. Coverage: params-v2-deserialize (1), evaluate_presence (3: online / offline / meta-filter), detect_ack (3: found / wrong-sender / ts-fallback). All 7 pass; the existing 8 v1 tests also still pass (15/15 total on the agent_contact suite)
- [x] Tool description in `#[tool(...)]` attribute updated — now mentions T-1716 surface, lists require_online + online_window_secs + ack_required + ack_timeout_secs with their clamp ranges + exit-code mapping. No longer claims "deferred to v2"

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
cargo test --release -p termlink-mcp agent_contact

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

### 2026-05-19T22:36:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1716-termlinkagentcontact-mcp-v2--add-require.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-31db1418
- **Timestamp:** 2026-05-20T19:25:34Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#7 (Agent)** — `fetch_topic_msgs_mcp(hub_socket, topic, slice_size)` helper does `channel.list` + `channel.subscribe` round-trip — uses local hub_socket like `termlink_agent_post`; symmetric with CLI's `fetch_topic_
  - **AC-verify-mismatch** (narrow, heuristic) — `path=commands/channel.rs in: `fetch_topic_msgs_mcp(hub_socket, topic, slice_size)` helper does `channel.list` + `channel.subscribe` round-trip — uses local hub_socket like `termli`

### 2026-05-20T19:25:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
