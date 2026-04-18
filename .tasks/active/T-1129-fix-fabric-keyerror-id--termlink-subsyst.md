---
id: T-1129
name: "Fix /fabric KeyError 'id' — termlink subsystems.yaml uses name not id"
description: >
  Fix /fabric KeyError 'id' — termlink subsystems.yaml uses name not id

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T20:07:18Z
last_update: 2026-04-18T20:07:18Z
date_finished: null
---

# T-1129: Fix /fabric KeyError 'id' — termlink subsystems.yaml uses name not id

## Context

`/fabric` page returns HTTP 500 with `KeyError: 'id'` at `.agentic-framework/web/blueprints/fabric.py:93` (`registered_ids = {s["id"] for s in subsystems}`). Termlink's `.fabric/subsystems.yaml` keys subsystems by `name:` not `id:`. Framework's own `subsystems.yaml` uses `id:`. Fix locally by adding `id:` keys; send upstream pickup so the loader is robust to either schema.

## Acceptance Criteria

### Agent
- [x] Confirmed root cause via watchtower.log traceback (KeyError: 'id' at fabric.py:93)
- [ ] Add `id:` field to each entry in `.fabric/subsystems.yaml` (use existing `name` value as id)
- [ ] `curl -sf http://localhost:3000/fabric` returns HTTP 200
- [ ] No regression: `fw fabric overview` and `fw fabric drift` still work
- [ ] Send upstream pickup envelope to framework — `fabric.py:93` should fall back to `s.get("id") or s.get("name")` for forward-compat with consumer projects using `name:` schema

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
python3 -c "import yaml; d=yaml.safe_load(open('.fabric/subsystems.yaml')); assert all('id' in s for s in d['subsystems']), 'every subsystem must have id'"
curl -sf http://localhost:3000/fabric > /dev/null

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

### 2026-04-18T20:07:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1129-fix-fabric-keyerror-id--termlink-subsyst.md
- **Context:** Initial task creation
