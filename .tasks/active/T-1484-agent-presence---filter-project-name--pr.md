---
id: T-1484
name: "agent presence --filter-project <name> — project-scoped fleet view"
description: >
  agent presence --filter-project <name> — project-scoped fleet view

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T14:51:25Z
last_update: 2026-05-04T15:00:35Z
date_finished: 2026-05-04T15:00:35Z
---

# T-1484: agent presence --filter-project <name> — project-scoped fleet view

## Context

T-1482 shipped fleet-wide `agent presence` showing every active peer. For
fleet-wide observability that's right; for project-scoped triage ("who's
working on 010-termlink right now?") the operator needs a project filter.
This task adds `--filter-project <name>` to scope the aggregation: only
posts whose `metadata.from_project` matches are counted toward
posts/last_seen, peers with zero matching posts in the window are dropped,
and `top_project` is always the filter (or absent if peer has no tagged
posts). Pure helper extension; no hub changes.

## Acceptance Criteria

### Agent
- [x] `--filter-project <name>` flag added to `agent presence` (clap parses via `--help`)
- [x] When set: only posts with `metadata.from_project == <name>` count toward presence; peers with zero matching in-window posts are excluded
- [x] When unset: behavior identical to T-1482 baseline (regression-safe)
- [x] `summarize_fleet_presence` helper extended with `filter_project: Option<&str>` parameter (signature change with all callers updated)
- [x] Empty result with filter: text mode prints "(no peers active in window matching project=<name>)"; JSON returns empty peers array with filter echoed in envelope
- [x] JSON envelope includes `filter_project` field when filter is set (omitted otherwise to stay backward-compatible)
- [x] 4+ unit tests for filter behavior: filter-matches / filter-excludes / filter-with-no-tagged-posts / filter-changes-top-project
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink fleet_presence` passes (>=11 tests — 7 baseline + 4 new)
- [x] Live smoke against own hub: `--filter-project 010-termlink` returns ≥1 peer; `--filter-project nonexistent-xyz` returns empty with the filter-mention message

### Human
- [ ] [REVIEW] Verify the empty-with-filter message reads naturally
  **Steps:**
  1. `target/release/termlink agent presence --filter-project nonexistent-xyz` (run from /opt/termlink)
  2. Observe the empty-state message
  **Expected:** the message names the filter so operator knows the filter was applied (vs. fleet truly being silent)
  **If not:** describe the unclear wording, suggest concrete improvement

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink fleet_presence 2>&1 | grep -qE "test result: ok\. (1[1-9]|[2-9][0-9])"
target/release/termlink agent presence --help 2>&1 | grep -q "filter-project"
out=$(target/release/termlink agent presence --filter-project 010-termlink --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('filter_project') == '010-termlink', d; assert isinstance(d['peers'], list); assert len(d['peers']) >= 1, d"
out=$(target/release/termlink agent presence --filter-project nonexistent-xyz-no-such --window-secs 86400 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('filter_project') == 'nonexistent-xyz-no-such', d; assert d['peers'] == [], d"

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

**Rationale:** Project-scoped fleet view ships cleanly. Filter applied at helper level so unit tests cover the full surface (4 new tests, 11/11 fleet_presence pass). Live smoke confirms positive case (1 peer matching `010-termlink` with 33 posts), negative case (filter-mention empty message), and JSON envelope (`filter_project` echoed). Backward-compatible — when filter unset, behavior identical to T-1482 baseline (and JSON envelope omits the field).

**Evidence:**
- Live invocations:
  - `agent presence --filter-project 010-termlink --window-secs 86400` → 1 peer (`d1993c2c3ec44c94`, 33 posts, `42m ago`), header line `# filter_project=010-termlink`, footer `1 peer(s) active in window=86400s matching project=010-termlink`
  - `agent presence --filter-project nonexistent-xyz-no-such` → exit 0, `(no peers active in window=86400s matching project=nonexistent-xyz-no-such)`
  - `agent presence --filter-project 010-termlink --json` → JSON envelope with `filter_project: "010-termlink"` + 1-element peers array
- Unit tests: 11/11 `fleet_presence_*` pass (7 baseline + 4 filter)
- Verification: 5/5 commands pass

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

### 2026-05-04T14:51:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1484-agent-presence---filter-project-name--pr.md
- **Context:** Initial task creation

### 2026-05-04T15:00:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
