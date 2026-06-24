---
id: T-1863
name: "termlink_recent_dm MCP wrapper — agent-callable parity for /recent-dm (T-1862 follow-on)"
description: >
  termlink_recent_dm MCP wrapper — agent-callable parity for /recent-dm (T-1862 follow-on)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-29T10:24:08Z
last_update: 2026-05-29T10:27:01Z
date_finished: 2026-05-29T10:27:01Z
---

# T-1863: termlink_recent_dm MCP wrapper — agent-callable parity for /recent-dm (T-1862 follow-on)

## Context

T-1862 shipped `/recent-dm <peer>` as a Claude Code slash skill wrapping
`scripts/recent-dm.sh`. This task adds MCP parity so autonomous agents
(orchestrators, supervisors) can fetch per-peer DM history programmatically.

Symmetric to T-1852 (chat_arc_recent MCP) and T-1858 (chat_arc_broadcast MCP).
Same shell-out subprocess pattern via `Self::run_t1836_subprocess`.

## Acceptance Criteria

### Agent
- [x] `RecentDmParams` struct exists in `crates/termlink-mcp/src/tools.rs` with: peer, topic, self_id, limit, since_hours, hub, hubs_file, filter_msg_type, all_msg_types, timeout_secs — added in the params block adjacent to AgentChatArcRecentParams
- [x] `termlink_recent_dm` tool method exists in same file, following T-1836 subprocess pattern — added between `termlink_agent_chat_arc_recent` and `termlink_check_fleet_doorbell_mail_health`; mutex-validates peer XOR topic; delegates to `recent-dm.sh` via `run_t1836_subprocess`
- [x] Tool description names the script (`scripts/recent-dm.sh`), parent skill (`/recent-dm`), parent task IDs (T-1862, T-1863), and the read-side discovery toolkit position — verified: description references "T-1862 wrapper from T-1863", "read-side asymmetric to termlink_agent_chat_arc_recent", and the full `scripts/recent-dm.sh` shell-out
- [x] `cargo check -p termlink-mcp` builds clean — completed in 12.75s with only one pre-existing unused-assignment warning (line 23128, unrelated)
- [x] Tool registered in MCP server — `grep -c '#[tool('` shows 251 total tools; `grep 'name = "termlink_recent_dm"'` finds the new registration; `#[tool(...)]` macro auto-registers at compile-time so successful cargo check IS the registration check

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

grep -q "pub struct RecentDmParams" crates/termlink-mcp/src/tools.rs
grep -q "name = \"termlink_recent_dm\"" crates/termlink-mcp/src/tools.rs
cargo check -p termlink-mcp 2>&1 | grep -q "Finished\|finished"

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

### 2026-05-29T10:24:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1863-termlinkrecentdm-mcp-wrapper--agent-call.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1a0e6e2c
- **Timestamp:** 2026-05-29T10:27:06Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#3 (Agent)** — Tool description names the script (`scripts/recent-dm.sh`), parent skill (`/recent-dm`), parent task IDs (T-1862, T-1863), and the read-side discovery toolkit position — verified: description referenc
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/recent-dm.sh in: Tool description names the script (`scripts/recent-dm.sh`), parent skill (`/recent-dm`), parent task IDs (T-1862, T-1863), and the read-side discovery`

### 2026-05-29T10:27:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
