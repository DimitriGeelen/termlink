---
id: T-1268
name: "Promote L-012 + L-004 to practice (8 + 4 apps)"
description: >
  Promote L-012 + L-004 to practice (8 + 4 apps)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T20:03:12Z
last_update: 2026-04-25T20:03:54Z
date_finished: 2026-04-25T20:03:54Z
---

# T-1268: Promote L-012 + L-004 to practice (8 + 4 apps)

## Context

Two more graduated learnings ready for promotion (3+ apps threshold satisfied):
- **L-012 (8 apps, T-020 origin):** Framework hook paths must reference final installation directory (symlinks), not version-specific paths — D2 Reliability (hooks survive tool upgrades).
- **L-004 (4 apps, T-064 origin):** Shell commands accepting user input must escape arguments before passing to `sh -c` — single-quote wrap with internal quote escaping. D2 Reliability + security (injection prevention).

Both are operational rules that have stabilized through repeated application. Encoding as practices makes them queryable + auditable.

## Acceptance Criteria

### Agent
- [x] `fw promote L-012 ...` exit 0 → PP-003 (apps: 9, origin T-020).
- [x] `fw promote L-004 ...` exit 0 → PP-004 (apps: 5, origin T-064).
- [x] Both new practices have `promoted_from` lineage; YAML parses cleanly.

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

python3 -c "import yaml; d=yaml.safe_load(open('.context/project/practices.yaml')); ids=[p.get('promoted_from') for p in d.get('practices',[])]; assert 'L-012' in ids and 'L-004' in ids, f'missing: {ids}'"

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

### 2026-04-25T20:03:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1268-promote-l-012--l-004-to-practice-8--4-ap.md
- **Context:** Initial task creation

### 2026-04-25T20:03:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
