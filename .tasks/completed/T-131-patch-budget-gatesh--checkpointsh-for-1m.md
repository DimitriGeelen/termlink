---
id: T-131
name: "Patch budget-gate.sh + checkpoint.sh for 1M context window"
description: >
  Patch budget-gate.sh + checkpoint.sh for 1M context window

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-14T12:59:44Z
last_update: 2026-03-14T14:36:02Z
date_finished: 2026-03-14T14:36:02Z
---

# T-131: Patch budget-gate.sh + checkpoint.sh for 1M context window

## Context

Framework OneDev commit 2051588d updated budget thresholds for 1M context window (Opus 4.6 GA). Homebrew tap not yet released. Manual patch to local install.

## Acceptance Criteria

### Agent
- [x] budget-gate.sh uses CONTEXT_WINDOW env var (default 1M) instead of hardcoded 200K
- [x] checkpoint.sh uses CONTEXT_WINDOW env var (default 1M) instead of hardcoded 200K
- [x] Thresholds: warn 60%, urgent 80%, critical 90%
- [x] Transcript tail window increased 2MB to 10MB
- [x] No remaining hardcoded 200000 or 200K references
- [x] Both scripts pass syntax check
- [x] `checkpoint.sh status` reports correct percentages against 1M window

## Verification

bash -n /usr/local/opt/agentic-fw/libexec/agents/context/budget-gate.sh
bash -n /usr/local/opt/agentic-fw/libexec/agents/context/checkpoint.sh
! grep -q '200000' /usr/local/opt/agentic-fw/libexec/agents/context/budget-gate.sh
! grep -q '200000' /usr/local/opt/agentic-fw/libexec/agents/context/checkpoint.sh
grep -q 'CONTEXT_WINDOW' /usr/local/opt/agentic-fw/libexec/agents/context/budget-gate.sh
grep -q 'CONTEXT_WINDOW' /usr/local/opt/agentic-fw/libexec/agents/context/checkpoint.sh

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

### 2026-03-14T12:59:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-131-patch-budget-gatesh--checkpointsh-for-1m.md
- **Context:** Initial task creation

### 2026-03-14T14:36:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
