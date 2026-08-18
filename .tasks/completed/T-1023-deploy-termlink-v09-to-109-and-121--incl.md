---
id: T-1023
name: "Deploy termlink v0.9+ to .109 and .121 — includes remote doctor and file transfer
  fixes"
description: >
  Build musl static binary and deploy to both remote hubs via termlink send-file +
  remote exec. Restart hub services so they run the new code (fixing the stale hub
  binary issue).

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:29:47Z
last_update: '2026-08-18T18:58:42Z'
date_finished: 2026-04-13T12:56:33Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:42Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-1023: Deploy termlink v0.9+ to .109 and .121 — includes remote doctor and file transfer fixes

## Context

Deploy latest termlink with remote doctor, fleet doctor, file transfer fixes to .109/.121 via termlink send-file + remote exec. Restart hub processes.

## Acceptance Criteria

### Agent
- [x] Musl static binary built (v0.9.809, 62 MCP tools, includes hub restart + fleet doctor)
- [x] Binary v0.9.807 deployed+running on .109 (hub restarted by .109 agent)
- [x] termlink remote ping ring20-management returns PONG
- [x] Binary v0.9.809 deployed+running on .121 (hub restarted by .121 agent with TCP)
- [x] termlink remote ping ring20-dashboard returns PONG
- [x] TOFU entries updated for both hubs

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

### 2026-04-13T12:29:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1023-deploy-termlink-v09-to-109-and-121--incl.md
- **Context:** Initial task creation

### 2026-04-13T12:44:56Z — status-update [task-update-agent]
- **Change:** status: started-work → issues
- **Reason:** Hub restart via termlink kills connectivity — chicken-and-egg problem. Binary installed on both hosts but hubs run old code. .109 hub is currently DOWN. Need supervisor or external restart mechanism.

### 2026-04-13T12:56:33Z — status-update [task-update-agent]
- **Change:** status: issues → work-completed
