---
id: T-1015
name: "Deploy termlink to .109 and .121 — ring20-management and ring20-dashboard"
description: >
  Deploy termlink to .109 and .121 — ring20-management and ring20-dashboard

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T10:29:29Z
last_update: 2026-04-13T10:29:29Z
date_finished: null
---

# T-1015: Deploy termlink to .109 and .121 — ring20-management and ring20-dashboard

## Context

Deploy termlink v0.9.450 (61 MCP tools) to two remote hosts using scripts/deploy-remote.sh (T-1013). Blocked on SSH key authorization.

- .109 = ring20-management (container claude-dev, PVE VM 200)
- .121 = ring20-dashboard (dashboard agent)

## Acceptance Criteria

### Agent
- [ ] SSH key authorized on .109
- [ ] deploy-remote.sh succeeds on .109 (ring20-management profile updated)
- [ ] termlink remote ping ring20-management returns pong
- [ ] SSH key authorized on .121
- [ ] deploy-remote.sh succeeds on .121 (ring20-dashboard profile created)
- [ ] termlink remote ping ring20-dashboard returns pong

### Human
- [ ] [REVIEW] Authorize SSH key on both hosts before agent can deploy
  **Steps:**
  1. `cd /opt/termlink && ssh-copy-id -i ~/.ssh/id_ed25519.pub root@192.168.10.109`
  2. `cd /opt/termlink && ssh-copy-id -i ~/.ssh/id_ed25519.pub root@192.168.10.121`
  **Expected:** Both commands succeed (may prompt for password)
  **If not:** Check if root login is allowed on the target hosts

## Verification

termlink remote ping ring20-management 2>&1 | grep -q PONG
termlink remote ping ring20-dashboard 2>&1 | grep -q PONG

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
