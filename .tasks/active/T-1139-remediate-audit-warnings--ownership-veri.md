---
id: T-1139
name: "Remediate audit warnings — ownership, verification gates, artifact references"
description: >
  Remediate audit warnings — ownership, verification gates, artifact references

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T12:11:00Z
last_update: 2026-04-19T12:11:00Z
date_finished: null
---

# T-1139: Remediate audit warnings — ownership, verification gates, artifact references

## Context

Audit 2026-04-19 flagged 19 warnings + 12 fails. Scope this task to the cheap remediable items — skip trend-analysis noise (uncommitted-changes trend is known and tracked upstream as P-038).

Targets:
- CTL-025 × 2: T-1021 + T-1022 work-completed in active/ but owner=agent (should be human)
- CTL-013 × 1: T-1130 verification re-run: 1 command failing
- C-001 × 1: T-1122 has artifact but task doesn't reference it
- C-001 × 1: T-1074 has no research artifact
- CTL-012 × 5: completed tasks T-1006/1113/1110/1059/1111 with unchecked ACs

Skipped (tracked elsewhere or not actionable):
- CTL-027 × 12 inception tasks missing ## Recommendation — scope of in-flight T-1112
- 14 episodics TODO — broad housekeeping
- Bugfix-learning 32% — broad housekeeping
- D2/D3/D5 trend warnings — reporting only
- CTL-003 budget-status stale — transient

## Acceptance Criteria

### Agent
- [ ] T-1021 + T-1022 ownership flipped to human via `fw task update --owner human`
- [ ] T-1130 failing verification command diagnosed + either fixed or documented why spurious
- [ ] T-1122 Updates section references `docs/reports/T-1122-*.md`
- [ ] T-1074 research artifact stub created at `docs/reports/T-1074-*.md`
- [ ] CTL-012 completed tasks investigated — either ACs legitimately checked or tasks moved back to active/
- [ ] `fw audit` re-run after changes shows reduced warn/fail count on the targeted items

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-19T12:11:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1139-remediate-audit-warnings--ownership-veri.md
- **Context:** Initial task creation
