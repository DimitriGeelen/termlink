---
id: T-098
name: "OneDev issue + framework agent pickup prompt for /capture and Conversation Guard"
description: >
  Create a OneDev issue in the framework repo for the framework agent to pick up and
  implement: (1) Exploratory Conversation Guard rule for CLAUDE.md template, (2) /capture
  skill for the framework skills library. After creating the issue, generate a formatted
  console prompt that the human can copy-paste to the framework agent to pick up the work.
status: captured
workflow_type: build
owner: agent
horizon: now
tags: [framework, onedev, handoff]
components: []
related_tasks: [T-094, T-095, T-096]
created: 2026-03-11T11:30:00Z
last_update: 2026-03-11T11:30:00Z
date_finished: null
---

# T-098: OneDev Issue + Framework Agent Pickup Prompt

## Context

Spawned from T-094. The remediations in T-095/T-096 fix this specific project (termlink).
But the framework serves all consumer projects. The framework agent must implement:
- The Conversation Guard rule in the CLAUDE.md template that `fw init` stamps into new projects
- The `/capture` skill in the framework's skills library

## Acceptance Criteria

### Agent
- [ ] OneDev issue created at onedev.docker.ring20.geelenandcompany.com for the framework repo
- [ ] Issue includes: problem description, 5-agent findings summary, proposed rule text, skill design
- [ ] Console prompt generated with all details for framework agent pickup

### Human
- [ ] Console prompt reviewed and pasted to framework agent

## Verification

# Manual check only — no shell command can verify OneDev issue creation

## Updates
