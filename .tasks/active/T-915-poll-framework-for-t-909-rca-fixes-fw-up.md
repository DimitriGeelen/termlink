---
id: T-915
name: "Poll framework for T-909 RCA fixes; fw upgrade when landed"
description: >
  Wait-and-poll task for 4 framework bugs surfaced during T-909 (symlink fix, 2026-04-11). When upstream fixes land, run fw upgrade from termlink and re-verify. If no fixes available, leave on horizon: later and recheck periodically.

FINDINGS (all framework-side; do not patch from termlink):

F1 [HIGH] — fw inception decide --force bypasses build readiness gate (G-020) and AC verification (P-010). T-909 was completed with all 3 Agent ACs unchecked and an empty Recommendation section. Episodic generated 1s after the bypass-completion. Framework should refuse to close any inception task with unchecked Agent ACs OR empty Recommendation, even with --force; --force should require explicit per-AC override flags.

F2 [MEDIUM] — Framework's task-review prompt prints the wrong runnable command path: it shows the in-repo bin/fw path, but consumer projects (like termlink) reach fw via .agentic-framework/bin/fw. T-609 'copy-pasteable commands' learning never propagated into the framework's own UI/output messages. Reproduced live during T-909.

F3 [MEDIUM] — Episodic for T-909 (.context/episodic/T-909.yaml) was generated immediately after fw inception decide --force, BEFORE the actual fix work commits. It captures only 2 evidence/research commits, missing the actual vendoring fix, the 5 follow-up tasks (T-910..T-914), the 3 risk subreports, and the enforcement baseline. Episodic generation should be deferred until task is genuinely closed.

F4 [LOW] — fw vendor is undocumented in CLAUDE.md (not in Quick Reference, not in Component Fabric, not anywhere). Manual workaround: add fw vendor line to local CLAUDE.md.

CHECK PROCEDURE: grep framework git log since 2026-04-11 for keywords (inception decide, --force, build readiness, G-020, episodic, fw vendor). If matches found, run fw upgrade and re-verify. If no matches, update last_update and leave horizon=later.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [framework, upgrade, rca, polling]
components: []
related_tasks: [T-909, T-910, T-911, T-912, T-913, T-914]
created: 2026-04-11T12:47:25Z
last_update: 2026-04-11T14:55:00Z
date_finished: null
---

# T-915: Poll framework for T-909 RCA fixes; fw upgrade when landed

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-11T12:47:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-915-poll-framework-for-t-909-rca-fixes-fw-up.md
- **Context:** Initial task creation

### 2026-04-11T14:55:00Z — poll-result [agent]
- **Action:** Ran the check procedure against `/opt/999-Agentic-Engineering-Framework`
- **Command used:** `git -C /opt/999-Agentic-Engineering-Framework log --all --oneline --since=2026-04-11`
- **Scanned for keywords:** inception decide --force, G-020, G-031..G-034, build readiness, episodic defer, fw vendor, T-909
- **Framework activity since 2026-04-11:** 10+ commits in the T-1100-series. Notable:
  - `39f9b2a7 T-1101: Register G-032..G-034 + widen G-031 from /opt/termlink T-909 RCA` — gaps *registered* (so the framework has seen our findings) but not fixed.
  - `eb07962e T-1105: Structural-fix discipline pass — chokepoint+test upgrades for T-1100..T-1104`
  - `a692c60d T-1106: Structural upgrade — chokepoint+invariant tests for Watchtower port bleed`
  - T-1107, T-1108: unrelated defense-in-depth / Watchtower rendering
- **Inspected code paths:** `git log --since=2026-04-09 -- bin/fw agents/inception lib/inception` → only one unrelated commit (`4fb978f4 T-1081: Fix fw gaps — honor T-397 rename to concerns.yaml`)
- **Result:** No fixes landed for F1 (decide --force bypass), F2 (wrong runnable path), F3 (premature episodic), or F4 (fw vendor docs). Framework has acknowledged the gaps (T-1101) but no remediation commits yet.
- **Action:** Leaving horizon=later. Will recheck next session.
