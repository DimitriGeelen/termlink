---
id: T-1469
name: "cut-readiness-daily.sh: emit --trend + --keep-days rotation"
description: >
  cut-readiness-daily.sh: emit --trend + --keep-days rotation

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-04T07:05:20Z
last_update: 2026-05-04T07:07:51Z
date_finished: 2026-05-04T07:07:51Z
---

# T-1469: cut-readiness-daily.sh: emit --trend + --keep-days rotation

## Context

T-1466 wraps `fleet doctor --legacy-usage` for daily cron use. Two follow-ups
land naturally on top of T-1467 + T-1468:

1. **Trend output** — once a snapshots directory has 2+ days, the cron MAILTO
   should carry the multi-day trend (sparkline + trajectory), not just a
   pairwise diff. T-1468 added `--trend <dir>`; threading it through the
   wrapper gives operators the time-series view in their inbox without
   running anything manually.

2. **Snapshot rotation** — the wrapper writes one file per day; without
   cleanup, the directory grows linearly. Add `--keep-days N` (default 90,
   matching `legacy_window_days` ceiling) so older snapshots are pruned
   after each run. Pure shell — no new dependencies.

## Acceptance Criteria

### Agent
- [x] `scripts/cut-readiness-daily.sh` invokes `fleet doctor --legacy-usage --save-snapshot $TODAY_PATH --trend $SNAPSHOTS_DIR --exit-code-on-verdict` on routine runs (replaces / supersedes the previous `--diff $PRIOR_PATH` invocation; `--trend` includes the diff via the trailing point)
- [x] First-run path (no prior snapshot) also calls `--trend` (with just today's value, will show single point — graceful)
- [x] `--keep-days N` flag added: deletes `$SNAPSHOTS_DIR/*.json` files where the basename's date is older than N days back from today. Default 90.
- [x] `--keep-days 0` disables rotation (escape hatch for operators who archive externally)
- [x] Rotation never prunes today's snapshot (invariant: cutoff date < today, today's basename is lex-greater so the comparison passes)
- [x] Help text (`--help`) lists the new flags with defaults
- [x] Rotation only matches files whose basename parses as `YYYY-MM-DD` — operator-placed notes/ad-hoc snapshots untouched
- [x] Smoke test: tmp dir + 6 fake snapshots spanning old (2026-01-*) and recent (2026-04-30..2026-05-03), `--keep-days 2`. Outcome: 4 pruned (everything older than today − 2d), 2 retained (`2026-05-02`, `2026-05-03`), plus today's = 3 files total.
- [x] Bash linting clean: `bash -n scripts/cut-readiness-daily.sh`

## Verification
bash -n /opt/termlink/scripts/cut-readiness-daily.sh
bash /opt/termlink/scripts/cut-readiness-daily.sh --help 2>&1 | grep -q -- '--keep-days'
bash /opt/termlink/scripts/cut-readiness-daily.sh --help 2>&1 | grep -q -- '--trend'

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

### 2026-05-04T07:05:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1469-cut-readiness-dailysh-emit---trend----ke.md
- **Context:** Initial task creation

### 2026-05-04T07:07:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
