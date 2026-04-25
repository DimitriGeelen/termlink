---
id: T-1284
name: "G-011 structural fix — fw doctor cache freshness check + own-hub live-read enforcement"
description: >
  Close G-011 (auth cache drift) by implementing the medium-term and long-term mitigations: (1) fw doctor compares ~/.termlink/secrets/<IP>.hex mtime/value against authoritative <runtime_dir>/hub.secret for self-hub profiles and warns on drift, (2) profiles using IP-keyed cache for self-hub read are deprecated with a migration hint to point secret_file directly at <runtime_dir>/hub.secret. Foundation for T-243 multi-turn agent conversation work — flaky auth blocks reliable multi-turn.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [auth, reliability, G-011]
components: []
related_tasks: []
created: 2026-04-25T22:31:17Z
last_update: 2026-04-25T22:31:17Z
date_finished: null
---

# T-1284: G-011 structural fix — fw doctor cache freshness check + own-hub live-read enforcement

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-25T22:31:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1284-g-011-structural-fix--fw-doctor-cache-fr.md
- **Context:** Initial task creation
