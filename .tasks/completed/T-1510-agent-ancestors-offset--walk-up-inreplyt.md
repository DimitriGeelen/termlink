---
id: T-1510
name: "agent ancestors offset — walk up in_reply_to chain to root"
description: >
  agent ancestors offset — walk up in_reply_to chain to root

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T07:12:38Z
last_update: 2026-05-20T13:22:55Z
date_finished: 2026-05-05T07:17:53Z
---

# T-1510: agent ancestors offset — walk up in_reply_to chain to root

## Context

`agent thread <root>` (T-1509) walks DOWN. `agent quote <offset>` (T-1505) walks UP one level. Missing: walk UP all the way to root from any leaf offset. `cmd_channel_ancestors` already exists for any topic. Thin wrapper hard-pinning topic to `agent-chat-arc`. Together with thread + quote + reply, completes the four-verb threading toolset (read up to root, write reply, read down to leaves).

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Ancestors { offset, hub, json }` variant in cli.rs
- [x] main.rs dispatch arm calling `cmd_channel_ancestors("agent-chat-arc", offset, hub, json)`
- [x] `cargo build --release -p termlink` clean
- [x] `agent ancestors --help` shows `<OFFSET>` positional and `--hub` / `--json` flags
- [x] Live smoke text: `agent ancestors 320` walks 320 → 319 → 318 (root-up rendering)
- [x] Live smoke JSON: `agent ancestors 320 --json` returns parseable envelope

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
- [ ] [REVIEW] Verify `agent ancestors` reads naturally as up-walk
  **Steps:**
  1. `target/release/termlink agent ancestors 320`
  **Expected:** chain rendered from leaf 320 up through 319 to root 318.
  **If not:** report layout suggestions.

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent ancestors --help 2>&1 | grep -q -- "--hub"
target/release/termlink agent ancestors --help 2>&1 | grep -qi "OFFSET"
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

## Recommendation

**Recommendation:** GO
**Rationale:** Closes the four-verb threading toolset symmetry. Up: ancestors (T-1510) + quote (T-1505, parent only). Write: reply (T-1507). Down: thread (T-1509). Operator can now pick any leaf, walk to root, render full conversation. Pure dispatch wrapper (~12 LOC).
**Evidence:**
- Build clean
- Verification gate 3/3 passed
- Live smoke text: `agent ancestors 320` rendered indented 318 → 319 → 320 (root-down)
- Live smoke JSON: `{ancestors: [...]}` parseable

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-05T07:12:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1510-agent-ancestors-offset--walk-up-inreplyt.md
- **Context:** Initial task creation

### 2026-05-05T07:17:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:22:55Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent ancestors`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
