---
id: T-825
name: "Fix pre-push hook — resolve audit.sh via .agentic-framework/ fallback"
description: >
  Fix pre-push hook — resolve audit.sh via .agentic-framework/ fallback

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:15:36Z
last_update: 2026-04-03T20:15:36Z
date_finished: null
---

# T-825: Fix pre-push hook — resolve audit.sh via .agentic-framework/ fallback

## Context

Pre-push hook blocks push because `framework_path:` was removed from `.framework.yaml` (T-498) and fallback only checks `$PROJECT_ROOT/agents/audit/audit.sh`. Audit script lives at `.agentic-framework/agents/audit/audit.sh`.

## Acceptance Criteria

### Agent
- [x] Pre-push hook resolves audit.sh via `.agentic-framework/agents/audit/audit.sh` fallback
- [x] `git push origin main` succeeds (8f29fce..6184b37)

## Verification

grep -q 'agentic-framework' .git/hooks/pre-push

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

### 2026-04-03T20:15:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-825-fix-pre-push-hook--resolve-auditsh-via-a.md
- **Context:** Initial task creation
