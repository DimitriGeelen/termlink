---
id: T-1120
name: "Make project root in ambient strip link to /project"
description: >
  Make project root in ambient strip link to /project

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:20:19Z
last_update: 2026-04-18T09:21:06Z
date_finished: 2026-04-18T09:21:06Z
---

# T-1120: Make project root in ambient strip link to /project

## Context

The project root display in the ambient strip is plain text. Linking it to /project (project docs page) makes it a discoverable shortcut to project-specific documentation.

## Acceptance Criteria

### Agent
- [x] Project root in ambient strip is wrapped in an anchor pointing to /project
- [x] Existing pages still render

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

curl -sf http://localhost:3000/ > /dev/null
curl -sf http://localhost:3000/ | grep -q 'href="/project"'

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

### 2026-04-18T09:20:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1120-make-project-root-in-ambient-strip-link-.md
- **Context:** Initial task creation

### 2026-04-18T09:21:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
