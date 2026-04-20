---
id: T-1171
name: "Add fw doctor check for local hub secret cache drift (G-011)"
description: >
  Add fw doctor check: compare mtime of each ~/.termlink/secrets/*.hex against the corresponding hub's authoritative secret file (where locally resolvable). Warn if cache is older. Also audit chmod 600 on all .hex files (proxmox4.hex currently 644 — security smell). Deliverable: one new check in agents/doctor/, test coverage, CLAUDE.md §Hub Auth Rotation Protocol updated with the 'read-live, not cache' rule.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-04-20T20:08:57Z
last_update: 2026-04-20T20:08:57Z
date_finished: null
---

# T-1171: Add fw doctor check for local hub secret cache drift (G-011)

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

### 2026-04-20T20:08:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1171-add-fw-doctor-check-for-local-hub-secret.md
- **Context:** Initial task creation
