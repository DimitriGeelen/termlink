---
id: T-109
name: "Framework PR — /capture skill and JSONL transcript reader"
description: >
  Create a framework PR in OneDev to contribute the /capture skill and JSONL
  transcript reader to the agentic-engineering-framework. Includes research
  artifact, OneDev PR, and pickup prompt for the framework agent.
status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [capture, framework-pr, skill, conversation-capture]
components: []
related_tasks: [T-108, T-106]
created: 2026-03-12T00:00:00Z
last_update: 2026-03-19T17:52:29Z
date_finished: 2026-03-14T12:45:59Z
---

# T-109: Framework PR — /capture Skill and JSONL Transcript Reader

## Context

T-108 delivered the `/capture` skill and JSONL transcript reader, validated with
a real-world test (artifact at `docs/reports/T-108-capture-2026-03-11-23.md`).

This task contributes those artifacts to the agentic-engineering-framework via
a OneDev PR, following the framework PR handoff pattern established in T-106.

## What to Deliver

1. **Research artifact** — `docs/reports/T-109-capture-skill-framework-pr.md`
   - What /capture is, why it exists, what it delivers
   - Files to include in the PR
   - Integration notes (path assumptions, Python dependency, Claude Code only)

2. **OneDev PR** — Submit PR to framework repo with:
   - `agents/capture/read-transcript.py`
   - `.claude/commands/capture.md`
   - `.fabric/components/capture-reader.yaml`
   - `.fabric/components/capture-skill.yaml`

3. **Pickup prompt** — `docs/framework-agent-pickups/T-109-capture-skill.md`
   - Self-contained prompt for the framework agent to pick up and execute the PR

## Acceptance Criteria

### Agent
- [x] Research artifact created at `docs/reports/T-109-capture-skill-framework-pr.md`
- [x] Pickup prompt written to `docs/framework-agent-pickups/T-109-capture-skill-pr.md`
- [x] OneDev PR content drafted in pickup prompt (no API auth available — human to submit)
- [x] T-108 final AC checked and T-108 closed

### Human
- [x] [REVIEW] Pickup prompt reviewed and approved for framework agent submission

## Verification

test -f docs/reports/T-109-capture-skill-framework-pr.md
test -f docs/framework-agent-pickups/T-109-capture-skill-pr.md

## Updates

### 2026-03-12T16:10:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-14T12:42:59Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-14T12:45:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
