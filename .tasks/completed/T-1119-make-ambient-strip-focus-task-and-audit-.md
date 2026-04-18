---
id: T-1119
name: "Make ambient strip focus task and audit status clickable"
description: >
  Make ambient strip focus task and audit status clickable

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:04:22Z
last_update: 2026-04-18T09:06:00Z
date_finished: 2026-04-18T09:06:00Z
---

# T-1119: Make ambient strip focus task and audit status clickable

## Context

The ambient strip currently shows focus task ID, audit status, and attention count as plain text. Making them clickable (focus task → /tasks/<id>, audit → /quality, attention → /tasks) reduces clicks for common operator workflows.

## Acceptance Criteria

### Agent
- [x] Focus task in ambient strip is a link to /tasks/<id> when a focus task exists
- [x] Audit status in ambient strip links to /quality
- [x] Attention count in ambient strip links to /tasks
- [x] Existing pages still render without errors

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

curl -sf http://localhost:3000/ > /dev/null
curl -sf http://localhost:3000/ | grep -q 'href="/quality"'
curl -sf http://localhost:3000/ | grep -q 'href="/tasks"'

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

### 2026-04-18T09:04:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1119-make-ambient-strip-focus-task-and-audit-.md
- **Context:** Initial task creation

### 2026-04-18T09:06:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
