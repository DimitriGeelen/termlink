---
id: T-1558
name: "agent inbox --watch — live cross-topic unread monitor (T-1553 completion)"
description: >
  agent inbox --watch — live cross-topic unread monitor (T-1553 completion)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-05T12:07:17Z
last_update: 2026-05-05T12:12:56Z
date_finished: 2026-05-05T12:12:56Z
---

# T-1558: agent inbox --watch — live cross-topic unread monitor (T-1553 completion)

## Context

`agent inbox` (T-1553) is single-shot: walks local cursor store + joins with hub-side counts to compute unread-per-topic. Operator workflow "what needs my attention now" benefits hugely from a live viewport — operator leaves a watch terminal open, new mail surfaces as it arrives. Same pattern as T-1557 typers --watch (just shipped): add `--watch` / `--watch-interval` flags to AgentAction::Inbox, loop in main.rs dispatch with ANSI clear + header + delegate to `cmd_channel_inbox`. `--watch` + `--json` mutually incompatible — reject combo (matches presence/recent/typers).

## Acceptance Criteria

### Agent
- [x] AgentAction::Inbox gains `--watch` and `--watch-interval` (default 5s) flags
- [x] main.rs dispatch handles watch: ANSI clear + header + call helper + sleep
- [x] `--watch` + `--json` rejected with operator-readable error
- [x] `cargo build --release --bin termlink` clean
- [x] `agent inbox --help` lists `--watch` and `--watch-interval`
- [x] Live smoke single-shot: still works
- [x] Live smoke watch: prints header + iterates without panicking

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
- [ ] [REVIEW] Verify `agent inbox --watch` reads naturally as live unread monitor
  **Steps:**
  1. `target/release/termlink agent inbox --watch`
  2. From another shell, post to a tracked topic
  **Expected:** Unread count for the affected topic updates within one tick.
  **If not:** report cadence / layout suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent inbox --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent inbox --help 2>&1 | grep -q -- "--watch-interval"
(target/release/termlink agent inbox --watch --json 2>&1 || true) | grep -qiE "incompatible|--watch.*--json"
target/release/termlink agent inbox 2>&1 | head -5 | grep -qiE "topic|unread|cursor|No"
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
**Rationale:** Closes the live-monitor completion of the inbox primitive. Single-shot `agent inbox` is a snapshot; a left-open `--watch` terminal turns it into the operator's dashboard for "did anything new arrive?" across all subscribed topics. Pattern-consistent with `agent typers --watch` (T-1557) shipped in this session, and the broader `presence/recent/on-thread --watch` family. 5s default cadence (vs typers' 1s) reflects unread counts being less time-sensitive than typing TTL.
**Evidence:**
- Build clean (4m 03s)
- Live smoke watch: 3 ticks captured cleanly showing 3 topics (chat-arc 64 unread, 1 DM 3 unread, 1 inbox topic 2 unread)
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

### 2026-05-05T12:07:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1558-agent-inbox---watch--live-cross-topic-un.md
- **Context:** Initial task creation

### 2026-05-05T12:12:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:45:31Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `timeout 6 target/release/termlink agent inbox --watch` ; `target/release/termlink agent inbox --help` ; `target/release/termlink agent inbox`
- **Result:** exit=124 (watch streams forever — timeout-terminated as expected); --watch + --watch-interval (clamp [1,300], default 5s) flags present; single-shot exit=0 showing 3 unread topics
- **Output:**
  ```
  $ target/release/termlink agent inbox
  3 topic(s) with unread content:
    agent-chat-arc — 1587 unread (latest=3198, cursor=1611)
    t-1358-inbox-1777360315 — 2 unread (latest=5, cursor=3)
    dm:9219671e28054458:d1993c2c3ec44c94 — 1 unread (latest=45, cursor=44)
  $ timeout 6 target/release/termlink agent inbox --watch
  (live monitor; ANSI clear each 5s tick; exit 124)
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
