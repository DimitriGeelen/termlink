---
id: T-1067
name: "Migrate ring20-management profile .126 → .122 (second renumber)"
description: >
  Migrate ring20-management profile .126 → .122 (second renumber)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T18:59:49Z
last_update: 2026-04-15T18:59:49Z
date_finished: null
---

# T-1067: Migrate ring20-management profile .126 → .122 (second renumber)

## Context

User report 2026-04-15 ~19:00Z: ".109 now is 122". Second renumber in one session (.109 → .126 → .122 — T-1065 handled the first). Verified: ping .122 OK (113ms, elevated latency hints at routing change), .126 no longer responds, port 9100 still refused on .122. Hub process revival remains T-1064.

Observation: back-to-back renumbers suggest ring20-management container is being actively rescheduled (PVE maintenance or instability). May correlate with OneDev 502 + .121 hub down reported in T-1064.

## Acceptance Criteria

### Agent
- [x] `~/.termlink/hubs.toml` ring20-management address `192.168.10.126:9100` → `192.168.10.122:9100`
- [x] Memory file `reference_ring20_infrastructure.md` updated with new IP + rename history
- [x] T-1064 updated with the second renumber fact
- [x] `termlink fleet doctor` attempts .122 for ring20-management

## Verification

grep -q '192.168.10.122:9100' /root/.termlink/hubs.toml
! grep -q '192.168.10.126' /root/.termlink/hubs.toml

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

### 2026-04-15T18:59:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1067-migrate-ring20-management-profile-126--1.md
- **Context:** Initial task creation
