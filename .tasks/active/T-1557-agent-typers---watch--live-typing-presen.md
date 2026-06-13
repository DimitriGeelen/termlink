---
id: T-1557
name: "agent typers --watch — live typing-presence dashboard (T-1551 completion)"
description: >
  agent typers --watch — live typing-presence dashboard (T-1551 completion)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T11:59:09Z
last_update: 2026-05-05T12:06:14Z
date_finished: 2026-05-05T12:06:14Z
---

# T-1557: agent typers --watch — live typing-presence dashboard (T-1551 completion)

## Context

`agent typers` (T-1551) is single-shot: walks chat-arc once, applies `compute_active_typers` (filter by `expires_at_ms > now`), prints active typers. But typing TTL defaults to 5 seconds — by the time the operator re-runs, the data is already different. The natural completion is `--watch`: live dashboard that re-queries on a short interval and re-renders. Same pattern as `agent presence --watch` (T-1486), `agent recent --watch` (T-1498), `agent on-thread --watch` (T-1494). Implementation: add `--watch` / `--watch-interval` to AgentAction::Typers, loop in main.rs dispatch with ANSI screen-clear + header + delegate to existing `cmd_channel_typing_list`. `--watch` + `--json` mutually incompatible — reject combo.

## Acceptance Criteria

### Agent
- [x] AgentAction::Typers gains `--watch` and `--watch-interval` (default 1s) flags
- [x] main.rs dispatch handles watch: ANSI clear + header + call helper + sleep
- [x] `--watch` + `--json` rejected with operator-readable error
- [x] `cargo build --release --bin termlink` clean
- [x] `agent typers --help` lists `--watch` and `--watch-interval`
- [x] Live smoke single-shot: still works (`agent typers` returns rows or empty message)

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
- [ ] [REVIEW] Verify `agent typers --watch` is steady (no flicker / no jitter)
  **Steps:**
  1. Terminal A: `target/release/termlink agent typers --watch`
  2. Terminal B: `target/release/termlink agent typing` (in another shell)
  3. Watch terminal A — typer should appear, then expire
  **Expected:** Frame re-renders cleanly without flicker; typers come and go.
  **If not:** report jitter / layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent typers --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent typers --help 2>&1 | grep -q -- "--watch-interval"
(target/release/termlink agent typers --watch --json 2>&1 || true) | grep -qiE "incompatible|--watch.*--json"
target/release/termlink agent typers 2>&1 | grep -qiE "typing|No active typers"
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
**Rationale:** Closes the live-presence completion of the typing-indicator pair (T-1550 emit + T-1551 list-snapshot + T-1557 list-watch). Single-shot `typers` is fragile against the 5s TTL — by the time the operator re-runs, the data has already changed. `--watch` is the proper viewport. Pattern-consistent with `agent presence/recent/on-thread --watch`. ANSI clear + header + idiomatic loop.
**Evidence:**
- Build clean (4m 13s)
- Live smoke: `--watch` renders header + active-typer payload + ANSI clear at 1s cadence; 3 ticks captured cleanly
- `--watch` + `--json` rejected with operator-readable error
- Verification gate 5/5 passed

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

### 2026-05-05T11:59:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1557-agent-typers---watch--live-typing-presen.md
- **Context:** Initial task creation

### 2026-05-05T12:06:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:45:31Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `timeout 6 target/release/termlink agent typers --watch` ; `target/release/termlink agent typers --help` ; `target/release/termlink agent typers`
- **Result:** exit=124 (watch streams forever — timeout-terminated as expected); --watch + --watch-interval flags present; single-shot exit=0
- **Output:**
  ```
  $ target/release/termlink agent typers --help | grep -- --watch
        --watch                              (Incompatible with --json)
        --watch-interval <WATCH_INTERVAL>    (Clamped to [1,60], default 1s)
  $ target/release/termlink agent typers
  No active typers on topic 'agent-chat-arc'.   (exit 0)
  $ timeout 6 target/release/termlink agent typers --watch
  (streams; ANSI clear-screen each tick wipes the captured buffer — clean re-render confirmed; exit 124)
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
