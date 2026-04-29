---
id: T-1412
name: "Update T-1166 migration doc with one-flag-flip cut procedure + reversibility (T-1411 follow-up)"
description: >
  Update T-1166 migration doc with one-flag-flip cut procedure + reversibility (T-1411 follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T22:19:38Z
last_update: 2026-04-29T22:19:38Z
date_finished: null
---

# T-1412: Update T-1166 migration doc with one-flag-flip cut procedure + reversibility (T-1411 follow-up)

## Context

T-1411 made the cut a one-character flip of `LEGACY_PRIMITIVES_ENABLED` in `crates/termlink-hub/src/router.rs`. The migration doc still describes a multi-step destructive procedure (router method removal → protocol bump → CLI rewriting → capability flip) and says "There is no roll-back after T-1166 lands". Both are now wrong. Replace with the structurally accurate procedure: one-line edit + recompile + restart, and document that the flag-off state is fully reversible until the source-cleanup follow-up task lands.

## Acceptance Criteria

### Agent
- [x] `docs/migrations/T-1166-retire-legacy-primitives.md` has a new `## Operator Cut Procedure` section between Timeline and Diagnostic that names the file path, the line, and the recompile/restart commands
- [x] Roll-Back section updated: states the flag-flip is reversible (flip back, rebuild, restart) until the source-cleanup follow-up lands
- [x] References list adds T-1406, T-1407, T-1408, T-1409, T-1410, T-1411
- [x] Markdown still renders cleanly (no broken anchors / unclosed code fences) — 20 code fences (even)

## Verification

grep -q "## Operator Cut Procedure" docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "LEGACY_PRIMITIVES_ENABLED" docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "T-1411" docs/migrations/T-1166-retire-legacy-primitives.md

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

### 2026-04-29T22:19:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1412-update-t-1166-migration-doc-with-one-fla.md
- **Context:** Initial task creation
