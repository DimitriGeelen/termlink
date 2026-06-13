---
id: T-1487
name: "agent ping <target> — operator-facing presence check verb"
description: >
  agent ping <target> — operator-facing presence check verb

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T15:40:03Z
last_update: 2026-05-04T15:46:30Z
date_finished: 2026-05-04T15:46:30Z
---

# T-1487: agent ping <target> — operator-facing presence check verb

## Context

T-1480 / T-1481 / T-1482 / T-1483 ship the observability primitives
(presence check, peer activity, fleet view, name resolution). For the
classic operator question — "is alice alive?" — the existing path is
either `agent who --target alice` (verbose: full activity dump) or
`agent presence | grep <fp>` (requires fp). Neither is the natural verb.

T-1487 adds `termlink agent ping <target>` as the canonical "is X alive
on chat-arc?" verb. Non-invasive (no posts, just chat-arc heartbeat
inspection). Single-line output. Exit 0 online / exit 1 offline.
Composes the existing `check_peer_online_via_chat_arc` (T-1480) +
local `manager::find_session` (T-1483) primitives — pure UX win.

## Acceptance Criteria

### Agent
- [x] `termlink agent ping <target>` parses via `--help`; positional arg required (or `--target-fp`)
- [x] `--target-fp <hex>` alternative; mutually exclusive with positional `<TARGET>`; one is required
- [x] `--window-secs N` (default 300, clamped [10, 86400] — same range as `--require-online`)
- [x] `--hub <addr>` and `--json` flags supported
- [x] On online (peer seen within window): exit 0; non-JSON prints `<peer-or-target> (<fp16>): online — last seen <age>`; JSON `{"target_or_fp": "...", "peer_fp": "...", "online": true, "last_seen_ms": ..., "window_secs": N}`
- [x] On offline: exit 1; non-JSON `... offline — last seen <age> | never`; JSON same shape with `online: false`
- [x] Pre-T-1436 peer (no identity_fingerprint when using `<TARGET>`): exit 8 with upgrade hint (mirror cmd_agent_who/contact via shared resolve_target_name_to_fp)
- [x] Target not found locally (when using `<TARGET>`): exit 1, names the missing session (mirror via resolve_target_name_to_fp)
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke positive: `agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400` exit 0, "online — last seen 28m ago"
- [x] Live smoke negative: `agent ping --target-fp deadbeefdeadbeef --window-secs 60` exit 1, "offline — last seen never"

### Human
- [ ] [REVIEW] Verify the one-liner output is operator-scannable
  **Steps:**
  1. `target/release/termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400` (run from /opt/termlink)
  2. `target/release/termlink agent ping --target-fp deadbeefdeadbeef --window-secs 60` (run from /opt/termlink)
  3. Compare output
  **Expected:** both lines fit on one row, online/offline distinction is obvious at a glance, you'd reach for this before `agent who` for the simple "alive?" question
  **If not:** describe what's hard to scan, suggest concrete improvement

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent ping --help 2>&1 | grep -q -- "--target-fp"
out=$(target/release/termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400 2>&1); echo "$out" | grep -qE "online|offline"
out=$(target/release/termlink agent ping --target-fp deadbeefdeadbeef --window-secs 60 2>&1 || true); echo "$out" | grep -qE "offline"
out=$(target/release/termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d['online'] is True, d; assert isinstance(d['peer_fp'], str)"

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

**Rationale:** The canonical operator "is X alive?" verb. Composes existing primitives (T-1480 `check_peer_online_via_chat_arc` + T-1483 `resolve_target_name_to_fp`) — no new wire protocol, no new helpers, just a clean UX wrapper. Single-line output is operator-scannable. Exit 0/1 makes it scriptable (`agent ping alice && tail-the-log`). Non-invasive (no posts on dm topic, just chat-arc heartbeat inspection) so it doesn't pollute the agent-to-agent record.

**Evidence:**
- Online live: `agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400` → exit 0, `d1993c2c3ec44c94 (d1993c2c3ec44c94): online — last seen 28m ago (window=86400s)`
- Offline live: `agent ping --target-fp deadbeefdeadbeef --window-secs 60` → exit 1, `... offline — last seen never (window=60s)`
- JSON shape: `{"target_or_fp", "peer_fp", "online", "last_seen_ms", "last_seen", "window_secs", "posts_in_window"}` (7 fields, all populated)
- Verification: 5/5 commands pass.

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

### 2026-05-04T15:40:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1487-agent-ping-target--operator-facing-prese.md
- **Context:** Initial task creation

### 2026-05-04T15:46:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-06-13T13:41:08Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `target/release/termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400`; `target/release/termlink agent ping --target-fp deadbeefdeadbeef --window-secs 60`
- **Result:** exit=0,1; ok — online (exit=0) vs offline (exit=1) clearly distinguished, single-line each
- **Output:**
  ```
  $ target/release/termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400
  d1993c2c3ec44c94 (d1993c2c3ec44c94): online — last seen 23m ago (window=86400s)
  [exit=0]
  $ target/release/termlink agent ping --target-fp deadbeefdeadbeef --window-secs 60
  deadbeefdeadbeef (deadbeefdeadbeef): offline — last seen never (window=60s)
  [exit=1]
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
