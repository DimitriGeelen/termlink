---
id: T-1246
name: "T-1246 G-046 fix: Watchtower decide form falls back to docs/reports/T-XXX-inception.md when task body lacks Recommendation section"
description: >
  T-1246 G-046 fix: Watchtower decide form falls back to docs/reports/T-XXX-inception.md when task body lacks Recommendation section

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T10:48:31Z
last_update: 2026-04-25T10:48:31Z
date_finished: null
---

# T-1246: T-1246 G-046 fix: Watchtower decide form falls back to docs/reports/T-XXX-inception.md when task body lacks Recommendation section

## Context

G-046 fix: Watchtower's `/inception/<id>` page reads the rationale prefill
from the task file's `## Recommendation` section. When the agent writes a
thorough recommendation in `docs/reports/T-XXX-inception.md` (per CTL-027)
but only puts a stub in the task file, the prefill is empty.

This task adds a fallback: if the task body lacks a Recommendation section,
read `docs/reports/T-{task_id}-inception.md` and extract from there.

## Acceptance Criteria

### Agent
- [x] `inception_detail` route reads `docs/reports/T-{task_id}-inception.md` when `_extract_section(task_body, "Recommendation")` returns empty
- [x] Existing behavior preserved: when task body has Recommendation, that takes precedence (no surprise overrides)
- [x] Watchtower process restarts cleanly after edit
- [x] curl /inception/T-1229 returns 200 OK
- [x] Manual probe via playwright: rationale textarea contains content from docs/reports/ when task file lacks recommendation

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification
python3 -c "import ast; ast.parse(open('.agentic-framework/web/blueprints/inception.py').read())"
curl -sf http://localhost:3100/inception/T-1229 > /dev/null

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

### 2026-04-25T10:48:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1246-t-1246-g-046-fix-watchtower-decide-form-.md
- **Context:** Initial task creation
