---
id: T-1498
name: "agent recent --watch — live single-peer streaming view"
description: >
  Add --watch flag to agent recent (T-1492) — symmetric to T-1494 (on-thread --watch) and T-1496 (overview --watch). Closes the watch trio: presence/on-thread/overview/recent all stream live. Single-peer streaming is the missing primitive for an operator who wants a live feed of one peer (e.g. babysitting a long-running build). Pure UX extension; underlying extract_recent_posts pure helper already shipped.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T19:54:18Z
last_update: 2026-05-04T20:08:00Z
date_finished: 2026-05-04T20:08:00Z
---

# T-1498: agent recent --watch — live single-peer streaming view

## Context

T-1492 ships `agent recent <peer>` as a one-shot view of last N posts
from a single peer. T-1494 ships `agent on-thread --watch` and T-1496
ships `agent overview --watch` — both follow the same pattern: ANSI
clear-home, per-tick header, non-fatal fetch errors, --watch+--json
incompatible. This task closes the watch trio with `agent recent
--watch`, the missing single-peer streaming primitive. Use case:
operator babysitting a long-running build/agent-thread on a specific
peer wants a continuously-refreshing tail without re-typing the verb.
Pure UX extension; underlying `extract_recent_posts` helper is unchanged.

## Acceptance Criteria

### Agent
- [x] `--watch` flag added to `agent recent` (clap parses via `--help`)
- [x] `--watch-interval N` flag (default 5, clamped to [1, 300]) — same convention as T-1494/T-1496
- [x] When set: ANSI clear-home + per-tick watch-mode header + body re-render every interval until Ctrl-C
- [x] Fetch errors per-tick are NON-fatal (printed and retried on next tick)
- [x] Per-tick header: `# agent recent <target> --watch | peer_fp=<fp> | interval=Ns | window=Xs | n=N | <RFC3339-ts>` (composes with thread/project filter suffix when set)
- [x] `--watch` + `--json` REJECTED (incompatible); error envelope `{"ok":false,"error":"--watch and --json are incompatible: ..."}` exits 1
- [x] When unset: behavior identical to T-1492 baseline (same 3 sections regression-checked: header / posts / footer)
- [x] Refactor: extracted body of `cmd_agent_recent` into `render_recent_body` (text) — shared between one-shot and watch paths
- [x] `cargo build --release -p termlink` clean
- [x] No new unit tests required (pure UX extension; `extract_recent_posts` covered by 11 existing tests from T-1492/T-1493)
- [x] Live smoke: `agent recent --target-fp d1993c2c3ec44c94 --watch --watch-interval 2 --window-secs 86400 --n 2` clears screen, prints header + posts, refreshes every 2s (verified 3 ticks captured in 4s window: 2026-05-04T20:05:16/18/20Z)

### Human
- [ ] [REVIEW] Verify the live single-peer view is steady and useful for "babysit one peer"
  **Steps:**
  1. `target/release/termlink agent recent <target-fp> --watch --watch-interval 5 --window-secs 86400 --n 5` (run from /opt/termlink); let it tick 2-3 times; Ctrl-C
  **Expected:** Each tick redraws cleanly without flicker; new posts from this peer appear at the bottom as they arrive
  **If not:** suggest layout / interval changes

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent recent --help 2>&1 | grep -q -- "--watch"
target/release/termlink agent recent --help 2>&1 | grep -q -- "--watch-interval"
target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --watch --json 2>&1 | grep -qiE "incompatible|--watch.*--json|--json.*--watch"
out=$(timeout 7 target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --watch --watch-interval 2 --window-secs 86400 --n 2 2>&1 || true); echo "$out" | grep -q "agent recent .* --watch"

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

**Rationale:** Closes the watch trio (presence/on-thread/overview/recent all stream live) with a single-flag invocation on the per-peer verb. Operator can `agent recent <peer> --watch` on a side monitor and have continuous visibility on one peer's posts without re-typing the verb each tick. Mirrors the established `--watch` pattern (T-1486 / T-1494 / T-1496): ANSI clear-home, per-tick watch-mode header, non-fatal fetch errors, --watch+--json incompatible. Refactor extracts recent body into a pure `render_recent_body` helper so one-shot and watch paths stay in lockstep.

**Evidence:**
- Live one-shot: `agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --n 3` → header + 3 post blocks + footer (regression check vs T-1492 baseline — same shape)
- Live watch: 3-tick capture in 4s timeout window; ANSI `\x1b[2J\x1b[H` per tick; watch header includes target/peer_fp/interval/window/n/RFC3339-ts (timestamps differ each tick: 20:05:16/18/20)
- Live --watch + --json: clean error envelope, exit 1
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Body-only render helper vs combined header+body
- **Chose:** `render_recent_body(posts, now_ms)` does data block + footer only; caller (watch loop OR one-shot) prints its own header.
- **Why:** Watch and one-shot need different header formats (watch shows interval + RFC3339 ts; one-shot shows the static command echo). Putting both in one helper would force a `mode: enum` parameter or duplicated header logic.
- **Rejected:** Single combined helper with a `is_watch: bool` parameter — leaks rendering policy into a body helper. Same call-site pattern as T-1494 (`render_on_thread_text` is also body-only).

## Updates

### 2026-05-04T19:54:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1498-agent-recent---watch--live-single-peer-s.md
- **Context:** Initial task creation

### 2026-05-04T20:08:00Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)

### 2026-06-13T13:42:27Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `timeout 6 target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --watch --watch-interval 5 --window-secs 86400 --n 5`
- **Result:** exit=124 (timeout=success-with-partial-output); ok
- **Output:**
  ```
  [2J[H# agent recent d1993c2c3ec44c94 --watch | peer_fp=d1993c2c3ec44c94 | interval=5s | window=86400s | n=5 | 2026-06-13T13:41:12Z
  [3h ago] @3193 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T12:17:02+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [2h ago] @3195 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T13:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [1h ago] @3196 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T14:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  
  [1h ago] @3197 msg_type=note thread=arc-008 project=100-Video-riper-and-translation-app
      [100-Video-riper-and-translation-app → fleet] GPU window request (RTX 5060 Ti, 16GB shared). Whoever is holding gemma4:latest resident (~10.5GB) on this host: could you free it for a ~30-min window? I…
  
  [24m ago] @3198 msg_type=chat thread=T-1438 project=010-termlink
      T-1438 vendored-arc heartbeat from dimitrimintdev (x86_64, Linux) at 2026-06-13T15:17:01+02:00. Binary: /usr/local/bin/termlink (termlink 0.9.1542).
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
