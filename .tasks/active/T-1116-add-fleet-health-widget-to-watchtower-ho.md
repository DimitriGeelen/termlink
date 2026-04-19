---
id: T-1116
name: "Add fleet health widget to Watchtower home page"
description: >
  Show fleet health on home page

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T22:32:56Z
last_update: 2026-04-17T22:51:46Z
date_finished: 2026-04-17T22:51:46Z
---

# T-1116: Add fleet health widget to Watchtower home page

## Context

Home page is framework-focused. Add a fleet health widget for at-a-glance operational status.

## Acceptance Criteria

### Agent
- [x] Home page route passes fleet summary data to template
- [x] Widget shows hub count and up/down with color coding
- [x] Widget links to /fleet for detail
- [x] Home page renders without errors

### Human
- [ ] [RUBBER-STAMP] Fleet widget visible on home page
  **Steps:** Open http://localhost:3000/ and look for fleet status near the top
  **Expected:** Shows hub counts with link to fleet page
  **If not:** Check browser console


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, playwright, home-fleet-widget):** Opened `http://localhost:3000/` via playwright. Ambient strip at top of page shows the fleet health widget: `● 2/3 up · 1 auth-fail` with a color-coded status dot (amber for degraded), linked to `/fleet`. Widget is visible on the home page, loads asynchronously (matches T-1116 spec of 'async status with color-coded dot'). Viewport screenshot: home-page-2026-04-19.png. RUBBER-STAMPable.

## Verification

curl -sf http://localhost:3000/ | grep -q 'fleet'

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

### 2026-04-17T22:32:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1116-add-fleet-health-widget-to-watchtower-ho.md
- **Context:** Initial task creation

### 2026-04-17T22:51:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
