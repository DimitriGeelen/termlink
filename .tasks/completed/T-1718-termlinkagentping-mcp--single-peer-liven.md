---
id: T-1718
name: "termlink_agent_ping MCP — single-peer liveness probe (T-1487 parity)"
description: >
  Close the MCP-parity gap for the agent ping CLI verb (T-1487, work-completed). MCP-aware agents have access to fleet-wide presence (termlink_agent_active_now, termlink_agent_presence_now) but no targeted single-peer probe. termlink_agent_ping closes that gap, returning {online, last_seen_ms, last_seen_human, posts_in_window, window_secs} for one peer via the existing T-1716 evaluate_presence_msgs helper. Pure additive — no behavior change in existing tools.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-20T06:06:50Z
last_update: 2026-05-20T19:25:46Z
date_finished: 2026-05-20T19:25:46Z
---

# T-1718: termlink_agent_ping MCP — single-peer liveness probe (T-1487 parity)

## Context

The `termlink agent ping <target>` CLI verb (T-1487) is a single-peer liveness probe — it walks `agent-chat-arc`, computes presence for one peer over a window, and prints/returns `{online, last_seen_ms, posts_in_window, window_secs}` plus a human-readable last-seen phrase (Xs/m/h/d ago). MCP-aware agents have access to fleet-wide presence views (`termlink_agent_active_now`, `termlink_agent_presence_now`, `termlink_agent_peers`) but no targeted single-peer probe. This task closes that gap. Reuses the T-1716 `fetch_topic_msgs_mcp` + `evaluate_presence_msgs` helpers — no new RPC surface, just a new MCP tool method + params + tests.

## Acceptance Criteria

### Agent
- [x] `AgentPingParams` struct defined at tools.rs:1347-1361 with: `target` (Option<String>), `target_fp` (Option<String>), `window_secs` (Option<u64>). Mutex matches CLI: exactly one of `target`/`target_fp` required — enforced in the tool method's opening match arm.
- [x] Pure helper `format_last_seen_human(now_ms: i64, last_seen_ms: Option<i64>) -> String` added at tools.rs:303-322 — mirrors CLI's age-bucketing (Xs / Xm / Xh / Xd ago, or "never" for None). Pure, no I/O. Defensive: negative ages (clock skew) clamp to 0 (test `agent_ping_format_last_seen_clamps_negative_to_zero`).
- [x] `termlink_agent_ping` tool method registered at tools.rs:9231 via `#[tool(name = "termlink_agent_ping", description = "...")]`. Implementation flow: target/target_fp mutex → resolve peer_fp (validate hex OR session.discover) → `fetch_topic_msgs_mcp("agent-chat-arc", 500)` → `evaluate_presence_msgs` over `window_ms` → return JSON `{ok, target_or_fp, peer_fp, online, last_seen_ms, last_seen, window_secs, posts_in_window}`. Bare-name targets with a `:project` suffix have project stripped (CLI ping ignores it).
- [x] `window_secs` defaults to 300 (5 min, matches CLI) and clamps to [10, 86_400] — `p.window_secs.unwrap_or(300).clamp(10, 86_400)`. PL-172 byte-identity: unset is the same as explicit 300.
- [x] Tool description cites T-1487 (CLI parity) + T-1716 (helper reuse), explains return shape, distinguishes single-peer scope from fleet-wide `termlink_agent_presence_now` / `termlink_agent_active_now`. "NEVER posts — pure read" stated explicitly.
- [x] `cargo build --release -p termlink-mcp` clean — finished in 1m 09s, only the pre-existing `cur_run_end` warning (now at tools.rs:15252 due to upstream insertions, unrelated to T-1718)
- [x] **8** new unit tests added — exceeded the ≥3 target. Coverage: params-minimal-target-fp (a), params-with-explicit-window (b), format_last_seen_human None→never (c), seconds-bucket (incl 0s edge), minutes-bucket (5m + 59m boundary), hours-bucket (2h + 23h boundary), days-bucket (5d), negative-age clamps to 0 (defensive). All 8 pass. The 22-test agent_contact suite also still passes — total 30 tests across the agent_contact + agent_ping suites.

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
cargo test --release -p termlink-mcp agent_ping

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

### 2026-05-20T06:06:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1718-termlinkagentping-mcp--single-peer-liven.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-c7593a09
- **Timestamp:** 2026-05-20T19:25:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet-wide`

### 2026-05-20T19:25:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
