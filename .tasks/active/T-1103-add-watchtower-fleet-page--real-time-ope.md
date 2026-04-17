---
id: T-1103
name: "Add Watchtower /fleet page — real-time operational dashboard with clickable hubs and sessions"
description: >
  Add Watchtower /fleet page — real-time operational dashboard with clickable hubs and sessions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T09:05:57Z
last_update: 2026-04-17T09:05:57Z
date_finished: null
---

# T-1103: Add Watchtower /fleet page — real-time operational dashboard with clickable hubs and sessions

## Context

T-1101 R2: Add an operations-focused Watchtower page at `/fleet` that shows real-time
fleet health, clickable hub profiles and session names, and actionable fix steps.
The page calls `termlink fleet status --json` and renders the results.

## Acceptance Criteria

### Agent
- [x] Blueprint `fleet.py` registered and route `/fleet` returns 200
- [x] Page shows each hub with status badge (UP green, DOWN red, AUTH yellow)
- [x] Page shows session count and latency for UP hubs
- [x] Page shows ACTIONS NEEDED section for broken hubs
- [x] Nav bar includes Fleet link under Architecture group
- [x] Page auto-refreshes every 30 seconds (JS interval)
- [x] JSON API at `/api/fleet/status` for programmatic access

### Human
- [ ] [REVIEW] Open `/fleet` page and verify it's useful for daily operations check
  **Steps:** Open `http://192.168.10.107:3002/fleet` in browser
  **Expected:** Fleet overview with status badges, session counts, actions
  **If not:** Check Watchtower log for errors

## Verification

bash -c 'curl -sf http://localhost:3002/fleet | grep -q "fleet"'

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

### 2026-04-17T09:05:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1103-add-watchtower-fleet-page--real-time-ope.md
- **Context:** Initial task creation
