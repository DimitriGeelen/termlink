---
id: T-1312
name: "fw metrics api-usage --json: machine-readable output for dashboards"
description: >
  fw metrics api-usage --json: machine-readable output for dashboards

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-1166, T-1304-followup, metrics, json, observability]
components: [.agentic-framework/agents/metrics/api-usage.sh]
related_tasks: [T-1304, T-1308, T-1309, T-1166]
created: 2026-04-27T12:46:13Z
last_update: 2026-04-27T12:46:13Z
date_finished: null
---

# T-1312: fw metrics api-usage --json: machine-readable output for dashboards

## Context

`fw metrics api-usage` outputs a human-readable table — fine for terminals,
useless for dashboards, watchtower pages, monitoring scrapers, or cron-aggregator
scripts. Today consumers would have to parse the table, which is brittle.

Add `--json` flag that emits structured output. Stable shape — once shipped,
downstream integrations depend on it. Default human-readable output is unchanged.

```json
{
  "audit_file": "/var/lib/termlink/rpc-audit.jsonl",
  "mode": "trend",
  "gate_pct": 1.0,
  "malformed_lines": 0,
  "windows": [
    {"days": 1, "total": 812, "legacy": 0, "legacy_pct": 0.0, "passing": true},
    {"days": 7, "total": 5904, "legacy": 3, "legacy_pct": 0.05, "passing": true},
    {"days": 30, "total": 18203, "legacy": 12, "legacy_pct": 0.07, "passing": true},
    {"days": 60, "total": 33421, "legacy": 29, "legacy_pct": 0.09, "passing": true}
  ],
  "top_methods": [
    {"method": "channel.post", "count": 28000, "pct": 83.8, "is_legacy": false},
    ...
  ],
  "legacy_callers": [
    {"method": "event.broadcast", "from": "framework-agent", "count": 23},
    {"method": "inbox.list", "from": "ring20-mgmt", "count": 4},
    {"method": "event.broadcast", "from": "(unknown)", "count": 2}
  ],
  "gate": {"window_days": 60, "passing": true}
}
```

Single-window mode emits the same shape with `mode: "single-window"` and
`windows` containing a single entry for the requested `--last-Nd N`. Exit code
preserved (0 = passing, 1 = failing) so existing CI consumers don't break.

Pure additive. No flag conflicts. JSON output goes to stdout; only on
errors (audit file missing, etc.) JSON-mode emits a `{"error": "..."}` envelope
to stdout AND non-zero exit, while human mode keeps the existing stderr path.

## Acceptance Criteria

### Agent
- [x] `--json` flag accepted in argument parsing; emits machine-readable output to stdout
- [x] Trend mode JSON shape: `{ audit_file, mode: "trend", gate_pct, malformed_lines, windows[], top_methods[], legacy_callers[], gate{} }`
- [x] Single-window mode JSON shape: same with `mode: "single-window"` and one entry in `windows`
- [x] Exit code preserved across both human and JSON output paths (0 = passing 60d-or-window, 1 = failing or audit missing)
- [x] When audit file is missing in `--json` mode, emit `{"error": "audit file not found", "audit_file": "..."}` to stdout and exit 1 (human mode behavior unchanged: error to stderr)
- [x] All Python `print()` calls in human mode are unchanged — JSON path is gated on the `--json` flag, not interleaved
- [x] At least 1 manual probe with synthetic input shows valid JSON output that `python3 -c "import sys, json; json.load(sys.stdin)"` accepts
- [x] `docs/operations/api-usage-metrics.md` documents the `--json` flag and shape
- [x] Help text in `usage()` block lists `--json` with one-line description
- [x] `bash -n .agentic-framework/agents/metrics/api-usage.sh` (syntax check) passes

## Verification

bash -n .agentic-framework/agents/metrics/api-usage.sh
grep -q -- '--json' .agentic-framework/agents/metrics/api-usage.sh
grep -q '"audit_file"' .agentic-framework/agents/metrics/api-usage.sh
grep -q "json" docs/operations/api-usage-metrics.md
test -x .agentic-framework/agents/metrics/api-usage.sh

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

### 2026-04-27T12:46:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1312-fw-metrics-api-usage---json-machine-read.md
- **Context:** Initial task creation

### 2026-04-27T13:05Z — build delivered [agent autonomous pass]
- **Flag:** `--json` accepted in argument parsing; threaded through as 4th positional arg (`json_out` boolean) to the embedded Python.
- **Helpers:** Two pure builders `build_top_methods(counts, total)` and `build_legacy_callers(legacy_callers)` shared between trend and single-window JSON paths.
- **Trend JSON:** `{audit_file, mode:"trend", gate_pct, malformed_lines, windows[1d,7d,30d,60d], top_methods, legacy_callers, gate{window_days:60, passing}}`. Exit 0 if 60d passes, 1 otherwise.
- **Single-window JSON:** Same shape with `mode:"single-window"`, `windows` length 1 for the requested `--last-Nd N`, `gate.window_days` matches.
- **Error envelope:** Missing audit file in JSON mode emits `{"error":"audit file not found","audit_file":"..."}` to stdout (consumers parse stdout only) and exits 1. Human mode unchanged: error to stderr.
- **Verification (P-011 gate):** `bash -n` ✓; manual probes show valid JSON parsed by `python3 -m json.tool`; exit codes correct (0 when passing, 1 when failing or missing). All 5 verification grep/test commands pass.
- **Docs:** `docs/operations/api-usage-metrics.md` gained the `--json` flag row and a "JSON output (`--json`)" subsection with the full shape and exit semantics.
- All Agent ACs ticked.
