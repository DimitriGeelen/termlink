---
id: T-148
name: "TermLink integration spec for framework pickup"
description: >
  Write the TermLink integration specification as a framework pickup prompt — covers
  session registration, hub management, inject/attach, and agent dispatch via TermLink

status: work-completed
workflow_type: specification
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-15T23:27:18Z
last_update: '2026-08-18T18:58:50Z'
date_finished: 2026-03-24T08:03:15Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:50Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 4
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=4 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-148: TermLink integration spec for framework pickup

## Context

Spec document for the framework team to pick up TermLink integration work.
Based on T-142 inception research. See docs/specs/T-148-termlink-framework-integration.md.

## Acceptance Criteria

### Agent
- [x] Spec document written with Phase 0 details, primitives table, phased rollout
- [x] Console-ready prompt for framework session included
- [x] Terminal cleanup lesson (T-074) documented

### Human
- [x] [REVIEW] Paste prompt into framework session and verify it picks up correctly
  **Steps:**
  1. Open a Claude Code session in the framework project
  2. Paste the prompt from `docs/specs/T-148-termlink-framework-integration.md`
  3. Verify the framework agent creates an inception or build task
  **Expected:** Framework agent scopes the work and starts Phase 0
  **If not:** Adjust the prompt or spec and retry

## Verification

test -f docs/specs/T-148-termlink-framework-integration.md
grep -q "Phase 0" docs/specs/T-148-termlink-framework-integration.md
grep -q "3-phase cleanup" docs/specs/T-148-termlink-framework-integration.md

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

### 2026-03-15T23:27:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-148-termlink-integration-spec-for-framework-.md
- **Context:** Initial task creation

### 2026-03-17T16:25:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-24T08:03:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
