---
id: T-161
name: "Critical review and draft README + setup instructions"
description: >
  Send 5 review agents to critically assess existing docs, then draft a
  comprehensive README.md with install, usage, architecture, and examples.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [docs, readme]
components: []
related_tasks: []
created: 2026-03-17T22:40:47Z
last_update: 2026-03-17T22:40:47Z
date_finished: null
---

# T-161: Critical review and draft README + setup instructions

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] 5 review agents dispatched and findings collected
- [x] README.md written with install, usage, architecture, examples
- [x] Setup instructions cover both macOS and Linux

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

test -f README.md
grep -q "Quick Start" README.md
grep -q "cargo install" README.md
grep -q "macOS" README.md
grep -q "Linux" README.md

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

### 2026-03-17T22:40:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-161-critical-review-and-draft-readme--setup-.md
- **Context:** Initial task creation
