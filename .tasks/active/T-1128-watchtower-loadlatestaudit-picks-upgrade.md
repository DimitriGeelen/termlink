---
id: T-1128
name: "Watchtower load_latest_audit picks upgrades.yaml — needs date-prefix filter"
description: >
  Watchtower load_latest_audit picks upgrades.yaml — needs date-prefix filter

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T18:39:00Z
last_update: 2026-04-18T18:40:30Z
date_finished: 2026-04-18T18:40:30Z
---

# T-1128: Watchtower load_latest_audit picks upgrades.yaml — needs date-prefix filter

## Context

`load_latest_audit()` does `sorted(audit_dir.glob("*.yaml"), reverse=True)` and returns the first match. `.context/audits/` contains date-named audit reports (`2026-04-18.yaml`) AND `upgrades.yaml`. Reverse-alphabetical sort puts `upgrades.yaml` first; it has no `summary` key, so the loader returns empty data and the ambient strip shows `Audit: unknown` even when fresh audits exist.

## Acceptance Criteria

### Agent
- [x] `load_latest_audit()` filters glob to date-prefixed files only (`[0-9][0-9][0-9][0-9]-*.yaml`)
- [x] After fix, ambient strip on `/` shows `Audit: WARN` (verified)
- [x] No regression: load_latest_audit still returns `(None, {}, [])` when no audit files exist (early-return preserved)

### Human
- [ ] [REVIEW] Verify ambient strip shows real audit status
  **Steps:**
  1. Reload http://localhost:3000/
  2. Look at ambient strip's `Audit:` field
  **Expected:** Shows `WARN` (matches today's `fw audit` summary)
  **If not:** Check `load_latest_audit()` is returning today's date file


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, playwright, audit-loader-date-filter):** Ambient strip shows `Audit: PASS` (green) on both `/` and `/fleet` — sourced from `.context/audits/2026-04-19.yaml` (today's audit report) rather than the sibling `upgrades.yaml` path that previously leaked through. Date-prefix filter is working; ambient strip reflects real audit status. REVIEW-approvable.

## Verification

# Shell commands that MUST pass before work-completed.
python3 -c "import ast; ast.parse(open('.agentic-framework/web/shared.py').read())"
PROJECT_ROOT=/opt/termlink PYTHONPATH=.agentic-framework python3 -c "from web.shared import load_latest_audit; ts,s,_=load_latest_audit(); assert s.get('warn',0)>=0 and 'pass' in s, f'expected real summary, got {s}'"

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

### 2026-04-18T18:39:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1128-watchtower-loadlatestaudit-picks-upgrade.md
- **Context:** Initial task creation

### 2026-04-18T18:40:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-18T19:46Z — evidence [agent]
- **Action:** Curled http://localhost:3000/ ambient strip.
- **Result:** Shows `Audit: WARN` (matches today's `.context/audits/2026-04-18.yaml` summary with `warn: 1`), not `Audit: unknown`. The date-prefix glob `[0-9][0-9][0-9][0-9]-*.yaml` correctly skipped `upgrades.yaml`.
- **Suggest:** Human can check the REVIEW box.
