---
id: T-1188
name: "T-1187 upstream mirror — apply pl007-scanner patch in framework repo"
description: >
  T-1187 built pl007-scanner.sh in the termlink-vendored copy at .agentic-framework/agents/context/pl007-scanner.sh. Since .agentic-framework is gitignored (vendored copy), the patch does not persist — next fw upgrade will overwrite. Mirror the patch into /opt/999-Agentic-Engineering-Framework/agents/context/pl007-scanner.sh and commit there. Use termlink dispatch to cross the project boundary per T-559 policy.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: [T-1187, T-1176]
created: 2026-04-22T11:18:03Z
last_update: 2026-04-22T11:18:03Z
date_finished: null
---

# T-1188: T-1187 upstream mirror — apply pl007-scanner patch in framework repo

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

### 2026-04-22T11:18:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1188-t-1187-upstream-mirror--apply-pl007-scan.md
- **Context:** Initial task creation
