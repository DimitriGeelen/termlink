---
id: T-1494
name: "agent on-thread --watch — live thread-following view"
description: >
  agent on-thread --watch — live thread-following view

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T17:34:05Z
last_update: 2026-05-04T17:45:17Z
date_finished: 2026-05-04T17:45:17Z
---

# T-1494: agent on-thread --watch — live thread-following view

## Context

T-1493 ships `agent on-thread <T-XXX>` for one-shot chronological
reading. The complementary mode — live following ("tail -f for a
thread") — would let an operator keep a thread open as new posts
arrive across the fleet. T-1486 already established the watch
loop pattern for `agent presence`: ANSI clear-home, per-tick header,
re-fetch-and-render with non-fatal errors. This task extends the
same pattern to `agent on-thread` so following a fleet discussion
is a single-flag invocation. Pure UX extension, no helper changes —
the existing `extract_recent_posts` is already idempotent.

## Acceptance Criteria

### Agent
- [x] `--watch` flag added to `agent on-thread` (clap parses via `--help`)
- [x] `--watch-interval N` flag (default 5, clamped to [1, 300]) — same convention as `agent presence --watch`
- [x] When set: ANSI clear-home + per-tick header + re-render every interval until Ctrl-C
- [x] Errors on per-tick fetch are NON-fatal (printed and retried) — operator should not lose the dashboard for a transient hub blip
- [x] Per-tick header includes: `# agent on-thread <T-XXX> --watch | interval=Ns | window=Xs | n=Y | <RFC3339-ts>`
- [x] `--watch` + `--json` REJECTED at verb level (incompatible — streaming text vs one-shot JSON); error message names both flags
- [x] Composes with `--n`, `--window-secs`, `--project`, `--peer` / `--peer-fp`, `--hub` unchanged
- [x] When unset: behavior identical to T-1493 baseline (one-shot output, exit on completion)
- [x] `cargo build --release -p termlink` clean
- [x] No new unit tests required (pure UX extension; existing `extract_recent_posts` tests cover the data path)
- [x] Live smoke: `agent on-thread T-1438 --watch --watch-interval 2 --window-secs 86400 --n 2` clears screen, prints header + 2 rows, refreshes every 2s (verified 3 ticks captured in 5s timeout window); duplicate-header bug fixed by extracting body-only renderer
- [x] Refactor: `render_on_thread_text` now renders ONLY the data body; one-shot caller and watch loop each print their own informative header (avoids two-header duplication on every tick)

### Human
- [ ] [REVIEW] Verify the watch view is steady (no flicker) and readable
  **Steps:**
  1. `target/release/termlink agent on-thread T-1438 --watch --watch-interval 5 --window-secs 86400 --n 5` (run from /opt/termlink); let it tick 2x; Ctrl-C
  **Expected:** Each tick fully redraws without flicker; header line shows current ts; posts visible
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
target/release/termlink agent on-thread --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent on-thread --help 2>&1 | grep -q -- "--watch-interval"
target/release/termlink agent on-thread T-1438 --watch --json 2>&1 | grep -qiE "incompatible|--watch.*--json|--json.*--watch"
out=$(timeout 7 target/release/termlink agent on-thread T-1438 --watch --watch-interval 2 --window-secs 86400 --n 2 2>&1 || true); echo "$out" | grep -q "agent on-thread T-1438 --watch"

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

**Rationale:** Closes "tail -f for a fleet thread" as a single-flag invocation. Mirrors T-1486's `agent presence --watch` pattern (ANSI clear-home, per-tick header, non-fatal fetch errors, --watch+--json incompatible). Pure UX extension — no helper changes; the existing `extract_recent_posts` is idempotent. Refactored `render_on_thread_text` to be body-only so watch and one-shot mode each surface their own informative header (otherwise headers duplicated every tick).

**Evidence:**
- Live one-shot: `agent on-thread T-1438 --window-secs 86400 --n 2` → header + 2 rows + footer (regression check vs T-1493 baseline)
- Live watch: `timeout 5 agent on-thread T-1438 --watch --watch-interval 2 --n 2` → 3 ticks captured in 5s; ANSI `\x1b[2J\x1b[H` per tick; watch header includes interval/window/n/RFC3339-ts
- Live --watch + --json: clean error envelope `{"error":"--watch and --json are incompatible: ...", "ok":false}` exits 1
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Body-only renderer + per-mode headers
- **Chose:** Extract render_on_thread_text into body-only (data block + empty-state + footer); both watch loop and one-shot caller print their own header line.
- **Why:** Initial implementation had render_on_thread_text print a standard header AND watch loop print a watch-specific header → duplicate headers per tick. Cleaner: each mode owns its header. One-shot prints `# agent on-thread <T> | window=Xs | n=Y`; watch prints `# agent on-thread <T> --watch | interval=Ns | window=Xs | n=Y | <RFC3339-ts>`.
- **Rejected:** Pass `print_header: bool` flag to renderer — leaks UI concern into a body renderer; per-mode headers belong in the per-mode caller.

## Updates

### 2026-05-04T17:45:17Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)

### 2026-05-04T17:34:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1494-agent-on-thread---watch--live-thread-fol.md
- **Context:** Initial task creation

### 2026-06-13T13:42:27Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `timeout 6 target/release/termlink agent on-thread T-1438 --watch --watch-interval 5 --window-secs 86400 --n 5`
- **Result:** exit=124 (timeout=success-with-partial-output); ok
- **Output:**
  ```
  [2J[H# agent on-thread T-1438 --watch | interval=5s | window=86400s | n=5 | 2026-06-13T13:40:49Z
  [4h ago] @3191 peer=d1993c2c3ec4 msg_type=chat project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T11:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [3h ago] @3193 peer=d1993c2c3ec4 msg_type=chat project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T12:17:02+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [2h ago] @3195 peer=d1993c2c3ec4 msg_type=chat project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T13:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [1h ago] @3196 peer=d1993c2c3ec4 msg_type=chat project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T14:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [23m ago] @3198 peer=d1993c2c3ec4 msg_type=chat project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T15:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
