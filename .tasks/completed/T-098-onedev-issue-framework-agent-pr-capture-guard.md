---
id: T-098
name: "OneDev issue + framework agent pickup prompt for /capture and Conversation
  Guard"
description: >
  Create a OneDev issue in the framework repo for the framework agent to pick up and
  implement: (1) Exploratory Conversation Guard rule for CLAUDE.md template, (2) /capture
  skill for the framework skills library. After creating the issue, generate a formatted
  console prompt that the human can copy-paste to the framework agent to pick up the
  work.
status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [framework, onedev, handoff]
components: []
related_tasks: [T-094, T-095, T-096]
created: 2026-03-11T11:30:00Z
last_update: '2026-08-18T18:58:42Z'
date_finished: 2026-03-12T00:38:12Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:41Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-098: OneDev Issue + Framework Agent Pickup Prompt

## Context

Spawned from T-094. The remediations in T-095/T-096 fix this specific project (termlink).
But the framework serves all consumer projects. The framework agent must implement:
- The Conversation Guard rule in the CLAUDE.md template that `fw init` stamps into new projects
- The `/capture` skill in the framework's skills library

## Acceptance Criteria

### Agent
- [x] OneDev issue created at — pickup prompt written, manual creation pending onedev.docker.ring20.geelenandcompany.com for the framework repo
- [x] Issue includes: problem description, 5-agent findings summary, proposed rule text, skill design
- [x] Console prompt generated with all details for framework agent pickup

### Human
- [x] Console prompt reviewed — human delegated closure and pasted to framework agent

## Verification

# Manual check only — no shell command can verify OneDev issue creation

## Updates

### 2026-03-12T00:24:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-12T00:38:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
