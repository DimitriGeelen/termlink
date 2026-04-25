---
id: T-1259
name: "Commit vendor framework upgrade 1.5.16→1.5.307 + preserve CLAUDE.md consumer path note"
description: >
  Commit vendor framework upgrade 1.5.16→1.5.307 + preserve CLAUDE.md consumer path note

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T18:03:23Z
last_update: 2026-04-25T18:07:14Z
date_finished: 2026-04-25T18:07:14Z
---

# T-1259: Commit vendor framework upgrade 1.5.16→1.5.307 + preserve CLAUDE.md consumer path note

## Context

`fw upgrade` ran at 16:19 (1.5.16→1.5.307), bringing in T-1461 Watchtower URL
render for handovers + reviewer/audit/context-script updates. PL-022 warned
this clobbers consumer-specific patches: CLAUDE.md's vendored-path note was
overwritten back to a generic line. Restore consumer wording, commit the
upgrade as a tracked unit.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md restored to consumer wording (`.agentic-framework/bin/fw`).
- [x] T-1255 G-007 fix verified intact in vendored handover.sh + audit.sh.
- [x] Stale active/ task files (T-1164, T-1253, T-1256, T-1257, T-1258) committed as deletions; counterparts present in completed/.
- [x] `.framework.yaml` records upgrade 1.5.16 → 1.5.307.
- [x] Single commit referencing T-1259.

## Verification

grep -q "agentic-framework/bin/fw" /opt/termlink/CLAUDE.md
grep -q "T-1255 (G-007)" /opt/termlink/.agentic-framework/agents/handover/handover.sh
grep -q "OneDev → GitHub mirror drift" /opt/termlink/.agentic-framework/agents/audit/audit.sh
grep -q "1.5.307" /opt/termlink/.framework.yaml
test -f /opt/termlink/.tasks/completed/T-1164-t-11557-migrate-filesendreceive--channel.md

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

### 2026-04-25T18:03:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1259-commit-vendor-framework-upgrade-15161530.md
- **Context:** Initial task creation

### 2026-04-25T18:07:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
