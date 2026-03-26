# T-292: Audit Health Decay — Root Cause Analysis

**Status:** In progress
**Date:** 2026-03-26

## Problem Statement

Over ~2 weeks of active development, audit warnings accumulated from 0 to 50+ without anyone noticing or acting. The pre-push audit existed and ran, but warnings were tolerated silently. Only when a FAIL (CTL-009) blocked the push did anyone look at the audit output. By then, 50 episodic summaries were missing, 63 files had no fabric cards, 5 inception tasks had no research artifacts, and 10 completed tasks had placeholder ACs.

This is a systemic failure — the framework has audit tooling, but it failed to prevent decay.

## Evidence: What Accumulated

| Category | Count | Time to accumulate |
|----------|-------|--------------------|
| Missing episodic summaries | 50 | ~10 days |
| Missing fabric cards | 63 | ~10 days |
| Missing research artifacts (completed inceptions) | 5 | ~5 days |
| Placeholder ACs in completed tasks | 10 | ~10 days |
| CTL-009 FAIL (inception without decision) | 1 | ~2 days |
| Stale gaps.yaml | 1 | unknown |

## Root Cause Analysis

### Hypothesis 1: Warnings are invisible during normal workflow

The audit runs only:
1. **On `git push`** (pre-push hook) — but push is rare (sometimes days between pushes)
2. **On manual `fw audit`** — but nobody runs this voluntarily
3. **Via cron** — but CTL-020 shows cron isn't configured on macOS

**Between pushes, warnings accumulate with zero visibility.** The agent creates tasks, writes code, commits, but never sees the audit output. The only feedback loop is the pre-push hook — which only fires at the very end.

### Hypothesis 2: Warnings don't block anything

The pre-push hook exits 0 on warnings, only blocking on FAILs. This means:
- 50 missing episodics = still pushable
- 63 missing fabric cards = still pushable
- 10 placeholder ACs = still pushable

**If warnings don't block, they don't matter.** The framework teaches agents that only FAILs are consequential. This is the core design flaw.

### Hypothesis 3: Episodic generation fails silently on macOS

The `generate-episodic` script uses `date -d` (GNU) which fails on macOS bash. When task completion triggers episodic generation, it silently fails for some tasks (those with dates that hit the parsing bug). The agent never sees the error because it's in the completion pipeline.

**Silent failures in completion pipeline → accumulated gaps nobody knows about.**

### Hypothesis 4: No feedback loop between audit and session work

The `checkpoint.sh` (PostToolUse hook) tracks budget but NOT audit health. There's no hook that says "you just completed a task but the episodic generation failed" or "your last 3 commits didn't update the fabric."

**The session-level governance (budget, task gates) and the project-level governance (audit) are disconnected.**

### Hypothesis 5: Completion gate (P-010/P-011) doesn't check episodic or fabric

When `fw task update T-XXX --status work-completed` runs:
- It checks ACs are checked (P-010)
- It runs verification commands (P-011)
- It generates episodic summary
- It does NOT verify the episodic was actually created
- It does NOT check fabric registration for new files

**The completion gate is incomplete — it generates but doesn't verify.**

## Structural Gaps Identified

1. **G-021: Audit-push feedback gap** — Warnings accumulate invisibly between pushes
2. **G-022: Warning impotence** — Warnings don't block, so they're ignored
3. **G-023: Silent episodic failures** — macOS date bug causes silent generation failures
4. **G-024: Completion pipeline doesn't verify outputs** — Episodic/fabric generation not verified
5. **G-025: No continuous health monitoring** — Only push-time and manual auditing exist

## Potential Fixes

### Fix A: Commit-time mini-audit (lightest touch)
Add a PostToolUse or post-commit check that runs 3 fast checks:
- Did the last completed task get an episodic? (1 file stat)
- Are there new .rs files without fabric cards? (1 glob comparison)
- Any unchecked agent ACs in completed tasks? (1 grep)

Cost: ~100ms per commit. Catches problems immediately.

### Fix B: Promote critical warnings to FAILs
Some "warnings" should be FAILs:
- Missing episodic on completed task (should be FAIL after 1 hour)
- Placeholder ACs on completed task (should always be FAIL)
- Inception without decision after >2 commits (already FAIL — CTL-009)

This makes the pre-push hook catch real problems.

### Fix C: Completion gate verifies outputs
After `generate-episodic`, verify the file exists. If not, block completion. After creating new files, check fabric drift. If drift > threshold, warn.

### Fix D: Periodic health check (cron or session-based)
Run a lightweight audit subset every N commits or every 30 minutes. The PostToolUse hook already tracks tool count — piggyback on it.

## Dialogue Log

### 2026-03-26 — Human identifies the problem
Human: "quite a big amount of neglect that buildup... not proper discipline... why is this situation, it can evolve into this state... what we could do to enable a more continuous health care"

Key insight: The framework HAS the tooling (audit, episodic, fabric), but it's not applied continuously — only at push time, which is too late.
