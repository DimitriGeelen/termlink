---
id: T-863
name: "Fix pre-push hook missing .agentic-framework audit path"
description: >
  Fix pre-push hook missing .agentic-framework audit path

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-04T20:43:14Z
last_update: 2026-04-04T20:43:14Z
date_finished: null
---

# T-863: Fix pre-push hook missing .agentic-framework audit path

## Context

Pre-push hook at `.git/hooks/pre-push` only checks `.framework.yaml -> framework_path` and `$PROJECT_ROOT/agents/audit/audit.sh` for the audit script. Since T-498 removed `.framework.yaml` and the project uses vendored framework at `.agentic-framework/`, it never finds the audit script and blocks all pushes.

## Acceptance Criteria

### Agent
- [ ] Pre-push hook checks `$PROJECT_ROOT/.agentic-framework/agents/audit/audit.sh` path
- [ ] `git push origin main` succeeds (8 unpushed commits reach OneDev)
- [ ] Error message lists all checked paths including `.agentic-framework/`

## Verification

grep -q '.agentic-framework/agents/audit/audit.sh' .git/hooks/pre-push

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

### 2026-04-04T20:43:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-863-fix-pre-push-hook-missing-agentic-framew.md
- **Context:** Initial task creation
