---
id: T-1491
name: "agent presence --by-project — fleet activity aggregated by project"
description: >
  agent presence --by-project — fleet activity aggregated by project

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T17:00:50Z
last_update: 2026-05-04T17:10:54Z
date_finished: 2026-05-04T17:10:54Z
---

# T-1491: agent presence --by-project — fleet activity aggregated by project

## Context

T-1482/1484/1486/1489/1490 ship `agent presence` for fleet-wide
**peer-aggregated** observability. The complementary question — "what
projects is the fleet active on?" — currently requires
`agent presence --json | jq` per-project tally. This task adds
`--by-project` to flip the aggregation: instead of one row per peer,
one row per project, with posts/distinct-peers/top-peer/last-seen.
Pure helper addition that walks the same msgs slice, no new wire
protocol. Composes with `--filter-project`, `--thread`, `--top N`,
`--watch`, `--json`.

## Acceptance Criteria

### Agent
- [x] `--by-project` flag added to `agent presence` (clap parses via `--help`)
- [x] When set: output is one row per project with posts/peers/top_peer/last_seen
- [x] When unset: behavior identical to T-1490 baseline (per-peer rows; backward-compat)
- [x] `summarize_fleet_by_project` helper added: `(msgs, now_ms, window_ms, filter_project, filter_thread) -> Vec<FleetProjectRow>`
- [x] FleetProjectRow has: project, posts, distinct_peers (count), top_peer_fp, last_seen_ms
- [x] Sort: posts desc, then project asc (alpha tie-break)
- [x] Untagged posts are excluded entirely (a `from_project` tag is required for by-project aggregation)
- [x] JSON envelope: when `--by-project` set, `peers` key replaced with `projects` (array of FleetProjectRow); `view: "by-project"` field added so callers can disambiguate
- [x] Text mode: PROJECT/POSTS/PEERS/TOP_PEER/LAST_SEEN columns; footer "(N of M)" truncation honored when --top set
- [x] Composes with `--top`, `--filter-project`, `--thread`, `--watch`, `--hub` unchanged (watch loop branches on by_project)
- [x] 6 unit tests: empty / single-project-multi-peer / multi-project-sort / untagged-excluded / filter-thread-applied / meta-skipped
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink fleet_by_project` passes (6/6 new)
- [x] Live smoke: `agent presence --by-project --window-secs 86400` emits 3-project table (010-termlink/002-Claude-Partner-Network/user-override-val); JSON envelope shape verified

### Human
- [ ] [REVIEW] Verify the by-project table is operator-readable
  **Steps:**
  1. `target/release/termlink agent presence --by-project --window-secs 86400` (run from /opt/termlink)
  **Expected:** Table is scannable with PROJECT in leftmost column; footer reports project count + window
  **If not:** suggest column-order or wording changes

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink fleet_by_project 2>&1 | grep -qE "test result: ok\. ([5-9]|[1-9][0-9])"
target/release/termlink agent presence --help 2>&1 | grep -q -- "--by-project"
out=$(target/release/termlink agent presence --by-project --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('view') == 'by-project', d; assert isinstance(d.get('projects'), list); assert 'peers' not in d"
out=$(target/release/termlink agent presence --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'view' not in d, 'unset --by-project must not emit view'; assert isinstance(d.get('peers'), list)"

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

**Rationale:** Closes the by-peer/by-project aggregation gap on `agent presence`. Pure helper extension paralleling `summarize_fleet_presence` — same msgs slice, same window/filter semantics, just keyed by `from_project` instead of `sender_id`. JSON envelope cleanly disambiguates via `view: "by-project"` + `projects:[]` (vs. default `peers:[]`). Untagged posts excluded by design — a project tag is the aggregation key. Composes with all existing flags including `--watch` (loop branches on by_project).

**Evidence:**
- Live: `agent presence --by-project --window-secs 86400` → 3-row table: 010-termlink (33 posts), 002-Claude-Partner-Network (1), user-override-val (1)
- Live JSON `--by-project`: envelope keys = `[projects, view, window_secs]`; `view = "by-project"`; `projects` is sorted-by-posts array
- Live JSON unset: keys = `[peers, window_secs]`, no `view` field (backward-compat)
- Unit tests: 6/6 `fleet_by_project_*` pass — covers empty / aggregate / sort / untagged-excluded / filter-thread / meta-skipped
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Untagged posts excluded unconditionally
- **Chose:** In by-project view, posts without `from_project` are dropped entirely (not bucketed under "(untagged)" or similar).
- **Why:** Project IS the aggregation key — a row labelled "(untagged)" would be a different shape than other rows and the `top_peer_fp` would be misleading (it'd be "the peer with most untagged posts" rather than "the peer most active on this project"). Heartbeats and meta-traffic don't carry project tags by design; including them would dilute the signal.
- **Rejected:** "(untagged)" pseudo-row — would conflate noise with signal; operator can fall back to default by-peer view if they need to see untagged activity.

## Updates

### 2026-05-04T17:00:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1491-agent-presence---by-project--fleet-activ.md
- **Context:** Initial task creation

### 2026-05-04T17:10:54Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)

### 2026-06-13T13:41:08Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps to capture fresh output (>2wk since build smoke)
- **Command(s):** `target/release/termlink agent presence --by-project --window-secs 86400`
- **Result:** exit=0; ok — by-project table, PROJECT leftmost, footer reports 4 projects + window
- **Output:**
  ```
  $ target/release/termlink agent presence --by-project --window-secs 86400
  PROJECT                     POSTS    PEERS TOP_PEER            LAST_SEEN
  010-termlink                   18        1 d1993c2c3ec44c94    23m ago
  100-Video-riper-and-translation-app        1        1 d1993c2c3ec44c94    1h ago
  proxmox-ring20-management        1        1 9219671e28054458    4h ago
  termlink                        1        1 9219671e28054458    3h ago
  
  4 project(s) active in window=86400s
  [exit=0]
  ```
- **Note:** Human [REVIEW] AC remains UNCHECKED — sovereignty; evidence provided for batch-confirm.
