---
id: T-293
name: "Remediate all audit warnings — fabric drift, version pin, stale tasks, T-530 refs"
description: >
  Remediate all audit warnings — fabric drift, version pin, stale tasks, T-530 refs

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-26T12:37:39Z
last_update: 2026-03-26T12:37:39Z
date_finished: null
---

# T-293: Remediate all audit warnings — fabric drift, version pin, stale tasks, T-530 refs

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] All 9 stale fabric edges removed from `.fabric/components/*.yaml` cards
- [ ] `fw fabric drift` reports 0 stale edges


## Verification

fw fabric drift 2>&1 | grep -q "stale: 0"

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

### 2026-03-26T12:37:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-293-remediate-all-audit-warnings--fabric-dri.md
- **Context:** Initial task creation
