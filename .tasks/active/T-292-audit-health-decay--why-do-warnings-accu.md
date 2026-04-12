---
id: T-292
name: "Audit health decay — why do warnings accumulate silently and what structural fix prevents it"
description: >
  Inception: Audit health decay — why do warnings accumulate silently and what structural fix prevents it

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-26T11:57:54Z
last_update: 2026-03-26T11:58:06Z
date_finished: null
---

# T-292: Audit health decay — why do warnings accumulate silently and what structural fix prevents it

## Problem Statement

Over ~2 weeks of development, audit warnings accumulated from 0 to 50+ without anyone noticing. The framework has audit tooling but it only runs at push time — too infrequent to prevent decay. 5 root causes identified: invisible warnings, warning impotence (don't block), silent episodic failures on macOS, disconnected session/project governance, and incomplete completion gate.

## Assumptions

- Fix A (completion gate verifies outputs) + Fix D (macOS date fix) are sufficient for >80% improvement
- Audit hook overhead can stay under 200ms per commit
- Changes are backward-compatible with existing update-task.sh API

## Exploration Plan

1. Validate all 5 root causes against framework source (DONE — confirmed on .107)
2. Prototype Fix A: verify episodic exists after generation (~10 lines)
3. Prototype Fix D: portable date handling in episodic.sh (~20 lines)
4. Evaluate Fix B (warning promotion) and Fix C (commit-time mini-audit) as follow-ups

## Technical Constraints

- macOS bash 3.2 lacks `date -d`, `declare -A`, and other GNU extensions
- Linux (.107) uses GNU date — fix must work on both platforms
- PostToolUse hooks must stay under 200ms to avoid session friction

## Scope Fence

**IN scope:** Framework-level fixes (update-task.sh, episodic.sh, audit.sh)
**OUT scope:** Consumer-project cleanup (already done in T-291)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (5 root causes confirmed against .107 framework source)
- [x] Assumptions tested (generate-episodic uses `|| true`, no verification, `date -d` on line 173)
- [x] Recommendation written with rationale (Fix A + Fix D, see research artifact)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Read the research artifact and recommendation in this task
  2. Evaluate go/no-go criteria against findings
  3. Run: `fw inception decide T-XXX go|no-go --rationale "your rationale"`
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- At least Fix A (completion gate verifies outputs) is feasible in framework
- Changes are backward-compatible with update-task.sh API
- Fix can be validated on both Linux and macOS

**NO-GO if:**
- Audit hook overhead exceeds 200ms per commit
- Fixing date bug requires abandoning bash (too large a scope change)

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
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

## Recommendation

**Recommendation:** GO

**Rationale:** All 5 root causes validated. Fix A (completion gate verifies episodic output exists) and Fix D (portable macOS date handling) cover >80% of decay. Bounded implementation.

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

### 2026-03-26 — research-artifact [agent]
- **Artifact:** `docs/reports/T-292-audit-health-decay.md`
- **Content:** Full RCA with 5 hypotheses, structural gaps, 4 proposed fixes

### 2026-03-26 — pickup-dispatched [agent]
- **Action:** Pushed pickup prompt to fw-agent and fw-master on .107
- **File:** `/tmp/termlink-inbox/T-292-pickup-prompt.md` (7463 bytes)
- **Content:** Complete inception with evidence, 4 fix options, go/no-go criteria
