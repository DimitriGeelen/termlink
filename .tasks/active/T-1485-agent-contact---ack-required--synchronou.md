---
id: T-1485
name: "agent contact --ack-required — synchronous engagement (T-1425 Phase-2 backlog)"
description: >
  agent contact --ack-required — synchronous engagement (T-1425 Phase-2 backlog)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T15:06:54Z
last_update: 2026-05-04T15:21:48Z
date_finished: 2026-05-04T15:21:48Z
---

# T-1485: agent contact --ack-required — synchronous engagement (T-1425 Phase-2 backlog)

## Context

T-1480 added `--require-online` (pre-flight presence check). T-1485 closes
the symmetric Phase-2 question: synchronous engagement *post-send*. After a
successful dm post, optionally wait for the peer to post back on the same
dm topic — proving they not only existed-pre-send but actively engaged.
Defaults to fire-and-forget (T-1429 Phase-1 baseline preserved). When set,
exits 0 on ack within timeout, exit 10 on timeout. Pure helper extracted
for unit tests; integrates cleanly with `--require-online` (combined: full
synchronous semantic).

This closes the last open T-1425 Phase-2 RFC question (Q4: synchronous
engagement). Pairs with T-1480 (Q3, fail-fast pre-flight).

## Acceptance Criteria

### Agent
- [x] `--ack-required` boolean flag added to `agent contact` (clap parses via `--help`)
- [x] `--ack-timeout-secs <N>` flag added (default 60, clamped to [5, 600])
- [x] After successful post: poll the dm topic for a non-meta message from peer_fp with ts > send_ts_ms; return as soon as found
- [x] On ack: exit 0; non-JSON prints "ack received from <peer_fp> after <Ns>"; JSON envelope `{"ok": true, "ack": {"ts_ms": ..., "wait_secs": ...}}`
- [x] On timeout: exit 10 (non-JSON) / exit 1 with `exit_code: 10` field (JSON, established codebase convention); error names peer_fp + timeout + recovery hints
- [x] Pure helper `detect_ack_in_msgs(msgs, peer_fp, send_ts_ms) -> Option<i64>` in commands/channel.rs
- [x] Async helper `wait_for_peer_ack(topic, peer_fp, send_ts_ms, hub, timeout_secs) -> Result<Option<i64>>` polling at ~1s cadence
- [x] Generic `fetch_topic_msgs(topic, hub, slice_size)` extracted from chat-arc helper; `fetch_recent_chat_arc_msgs` now thin wrapper
- [x] Combines correctly with `--require-online`: pre-flight first, then post, then ack-wait
- [x] 5+ unit tests for `detect_ack_in_msgs` (7 actual): empty / no-match-by-fp / ts-before-send / at-exact-ts (strict) / meta-msgs-skipped / first-match-wins / post-after-send-returns-ts
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink detect_ack` passes (7/7)
- [x] Live smoke timeout: `--ack-required --ack-timeout-secs 5` against a never-responding peer-FP returns exit 10 within ~5-6s with operator-actionable error

### Human
- [ ] [REVIEW] Verify timeout error wording is operator-actionable
  **Steps:**
  1. `target/release/termlink agent contact --target-fp deadbeefdeadbeef --message hi --ack-required --ack-timeout-secs 5` (run from /opt/termlink)
  2. Observe the timeout error on stderr
  **Expected:** error names the peer_fp, the timeout, and the next-step (e.g. "rerun without --ack-required to fire-and-forget" or similar)
  **If not:** describe unclear wording, suggest concrete improvement

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink detect_ack 2>&1 | grep -qE "test result: ok\. [5-9]+ passed|test result: ok\. 1[0-9] passed"
target/release/termlink agent contact --help 2>&1 | grep -q "ack-required"
target/release/termlink agent contact --help 2>&1 | grep -q "ack-timeout-secs"
out=$(timeout 15 target/release/termlink agent contact --target-fp deadbeefdeadbeef --message ack-smoke --ack-required --ack-timeout-secs 5 2>&1 || true); echo "$out" | grep -qiE "ack|timeout"

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

**Rationale:** Closes the last open T-1425 Phase-2 RFC question (Q4: synchronous engagement). Pure-helper extraction means `detect_ack_in_msgs` has 7 unit tests covering every edge (strict-`>` boundary, meta filter, sender mismatch, empty slice, multi-match-first-wins). Live timeout test confirms operator-friendly error wording (names peer_fp, topic, timeout, recovery hint pointing to chat-arc durability and `--ack-timeout-secs`). NDJSON-style output for `--json --ack-required` (delivered envelope, then ack envelope) is consistent with existing termlink multi-step composition. Backward-compatible: default fire-and-forget preserved, T-1429 Phase-1 behavior unchanged when flag is absent.

**Evidence:**
- Live invocations (timeout path):
  - `agent contact --target-fp deadbeefdeadbeef --message ack-smoke --ack-required --ack-timeout-secs 5` → exit 10 in 5.0s, error: "no ack from peer fp=deadbeefdeadbeef within 5s on topic=dm:d1993c2c3ec44c94:deadbeefdeadbeef. The post landed (chat-arc is offset-durable) — peer just hasn't responded yet. Re-run without --ack-required for fire-and-forget, or increase --ack-timeout-secs."
  - Same with `--json` → NDJSON: delivered envelope (offset/ts) + ack envelope (`{"ok":false, "exit_code":10, ...}`), exit 1 (JSON convention).
- Unit tests: 7/7 `detect_ack_*` pass (strict-`>` boundary verified, meta filter, multi-msg first-wins).
- Verification: 5/5 commands pass.

**Live success path** (peer ack within timeout): not tested live — fleet has only 1 peer FP and self-DM would self-ack on the just-posted message (strict-`>` boundary doesn't help when peer_fp == my_fp). Detection logic is fully unit-tested.

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

### 2026-05-04T15:06:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1485-agent-contact---ack-required--synchronou.md
- **Context:** Initial task creation

### 2026-05-04T15:21:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:41:08Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `target/release/termlink agent contact --target-fp deadbeefdeadbeef --message hi --ack-required --ack-timeout-secs 5`
- **Result:** exit=1; error — loud named error (retention-policy conflict on pre-existing dm topic; differs from documented timeout path but still names peer_fp & topic, exit=1)
- **Output:**
  ```
  $ target/release/termlink agent contact --target-fp deadbeefdeadbeef --message hi --ack-required --ack-timeout-secs 5
  Error: agent contact: posting to dm topic for peer fp=deadbeefdeadbeef failed
  
  Caused by:
      channel.create failed: JSON-RPC error -32603: channel.create: topic "dm:d1993c2c3ec44c94:deadbeefdeadbeef" already exists with a different retention policy (existing=Forever, requested=Messages(1000))
  [exit=1]
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
