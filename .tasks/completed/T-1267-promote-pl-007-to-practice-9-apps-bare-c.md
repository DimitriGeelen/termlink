---
id: T-1267
name: "Promote PL-007 to practice (9 apps, bare-command output forbidden)"
description: >
  Promote PL-007 to practice (9 apps, bare-command output forbidden)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:01:26Z
last_update: 2026-04-25T20:03:04Z
date_finished: 2026-04-25T20:03:04Z
---

# T-1267: Promote PL-007 to practice (9 apps, bare-command output forbidden)

## Context

Original target was PL-007 (bare-command output) but `fw promote` only accepts L-* prefix (project-prefixed PL-* learnings remain in project corpus). Redirected to **L-007** which has 11 applications: pattern about Rust 2024 edition requiring `unsafe` around `std::env::set_var` due to data-race concerns. Affects all test code that mutates env vars; consistent fix requires SAFETY comment documenting single-threaded access. Maps to D2 Reliability (compile-time safety contract — silent removal would let UB through). PL-007 follow-up is captured separately if needed.

## Acceptance Criteria

### Agent
- [x] `fw promote L-007 ...` exit 0 → created PP-002 in `.context/project/practices.yaml`.
- [x] PP-002 has `promoted_from: L-007`, `directive: D2`, `applications: 11`, origin T-172.
- [x] YAML parses cleanly.

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

python3 -c "import yaml; d=yaml.safe_load(open('.context/project/practices.yaml')); assert any('L-007' in str(p) for p in d.get('practices',[])), 'no practice references L-007'"

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

### 2026-04-25T20:01:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1267-promote-pl-007-to-practice-9-apps-bare-c.md
- **Context:** Initial task creation

### 2026-04-25T20:03:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
