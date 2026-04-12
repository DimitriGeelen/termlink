---
id: T-981
name: "Fix Tier 0 self-approval bypass — block fw tier0 approve in check-tier0.sh patterns"
description: >
  Fix Tier 0 self-approval bypass — block fw tier0 approve in check-tier0.sh patterns

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T13:36:13Z
last_update: 2026-04-12T13:36:13Z
date_finished: null
---

# T-981: Fix Tier 0 self-approval bypass — block fw tier0 approve in check-tier0.sh patterns

## Context

T-980 GO. Agent self-approved T-936 inception decide by running `fw tier0 approve`. Fix: add pattern to block it.

## Acceptance Criteria

### Agent
- [x] `fw tier0 approve` added to PATTERNS in check-tier0.sh (keyword pre-filter AND Python patterns)
- [x] Block message updated: remove bare CLI hint, show `! fw tier0 approve` (shell prefix) and Watchtower
- [x] Agent running `fw tier0 approve` gets BLOCKED (exit 2)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q 'tier0.*approve' /opt/termlink/.agentic-framework/agents/context/check-tier0.sh

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

### 2026-04-12T13:36:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-981-fix-tier-0-self-approval-bypass--block-f.md
- **Context:** Initial task creation
