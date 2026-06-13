---
id: T-1559
name: "agent dms/unread --watch — complete the personal-identity --watch family (T-1559+)"
description: >
  agent dms/unread --watch — complete the personal-identity --watch family (T-1559+)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T12:13:39Z
last_update: 2026-05-05T12:20:39Z
date_finished: 2026-05-05T12:20:39Z
---

# T-1559: agent dms/unread --watch — complete the personal-identity --watch family (T-1559+)

## Context

Two thin-wrap features in one wave to amortize the 4-min build cost:

1. **`agent dms --watch`** — live DM directory monitor. Inbox-watch covers all topics; this one is DM-only focused viewport. 5s default cadence (matches inbox).
2. **`agent unread --watch`** — live chat-arc unread count monitor. Single-shot is "how many did I miss?"; watch is "tell me when more arrive". 3s default cadence (chat-arc more active than full inbox).

Both follow the established `--watch` dispatch pattern: ANSI clear + header + delegate to existing helper + sleep. Both reject `--watch` + `--json`.

## Acceptance Criteria

### Agent
- [x] AgentAction::Dms gains `--watch` and `--watch-interval` (default 5s) flags
- [x] AgentAction::Unread gains `--watch` and `--watch-interval` (default 3s) flags
- [x] Both dispatch arms handle watch with ANSI clear + header + helper + sleep
- [x] Both reject `--watch` + `--json` with operator-readable error
- [x] `cargo build --release --bin termlink` clean
- [x] `agent dms --help` and `agent unread --help` both list `--watch` + `--watch-interval`
- [x] Live smoke watch: both render header + iterate without panic over 3 ticks

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
- [ ] [REVIEW] Verify both `--watch` views are steady and useful
  **Steps:**
  1. `target/release/termlink agent dms --watch --unread`
  2. `target/release/termlink agent unread --watch`
  **Expected:** Both render cleanly, refresh without flicker, surface change as it happens.
  **If not:** report cadence / layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent dms --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent unread --help 2>&1 | grep -q -- "--watch"
(target/release/termlink agent dms --watch --json 2>&1 || true) | grep -qiE "incompatible"
(target/release/termlink agent unread --watch --json 2>&1 || true) | grep -qiE "incompatible"
(target/release/termlink agent dms 2>&1 || true) | head -5 | grep -qiE "dm:|No DM topics|topic"
(target/release/termlink agent unread 2>&1 || true) | grep -qiE "unread|posts since|0 unread"
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
**Rationale:** Two thin-wrap features in one wave to amortize the 4-min build cost. `agent dms --watch` is the DM-only viewport companion to `agent inbox --watch` (all-topics). `agent unread --watch` is the chat-arc-only live monitor (3s default — chat-arc activity is higher frequency than full inbox). Both follow the established `--watch` pattern from T-1486/T-1498/T-1557/T-1558.
**Evidence:**
- Build clean (4m 05s)
- Both single-shot paths still work
- Both `--watch` views rendered cleanly across 3 ticks (28 DM topics surfaced; chat-arc unread=56 shown live)
- Both `--watch` + `--json` rejected with operator-readable error
- Verification gate 7/7 passed

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

### 2026-05-05T12:13:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1559-agent-dmsunread---watch--complete-the-pe.md
- **Context:** Initial task creation

### 2026-05-05T12:20:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:45:31Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `timeout 6 target/release/termlink agent dms --watch --unread` ; `timeout 6 target/release/termlink agent unread --watch` ; help+single-shot for both
- **Result:** both exit=124 (watch streams — timeout-terminated as expected); both --watch flags present; both single-shots exit=0. dms --watch --unread captured a clean partial frame (header + DM rows) before next clear.
- **Output:**
  ```
  $ timeout 6 target/release/termlink agent dms --watch --unread
  # agent dms --watch | interval=5s | 2026-06-13T13:44:29Z
  dm:bob-122-3107700:d1993c2c3ec44c94  (peer=bob-122-3107700)  unread=2  first=1
  dm:bob-122-3114651:d1993c2c3ec44c94  (peer=bob-122-3114651)  unread=2  first=1
  ... (28 DM topics surfaced; exit 124)
  $ target/release/termlink agent unread
  Topic 'agent-chat-arc': 4 unread for d1993c2c3ec44c94 (first new offset 3195, last 3198, last receipt up_to=3194)
  $ timeout 6 target/release/termlink agent unread --watch    (streams; exit 124)
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
