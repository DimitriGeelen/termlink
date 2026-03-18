---
id: T-148
name: "TermLink integration spec for framework pickup"
description: >
  Write the TermLink integration specification as a framework pickup prompt — covers session registration, hub management, inject/attach, and agent dispatch via TermLink

status: work-completed
workflow_type: specification
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-15T23:27:18Z
last_update: 2026-03-17T16:25:43Z
date_finished: 2026-03-17T16:25:43Z
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
- [ ] [REVIEW] Paste prompt into framework session and verify it picks up correctly
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
