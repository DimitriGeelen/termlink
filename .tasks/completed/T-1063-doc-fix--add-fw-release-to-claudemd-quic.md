---
id: T-1063
name: "Doc fix — add fw release to CLAUDE.md Quick Reference with Tier-0 warning"
description: >
  Doc fix — add fw release to CLAUDE.md Quick Reference with Tier-0 warning

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T15:29:07Z
last_update: 2026-04-15T17:00:29Z
date_finished: 2026-04-15T17:00:29Z
---

# T-1063: Doc fix — add fw release to CLAUDE.md Quick Reference with Tier-0 warning

## Context

fw doctor flagged: "Doc drift: 1 fw subcommand(s) missing from CLAUDE.md Quick Reference — Missing: release". Prompted by 2026-04-15 incident where agent ran `fw release` thinking it was help, auto-tagged v0.9.1, and triggered a real GH Actions release. Row must make the Tier-0 consequence explicit.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Project CLAUDE.md Quick Reference table includes an `fw release` row
- [x] Row text warns that this is Tier-0 equivalent (creates + pushes release tag + triggers GH Actions)
- [x] Discovered scope limit: `fw doctor` Doc-drift check reads the FRAMEWORK-level CLAUDE.md at `/root/.agentic-framework/CLAUDE.md`, not the project's. Project edit is correct for project users; framework-level warning remains and needs upstream fix.

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

grep -q "^| \*\*Release a new version\*\*" CLAUDE.md
grep -q "Tier 0" CLAUDE.md

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

### 2026-04-15T15:29:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1063-doc-fix--add-fw-release-to-claudemd-quic.md
- **Context:** Initial task creation

### 2026-04-15T17:00:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
