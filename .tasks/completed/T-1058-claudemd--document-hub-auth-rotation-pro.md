---
id: T-1058
name: "CLAUDE.md — document hub auth rotation protocol and heal paths"
description: >
  CLAUDE.md — document hub auth rotation protocol and heal paths

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T20:40:59Z
last_update: 2026-04-15T13:36:17Z
date_finished: 2026-04-15T13:36:17Z
---

# T-1058: CLAUDE.md — document hub auth rotation protocol and heal paths

## Context

Final build task from T-1051 inception (Option D). Adds a "Hub Auth Rotation
Protocol" section to CLAUDE.md's currently-empty Project-Specific Rules block.

Future agents hit the rotation failure class (invalid signature / stale cached
secret) need to find, in-tree, without grepping code:
- how the failure manifests and how to recognize it
- the heal paths: `termlink fleet reauth <profile>` (Tier-1 print) and
  `termlink fleet reauth <profile> --bootstrap-from <SOURCE>` (Tier-2 autoheal)
- the meaning of `hub_fingerprint=` in auto-registered learnings (R1 drift
  detection)
- the R2 out-of-band anchor requirement
- links into the inception artifact (`docs/reports/T-1051-termlink-auth-reliability-inception.md`)

## Acceptance Criteria

### Agent
- [x] `CLAUDE.md` under `## Project-Specific Rules` gains a "### Hub Auth Rotation Protocol" subsection
- [x] Section covers: symptom recognition, Tier-1 heal command, Tier-2 `--bootstrap-from`, R1 fingerprint-drift detection, R2 out-of-band anchor rule, auto-registered learnings/concerns (T-1052/T-1053 behavior)
- [x] Section links to the inception artifact (`docs/reports/T-1051-termlink-auth-reliability-inception.md`) and the 7 related task IDs (T-1051–T-1057, plus this T-1058)
- [x] CLAUDE.md still parses cleanly as markdown (verified: preceding `## CI / Release Flow` and trailing `## Core Principle` sections intact)
- [x] No behavioral code changes — this is docs only

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

grep -q "### Hub Auth Rotation Protocol" CLAUDE.md
grep -q "fleet reauth.*--bootstrap-from" CLAUDE.md
grep -q "T-1051" CLAUDE.md

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

### 2026-04-14T20:40:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1058-claudemd--document-hub-auth-rotation-pro.md
- **Context:** Initial task creation

### 2026-04-15T13:36:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
