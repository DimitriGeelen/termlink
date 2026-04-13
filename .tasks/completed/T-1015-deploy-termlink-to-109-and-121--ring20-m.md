---
id: T-1015
name: "Deploy termlink to .109 and .121 — ring20-management and ring20-dashboard"
description: >
  Deploy termlink to .109 and .121 — ring20-management and ring20-dashboard

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T10:29:29Z
last_update: 2026-04-13T12:00:38Z
date_finished: 2026-04-13T12:00:38Z
---

# T-1015: Deploy termlink to .109 and .121 — ring20-management and ring20-dashboard

## Context

Deploy termlink v0.9.795 (musl static, 61 MCP tools) to two remote hosts via termlink send-file + remote exec.

- .109 = ring20-management (container claude-dev, PVE VM 200)
- .121 = ring20-dashboard (dashboard agent)

## Acceptance Criteria

### Agent
- [x] termlink v0.9.795 (musl static) built and deployed to .109 via termlink send-file + remote exec
- [x] ring20-management hub profile configured in ~/.termlink/hubs.toml
- [x] termlink remote ping ring20-management returns PONG
- [x] termlink v0.9.795 (musl static) built and deployed to .121 via termlink send-file + remote exec
- [x] ring20-dashboard hub profile configured in ~/.termlink/hubs.toml
- [x] termlink remote ping ring20-dashboard returns PONG

### Human
- [x] [RUBBER-STAMP] Verify connectivity to both hubs
  **Steps:**
  1. `cd /opt/termlink && termlink remote ping ring20-management`
  2. `cd /opt/termlink && termlink remote ping ring20-dashboard`
  **Expected:** Both return PONG
  **If not:** Check hub service on remote host

## Verification

termlink remote ping ring20-management 2>&1 | grep -q PONG
termlink remote ping ring20-dashboard 2>&1 | grep -q PONG
termlink remote exec ring20-management ring20-manager "termlink --version" 2>&1 | grep -q "0.9"
termlink remote exec ring20-dashboard ring20-dashboard "termlink --version" 2>&1 | grep -q "0.9"

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

### 2026-04-13T10:29:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1015-deploy-termlink-to-109-and-121--ring20-m.md
- **Context:** Initial task creation

### 2026-04-13T12:00:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
