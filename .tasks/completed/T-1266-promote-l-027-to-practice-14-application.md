---
id: T-1266
name: "Promote L-027 to practice (14 applications, PROJECT_ROOT explicit-passing)"
description: >
  Promote L-027 to practice (14 applications, PROJECT_ROOT explicit-passing)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:00:25Z
last_update: 2026-04-25T20:01:19Z
date_finished: 2026-04-25T20:01:19Z
---

# T-1266: Promote L-027 to practice (14 applications, PROJECT_ROOT explicit-passing)

## Context

L-027 originated in T-290 and has accumulated 14 applications across 8 different tasks (PL-007/PL-022/etc reference it). Pattern: framework scripts invoked from consumer projects without explicit `PROJECT_ROOT` env fall back to `git rev-parse --show-toplevel` which silently resolves to the framework repo (when `.agentic-framework` is a symlink) or to the consumer project — non-deterministic. Consumers like termlink reliably hit this. At 14 applications, this graduates from "learning" to "practice" — encode as a structural rule. Maps to D2 Reliability (predictable, no silent fallback).

## Acceptance Criteria

### Agent
- [x] `fw promote L-027 --name "Consumer hooks pass PROJECT_ROOT explicitly" --directive D2` exit 0 → created PP-001 in `.context/project/practices.yaml`.
- [x] PP-001 has `promoted_from: L-027`, `directive: D2`, `applications: 15`, `status: active`. (Note: framework convention is to record promotion lineage on the practice side via `promoted_from`, not flip a flag on the learning side.)
- [x] Practice entry references L-027 origin (`promoted_from: L-027`) and stamps `applications: 15`.

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

python3 -c "import yaml; d=yaml.safe_load(open('.context/project/practices.yaml')); assert any('L-027' in str(p) for p in (d.get('practices') or d.get('rules') or [])), 'no practice references L-027'"
test -f .context/project/practices.yaml

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

### 2026-04-25T20:00:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1266-promote-l-027-to-practice-14-application.md
- **Context:** Initial task creation

### 2026-04-25T20:01:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
