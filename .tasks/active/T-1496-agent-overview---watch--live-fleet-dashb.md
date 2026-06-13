---
id: T-1496
name: "agent overview --watch — live fleet dashboard"
description: >
  agent overview --watch — live fleet dashboard

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T17:53:27Z
last_update: 2026-05-04T18:00:08Z
date_finished: 2026-05-04T18:00:08Z
---

# T-1496: agent overview --watch — live fleet dashboard

## Context

T-1495 ships `agent overview` as a one-shot session-start digest.
The natural extension — leave it open as a live fleet dashboard —
follows the same `--watch` pattern established by T-1486 (presence)
and T-1494 (on-thread): ANSI clear-home, per-tick header, non-fatal
fetch errors, --watch+--json incompatible. Pure UX extension; the
existing `cmd_agent_overview` body is already idempotent. Deliverable:
a "leave running on a side monitor" live fleet dashboard for an
operator coordinating a multi-agent ring.

## Acceptance Criteria

### Agent
- [x] `--watch` flag added to `agent overview` (clap parses via `--help`)
- [x] `--watch-interval N` flag (default 5, clamped to [1, 300]) — same convention as agent presence/on-thread --watch
- [x] When set: ANSI clear-home + per-tick watch-mode header + body re-render every interval until Ctrl-C
- [x] Fetch errors per-tick are NON-fatal (printed and retried)
- [x] Per-tick header: `# agent overview --watch | interval=Ns | window=Xs | top=N | <RFC3339-ts>`
- [x] `--watch` + `--json` REJECTED (incompatible); error envelope `{"ok":false,"error":"--watch and --json are incompatible: ..."}` exits 1
- [x] When unset: behavior identical to T-1495 baseline (regression-checked: text mode prints same 3 sections)
- [x] Refactor: extracted body of `cmd_agent_overview` into `render_overview_body` (text) and `compose_overview_json` (JSON) — both pure helpers shared between one-shot and watch paths
- [x] `cargo build --release -p termlink` clean
- [x] No new unit tests required (pure UX extension; underlying helpers covered by 32 existing tests)
- [x] Live smoke: `agent overview --watch --watch-interval 2 --window-secs 86400 --top 2` clears screen, prints header + 3 sections, refreshes every 2s (verified 2 ticks captured in 4s window)

### Human
- [ ] [REVIEW] Verify the live overview is steady and useful as a "leave it running" dashboard
  **Steps:**
  1. `target/release/termlink agent overview --watch --watch-interval 5 --window-secs 86400 --top 5` (run from /opt/termlink); let it tick 2-3 times; Ctrl-C
  **Expected:** Each tick redraws cleanly without flicker; recent-posts section reveals new fleet activity as it arrives
  **If not:** suggest layout / interval changes

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

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent overview --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent overview --help 2>&1 | grep -q -- "--watch-interval"
target/release/termlink agent overview --watch --json 2>&1 | grep -qiE "incompatible|--watch.*--json|--json.*--watch"
out=$(timeout 7 target/release/termlink agent overview --watch --watch-interval 2 --window-secs 86400 --top 2 2>&1 || true); echo "$out" | grep -q "agent overview --watch"

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

## Recommendation

**Recommendation:** GO

**Rationale:** Closes "live fleet dashboard" as a single-flag invocation on the digest verb — operator can `agent overview --watch` on a side monitor and have continuous fleet visibility without re-typing the command every minute. Mirrors the established `--watch` pattern (T-1486 / T-1494): ANSI clear-home, per-tick watch-mode header, non-fatal fetch errors, --watch+--json incompatible. Refactor extracts overview body into two pure helpers (`render_overview_body` + `compose_overview_json`) so one-shot and watch paths stay in lockstep.

**Evidence:**
- Live one-shot: `agent overview --window-secs 86400 --top 2` → 3 sections + footer (regression check vs T-1495 baseline — same shape)
- Live watch: 2-tick capture in 4s timeout window; ANSI `\x1b[2J\x1b[H` per tick; watch header includes interval/window/top/RFC3339-ts
- Live --watch + --json: clean error envelope, exit 1
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Two helpers (render_overview_body + compose_overview_json) vs body-only refactor
- **Chose:** Two pure helpers — text body and JSON envelope — both consume `&[Value]` (msgs slice).
- **Why:** Watch loop only needs the text body (--json is rejected with --watch). One-shot needs both paths. Two helpers cleanly separate concerns and avoid having a single helper that branches on `json: bool`.
- **Rejected:** Single combined helper with `output_format: enum {Text, Json}` — leaks render concern into a data-shape helper.

## Updates

### 2026-05-04T18:00:08Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)

### 2026-05-04T17:53:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1496-agent-overview---watch--live-fleet-dashb.md
- **Context:** Initial task creation

### 2026-06-13T13:42:27Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `timeout 6 target/release/termlink agent overview --watch --watch-interval 5 --window-secs 86400 --top 5`
- **Result:** exit=124 (timeout=success-with-partial-output); ok
- **Output:**
  ```
  [2J[H# agent overview --watch | interval=5s | window=86400s | top=5 | 2026-06-13T13:40:55Z
  ## Top Peers (window=86400s, top=5)
  PEER_FP                 LAST_SEEN    POSTS  TOP_PROJECT
  d1993c2c3ec44c94          23m ago       19  010-termlink
  9219671e28054458           3h ago        2  proxmox-ring20-management
  
  ## Top Projects (window=86400s, top=5)
  PROJECT                     POSTS    PEERS TOP_PEER            LAST_SEEN
  010-termlink                   18        1 d1993c2c3ec44c94    23m ago
  100-Video-riper-and-translation-app        1        1 d1993c2c3ec44c94    1h ago
  proxmox-ring20-management        1        1 9219671e28054458    4h ago
  termlink                        1        1 9219671e28054458    3h ago
  
  ## Recent Posts (window=86400s, top=5)
  [3h ago] peer=d1993c2c3ec4 msg_type=chat thread=T-1438 project=010-termlink
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
