---
id: T-132
name: "Document SSH tunneling setup for cross-machine TermLink"
description: >
  Document SSH tunneling setup for cross-machine TermLink

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-14T15:17:12Z
last_update: 2026-03-14T15:18:25Z
date_finished: 2026-03-14T15:18:25Z
---

# T-132: Document SSH tunneling setup for cross-machine TermLink

## Context

Phase 0 from T-011 inception. SSH tunneling works today with zero code changes — document how.

## Acceptance Criteria

### Agent
- [x] SSH tunneling guide exists at `docs/ssh-tunneling.md`
- [x] Covers: single session forwarding, hub forwarding, socat bridge for TCP
- [x] Includes working SSH commands with TermLink-specific paths

## Verification

test -f docs/ssh-tunneling.md
grep -q 'ssh -L' docs/ssh-tunneling.md

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

### 2026-03-14T15:17:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-132-document-ssh-tunneling-setup-for-cross-m.md
- **Context:** Initial task creation

### 2026-03-14T15:18:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
