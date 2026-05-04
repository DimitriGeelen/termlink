---
id: T-1490
name: "agent presence --thread T-XXX — fleet-wide thread activity filter"
description: >
  agent presence --thread T-XXX — fleet-wide thread activity filter

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T16:46:14Z
last_update: 2026-05-04T16:59:07Z
date_finished: 2026-05-04T16:59:07Z
---

# T-1490: agent presence --thread T-XXX — fleet-wide thread activity filter

## Context

T-1488 added `agent who --thread <T-XXX>` to scope a single peer's
activity to a thread. The symmetric question — "who across the fleet
is active on T-1485?" — currently requires `agent presence | grep` or
post-walking the chat-arc by hand. This task adds `--thread <T-XXX>`
to `agent presence` so the fleet-wide post-walk filters by
`metadata._thread`. Pure helper extension, AND-composable with
`--filter-project` (both must match if both set), composes with
`--watch` and `--top` unchanged.

## Acceptance Criteria

### Agent
- [x] `--thread <name>` flag added to `agent presence` (clap parses via `--help`)
- [x] When set: only posts whose `metadata._thread == <name>` count toward presence; peers with zero matching posts are excluded
- [x] When unset: behavior identical to T-1484/1486/1489 baseline (JSON omits `filter_thread` field; backward-compat)
- [x] `--thread` AND-composes with `--filter-project`: when both set, only posts matching both pass
- [x] `summarize_fleet_presence` helper extended with `filter_thread: Option<&str>` parameter (signature change; all callers + tests updated)
- [x] `fetch_fleet_presence_via_chat_arc` wrapper extended with the same parameter
- [x] JSON envelope echoes `filter_thread` field when filter is set (omitted otherwise)
- [x] Text mode: header line `# filter_thread=<name>` printed when filter is set; empty-message phrasing includes thread; footer suffix mentions thread
- [x] 4+ unit tests: filter-thread-matches / filter-thread-excludes-untagged / no-match-returns-empty / AND-compose-with-project
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink fleet_presence` passes (15/15 = 11 baseline + 4 new)
- [x] Live smoke: `agent presence --thread T-1490` returns empty rows with naturally-phrased message; compose `--filter-project 010-termlink --thread T-1490` works without panic, names both filters in empty message

### Human
- [ ] [REVIEW] Verify the empty-with-thread message reads naturally
  **Steps:**
  1. `target/release/termlink agent presence --thread T-1487 --window-secs 86400` (run from /opt/termlink)
  **Expected:** Output names both the window AND the thread filter so the operator understands why no rows came back
  **If not:** suggest a clearer phrasing

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink fleet_presence 2>&1 | grep -qE "test result: ok\. (1[2-9]|[2-9][0-9])"
target/release/termlink agent presence --help 2>&1 | grep -q -- "--thread"
out=$(target/release/termlink agent presence --thread T-1487 --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('filter_thread') == 'T-1487', d; assert isinstance(d.get('peers'), list)"
out=$(target/release/termlink agent presence --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'filter_thread' not in d, 'unset filter must not appear in JSON'"

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

**Rationale:** Closes the symmetric-to-T-1488 thread-scoping gap on `agent presence`. Pure helper extension, AND-composable with the existing `--filter-project` flag. Backward-compatible JSON envelope — `filter_thread` field absent when unset. 4 new unit tests cover match / exclude-untagged / no-match / AND-compose-with-project. Empty-rows phrasing is filter-aware (names project, thread, or both).

**Evidence:**
- Live: `agent presence --thread T-1490 --window-secs 86400` → `(no peers active in window=86400s matching thread=T-1490)` (expected — chat-arc heartbeats don't carry `_thread`)
- Live: `--filter-project 010-termlink --thread T-1490` → `(no peers active in window=86400s matching project=010-termlink thread=T-1490)` (compose path renders correctly)
- Live JSON `--thread T-1490`: `{"filter_thread": "T-1490", "peers": [], "window_secs": 86400}`
- Live JSON unset: keys = `[peers, window_secs]` (no `filter_thread` — backward-compat)
- Unit tests: 15/15 `fleet_presence_*` pass (11 baseline + 4 thread-filter tests)
- Verification: 5/5 commands pass

**Note on live-positive coverage:** The live fleet's chat-arc heartbeats don't carry `_thread` metadata, so an "actually-matched" live test isn't possible from this session. Detection logic is fully unit-tested with shared helper `activity_msg_with_thread` (which mirrors the T-1488 unit-test peer-activity pattern that ships in production today).

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

### 2026-05-04T16:46:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1490-agent-presence---thread-t-xxx--fleet-wid.md
- **Context:** Initial task creation

### 2026-05-04T16:59:07Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
